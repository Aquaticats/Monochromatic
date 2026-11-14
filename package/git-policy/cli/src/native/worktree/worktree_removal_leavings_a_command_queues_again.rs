//! What:
//!  The look every command of a repository makes at the subvolumes its removals renamed aside:
//!  the delete of one no queue record owes any more is queued again,
//!  and a place this program may not act on is named for the person.
//! Why:
//!  A removal renames its subvolume to `cli-git-deleting-<id>` and records the delete for the commit service
//!  (`worktree_remove_hand_over.rs`).
//!  The record can be settled before the rename and the person's own command then deletes the directory itself,
//!  which fails where the kernel keeps a reason to refuse:
//!  a subvolume that holds another one (`ENOTEMPTY`),
//!  a mount without `user_subvol_rm_allowed` (`EPERM`),
//!  a volume gone read-only.
//!  A command killed between the rename and that repair leaves the same state with nothing at all to notice it,
//!  and a round that gives up settles the record in `failed`,
//! which nothing scans.
//!  The leaving record written before the rename outlives all of them
//!  (`worktree_removal_leaving_that_outlives_its_record.rs`),
//!  and this is the step that reads it.
//!  It queues only what it can prove:
//!  the directory must be named the way nothing but this program's rename names one,
//!  it must be a subvolume,
//!  and its id must be the one the record holds,
//!  which is the proof the service's own delete makes before it destroys anything.
//!  A directory that is not that subvolume is left exactly as it is and named,
//!  because a person's directory must never be deleted for a leaving this program lost.
//!  Nothing here destroys a subvolume:
//!  the delete goes to the queue a round runs,
//! as every handed-over delete does
//!  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { queued, said } = queueTheLeavingsARecordNoLongerNames(commonDir);
//! if (queued) await wakeTheService(context, commonDir);
//! for (const line of said) stderr.write(line + '\n');
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{open_directory_at, open_root};
use crate::fs_btrfs_ioctl::subvolume_id;
use crate::plain_file::read_plain_file;
use crate::service_work::{MAX_TRIES, work_directory};
use crate::work_item::queued_ids;
use crate::worktree_removal_finishes_an_interrupted_destroy::owner_may_be_running;
use crate::worktree_removal_leaving_that_outlives_its_record::{Leaving, RECORD_LIMIT, forget_the_leaving, leaving_records, queue_the_delete_again, read_the_leaving};
use crate::worktree_removal_takes_its_pending_deletions::is_a_leaving_name;
use crate::worktree_remove_hand_over::{DELETE_REFERENCE, recorded_subvolume};
use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeavings";

/// The words every line of this module starts with,
/// so a person reading a terminal knows which program said it.
const SAID: &str = "monochromatic-git-policy-cli:";

/// What:
///  What one command's look at the leaving records came to.
/// Why:
///   The caller tells the service once when a delete was queued,
///  and writes every line it is given,
///  so the two halves of the answer travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Leavings = { queued: boolean; said: string[] };
/// ```
pub struct Leavings {
    /// Whether this command queued a delete a round of the service will run.
    pub queued: bool,
    /// What the person reads:
    /// one line per leaving this program may not delete and per queueing that could not be made.
    pub said: Vec<String>,
}

/// What:
///  Queue the delete of every leaving no queue record owes any more,
///  and say what stands where nothing of this program's may act.
/// Why:
///   This is what every command of the repository runs,
/// so a repository in which no removal left anything pays one open
///  that finds no folder.
///   A record whose command may still be running is left alone:
///  the record is written before the rename,
///  so only that command knows whether the rename happened,
///  and a look that decided before it did would drop the only record of a directory about to appear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = { queued: false, said: [] };
/// for (const [id, record] of leavingRecords(commonDir)) queueOneLeaving({ commonDir, record, id, answer });
/// return answer;
/// ```
pub fn queue_the_leavings_a_record_no_longer_names(common_dir: &Path) -> Leavings {
    let mut answer: Leavings = Leavings { queued: false, said: Vec::new() };
    let records: Vec<(String, PathBuf)> = leaving_records(common_dir);
    if records.is_empty() {
        return answer;
    }
    let work: PathBuf = work_directory(common_dir);
    // One listing of each state a round acts on, asked only when a removal left something to decide about.
    let owed: HashSet<String> = queued_ids(work.as_path());
    for (id, record) in records {
        queue_one_leaving(common_dir, record.as_path(), id.as_str(), &owed, &mut answer);
    }
    return answer;
}

