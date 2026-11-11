//! What:
//!  Controls for the record of a commit a reference lock keeps off its branch:
//!  what it holds,
//!  what it reads back,
//!  and what a record this build cannot read leaves alone.
//! Why:
//!  The record is the only thing that makes such a commit durable,
//!  so a byte that does not survive the text form,
//!  or a record removed because it could not be parsed,
//!  loses a person's commit.
//!  The controls use disposable directories under the test root and start no Git.

use super::{HELD_EXTENSION, Held, Holding, directory_of, hold, list, path_of, write};
use crate::service_journal::Intent;
use crate::test_support::remove;
use std::path::{Path, PathBuf};

/// A directory of this control's own,
/// holding a Git directory with nothing in it.
fn fixture(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = crate::test_support::fixture(name);
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory");
    return (root, git_dir);
}

/// The intent of a batch whose branch was `old` and is `new` after it.
fn intent_of(sequence: u64, old: &str, new: &str, commits: &[&str]) -> Intent {
    return Intent {
        seq: sequence,
        repository: PathBuf::from("/repo"),
        common_dir: PathBuf::from("/repo/.git"),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: old.to_string(),
        new: new.to_string(),
        ids: vec![String::from("request-1")],
        commits: commits.iter().map(|commit| { return commit.to_string(); }).collect(),
        paths: vec![b"f.txt".to_vec()],
        derived: false,
        cleanup: None,
        index: PathBuf::from("/repo/.git/index"),
        git_dir: PathBuf::from("/repo/.git"),
    };
}

/// A held batch of one commit,
/// with no lock standing yet.
fn held_of(git_dir: &Path, sequence: u64) -> Held {
    let intent: Intent = intent_of(sequence, "1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a", "2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", &["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"]);
    return Held {
        seq: sequence,
        journal: Path::new("/state/service/instance").to_path_buf(),
        repository: PathBuf::from("/repo"),
        common_dir: PathBuf::from("/repo/.git"),
        git_dir: git_dir.to_path_buf(),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        old: intent.old.clone(),
        new: intent.new.clone(),
        commits: intent.commits.clone(),
        index: PathBuf::from("/repo/.git/index"),
        // Bytes no text form could carry, so the hexadecimal field is what the reader has to decode.
        reflog: b"commit: caf\xc3\xa9 \xff\n".to_vec(),
        locks: vec![git_dir.join("refs/heads/main.lock")],
        created: String::from("2026-11-09T18:30:54-05:00"),
    };
}

/// What:
///  A record survives the text form byte for byte,
///  and a reader answers the same batch it was given.
/// Why:
///  The reflog message and the lock paths are bytes and not text,
///  and the branch move writes exactly the bytes the record carries.
#[test]
fn a_record_reads_back_as_it_was_written() {
    let (root, git_dir) = fixture("held-record-round-trip");
    let held: Held = held_of(git_dir.as_path(), 7);
    write(git_dir.as_path(), &held).expect("the record is published");
    let (found, damaged) = list(git_dir.as_path());
    assert!(damaged.is_empty(), "nothing is damaged: {damaged:?}");
    assert_eq!(found, vec![held]);
    remove(root.as_path());
}

/// What:
///  A record is found by its batch number,
///  and two batches of one worktree are listed oldest first.
/// Why:
///  Two held commits on one branch are settled in the order their batches were made.
#[test]
fn two_records_are_listed_oldest_first() {
    let (root, git_dir) = fixture("held-record-order");
    write(git_dir.as_path(), &held_of(git_dir.as_path(), 9)).expect("the later record");
    write(git_dir.as_path(), &held_of(git_dir.as_path(), 4)).expect("the earlier record");
    assert_eq!(path_of(git_dir.as_path(), 4), directory_of(git_dir.as_path()).join(format!("4.{HELD_EXTENSION}")));
    let numbers: Vec<u64> = list(git_dir.as_path()).0.iter().map(|held| { return held.seq; }).collect();
    assert_eq!(numbers, vec![4, 9]);
    remove(root.as_path());
}

/// What:
///  A record this build cannot read is named and kept,
///  never removed with the batch beside it.
/// Why:
///  It names a commit only the batch's side reference holds,
///  so removing it would leave that commit pointing at nothing
///  (the owner's rule "never discard user data").
#[test]
fn a_record_that_cannot_be_read_is_kept_and_named() {
    let (root, git_dir) = fixture("held-record-damaged");
    let path: PathBuf = path_of(git_dir.as_path(), 3);
    std::fs::create_dir_all(directory_of(git_dir.as_path()).as_path()).expect("the folder");
    std::fs::write(path.as_path(), "seq=3\nref=refs/heads/main\n").expect("a record that stops early");
    let (found, damaged) = list(git_dir.as_path());
    assert!(found.is_empty(), "nothing is read from a record that stops early: {found:?}");
    assert_eq!(damaged, vec![path.clone()]);
    assert!(path.exists(), "a record that cannot be read is left where it is");
    remove(root.as_path());
}

/// What:
///  A landing records a held batch from its own intent,
///  and the record names the branch values the intent carries.
/// Why:
///  The record and the intent describe one batch,
///  so the settlement reads the branch values from the intent and the locks from the record.
#[test]
fn holding_writes_the_record_the_intent_describes() {
    let (root, git_dir) = fixture("held-record-holding");
    let intent: Intent = intent_of(5, "1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a", "2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", &["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"]);
    let lock: PathBuf = git_dir.join("refs/heads/main.lock");
    let holding: Holding<'_> = Holding {
        journal: Path::new("/state/service/instance"),
        repository: Path::new("/repo"),
        common_dir: Path::new("/repo/.git"),
        git_dir: git_dir.as_path(),
        index: Path::new("/repo/.git/index"),
        reference: &crate::ref_name::RefName::from_text("refs/heads/main"),
        seq: 5,
        intent: &intent,
        locks: vec![lock.clone()],
        reflog: b"commit: the edit\n",
    };
    let held: Held = hold(holding).expect("the record is published");
    assert_eq!(held.old, intent.old);
    assert_eq!(held.new, intent.new);
    assert_eq!(held.commits, intent.commits);
    assert_eq!(held.locks, vec![lock]);
    assert!(path_of(git_dir.as_path(), 5).exists());
    remove(root.as_path());
}
