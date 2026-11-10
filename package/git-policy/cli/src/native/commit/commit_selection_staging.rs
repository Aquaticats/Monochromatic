//! What:
//!  The worktree paths an explicit-path commit stages into its private index:
//!  which selected elements name one file,
//!  which name a directory or a pattern,
//!  and the two `git add` starts that stage them.
//! Why:
//!  Native Git commits the tracked paths and the already-staged additions a directory or a pattern matches,
//! and never a file nothing staged,
//!  while an exact path the person named is taken as the wrapper's own deliberate difference.
//!  The file is split from `commit_index.rs`,
//! which initializes the index and stays inside its line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await git(['add', '--', ...exactFiles], { index: commitIndex }); await git(['add', '--update', '--', ...patterns], { index: commitIndex });
//! ```

use crate::commit_index_selection::{IndexSelection, selection_arguments};
use crate::commit_staged_additions::{explain_ignored_refusal, pathspecs_of_file};
use crate::private_index_request::{CaptureIndexes, caller_pathspec_request};
use crate::transaction_git::{GitContext, GitFailure, GitRequest, run_git_checked};
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// What:
///  Stages the named paths into the private index,
/// as Git's partial commit does:
/// what the worktree holds at them,
/// deletions
///  included.
/// Why:
///   Both ways of making the private index (reset to the base,
/// or the real index kept as it is) end here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['add', '--', ...exactFiles], { index: commitIndex }); await git(['add', '--update', '--', ...patterns], { index: commitIndex })
/// ```
pub(crate) fn add_selected_paths(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
    match selection_items(selection) {
        Selected::Items(items) => return stage_items(context, cwd, indexes, items.as_slice()),
        // An empty selection stages nothing, and the scratch commit then refuses as native Git refuses the command.
        Selected::Empty => return Ok(()),
        // A pathspec file too large to name on one command line keeps its own form, and only the tracked paths it matches
        // are staged, so no file nothing staged reaches the commit.
        Selected::File => {
            let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--update")];
            arguments.extend(selection_arguments(selection));
            return private_git_with_modes(context, cwd, indexes, arguments.as_slice()).map_err(explain_ignored_refusal);
        }
        // A pathspec file this reader cannot take is Git's to read, and every path it names is staged as before.
        Selected::Unreadable => {
            let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--all")];
            arguments.extend(selection_arguments(selection));
            return private_git_with_modes(context, cwd, indexes, arguments.as_slice()).map_err(explain_ignored_refusal);
        }
    }
}

/// What:
///  Stages the selection's elements into the private index:
/// an element that names one file as it is,
/// and a directory,
/// a pattern or pathspec magic as the tracked paths it matches.
/// Why:
///  Native Git commits the tracked and already-staged paths a directory or a pattern matches,
/// and never a file nothing staged,
///  while a `git add --all` over the same pathspec staged every untracked file it matched:
/// a commit of `.`,
///  `*` or a directory then published scratch files nobody staged.
///  An exact path the worktree holds is the one shape step 3 of `SPEC.md` "Explicit-path commit" takes from the worktree:
///  a path an ignore rule matches is still refused by Git with the hint the remedy explains,
/// and a file nothing staged is committed where native Git refuses the pathspec (the wrapper's deliberate difference).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['add', '--', ...exactFiles]); await git(['add', '--update', '--', ...patterns]);
/// ```
fn stage_items(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, items: &[OsString]) -> Result<(), GitFailure> {
    let literal: bool = pattern_characters_are_literal(context);
    let mut named_files: Vec<&OsString> = Vec::new();
    let mut patterns: Vec<&OsString> = Vec::new();
    for item in items {
        if names_one_file(cwd, item.as_os_str(), literal) {
            named_files.push(item);
        } else {
            patterns.push(item);
        }
    }
    if !named_files.is_empty() {
        let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--")];
        arguments.extend(named_files.into_iter().cloned());
        private_git_with_modes(context, cwd, indexes, arguments.as_slice()).map_err(explain_ignored_refusal)?;
    }
    // A directory or a pattern stages the tracked paths it matches (`--update`), never a file nothing staged. A pathspec
    // that matches no entry of the private index fails here as native Git fails it.
    if !patterns.is_empty() {
        let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--update"), OsString::from("--")];
        arguments.extend(patterns.into_iter().cloned());
        private_git_with_modes(context, cwd, indexes, arguments.as_slice()).map_err(explain_ignored_refusal)?;
    }
    return Ok(());
}

/// What:
///  What one selection gives this staging step:
/// its elements when they can be read,
/// the pathspec file itself when it is too large to name,
/// which is what an unreadable one leaves to Git,
/// and nothing for an empty selection.
/// Why:
///   Every shape stages differently,
/// and a reader that does not take a file must not change what Git does with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Selected = { kind: 'items', items: string[] } | { kind: 'file' } | { kind: 'unreadable' };
/// ```
pub(crate) enum Selected {
    /// The selection's elements,
    /// from the command line or from a pathspec file.
    Items(Vec<OsString>),
    /// A pathspec file with no element.
    Empty,
    /// A pathspec file too large to name on one command line.
    File,
    /// A pathspec file this reader cannot take.
    Unreadable,
}

