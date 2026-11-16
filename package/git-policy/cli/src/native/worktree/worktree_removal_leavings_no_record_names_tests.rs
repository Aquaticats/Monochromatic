//! Controls of the look every command makes for a subvolume a removal renamed aside and no leaving record kept:
//! the mark this program sets on a subvolume before it moves it and a record of its delete are the proofs,
//! a name alone proves nothing,
//! the main checkout's directory is no place a removal renames into,
//! an entry is opened once and never waited for,
//! a place whose name is not text is refused,
//! and a delete a round already owes is told within a bound.

use super::*;
use crate::claim_time::ClaimTime;
use crate::fs_btrfs_ioctl::{snapshot_destroy_in, subvolume_create_at, subvolume_id};
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::service_work::{MAX_TRIES, Verdict, work_directory};
use crate::test_support::remove;
use crate::work_item::{Claimed, DONE, PENDING, WorkItem, WorkKind, claim_next_of, enqueue, settle, state_directory};
use crate::worktree_remove_hand_over::{delete_subvolume_of, removal_item};
use crate::fs_beneath::open_root;
use crate::private_storage::write_private_file_unsynced;
use crate::worktree_removal_leaving_that_outlives_its_record::{leaving_records, mark_the_leaving, read_the_leaving, record_a_leaving_with_owner};
use crate::worktree_removal_leavings_a_command_queues_again::queue_the_leavings_a_record_no_longer_names;
use crate::worktree_remove_hand_over::{ToldOfHandOver, hand_over_with};
use std::sync::{Arc, Barrier};

/// What:
///  A wake that reached no service,
///  which a control of the hand-over's own records needs and never reads.
/// Why:
///   A hand-over is a record and a rename without a service,
///  and the absence of a wake is not the removal's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NobodyTold { told() { return undefined } }
/// ```
struct NobodyTold;

impl ToldOfHandOver for NobodyTold {
    fn told(&self, _common_dir: &Path) -> Result<(), String> {
        return Ok(());
    }
}

/// The name a removal gives the subvolume it renames aside,
/// which a person's directory can carry too.
const A_LEAVING_SHAPE: &str = "cli-git-deleting-";

/// The seconds this control lets a look at a place take before the test ends,
/// so an open that waits for a writer that never comes fails the control instead of holding the suite.
const PATIENCE_SECONDS: u32 = 20;

/// What:
///  A repository whose worktrees stand in one directory,
///  which is the directory a removal of one of them renames its subvolume into,
///  or a repository with no linked worktree at all.
/// Why:
///   The look is about the directories a removal of this repository renames into,
///  so a control needs the repository's own registrations, written the way Git writes them,
///  and one that needs the main checkout alone writes no registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('name'); // { root, common, place }
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

/// The sixteen hexadecimal digits a leaving name carries the subvolume id in.
fn name_of_the_id(id: u64) -> String {
    return format!("{id:016x}");
}

/// What:
///  One repository whose single linked worktree stands beside the place a removal renames into.
/// Why:
///   Every control about a leftover of a linked-worktree removal needs that registration,
///  because the look reads the registrations to know where a removal renames.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('review-main-sibling');
/// ```
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

/// What:
///  One repository whose only checkout is the main one,
///  which registers no worktree anywhere.
/// Why:
///   Git removes no main worktree,
///  so no removal of this program renames a subvolume into the directory that holds it,
///  and a control proves the look does not reach there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alone = aMainCheckoutWithNoLinkedWorktree('main-only'); // { root, common: root/main/.git, place: root }
/// ```
fn a_main_checkout_with_no_linked_worktree(name: &str) -> Option<Place> {
    let root: PathBuf = btrfs_fixture(name)?;
    let checkout: PathBuf = root.join("main");
    let common: PathBuf = checkout.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the main checkout and its common Git directory");
    let place: PathBuf = root.clone();
    return Some(Place { root, common, place });
}

/// What:
///  A subvolume under a leaving name of its own id,
///  with a file in it that no removal wrote.
/// Why:
///   Every control needs the object the look must judge,
///  and a file inside it is what a wrong answer deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place, 'kept.txt', b'held\n');
/// ```
fn a_subvolume_under_its_own_leaving_name(place: &Path, file: &str, bytes: &[u8]) -> (u64, PathBuf) {
    let made: PathBuf = place.join("made");
    subvolume_create_at(made.as_path()).expect("a subvolume");
    let held: File = File::open(made.as_path()).expect("the subvolume");
    let id: u64 = subvolume_id(&held, made.as_path()).expect("its own id");
    let hidden: PathBuf = place.join(format!("{A_LEAVING_SHAPE}{}", name_of_the_id(id)));
    std::fs::rename(made.as_path(), hidden.as_path()).expect("a name built from its own id");
    std::fs::write(hidden.join(file).as_path(), bytes).expect("a file of the person's");
    return (id, hidden);
}

/// The queue of one fixture's repository.
fn work_of(place: &Place) -> PathBuf {
    return work_directory(place.common.as_path());
}

/// Whether a round owes the delete of `item`.
fn owed(place: &Place, item: &WorkItem) -> bool {
    return state_directory(work_of(place).as_path(), PENDING).join(item.id()).is_file();
}

