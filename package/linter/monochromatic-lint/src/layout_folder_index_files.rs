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

/// Import the words a `uses` line writes beside its folders,
///  the finding shape,
///  the code check of an index file and the tree.
use crate::{
    layout_allow_entries::{
        ARROW,
        FOLDER_USE_WORD,
        SET_OFF_ARROW,
    },
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
///  Why one name can never be written on a list line.
/// Why:
///  Each reason gives the refusal its own sentence,
///  so the message says which one the name earns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnlistableName = 'ends-in-trimmed-whitespace' | 'holds-a-line-break' | 'holds-a-backslash' | 'holds-the-arrow-words';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnlistableName {
    /// The name ends in a character the list reader trims away.
    EndsInTrimmedWhitespace,
    /// The name holds a line break, which ends a list line.
    HoldsLineBreak,
    /// The name holds a backslash, which a `uses` line writes as a folder separator.
    HoldsBackslash,
    /// The name would leave a `uses` line unable to say which folder is the one used.
    HoldsArrowWords,
}

/// What:
///  Whether a name can never be written on a list line,
///  and that name with every such character written as an escape.
/// Why:
///  A list line is read without the spaces and tabs around it,
///  so a key that ends in one can never be written on a line;
///  a module file named `x.rs ` or `x.rs<TAB>` would key findings no line could hold.
///  A key also never holds a newline,
///  because a list line ends at one,
///  so a name that holds a line break is refused for the same reason.
///  The escape spells the character a reader cannot see, so the message names the file exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistableNamePart(name: string): [string, UnlistableName] | undefined;
/// ```
pub(crate) fn unlistable_name_part(name: &str) -> Option<(String, UnlistableName)> {
    // What: the name with every line break spelled `\u{a}`.
    // Why: the message that names the file, and the key a line would hold, must each stay on one line.
    if name.contains('\n') {
        return Some((name.replace('\n', "\\u{a}"), UnlistableName::HoldsLineBreak));
    }
    // What: the name's last character, or nothing when the name is empty.
    // Why: only the last character can be the one the reader trims away.
    let last: char = name.chars().next_back()?;
    if !last.is_whitespace() {
        return None;
    }
    // What: the name with its last character spelled `\u{20}`, `\u{9}` or `\u{a0}`.
    // Why: the character itself is invisible in a terminal, so the message would not name the file.
    let spelled: String = format!(
        "{}\\u{{{:x}}}",
        &name[..name.len() - last.len_utf8()],
        last as u32
    );
    return Some((spelled, UnlistableName::EndsInTrimmedWhitespace));
}

/// What:
///  Whether a folder name can never be written on a list line,
///  and that name with such a character written as an escape.
/// Why:
///  A `uses` line writes a folder as a path,
///  and a path is written with `/` between its segments on every machine,
///  so a backslash inside a folder name would name another folder on a machine that separates folders with it;
///  this check is for the folders alone,
///  because a file name and an entry of a `uses` folder are no paths and their keys write them as they are.
///  A `uses` line is read from the two sides of the arrow it holds,
///  so a folder name that leaves the line unable to say where one folder ends is refused too,
///  while the caller that selects no rule reading a `uses` line passes `arrow_words_are_refused` false:
///  the file rule names such a folder in a key, and a key holds the name as written.
///  Everything a key cannot write of any name is refused here as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistableFolderNamePart(name: string, arrowWordsAreRefused: boolean): [string, UnlistableName] | undefined;
/// ```
pub(crate) fn unlistable_folder_name_part(
    name: &str,
    arrow_words_are_refused: bool,
) -> Option<(String, UnlistableName)> {
    if name.contains('\\') {
        return Some((String::from(name), UnlistableName::HoldsBackslash));
    }
    if arrow_words_are_refused && arrow_word_shape(name).is_some() {
        return Some((String::from(name), UnlistableName::HoldsArrowWords));
    }
    return unlistable_name_part(name);
}

/// What:
///  The one shape of the arrow words a folder name has,
///  or nothing when its name holds none of them.
/// Why:
///  The rule prints `uses <folder> -> <folder>`,
///  and the reader takes the first ` -> ` of that line as the arrow.
///  A name that closes another ` -> ` with the space the line sets its words off with
///  makes the reader split inside the name and read another pair of folders.
///  The line's own shape is the guard: a name is refused only when the line that would name it on the side before the arrow
///  really does hold a second ` -> `,
///  so the check cannot drift from what the printer writes and the reader reads.
///  The sentence that shape earns names exactly one cause, never a list of four,
///  because a reader who is told four reasons has to check which one is theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const arrowWordShape = (name: string) => {
///   if (!`uses ${name} `.includes(' -> ')) return undefined
///   if (name === '->') return 'is `->`'
///   if (name.startsWith('-> ')) return 'begins with `-> `'
///   if (name.endsWith(' ->')) return 'ends with ` ->`'
///   if (name.includes(' -> ')) return 'holds ` -> `'
///   return 'holds an arrow word'
/// }
/// ```
pub(crate) fn arrow_word_shape(name: &str) -> Option<&'static str> {
    // What: whether the line naming this folder before its arrow would hold a second arrow.
    // Why: the four shapes below are the ways a name does that, and this is the one test they must all satisfy.
    if !format!("{FOLDER_USE_WORD} {name} ").contains(SET_OFF_ARROW) {
        return None;
    }
    if name == ARROW {
        return Some("is `->`");
    }
    if name.starts_with("-> ") {
        return Some("begins with `-> `");
    }
    if name.ends_with(" ->") {
        return Some("ends with ` ->`");
    }
    if name.contains(SET_OFF_ARROW) {
        return Some("holds ` -> `");
    }
    // What: a name the line cannot be set off from in a way the four shapes above do not name.
    // Why: the guard above already refused it, so the run says the name holds an arrow word rather than letting it pass.
    return Some("holds an arrow word");
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
            "{} in the folder {} holds a space, so no `file-uses` line can name it: a use between two files is written as five words.",
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
