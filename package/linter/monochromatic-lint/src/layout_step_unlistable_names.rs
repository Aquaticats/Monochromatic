//! What:
//!  The first name of a package that no allow-list line could hold:
//!  a module file,
//!  a folder of a module path,
//!  or an entry of a `uses` folder,
//!  with the reason it cannot be written.
//! Why:
//!  Every key about such a path holds the name,
//!  so no finding about it could be written down and no line could tolerate one,
//!  and the run says which name it stops at and what renaming must give.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function unlistablePath(paths: Path[], base: Path): [Path, string, UnlistableName] | undefined;
//! function unlistableName(tree: ModuleTree, base: Path): [Path, string, UnlistableName] | undefined;
//! function unlistableLinkName(tree: ModuleTree, base: Path): [Path, string, UnlistableName] | undefined;
//! ```

/// Import the key's own spelling of a path,
///  the checks a name has to pass,
///  the tree and the folder of a module,
///  and the name a `uses` folder goes by.
use crate::{
    layout_finding::key_names,
    layout_folder_index_files::{
        UnlistableName,
        unlistable_folder_name_part,
        unlistable_name_part,
    },
    layout_link_entries::USES_FOLDER,
    layout_module_tree::ModuleTree,
    layout_module_uses::folder_of,
};
/// Import ordered collections and native paths.
use std::{
    collections::BTreeSet,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  The first path whose name no list line could hold,
///  with that path and the name written with the character escaped.
/// Why:
///  Every key about such a path holds the name,
///  and a line that ends in a trimmed character reads as if the character were not there,
///  while a line ends at a newline,
///  a `uses` line writes a folder as a path with `/` between its segments,
///  so a folder name that holds a backslash would name another folder on a machine that separates folders with it,
///  and a folder name that ends or begins with the words a `uses` line sets its arrow off with,
///  or that is the arrow alone,
///  leaves that line unable to say which of its two folders is the one used,
///  so no finding about it could be written down;
///  a run over such a tree would report findings no list can hold.
///  The caller passes the paths the keys are built from,
///  so the parts checked are exactly those parts,
///  and the last part of each is a name rather than a folder,
///  because a file name and an entry of a `uses` folder are no paths:
///  their keys write them as the source does, backslash and arrow words included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistablePath(paths: Path[], base: Path): [Path, string, UnlistableName] | undefined;
/// ```
fn unlistable_path(
    paths: &[PathBuf],
    base: &Path,
) -> Option<(PathBuf, String, UnlistableName)> {
    for path in paths {
        let mut parts: Vec<String> = Vec::<String>::new();
        let mut unlistable: Option<UnlistableName> = None;
        // What: each name a key writes, as the file system spells it.
        // Why: a key writes a name's own backslash as written, so the parts of the written path cannot show one.
        let names: Vec<String> = key_names(path, base);
        let last: usize = names.len().saturating_sub(1);
        for (index, part) in names.iter().enumerate() {
            // What: whether this part is a folder, which a `uses` line must name as a path, or the name the path ends in.
            // Why: only a folder name can leave a `uses` line unable to say where one folder ends.
            let folder: bool = index != last;
            let checked: Option<(String, UnlistableName)> = if folder {
                unlistable_folder_name_part(part.as_str())
            } else {
                unlistable_name_part(part.as_str())
            };
            match checked {
                Some((escaped, reason)) => {
                    if unlistable.is_none() {
                        unlistable = Some(reason);
                    }
                    parts.push(escaped);
                }
                None => parts.push(part.clone()),
            }
        }
        if let Some(reason) = unlistable {
            return Some((path.clone(), parts.join("/"), reason));
        }
    }
    return None;
}

/// What:
///  The first module file of a package whose name no list line could hold.
/// Why:
///  The walk of the module tree names the files and folders every key is built from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistableName(tree: ModuleTree, base: Path): [Path, string, UnlistableName] | undefined;
/// ```
pub(crate) fn unlistable_name(
    tree: &ModuleTree,
    base: &Path,
) -> Option<(PathBuf, String, UnlistableName)> {
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for module in &tree.modules {
        paths.push(module.file.clone());
    }
    return unlistable_path(paths.as_slice(), base);
}

/// What:
///  The first entry of a `uses` folder of the package whose name no list line could hold.
/// Why:
///  A link's key holds the entry's own name,
///  so an entry named with a trailing space or a line break keys a finding no line could hold,
///  although no module file of the package carries that name.
///  An entry is no path and never appears on a `uses` line,
///  so its key writes its name as the source does:
///  a backslash or the arrow words are no reason to refuse it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistableLinkName(tree: ModuleTree, base: Path): [Path, string, UnlistableName] | undefined;
/// ```
pub(crate) fn unlistable_link_name(
    tree: &ModuleTree,
    base: &Path,
) -> Option<(PathBuf, String, UnlistableName)> {
    let mut folders: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for index in 0..tree.modules.len() {
        folders.insert(folder_of(tree, index));
    }
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for folder in &folders {
        let uses: PathBuf = folder.join(USES_FOLDER);
        let Ok(listing) = std::fs::read_dir(&uses) else {
            continue;
        };
        for entry in listing.flatten() {
            paths.push(entry.path());
        }
    }
    paths.sort();
    // What: an entry of a `uses` folder is a name and no path, so only the character the reader trims and a line break forbid it.
    // Why: its key writes the entry as written, and no `uses` line names an entry.
    return unlistable_path(paths.as_slice(), base);
}