/// What:
///  A subvolume a person named after its own id, with no mark and no record of any removal,
///  is never deleted and is named once.
/// Why:
///  A name is no proof of who renamed a directory:
///  a person can read a subvolume's id and name their own directory after it,
///  and only the mark this program sets or a record of its delete tells the two apart
///  (`worktree_removal_leavings_no_record_names.rs`).
///  The line names the directory and the command that removes a subvolume of theirs,
///  so the space is theirs to free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('scan-no-proof');
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'mine.txt', b'mine\n');
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(false); expect(answer.said.length).toBe(1);
/// expect(readFile(`${hidden}/mine.txt`)).toEqual(b'mine\n');
/// expect(owed(place, removalItem(place.common, place.place, id))).toBe(false);
/// expect(queueTheLeavingsARecordNoLongerNames(place.common).said).toEqual([]);
/// ```
#[test]
fn a_subvolume_no_mark_and_no_record_names_is_left_as_it_is_and_named_once() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-no-proof") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "mine.txt", b"mine\n");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!answer.queued, "nothing proves it this program's, so its delete is never queued: {:?}", answer.said);
    assert_eq!(answer.said.len(), 1, "the person reads one line about it: {:?}", answer.said);
    let words: &str = answer.said[0].as_str();
    assert!(words.contains(hidden.display().to_string().as_str()), "the line names the directory: {words}");
    assert!(words.contains(format!("`btrfs subvolume delete {}", hidden.display()).as_str()), "and the command that removes it: {words}");
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    assert!(!owed(&place, &item), "no round is given work for a directory nothing proves");
    assert_eq!(std::fs::read(hidden.join("mine.txt").as_path()).expect("the person's file"), b"mine\n", "and its content is still there");
    let again: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!again.queued, "the next command queues nothing either");
    assert!(again.said.is_empty(), "and says nothing more: {:?}", again.said);
    assert!(leaving_records(place.common.as_path()).is_empty(), "no record of this program's was written for it");
    snapshot_destroy_in(&File::open(place.place.as_path()).expect("the place"), hidden.file_name().expect("a name"), hidden.as_path()).expect("the control removes its own subvolume");
    remove(place.root.as_path());
}

/// What:
///  A subvolume beside the main checkout of a repository with no linked worktree is never looked at,
///  and nothing is said about it.
/// Why:
///  Git removes no main worktree,
///  so no removal of this program renames a subvolume into the directory that holds the main checkout,
///  and a look that reached there would judge a person's directory in a place this program never writes
///  (`the_directories_a_removal_renames_inside`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alone = aMainCheckoutWithNoLinkedWorktree('scan-main-only');
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(alone.place, 'mine.txt', b'mine\n');
/// expect(queueTheLeavingsARecordNoLongerNames(alone.common)).toEqual({ queued: false, said: [] });
/// expect(readFile(`${hidden}/mine.txt`)).toEqual(b'mine\n');
/// ```
#[test]
fn a_subvolume_beside_the_main_checkout_alone_is_never_looked_at() {
    let Some(alone): Option<Place> = a_main_checkout_with_no_linked_worktree("scan-main-only") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(alone.place.as_path(), "mine.txt", b"mine\n");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(alone.common.as_path());
    assert!(!answer.queued, "no removal of this repository renames beside its main checkout");
    assert!(answer.said.is_empty(), "so nothing is said about it either: {:?}", answer.said);
    let item: WorkItem = removal_item(alone.common.as_path(), alone.place.as_path(), id);
    assert!(!owed(&alone, &item), "no round is given work for it");
    assert_eq!(std::fs::read(hidden.join("mine.txt").as_path()).expect("the person's file"), b"mine\n", "and it stands exactly as it was");
    assert!(leaving_records(alone.common.as_path()).is_empty(), "no record of this program's was written for it");
    snapshot_destroy_in(&File::open(alone.place.as_path()).expect("the place"), hidden.file_name().expect("a name"), hidden.as_path()).expect("the control removes its own subvolume");
    remove(alone.root.as_path());
}

/// What:
///  A named pipe at a leaving name is refused at once and named once,
///  so a command that runs the look never waits for a writer that need never come.
/// Why:
///  The entry is opened below the held directory as a directory and without waiting,
///  so an entry a concurrent command replaced is answered at once instead of being followed
///  (`open_directory_at_without_waiting`).
///  A look that opened the path as a file would wait here for ever,
///  and this control ends the test at its patience rather than letting the suite hang.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('scan-named-pipe');
/// mkfifo(`${place.place}/cli-git-deleting-${nameOfTheId(0)}`);
/// alarm(PATIENCE_SECONDS); const answer = queueTheLeavingsARecordNoLongerNames(place.common); alarm(0);
/// expect(answer.queued).toBe(false); expect(answer.said.length).toBe(1);
/// ```
#[test]
fn a_named_pipe_at_a_leaving_name_does_not_hold_the_command() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-named-pipe") else { return };
    let pipe: PathBuf = place.place.join(format!("{A_LEAVING_SHAPE}{}", name_of_the_id(0)));
    let made: std::process::ExitStatus = std::process::Command::new("mkfifo").arg("--").arg(pipe.as_path()).status().expect("a named pipe");
    assert!(made.success(), "the control makes a named pipe at {}", pipe.display());
    // What: The control's own patience, and why it is safe here.
    // Why: A look that waits inside an open holds this test's process until a writer comes, and a signal states the failure
    //      instead of hanging the run; every other test of the suite runs in a process of its own.
    // SAFETY: `alarm` only asks the kernel for a signal to this process; nothing is read or written.
    let _armed: libc::c_uint = unsafe { libc::alarm(PATIENCE_SECONDS) };
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    // SAFETY: as above; a second call replaces the pending alarm with none.
    let _disarmed: libc::c_uint = unsafe { libc::alarm(0) };
    assert!(!answer.queued, "a named pipe is no subvolume and its delete is never queued");
    assert_eq!(answer.said.len(), 1, "the person reads one line about it: {:?}", answer.said);
    assert!(answer.said[0].contains(pipe.display().to_string().as_str()), "the line names the entry: {}", answer.said[0]);
    assert!(answer.said[0].contains("cannot be opened as a directory"), "and says what stood in the way: {}", answer.said[0]);
    std::fs::remove_file(pipe.as_path()).expect("the control removes its own pipe");
    remove(place.root.as_path());
}

