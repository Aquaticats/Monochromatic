//! Controls for the words a move that found nothing writes:
//! which order left the record where it is,
//! and what became of its work.

use super::*;
use crate::test_support::{fixture, remove};
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
    assert!(said.contains(RECOVERING), "the recovery's directory is named: {said}");
    assert!(!said.contains(&format!("{PENDING}/{id}")), "a record a recovery is moving is not reported as one the queue holds: {said}");
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
