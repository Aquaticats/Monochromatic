//! Controls of the record of one leaving a removal makes:
//! what it holds,
//! when it goes,
//! and the queueing of a delete that no round owes any more.

use super::*;
use crate::claim_time::ClaimTime;
use crate::test_support::{fixture, remove};
use crate::work_item::{Claimed, WorkKind, claim_next_of, enqueue, settle};
use crate::worktree_remove_hand_over::{deleting_name, removal_item};

/// What:
///  A repository's common directory and one item of a renamed subvolume inside a fresh fixture root.
/// Why:
///  A leaving names a path below a directory that holds worktrees and stands in the queue of a common directory,
/// and
///  nothing else about either is asked here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = aRemovalOfASubvolume('leaving-name'); // { common, parent, item }
/// ```
struct Removal {
    /// The root the control removes when it ends.
    root: PathBuf,
    /// The common Git directory,
    /// which is where every record of these goes.
    common: PathBuf,
    /// The directory that holds the removed worktree and the name it was renamed to.
    parent: PathBuf,
    /// The record of the delete.
    item: WorkItem,
}

fn a_removal_of_a_subvolume(name: &str) -> Removal {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let parent: PathBuf = root.join("worktrees");
    std::fs::create_dir(parent.as_path()).expect("the directory that holds worktrees");
    let item: WorkItem = removal_item(common.as_path(), parent.as_path(), 0x2a);
    return Removal { root, common, parent, item };
}

/// A leaving that was written by a command that has ended,
/// as a removal killed after its rename leaves it.
fn a_leaving_of_a_removal_that_ended(removal: &Removal) -> Leaving {
    return record_a_leaving_with_owner(removal.common.as_path(), &removal.item, Some("1:ended")).expect("the leaving is recorded");
}

/// The bytes of the one record a folder of leavings holds,
/// or nothing when it holds none.
fn the_leaving_record(removal: &Removal) -> Option<Vec<u8>> {
    return std::fs::read(record_path(removal.common.as_path(), removal.item.id().as_str())).ok();
}

/// What:
///  The record of a leaving names the directory the rename makes,
///  the subvolume id it holds,
///  and the command that wrote it,
///  and the count of queueings starts at nought.
/// Why:
///  These four are the whole of what a later command needs:
/// which path,
/// which subvolume,
/// whether the command is still running,
/// and how many attempts it has bought.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leaving = await recordALeaving(common, item);
/// expect(leaving).toEqual({ owner: `${pid}:${birth}`, tries: 0, item });
/// expect(readTheLeaving(bytes)).toEqual(leaving);
/// ```
#[test]
fn a_leaving_names_the_path_the_subvolume_and_the_command_that_renamed_it() {
    let removal: Removal = a_removal_of_a_subvolume("leaving-names");
    let recorded: Leaving = record_a_leaving(removal.common.as_path(), &removal.item).expect("the leaving is recorded");
    assert!(recorded.owner.is_some(), "the command names itself, so a later command knows it may still be running");
    assert_eq!(recorded.tries, 0, "no command has queued a delete for it yet");
    assert_eq!(recorded.item, removal.item, "the record holds the delete's own item");
    assert_eq!(removal.item.parent, removal.parent.join(deleting_name(0x2a)).display().to_string(), "the record names the place the rename makes");
    let bytes: Vec<u8> = the_leaving_record(&removal).expect("the record is written");
    let read: Leaving = read_the_leaving(bytes.as_slice()).expect("a record this build wrote is one it reads");
    assert_eq!(read.owner, recorded.owner, "the command is read back");
    assert_eq!(read.tries, 0, "and the count with it");
    assert_eq!(read.item, removal.item, "and the place it names, byte for byte");
    remove(removal.root.as_path());
}

/// What:
///  A record that is no leaving is refused,
///  so a command never queues a delete for a path a stranger wrote there.
/// Why:
///  The folder is a directory in the Git directory,
///  and nothing stops another program from writing in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => readTheLeaving('not a record')).toThrow();
/// expect(() => readTheLeaving('tries=lots\n{}')).toThrow();
/// ```
#[test]
fn a_record_that_is_no_leaving_is_refused() {
    assert!(read_the_leaving(b"this is not a record\n").is_err(), "a line that is no item names nothing");
    assert!(read_the_leaving(b"tries=lots\n").is_err(), "a count that is no count names nothing");
    assert!(read_the_leaving(b"owner=1:ended\ntries=2\n{}\n").is_err(), "an item without its fields names nothing");
}

