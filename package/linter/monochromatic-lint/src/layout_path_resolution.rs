//! What:
//!  Turn a module path such as `crate::a::b::Item` into the module it names,
//!  using the real module tree.
//! Why:
//!  The folder a path points at is the folder of the deepest module the path names;
//!  the names after it are items (a function,
//! a type,
//! an enum variant) that live in that module.
//!  A module name is also found through `pub use path::*` re-exports of a module,
//!  because the crate-root glob lines of a flat layout (`pub use folder::*;`) are how
//!  `crate::module` reaches a module that a folder defines.
//!  The path of a glob itself is read through direct children only;
//!  that keeps the answer finite when globs refer to each other.
//!  An item that a module re-exports by name (`pub use sibling::Name;`,
//!  also under a new name with `as`) is followed to the module that defines it,
//!  through any chain of such re-exports;
//!  a use that reaches the item through the index then counts as a use of the defining file or folder.
//!  Known limit:
//!  an item (not a module) that a glob re-export supplies resolves to the module that re-exports it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function resolvePath(tree: ModuleTree, from: number, segments: string[], throughGlobs: boolean): number | undefined;
//! function resolveUsePath(tree: ModuleTree, from: number, segments: string[]): number | undefined;
//! function linkGlobTargets(tree: ModuleTree): void;
//! ```

/// Import the tree the paths are resolved in and the reference a re-export is read from.
use crate::{layout_module_tree::ModuleTree, layout_reference_paths::RawReference};
/// Import an ordered set for the visited modules.
use std::collections::BTreeSet;

/// What:
///  The module around all others in one root's tree.
/// Why:
///  `crate::` in any module names the root of the tree it is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootOf(tree: ModuleTree, module: number): number;
/// ```
fn root_of(tree: &ModuleTree, module: usize) -> usize {
    let mut current: usize = module;
    while let Some(parent) = tree.modules[current].parent {
        current = parent;
    }
    return current;
}

/// What:
///  Find the child module of that name,
///  directly or through the glob re-exports of the module.
/// Why:
///  A glob target is searched as a stack with a visited set,
///  so a ring of globs ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findChild(tree: ModuleTree, module: number, name: string, throughGlobs: boolean): number | undefined;
/// ```
fn find_child(tree: &ModuleTree, module: usize, name: &str, through_globs: bool) -> Option<usize> {
    if let Some(child) = tree.modules[module].children.get(name) {
        return Some(*child);
    }
    if !through_globs {
        return None;
    }
    let mut visited: BTreeSet<usize> = BTreeSet::<usize>::new();
    visited.insert(module);
    let mut stack: Vec<usize> = tree.modules[module].glob_targets.clone();
    while let Some(next) = stack.pop() {
        if !visited.insert(next) {
            continue;
        }
        if let Some(child) = tree.modules[next].children.get(name) {
            return Some(*child);
        }
        stack.extend(tree.modules[next].glob_targets.iter().copied());
    }
    return None;
}

/// What:
///  Walk a path through the tree:
///  the deepest module it names and the index of the first segment that is not a module.
/// Why:
///  `crate` starts at the root,
///  `super` and `self` at the module around or at the code,
///  and a plain name only when it is a module declared next to the code
///  (so `std::mem::swap` and an external crate's path name nothing).
///  The first segment that is not a module is the name of an item of the deepest module,
///  and a re-export may send that item to another file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walkPath(tree: ModuleTree, from: number, segments: string[], throughGlobs: boolean): { module: number; next: number } | undefined;
/// ```
fn walk_path(
    tree: &ModuleTree,
    from: usize,
    segments: &[String],
    through_globs: bool,
) -> Option<(usize, usize)> {
    let first: &String = segments.first()?;
    let mut current: usize = from;
    let mut index: usize = 0;
    match first.as_str() {
        "crate" => {
            current = root_of(tree, from);
            index = 1;
        }
        "super" | "self" => {}
        name => {
            find_child(tree, from, name, through_globs)?;
        }
    }
    while index < segments.len() {
        match segments[index].as_str() {
            "super" => current = tree.modules[current].parent?,
            "self" => {}
            name => match find_child(tree, current, name, through_globs) {
                Some(child) => current = child,
                None => break,
            },
        }
        index += 1;
    }
    return Some((current, index));
}

