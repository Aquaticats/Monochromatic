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
use crate::failpoints::{Point, arm, fires};
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

/// A judge that refuses the live lockfile,
/// so that a recovery which goes ahead restores the snapshot.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// The four ways the listing of the journals can fail,
/// each injected inside the listing,
/// with the second entry as the one that fails so that the first is already in the partial listing.
const LISTING_FAULTS: [Point; 4] = [Point::ListingOpen, Point::ListingIteration, Point::ListingInspection, Point::ListingEntryNotFound];

/// Round 7 (6b2 1),
/// round 8 (acceptance row V1,
/// 7b P1-2):
/// when the journals cannot be listed completely,
/// however it fails,
/// a recovery changes nothing and says it could not list them.
#[test]
fn a_journal_listing_that_fails_in_any_way_stops_the_recovery() {
    for fault in LISTING_FAULTS {
        let root = repository();
        let _first = plant(root.path(), &[snapshot("original")], &[]);
        let _second = plant(root.path(), &[snapshot("original")], &[]);
        arm(fault);
        let _ = crate::bounded_run::take_trace_for_test();

        let messages: Vec<String> = recover(root.path(), &refusing);

        // The fault fired where the schedule says: at the second entry, after the first journal was accumulated into the partial
        // listing (the open of the directory has no entry yet). An injection at an earlier boundary would leave no such event.
        let trace: Vec<String> = crate::bounded_run::take_trace_for_test().into_iter().filter(|event| { return event.starts_with("listing fault "); }).collect();
        if matches!(fault, Point::ListingOpen) {
            assert!(trace.is_empty(), "{fault:?}: {trace:?}");
        } else {
            assert_eq!(trace.len(), 1, "{fault:?}: {trace:?}");
            assert!(trace[0].contains("at entry 1 after 1 accumulated"), "{fault:?}: {}", trace[0]);
        }

        let live: String = fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock");
        assert_eq!(live, LEFT_BY_THE_KILLED_RUN, "{fault:?}: a recovery went ahead on a journal set it could not list: {messages:?}");
        assert!(messages.iter().any(|message| { return message.contains("could not be listed"); }), "{fault:?}: the person was not told: {messages:?}");
        assert_eq!(journal_dirs(root.path()).len(), 2, "{fault:?}: a journal was removed");
    }
}

/// Round 7 (6b2 1),
/// round 8 (acceptance row V1,
/// 7b P1-2):
/// the same failures stop a new transaction,
/// and in particular an entry that vanished while the journals were listed (`NotFound` of one entry) is unknown,
/// not "no earlier journals":
/// the listing so far,
/// which holds a journal,
/// is not discarded.
#[test]
fn a_journal_listing_that_fails_in_any_way_stops_a_new_transaction() {
    for fault in LISTING_FAULTS {
        let root = repository();
        let _first = plant(root.path(), &[snapshot("original")], &[]);
        let _second = plant(root.path(), &[snapshot("original")], &[]);
        arm(fault);

        let started = begin(root.path(), &[snapshot("second")], false);

        // Sixth pass, group 5: the refusal is the listing's own (the fault was reached, and the reason says the journals could not be
        // listed), not a refusal that two journals already exist.
        assert!(!fires(fault), "{fault:?}: the listing fault was never reached");
        let reason: String = started.err().map(|error| { return error.to_string(); }).unwrap_or_else(|| { panic!("{fault:?}: a transaction started on a journal set it could not list"); });
        assert!(reason.contains("could not be listed"), "{fault:?}: the refusal is not the listing's: {reason}");
        assert_eq!(journal_dirs(root.path()).len(), 2, "{fault:?}: the refused transaction left a journal");
    }
}

/// Sixth pass, group 5 (V1, the completed journals):
/// a completed journal (`.done`) is examined like any other entry, so an examination that fails on it fails the listing too.
/// Both journals here are completed, so the entry that fails is a completed one whichever the directory lists second.
#[test]
fn a_completed_journal_that_cannot_be_examined_fails_the_listing() {
    for fault in [Point::ListingInspection, Point::ListingEntryNotFound] {
        let root = repository();
        let first = plant(root.path(), &[snapshot("original")], &[]);
        let second = plant(root.path(), &[snapshot("original")], &[]);
        for journal in [&first, &second] {
            let done: std::path::PathBuf = std::path::PathBuf::from(format!("{}{}", journal.display(), crate::journal::DONE_SUFFIX));
            fs::rename(journal, &done).expect("mark the journal done");
        }
        arm(fault);

        let listed = crate::journal_recover::list_journals(&root.path().join(crate::journal::JOURNAL_DIR));

        assert!(!fires(fault), "{fault:?}: the listing fault was never reached");
        assert!(listed.is_err(), "{fault:?}: a completed journal that could not be examined was listed without examination");
    }
}

/// Acceptance row V1,
/// the separate positive case:
/// a workspace that has no journal directory yet has no earlier journals,
/// and both a recovery and a new transaction go on.
#[test]
fn an_absent_journal_directory_is_no_journals() {
    let root = repository();

    let messages: Vec<String> = recover(root.path(), &refusing);
    let started = begin(root.path(), &[snapshot("second")], false);

    assert!(messages.is_empty(), "{messages:?}");
    assert!(started.is_ok(), "{:?}", started.err());
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

    // The consequences, through the real consumers (certification, third pass): the recovery keeps the journal, restores nothing and
    // says why, and a new transaction is not admitted beside it.
    let messages: Vec<String> = recover(root.path(), &refusing);
    assert_eq!(journal_dirs(root.path()), vec![journal.clone()], "the journal with lost records was removed or replaced: {messages:?}");
    assert_eq!(fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock")).expect("lock"), LEFT_BY_THE_KILLED_RUN, "a recovery restored beside a journal with lost records: {messages:?}");
    assert!(messages.iter().any(|message| { return message.contains(journal.to_string_lossy().as_ref()); }), "the recovery did not name the journal: {messages:?}");
    let admitted = crate::journal::begin(root.path(), &[snapshot(LEFT_BY_THE_KILLED_RUN)], false);
    assert!(admitted.is_err(), "a transaction was admitted beside a journal with lost records");
    assert_eq!(journal_dirs(root.path()), vec![journal], "the refused admission left a journal behind");
}

/// Round 7 (6b2 2),
/// positive counterpart:
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

/// Round 7 (6b2 2),
/// positive counterpart:
/// a journal whose group record exists and is empty never started a child,
/// because a record is written before every child starts.
#[test]
fn a_journal_with_an_empty_group_record_is_no_survivor() {
    let root = repository();
    let journal = plant(root.path(), &[snapshot("original")], &[]);

    assert_eq!(survivor_note(&journal), None);
}