/// What:
///  A place whose directory name is not text is refused and named once,
///  and nothing is queued for it.
/// Why:
///  A record of this program names its paths in text,
///  so a path that is not text names a different directory,
///  and a delete queued for it would report success at a place that does not exist while the real subvolume stood
/// (`the_removal_of` refuses such a parent for the hand-over,
/// `say_unproved` here).
///  The person reads the directory and the command that removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = placeWhoseNameIsNotText('scan-non-text'); // worktrees-0xff
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'kept.txt', b'held\n');
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(false); expect(answer.said.length).toBe(1);
/// expect(answer.said[0]).toContain('not named by text');
/// expect(leavingRecords(place.common)).toEqual([]);
/// ```
#[test]
fn a_place_whose_name_is_not_text_is_refused_and_named_once() {
    let Some(root): Option<PathBuf> = btrfs_fixture("scan-non-text") else { return };
    let not_text: std::ffi::OsString = <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(b"worktrees-\xff".to_vec());
    let place: PathBuf = root.join(not_text);
    let common: PathBuf = root.join("common");
    std::fs::create_dir(place.as_path()).expect("a directory whose name is not text");
    std::fs::create_dir_all(common.join("worktrees").join("one").as_path()).expect("a registration");
    let checkout: PathBuf = place.join("topic");
    std::fs::create_dir(checkout.as_path()).expect("a checkout below it");
    let mut back_link: Vec<u8> = std::os::unix::ffi::OsStrExt::as_bytes(checkout.as_os_str()).to_vec();
    back_link.extend_from_slice(b"/.git\n");
    std::fs::write(common.join("worktrees").join("one").join("gitdir").as_path(), back_link.as_slice()).expect("the registration's raw bytes");
    let (_id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.as_path(), "kept.txt", b"held\n");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(common.as_path());
    assert!(!answer.queued, "no record can name a path that is not text, so nothing is queued: {:?}", answer.said);
    assert_eq!(answer.said.len(), 1, "the person reads one line about it: {:?}", answer.said);
    assert!(answer.said[0].contains("not named by text"), "and it says why nothing was queued: {}", answer.said[0]);
    assert!(answer.said[0].contains(format!("`btrfs subvolume delete {}", hidden.display()).as_str()), "and the command that removes it: {}", answer.said[0]);
    assert!(leaving_records(common.as_path()).is_empty(), "no record of this program's was written for it");
    assert_eq!(std::fs::read(hidden.join("kept.txt").as_path()).expect("the file at the real path"), b"held\n", "the real subvolume still holds its files");
    snapshot_destroy_in(&File::open(place.as_path()).expect("the place"), hidden.file_name().expect("a name"), hidden.as_path()).expect("the control removes its own subvolume");
    remove(root.as_path());
}

/// What:
///  A delete a round already owes is told again,
///  and the wakes a service that never runs costs are bounded by the attempts a record may cost.
/// Why:
///  A command killed between queueing a delete and waking the service leaves a record nothing starts a round for,
///  so a look that only told the service about a delete it had just queued would leave it pending for ever
///  (`wake_a_pending_delete`).
///  Each ask is written into the leaving's own count,
///  so a service that cannot run ends with one line naming the directory and the command that removes it,
///  instead of a wake on every command of the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'kept.txt', b'held\n');
/// enqueue(work, removalItem(place.common, place.place, id));        // the killed command's pending record
/// for (const round of range(MAX_TRIES)) expect(queueTheLeavings(place.common).queued).toBe(true);
/// const spent = queueTheLeavings(place.common); expect(spent.queued).toBe(false); expect(spent.said.length).toBe(1);
/// ```
#[test]
fn a_pending_delete_found_again_is_told_and_the_wakes_are_bounded() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-already-pending") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "kept.txt", b"held\n");
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    enqueue(work_of(&place).as_path(), &item).expect("the pending record a command left before its wake was killed");
    for round in 0..MAX_TRIES {
        let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
        assert!(answer.queued, "round {round} tells the service about the delete it already owes: {:?}", answer.said);
        assert!(answer.said.is_empty(), "and says nothing: {:?}", answer.said);
        assert!(owed(&place, &item), "the delete is still the round's to run: {}", item.id());
        assert_eq!(leaving_records(place.common.as_path()).len(), 1, "one leaving names it, written the first time");
    }
    let spent: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!spent.queued, "the attempts a record may cost are spent, so the service is not woken again");
    assert_eq!(spent.said.len(), 1, "the person reads one line about it: {:?}", spent.said);
    assert!(spent.said[0].contains(hidden.display().to_string().as_str()), "the line names the directory: {}", spent.said[0]);
    assert!(spent.said[0].contains(format!("`btrfs subvolume delete {}", hidden.display()).as_str()), "and the command that removes it: {}", spent.said[0]);
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the control takes the subvolume the words name");
    remove(place.root.as_path());
}

