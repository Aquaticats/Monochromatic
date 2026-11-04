//! What:
//! Round 6 tests of the recovery of journals that never became ready and of the ready record's payload set.
//! Why:
//! Fifth review 5b2 findings 3, 4 and 5: unready cleanup must not delete a nonredundant payload or make a durable copy
//! disappear before its live equivalent is durable, and the ready record must name exactly the payload the indexes imply.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('recovery of unready journals', () => { it(...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::crates::LockSnapshot;
use crate::journal::{begin, recover};
use crate::journal_recover::read_ready;
use crate::journal_support::{journal_dirs, plant};
use crate::manifest_edit::ManifestEdit;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/r/one";

/// A snapshot of the lockfile of the case.
fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

/// A repository with one crate whose lockfile and manifest hold the given text.
fn repository(lock: &str, manifest: &str) -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.toml"), manifest).expect("manifest");
    return root;
}

/// The manifest edit of the case.
fn edit(root: &Path, before: &str) -> ManifestEdit {
    return ManifestEdit { crate_dir: CRATE_DIR.to_string(), path: root.join(CRATE_DIR).join("Cargo.toml"), before: before.to_string(), produced: None };
}

/// Recovers with a policy that refuses everything it is asked about.
fn run_recovery(root: &Path) -> Vec<String> {
    return recover(root, &|_before, _live| { return vec!["refused".to_string()]; });
}

/// A journal that crashed while its manifests were being journaled: snapshots written, no ready record.
fn unready_journal_with_manifest(root: &Path, lock: &str, manifest_before: &str) -> PathBuf {
    let transaction = begin(root, &[snapshot(lock)], true).expect("begin");
    drop(transaction);
    let journal = journal_dirs(root).into_iter().next().expect("a journal");
    fs::write(journal.join("m0.toml"), manifest_before).expect("manifest snapshot");
    fs::write(journal.join("manifests.txt"), format!("v1\n{CRATE_DIR}\t{}\nend 1\n", crate::journal_transaction::encode_path(&root.join(CRATE_DIR).join("Cargo.toml")))).expect("manifest index");
    return journal;
}

/// Round 6 (5b2 3): equal lockfiles do not make a manifest snapshot redundant.
#[test]
fn an_unready_journal_whose_manifest_differs_from_its_snapshot_is_kept() {
    let root = repository("same lock", "manifest after");
    let journal = unready_journal_with_manifest(root.path(), "same lock", "manifest before");

    let messages = run_recovery(root.path());

    assert!(journal.join("m0.toml").exists(), "the only copy of the manifest was deleted: {messages:?}");
}

/// Round 6 (5b2 3): a journal that lost its index but still holds payload is not empty.
#[test]
fn an_unready_journal_that_lost_its_index_but_holds_snapshots_is_kept() {
    let root = repository("same lock", "manifest");
    let journal = unready_journal_with_manifest(root.path(), "same lock", "manifest");
    fs::remove_file(journal.join("index.txt")).expect("lose the index");

    let messages = run_recovery(root.path());

    assert!(journal.join("0.lock").exists(), "a snapshot was deleted without an index to judge it: {messages:?}");
}

/// Round 6 (5b2 4): a redundant journal is deleted only after the live files that make it redundant are durable.
#[test]
fn an_unready_journal_is_kept_when_the_live_files_cannot_be_made_durable() {
    let root = repository("same lock", "manifest");
    let journal = unready_journal_with_manifest(root.path(), "same lock", "manifest");
    crate::failpoints::arm(crate::failpoints::Point::RecoverDurability);

    let messages = run_recovery(root.path());

    assert!(journal.exists(), "the durable copy was deleted before the live files were durable: {messages:?}");
}

/// A planted ready journal with one manifest, and the text of its ready record.
fn ready_journal(root: &Path) -> (PathBuf, String) {
    let journal = plant(root, &[snapshot("lock")], &[edit(root, "manifest before")]);
    let ready = fs::read_to_string(journal.join("ready")).expect("ready record");
    return (journal, ready);
}

/// Rewrites the ready record with the given file lines (name and length) and the matching end line.
fn write_ready(journal: &Path, lines: &[String]) {
    let mut text = String::from("v1\nmanifests 1\n");
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str(&format!("end {}\n", lines.len()));
    fs::write(journal.join("ready"), text).expect("write ready");
}

/// The file lines of a ready record.
fn file_lines(ready: &str) -> Vec<String> {
    return ready.lines().filter(|line| { return line.contains('\t'); }).map(str::to_string).collect();
}

/// Round 6 (5b2 5): a ready record that leaves out a manifest snapshot is rejected,
/// so a truncated snapshot cannot pass unchecked.
#[test]
fn a_ready_record_that_omits_a_manifest_snapshot_is_rejected() {
    let root = repository("lock", "manifest");
    let (journal, ready) = ready_journal(root.path());
    let kept: Vec<String> = file_lines(&ready).into_iter().filter(|line| { return !line.starts_with("m0.toml"); }).collect();
    write_ready(&journal, &kept);

    assert!(read_ready(&journal).is_err(), "an omitted manifest snapshot passed");
}

/// Round 6 (5b2 5): a duplicate entry is rejected.
#[test]
fn a_ready_record_with_a_duplicate_entry_is_rejected() {
    let root = repository("lock", "manifest");
    let (journal, ready) = ready_journal(root.path());
    let mut lines = file_lines(&ready);
    lines.push(lines[0].clone());
    write_ready(&journal, &lines);

    assert!(read_ready(&journal).is_err(), "a duplicate entry passed");
}

/// Round 6 (5b2 5): an entry that is no part of the payload is rejected.
#[test]
fn a_ready_record_with_an_unexpected_entry_is_rejected() {
    let root = repository("lock", "manifest");
    let (journal, ready) = ready_journal(root.path());
    let mut lines = file_lines(&ready);
    let length = fs::metadata(journal.join("owner")).expect("owner").len();
    lines.push(format!("owner\t{length}"));
    write_ready(&journal, &lines);

    assert!(read_ready(&journal).is_err(), "an unexpected entry passed");
}

/// Round 6 (5b 4): the advice about exhausted temporary names never offers a copy-and-delete as safe,
/// because a program that still has the file open keeps writing to the original.
#[test]
fn the_exhausted_names_advice_does_not_recommend_archiving_and_names_the_open_file_risk() {
    let message = crate::journal_replace::names_exhausted_message(Path::new("/x"), "Cargo.lock");

    assert!(!message.contains("archive"), "{message}");
    assert!(message.contains("open"), "{message}");
}
