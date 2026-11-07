//! What:
//!  Find the group folders of a package and the entry each folder belongs to.
//! Why:
//!  The group checks and the file rule need the same answer to "is this folder a group":
//!  a folder whose index is a `mod.rs` and that has a source folder directly inside it.
//!  The folder of a crate root is where the entries sit and is never a group.
//!  An **entry** is a group,
//!  or a source folder outside every group,
//!  and a folder inside a group has that group as its entry.
//!  The rule is one level of groups,
//!  so a group inside a group counts for the group around it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function findGroups(tree: ModuleTree): Set<Path>;
//! function ownerOf(folder: Path, groups: Set<Path>): Path | undefined;
//! function entryOf(folder: Path, groups: Set<Path>): Path;
//! ```

/// Import the file and index readers and the tree.
use crate::{
    layout_folder_index_files::{file_name, files_by_folder, index_file},
    layout_module_tree::ModuleTree,
};
/// Import ordered collections and native paths.
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

/// What:
///  The group folders of a package.
/// Why:
///  A group is a folder whose index is a `mod.rs` and that has a source folder directly inside it;
///  the folder of a crate root is where the entries sit and is never a group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findGroups(tree: ModuleTree): Set<Path>;
/// ```
pub fn find_groups(tree: &ModuleTree) -> BTreeSet<PathBuf> {
    let (by_folder, roots) = files_by_folder(tree);
    let mut parents: BTreeSet<&Path> = BTreeSet::<&Path>::new();
    for folder in by_folder.keys() {
        if let Some(parent) = folder.parent() {
            parents.insert(parent);
        }
    }
    let mut groups: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for (folder, files) in &by_folder {
        let Some(index) = index_file(folder, files, &roots) else {
            continue;
        };
        if file_name(&index) == "mod.rs" && parents.contains(folder.as_path()) {
            groups.insert(folder.clone());
        }
    }
    return groups;
}

/// What:
///  The outermost group above a folder,
///  not counting the folder itself.
/// Why:
///  Every folder inside a group counts for that group's summary,
///  and the rule has one level of groups,
///  so a nested group counts for the group around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerOf(folder: Path, groups: Set<Path>): Path | undefined;
/// ```
pub fn owner_of(folder: &Path, groups: &BTreeSet<PathBuf>) -> Option<PathBuf> {
    let mut outermost: Option<PathBuf> = None;
    for ancestor in folder.ancestors().skip(1) {
        if groups.contains(ancestor) {
            outermost = Some(ancestor.to_path_buf());
        }
    }
    return outermost;
}

/// What:
///  The entry a folder belongs to:
///  its outermost group,
///  or the folder itself outside every group.
/// Why:
///  The groups and the one-folder entries are the nodes of the loop check and of the summaries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryOf(folder: Path, groups: Set<Path>): Path;
/// ```
pub fn entry_of(folder: &Path, groups: &BTreeSet<PathBuf>) -> PathBuf {
    return match owner_of(folder, groups) {
        Some(group) => group,
        None => folder.to_path_buf(),
    };
}
