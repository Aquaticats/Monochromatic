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
        arrow_word_shape,
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
///  One name of a package that no list line could hold:
///  the file to point the finding at,
///  the name to spell in its sentence,
///  whether that name is a folder's,
///  why it cannot be written,
///  and, for the arrow words, the one shape it has.
/// Why:
///  A finding points at a file and names a cause,
///  and the cause is a name in the path rather than the file itself:
///  a folder ending in the arrow makes a module file inside it unwritable,
///  so a sentence naming that file sends its reader to a name that is not the one to change.
///  The shape says which of the arrow shapes the folder's name has,
///  because a reader told four alternatives has to work out which one is theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnlistableFound = { file: Path; written: string; kind: UnlistableKind; reason: UnlistableName; shape?: string };
/// ```
#[derive(Clone, Debug)]
pub(crate) struct UnlistableFound {
    /// The file whose key the name would appear in.
    pub(crate) file: PathBuf,
    /// The path of the offending name, written from the package folder.
    pub(crate) written: String,
    /// Which of the package's names this is, so the sentence says what to rename.
    pub(crate) kind: UnlistableKind,
    /// Why no list line could hold the name.
    pub(crate) reason: UnlistableName,
    /// The one shape of the arrow words, when that is the reason.
    pub(crate) shape: Option<&'static str>,
}

/// What:
///  Which of a package's names a refusal points at.
/// Why:
///  The sentence that follows says what to rename,
///  and a folder, a module file and an entry of a `uses` folder are three different things to go and look at:
///  a reader told to rename "the file" for an entry of a `uses` folder looks for a module file that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnlistableKind = 'folder' | 'file' | 'link-entry';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnlistableKind {
    /// A folder of a module path.
    Folder,
    /// The module file a path ends in.
    File,
    /// An entry of a `uses` folder, which is a link or a name that should be one.
    LinkEntry,
}

/// What:
///  The first path whose name no list line could hold,
///  with the file it belongs to and the name written with the character escaped.
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
///  The arrow words are refused only when a rule that reads a `uses` line is selected for the package:
///  the file rule reads no such line, and it names such a folder in a key, which holds the name as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistablePath(paths: Path[], base: Path, arrowWordsAreRefused: boolean): UnlistableFound | undefined;
/// ```
fn unlistable_path(
    paths: &[PathBuf],
    base: &Path,
    arrow_words_are_refused: bool,
) -> Option<UnlistableFound> {
    for path in paths {
        let mut parts: Vec<String> = Vec::<String>::new();
        let mut unlistable: Option<(usize, UnlistableName)> = None;
        // What: each name a key writes, as the file system spells it.
        // Why: a key writes a name's own backslash as written, so the parts of the written path cannot show one.
        let names: Vec<String> = key_names(path, base);
        let last: usize = names.len().saturating_sub(1);
        for (index, part) in names.iter().enumerate() {
            // What: whether this part is a folder, which a `uses` line must name as a path, or the name the path ends in.
            // Why: only a folder name can leave a `uses` line unable to say where one folder ends.
            let folder: bool = index != last;
            let checked: Option<(String, UnlistableName)> = if folder {
                unlistable_folder_name_part(part.as_str(), arrow_words_are_refused)
            } else {
                unlistable_name_part(part.as_str())
            };
            match checked {
                Some((escaped, reason)) => {
                    if unlistable.is_none() {
                        unlistable = Some((index, reason));
                    }
                    parts.push(escaped);
                }
                None => parts.push(part.clone()),
            }
        }
        let Some((index, reason)) = unlistable else {
            continue;
        };
        // What: which name to change, which is a folder of the path or the name the path ends in.
        // Why: the sentence that follows says which one to rename.
        let kind: UnlistableKind = if index == last {
            UnlistableKind::File
        } else {
            UnlistableKind::Folder
        };
        // What: the one shape of the arrow words, which only a folder name can have.
        // Why: the arrow case spells a single cause, so the sentence must not list four.
        let shape: Option<&'static str> = match reason {
            UnlistableName::HoldsArrowWords => arrow_word_shape(names[index].as_str()),
            UnlistableName::EndsInTrimmedWhitespace
            | UnlistableName::HoldsLineBreak
            | UnlistableName::HoldsBackslash => None,
        };
        return Some(UnlistableFound {
            file: path.clone(),
            written: parts[..=index].join("/"),
            kind,
            reason,
            shape,
        });
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
/// function unlistableName(tree: ModuleTree, base: Path, arrowWordsAreRefused: boolean): UnlistableFound | undefined;
/// ```
pub(crate) fn unlistable_name(
    tree: &ModuleTree,
    base: &Path,
    arrow_words_are_refused: bool,
) -> Option<UnlistableFound> {
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for module in &tree.modules {
        paths.push(module.file.clone());
    }
    return unlistable_path(paths.as_slice(), base, arrow_words_are_refused);
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
///  The arrow words of a folder above it are refused only when a rule reading a `uses` line is selected,
///  which is the caller's answer to pass through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unlistableLinkName(tree: ModuleTree, base: Path, arrowWordsAreRefused: boolean): UnlistableFound | undefined;
/// ```
pub(crate) fn unlistable_link_name(
    tree: &ModuleTree,
    base: &Path,
    arrow_words_are_refused: bool,
) -> Option<UnlistableFound> {
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
    // What: the first name of that folder no list line could hold.
    // Why: its key writes the entry as written, and no `uses` line names an entry.
    let mut found: Option<UnlistableFound> =
        unlistable_path(paths.as_slice(), base, arrow_words_are_refused);
    // What: that name marked as an entry of a `uses` folder, which the last part of such a path always is.
    // Why: the sentence says what to rename, and this name is a link rather than a module file.
    if let Some(item) = found.as_mut()
        && item.kind == UnlistableKind::File
    {
        item.kind = UnlistableKind::LinkEntry;
    }
    return found;
}