/// What:
///  A settled record of a release that wrote no leaving record proves its leftover:
///  the delete is queued again and the subvolume goes.
/// Why:
///  This is the state an earlier release left when a round looked at a delete before the rename and settled it done:
///  the queue never forgot the place,
///  only the leaving did,
///  and the record names the exact path and subvolume a delete acts on
/// (`a_settled_record_names`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'kept.txt', b'held\n');
/// const item = removalItem(place.common, place.place, id);
/// enqueue(work, item); settle(work, claimNext(work), 'done');          // the round that looked too early
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(true); expect(answer.said).toEqual([]);
/// expect(deleteSubvolumeOf(item)).toBe('done'); expect(exists(hidden)).toBe(false);
/// ```
#[test]
fn a_settled_record_of_this_program_proves_the_leftover_of_an_earlier_release() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-legacy-settled") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "kept.txt", b"held\n");
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    enqueue(work_of(&place).as_path(), &item).expect("the delete an earlier release queued before its rename");
    let claimed: Claimed = claim_next_of(work_of(&place).as_path(), &ClaimTime::read(), &[WorkKind::SubvolumeDelete]).expect("a claim").expect("the record is claimable");
    settle(work_of(&place).as_path(), &claimed, DONE).expect("the round looked before the rename and settled a delete it found nothing for");
    assert!(!owed(&place, &item), "no round owes it any more");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(answer.queued, "the record names the place, so its delete is queued again: {:?}", answer.said);
    assert!(answer.said.is_empty(), "and nothing is said: {:?}", answer.said);
    assert!(owed(&place, &item), "a round finds the delete: {}", item.id());
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the round's delete takes the subvolume");
    assert!(!hidden.exists(), "the person's files are gone with it");
    let after: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!after.queued, "nothing is queued for a directory that is gone");
    assert!(leaving_records(place.common.as_path()).is_empty(), "and the next look drops the leaving with the directory it named");
    remove(place.root.as_path());
}

/// What:
///  The mark this program sets before it renames a subvolume proves a leftover whose records are all gone,
///  and a mark naming another subvolume proves nothing.
/// Why:
///  The mark travels with the subvolume,
///  so it is the one proof that survives a person deleting the repository's state,
///  and a copy of the attribute onto another subvolume names an id that directory does not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'kept.txt', b'held\n');
/// await markTheLeaving(held, id);
/// expect(queueTheLeavingsARecordNoLongerNames(place.common).queued).toBe(true);
/// // and with the mark of another id, nothing is queued and one line is said
/// ```
#[test]
fn the_mark_this_program_sets_proves_a_leftover_and_another_id_proves_nothing() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-mark") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "kept.txt", b"held\n");
    let held: File = File::open(hidden.as_path()).expect("the subvolume the rename made");
    mark_the_leaving(&held, id ^ 1).expect("a mark naming a subvolume that stands elsewhere");
    let foreign: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!foreign.queued, "a mark naming another subvolume proves nothing: {:?}", foreign.said);
    assert_eq!(foreign.said.len(), 1, "and the person reads one line: {:?}", foreign.said);
    mark_the_leaving(&held, id).expect("the mark this program sets on a subvolume it renames aside");
    let proved: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(proved.queued, "the mark names this subvolume's own id, so its delete is queued: {:?}", proved.said);
    assert!(proved.said.is_empty(), "and nothing more is said: {:?}", proved.said);
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "a round's delete takes the subvolume");
    assert!(!hidden.exists(), "the person's files are gone with it");
    let after: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!after.queued, "nothing is queued for a directory that is gone");
    assert!(leaving_records(place.common.as_path()).is_empty(), "and the next look drops the leaving with the directory");
    remove(place.root.as_path());
}

/// What:
///  The leaving record a proved leftover earns names the place and the subvolume and says it was found, not renamed.
/// Why:
///  The record is what the next command reads,
///  and the owner gate that keeps it from deciding about a rename in flight must not hold a found one back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'kept.txt', b'held\n');
/// await markTheLeaving(held, id); queueTheLeavings(place.common);
/// const leaving = readTheLeaving(reads[0]);
/// expect(leaving.adopted).toBe(true); expect(leaving.owner).toBe(undefined);
/// ```
#[test]
fn the_leaving_a_found_leftover_earns_names_the_place_and_says_it_was_found() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-mark-record") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "kept.txt", b"held\n");
    let held: File = File::open(hidden.as_path()).expect("the subvolume the rename made");
    mark_the_leaving(&held, id).expect("the mark this program sets");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(answer.queued, "the marked subvolume is proved and queued: {:?}", answer.said);
    let records: Vec<(String, PathBuf)> = leaving_records(place.common.as_path());
    assert_eq!(records.len(), 1, "one leaving is written for it");
    let bytes: Vec<u8> = std::fs::read(records[0].1.as_path()).expect("the leaving record");
    let leaving = read_the_leaving(bytes.as_slice()).expect("a record a command reads");
    assert_eq!(leaving.item.parent, hidden.display().to_string(), "it names the place the name holds");
    assert_eq!(leaving.item.oid, format!("{id:040x}"), "and the subvolume the kernel made");
    assert!(leaving.adopted, "it says a command found this leaving rather than renaming into it");
    assert!(leaving.owner.is_none(), "and names no command, because none of this program wrote that name");
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the control takes the subvolume");
    remove(place.root.as_path());
}

