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

/// What:
///  A repository whose worktrees stand in one directory,
///  which is the directory this program renames a removed worktree's subvolume into,
///  and a common directory no record of a removal has written anything to.
/// Why:
///   A leftover no record names can only be found where a removal renames,
///  and the repository's own registrations are what names that directory,
///  so the control writes one the way Git does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('leaving-by-name'); // { root, common, place }
/// ```
struct Place {
    /// The fixture root the control removes when it ends.
    root: PathBuf,
    /// The common Git directory,
    /// which is where the queue and the reports live.
    common: PathBuf,
    /// The directory the repository's worktrees stand in,
    /// which is where a removal renames a subvolume aside.
    place: PathBuf,
}

/// The name a person's or another program's directory takes in the controls below,
/// which is the shape a removal gives a subvolume and is never enough proof on its own.
const A_LEAVING_SHAPE: &str = "cli-git-deleting-";

/// The fixture of one repository whose worktree stands beside the place a removal renames into.
fn a_place_a_removal_renames_into(name: &str) -> Option<Place> {
    let root: PathBuf = btrfs_fixture(name)?;
    let place: PathBuf = root.join("worktrees");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(place.as_path()).expect("the directory the worktrees stand in");
    std::fs::create_dir_all(common.join("worktrees").join("one").as_path()).expect("a registration");
    let checkout: PathBuf = place.join("topic");
    std::fs::create_dir(checkout.as_path()).expect("a worktree of the repository");
    std::fs::write(common.join("worktrees").join("one").join("gitdir").as_path(), format!("{}/.git\n", checkout.display()).as_bytes()).expect("the registration's back-link");
    return Some(Place { root, common, place });
}

/// The decimal id in the sixteen hexadecimal digits of a leaving name.
fn name_of_the_id(id: u64) -> String {
    return format!("{id:016x}");
}

/// What:
///  A subvolume standing where a removal renames with a name no record names is queued for its delete,
///  and the round that runs it leaves nothing of the person's files behind.
/// Why:
///  This is the state a release before the leaving record left:
///  the rename happened, the queue record was settled by a round that looked too early,
///  and nothing on disk names the directory any more.
///  The name is the subvolume's own id, which only this program writes,
///  so the look can prove what it found and queue a delete of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('leaving-by-name-found');
/// const [id, hidden] = aSubvolumeNamed(place.place, nameOfTheId(someId));
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(true); expect(answer.said).toEqual([]);
/// expect(exists(pendingOf(place, id))).toBe(true);
/// expect(deleteSubvolumeOf(removalItem(place.common, place.place, id))).toBe('done');
/// expect(exists(hidden)).toBe(false);
/// expect(queueTheLeavingsARecordNoLongerNames(place.common).queued).toBe(false);
/// ```
#[test]
fn a_leaving_no_record_names_is_queued_by_its_name_and_the_kernel_s_id() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("leaving-by-name-found") else { return };
    let made: PathBuf = place.place.join("made");
    subvolume_create_at(made.as_path()).expect("a subvolume a removal leaves");
    let held: File = File::open(made.as_path()).expect("the subvolume");
    let id: u64 = subvolume_id(&held, made.as_path()).expect("its own id");
    let hidden: PathBuf = place.place.join(format!("{A_LEAVING_SHAPE}{}", name_of_the_id(id)));
    std::fs::rename(made.as_path(), hidden.as_path()).expect("the rename a removal makes");
    std::fs::write(hidden.join("kept.txt").as_path(), b"what the worktree held\n").expect("a file of the worktree");
    assert!(leaving_records(place.common.as_path()).is_empty(), "no record of this removal survived");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(answer.queued, "the name and the kernel's id agree, so the delete is queued: {:?}", answer.said);
    assert!(answer.said.is_empty(), "and nothing is said, because the look could do what was owed: {:?}", answer.said);
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    let pending: PathBuf = state_directory(work_directory(place.common.as_path()).as_path(), PENDING).join(item.id());
    assert!(pending.is_file(), "a round finds the delete: {}", pending.display());
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the round's own delete takes the subvolume");
    assert!(!hidden.exists(), "the person's files are gone with it");
    let again: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!again.queued, "nothing is queued for a directory that is gone");
    assert!(again.said.is_empty(), "and nothing is said: {:?}", again.said);
    assert!(leaving_records(place.common.as_path()).is_empty(), "the leaving the look wrote goes with the directory");
    remove(place.root.as_path());
}

