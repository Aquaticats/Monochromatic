//! What:
//!  The named files of an explicit-path commit that Git will not add because an ignore rule matches them,
//! and the finding that names them and both ways forward.
//! Why:
//!  An explicit pathspec selects worktree content,
//! and `git add` refuses a path an ignore rule matches while the index does not track it,
//! so the private staging of a commit that names such a path ends the whole commit.
//!  That is an input refusal of the command line,
//! not an engine or transaction failure,
//! so it is decided here from the selection's own named elements before any `git add` runs,
//! and it is reported as a finding with its own code (issue 6278).
//!  A path the real index tracks is not refused:
//! the private index carries that entry (`commit_staged_additions.rs`),
//! and `git add` then refreshes it without asking an ignore rule (issue 1807).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! git check-ignore -z --stdin // the elements that name one file
//! ```

use crate::commit_index_selection::IndexSelection;
use crate::commit_selection_staging::{Selected, names_one_file, pattern_characters_are_literal, selection_items};
use crate::path_bytes::path_message_text;
use crate::private_index_request::indexed_request;
use crate::private_storage::{ReadRefusal, read_regular_file};
use crate::repo_read_index::{IndexEntry, parse_index};
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, command_text, run_git};
use std::ffi::OsString;
use std::path::Path;

/// The core of the finding:
///  the fixed behavior of an explicit-path commit's selection,
/// which no configuration can turn off.
pub const NAMED_IGNORED_CORE_ID: &str = "explicit-path-commit";

/// The code of the finding,
/// reported as `explicit-path-commit/named-path-ignored`.
pub const NAMED_IGNORED_CODE: &str = "named-path-ignored";

/// What:
///  The selected paths that name one file of the worktree and that Git will not add,
/// in the order the selection names them.
/// Why:
///   Only these paths make the private staging refuse;
/// nothing here writes an index,
/// an object or a worktree file,
///  and the answer comes from the selection's own elements rather than from the words of a Git message.
///  A path the real index holds no entry at is the only kind `git add` can refuse this way,
/// so those are the only ones asked about,
///  and a commit whose named paths the index holds starts no Git at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = elements.filter(namesOneFile); const untracked = named.filter((path) => !indexHolds(path)); return untracked.length ? checkIgnore(untracked) : [];
/// ```
pub fn named_ignored_files(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    selection: &IndexSelection,
) -> Result<Vec<Vec<u8>>, GitFailure> {
    // A selection with no element this reader takes stages nothing under a name of its own: `Selected::File` and
    // `Selected::Unreadable` hand the pathspec file to Git, so the elements that could be refused are not known here, and
    // that staging decides for itself (`add_selected_paths`).
    let Selected::Items(items) = selection_items(selection) else {
        return Ok(Vec::new());
    };
    let literal: bool = pattern_characters_are_literal(context);
    let mut named: Vec<&OsString> = Vec::new();
    for item in items.iter() {
        if names_one_file(cwd, item.as_os_str(), literal) {
            named.push(item);
        }
    }
    if named.is_empty() {
        return Ok(Vec::new());
    }
    // The reader that declines the index leaves every name to Git, which is the answer that was right before this saving.
    let untracked: Vec<&OsString> = match untracked_among(context, cwd, real_index, named.as_slice()) {
        Some(found) => found,
        None => named,
    };
    if untracked.is_empty() {
        return Ok(Vec::new());
    }
    return ignored_among(context, cwd, real_index, untracked.as_slice());
}

/// The largest index file this reads.
///  The same bound the stat refresh of a private index reads its file with
/// (`INDEX_LIMIT` in `commit_index_stat_staleness.rs`),
/// classed once in `doc/planning/cli-git-shipped-limits-audit.md`.
const INDEX_LIMIT_BYTES: u64 = 1 << 30;

