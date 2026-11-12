//! What:
//!  The entries a landing installs in the worktree's index:
//! the paths a commit changed,
//! the pathspecs a reset
//!  reads them as,
//! and the one step that resets them,
//! with a synchronous face and an awaited one.
//! Why:
//!  A commit Git built in a scratch repository carries no list of changes,
//! so the entries come from the repository itself,
//! as bytes,
//! so a name that is not UTF-8,
//! a long list and a gitlink need no special case.
//! The paths and the step are
//!  this file rather than `service_land_follow.rs` to keep both under the file-size limit
//! (`AGENTS.md`,
//! `MXL`),
//! and no other step of the follow-up changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await resetEntriesAwaiting(git, repository, index, newCommit, names);
//! ```

use crate::inner_git_words::record_words;
use crate::service_protocol::{FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::path::Path;

/// Which index entries a landing updates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexPaths<'a> {
    /// The paths the request listed.
    Listed(&'a [String]),
    /// Paths given as bytes,
    /// for a client that reconciles the index without a landing.
    Named(&'a [Vec<u8>]),
    /// Every path that differs between the branch before the landing and the new commit.
    Derived {
        /// The branch before the landing;
        /// the all-zero name when the branch did not exist.
        old: &'a str,
    },
}

/// What:
///  The question one step of this module asks Git,
/// with the input it feeds it and the index it reads.
/// Why:
///  Both faces ask one request,
/// so neither can run another command,
/// feed other bytes or read another index than
///  the other;
/// every pathspec this module writes is its own,
/// so no pathspec mode of the caller's reaches it
///  (`GitRequest::own_pathspecs`),
/// whether the caller set it as a variable or as a global option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { ...new GitRequest(repository, arguments), ownPathspecs: true, input, index };
/// ```
fn request_of(repository: &Path, arguments: &[&str], input: Option<Vec<u8>>, index: Option<&Path>) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(repository, arguments).own_pathspecs();
    request.input = input;
    request.index = index.map(Path::to_path_buf);
    return request;
}

/// What:
///  Git's output of one step,
///  and the failure a Git that could not be started stands for.
/// Why:
///  Both faces read the run through this one function,
/// so a step Git could not start is the same failure for both of
///  them,
/// and every other answer is Git's own output whichever face asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runAnswered = (answered) => answered.error ? 'unavailable' : answered;
/// ```
fn run_answered(answered: std::io::Result<GitOutput>) -> Result<GitOutput, FailureCode> {
    return match answered {
        Ok(output) => Ok(output),
        Err(_unstartable) => Err(FailureCode::Unavailable),
    };
}

fn run(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>, index: Option<&Path>) -> Result<GitOutput, FailureCode> {
    return run_answered(run_git(git, &request_of(repository, arguments, input, index)));
}

/// What:
///  `run` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and every step of this module runs
///  from the landing once that landing is a task.
/// The question and the reading of the answer are the synchronous face's,
/// so
///  the two cannot differ in what they run or in how they fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runAwaiting(git, repository, arguments, input, index);
/// ```
async fn run_awaiting(git: &GitContext, repository: &Path, arguments: &[&str], input: Option<Vec<u8>>, index: Option<&Path>) -> Result<GitOutput, FailureCode> {
    return run_answered(run_git_awaiting(git, &request_of(repository, arguments, input, index)).await);
}

/// The failure a Git step that did not succeed stands for.
pub fn classify(output: &GitOutput) -> FailureCode {
    let text: String = output.error_text().to_lowercase();
    if text.contains("no space left") || text.contains("quota exceeded") {
        return FailureCode::RepositoryFull;
    }
    if text.contains("read-only file system") {
        return FailureCode::RepositoryReadOnly;
    }
    if text.contains("permission denied") {
        return FailureCode::PermissionDenied;
    }
    if text.contains("index.lock") {
        return FailureCode::IndexInUse;
    }
    return FailureCode::Unavailable;
}

/// A path in the form `reset` reads literally from the top of the repository.
fn literal(path: &[u8]) -> Vec<u8> {
    let mut spec: Vec<u8> = b":(top,literal)".to_vec();
    spec.extend_from_slice(path);
    return spec;
}

/// The names (bytes,
/// NUL-separated) of the paths that differ between `old` and `new`.
pub(crate) fn derived_names(git: &GitContext, repository: &Path, old: &str, new: &str) -> Result<Vec<u8>, FailureCode> {
    let output: GitOutput = run(git, repository, &derived_names_arguments(old, new), None, None)?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(output.stdout);
}

/// What:
///  `derived_names` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the paths a batch changed are read when the index is
///  updated from the landing.
/// The arguments and the failure a refused step stands for are the synchronous face's,
/// so the two
///  read one list of names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = await derivedNamesAwaiting(git, repository, old, new);
/// ```
pub(crate) async fn derived_names_awaiting(git: &GitContext, repository: &Path, old: &str, new: &str) -> Result<Vec<u8>, FailureCode> {
    let output: GitOutput = run_awaiting(git, repository, &derived_names_arguments(old, new), None, None).await?;
    if !output.succeeded() {
        return Err(classify(&output));
    }
    return Ok(output.stdout);
}

/// The arguments of the question `derived_names` asks Git.
fn derived_names_arguments<'a>(old: &'a str, new: &'a str) -> Vec<&'a str> {
    let mut arguments: Vec<&str> = vec!["diff-tree", "-r", "--name-only", "-z", "--no-renames"];
    if is_zero_object(old) {
        // A branch that did not exist: the commit is a root commit, and `--root` lists everything it holds.
        arguments.push("--root");
        arguments.push(new);
    } else {
        arguments.push(old);
        arguments.push(new);
    }
    return arguments;
}

/// What:
///  Whether a byte is the NUL that separates the names of a pathspec list.
/// Why:
///  `pathspec_input` splits a NUL-separated list into names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isZeroByte = (byte: number) => byte === 0;
/// ```
fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether a name from a NUL-separated list is not empty.
/// Why:
///  Two NULs in a row leave an empty name that names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNotEmptyName = (name: Uint8Array) => name.length > 0;
/// ```
fn is_not_empty_name(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// What:
///  A path as a pathspec that matches that path from the top of the worktree and nothing else.
/// Why:
///  `reset` is given one such spec per path the batch changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const topLiteralSpec = (path: string) => `:(top,literal)${path}`;
/// ```
fn top_literal_spec(path: &String) -> String {
    return format!(":(top,literal){path}");
}

/// The standard input of `reset --pathspec-from-file=- --pathspec-file-nul` for a list of NUL-separated names.
fn pathspec_input(names: &[u8]) -> Vec<u8> {
    let mut input: Vec<u8> = Vec::new();
    for name in names.split(is_zero_byte).filter(is_not_empty_name) {
        input.extend(literal(name));
        input.push(0);
    }
    return input;
}

/// One try of the index update.
pub(super) fn reset_once(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, index: Option<&Path>) -> Result<(), FailureCode> {
    return match reset_once_asked(git, repository, new, paths, index)? {
        Some(output) => reset_answered(output),
        // No path to reset: the index is left as it is and no step ran.
        None => Ok(()),
    };
}

/// One try of the index update,
/// with its Git child awaited on the runtime.
pub(super) async fn reset_once_awaiting(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, index: Option<&Path>) -> Result<(), FailureCode> {
    return match reset_once_asked_awaiting(git, repository, new, paths, index).await? {
        Some(output) => reset_answered(output),
        // No path to reset: the index is left as it is and no step ran.
        None => Ok(()),
    };
}

/// What:
///  What a reset that ran stands for:
/// done when Git took it,
/// and the failure code when it refused it.
/// Why:
///  Both faces read the reset through this one function,
/// so a refusal reaches the caller with the same code whichever face ran
///  it,
/// and the list of paths a reset left changed is recorded once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resetAnswered = (output) => { if (!output.succeeded) return classify(output); recordWords(['reset', '<commit>', '--', '<paths>'], output); }
/// ```
fn reset_answered(output: GitOutput) -> Result<(), FailureCode> {
    if !output.succeeded() {
        return Err(classify(&output));
    }
    // The `reset` carries no `-q`: its list of paths left changed in the worktree ("Unstaged changes after reset:") is kept
    //  in the service's log.
    record_words(&["reset", "<commit>", "--", "<paths>"], &output);
    return Ok(());
}

/// What:
///  The reset one try asks Git for,
///  whatever shape the paths take.
/// Why:
///  A listed list is its own pathspecs on the command line,
/// a named list and a derived list are fed as a
///  NUL-separated pathspec file,
/// and a list that holds no name resets nothing,
/// because `reset` with no path would
///  reset the whole index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resetOnceAsked(git, repository, new, paths, index): Promise<GitOutput>
/// ```
fn reset_once_asked(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, index: Option<&Path>) -> Result<Option<GitOutput>, FailureCode> {
    return match paths {
        IndexPaths::Listed(listed) => {
            let specs: Vec<String> = listed.iter().map(top_literal_spec).collect();
            let mut arguments: Vec<&str> = vec!["reset", "--no-refresh", new, "--"];
            arguments.extend(specs.iter().map(String::as_str));
            run(git, repository, &arguments, None, index).map(Some)
        }
        IndexPaths::Named(names) => {
            // Each name followed by a NUL, as `--pathspec-file-nul` reads them.
            let mut joined: Vec<u8> = Vec::new();
            for name in names.iter() {
                joined.extend(name.iter().copied());
                joined.push(0);
            }
            let input: Vec<u8> = pathspec_input(joined.as_slice());
            if input.is_empty() {
                return Ok(None);
            }
            run(git, repository, &["reset", "--no-refresh", new, "--pathspec-from-file=-", "--pathspec-file-nul"], Some(input), index).map(Some)
        }
        IndexPaths::Derived { old } => {
            let input: Vec<u8> = pathspec_input(derived_names(git, repository, old, new)?.as_slice());
            if input.is_empty() {
                return Ok(None);
            }
            run(git, repository, &["reset", "--no-refresh", new, "--pathspec-from-file=-", "--pathspec-file-nul"], Some(input), index).map(Some)
        }
    };
}

/// `reset_once_asked` with its Git child awaited on the runtime.
async fn reset_once_asked_awaiting(git: &GitContext, repository: &Path, new: &str, paths: IndexPaths<'_>, index: Option<&Path>) -> Result<Option<GitOutput>, FailureCode> {
    return match paths {
        IndexPaths::Listed(listed) => {
            let specs: Vec<String> = listed.iter().map(top_literal_spec).collect();
            let mut arguments: Vec<&str> = vec!["reset", "--no-refresh", new, "--"];
            arguments.extend(specs.iter().map(String::as_str));
            run_awaiting(git, repository, &arguments, None, index).await.map(Some)
        }
        IndexPaths::Named(names) => {
            // Each name followed by a NUL, as `--pathspec-file-nul` reads them.
            let mut joined: Vec<u8> = Vec::new();
            for name in names.iter() {
                joined.extend(name.iter().copied());
                joined.push(0);
            }
            let input: Vec<u8> = pathspec_input(joined.as_slice());
            if input.is_empty() {
                return Ok(None);
            }
            run_awaiting(git, repository, &["reset", "--no-refresh", new, "--pathspec-from-file=-", "--pathspec-file-nul"], Some(input), index).await.map(Some)
        }
        IndexPaths::Derived { old } => {
            let input: Vec<u8> = pathspec_input(derived_names_awaiting(git, repository, old, new).await?.as_slice());
            if input.is_empty() {
                return Ok(None);
            }
            run_awaiting(git, repository, &["reset", "--no-refresh", new, "--pathspec-from-file=-", "--pathspec-file-nul"], Some(input), index).await.map(Some)
        }
    };
}

/// What:
///  Resets the index entries of `names` in `index` to the commit `new`,
///  without waiting and without a refresh.
/// Why:
///   The landing's own index update and the repair of an update it left owed are one step
///   (`owed_index_update.rs`):
///  a path whose entry the landing left is reset the same way by both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resetEntries(git, repository, index, newCommit, names);
/// ```
///
/// # Errors
///
/// The failure code of the `reset` that was refused.
pub fn reset_entries(git: &GitContext, repository: &Path, index: Option<&Path>, new: &str, names: &[Vec<u8>]) -> Result<(), FailureCode> {
    if names.is_empty() {
        return Ok(());
    }
    return reset_once(git, repository, new, IndexPaths::Named(names), index);
}

/// What:
///  `reset_entries` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child,
/// and the entries a repair of the landing's own update resets are
///  reset from the worktree's command once that command is a task.
/// An empty list changes nothing on both faces,
/// and the reset
///  itself is the synchronous face's,
/// so the two install one index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resetEntriesAwaiting(git, repository, index, newCommit, names);
/// ```
pub async fn reset_entries_awaiting(git: &GitContext, repository: &Path, index: Option<&Path>, new: &str, names: &[Vec<u8>]) -> Result<(), FailureCode> {
    if names.is_empty() {
        return Ok(());
    }
    return reset_once_awaiting(git, repository, new, IndexPaths::Named(names), index).await;
}