/// What:
///  What stands where a leaving record says the rename put its subvolume.
/// Why:
///   A delete may act on one of these and must not act on the others,
///  so the look answers which one it found rather than a bare yes or no.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Standing = 'gone' | 'not-this-leaving' | 'somebody-elses' | 'the-subvolume-it-names' | { unreadable: string };
/// ```
enum Standing {
    /// Nothing is at the path,
    /// or nothing a delete of this record could act on:
    /// the leaving is finished.
    Gone,
    /// A path no leaving of this program is named:
    /// the record names something else.
    NotThisLeaving,
    /// A directory that is not the subvolume the record names,
    /// which is someone else's now.
    SomebodyElses,
    /// The subvolume the record names.
    TheSubvolumeItNames,
    /// A place that cannot be looked at,
    /// with the reason.
    Unreadable(String),
}

/// What:
///  What stands at `target`,
///  opened below its parent as the service's own delete opens it,
///  or the reason this command cannot tell.
/// Why:
///   The id names one subvolume for good,
///  while a name names whatever stands there when the call runs,
///  so a directory whose id is another one is not the leaving this record is about and is never queued for a delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = whatStandsAt(target, recorded); // opens the parent, then the name, then asks the id
/// ```
fn what_stands_at(target: &Path, recorded: u64) -> Standing {
    if !target.is_absolute() || !is_a_leaving_name(target) {
        return Standing::NotThisLeaving;
    }
    let Some(parent_path) = target.parent() else { return Standing::NotThisLeaving };
    let Some(name) = target.file_name() else { return Standing::NotThisLeaving };
    let parent: File = match open_root(parent_path) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Standing::Gone,
        Err(error) => return Standing::Unreadable(format!("the directory {} cannot be opened ({error})", parent_path.display())),
    };
    let held: File = match open_directory_at(&parent, name) {
        Ok(opened) => opened,
        Err(error) if is_gone(error.kind()) => return Standing::Gone,
        Err(error) => return Standing::Unreadable(format!("{} cannot be opened ({error})", target.display())),
    };
    return match subvolume_id(&held, target) {
        Ok(id) if id == recorded => Standing::TheSubvolumeItNames,
        Ok(_other) => Standing::SomebodyElses,
        Err(error) if error.errno == libc::ENOENT => Standing::Gone,
        Err(error) => Standing::Unreadable(format!("the subvolume of {} cannot be identified ({error})", target.display())),
    };
}

/// Whether an error means nothing stands at the name any more.
fn is_gone(kind: std::io::ErrorKind) -> bool {
    return kind == std::io::ErrorKind::NotFound || kind == std::io::ErrorKind::NotADirectory;
}

