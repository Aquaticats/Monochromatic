//! Controls for the words a move that found nothing writes:
//! which order left the record where it is,
//! and what became of its work.

use super::*;
use crate::test_support::{fixture, remove};
use std::cell::RefCell;
use std::time::SystemTime;
use crate::work_item::{Claimed, WorkItem, WorkKind, claim_next_of, enqueue, state_directory};
use std::path::{Path, PathBuf};

const OID: &str = "0123456789abcdef0123456789abcdef01234567";

/// A claimed push item in a fresh work directory.
fn a_claim(name: &str) -> (PathBuf, PathBuf, Claimed) {
    let root: PathBuf = fixture(name);
    let work_dir: PathBuf = root.join("work");
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: root.to_path_buf(),
        git_dir: root.to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: OID.to_owned(),
        parent: String::new(),
        created: String::from("2026-01-01T00:00:00.000Z"),
        tries: 0,
    };
    enqueue(work_dir.as_path(), &item).expect("enqueue");
    let claimed: Claimed = claim_next_of(work_dir.as_path(), &crate::claim_time::ClaimTime::read(), &[WorkKind::Push]).expect("claim").expect("one claim");
    return (root, work_dir, claimed);
}

/// What:
///  The generation the claim's file name carries,
///  which is the part of the name before the reading.
/// Why:
///  The words are asked for with the generation of the claim that failed to move,
/// and a control that has only the name would otherwise hand the whole name over as a generation.
fn generation_of(name: &str) -> String {
    let (token, _stamp) = crate::work_item::parse_claim_name(name).expect("a claim's name carries a generation");
    return token.to_owned();
}

/// What:
///  A record's id and its claim's generation,
///  in a work directory where `claimed` no longer holds the claim:
///  the state a move that found nothing leaves behind.
/// Why:
///  Each control then puts the record where its order leaves it,
/// and asks the words to say which order that was.
fn a_record_whose_move_found_nothing(name: &str) -> (PathBuf, PathBuf, String, String) {
    let (root, work_dir, claimed) = a_claim(name);
    std::fs::remove_file(state_directory(work_dir.as_path(), CLAIMED).join(claimed.name.as_str())).expect("the claim left claimed");
    return (root, work_dir, claimed.id, generation_of(claimed.name.as_str()));
}

/// A file of `id` in the state `state`,
/// which is what a settlement, a recovery or the queue leaves there.
fn a_record_in(work_dir: &Path, state: &str, id: &str) {
    let directory: PathBuf = state_directory(work_dir, state);
    std::fs::create_dir_all(directory.as_path()).expect("the state directory");
    std::fs::write(directory.join(id), b"a record").expect("the record");
}

/// A settlement that moved the record before this one is said,
/// and that its work is there as finished:
/// `done` is where a settlement that succeeded leaves a record,
/// and nothing else moves a record there.
#[test]
fn a_move_that_finds_the_record_settled_says_its_work_is_recorded_as_finished() {
    let (root, work_dir, id, token) = a_record_whose_move_found_nothing("words-record-in-done");
    a_record_in(work_dir.as_path(), DONE, id.as_str());
    let said: String = words_for_a_move_that_found_nothing(work_dir.as_path(), id.as_str(), token.as_str());
    assert!(said.contains(&format!("{DONE}/{id}")), "the words name the directory and the record that is in it: {said}");
    assert!(said.contains("a settlement moved it"), "the words say what moved it: {said}");
    assert!(said.contains("finished"), "the words say what became of the work: {said}");
    assert!(!said.contains("probably"), "the words say what the record's places show instead of guessing: {said}");
    remove(root.as_path());
}

/// A record a recovery is moving is named as that,
/// before the queue it is going back to is asked:
/// the file in `recovering` is the one the take made,
/// and a record the queue holds is the one a take has already queued.
#[test]
fn a_move_that_finds_a_take_in_flight_names_the_recovery_before_the_queue() {
    let (root, work_dir, id, token) = a_record_whose_move_found_nothing("words-record-in-recovering");
    // A record is in one of them at a time; the control fills both to read which one the words look at first.
    a_record_in(work_dir.as_path(), PENDING, id.as_str());
    a_record_in(work_dir.as_path(), RECOVERING, id.as_str());
    let said: String = words_for_a_move_that_found_nothing(work_dir.as_path(), id.as_str(), token.as_str());
    assert!(said.contains(&format!("{RECOVERING}/{id}")), "the file that holds the record is named: {said}");
    assert!(!said.contains("left it queued as"), "a record a move is between steps of is not reported as one the queue already holds: {said}");
    remove(root.as_path());
}

