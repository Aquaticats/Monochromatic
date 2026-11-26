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
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
    },
    layout_index_file_items::{
        IndexRole,
        check_index_items,
    },
    layout_module_tree::ModuleTree,
};
/// Import ordered collections and native paths.
use std::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    path::{
        Path,
        PathBuf,
    },
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
///  Whether a name ends in a character the allow-list reader trims away,
///  and that name with the character written as an escape.
/// Why:
///  A list line is read without the spaces and tabs around it,
///  so a key that ends in one can never be written on a line;
///  a module file named `x.rs ` or `x.rs<TAB>` would key findings no line could hold.
///  The escape spells the character a reader cannot see, so the message names the file exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nameEndingInTrimmedWhitespace(name: string): string | undefined;
/// ```
pub(crate) fn name_ending_in_trimmed_whitespace(name: &str) -> Option<String> {
    // What: the name's last character, or nothing when the name is empty.
    // Why: only the last character can be the one the reader trims away.
    let last: char = name.chars().next_back()?;
    if !last.is_whitespace() {
        return None;
    }
    // What: the name with its last character spelled `\u{20}`, `\u{9}` or `\u{a0}`.
    // Why: the character itself is invisible in a terminal, so the message would not name the file.
    return Some(format!(
        "{}\\u{{{:x}}}",
        &name[..name.len() - last.len_utf8()],
        last as u32
    ));
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
    tree: &ModuleTree
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
///  A `mod.rs` is the folder's index when the module tree holds it,
///  and `lib.rs` and `main.rs` stay tied to the tree's roots,
///  because a file of that name beside a root is another executable and not an index.
///  The layout rules read what `mod` lines reach and nothing else,
///  so a `mod.rs` that no `mod` line declares is not its folder's index:
///  a leftover file moves no key, hides no code check of the folder's real index,
///  and makes no group of the folder around it.
///  The one such file that is read is the one that cannot be read at all:
///  the module tree found the folder's files without reading that index
///  (a `#[path]` declaration in another folder named them),
///  so the folder does hold an index and saying that it cannot be read,
///  with one missing entry per file it holds, is the honest answer
///  rather than a folder with no top doc at all.
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
    let candidate: PathBuf = folder.join("mod.rs");
    // What: the index the module tree reached, which decides the folder's keys and whether it is a group.
    // Why: a file no `mod` line declares is no part of the package, whatever its name suggests.
    if files.contains(&candidate) {
        return Some(candidate);
    }
    for name in ["lib.rs", "main.rs"] {
        let root: PathBuf = folder.join(name);
        if files.contains(&root) && roots.contains(&root) {
            return Some(root);
        }
    }
    // What: a `mod.rs` on disk that the tree never read, taken only when it cannot be read.
    // Why: the folder's files were declared from elsewhere, and a doc that cannot be read lists nothing.
    if candidate.is_file() && std::fs::read_to_string(&candidate).is_err() {
        return Some(candidate);
    }
    return None;
}

/// What:
///  Whether a module file's name holds a space.
/// Why:
///  A `file-uses` line separates its words by spaces,
///  so such a name cannot be written in one,
///  and a use through that file can never be tolerated.
///  A `#[path]` attribute can declare such a file, so the rule must answer it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nameHoldsSpace = (file: Path) => fileName(file).includes(' ');
/// ```
pub(crate) fn name_holds_space(file: &Path) -> bool {
    return file_name(file).contains(' ');
}

/// What:
///  The finding for a module file whose name holds a space.
/// Why:
///  A key line is read as its whole text,
///  so it names the file exactly, spaces and all,
///  and renaming the file kills the line rather than passing it to another file.
///  What such a name does forbid is a `file-uses` line,
///  which separates its words by spaces,
///  so a loop through this file is reported and cannot be tolerated:
///  the file is renamed, or the loop is untangled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spaceFinding(file: Path, folder: Path, base: Path): LayoutFinding;
/// ```
pub(crate) fn space_finding(
    file: &Path,
    folder: &Path,
    base: &Path,
) -> LayoutFinding {
    return LayoutFinding {
        file: file.to_path_buf(),
        line: 0,
        message: format!(
            "{} in the folder {} holds a space, so no `file-uses` line can name it: a use between two files is written as three words.",
            file_name(file),
            shown(folder, base)
        ),
        help: Some(String::from(
            "Rename the file so its name holds no space, and update the mod line or the `#[path]` attribute that declares it; until then a loop through this file cannot be tolerated.",
        )),
        key: format!(
            "space-in-file-name {} {}",
            key_path(folder, base),
            file_name(file)
        ),
    };
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