/// What:
///  A plain directory at a leaving name is never queued for a delete and is named once,
///  because a rename of this program moves a subvolume and nothing else.
/// Why:
///  A person's own directory can carry the name,
///  and a delete that acted on a name would take its files
/// (`the_subvolume_of_the_open_entry` asks the descriptor what it is).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('scan-plain-directory');
/// mkdir(`${place.place}/cli-git-deleting-${nameOfTheId(0)}`); write('kept.txt', b'a person's bytes\n');
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(false); expect(answer.said.length).toBe(1);
/// ```
#[test]
fn a_plain_directory_at_a_leaving_name_is_left_as_it_is_and_named_once() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-plain-directory") else { return };
    let plain: PathBuf = place.place.join(format!("{A_LEAVING_SHAPE}{}", name_of_the_id(0)));
    std::fs::create_dir(plain.as_path()).expect("a person's directory at a leaving name");
    let kept: PathBuf = plain.join("kept.txt");
    std::fs::write(kept.as_path(), b"a person's bytes\n").expect("a file of the person's");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!answer.queued, "a plain directory is no leaving of this program's");
    assert_eq!(answer.said.len(), 1, "the person reads one line about it: {:?}", answer.said);
    assert!(answer.said[0].contains(plain.display().to_string().as_str()), "the line names the directory: {}", answer.said[0]);
    assert_eq!(std::fs::read(kept.as_path()).expect("the person's file is still there"), b"a person's bytes\n", "and its content is untouched");
    let again: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(again.said.is_empty(), "the same line is not said again: {:?}", again.said);
    remove(place.root.as_path());
}

/// What:
///  The place a removal of this release renames in is recorded in the common Git directory,
///  written before the rename and read beside the live registrations,
///  so a directory whose last registration went is still looked at.
/// Why:
///  The scan finds its places in the registrations of the linked worktrees that stand now,
///  and a removal is exactly what takes one away:
///  where a directory held one worktree, the command that renames its subvolume also removes the last registration,
///  and every later command would have no directory to read
/// (`the_directories_a_removal_renames_inside`).
///  What the removal knows at that moment is the directory it renames in,
///  so that is written down before the rename, as the mark is,
///  and cleared by a command that finds nothing of this program's left there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceWithTwoRegisteredWorktrees('scan-last-registration-gone');
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'kept.txt', b'held\n');
/// await settleAQueuedDelete(removalItem(place.common, place.place, id));   // the earlier release's proof
/// await handOverWith({ ...hiddenSubvolumeOfTheLastWorktree });             // this release records the place
/// rm(join(place.common, 'worktrees'));                                    // both registrations go
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(true); expect(exists(pendingOf(place, id))).toBe(true);
/// ```
#[test]
fn a_leftover_of_a_directory_whose_last_registration_went_is_found_through_the_place_its_removal_recorded() {
    let Some(root): Option<PathBuf> = btrfs_fixture("scan-last-registration-gone") else { return };
    let place: PathBuf = root.join("worktrees");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(place.as_path()).expect("the directory the worktrees stand in");
    for registration in ["one", "two"] {
        std::fs::create_dir_all(common.join("worktrees").join(registration).as_path()).expect("a registration");
    }
    let first: PathBuf = place.join("first");
    std::fs::create_dir(first.as_path()).expect("a worktree of the repository");
    std::fs::write(common.join("worktrees").join("one").join("gitdir").as_path(), format!("{}/.git\n", first.display()).as_bytes()).expect("the registration's back-link");
    // The last worktree of this directory, which the removal of this release hands over.
    let last: PathBuf = place.join("last");
    subvolume_create_at(last.as_path()).expect("the subvolume of the last worktree");
    let last_id: u64 = subvolume_id(&File::open(last.as_path()).expect("the last subvolume"), last.as_path()).expect("its own id");
    std::fs::write(common.join("worktrees").join("two").join("gitdir").as_path(), format!("{}/.git\n", last.display()).as_bytes()).expect("the registration's back-link");
    let holder: Place = Place { root: root.clone(), common: common.clone(), place: place.clone() };
    // The leftover of an earlier release: renamed aside, its delete settled done, no leaving record of any kind.
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.as_path(), "kept.txt", b"held\n");
    let leftover: WorkItem = removal_item(common.as_path(), place.as_path(), id);
    enqueue(work_of(&holder).as_path(), &leftover).expect("the delete the earlier release queued before its rename");
    let claimed: Claimed = claim_next_of(work_of(&holder).as_path(), &ClaimTime::read(), &[WorkKind::SubvolumeDelete]).expect("a claim").expect("the record is claimable");
    settle(work_of(&holder).as_path(), &claimed, DONE).expect("the round looked before the rename and settled a delete it found nothing for");
    // The removal of this release hands its own subvolume over, which is where it records the directory it renames in.
    let parent: File = open_root(place.as_path()).expect("the directory the worktrees stand in");
    hand_over_with(common.as_path(), &parent, place.as_path(), std::ffi::OsStr::new("last"), last_id, last.as_path(), std::ffi::OsStr::new("two"), &NobodyTold).expect("the last removal hands its subvolume over");
    let handed: WorkItem = removal_item(common.as_path(), place.as_path(), last_id);
    assert_eq!(delete_subvolume_of(&handed), Verdict::Done, "the service deletes the subvolume it was handed");
    // The leaving of the subvolume the service deleted is dropped here rather than by a look, because a look run while the
    // registrations still stand would adopt the earlier release's leftover, which is the state this control must not enter.
    crate::worktree_removal_leaving_that_outlives_its_record::forget_the_leaving(common.as_path(), handed.id().as_str());
    // Every removal takes its own registration away, so this directory holds none any more.
    std::fs::remove_dir_all(common.join("worktrees").as_path()).expect("the registrations of the directory go with their removals");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(common.as_path());
    assert!(answer.queued, "the directory the removal recorded is read although no worktree stands in it: {:?}", answer.said);
    assert!(owed(&holder, &leftover), "and the leftover's delete is queued again");
    assert!(hidden.is_dir(), "the control takes the subvolume itself");
    assert_eq!(delete_subvolume_of(&leftover), Verdict::Done, "the control takes the subvolume the record names");
    let cleared: Leavings = queue_the_leavings_a_record_no_longer_names(common.as_path());
    assert!(!cleared.queued, "nothing is queued once the directory holds nothing of this program's");
    remove(root.as_path());
}