/// What:
///  Decide the one leaving of `record`,
///  adding what it queued and what the person must read to `answer`.
/// Why:
///   One record's decision sits in one place,
///  so every record is judged the same way,
///  and the shape of the folder needs no more of this file than the loop above.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queueOneLeaving({ commonDir, record, id, owed, answer });
/// ```
fn queue_one_leaving(common_dir: &Path, record: &Path, id: &str, owed: &HashSet<String>, answer: &mut Leavings) {
    let bytes: Vec<u8> = match read_plain_file(record, RECORD_LIMIT) {
        Ok(read) => read,
        Err(error) => {
            // A named pipe, a device or a directory in this folder is not a record of this program's, and opening one would
            // wait for a writer that never comes, on every command of the repository.
            debug(TAG, format!("{} is not a plain record and is left alone ({error})", record.display()).as_str());
            return;
        }
    };
    let leaving: Leaving = match read_the_leaving(bytes.as_slice()) {
        Ok(found) => found,
        Err(reason) => {
            answer.said.push(format!(
                "{SAID} the record of a worktree removal's renamed subvolume {record:?} cannot be used ({reason}), so whatever stands where it named it was left exactly as it is; `btrfs subvolume delete <that directory>` removes it by hand."
            ));
            forget_the_leaving(common_dir, id);
            return;
        }
    };
    if owner_may_be_running(leaving.owner.as_ref()) {
        debug(TAG, format!("the record {record:?} was written by a command that may still be running, so it is left alone").as_str());
        return;
    }
    if leaving.item.branch_ref != DELETE_REFERENCE {
        answer.said.push(format!("{SAID} the record {record:?} does not name a subvolume a removal renamed aside, so it was left exactly as it is."));
        forget_the_leaving(common_dir, id);
        return;
    }
    let Some(recorded): Option<u64> = recorded_subvolume(leaving.item.oid.as_str()) else {
        answer.said.push(format!(
            "{SAID} the record {record:?} names no subvolume id ({}), so nothing proves which subvolume stands at {} and it was left exactly as it is",
            leaving.item.oid, leaving.item.parent
        ));
        forget_the_leaving(common_dir, id);
        return;
    };
    let target: PathBuf = PathBuf::from(leaving.item.parent.as_str());
    match what_stands_at(target.as_path(), recorded) {
        Standing::Gone => {
            debug(TAG, format!("nothing of a removal's stands at {}, so its leaving is finished", target.display()).as_str());
            forget_the_leaving(common_dir, id);
            return;
        }
        Standing::NotThisLeaving => {
            answer.said.push(format!(
                "{SAID} the record {record:?} names {named}, which is not a name a removal of a worktree gives the subvolume it renames aside, so it was left exactly as it is.",
                named = target.display()
            ));
            forget_the_leaving(common_dir, id);
            return;
        }
        Standing::SomebodyElses => {
            answer.said.push(format!(
                "{SAID} the removal that recorded {named} renamed the subvolume {recorded} aside, and a directory that is not that subvolume stands there now, so it was left exactly as it is; `btrfs subvolume delete {named}` removes a directory of your own there.",
                named = target.display()
            ));
            forget_the_leaving(common_dir, id);
            return;
        }
        Standing::Unreadable(reason) => {
            answer.said.push(format!(
                "{SAID} {named} is where a removal renamed the subvolume of a worktree aside, and it cannot be looked at ({reason}), so no delete of it was queued; `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
                named = target.display()
            ));
            return;
        }
        Standing::TheSubvolumeItNames => {}
    }
    queue_the_delete_of_one_standing_leaving(common_dir, &leaving, target.as_path(), owed, answer);
}

/// What:
///  Queue the delete of a leaving whose subvolume still stands where the rename put it,
///  unless a round already owes that delete,
///  and say so when the attempts a record may cost are spent.
/// Why:
///   A record in `pending`,
/// `claimed`,
/// `recovering` or `held` is work a round has or will be handed,
///  so queueing it again would cost a round and say nothing.
///   Everything else is a delete nothing reads again:
///  a record settled in `done` by a round that looked before the rename,
///  one settled in `failed` once its attempts ran out,
///  or no record at all.
///   The count of queueings is what ends this:
///  a directory the kernel refuses every time must be the person's to remove rather than an attempt on every command,
///  and the words name the directory and the two commands that end it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (owed.has(leaving.item.id())) return;
/// if (leaving.tries >= MAX_TRIES) { answer.said.push(spentWords); return; }
/// const queued = queueTheDeleteAgain(commonDir, leaving); if (queued.error) { answer.said.push(cannotQueueWords(queued.error)); return; }
/// answer.queued = true;
/// ```
fn queue_the_delete_of_one_standing_leaving(common_dir: &Path, leaving: &Leaving, target: &Path, owed: &HashSet<String>, answer: &mut Leavings) {
    if owed.contains(leaving.item.id().as_str()) {
        debug(TAG, format!("a round of the service owes the delete of {}, so nothing here is queued", target.display()).as_str());
        return;
    }
    if leaving.tries >= MAX_TRIES {
        answer.said.push(format!(
            "{SAID} the subvolume a removed worktree was renamed to at {named} is still there after {} more attempts to delete it, so nothing here deletes it; `journalctl --identifier=monochromatic-git-policy-cli` says why the commit service could not, and `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
            leaving.tries,
            named = target.display()
        ));
        return;
    }
    if let Err(reason) = queue_the_delete_again(common_dir, leaving) {
        answer.said.push(format!(
            "{SAID} the subvolume a removed worktree was renamed to at {named} still stands there, and the delete that would remove it cannot be queued ({reason}), so `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
            named = target.display()
        ));
        return;
    }
    answer.queued = true;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_leavings_a_command_queues_again_tests.rs"]
mod tests;
