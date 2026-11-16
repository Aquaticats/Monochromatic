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
use crate::service_work::work_directory;
use crate::work_item::queued_ids;
use crate::worktree_removal_finishes_an_interrupted_destroy::owner_may_be_running;
use crate::worktree_removal_leaving_attempts::{Attempt, Slot};
use crate::worktree_removal_leaving_that_outlives_its_record::{Leaving, RECORD_LIMIT, forget_the_leaving, leaving_records, queue_the_delete_again, read_the_leaving};
use crate::worktree_removal_leavings_already_reported::{forget_the_report, report_once};
use crate::worktree_removal_leavings_no_record_names::queue_the_leavings_no_record_names;
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
    /// What:
    ///  Whether this command asked for a round of the service:
    ///  it queued a delete, or a delete a round already owes was told again.
    /// Why:
    ///   The caller wakes the service on this answer, and a record a killed command queued and never woke is exactly the
    ///  state a wake is owed for.
    pub queued: bool,
    /// What:
    ///  Every ask this command took an attempt slot for,
    ///  one slot per pending leaving it found,
    ///  so each is marked once the wake it is part of is made or given back when it is not.
    /// Why:
    ///  One command wakes the service once for every pending leaving it found,
    ///  so a single slot would count one leaving's ask and let an earlier one ask for ever.
    pub asks: Vec<Slot>,
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
///  that finds no folder, and one listing of each directory a linked worktree of it stands in.
///   A record whose command may still be running is left alone:
///  the record is written before the rename,
///  so only that command knows whether the rename happened,
///  and a look that decided before it did would drop the only record of a directory about to appear.
///   A directory of this program's leaving shape that no leaving record names at all,
///  which a release before the leaving record left,
///  is proved by the mark on it or by a record of its delete and queued as well
///  (`worktree_removal_leavings_no_record_names.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = { queued: false, said: [] };
/// for (const [id, record] of leavingRecords(commonDir)) queueOneLeaving({ commonDir, record, id, answer });
/// merge(answer, queueTheLeavingsNoRecordNames({ commonDir, work, named }));
/// return answer;
/// ```
pub fn queue_the_leavings_a_record_no_longer_names(common_dir: &Path) -> Leavings {
    let mut answer: Leavings = Leavings { queued: false, asks: Vec::new(), said: Vec::new() };
    let records: Vec<(String, PathBuf)> = leaving_records(common_dir);
    let work: PathBuf = work_directory(common_dir);
    // The places the records name, so a leftover a record names stays that record's to decide and the search by name skips it.
    let mut named: HashSet<PathBuf> = HashSet::new();
    if !records.is_empty() {
        // One listing of each state a round acts on, asked only when a removal left a record to decide about.
        let owed: HashSet<String> = queued_ids(work.as_path());
        for (id, record) in records {
            queue_one_leaving(common_dir, record.as_path(), id.as_str(), &owed, &mut named, &mut answer);
        }
    }
    let found: Leavings = queue_the_leavings_no_record_names(common_dir, work.as_path(), &named);
    answer.queued = answer.queued || found.queued;
    answer.asks.extend(found.asks);
    answer.said.extend(found.said);
    // What: The memories of places that are no longer the objects they named go, whoever removed them.
    // Why:  A memory drops when the look proves or queues its place, but a person who removes the directory themselves
    //       leaves the memory behind, and the next different object at that name would be said nothing about.
    crate::worktree_removal_leavings_already_reported::forget_the_reports_of_places_that_moved(common_dir);
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
/// queueOneLeaving({ commonDir, record, id, owed, named, answer });
/// ```
fn queue_one_leaving(common_dir: &Path, record: &Path, id: &str, owed: &HashSet<String>, named: &mut HashSet<PathBuf>, answer: &mut Leavings) {
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
            // A record that names no place leaves that place to the search by name, which needs no record; the line about the
            // record itself is said once and the record dropped, because nothing can use it later either.
            report_once(
                common_dir,
                record,
                format!(
                    "{SAID} the record of a worktree removal's renamed subvolume {record:?} cannot be used ({reason}), so whatever stands where it named it was left exactly as it is; `btrfs subvolume delete <that directory>` removes it by hand."
                ),
                &mut answer.said,
            );
            forget_the_leaving(common_dir, id);
            return;
        }
    };
    let target: PathBuf = PathBuf::from(leaving.item.parent.as_str());
    named.insert(target.clone());
    if !leaving.adopted && owner_may_be_running(leaving.owner.as_ref()) {
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
    match what_stands_at(target.as_path(), recorded) {
        Standing::Gone => {
            debug(TAG, format!("nothing of a removal's stands at {}, so its leaving is finished", target.display()).as_str());
            forget_the_leaving(common_dir, id);
            forget_the_report(common_dir, target.as_path());
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
            report_once(
                common_dir,
                target.as_path(),
                format!(
                    "{SAID} {named} is where a removal renamed the subvolume of a worktree aside, and it cannot be looked at ({reason}), so no delete of it was queued; `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
                    named = target.display()
                ),
                &mut answer.said,
            );
            return;
        }
        Standing::TheSubvolumeItNames => {}
    }
    queue_the_delete_of_one_standing_leaving(common_dir, &leaving, target.as_path(), owed, answer);
}

