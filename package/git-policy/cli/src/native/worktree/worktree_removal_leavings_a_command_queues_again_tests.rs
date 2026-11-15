//! Controls of the look every command makes at the subvolumes its removals renamed aside:
//! the leaving a killed removal left is queued again,
//! the one a command may still be working on is left alone,
//! a directory that is not the subvolume the record names is never queued for a delete,
//! and a delete the kernel keeps refusing ends in words a person acts on.

use super::*;
use crate::claim_time::ClaimTime;
use crate::fs_btrfs_ioctl::{snapshot_destroy_in, subvolume_create_at, subvolume_id};
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::service_work::Verdict;
use crate::test_support::remove;
use crate::work_item::{Claimed, DONE, FAILED, PENDING, WorkItem, WorkKind, claim_next_of, enqueue, settle, state_directory};
use crate::worktree_remove_hand_over::{delete_subvolume_of, removal_item};
use crate::worktree_removal_leaving_that_outlives_its_record::{LEAVINGS_FOLDER, leaving_records, record_a_leaving, record_a_leaving_with_owner};

/// What:
///  A subvolume a removal renamed aside,
///  and the records a killed command leaves around it:
///  the leaving written before the rename,
///  and the delete's queue record,
///  which the control settles or leaves as the state it builds needs.
/// Why:
///  The rename is the one step that cannot be undone,
///  so a control of the orders a kill can leave starts from exactly the place the killed command left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const killed = aRemovalKilledAfterItsRename('leaving-killed'); // { root, common, hidden, item }
/// ```
struct Killed {
    /// The fixture root the control removes when it ends.
    root: PathBuf,
    /// The common Git directory,
    /// which is where the queue and the leaving records are.
    common: PathBuf,
    /// The directory the rename made,
    /// which is where the person's files stand.
    hidden: PathBuf,
    /// The record of the delete.
    item: WorkItem,
}

/// The fixture of one removal killed between its rename and everything after it,
/// or nothing when this run acknowledged that it has no btrfs root.
fn a_removal_killed_after_its_rename(name: &str) -> Option<Killed> {
    let root: PathBuf = btrfs_fixture(name)?;
    let made: PathBuf = root.join("worktree");
    subvolume_create_at(made.as_path()).expect("the subvolume of a worktree");
    let held: File = File::open(made.as_path()).expect("the subvolume");
    let id: u64 = subvolume_id(&held, made.as_path()).expect("its own id");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let item: WorkItem = removal_item(common.as_path(), root.as_path(), id);
    let hidden: PathBuf = PathBuf::from(item.parent.as_str());
    std::fs::rename(made.as_path(), hidden.as_path()).expect("the rename a killed removal made");
    std::fs::write(hidden.join("kept.txt").as_path(), b"what the worktree held\n").expect("a file of the worktree");
    return Some(Killed { root, common, hidden, item });
}

/// The queue of one fixture's repository.
fn work_of(killed: &Killed) -> PathBuf {
    return work_directory(killed.common.as_path());
}

/// The pending record of the fixture's delete,
/// when a round would find one.
fn pending_of(killed: &Killed) -> PathBuf {
    return state_directory(work_of(killed).as_path(), PENDING).join(killed.item.id());
}

