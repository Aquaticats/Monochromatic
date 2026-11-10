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
/// nothing here writes an index, an object or a worktree file,
///  and the answer comes from the selection's own elements rather than from the words of a Git message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = elements.filter(namesOneFile); return checkIgnore(named);
/// ```
pub fn named_ignored_files(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    selection: &IndexSelection,
) -> Result<Vec<Vec<u8>>, GitFailure> {
    // A selection with no element this reader takes stages nothing from the worktree by a name, so nothing can be refused
    // this way (`Selected::File` and `Selected::Unreadable` stage with `--update` or `--all`, which add no ignored path).
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
    return ignored_among(context, cwd, real_index, named.as_slice());
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
    let arguments: Vec<OsString> = ["check-ignore", "-z", "--stdin"].iter().map(OsString::from).collect();
    let mut request: GitRequest = indexed_request(cwd, arguments.as_slice(), Some(real_index), None);
    let mut input: Vec<u8> = Vec::new();
    for name in named {
        input.extend_from_slice(name.as_encoded_bytes());
        // `-z` reads and writes NUL-terminated names, so a name that holds a newline needs no quoting.
        input.push(0);
    }
    request.input = Some(input);
    let output: GitOutput = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(GitFailure(format!(
                "{} could not start: {error}",
                command_text(arguments.as_slice())
            )));
        }
    };
    match output.code {
        Some(0) => {}
        Some(1) => return Ok(Vec::new()),
        _ => {
            return Err(GitFailure(format!(
                "{} failed: {}",
                command_text(arguments.as_slice()),
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
