//! What:
//!  List every use of one module by another in a package,
//!  with a file and line that prove it.
//! Why:
//!  Both layout rules start from these uses:
//!  the folder rule groups them by folder,
//!  and the file rule keeps the ones inside a single folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ModuleUse = { from: number; to: number; file: Path; line: number; text: string };
//! function collectUses(tree: ModuleTree): ModuleUse[];
//! ```

/// Import the tree and the resolver.
use crate::{
    layout_module_tree::ModuleTree,
    layout_path_resolution::{resolve_path, resolve_use_path},
};
/// Import the native path type.
use std::path::{Path, PathBuf};

/// What:
///  One module naming another.
/// Why:
///  `text` is the path as written,
///  so a finding can quote the reference that proves the use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ModuleUse = { from: number; to: number; file: Path; line: number; text: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleUse {
    /// The module whose code names the other.
    pub from: usize,
    /// The module named.
    pub to: usize,
    /// The file of the naming code.
    pub file: PathBuf,
    /// One-based line of the path.
    pub line: usize,
    /// The path as written,
    ///  such as `crate::a::b::Item` or `super::x::*`.
    pub text: String,
}

/// What:
///  Every use of a module by a different module.
/// Why:
///  A path that names the module it is in is not a dependency.
///  A path that reaches an item through a re-export is a use of the module it names
///  and also of the module that defines the item,
///  because dropping the first would hide a loop through the re-exporting file
///  (measured on `package/git-policy/cli`:
///  replacing it let three files of a fifteen-file loop escape).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectUses(tree: ModuleTree): ModuleUse[];
/// ```
pub fn collect_uses(tree: &ModuleTree) -> Vec<ModuleUse> {
    let mut uses: Vec<ModuleUse> = Vec::<ModuleUse>::new();
    for (index, module) in tree.modules.iter().enumerate() {
        for reference in &module.references {
            let Some(named) = resolve_path(tree, index, reference.raw.segments.as_slice(), true)
            else {
                continue;
            };
            // A path that reaches an item through re-exports also uses the module that defines it.
            let defining: usize =
                resolve_use_path(tree, index, reference.raw.segments.as_slice()).unwrap_or(named);
            let mut text: String = reference.raw.segments.join("::");
            if reference.raw.glob {
                text.push_str("::*");
            }
            let mut targets: Vec<usize> = vec![named];
            if defining != named {
                targets.push(defining);
            }
            for target in targets {
                if target == index {
                    continue;
                }
                uses.push(ModuleUse {
                    from: index,
                    to: target,
                    file: module.file.clone(),
                    line: reference.line,
                    text: text.clone(),
                });
            }
        }
    }
    return uses;
}

/// What:
///  The folder that holds a module's file.
/// Why:
///  A module belongs to the folder of its file:
///  `a/mod.rs` and `a/b.rs` both belong to `a`,
///  and an inline module belongs to the folder of the file around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function folderOf(tree: ModuleTree, module: number): Path;
/// ```
pub fn folder_of(tree: &ModuleTree, module: usize) -> PathBuf {
    return match tree.modules[module].file.parent() {
        Some(folder) => folder.to_path_buf(),
        None => Path::new("").to_path_buf(),
    };
}

/// Controls of use collection stay outside release artifacts.
#[cfg(test)]
#[path = "layout_module_uses_tests.rs"]
mod tests;