/// What:
///  A removal killed after its rename leaves a directory its queue record no longer names,
///  and the next command of the repository queues that delete again and the subvolume goes.
/// Why:
///  The record is written before the rename,
///  so a round of the commit service can claim it,
///  look while the directory still stands at its old name,
///  and settle the delete as done
///  (`worktree_remove_hand_over.rs`,
/// `a_record_is_queued`).
///  The command's own repair is the next thing that runs,
///  and a command killed there leaves the state this control builds:
///  the directory stands,
///  and nothing in the queue owes its delete.
///  The leaving record written before the rename is what the next command finds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const killed = aRemovalKilledAfterItsRename('leaving-killed');
/// settleTheQueuedDelete(killed, DONE);
/// expect(exists(pendingOf(killed))).toBe(false);
/// const answer = queueTheLeavingsARecordNoLongerNames(killed.common);
/// expect(answer.queued).toBe(true); expect(exists(pendingOf(killed))).toBe(true);
/// expect(deleteSubvolumeOf(killed.item)).toBe('done'); expect(exists(killed.hidden)).toBe(false);
/// expect(queueTheLeavingsARecordNoLongerNames(killed.common)).toEqual({ queued: false, said: [] });
/// ```
#[test]
fn a_leaving_a_killed_removal_left_is_queued_again_by_the_next_command() {
    let Some(killed): Option<Killed> = a_removal_killed_after_its_rename("leaving-killed") else { return };
    record_a_leaving_with_owner(killed.common.as_path(), &killed.item, Some("1:ended")).expect("the record the killed command wrote before its rename");
    enqueue(work_of(&killed).as_path(), &killed.item).expect("the delete the killed command queued");
    let claimed: Claimed = claim_next_of(work_of(&killed).as_path(), &ClaimTime::read(), &[WorkKind::SubvolumeDelete]).expect("a claim").expect("the record is claimable");
    settle(work_of(&killed).as_path(), &claimed, DONE).expect("a round looked before the rename and settled a delete it found nothing for");
    assert!(!pending_of(&killed).is_file(), "no round owes this delete, which is the state that leaves the directory unnamed");
    assert!(killed.hidden.is_dir(), "the person's files stand where the rename put them");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(killed.common.as_path());
    assert!(answer.queued, "the next command queues the delete again: {:?}", answer.said);
    assert!(answer.said.is_empty(), "and says nothing, because it could do what was owed: {:?}", answer.said);
    assert!(pending_of(&killed).is_file(), "a round finds the delete: {}", pending_of(&killed).display());
    let removed: WorkItem = WorkItem::decode(std::fs::read_to_string(pending_of(&killed).as_path()).expect("the queued record").as_str()).expect("a record a round reads");
    assert_eq!(removed, killed.item, "and it names the place the rename made and the subvolume it holds");
    assert_eq!(delete_subvolume_of(&removed), Verdict::Done, "the round's own delete takes the subvolume");
    assert!(!killed.hidden.exists(), "the person's files are gone with it");
    let again: Leavings = queue_the_leavings_a_record_no_longer_names(killed.common.as_path());
    assert!(!again.queued, "nothing is queued for a directory that is gone");
    assert!(again.said.is_empty(), "and nothing is said: {:?}", again.said);
    assert!(leaving_records(killed.common.as_path()).is_empty(), "the leaving goes with the directory it named");
    remove(killed.root.as_path());
}

/// What:
///  A leaving whose command may still be running is left alone,
///  because that command has yet to say whether its rename happened.
/// Why:
///  The leaving is written before the rename,
///  so a command that decided in that window would drop the only record of a directory about to appear.
///  This is the control of the window's other side:
///  the same state with the command still there is no leaving to act on yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const killed = aRemovalKilledAfterItsRename('leaving-live-owner');
/// await recordALeaving(common, item);            // this command, which is running
/// rm(killed.hidden);                              // the rename has not happened yet
/// expect(queueTheLeavingsARecordNoLongerNames(common)).toEqual({ queued: false, said: [] });
/// expect(leavingRecords(common).length).toBe(1);  // its record is kept for it
/// ```
#[test]
fn a_leaving_whose_command_may_still_be_running_is_left_alone() {
    let Some(killed): Option<Killed> = a_removal_killed_after_its_rename("leaving-live-owner") else { return };
    record_a_leaving(killed.common.as_path(), &killed.item).expect("the record this command writes before its rename");
    // The rename has not happened yet, which is the moment the record's owner is in.
    snapshot_destroy_in(&File::open(killed.root.as_path()).expect("the fixture root"), killed.hidden.file_name().expect("a name"), killed.hidden.as_path()).expect("the directory a control removes to stand before a rename");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(killed.common.as_path());
    assert!(!answer.queued, "a command that may still rename something queues nothing");
    assert!(answer.said.is_empty(), "and says nothing: {:?}", answer.said);
    assert_eq!(leaving_records(killed.common.as_path()).len(), 1, "its own record is kept for it");
    remove(killed.root.as_path());
}

/// What:
///  A directory that is not the subvolume the record names is never queued for a delete,
///  and the person reads that it was left exactly as it is.
/// Why:
///  The id names one subvolume for good,
///  and a name names whatever stands there when the call runs,
///  so a directory another command put at that name is that command's and never this removal's to delete
///  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
/// "Step 1").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const killed = aRemovalKilledAfterItsRename('leaving-a-replacement');
/// await recordALeavingWithOwner(common, item, '1:ended');
/// moveAside(killed.hidden); subvolumeCreate(killed.hidden); write(killed.hidden + '/mine.txt');
/// const answer = queueTheLeavingsARecordNoLongerNames(common);
/// expect(answer.queued).toBe(false); expect(answer.said.length).toBe(1);
/// expect(readFile(killed.hidden + '/mine.txt')).toBe('another command's bytes\n'); expect(leavingRecords(common)).toEqual([]);
/// ```
#[test]
fn a_directory_that_is_not_the_subvolume_the_record_names_is_left_as_it_is_and_named() {
    let Some(killed): Option<Killed> = a_removal_killed_after_its_rename("leaving-a-replacement") else { return };
    record_a_leaving_with_owner(killed.common.as_path(), &killed.item, Some("1:ended")).expect("the record of the removal");
    // Another command put a subvolume of its own at the name the removal's was renamed to.
    let elsewhere: PathBuf = killed.root.join("the-original");
    std::fs::rename(killed.hidden.as_path(), elsewhere.as_path()).expect("the subvolume of the removal is moved away");
    subvolume_create_at(killed.hidden.as_path()).expect("another command's subvolume takes the name");
    let mine: PathBuf = killed.hidden.join("mine.txt");
    std::fs::write(mine.as_path(), b"another command's bytes\n").expect("a file of the other command's");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(killed.common.as_path());
    assert!(!answer.queued, "a directory that is not the record's subvolume is never queued for a delete");
    assert_eq!(answer.said.len(), 1, "the person reads one line about it: {:?}", answer.said);
    assert!(answer.said[0].contains(killed.hidden.display().to_string().as_str()), "the line names the directory: {}", answer.said[0]);
    assert!(answer.said[0].contains("btrfs subvolume delete"), "and the action that removes one: {}", answer.said[0]);
    assert_eq!(std::fs::read(mine.as_path()).expect("the other command's file is still there"), b"another command's bytes\n");
    assert!(leaving_records(killed.common.as_path()).is_empty(), "the record goes, because the leaving it named is no longer there");
    remove(killed.root.as_path());
}