/// The most bytes of pathspec elements this wrapper puts on one command line instead of naming the pathspec file.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// above it the file is passed as a file with `--update`,
/// so the commit still takes the tracked paths and only a file nothing staged stays out;
/// one argument may not exceed 128 KiB (`MAX_ARG_STRLEN`) and a whole Linux command line stays under 2 MiB (`ARG_MAX`),
/// so this bound leaves room for the rest of the command.
const ITEMS_BYTES_LIMIT: u64 = 64 * 1024;

/// What:
///  The elements one selection names,
/// and which form the staging step takes for them.
/// Why:
///   Telling an exact file name from a directory or a pattern needs the elements,
/// and a file's elements are the same selection a command line carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selected = pathspecFile ? readFile(pathspecFile).length <= LIMIT ? items(pathspecsOfFile(...)) : { kind: 'file' } : items(pathspecs);
/// ```
pub(crate) fn selection_items(selection: &IndexSelection) -> Selected {
    let Some(file) = selection.pathspec_file else {
        if selection.pathspecs.is_empty() {
            return Selected::Empty;
        }
        return Selected::Items(selection.pathspecs.to_vec());
    };
    let Ok(size) = std::fs::metadata(file).map(size_of) else { return Selected::Unreadable };
    if size > ITEMS_BYTES_LIMIT {
        return Selected::File;
    }
    let Ok(bytes) = std::fs::read(file) else { return Selected::Unreadable };
    let Some(items) = pathspecs_of_file(bytes.as_slice(), selection.pathspec_file_nul) else { return Selected::Unreadable };
    if items.is_empty() {
        return Selected::Empty;
    }
    return Selected::Items(items);
}

/// The length of a file's metadata,
/// for the bound on the bytes a pathspec file may name.
fn size_of(held: std::fs::Metadata) -> u64 {
    return held.len();
}

/// Whether one byte is a pattern character of the glob mode:
/// `*`,
/// `?` or `[`.
fn is_pattern_character(byte: &u8) -> bool {
    return matches!(*byte, b'*' | b'?' | b'[');
}

/// What:
///  Whether the pathspec modes in effect make a glob character ordinary text,
/// so `st*r.txt` names the file of that name.
/// Why:
///   The elements the person named decide both which paths this stage takes from the worktree and which of them an
/// ignore rule can refuse,
///  so both ask one owner rather than each reading the modes again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const literal = globCharactersAreLiteral(modesInEffect(globalPrefix, overlay));
/// ```
pub(crate) fn pattern_characters_are_literal(context: &GitContext) -> bool {
    return crate::conflict_side_modes::glob_characters_are_literal(
        crate::conflict_side_modes::modes_in_effect(context.global_prefix.as_slice(), context.overlay.as_slice()).as_slice(),
    );
}

/// What:
///  Whether one pathspec token names one file of the worktree rather than a directory or a pattern.
/// Why:
///   An exact name is the only shape step 3 of `SPEC.md` "Explicit-path commit" selects from the worktree,
/// and a directory or a pattern must take what native Git takes.
///  Pathspec magic selects a set and never one name,
/// and a glob character is a pattern character only in a mode that reads it as one:
///  `--literal-pathspecs` and `--noglob-pathspecs` make `st*r.txt` the file of that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesOneFile = (cwd, token, literal) => !token.startsWith(':') && (literal || !/[*?[]/.test(token)) && lstat(join(cwd, token)).isFile();
/// ```
pub(crate) fn names_one_file(cwd: &Path, token: &OsStr, literal: bool) -> bool {
    let bytes: &[u8] = token.as_encoded_bytes();
    if bytes.first() == Some(&b':') {
        return false;
    }
    // A backslash is Git's escape and not a pattern character here: `a\\b.txt` names the file of that name when one is
    // there, and a token whose literal pathname is not there is left to Git as the pattern it may be.
    if !literal && bytes.iter().any(is_pattern_character) {
        return false;
    }
    let Ok(held) = std::fs::symlink_metadata(cwd.join(token)) else { return false };
    let kind = held.file_type();
    return kind.is_file() || kind.is_symlink();
}

/// What:
///  `private_git` with the caller's own pathspec-mode options reaching Git.
/// Why:
///   The staging pathspecs are the caller's,
/// so the mode the caller wrote as a global option must reach them exactly as the mode they wrote as a variable does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runGit(callerPathspecRequest(context, cwd, arguments, commitIndex, objectDirectory))
/// ```
fn private_git_with_modes(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, arguments: &[OsString]) -> Result<(), GitFailure> {
    let request: GitRequest = caller_pathspec_request(
        context,
        cwd,
        arguments,
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    run_git_checked(context, &request)?;
    return Ok(());
}
