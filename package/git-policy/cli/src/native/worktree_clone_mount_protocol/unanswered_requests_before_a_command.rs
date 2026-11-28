//! What:
//!  The check one forwarded command makes in its repository's folder of
//!  unanswered requests,
//!  before the child Git starts:
//!  one `stat` when nothing waits,
//!  and the receipt read that removes an entry a pass has answered.
//! Why:
//!  The account's clone-local nudge has to be asked for on the command path
//!  the person already runs,
//!  and a command whose repository holds no entry must pay almost nothing,
//!  which is the cheap negative the record owes
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What a command pays,
//! in each case").
//!  Every failure on this path is silent and changes no status:
//!  a command that cannot write its `.git` leaves the trigger unavailable for
//!  its own repository,
//!  and the next trigger or `doctor` recovers the clone.
//!  The nudge itself stands behind its compiled switch:
//!  `ask_for_a_nudge` is called only in a build that turns
//!  `zfs-lost-event-nudge` on,
//!  which step 22 does once M8 sizes the gaps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! checkBeforeACommand(commonDir); // one stat, or the receipt read
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::unanswered_request_entry::RequestEntry;
use crate::worktree_clone_mount_protocol::unanswered_request_names::EntryName;
use crate::worktree_clone_mount_protocol::unanswered_requests_folder::UnansweredRequests;
use std::path::Path;

/// What:
///  Check this repository's folder once,
///  for one forwarded command.
/// Why:
///   A repository whose layout the wrapper declined names no common directory,
///  so a command there reads nothing at all,
///  and every other command pays the folder's one `stat`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = UnansweredRequests.inTheCommonDir(commonDir);
/// if (!folder.stands()) return; // one stat
/// checkAt(folder, ReceiptsFolder.live());
/// ```
pub fn check_before_a_command(common_dir: Option<&Path>) {
    let Some(where_the_git_directory_is) = common_dir else {
        return;
    };
    let folder: UnansweredRequests =
        UnansweredRequests::in_the_common_dir(where_the_git_directory_is);
    check_at(&folder, &ReceiptsFolder::live());
}

/// What:
///  The check itself,
///  against the folder and the receipts a caller names.
/// Why:
///   A control hands in a fixture folder and its own receipts folder,
///  so the same reads are exercised without a repository's `.git` or `/run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [name, entry] of folder.theEntries())
///   if (answered(receipts, entry)) folder.removeEveryAttempt(entry.guid);
/// ```
pub(crate) fn check_at(
    folder: &UnansweredRequests,
    receipts: &ReceiptsFolder,
) {
    if !folder.stands() {
        return;
    }
    for (name, entry) in folder.the_entries() {
        if the_receipt_answers(receipts, &entry) {
            folder.remove_every_attempt(entry.guid);
            continue;
        }
        the_nudge_a_due_entry_asks_for(folder, name, &entry);
    }
}

/// Whether the clone's receipt answers one entry:
/// the same nonce and the same operation,
/// which is the pair the wait itself compares before it acts.
fn the_receipt_answers(
    receipts: &ReceiptsFolder,
    entry: &RequestEntry,
) -> bool {
    let Ok(Some(receipt)) = receipts.read(entry.guid) else {
        // A folder that is not there, a receipt root did not write and a file this reader refuses are all
        // the same answer here: nothing has answered the entry, and the command says nothing about it.
        return false;
    };
    return receipt.nonce == entry.nonce && receipt.operation == entry.operation;
}

/// What:
///  Ask for a nudge of one entry the receipt does not answer,
///  in a build whose nudge switch is on.
/// Why:
///   The switch is a compiled one,
///  so a release built without it carries no nudge at all and the command path
///  reads nothing of the folder beyond the answered entries it removes
///  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
///  "Counts and numbers this design does not ship").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (NUDGE_IS_COMPILED) askForANudge(folder, name, entry);
/// ```
fn the_nudge_a_due_entry_asks_for(
    folder: &UnansweredRequests,
    name: EntryName,
    entry: &RequestEntry,
) {
    // What: a build without the switch compiles no nudge, so its command path names none of this.
    // Why:  the switch is the coordinator's requirement in the record, which states that the nudge ships only
    //       after M8 sizes its gaps, and step 22 turns it on.
    #[cfg(feature = "zfs-lost-event-nudge")]
    crate::worktree_clone_mount_protocol::unanswered_requests_nudge::ask_for_a_nudge(
        folder, name, entry,
    );
    #[cfg(not(feature = "zfs-lost-event-nudge"))]
    let _no_nudge_in_this_build = (folder, name, entry);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unanswered_requests_before_a_command_tests.rs"]
mod tests;