/// What one requeue of a leaving came to.
/// Why:
///   Three paths queue a delete again (the deleting look, the scan of a leftover and the hand-over's repair),
///  and each words the three endings differently,
///  so they share the act and not the words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Attempted = { kind: 'queued' } | { kind: 'spent' } | { kind: 'uncounted'; reason: string } | { kind: 'failed'; reason: string };
/// ```
pub(crate) enum Attempted {
    /// The delete was queued, and its attempt counted.
    Queued,
    /// The delete was queued, and the attempts could not be counted, with the reason a person reads.
    QueuedWithoutACount(String),
    /// Every attempt this leaving may cost is taken or made.
    Spent,
    /// The delete could not be queued, and the attempt it would have cost was given back.
    Failed(String),
}

/// What:
///  Queue the delete of one leaving as its next attempt:
///  take a slot before the act,
///  queue the delete,
///  and mark the slot made,
///  or give it back when the delete could not be queued.
/// Why:
///   A slot taken before the act is what two commands cannot both pass,
///  and a slot given back on failure is what keeps a failed write from costing an attempt.
///  A command killed between the slot and the act leaves the slot to the next command,
///  which takes it over and makes the attempt
/// (`worktree_removal_leaving_attempts.rs`).
///   A count that cannot be read is not a count:
///  when the attempt slots cannot be read or written,
///  the delete is queued anyway and the person reads what could not be read,
///  because a subvolume nothing will delete is the defect this look exists to end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queued = await queueTheDeleteAsTheNextAttempt(commonDir, leaving); // Queued | QueuedWithoutACount | Spent | Failed
/// ```
pub(crate) fn queue_the_delete_as_the_next_attempt(common_dir: &Path, leaving: &Leaving) -> Attempted {
    let id: String = leaving.item.id();
    let owner: String = match crate::process_identity::current_birth_identity() {
        Ok(identity) => format!("{}:{identity}", std::process::id()),
        Err(error) => return Attempted::Failed(format!("this command cannot name itself ({error})")),
    };
    let slot: Slot = match crate::worktree_removal_leaving_attempts::take_an_attempt(common_dir, id.as_str(), owner.as_str()) {
        Attempt::Took(slot) => slot,
        Attempt::Spent => return Attempted::Spent,
        Attempt::Unreadable(reason) => {
            return match queue_the_delete_again(common_dir, leaving) {
                Ok(()) => Attempted::QueuedWithoutACount(reason),
                Err(failure) => Attempted::Failed(failure),
            };
        }
    };
    // What: The step a control can kill a command at between the slot it took and the delete it queues.
    // Why:  The next command must still queue the delete, whatever the killed one spent.
    #[cfg(test)]
    crate::worktree_attempt_budget_test_support::reached(crate::worktree_attempt_budget_test_support::BEFORE_THE_DELETE_IS_QUEUED);
    if let Err(reason) = queue_the_delete_again(common_dir, leaving) {
        crate::worktree_removal_leaving_attempts::release_the_attempt(&slot);
        return Attempted::Failed(reason);
    }
    crate::worktree_removal_leaving_attempts::mark_the_attempt_made(&slot);
    return Attempted::Queued;
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
/// if (!consumeAnAttempt(commonDir, leaving.item.id())) { answer.said.push(spentWords); return; }
/// const queued = queueTheDeleteAgain(commonDir, leaving); if (queued.error) { answer.said.push(cannotQueueWords(queued.error)); return; }
/// answer.queued = true;
/// ```
fn queue_the_delete_of_one_standing_leaving(common_dir: &Path, leaving: &Leaving, target: &Path, owed: &HashSet<String>, answer: &mut Leavings) {
    // A record a release before the branch field wrote stands under the older name, and the work it owes is the same.
    if crate::worktree_removal_leaving_that_outlives_its_record::the_ids_of(&leaving.item).iter().any(|id| { return owed.contains(id.as_str()) }) {
        wake_a_pending_delete(common_dir, leaving, target, answer);
        return;
    }
    // What: The steps a command can be killed at between the proof above and the delete it queues.
    // Why:  A control kills one command at each of them and proves the next command still queues the delete.
    #[cfg(test)]
    crate::worktree_attempt_budget_test_support::reached(crate::worktree_attempt_budget_test_support::AFTER_THE_PROOF);
    match queue_the_delete_as_the_next_attempt(common_dir, leaving) {
        Attempted::Queued => answer.queued = true,
        // What: The delete is queued and nothing counted it, so the person reads what could not be read.
        // Why:  The count was the only reason to hold the delete back, and a count that cannot be read is not a count.
        Attempted::QueuedWithoutACount(reason) => {
            answer.queued = true;
            report_once(
                common_dir,
                target,
                format!(
                    "{SAID} the subvolume a removed worktree was renamed to at {named} still stands there, and its delete is queued again while nothing counts how many times this program has asked for it ({reason}), so `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
                    named = target.display()
                ),
                &mut answer.said,
            );
        }
        Attempted::Spent => answer.said.push(attempts_are_spent_line(common_dir, target, leaving.item.id().as_str())),
        Attempted::Failed(reason) => answer.said.push(format!(
            "{SAID} the subvolume a removed worktree was renamed to at {named} still stands there, and the delete that would remove it cannot be queued ({reason}), so `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
            named = target.display()
        )),
    }
}

/// What:
///  Ask the service for the round that already owes one leaving's delete,
///  counting the ask against the attempts a record may cost.
/// Why:
///   A command killed between queueing a delete and waking the service leaves a record nothing starts a round for,
///  and no later command would ever wake it if only a newly queued delete were told.
///  Every ask takes one attempt slot,
///  so a service that cannot run costs a bounded number of wakes and not one on every command of the repository.
///  Once those are spent the person reads the directory and the command that removes it by hand.
///   When the slots cannot be read or written,
///  the ask is made anyway and nothing counts it:
///  the delete a round already owes must not wait for a count nobody can read,
///  and the person reads which name stood in the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = takeAnAttempt(commonDir, leaving.item.id(), owner);
/// if (slot.kind === 'spent') return said.push(attemptsAreSpentLine(commonDir, target, leaving.item.id()));
/// answer.queued = true; if (slot.kind === 'took') answer.asks.push(slot.slot);
/// ```
pub(crate) fn wake_a_pending_delete(common_dir: &Path, leaving: &Leaving, target: &Path, answer: &mut Leavings) {
    let id: String = leaving.item.id();
    let owner: String = match crate::process_identity::current_birth_identity() {
        Ok(identity) => format!("{}:{identity}", std::process::id()),
        Err(error) => {
            answer.said.push(format!(
                "{SAID} the subvolume a removed worktree was renamed to at {named} still stands there, and this command cannot name itself ({error}), so it does not ask the service for it; `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
                named = target.display()
            ));
            return;
        }
    };
    match crate::worktree_removal_leaving_attempts::take_an_attempt(common_dir, id.as_str(), owner.as_str()) {
        Attempt::Took(slot) => {
            debug(TAG, format!("a round of the service owes the delete of {}, so it is told again and nothing is queued", target.display()).as_str());
            answer.queued = true;
            // What: The slot this ask took, to be marked once the wake it is part of is made.
            // Why:  A command killed between the slot and the wake leaves the slot to the next command, which takes it over.
            answer.asks.push(slot);
        }
        Attempt::Spent => answer.said.push(attempts_are_spent_line(common_dir, target, id.as_str())),
        Attempt::Unreadable(reason) => {
            answer.queued = true;
            report_once(
                common_dir,
                target,
                format!(
                    "{SAID} the subvolume a removed worktree was renamed to at {named} still stands there, and the commit service is asked for its delete while nothing counts how many times this program has asked ({reason}), so `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
                    named = target.display()
                ),
                &mut answer.said,
            );
        }
    }
}

/// What:
///  The line a person reads once the attempts a leaving may cost have run out.
/// Why:
///   The directory is theirs to remove then,
///  and the line names it,
///  where the service's reason is,
///  and the command that takes the subvolume,
///  so the space comes back without this program queueing the same work on every command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = `${SAID} the subvolume a removed worktree was renamed to at ${path} is still there after ${tries} more attempts to delete it, ...`;
/// ```
pub(crate) fn attempts_are_spent_line(common_dir: &Path, target: &Path, id: &str) -> String {
    let taken: u32 = crate::worktree_removal_leaving_attempts::attempts_taken(common_dir, id);
    return format!(
        "{SAID} the subvolume a removed worktree was renamed to at {named} is still there after {taken} attempts by commands of this repository to have the commit service delete it, so this command asks for no more; `journalctl --identifier=monochromatic-git-policy-cli` holds what the service's own tries came to, and `btrfs subvolume delete {named}` removes the subvolume by hand and frees the space it holds.",
        named = target.display()
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_leavings_a_command_queues_again_tests.rs"]
mod tests;
