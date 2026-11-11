//! What:
//!  The selected elements of an explicit-path commit that match no entry of the commit's private index,
//! and the finding that names them and both ways forward.
//! Why:
//!  Every element after `--` is a path,
//! so an option written there (`-m`, `--file`, a word of the message a person meant to give)
//! is read as a pathspec, exactly as native Git reads it,
//! and the private staging refuses the whole selection.
//! That refusal is the command line's,
//! not the engine's or the transaction's,
//! so it ends the command as a finding with its own code and exit `1`
//! rather than as an `engine-failure` that tells the person to look for a landing that cannot exist
//! (`doc/planning/cli-git-commits-that-stopped-after-their-add-steps.md`).
//!  The question is asked only after the private staging refused the selection,
//! so a commit that stages its selection correctly starts no Git for it,
//! and Git answers it over the same index and the same pathspec modes the staging used,
//! never from the words of a Git message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! git(['ls-files', '--cached', '-z', '--', element], { index: commitIndex }).stdout.length === 0
//! ```

use crate::commit_index_selection::IndexSelection;
use crate::commit_selection_staging::{Selected, names_one_file, pattern_characters_are_literal, selection_items};
use crate::path_bytes::path_message_text;
use crate::private_index_request::caller_pathspec_request;
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, command_text, run_git};
use std::ffi::OsString;
use std::path::Path;

/// The core of the finding:
///  the fixed behavior of an explicit-path commit's selection,
/// which no configuration can turn off.
pub const NAMED_UNMATCHED_CORE_ID: &str = "explicit-path-commit";

/// The code of the finding,
/// reported as `explicit-path-commit/named-path-unmatched`.
pub const NAMED_UNMATCHED_CODE: &str = "named-path-unmatched";

/// What:
///  The selected elements that match no entry the private index holds,
/// in the order the selection names them.
/// Why:
///   Only these elements make the private staging refuse,
/// and every one of them is named so a person sees what Git read as a path.
///  An element that names one file of the worktree is not asked about:
/// the worktree is the question asked of it,
///  and a refusal there is Git's own text.
/// A selection whose every element matches asks Git nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const patterns = elements.filter((element) => !namesOneFile(cwd, element)); return patterns.filter((element) => !indexHoldsAPathFor(commitIndex, element));
/// ```
pub fn named_unmatched_elements(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
    selection: &IndexSelection,
) -> Result<Vec<Vec<u8>>, GitFailure> {
    // A selection read from a pathspec file this reader does not take stages under a name of its own: the file is
    // Git's to read, so the elements that could be named here are not known, and that staging answers for itself
    // (`add_selected_paths`).
    let Selected::Items(items) = selection_items(selection) else {
        return Ok(Vec::new());
    };
    let literal: bool = pattern_characters_are_literal(context);
    let mut unmatched: Vec<Vec<u8>> = Vec::new();
    for item in items.iter() {
        if names_one_file(cwd, item.as_os_str(), literal) {
            continue;
        }
        if matches_nothing(context, cwd, commit_index, item)? {
            unmatched.push(item.as_encoded_bytes().to_vec());
        }
    }
    return Ok(unmatched);
}

/// What:
///  Whether one element matches no entry of the private index,
/// asked of Git over that index in one start.
/// Why:
///   The private staging stages every element that names no single file with `git add --update`,
/// which takes the entries the index already holds and nothing else,
///  so an element this question finds no entry for is one that staging cannot take either.
///  Git answers the question with pathspec semantics the crate does not reimplement (glob modes,
/// magic, a directory that holds entries), and exit `1` from `git ls-files` is a pathspec Git refuses to read,
/// which is not a refusal to report as the command line's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git(['ls-files', '--cached', '-z', '--', element], { index: commitIndex, pathspecModes: inEffect })
/// ```
fn matches_nothing(context: &GitContext, cwd: &Path, commit_index: &Path, element: &OsString) -> Result<bool, GitFailure> {
    let mut arguments: Vec<OsString> = ["ls-files", "--cached", "-z", "--"].iter().map(OsString::from).collect();
    arguments.push(element.clone());
    let request: GitRequest = caller_pathspec_request(context, cwd, arguments.as_slice(), Some(commit_index), None);
    let output: GitOutput = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(GitFailure(format!(
                "{} could not start: {error}",
                command_text(request.arguments.as_slice())
            )));
        }
    };
    if !output.succeeded() {
        return Err(GitFailure(format!(
            "{} failed: {}",
            command_text(request.arguments.as_slice()),
            output.error_text()
        )));
    }
    return Ok(output.stdout.is_empty());
}

/// What:
///  The message of the finding:
/// the unmatched elements,
/// what happened to the commit,
/// where an option belongs,
/// and both legitimate ways forward.
/// Why:
///  A person who wrote the message after the paths needs to see the order,
/// and one who named a path that is not there needs to see that too;
/// the diagnostic names both rather than guessing which mistake was made,
/// and it never asks whether the commit landed,
/// because it cannot have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ... so this commit was not made and nothing was staged by it: ${JSON.stringify(names)}. ...`
/// ```
pub fn named_unmatched_message(elements: &[Vec<u8>]) -> String {
    let mut names: Vec<String> = Vec::new();
    for element in elements {
        names.push(path_message_text(element.as_slice()));
    }
    let listed: String = serde_json::to_string(&names).unwrap_or_default();
    return format!(
        "monochromatic-git-policy-cli: git commit with explicit paths commits the worktree content of the paths it \
names, and these elements match no file of the worktree and no entry of the commit's index, so this commit was not \
made and nothing was staged by it: {listed}. Native Git refuses the same command line with exit status 1. Every \
element after `--` is a path and never an option, so a message written after the paths is read as a path: write it \
before them, as git commit -m <message> -- <paths>, or name elements the worktree or the commit's index holds."
    );
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_named_unmatched_files_tests.rs"]
mod tests;
