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

// The names of the files this step was split into stay reachable here,
// so a caller and a control still name the reset alone.
pub(crate) use crate::service_land_follow_index_reset_git_questions::classify;
pub(crate) use crate::service_land_follow_index_reset_paths::{
    derived_names,
    derived_names_awaiting,
};

use crate::inner_git_words::record_words;
use crate::service_land_follow_index_reset_git_questions::{
    run,
    run_awaiting,
};
use crate::service_land_follow_index_reset_paths::{
    pathspec_input,
    top_literal_spec,
};
use crate::service_protocol::FailureCode;
use crate::transaction_git::{
    GitContext,
    GitOutput,
};
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

/// One try of the index update.
pub(super) fn reset_once(
    git: &GitContext,
    repository: &Path,
    new: &str,
    paths: IndexPaths<'_>,
    index: Option<&Path>,
) -> Result<(), FailureCode> {
    return match reset_once_asked(git, repository, new, paths, index)? {
        Some(output) => reset_answered(output),
        // No path to reset: the index is left as it is and no step ran.
        None => Ok(()),
    };
}

/// One try of the index update,
/// with its Git child awaited on the runtime.
pub(super) async fn reset_once_awaiting(
    git: &GitContext,
    repository: &Path,
    new: &str,
    paths: IndexPaths<'_>,
    index: Option<&Path>,
) -> Result<(), FailureCode> {
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
fn reset_once_asked(
    git: &GitContext,
    repository: &Path,
    new: &str,
    paths: IndexPaths<'_>,
    index: Option<&Path>,
) -> Result<Option<GitOutput>, FailureCode> {
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
            run(
                git,
                repository,
                &[
                    "reset",
                    "--no-refresh",
                    new,
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                Some(input),
                index,
            )
            .map(Some)
        }
        IndexPaths::Derived {
            old,
        } => {
            let input: Vec<u8> =
                pathspec_input(derived_names(git, repository, old, new)?.as_slice());
            if input.is_empty() {
                return Ok(None);
            }
            run(
                git,
                repository,
                &[
                    "reset",
                    "--no-refresh",
                    new,
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                Some(input),
                index,
            )
            .map(Some)
        }
    };
}

/// `reset_once_asked` with its Git child awaited on the runtime.
async fn reset_once_asked_awaiting(
    git: &GitContext,
    repository: &Path,
    new: &str,
    paths: IndexPaths<'_>,
    index: Option<&Path>,
) -> Result<Option<GitOutput>, FailureCode> {
    return match paths {
        IndexPaths::Listed(listed) => {
            let specs: Vec<String> = listed.iter().map(top_literal_spec).collect();
            let mut arguments: Vec<&str> = vec!["reset", "--no-refresh", new, "--"];
            arguments.extend(specs.iter().map(String::as_str));
            run_awaiting(git, repository, &arguments, None, index)
                .await
                .map(Some)
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
            run_awaiting(
                git,
                repository,
                &[
                    "reset",
                    "--no-refresh",
                    new,
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                Some(input),
                index,
            )
            .await
            .map(Some)
        }
        IndexPaths::Derived {
            old,
        } => {
            let input: Vec<u8> = pathspec_input(
                derived_names_awaiting(git, repository, old, new)
                    .await?
                    .as_slice(),
            );
            if input.is_empty() {
                return Ok(None);
            }
            run_awaiting(
                git,
                repository,
                &[
                    "reset",
                    "--no-refresh",
                    new,
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                Some(input),
                index,
            )
            .await
            .map(Some)
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
pub fn reset_entries(
    git: &GitContext,
    repository: &Path,
    index: Option<&Path>,
    new: &str,
    names: &[Vec<u8>],
) -> Result<(), FailureCode> {
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
pub async fn reset_entries_awaiting(
    git: &GitContext,
    repository: &Path,
    index: Option<&Path>,
    new: &str,
    names: &[Vec<u8>],
) -> Result<(), FailureCode> {
    if names.is_empty() {
        return Ok(());
    }
    return reset_once_awaiting(git, repository, new, IndexPaths::Named(names), index).await;
}