/// The claim's own name is found by its id and its generation,
/// so a name of that id under another generation is another instance's claim,
/// never this one's.
#[test]
fn a_name_of_the_same_id_under_another_generation_is_another_claim() {
    let (root, work_dir, id, token) = a_record_whose_move_found_nothing("words-another-generation");
    let other: String = format!("{id}~0123456789abcdef0123456789abcdef~0123456789abcdef~5000000~1700000000");
    std::fs::write(state_directory(work_dir.as_path(), CLAIMED).join(other.as_str()), b"another instance's record").expect("another claim of the record");
    let said: String = words_for_a_move_that_found_nothing(work_dir.as_path(), id.as_str(), token.as_str());
    assert!(said.contains("claimed it again"), "the words say the claim is another instance's: {said}");
    assert!(said.contains(other.as_str()), "the words name the file that holds the record now: {said}");
    assert!(!said.contains("still in"), "this holder's claim is not reported as still there: {said}");
    remove(root.as_path());
}

/// The record's own place is named when the claim is still there,
/// which is a move that failed for a reason other than the claim's absence:
/// nothing of the record moved,
/// and the look that comes when its lease runs out runs its work again.
#[test]
fn a_move_that_finds_the_claim_still_there_says_nothing_of_the_record_moved() {
    let (root, work_dir, claimed) = a_claim("words-claim-still-there");
    let said: String = words_for_a_move_that_found_nothing(work_dir.as_path(), claimed.id.as_str(), generation_of(claimed.name.as_str()).as_str());
    assert!(said.contains(&format!("{CLAIMED}/{}", claimed.name)), "the words name the file that holds the record: {said}");
    assert!(said.contains("nothing of the record moved here"), "the words say what happened: {said}");
    assert!(said.contains("when its lease runs out"), "the words say when its work is run again: {said}");
    remove(root.as_path());
}

/// What:
///  A state directory made reachable by name but not listable,
///  put back when this value ends.
/// Why:
///  The accident the words must not read as an absence is a directory the filesystem refuses,
/// and the mode has to be given back however the control ends.
struct Unlistable<'directory>(&'directory Path);

impl Unlistable<'_> {
    /// Take the read permission away, keeping search, or all of it for a single name probe.
    fn of(directory: &Path, mode: u32) -> Unlistable<'_> {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(mode)).expect("the directory's mode");
        return Unlistable(directory);
    }
}

impl Drop for Unlistable<'_> {
    /// Give the read permission back.
    fn drop(&mut self) {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(self.0, std::fs::Permissions::from_mode(0o755)).expect("the directory can be listed again");
    }
}

/// A `claimed` directory that could not be listed is said as that,
/// with the error the filesystem gave and the permission to look at,
/// and never as a record that is in none of its places:
/// the claim is still there.
#[test]
fn a_move_that_finds_an_unlistable_claimed_directory_says_the_look_failed() {
    let (root, work_dir, claimed) = a_claim("words-unlistable-claimed");
    let claimed_directory: PathBuf = state_directory(work_dir.as_path(), CLAIMED);
    let said: String;
    {
        let _unlistable: Unlistable<'_> = Unlistable::of(claimed_directory.as_path(), 0o111);
        said = words_for_a_move_that_found_nothing(work_dir.as_path(), claimed.id.as_str(), generation_of(claimed.name.as_str()).as_str());
    }
    let still_there: bool = claimed_directory.join(claimed.name.as_str()).exists();
    remove(root.as_path());
    assert!(still_there, "the look removed the claim it could not read");
    assert!(said.contains(CLAIMED) && said.contains("could not be read"), "the words name the directory and that it could not be read: {said}");
    assert!(said.contains("Permission denied"), "the words keep the error the listing gave: {said}");
    assert!(said.contains("could not be found out"), "the words say what could not be found out: {said}");
    assert!(!said.contains("no file of"), "the claim was there, but the words said it is in no place: {said}");
}

/// A place of one record that could not be looked at is said as that,
/// and never as the record being absent from it:
/// `done` is there and refuses every look into it.
#[test]
fn a_move_that_finds_an_unreadable_done_says_the_look_failed() {
    let (root, work_dir, id, token) = a_record_whose_move_found_nothing("words-unreadable-done");
    a_record_in(work_dir.as_path(), DONE, id.as_str());
    let done: PathBuf = state_directory(work_dir.as_path(), DONE);
    let said: String;
    {
        let _unreadable: Unlistable<'_> = Unlistable::of(done.as_path(), 0o000);
        said = words_for_a_move_that_found_nothing(work_dir.as_path(), id.as_str(), token.as_str());
    }
    remove(root.as_path());
    assert!(said.contains(DONE) && said.contains("could not be looked at"), "the words name the state and that it could not be looked at: {said}");
    assert!(said.contains("Permission denied"), "the words keep the error the look gave: {said}");
    assert!(!said.contains("no file of"), "the record was there, but the words said it is in no place: {said}");
}