/// What:
///  A marked leftover under a directory whose name is not text is refused and named once,
///  and no record is written for a path that would be another directory.
/// Why:
///  A record names its paths in text,
///  so a delete queued for a place that is not text names a directory with the replacement character in it,
///  which the service finds empty and settles as done while the real subvolume stands
/// (`the_removal_of` refuses such a parent for the hand-over,
/// `say_unproved` here).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = placeWhoseNameIsNotText('scan-marked-non-text');
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'kept.txt', b'held\n');
/// await markTheLeaving(held, id);
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(false); expect(answer.said.length).toBe(1);
/// expect(leavingRecords(place.common)).toEqual([]); expect(exists(join(work, 'pending'))).toBe(false);
/// ```
#[test]
fn a_marked_leftover_under_a_parent_whose_name_is_not_text_is_refused_and_named_once() {
    let Some(root): Option<PathBuf> = btrfs_fixture("scan-marked-non-text") else { return };
    let not_text: std::ffi::OsString = <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(b"worktrees-\xff".to_vec());
    let place: PathBuf = root.join(not_text);
    let common: PathBuf = root.join("common");
    std::fs::create_dir(place.as_path()).expect("a directory whose name is not text");
    std::fs::create_dir_all(common.join("worktrees").join("one").as_path()).expect("a registration");
    let checkout: PathBuf = place.join("topic");
    std::fs::create_dir(checkout.as_path()).expect("a checkout below it");
    let mut back_link: Vec<u8> = std::os::unix::ffi::OsStrExt::as_bytes(checkout.as_os_str()).to_vec();
    back_link.extend_from_slice(b"/.git\n");
    std::fs::write(common.join("worktrees").join("one").join("gitdir").as_path(), back_link.as_slice()).expect("the registration's raw bytes");
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.as_path(), "kept.txt", b"held\n");
    let held: File = File::open(hidden.as_path()).expect("the subvolume the rename made");
    mark_the_leaving(&held, id).expect("the mark this program sets on a subvolume it renames aside");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(common.as_path());
    assert!(!answer.queued, "a place that is not text gets no delete queued, whatever proves the subvolume: {:?}", answer.said);
    assert_eq!(answer.said.len(), 1, "the person reads one line about it: {:?}", answer.said);
    assert!(answer.said[0].contains("not named by text"), "and it says why nothing was queued: {}", answer.said[0]);
    assert!(leaving_records(common.as_path()).is_empty(), "no record of this program's was written for it");
    assert!(!state_directory(work_directory(common.as_path()).as_path(), PENDING).exists(), "and nothing was queued either");
    assert_eq!(std::fs::read(hidden.join("kept.txt").as_path()).expect("the file at the real path"), b"held\n", "the real subvolume still holds its files");
    snapshot_destroy_in(&File::open(place.as_path()).expect("the place"), hidden.file_name().expect("a name"), hidden.as_path()).expect("the control removes its own subvolume");
    remove(root.as_path());
}

/// What:
///  A queue record an earlier release named by the older id is proof of its leftover,
///  and one that is still pending is work this command wakes the service for.
/// Why:
///  `WorkItem` still reads a record named `subvolume-delete-<object>` (`id_before_the_branch`),
///  and a record a release before the branch field wrote carries that name,
///  so a look that asks the new name alone proves nothing and wakes nothing
/// (`a_settled_record_names`, `queue_the_leaving_found`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = aPlaceARemovalRenamesInto('scan-legacy-id');
/// const [id, hidden] = aSubvolumeUnderItsOwnLeavingName(place.place, 'kept.txt', b'held\n');
/// const item = removalItem(place.common, place.place, id);
/// write(join(work, 'done', item.idBeforeTheBranch()), item.encode());      // a settled record of an older release
/// expect(queueTheLeavingsARecordNoLongerNames(place.common).queued).toBe(true);
/// // and a pending one of the same shape is told
/// ```
#[test]
fn a_settled_record_an_earlier_release_named_by_the_older_id_proves_its_leftover() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-legacy-settled-id") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "kept.txt", b"held\n");
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    let older: PathBuf = state_directory(work_of(&place).as_path(), DONE).join(item.id_before_the_branch());
    std::fs::create_dir_all(state_directory(work_of(&place).as_path(), DONE).as_path()).expect("the settled state directory");
    write_private_file_unsynced(older.as_path(), item.encode().as_bytes()).expect("the settled record an older release wrote");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(answer.queued, "the record of the older name proves the leftover and its delete is queued: {:?}", answer.said);
    assert!(answer.said.is_empty(), "and nothing is said: {:?}", answer.said);
    assert!(owed(&place, &item), "a round finds the delete under the name this release writes");
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the round's delete takes the subvolume");
    assert!(!hidden.exists(), "the person's files are gone with it");
    remove(place.root.as_path());
}