/// What:
///  A subvolume a person or another program put at a leaving name that is not its own id is never deleted,
///  and the person reads one line about it with the command that removes it,
///  once.
/// Why:
///  The name alone proves nothing:
///  only this program writes a name holding the subvolume's own id,
///  and a subvolume another id names is somebody else's,
///  so a look that deleted by name would destroy another program's volume.
///  The line stands until the person acts, so saying it on every command would be noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('leaving-by-name-another-id');
/// const [id, hidden] = aSubvolumeNamed(place.place, nameOfTheId(idOfAnotherSubvolume));
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(false); expect(answer.said.length).toBe(1);
/// expect(readFile(hidden + '/mine.txt')).toBe('another program's bytes\n');
/// expect(queueTheLeavingsARecordNoLongerNames(place.common).said).toEqual([]); // said once
/// ```
#[test]
fn a_subvolume_at_a_leaving_name_that_is_not_its_own_id_is_left_as_it_is_and_named_once() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("leaving-by-name-other-id") else { return };
    let made: PathBuf = place.place.join("made");
    subvolume_create_at(made.as_path()).expect("a subvolume of another program's");
    let held: File = File::open(made.as_path()).expect("the subvolume");
    let id: u64 = subvolume_id(&held, made.as_path()).expect("its own id");
    // The name holds another id than the one the kernel gives the directory, so nothing proves it this program's.
    let hidden: PathBuf = place.place.join(format!("{A_LEAVING_SHAPE}{}", name_of_the_id(id ^ 1)));
    std::fs::rename(made.as_path(), hidden.as_path()).expect("another program's directory takes a leaving name");
    let mine: PathBuf = hidden.join("mine.txt");
    std::fs::write(mine.as_path(), b"another program's bytes\n").expect("a file of the other program's");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!answer.queued, "a subvolume another id names is never queued for a delete");
    assert_eq!(answer.said.len(), 1, "the person reads one line about it: {:?}", answer.said);
    let words: &str = answer.said[0].as_str();
    assert!(words.contains(hidden.display().to_string().as_str()), "the line names the directory: {words}");
    assert!(words.contains(format!("`btrfs subvolume delete {}", hidden.display()).as_str()), "and the command that removes a subvolume: {words}");
    assert_eq!(std::fs::read(mine.as_path()).expect("the other program's file is still there"), b"another program's bytes\n");
    let again: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(again.said.is_empty(), "the same line is not said on the next command: {:?}", again.said);
    assert!(hidden.is_dir(), "and the directory stands exactly as it was");
    snapshot_destroy_in(&File::open(place.place.as_path()).expect("the place"), hidden.file_name().expect("a name"), hidden.as_path()).expect("the control removes what it made");
    remove(place.root.as_path());
}

/// What:
///  A plain directory at a leaving name is never deleted and is named once,
///  because a rename of this program's moves a subvolume and nothing else.
/// Why:
///  The name is what a person reads first, so a person's own directory can carry it,
///  and a look that acted on the name would empty it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('leaving-by-name-plain');
/// const [id] = aSubvolumeNamed(place.place, nameOfTheId(anId)); destroyItByHand();
/// mkdir(`${place.place}/cli-git-deleting-${nameOfTheId(id)}`); write(kept.txt);
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(false); expect(answer.said.length).toBe(1); expect(readFile(kept.txt))....
/// ```
#[test]
fn a_plain_directory_at_a_leaving_name_is_left_as_it_is_and_named_once() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("leaving-by-name-plain") else { return };
    let made: PathBuf = place.place.join("made");
    subvolume_create_at(made.as_path()).expect("a subvolume this program once left");
    let id: u64 = subvolume_id(&File::open(made.as_path()).expect("the subvolume"), made.as_path()).expect("its own id");
    snapshot_destroy_in(&File::open(place.place.as_path()).expect("the place"), std::ffi::OsStr::new("made"), made.as_path()).expect("the subvolume goes and a plain directory can take its name");
    let plain: PathBuf = place.place.join(format!("{A_LEAVING_SHAPE}{}", name_of_the_id(id)));
    std::fs::create_dir(plain.as_path()).expect("a person's directory at a leaving name");
    let kept: PathBuf = plain.join("kept.txt");
    std::fs::write(kept.as_path(), b"a person's bytes\n").expect("a file of the person's");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!answer.queued, "a plain directory is no leaving of this program's");
    assert_eq!(answer.said.len(), 1, "the person reads one line about it: {:?}", answer.said);
    assert!(answer.said[0].contains(plain.display().to_string().as_str()), "the line names the directory: {}", answer.said[0]);
    assert_eq!(std::fs::read(kept.as_path()).expect("the person's file is still there"), b"a person's bytes\n");
    let again: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(again.said.is_empty(), "the same line is not said again: {:?}", again.said);
    remove(place.root.as_path());
}
