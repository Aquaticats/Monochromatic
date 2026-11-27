//! What:
//!  The modules a library root reaches,
//!  by the chain of `mod` declarations from the root,
//!  with the visibility each declaration has.
//! Why:
//!  Whether another crate can name an item is decided above the item,
//!  by the `pub mod` chain that reaches its file,
//!  and no single file shows that chain:
//!  a scan of the source tree reads it once here.
//! Gotcha:
//!  A `#[path = "..."]` declaration names a file whose module name is the declared one,
//!  not the file's own name,
//!  so a chain is read from declarations and never from file names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const modules = readTheTree(srcRoot); // Map<chain, { file, public, testOnly, children }>
//! ```

use crate::source_scan_tests::declaration::declarations;
use crate::source_scan_tests::tree::locate;
use std::collections::BTreeMap;
use std::path::Path;

/// What:
///  `pub(crate) struct ModuleOfTheTree` is one module:
///  its file,
///  whether every `mod` line from the root to here is exactly `pub mod`,
///  whether only tests compile it,
///  and its child modules with the chain each one has.
/// Why:
///  The surface scan asks these four facts of every item's module,
///  and a `pub use` path resolves through the children.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ModuleOfTheTree = { file: string; public: boolean; testOnly: boolean; children: Map<string, string> }
/// ```
pub(crate) struct ModuleOfTheTree {
    /// The file holding the module,
    /// relative to the source root.
    pub(crate) file: String,
    /// Whether every `mod` line from the root to here is exactly `pub mod`.
    pub(crate) public: bool,
    /// Whether a `cfg` on the way here compiles the module only when testing.
    pub(crate) test_only: bool,
    /// Each child's name and chain,
    /// whichever visibility its declaration has.
    pub(crate) children: BTreeMap<String, String>,
}

/// What:
///  `fn chain_of(parent: &str, name: &str) -> String` is the chain a child module has,
///  written `a::b`.
///  `&str` is a borrowed view of text (the sibling `String` is the owned one);
///  `String` outlives the line the name was cut from.
/// Why:
///  One spelling of a chain keys the map and is what a `pub use` path resolves against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const chainOf = (parent: string, name: string): string => (parent ? `${parent}::${name}` : name);
/// ```
fn chain_of(
    parent: &str,
    name: &str,
) -> String {
    if parent.is_empty() {
        return name.to_owned();
    }
    return format!("{parent}::{name}");
}

/// What:
///  `pub(crate) fn read_the_tree(root: &Path) -> Result<BTreeMap<String, ModuleOfTheTree>, String>` follows every
///  `mod` declaration from `lib.rs` and returns each module by its chain.
///  `BTreeMap` is a sorted map (as TS's `Map` iterated in key order),
///  and `Result<T, E>` is "answer or why not".
/// Why:
///  The reachability of one item is the reachability of every module above it,
///  so the chain is read once and every scan of the surface reads the same tree.
///  A declaration the resolver cannot place is an error,
///  because a guess here would decide the surface silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTheTree(root: string): Map<string, ModuleOfTheTree>  // throws Error
/// ```
pub(crate) fn read_the_tree(root: &Path) -> Result<BTreeMap<String, ModuleOfTheTree>, String> {
    let mut modules: BTreeMap<String, ModuleOfTheTree> = BTreeMap::new();
    let mut pending: Vec<(String, String, bool, bool, bool)> =
        vec![(String::from("lib.rs"), String::new(), true, false, true)];
    while let Some((file, chain, parent_public, test_only, owns_directory)) = pending.pop() {
        if modules.contains_key(chain.as_str()) {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.as_str()))
            .map_err(|error| return format!("cannot read {file}: {error}"))?;
        let declared =
            declarations(text.as_str()).map_err(|error| return format!("{file}: {error}"))?;
        let mut entry = ModuleOfTheTree {
            file: file.clone(),
            public: parent_public,
            test_only,
            children: BTreeMap::new(),
        };
        let mut children: Vec<(String, String, bool, bool, bool)> = Vec::new();
        for declaration in declared {
            let (child, child_owns): (String, bool) =
                locate(root, file.as_str(), owns_directory, &declaration)?;
            let child_chain: String = chain_of(chain.as_str(), declaration.name.as_str());
            entry
                .children
                .insert(declaration.name.clone(), child_chain.clone());
            children.push((
                child,
                child_chain,
                parent_public && declaration.public,
                test_only || declaration.test_only,
                child_owns,
            ));
        }
        modules.insert(chain, entry);
        for child in children {
            pending.push(child);
        }
    }
    return Ok(modules);
}

/// What:
///  `pub(crate) fn chain_the_target_reaches(modules, from, target) -> Option<String>` answers which module chain a
///  `pub use` path names,
///  started from the module `from` that holds the line.
/// Why:
///  `crate::name` starts at the root,
/// `self::name` at the module that writes the line,
/// `super::name` at its parent,
/// and a bare `name` at the module that writes the line.
///  A name at the crate root may also be a module another folder's glob re-export publishes,
///  so a root lookup that misses falls back to the modules of that name the tree holds.
/// Gotcha:
///  The answer is a chain whether or not `pub mod` lines reach it;
/// the caller decides whether that chain publishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chainTheTargetReaches(modules, from: string, target: string): string | undefined
/// ```
pub(crate) fn chain_the_target_reaches(
    modules: &BTreeMap<String, ModuleOfTheTree>,
    from: &str,
    target: &str,
) -> Option<String> {
    let segments: Vec<&str> = target
        .split("::")
        .filter(|segment| return !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return None;
    }
    let mut current: String = String::new();
    let mut index: usize = 0;
    match segments[0] {
        "crate" => index = 1,
        "self" => {
            current = from.to_owned();
            index = 1;
        }
        "super" => {
            let mut parent: &str = from;
            while index < segments.len() && segments[index] == "super" {
                parent = parent.rsplit_once("::").map_or("", |(head, _)| return head);
                index += 1;
            }
            current = parent.to_owned();
        }
        _ => {}
    }
    let mut first: bool = true;
    while index < segments.len() {
        let name: &str = segments[index];
        let step: Option<String> = modules
            .get(current.as_str())
            .and_then(|module| return module.children.get(name).cloned());
        let found: Option<String> = match step {
            Some(child) => Some(child),
            None if first && current.is_empty() => modules
                .iter()
                .find(|(chain, module)| {
                    let last: Option<&str> = chain.rsplit("::").next();
                    return module.public && last == Some(name);
                })
                .map(|(chain, _)| return chain.clone()),
            None => None,
        };
        // What: `?` hands back `None` when the step found no module, which makes the whole path unreachable.
        // Why: clippy refuses the `let ... else` spelling that does the same thing.
        let child: String = found?;
        current = child;
        first = false;
        index += 1;
    }
    return Some(current);
}
