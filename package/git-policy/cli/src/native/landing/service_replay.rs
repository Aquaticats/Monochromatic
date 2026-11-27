//! What:
//!  The retry of a commit the service sent back because a path it names changed on the branch since its base:
//! the change
//!  the first preparation captured is merged onto the branch as it is now,
//! and the worktree is not read again.
//! Why:
//!  A commit lands the bytes that were there when the person ran the command (`doc/decision/cli-git-concurrent-commits.md`,
//!  "Serial landing";
//! `SPEC.md`:
//! "Cli-git never re-prepares a commit from current worktree bytes").
//! Preparing again from
//!  the worktree landed bytes written after the command,
//! and bytes another command meant.
//! The merge base is the
//!  preparation base,
//! and a path the other commits changed that the captured change already contains is kept as captured
//!  (the same subsumption rule the earlier landing path applies,
//! `replay_subsumption.rs`).
//! The first merge is one Git
//!  run;
//! the subsumption runs only for the paths that conflicted,
//! so a retry that merges cleanly costs three runs here:
//!  `write-tree` for the captured tree,
//! `merge-tree`,
//! and `read-tree` of the merged tree into the private index.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const captured = await captureTree(readyCommit);              // after the service sent the commit back
//! const merged = await mergeOntoBranch(captured, currentTip);   // { tree } | { conflict }
//! ```

pub use super::service_replay_the_captured_change::{
    Captured,
    MergeSite,
    Merged,
    Retry,
    capture_stamp,
    captured_tree,
    retry_of,
    retry_of_a_prepared_tree,
    tree_of_commit,
};
pub use super::service_replay_the_merge_onto_the_branch::{
    first_landed,
    merge_captured,
};
pub use super::service_replay_the_word_pass::word_merge_notice;

use crate::commit_transaction::TransactionResult;
use crate::path_bytes::path_message_text;
use crate::policy_events::PolicyEvent;
use crate::service_messages::NOTHING_CHANGED;
pub use crate::service_prepare_ready_shapes::{
    LandedFirst,
    ReplayConflict,
};
use std::path::Path;

/// The transform that names the finding a retry ends with,
/// as for every other concurrent-commit finding.
const CORE_ID: &str = "concurrent-commit";

/// What:
///  A path as the person reads it in quotes.
/// Why:
///  `paths_text` lists the paths in conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quotedPath = (path: Uint8Array) => `\`${pathMessageText(path)}\``;
/// ```
fn quoted_path(path: &Vec<u8>) -> String {
    return format!("`{}`", path_message_text(path.as_slice()));
}

/// The paths as the person reads them:
/// one in quotes,
/// several as a list.
fn paths_text(paths: &[Vec<u8>]) -> String {
    let quoted: Vec<String> = paths.iter().map(quoted_path).collect();
    return match quoted.as_slice() {
        [only] => format!("the file {only}"),
        many => format!("these files: {}", many.join(", ")),
    };
}

/// What:
///  The words for a conflict:
/// what overlapped,
/// which commit landed first,
/// where the message is saved,
/// how to commit again.
/// Why:
///   Plain words,
/// every input named and every way forward given,
/// nothing about how the commit is landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// conflictText(conflict, savedMessage)
/// ```
pub fn conflict_text(
    conflict: &ReplayConflict,
    saved: Option<&Path>,
) -> String {
    let landed: String = match &conflict.landed {
        Some(first) => format!("the commit {} (\"{}\")", first.short, first.subject),
        None => String::from("another commit"),
    };
    let look: String = match &conflict.landed {
        Some(first) => format!(
            "Look at what that commit changed with `git show {}`, bring your",
            first.short
        ),
        None => String::from("Look at what the branch changed, bring your"),
    };
    let again: String = match saved {
        Some(file) => format!(
            "Your message is saved in {}. {look} edit up to date with it, and run the commit again with `git commit -F {}` and the same paths.",
            file.display(),
            file.display()
        ),
        None => format!("{look} edit up to date with it, and run the commit again."),
    };
    return format!(
        "This commit was not made. While it was being prepared, {landed} changed {}, which this commit also changes, and the two \
         changes cannot be combined. {again} {NOTHING_CHANGED}",
        paths_text(conflict.paths.as_slice())
    );
}

/// The finding a conflict ends the command with.
pub fn conflict_result(
    conflict: &ReplayConflict,
    saved: Option<&Path>,
) -> TransactionResult {
    return TransactionResult {
        events: vec![PolicyEvent::CoreFinding {
            core_id: CORE_ID,
            code: "replay-conflict",
            message: conflict_text(conflict, saved),
        }],
        exit_code: 1,
        landed_oid: None,
        uses_lfs: false,
    };
}

/// The ending of a retry whose merged result is what the branch already holds.
pub fn already_committed_result(branch: &str) -> TransactionResult {
    return TransactionResult {
        events: vec![PolicyEvent::CoreFinding {
            core_id: CORE_ID,
            code: "already-committed",
            message: format!(
                "This commit was not made. While it was being prepared, other commits put everything it changes on the branch {branch}, \
                 so nothing is left to commit. Run `git log` to see them. {NOTHING_CHANGED}"
            ),
        }],
        exit_code: 1,
        landed_oid: None,
        uses_lfs: false,
    };
}

/// Replay controls stay out of the release executable.
#[cfg(test)]
#[path = "service_replay_tests.rs"]
mod tests;