/// What:
///  A leaving whose delete the kernel keeps refusing ends in one line naming the directory,
///  the journal that says why,
///  and the command that removes it,
///  and no more queueing than the attempts a record may cost.
/// Why:
///  A directory the kernel refuses every time must be the person's to remove rather than an attempt on every command,
///  and the words must be on the command they run rather than only in the service's journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const killed = aRemovalKilledAfterItsRename('leaving-attempts-spent');
/// await recordALeavingWithOwner(common, item, '1:ended');
/// for (const round of range(MAX_TRIES)) { expect(queueTheLeavings(common).queued).toBe(true); settleTheQueuedDelete(killed, FAILED); }
/// const spent = queueTheLeavings(common);
/// expect(spent.queued).toBe(false); expect(spent.said.length).toBe(1); expect(exists(pendingOf(killed))).toBe(false);
/// expect(exists(killed.hidden)).toBe(true);
/// ```
#[test]
fn a_leaving_whose_attempts_ran_out_names_the_directory_and_the_commands_that_end_it() {
    let Some(killed): Option<Killed> = a_removal_killed_after_its_rename("leaving-attempts-spent") else { return };
    record_a_leaving_with_owner(killed.common.as_path(), &killed.item, Some("1:ended")).expect("the record of the removal");
    for _round in 0..MAX_TRIES {
        let queued: Leavings = queue_the_leavings_a_record_no_longer_names(killed.common.as_path());
        assert!(queued.queued, "a leaving whose attempts are not spent is queued again: {:?}", queued.said);
        let claimed: Claimed = claim_next_of(work_of(&killed).as_path(), &ClaimTime::read(), &[WorkKind::SubvolumeDelete]).expect("a claim").expect("the queued delete is claimable");
        settle(work_of(&killed).as_path(), &claimed, FAILED).expect("a round the kernel refuses gives up on it");
    }
    let spent: Leavings = queue_the_leavings_a_record_no_longer_names(killed.common.as_path());
    assert!(!spent.queued, "a leaving whose attempts are spent is not queued again");
    assert_eq!(spent.said.len(), 1, "the person reads one line about it: {:?}", spent.said);
    let words: &str = spent.said[0].as_str();
    assert!(words.contains(killed.hidden.display().to_string().as_str()), "the line names the directory: {words}");
    assert!(words.contains("journalctl --identifier=monochromatic-git-policy-cli"), "and where the service's reason is: {words}");
    assert!(words.contains(format!("`btrfs subvolume delete {}", killed.hidden.display()).as_str()), "and the command that removes it: {words}");
    assert!(!pending_of(&killed).is_file(), "nothing is queued for it any more");
    assert!(killed.hidden.is_dir(), "the directory the words name is still there");
    remove(killed.root.as_path());
}

/// What:
///  A leaving whose directory is gone is forgotten,
///  and a repository in which no removal left anything costs one look that finds no folder.
/// Why:
///  Every command of the repository runs this,
///  so the state of an ordinary repository must be the cheapest one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(queueTheLeavingsARecordNoLongerNames(common)).toEqual({ queued: false, said: [] });
/// expect(exists(join(common, LEAVINGS_FOLDER))).toBe(false);
/// ```
#[test]
fn a_repository_in_which_no_removal_left_anything_says_nothing() {
    let root: PathBuf = crate::test_support::fixture("leaving-none");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(common.as_path());
    assert!(!answer.queued, "nothing is queued");
    assert!(answer.said.is_empty(), "and nothing is said: {:?}", answer.said);
    assert!(!common.join(LEAVINGS_FOLDER).exists(), "no folder was made either");
    remove(root.as_path());
}
