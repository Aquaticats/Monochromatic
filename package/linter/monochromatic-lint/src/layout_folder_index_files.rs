//! What:
//!  Read the files of each folder of a package,
//!  find the index file of each folder,
//!  and check what the index files and the other executable roots hold.
//! Why:
//!  The file rule works folder by folder.
//!  Its index file is left out of the loops,
//!  so it must hold no code;
//!  executables beside it (a `main.rs` beside `lib.rs`,
//!  a single file under `src/bin`) are read for code like an entry point
//!  and are not listed either.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function filesByFolder(tree: ModuleTree): { byFolder: Map<Path, Set<Path>>; roots: Set<Path> };
//! function indexFile(folder: Path, files: Set<Path>, roots: Set<Path>): Path | undefined;
//! function indexFileFindings(index: Path | undefined, files: Set<Path>, roots: Set<Path>, base: Path): LayoutFinding[];
//! ```

/// Import the finding shape,
///  the code check of an index file and the tree.
use crate::{
    layout_finding::LayoutFinding,
    layout_index_file_items::{IndexRole, check_index_items},
    layout_module_tree::ModuleTree,
};
/// Import ordered collections and native paths.
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

/// What:
///  The name of a file without its folder.
/// Why:
///  The top doc lists bare file names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fileName(file: Path): string;
/// ```
pub(crate) fn file_name(file: &Path) -> String {
    return match file.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::new(),
    };
}

/// What:
///  The files of each folder,
///  and the files that are roots of a crate.
/// Why:
///  A file is the unit of the rule,
/// however many modules it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filesByFolder(tree: ModuleTree): { byFolder: Map<Path, Set<Path>>; roots: Set<Path> };
/// ```
pub(crate) fn files_by_folder(
    tree: &ModuleTree,
) -> (BTreeMap<PathBuf, BTreeSet<PathBuf>>, BTreeSet<PathBuf>) {
    let mut by_folder: BTreeMap<PathBuf, BTreeSet<PathBuf>> =
        BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
    for module in &tree.modules {
        if let Some(folder) = module.file.parent() {
            by_folder
                .entry(folder.to_path_buf())
                .or_default()
                .insert(module.file.clone());
        }
    }
    let mut roots: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for root in &tree.roots {
        roots.insert(tree.modules[*root].file.clone());
    }
    return (by_folder, roots);
}

/// What:
///  The file that holds a folder's top doc.
/// Why:
///  `mod.rs` for a module folder;
/// `lib.rs`,
/// else `main.rs`,
/// for the folder of a crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexFile(folder: Path, files: Set<Path>, roots: Set<Path>): Path | undefined;
/// ```
pub(crate) fn index_file(
    folder: &Path,
    files: &BTreeSet<PathBuf>,
    roots: &BTreeSet<PathBuf>,
) -> Option<PathBuf> {
    for name in ["mod.rs", "lib.rs", "main.rs"] {
        let candidate: PathBuf = folder.join(name);
        if files.contains(&candidate) && (name == "mod.rs" || roots.contains(&candidate)) {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
///  Whether a crate root that is not its folder's index is an entry point to read for code.
/// Why:
///  A `main.rs` beside a `lib.rs` and a single-file binary under `src/bin` are executables;
///  a root named by a `[[bin]]` path elsewhere is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isOtherEntryPoint(file: Path): boolean;
/// ```
fn is_other_entry_point(file: &Path) -> bool {
    if file_name(file) == "main.rs" {
        return true;
    }
    let Some(folder) = file.parent() else {
        return false;
    };
    return file_name(folder) == "bin";
}

/// What:
///  The findings about the code in a folder's index file and in the executables beside it.
/// Why:
///  An index that is a `main.rs` is an entry point;
///  every other index only declares and re-exports.
///  Executables beside the index are read like an entry point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexFileFindings(index: Path | undefined, files: Set<Path>, roots: Set<Path>, base: Path): LayoutFinding[];
/// ```
pub(crate) fn index_file_findings(
    index: Option<&PathBuf>,
    files: &BTreeSet<PathBuf>,
    roots: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    if let Some(index_path) = index {
        let role: IndexRole = if file_name(index_path) == "main.rs" {
            IndexRole::EntryPoint
        } else {
            IndexRole::Index
        };
        findings.extend(check_index_items(index_path, role, base));
    }
    for file in files {
        if roots.contains(file) && Some(file) != index && is_other_entry_point(file) {
            findings.extend(check_index_items(file, IndexRole::EntryPoint, base));
        }
    }
    return findings;
}