/// What:
///  The record of a leaving goes when nothing of this program's stands where it named,
///  and the folder goes with the last one.
/// Why:
///  A record that outlived its directory would make every command look at a path that has nothing to do with the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordALeaving(common, item); forgetTheLeaving(common, item.id());
/// expect(leavingRecords(common)).toEqual([]); expect(exists(join(common, LEAVINGS_FOLDER))).toBe(false);
/// ```
#[test]
fn a_leaving_that_is_finished_is_forgotten_and_the_folder_goes_with_it() {
    let removal: Removal = a_removal_of_a_subvolume("leaving-forgotten");
    a_leaving_of_a_removal_that_ended(&removal);
    assert_eq!(leaving_records(removal.common.as_path()).len(), 1, "the record is there to go");
    forget_the_leaving(removal.common.as_path(), removal.item.id().as_str());
    assert!(leaving_records(removal.common.as_path()).is_empty(), "the record is gone");
    assert!(!removal.common.join(LEAVINGS_FOLDER).exists(), "and the folder with the last one");
    forget_the_leaving(removal.common.as_path(), removal.item.id().as_str());
    assert!(leaving_records(removal.common.as_path()).is_empty(), "forgetting a record that is gone changes nothing");
    remove(removal.root.as_path());
}

/// What:
///  The delete of a leaving whose round gave up is queued again from the record that round left,
///  and the queue owes one delete of that place rather than two.
/// Why:
///  Nothing scans `failed`,
///  so without this a leaving whose destroy the kernel refused stands for ever with nothing naming its delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(work, item); claim; settle(work, claimed, FAILED);
/// await queueTheDeleteAgain(common, leaving);
/// expect(pending(work, item.id())).toBe(true); expect(exists(failed(work, item.id()))).toBe(false);
/// expect(readTheLeaving(bytes).tries).toBe(1);
/// ```
#[test]
fn the_delete_of_a_leaving_a_round_gave_up_on_is_queued_again() {
    let removal: Removal = a_removal_of_a_subvolume("leaving-gave-up");
    let leaving: Leaving = a_leaving_of_a_removal_that_ended(&removal);
    let work: PathBuf = work_directory(removal.common.as_path());
    enqueue(work.as_path(), &removal.item).expect("the delete is queued");
    let claimed: Claimed = claim_next_of(work.as_path(), &ClaimTime::read(), &[WorkKind::SubvolumeDelete]).expect("a claim").expect("the record is claimable");
    settle(work.as_path(), &claimed, FAILED).expect("the round gives up on it");
    assert!(!leaving_records(removal.common.as_path()).is_empty(), "the leaving record outlives the round that gave up");
    queue_the_delete_again(removal.common.as_path(), &leaving).expect("the delete is queued again");
    assert!(state_directory(work.as_path(), PENDING).join(removal.item.id()).is_file(), "a round finds it again");
    assert!(!state_directory(work.as_path(), FAILED).join(removal.item.id()).exists(), "and the record it was is gone, so the queue owes one delete");
    let bytes: Vec<u8> = the_leaving_record(&removal).expect("the record is still there");
    assert_eq!(read_the_leaving(bytes.as_slice()).expect("its own record").tries, 1, "the count of queueings travels with it");
    remove(removal.root.as_path());
}

/// What:
///  The delete of a leaving a round settled in `done` before the rename is queued again whole,
///  and the record that claims a delete that did not happen is taken away with it.
/// Why:
///  A `done` record is one no round reads again,
///  and its own bytes are a delete that never ran:
///  the queue must owe the delete once,
/// from a record a round will find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(work, item); claim; settle(work, claimed, DONE);
/// await queueTheDeleteAgain(common, leaving);
/// expect(pending(work, item.id())).toBe(true); expect(exists(done(work, item.id()))).toBe(false);
/// ```
#[test]
fn the_delete_of_a_leaving_settled_before_the_rename_is_queued_again_whole() {
    let removal: Removal = a_removal_of_a_subvolume("leaving-settled-done");
    let leaving: Leaving = a_leaving_of_a_removal_that_ended(&removal);
    let work: PathBuf = work_directory(removal.common.as_path());
    enqueue(work.as_path(), &removal.item).expect("the delete is queued");
    let claimed: Claimed = claim_next_of(work.as_path(), &ClaimTime::read(), &[WorkKind::SubvolumeDelete]).expect("a claim").expect("the record is claimable");
    settle(work.as_path(), &claimed, DONE).expect("the round settles a delete it found nothing for");
    queue_the_delete_again(removal.common.as_path(), &leaving).expect("the delete is queued again");
    let pending: PathBuf = state_directory(work.as_path(), PENDING).join(removal.item.id());
    assert!(pending.is_file(), "a round finds the delete again: {}", pending.display());
    assert!(!state_directory(work.as_path(), DONE).join(removal.item.id()).exists(), "the settled record is taken away, so one delete is owed");
    let again: WorkItem = WorkItem::decode(std::fs::read_to_string(pending.as_path()).expect("the queued record").as_str()).expect("a record a round reads");
    assert_eq!(again, removal.item, "the queued record is the leaving's own item");
    let claimed_again: Claimed = claim_next_of(work.as_path(), &ClaimTime::read(), &[WorkKind::SubvolumeDelete]).expect("a claim").expect("the queued delete is claimable");
    assert_eq!(claimed_again.item.parent, removal.item.parent, "and it names the place the rename made");
    remove(removal.root.as_path());
}