/// What:
///  A pending record an earlier release named by the older id is work this command wakes the service for.
/// Why:
///  The id in the queue and the id this look derives can differ by the name a release wrote,
///  and a wake that only the new name earns leaves the older one waiting for a round nobody starts
/// (`wake_a_pending_delete`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(join(work, 'pending', item.idBeforeTheBranch()), item.encode());
/// const answer = queueTheLeavingsARecordNoLongerNames(place.common);
/// expect(answer.queued).toBe(true); expect(exists(pendingOf(place, item.id()))).toBe(false);
/// ```
#[test]
fn a_pending_record_an_earlier_release_named_by_the_older_id_is_told() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-legacy-pending-id") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "kept.txt", b"held\n");
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    let older: PathBuf = state_directory(work_of(&place).as_path(), PENDING).join(item.id_before_the_branch());
    std::fs::create_dir_all(state_directory(work_of(&place).as_path(), PENDING).as_path()).expect("the pending state directory");
    write_private_file_unsynced(older.as_path(), item.encode().as_bytes()).expect("the pending record an older release wrote");
    let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(answer.queued, "the pending delete of the older name is told, not left waiting for a wake nobody sends: {:?}", answer.said);
    assert!(answer.said.is_empty(), "and nothing is said: {:?}", answer.said);
    assert!(older.is_file(), "the older record is still the work a round owes");
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the control takes the subvolume");
    assert!(!hidden.exists(), "the person's files are gone with it");
    remove(place.root.as_path());
}

/// What:
///  Commands running at once ask for no more wakes than the attempts a record may cost,
///  and the count is taken in the one place a race cannot lose it.
/// Why:
///  Every command that finds a pending delete reads the leaving record, decides whether the attempts are spent,
///  and wakes the service. Commands that read the same count before any of them writes it all decide alike,
///  so a count written back from the record each of them read lets four commands ask for four wakes while the record
///  advances from nought to one.
///  An attempt is therefore a file a command creates exclusively (`create_new`),
///  which only one command of any number can do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [leaving] = await adoptALeaving(place, id);           // tries=0, its delete still pending
/// const barrier = new Barrier(N + 1);
/// const woke = await Promise.all(range(N).map(() => async () => {
///   const read = await readTheLeaving(recordPath);            // all read tries=0 before any writes
///   await barrier.wait();
///   return wakeAPendingDelete(common, read, hidden, answer).queued;
/// }));
/// expect(woke.filter(Boolean).length).toBe(MAX_TRIES);
/// ```
#[test]
fn commands_at_once_ask_for_no_more_wakes_than_the_attempts_a_record_may_cost() {
    let Some(place): Option<Place> = a_place_a_removal_renames_into("scan-wakes-at-once") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "kept.txt", b"held\n");
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    enqueue(work_of(&place).as_path(), &item).expect("the pending record a command left before its wake was killed");
    crate::worktree_removal_leaving_that_outlives_its_record::record_a_leaving_found_by_a_scan(place.common.as_path(), &item).expect("the leaving a look writes");
    let record: PathBuf = leaving_records(place.common.as_path()).first().expect("the leaving record").1.clone();
    // One more thread than the attempts a record may cost, so a lost count shows as an extra wake.
    let commands: usize = usize::try_from(MAX_TRIES).expect("a count of attempts") + 1;
    let barrier: Arc<Barrier> = Arc::new(Barrier::new(commands + 1));
    let woke: usize = std::thread::scope(|_scope| {
        let mut handles: Vec<std::thread::JoinHandle<bool>> = Vec::new();
        for _command in 0..commands {
            let shared: Arc<Barrier> = Arc::clone(&barrier);
            let their_common: PathBuf = place.common.clone();
            let their_record: PathBuf = record.clone();
            let their_target: PathBuf = hidden.clone();
            handles.push(std::thread::spawn(move || {
                let bytes: Vec<u8> = std::fs::read(their_record.as_path()).expect("the leaving record");
                let leaving: crate::worktree_removal_leaving_that_outlives_its_record::Leaving = read_the_leaving(bytes.as_slice()).expect("a record a command reads");
                // The barrier is what makes every command decide on the same count, which is the state a real overlap is in.
                let _together: std::sync::BarrierWaitResult = shared.wait();
                let mut answer: Leavings = Leavings { queued: false, said: Vec::new() };
                crate::worktree_removal_leavings_a_command_queues_again::wake_a_pending_delete(their_common.as_path(), &leaving, their_target.as_path(), &mut answer);
                return answer.queued;
            }));
        }
        let _all_read: std::sync::BarrierWaitResult = barrier.wait();
        let mut counted: usize = 0;
        for handle in handles {
            if handle.join().expect("a command that ran") {
                counted += 1;
            }
        }
        return counted;
    });
    let allowed: usize = usize::try_from(MAX_TRIES).expect("a count of attempts");
    assert!(woke <= allowed, "{woke} commands asked for a wake, and a record may cost {allowed} attempts");
    assert_eq!(woke, allowed, "and the attempts a record may cost are spent by the commands that asked: {woke}");
    assert!(owed(&place, &item), "the delete is still the round's to run");
    let after: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
    assert!(!after.queued, "no later command asks for another wake");
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the control takes the subvolume");
    remove(place.root.as_path());
}

/// The exit status a command killed at one instruction of the look ends with,
/// which no other ending of the test binary uses.
const KILLED_AT_A_STEP: i32 = 77;