/// What:
///  The walk a control hands in:
///  the states it was asked for, in the order it was asked,
///  and what it answers for each one.
/// Why:
///   The words read the states of a record in an order,
/// and both the order and a walk that stops part-way are states of the reading
///  a real directory cannot be made to reach on a local file system.
struct WalksAsTheControlSays<'a> {
    /// The state whose walk the control answers with `answer`, or nothing to walk the real directory.
    state: &'a str,
    /// What that walk reached.
    answer: &'a WhatTheWalkReached,
    /// The states asked for so far, in order.
    asked: RefCell<Vec<String>>,
}

impl WalksAsTheControlSays<'_> {
    /// The walk itself, for `words_from_the_walk`.
    fn walk(&self) -> impl Fn(&Path, &str) -> WhatTheWalkReached + '_ {
        return |work_dir: &Path, state: &str| {
            self.asked.borrow_mut().push(state.to_owned());
            if state == self.state {
                let stopped_by: Option<std::io::Error> = self.answer.stopped_by.as_ref().map(|error| { return std::io::Error::new(error.kind(), error.to_string()); });
                return WhatTheWalkReached { names: self.answer.names.clone(), stopped_by };
            }
            return the_names_in(work_dir, state);
        };
    }
}

/// A walk over `claimed` that stopped part-way is said as a walk that stopped, with how many of its names it reached,
/// and never as a record that is in none of the places:
/// the claim may be among the names the walk never read.
#[test]
fn a_move_that_finds_a_walk_that_stopped_part_way_says_what_could_not_be_found_out() {
    let (root, work_dir, claimed) = a_claim("words-walk-stopped");
    let id: String = claimed.id;
    let token: String = generation_of(claimed.name.as_str());
    let stopped: WhatTheWalkReached = WhatTheWalkReached {
        names: vec![(String::from("push-0000000000000000000000000000000000000000-f921bd05e68b0374~0123456789abcdef0123456789abcdef~x"), SystemTime::UNIX_EPOCH)],
        stopped_by: Some(std::io::Error::from_raw_os_error(5)),
    };
    let walks: WalksAsTheControlSays<'_> = WalksAsTheControlSays { state: CLAIMED, answer: &stopped, asked: RefCell::new(Vec::new()) };
    let walk = walks.walk();
    let said: String = words_from_the_walk(work_dir.as_path(), id.as_str(), token.as_str(), &walk);
    remove(root.as_path());
    assert!(said.contains("stopped after"), "the words say the walk stopped and how far it got: {said}");
    assert!(said.contains("1 name"), "the words say how many of the names it reached: {said}");
    assert!(said.contains("could not be found out"), "the words say what could not be found out: {said}");
    assert!(!said.contains("no file of"), "the claim may be in the names the walk never read, but the words said it is nowhere: {said}");
    assert!(!said.contains("something other than a move of this queue"), "the words name a removal nobody proved: {said}");
}

/// A take that publishes its claim in `claimed` between two of the looks is found by the look made after them:
/// the words name the claim that holds the record and never say the record is in none of the places.
#[test]
fn a_move_that_finds_the_record_claimed_again_between_its_looks_says_where_it_is() {
    let (root, work_dir, claimed) = a_claim("words-claimed-between-looks");
    let id: String = claimed.id;
    let token: String = generation_of(claimed.name.as_str());
    let staged: String = format!("{id}~0123456789abcdef0123456789abcdef~{}", crate::claim_time::ClaimTime::read().encode());
    let claimed_directory: PathBuf = state_directory(work_dir.as_path(), CLAIMED);
    std::fs::rename(claimed_directory.join(claimed.name.as_str()), state_directory(work_dir.as_path(), RECOVERING).join(staged.as_str())).expect("the take stages the claim");
    // The move finishes while the words are between the look at `recovering` and the look at `pending`, which is the window a real take opens.
    let publishing = |looked_at: &Path, state: &str| {
        if state == RECOVERING {
            std::fs::rename(state_directory(looked_at, RECOVERING).join(staged.as_str()), claimed_directory.join(staged.as_str())).expect("the take publishes the claim");
        }
        return the_names_in(looked_at, state);
    };
    let said: String = words_from_the_walk(work_dir.as_path(), id.as_str(), token.as_str(), &publishing);
    remove(root.as_path());
    assert!(said.contains(&format!("{CLAIMED}/{staged}")), "the words name the claim that holds the record now: {said}");
    assert!(said.contains("claimed it again"), "the words say another instance claimed the record again: {said}");
    assert!(!said.contains("no file of"), "a take finished between the looks, but the words said the record is nowhere: {said}");
}