/// What:
///  The deepest module a path names,
///  or nothing when the path names no module of this package.
/// Why:
///  The names after the deepest module are items that live in it,
///  and this answer does not look at what the module re-exports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolvePath(tree: ModuleTree, from: number, segments: string[], throughGlobs: boolean): number | undefined;
/// ```
pub fn resolve_path(
    tree: &ModuleTree,
    from: usize,
    segments: &[String],
    through_globs: bool,
) -> Option<usize> {
    let (module, _): (usize, usize) = walk_path(tree, from, segments, through_globs)?;
    return Some(module);
}

/// What:
///  The name under which a re-export makes something visible:
///  its rename after `as`,
///  or else the last name of its path.
/// Why:
///  `pub use two::Thing as Y;` is reached as `Y`,
///  and `pub use two::Y;` as `Y`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exportedName(reference: RawReference): string | undefined;
/// ```
fn exported_name(reference: &RawReference) -> Option<&String> {
    if reference.alias.is_some() {
        return reference.alias.as_ref();
    }
    return reference.segments.last();
}

/// What:
///  The module that defines what a path names,
///  following every named re-export on the way.
/// Why:
///  `super::Name` from a file names an item of the folder's index,
///  but when the index only holds `pub use sibling::Name;` the file that is really used is `sibling`.
///  Each hop looks for a `pub use` that is not a glob and that exports the name,
///  and resolves that re-export's own path from the module that holds it.
///  A glob re-export,
///  a private `use` and a path outside the package end the walk where it stands.
///  A name is followed in each module once,
///  so a ring of re-exports ends and the use stays on the module where the walk met itself again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveUsePath(tree: ModuleTree, from: number, segments: string[]): number | undefined;
/// ```
pub fn resolve_use_path(tree: &ModuleTree, from: usize, segments: &[String]) -> Option<usize> {
    let (mut module, mut next): (usize, usize) = walk_path(tree, from, segments, true)?;
    let mut path: &[String] = segments;
    let mut followed: BTreeSet<(usize, &String)> = BTreeSet::<(usize, &String)>::new();
    while let Some(name) = path.get(next) {
        if !followed.insert((module, name)) {
            return Some(module);
        }
        let mut reexport: Option<&RawReference> = None;
        for reference in &tree.modules[module].references {
            if reference.raw.reexport
                && !reference.raw.glob
                && exported_name(&reference.raw) == Some(name)
            {
                reexport = Some(&reference.raw);
                break;
            }
        }
        let Some(found) = reexport else {
            return Some(module);
        };
        let Some((target, after)) = walk_path(tree, module, found.segments.as_slice(), true) else {
            return Some(module);
        };
        path = found.segments.as_slice();
        module = target;
        next = after;
    }
    return Some(module);
}

/// What:
///  Record,
/// for every module,
/// the modules it re-exports with `pub use path::*`.
/// Why:
///  Later lookups of a module name can pass through these re-exports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linkGlobTargets(tree: ModuleTree): void;
/// ```
pub fn link_glob_targets(tree: &mut ModuleTree) {
    let mut found: Vec<(usize, usize)> = Vec::<(usize, usize)>::new();
    for (index, module) in tree.modules.iter().enumerate() {
        for reference in &module.references {
            if !(reference.raw.glob && reference.raw.reexport) {
                continue;
            }
            if let Some(target) =
                resolve_path(tree, index, reference.raw.segments.as_slice(), false)
                && target != index
            {
                found.push((index, target));
            }
        }
    }
    for (index, target) in found {
        tree.modules[index].glob_targets.push(target);
    }
}

/// Controls of path resolution stay outside release artifacts.
#[cfg(test)]
#[path = "layout_path_resolution_tests.rs"]
mod tests;