/// What:
///  A command killed at any instruction between proving a leftover and queueing the delete its leaving owes
///  leaves that delete to be queued by the next command,
///  whatever the command had spent before it died.
/// Why:
///  A process can be killed at any instruction,
///  and the durable attempt files are the budget that stops a stuck delete from costing a wake on every command.
///  A budget spent before the act it counts turns that permitted kill into a stall:
///  the next command finds every attempt spent,
///  queues nothing,
///  and says attempts were made that no service ever ran
/// (`consume_an_attempt`,
/// `queue_the_delete_of_one_standing_leaving`).
///  So the control dies at each instruction of that window, once for each attempt the record may cost, and then asks
///  the next command to queue the delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const step of [AFTER_THE_PROOF, BEFORE_THE_DELETE_IS_QUEUED]) {
///   for (const attempt of range(MAX_TRIES)) expect(await killAt(step)).toBe(77);
///   const answer = queueTheLeavingsARecordNoLongerNames(common);
///   expect(answer.queued).toBe(true); expect(pendingOf(item).isFile()).toBe(true); expect(answer.said).toEqual([]);
/// }
/// ```
#[test]
fn a_command_killed_before_the_queued_delete_leaves_the_delete_to_be_queued() {
    // The child of this same test binary: it dies at the one instruction it was told, and never runs past it.
    if let Ok(common) = std::env::var("ATTEMPT_CONTROL_COMMON") {
        let watching: String = std::env::var("ATTEMPT_CONTROL_STEP").expect("the step this child dies at");
        let watched: String = watching.clone();
        crate::worktree_attempt_budget_test_support::act_at_each_step(move |step: &str| {
            if step == watched {
                // What a command killed at this instruction leaves behind is exactly what ran before it.
                // SAFETY: `_exit` ends this process at once and reads nothing this program owns.
                unsafe { libc::_exit(KILLED_AT_A_STEP) };
            }
        });
        let answer: Leavings = queue_the_leavings_a_record_no_longer_names(Path::new(&common));
        panic!("the child ran past {watching}: queued={} said={:?}", answer.queued, answer.said);
    }
    let Some(place): Option<Place> = a_place_a_removal_renames_into("attempt-budget-kills") else { return };
    let (id, hidden): (u64, PathBuf) = a_subvolume_under_its_own_leaving_name(place.place.as_path(), "kept.txt", b"held\n");
    let item: WorkItem = removal_item(place.common.as_path(), place.place.as_path(), id);
    // The state a round that looked before the rename leaves: the leaving record stands and no queue record owes it.
    record_a_leaving_with_owner(place.common.as_path(), &item, Some("1:ended")).expect("the leaving record of a command that ended");
    assert!(!owed(&place, &item), "no queue record owes the delete");
    let executable: PathBuf = std::env::current_exe().expect("this test binary");
    let steps: [&str; 2] = [crate::worktree_attempt_budget_test_support::AFTER_THE_PROOF, crate::worktree_attempt_budget_test_support::BEFORE_THE_DELETE_IS_QUEUED];
    for step in steps {
        // Each instruction is one experiment on the same state: nothing owed and no attempt spent.
        clear_the_queue_and_the_attempts(&place, &item);
        for attempt in 1..=MAX_TRIES {
            let status: std::process::ExitStatus = std::process::Command::new(executable.as_path())
                .arg("a_command_killed_before_the_queued_delete_leaves_the_delete_to_be_queued")
                .arg("--nocapture")
                .env("ATTEMPT_CONTROL_COMMON", place.common.as_path())
                .env("ATTEMPT_CONTROL_STEP", step)
                .status()
                .expect("a child killed at one instruction of the look");
            assert_eq!(status.code(), Some(KILLED_AT_A_STEP), "attempt {attempt} died at {step}");
        }
        let answer: Leavings = queue_the_leavings_a_record_no_longer_names(place.common.as_path());
        assert!(answer.queued, "after {MAX_TRIES} commands died at {step}, the next command still queues the delete: {:?}", answer.said);
        assert!(owed(&place, &item), "and a round finds the delete it owes: {}", item.id());
        assert!(answer.said.is_empty(), "and nothing is said about attempts that queued no delete: {:?}", answer.said);
    }
    clear_the_queue_and_the_attempts(&place, &item);
    assert_eq!(delete_subvolume_of(&item), Verdict::Done, "the control takes the subvolume it made");
    assert!(!hidden.exists(), "and the directory the name held is gone with it");
    remove(place.root.as_path());
}

/// What:
///  Take away every queue record of one delete and every attempt its leaving has taken,
///  so a control's next experiment starts from the state a round that looked before the rename left.
/// Why:
///   The control runs one experiment per instruction of the window,
///  and each must begin with nothing owed and nothing spent;
///  the attempts are what the children are meant to spend, so each experiment begins with none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearTheQueueAndTheAttempts(place, item); // no pending record, no leaving-attempts file
/// ```
fn clear_the_queue_and_the_attempts(place: &Place, item: &WorkItem) {
    for state in [PENDING, crate::work_item::CLAIMED, crate::work_item::DONE, crate::work_item::FAILED] {
        for id in [item.id(), item.id_before_the_branch()] {
            let _removed: std::io::Result<()> = std::fs::remove_file(state_directory(work_of(place).as_path(), state).join(id));
        }
    }
    for number in 1..=MAX_TRIES {
        let attempt: PathBuf = place.common.join(crate::worktree_removal_leaving_that_outlives_its_record::LEAVING_ATTEMPTS_FOLDER).join(format!("{}.{number}", item.id()));
        let _removed: std::io::Result<()> = std::fs::remove_file(attempt);
    }
}
