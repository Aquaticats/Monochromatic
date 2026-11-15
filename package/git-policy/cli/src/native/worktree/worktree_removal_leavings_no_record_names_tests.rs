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
use crate::worktree_removal_leaving_that_outlives_its_record::{leaving_records, mark_the_leaving, read_the_leaving};
use crate::worktree_removal_leavings_a_command_queues_again::queue_the_leavings_a_record_no_longer_names;

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
