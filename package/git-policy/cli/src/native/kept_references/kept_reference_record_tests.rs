//! What:
//!  Controls for the record written beside a kept reference.
//! Why:
//!  The age the pruning judges a reference by,
//!  and the worktree a person reads afterwards,
//!  are nowhere in the reference itself,
//!  so a record that a second writer could replace would let a reference outlive its age,
//!  and one that does not round-trip loses the worktree entirely.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(writeIfAbsent(dir, first)?.keptAtMs).toBe(first.keptAtMs);
//! ```

use super::{FIRST_SEEN, KEPT_FOR_A_REMOVAL, KeptReferenceRecord, read_record, record_folder, remove_record, write_if_absent};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// A record of one kept reference.
fn a_record(reference: &str, kept_at_ms: u128, worktree: Option<&Path>) -> KeptReferenceRecord {
    return KeptReferenceRecord {
        reference: String::from(reference),
        commit: String::from("deb10b333be2e7c350d4d5cce35b8d474161fbbc"),
        batch: 1794071191963902,
        rescued: reference.ends_with("-rescue"),
        worktree: worktree.map(|path| { return path.display().to_string(); }),
        kept_at_ms,
        why: String::from(KEPT_FOR_A_REMOVAL),
    };
}

/// A record written once is read back whole,
/// worktree included.
#[test]
fn a_record_round_trips() {
    let root: PathBuf = fixture("kept-reference-record-round-trip");
    let record: KeptReferenceRecord = a_record("refs/cli-git/kept/1794071191963902-rescue", 1_791_000_000_000, Some(Path::new("/a worktree/with a space")));
    let written: Option<KeptReferenceRecord> = write_if_absent(root.as_path(), &record);
    assert_eq!(written, Some(record.clone()));
    assert_eq!(read_record(root.as_path(), record.reference.as_str()), Some(record));
    remove(&root);
}

/// A second writer's record never replaces the first,
/// so a reference cannot be made younger by a pass that meets it.
#[test]
fn the_first_record_of_a_reference_is_the_one_that_stays() {
    let root: PathBuf = fixture("kept-reference-record-first");
    let first: KeptReferenceRecord = a_record("refs/cli-git/kept/7", 1_000, None);
    let second: KeptReferenceRecord = a_record("refs/cli-git/kept/7", 9_000, Some(Path::new("/other")));
    assert_eq!(write_if_absent(root.as_path(), &first).map(|held| { return held.kept_at_ms; }), Some(1_000));
    assert_eq!(write_if_absent(root.as_path(), &second).map(|held| { return held.kept_at_ms; }), Some(1_000));
    assert!(std::fs::read_dir(record_folder(root.as_path())).expect("the record folder").count() == 1);
    remove(&root);
}

/// The record of a reference written by an earlier release is the first pass's to write,
/// and it says no worktree was recorded.
#[test]
fn a_reference_no_release_recorded_has_no_worktree() {
    let root: PathBuf = fixture("kept-reference-record-first-seen");
    let seen: KeptReferenceRecord = a_record("refs/cli-git/kept/11", 5_000, None);
    let written: KeptReferenceRecord = write_if_absent(root.as_path(), &KeptReferenceRecord { why: String::from(FIRST_SEEN), ..seen }).expect("a record");
    assert_eq!(written.worktree, None);
    assert_eq!(written.why, FIRST_SEEN);
    remove(&root);
}

/// A record goes with the reference it belongs to.
#[test]
fn removing_the_record_leaves_the_folder_empty() {
    let root: PathBuf = fixture("kept-reference-record-removal");
    let record: KeptReferenceRecord = a_record("refs/cli-git/kept/12", 7_000, None);
    write_if_absent(root.as_path(), &record).expect("a record");
    remove_record(root.as_path(), record.reference.as_str());
    assert_eq!(read_record(root.as_path(), record.reference.as_str()), None);
    remove(&root);
}

/// A record that is a named pipe is read as no record,
/// at the bound,
/// and a sweep of the folder skips it rather than waiting for a writer (issue 6415).
#[test]
fn a_record_that_is_a_named_pipe_is_answered_at_once() {
    let root: PathBuf = fixture("kept-reference-named-pipe");
    std::fs::create_dir_all(record_folder(root.as_path()).as_path()).expect("the record folder");
    crate::subvolume_sync_attrs::make_fifo(record_folder(root.as_path()).join("a-record").as_path(), 0o600).expect("a named pipe where a record goes");
    let inside: PathBuf = root.clone();
    let answered = crate::test_support::finished_within(move || {
        crate::kept_reference_record::sweep_records_of_references_that_are_gone(inside.as_path(), &[], 10_000_000, std::time::Duration::ZERO);
        return read_record(inside.as_path(), "refs/cli-git/kept/13");
    });
    let Some(record) = answered else { panic!("a record that is a named pipe held the pass") };
    assert_eq!(record, None, "a pipe is no record");
    assert!(record_folder(root.as_path()).join("a-record").exists(), "and the sweep left it where it is");
    remove(&root);
}
