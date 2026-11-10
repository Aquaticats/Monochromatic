//! What:
//!  The merge step of one replay:
//! the tree from `git merge-tree --write-tree` of the prepared change onto the landed tip.
//! Why:
//!  A commit that lost the race to another is prepared again over the tip the service names;
//! the merge says whether the two
//!  changes meet cleanly and,
//! when they do not,
//! which paths conflict,
//! in byte order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await mergeReplayTree({ gitPath, shadowPath, mergeBase, current, prepared, worktreeRoot });
//! ```

/// The conflicts of a merge tree,
/// with their stages,
/// for the word-level second pass.
use crate::merge_second_pass::{Conflict, conflicts_of_merge_tree};
/// The attribute tree a merge from objects reads.
use crate::child_environment::environment_with_overlay;
use crate::merge_second_pass::ContextRunner;
use crate::merge_second_pass_tree::{AttributeProbe, attribute_tree_variables_on_the_runtime};
/// Shadow runs and the raw runner for a merge whose exit status matters.
use crate::shadow_git::shadow_request;
use crate::transaction_git::{GitContext, GitFailure, GitRequest, run_git};
/// Filesystem paths.
use std::path::Path;

/// `git merge-tree` exit status reporting conflicts;
/// a tree ID still prints.
const MERGE_CONFLICT_EXIT: i32 = 1;

/// What:
///  The outcome of merging the prepared change onto the current target.
/// Why:
///   A clean merge yields a tree to rebuild a commit on;
/// a conflict yields the paths that name the winning commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReplayMerge = { kind: 'merged'; treeOid } | { kind: 'conflict'; paths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayMerge {
    /// The change applies cleanly;
    /// the merged tree is in the shadow store.
    Merged {
        /// Merged tree.
        tree_oid: String,
    },
    /// The change conflicts with the target.
    Conflict {
        /// Conflicting paths in Git path byte order,
        /// each once,
        /// as the exact bytes Git reported.
        paths: Vec<Vec<u8>>,
        /// The tree Git wrote,
        /// holding the conflicted files with markers.
        tree_oid: String,
        /// The conflicted paths with their stages,
        /// for the word-level second pass.
        conflicts: Vec<Conflict>,
    },
}

/// What:
///  A merge or rebuild failure:
///       a Git command failed,
/// or the commit could not be rebuilt.
/// Why:
///   Replay reports each as `transaction-failed`,
/// never as a conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(message);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayMergeError {
    /// A Git command failed,
    /// or `merge-tree` exited above `1`.
    Git(GitFailure),
    /// The prepared commit could not be rebuilt.
    Rebuild(String),
}

/// What:
///  Merges the prepared change onto the current target in the shadow:
///       the tree when clean,
/// or the conflicted paths in byte order.
/// Why:
///   `git merge-tree --write-tree` gives the answer with exit `1` for a conflict and a tree ID in both cases;
///       any other non-zero exit is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mergeReplayTree({ gitPath, shadowPath, mergeBase, current, prepared, worktreeRoot });
/// ```
pub fn merge_replay_tree(
    context: &GitContext,
    shadow: &Path,
    worktree_root: &Path,
    merge_base: &str,
    current: &str,
    prepared: &str,
) -> Result<ReplayMerge, ReplayMergeError> {
    let merge_base_argument: String = format!("--merge-base={merge_base}");
    let mut request: GitRequest = shadow_request(
        shadow,
        worktree_root,
        &[
            "merge-tree",
            "--write-tree",
            "-z",
            merge_base_argument.as_str(),
            current,
            prepared,
        ],
    );
    // No worktree is read, so the landed tip's `.gitattributes` is the one the merge applies (the `union` line included).
    let process_environment: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os().collect();
    let variables = environment_with_overlay(process_environment.as_slice(), context.overlay.as_slice());
    // The attribute read keeps none of the caller's other options, and the repository the line named is carried here.
    let repository = crate::repository_selection::from_global_options(worktree_root, context.global_prefix.as_slice());
    let probing = ContextRunner { context, cwd: worktree_root, without_prefix: true, repository, object_directory: None, environment: Vec::new() };
    let probe = AttributeProbe { directory: worktree_root, global_prefix: &[], variables: variables.as_slice(), runner: &probing };
    request.environment.extend(attribute_tree_variables_on_the_runtime(current, &probe));
    let output = match run_git(context, &request) {
        Ok(done) => done,
        Err(error) => {
            return Err(ReplayMergeError::Git(GitFailure(format!("git merge-tree could not start: {error}"))));
        }
    };
    let code: Option<i32> = output.code;
    if code != Some(0) && code != Some(MERGE_CONFLICT_EXIT) {
        return Err(ReplayMergeError::Git(GitFailure(format!(
            "git merge-tree could not replay {prepared} onto {current} (exit {code:?}): {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))));
    }
    // The first field is the tree; on a conflict the stages of the conflicted paths follow up to an empty field.
    // The paths stay bytes: a name that is not UTF-8 is reported with its exact bytes.
    let (tree_oid, conflicts) = conflicts_of_merge_tree(output.stdout.as_slice());
    if code == Some(0) {
        return Ok(ReplayMerge::Merged { tree_oid });
    }
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for conflict in &conflicts {
        paths.push(conflict.path.clone());
    }
    paths.sort();
    paths.dedup();
    return Ok(ReplayMerge::Conflict { paths, tree_oid, conflicts });
}

#[cfg(test)]
#[path = "replay_merge_tests.rs"]
mod tests;