/// What:
///  The paths of `named` the real index holds no entry at,
/// and `None` when the in-process reader cannot say.
/// Why:
///   `git add` refuses a named path as ignored only while the index holds no entry at it,
/// so a path the index tracks cannot reach that refusal:
/// the private index carries the entry first (`commit_staged_additions.rs`),
///  and `git add` then refreshes it without asking any ignore rule (issue 1807).
///  Reading the index in process is what makes a commit of tracked paths start no Git at all,
/// which the release gate's budget counts (`binary_budget_tests.rs`).
///  Every reason to decline answers with `None`,
/// because a name left to Git costs one start,
///  where a name wrongly believed tracked would let the refusal return as an engine failure.
///  The spellings are the selection's,
/// so a token Git reads as a path relative to the top level (`/a.txt`)
///  is measured as the directory path it names and is then left to Git,
/// which answers for it correctly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayoutIn(cwd, overlay); const entries = parseIndex(readRegularFile(realIndex), layout.objectFormat);
/// return named.filter((path) => !entries.some((entry) => entry.path === layout.prefix + path));
/// ```
fn untracked_among<'named>(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    named: &[&'named OsString],
) -> Option<Vec<&'named OsString>> {
    let overlay: Vec<(OsString, OsString)> = crate::git_system_config::with_real_git(context.overlay.as_slice(), context.real_git.as_path());
    let layout: crate::repo_read_layout::RepoLayout = crate::repo_read_layout::read_layout_in(cwd, overlay.as_slice())?;
    // A repository nothing was ever staged in has no index, so no entry tracks any name and every name may be refused.
    let entries: Vec<IndexEntry> = match read_regular_file(real_index, INDEX_LIMIT_BYTES) {
        Ok(bytes) => parse_index(bytes.as_slice(), layout.object_format)?,
        // An index this reading cannot open is one Git answers for.
        Err(ReadRefusal::Missing) => Vec::new(),
        Err(_) => return None,
    };
    let mut untracked: Vec<&'named OsString> = Vec::new();
    for name in named {
        let spelled: Vec<u8> = top_relative(layout.prefix.as_slice(), name.as_encoded_bytes());
        // The reader refuses an index whose entries are out of order, so a miss here is a path no entry holds.
        if entries.binary_search_by_key(&spelled.as_slice(), held_path).is_err() {
            untracked.push(*name);
        }
    }
    return Some(untracked);
}

/// What:
///  The path one index entry holds,
/// the key the binary search over the entries compares.
/// Why:
///   The entries are in path order,
/// so asking whether one holds a path is a search and not a walk,
///  and the search takes the key as a function that captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heldPath = (entry) => entry.path;
/// ```
fn held_path(entry: &IndexEntry) -> &[u8] {
    return entry.path.as_slice();
}

/// What:
///  One selection element as the index spells it:
/// the directory prefix the command runs below,
/// then the element.
/// Why:
///   Index paths are relative to the top of the worktree,
///  while an element is read from the directory the command runs in,
/// and the crate maps one to the other the same way where a private index
///  asks which entries a selection restages (`restaged_paths` in `commit_index_stat_staleness.rs`).
///  A miss only costs one `check-ignore` start,
/// so a spelling this mapping does not resolve needs no second rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const topRelative = (prefix, spelled) => prefix + spelled;
/// ```
fn top_relative(prefix: &[u8], spelled: &[u8]) -> Vec<u8> {
    let mut path: Vec<u8> = Vec::with_capacity(prefix.len() + spelled.len());
    path.extend_from_slice(prefix);
    path.extend_from_slice(spelled);
    return path;
}