/// What:
///  A leaving whose path the rename never made is not a delete of anything a person has.
/// Why:
///  The suite hands two subvolumes one id on two filesystems a record each,
///  so a record is keyed by its path as well as its id;
///  this is the control that the folder of leavings keys them the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = await recordALeaving(common, removalItem(common, one, 257));
/// const second = await recordALeaving(common, removalItem(common, two, 257));
/// expect(first.item.id()).not.toBe(second.item.id()); expect(leavingRecords(common).length).toBe(2);
/// ```
#[test]
fn two_directories_that_renamed_subvolumes_of_one_id_are_two_leavings() {
    let root: PathBuf = fixture("leaving-two-parents");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let first_parent: PathBuf = root.join("first");
    let second_parent: PathBuf = root.join("second");
    std::fs::create_dir(first_parent.as_path()).expect("the first directory");
    std::fs::create_dir(second_parent.as_path()).expect("the second directory");
    let first: Leaving = a_leaving_of_a_removal_that_ended(&Removal {
        root: root.clone(),
        common: common.clone(),
        parent: first_parent.clone(),
        item: removal_item(common.as_path(), first_parent.as_path(), 257),
    });
    let second: Leaving = a_leaving_of_a_removal_that_ended(&Removal {
        root: root.clone(),
        common: common.clone(),
        parent: second_parent.clone(),
        item: removal_item(common.as_path(), second_parent.as_path(), 257),
    });
    assert_ne!(first.item.id(), second.item.id(), "a subvolume id is unique on one filesystem only, so the directory that renamed it is part of the name");
    let records: Vec<(String, PathBuf)> = leaving_records(common.as_path());
    assert_eq!(records.len(), 2, "both keep their own record: {records:?}");
    let names: Vec<&str> = records.iter().map(|(id, _path)| { return id.as_str(); }).collect();
    assert!(names.contains(&first.item.id().as_str()), "the first is named by its own id: {names:?}");
    assert!(names.contains(&second.item.id().as_str()), "and the second by its: {names:?}");
    remove(root.as_path());
}

/// What:
///  One removal has one record,
///  however many times a command writes it,
///  so two removals are two records and a repeated write changes nothing.
/// Why:
///  The name is the item's id,
///  which holds the subvolume and the directory that renamed it,
///  so a second write of the same removal finds and replaces the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordALeaving(common, item); await recordALeaving(common, item);
/// expect(leavingRecords(common).length).toBe(1);
/// ```
#[test]
fn the_leaving_of_one_removal_is_named_by_one_id() {
    let removal: Removal = a_removal_of_a_subvolume("leaving-one-id");
    let first: Leaving = a_leaving_of_a_removal_that_ended(&removal);
    let second: Leaving = a_leaving_of_a_removal_that_ended(&removal);
    assert_eq!(first.item.id(), second.item.id(), "one removal is one record, however many times a command writes it");
    assert_eq!(leaving_records(removal.common.as_path()).len(), 1, "and the folder holds one");
    remove(removal.root.as_path());
}

/// What:
///  A subvolume a removal moves out of a tree for the service is named by a leaving record before the move,
///  so a round that settled its delete before the move leaves nothing unnamed.
/// Why:
///  The pools ahead of an add and a pending deletion inside a worktree are subvolumes the kernel refuses to destroy
///  through,
///  and a removal moves each one out with a record of its delete
///  (`worktree_removal_leaves_a_subvolume_for_the_service.rs`).
///  That record is settled by the first round that looks at it,
///  and a round that looked before the move finds no directory,
///  so without a record of its own the subvolume would stand under a name nothing deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await leaveASubvolumeForTheService(commonDir, from, parent);
/// expect(exists(left)).toBe(true); expect(leavingRecords(commonDir).length).toBe(1);
/// ```
#[test]
fn a_subvolume_a_removal_moves_out_of_a_tree_is_named_by_a_leaving_record() {
    let Some(root): Option<PathBuf> = crate::fs_btrfs_test_support::btrfs_fixture("leaving-moved-out-of-a-tree") else { return };
    let made: PathBuf = root.join("worktree");
    crate::fs_btrfs_ioctl::subvolume_create_at(made.as_path()).expect("the subvolume of a worktree");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let pool: PathBuf = root.join("pool");
    std::fs::create_dir(pool.as_path()).expect("the directory the subvolume is moved into");
    let left: PathBuf = crate::worktree_removal_leaves_a_subvolume_for_the_service::leave_a_subvolume_for_the_service(common.as_path(), made.as_path(), pool.as_path()).expect("the subvolume is moved out");
    assert!(left.is_dir(), "the subvolume stands where the move put it: {}", left.display());
    let records: Vec<(String, PathBuf)> = leaving_records(common.as_path());
    assert_eq!(records.len(), 1, "a leaving record names it: {records:?}");
    assert!(records[0].0.starts_with("subvolume-delete-"), "named by the delete it is: {}", records[0].0);
    let bytes: Vec<u8> = std::fs::read(records[0].1.as_path()).expect("the record");
    let recorded: Leaving = read_the_leaving(bytes.as_slice()).expect("a record this build wrote");
    assert_eq!(recorded.item.parent, left.display().to_string(), "and it names the place the move made");
    remove(root.as_path());
}
