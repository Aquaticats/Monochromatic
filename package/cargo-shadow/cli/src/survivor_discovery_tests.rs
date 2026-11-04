//! What:
//! Round 7 tests of how a workspace finds the journals whose runs may have left a writer (6b2 findings 1 and 2).
//! Why:
//! A survivor check that fails open lets a recovery or a new transaction change files a surviving process still writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('survivor discovery', () => { it('refuses when the listing is incomplete', ...) })
//! ```

use std::fs;
use std::path::Path;

use crate::crates::LockSnapshot;
use crate::failpoints::{Point, arm};
use crate::journal::{begin, recover};
use crate::journal_recover::survivor_note;
use crate::journal_support::{journal_dirs, plant};
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/s/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A disposable repository whose crate root holds the output of a killed run.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), LEFT_BY_THE_KILLED_RUN).expect("lock");
    return root;
}

/// A judge that refuses the live lockfile, so that a recovery which goes ahead restores the snapshot.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// Round 7 (6b2 1):
/// when a journal entry cannot be listed, the set of journals is not known,
/// and a recovery changes nothing.
#[test]
fn a_journal_listing_that_fails_stops_the_recovery() {
    let root = repository();
    let _journal = plant(root.path(), &[snapshot("original")], &[]);
    arm(Point::JournalListing);

    let messages: Vec<String> = recover(root.path(), &refusing);

    let live: String = fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock");
    assert_eq!(live, LEFT_BY_THE_KILLED_RUN, "a recovery went ahead on a journal set it could not list: {messages:?}");
    assert!(messages.iter().any(|message| { return message.contains("could not be listed"); }), "the person was not told: {messages:?}");
}

/// Round 7 (6b2 1):
/// the same incomplete listing stops a new transaction.
#[test]
fn a_journal_listing_that_fails_stops_a_new_transaction() {
    let root = repository();
    let _journal = plant(root.path(), &[snapshot("original")], &[]);
    arm(Point::JournalListing);

    let started = begin(root.path(), &[snapshot("second")], false);

    assert!(started.is_err(), "a transaction started on a journal set it could not list");
}

/// Round 7 (6b2 2):
/// a journal that lost both its ready record and its group record is uncertain,
/// not proof that no child was ever started.
#[test]
fn a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor() {
    let root = repository();
    let journal = plant(root.path(), &[snapshot("original")], &[]);
    fs::remove_file(journal.join("groups")).expect("lose the group record");
    fs::remove_file(journal.join("ready")).expect("lose the ready record");

    let note: Option<String> = survivor_note(&journal);

    assert!(note.is_some(), "a journal with lost records was read as one that never started a child");
}

/// Round 7 (6b2 2), positive counterpart:
/// the directory a begin that was cut off right after its creation leaves holds nothing,
/// and no child can have started.
#[test]
fn an_empty_journal_directory_is_no_survivor() {
    let root = repository();
    let journals: &Path = &root.path().join(crate::journal::JOURNAL_DIR);
    fs::create_dir_all(journals.join("cut-off")).expect("an empty journal directory");

    assert_eq!(survivor_note(&journals.join("cut-off")), None);
    assert_eq!(journal_dirs(root.path()).len(), 1);
}

/// Round 7 (6b2 2), positive counterpart:
/// a journal whose group record exists and is empty never started a child,
/// because a record is written before every child starts.
#[test]
fn a_journal_with_an_empty_group_record_is_no_survivor() {
    let root = repository();
    let journal = plant(root.path(), &[snapshot("original")], &[]);

    assert_eq!(survivor_note(&journal), None);
}