/// What:
///  The paths of `named` that an ignore rule matches and that the index does not track,
/// asked of Git in one start.
/// Why:
///   `git check-ignore` consults the index,
/// so a path the real index tracks is left out here exactly as `git add` leaves it out;
///  `--no-index` would answer for the ignore rules alone and refuse a path the commit can take.
///  Exit status 1 is Git's "no path matched",
/// and every other failure is fatal,
///  because a question this wrapper cannot get an answer to is not a refusal to report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(['check-ignore', '-z', '--stdin'], { index: realIndex, input: named.join('\0') })
/// ```
fn ignored_among(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    named: &[&OsString],
) -> Result<Vec<Vec<u8>>, GitFailure> {
    let mut input: Vec<u8> = Vec::new();
    for name in named {
        input.extend_from_slice(name.as_encoded_bytes());
        // `-z` reads and writes NUL-terminated names, so a name that holds a newline needs no quoting.
        input.push(0);
    }
    let request: GitRequest = refused_paths_request(context, cwd, real_index, input.as_slice());
    let output: GitOutput = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(GitFailure(format!(
                "{} could not start: {error}",
                command_text(request.arguments.as_slice())
            )));
        }
    };
    match output.code {
        Some(0) => {}
        Some(1) => return Ok(Vec::new()),
        _ => {
            return Err(GitFailure(format!(
                "{} failed: {}",
                command_text(request.arguments.as_slice()),
                output.error_text()
            )));
        }
    }
    let mut ignored: Vec<Vec<u8>> = Vec::new();
    for path in output.stdout.split(is_nul_byte) {
        if path.is_empty() {
            continue;
        }
        ignored.push(path.to_vec());
    }
    return Ok(ignored);
}

/// What:
///  The one `git check-ignore` start that answers which of `paths` an ignore rule refuses,
/// `paths` being the NUL-terminated names to ask about.
/// Why:
///   Four of this request's fields decide the answer,
/// so each has one owner here rather than at the start:
/// the real index as `GIT_INDEX_FILE`,
///  because the answer counts only where `git add` would refuse the path;
/// the caller's repository selection,
///  because the command may name a repository its directory does not hold
/// (`repository_selection`);
/// the wrapper's own pathspecs,
///  because Git refuses a pathspec mode for this command,
/// answering that the magic is unsupported there when the mode is inherited;
/// and the names on standard input,
///  so a name that holds any byte needs no quoting.
///  The object directory stays unset,
/// because this command writes no object,
/// and the private store belongs to the staging that follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inTheCallersRepository({ cwd, prefix }) // { index: realIndex, input, args: ['check-ignore', '-z', '--stdin'] }
/// ```
fn refused_paths_request(context: &GitContext, cwd: &Path, real_index: &Path, paths: &[u8]) -> GitRequest {
    let arguments: Vec<OsString> = ["check-ignore", "-z", "--stdin"].iter().map(OsString::from).collect();
    let mut request: GitRequest = indexed_request(context, cwd, arguments.as_slice(), Some(real_index), None).own_pathspecs();
    request.input = Some(paths.to_vec());
    return request;
}

/// What:
///  Whether one byte is the NUL byte that ends each name of a `-z` listing.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The message of the finding:
/// the rejected paths,
/// what happened to the commit,
/// and both legitimate ways forward.
/// Why:
///  Leaving a generated file ignored and tracking a file on purpose are both correct answers,
/// and the diagnostic must not pick one;
///  the force-add belongs only to the second,
/// and the commit that follows it records the worktree content as native Git does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ... so this commit was not made and nothing was staged by it: ${listed}. ...`
/// ```
pub fn named_ignored_message(paths: &[Vec<u8>]) -> String {
    let mut names: Vec<String> = Vec::new();
    for path in paths {
        names.push(path_message_text(path.as_slice()));
    }
    let listed: String = serde_json::to_string(&names).unwrap_or_default();
    return format!(
        "monochromatic-git-policy-cli: git commit with explicit paths selects worktree content, and git add refuses a \
path an ignore rule matches until the index tracks it, so this commit was not made and nothing was staged by it: \
{listed}. Choose explicitly: leave the ignored file out of the commit's paths and keep it untracked, or, when the file \
is meant to become tracked, stage it with git add --force -- <path> and run the same commit again; the commit then \
records the worktree content, as native Git does."
    );
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_named_ignored_files_tests.rs"]
mod tests;
