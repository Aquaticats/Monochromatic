//! Controls for the evidence a record written before a snapshot leaves:
//! the source it is taken from,
//! and the transaction that creation must be later than.

use super::*;
use crate::test_support::remove;
use std::path::PathBuf;
use crate::worktree_copy_journal::{journal_root, write_atomic};

/// A subvolume that stands for a checkout,
/// and a journal directory that has taken one durable write,
/// as the binding record of an add does before this rung.
#[cfg(target_os = "linux")]
fn source_and_a_written_journal(label: &str) -> Option<(PathBuf, PathBuf)> {
    let root: PathBuf = crate::fs_btrfs_test_support::btrfs_fixture(label)?;
    let source: PathBuf = root.join("source");
    crate::fs_btrfs_ioctl::subvolume_create_at(source.as_path()).expect("the source subvolume");
    std::fs::write(source.join("tracked.txt"), b"tracked").expect("a file");
    let common: PathBuf = source.join(".git");
    std::fs::create_dir_all(journal_root(common.as_path())).expect("the journal root");
    // The write is flushed, as the binding record is, so the journal's subvolume carries the transaction it took.
    write_atomic(journal_root(common.as_path()).join("anchor").as_path(), b"anchor\n", "anchor").expect("a durable write");
    return Some((root, source));
}

/// A snapshot made after the record is proved by the source it names and the transaction it follows.
#[cfg(target_os = "linux")]
#[test]
fn a_snapshot_made_after_the_record_is_proved() {
    let Some((root, source)) = source_and_a_written_journal("origin-after") else { return };
    let origin: SnapshotOrigin = origin_of_the_source(source.as_path(), source.join(".git").as_path()).expect("the source is a subvolume");
    let stage: PathBuf = root.join("stage");
    crate::fs_btrfs_ioctl::snapshot_create_at(source.as_path(), stage.as_path()).expect("the snapshot");
    assert_eq!(proved_created_after_the_record(stage.as_path(), &origin), Ok(()), "the filesystem says this snapshot followed the recorded transaction");
    remove(root.as_path());
}

/// A snapshot a person made before the record is not proved,
/// because the filesystem says it was made in or before the transaction the record names.
#[cfg(target_os = "linux")]
#[test]
fn a_snapshot_made_before_the_record_is_not_proved() {
    let Some((root, source)) = source_and_a_written_journal("origin-before") else { return };
    let person: PathBuf = root.join("a-persons-snapshot");
    crate::fs_btrfs_ioctl::snapshot_create_at(source.as_path(), person.as_path()).expect("the person's snapshot");
    write_atomic(journal_root(source.join(".git").as_path()).join("anchor").as_path(), b"anchor\n", "anchor").expect("a later durable write");
    let origin: SnapshotOrigin = origin_of_the_source(source.as_path(), source.join(".git").as_path()).expect("the source is a subvolume");
    let reason: String = proved_created_after_the_record(person.as_path(), &origin).expect_err("the person's snapshot was made before the record");
    assert!(reason.contains("made in transaction"), "the reason names when the filesystem says it was made: {reason}");
    remove(root.as_path());
}

/// A snapshot of another source made after the record is not proved,
/// because the filesystem says it was taken from elsewhere.
#[cfg(target_os = "linux")]
#[test]
fn a_snapshot_of_another_source_is_not_proved() {
    let Some((root, source)) = source_and_a_written_journal("origin-elsewhere") else { return };
    let elsewhere: PathBuf = root.join("elsewhere");
    crate::fs_btrfs_ioctl::subvolume_create_at(elsewhere.as_path()).expect("another subvolume");
    let origin: SnapshotOrigin = origin_of_the_source(source.as_path(), source.join(".git").as_path()).expect("the source is a subvolume");
    let stage: PathBuf = root.join("stage");
    crate::fs_btrfs_ioctl::snapshot_create_at(elsewhere.as_path(), stage.as_path()).expect("a snapshot of another source");
    let reason: String = proved_created_after_the_record(stage.as_path(), &origin).expect_err("the snapshot is of another source");
    assert!(reason.contains("was not taken from the source"), "the reason names what the snapshot was taken from: {reason}");
    remove(root.as_path());
}

/// A plain directory is not proved,
/// whatever it holds,
/// because it is no snapshot at all.
#[cfg(target_os = "linux")]
#[test]
fn a_plain_directory_is_not_proved() {
    let Some((root, source)) = source_and_a_written_journal("origin-plain") else { return };
    let origin: SnapshotOrigin = origin_of_the_source(source.as_path(), source.join(".git").as_path()).expect("the source is a subvolume");
    let plain: PathBuf = root.join("a-person's-directory");
    std::fs::create_dir_all(plain.as_path()).expect("a plain directory");
    std::fs::write(plain.join("notes.txt"), b"a person's only copy").expect("a person's file");
    let reason: String = proved_created_after_the_record(plain.as_path(), &origin).expect_err("a plain directory is no snapshot");
    assert!(reason.contains("is not a btrfs subvolume root"), "the reason names the shape it found: {reason}");
    remove(root.as_path());
}

/// A source that is no subvolume names no origin,
/// so the record keeps its folder rather than delete it.
#[cfg(target_os = "linux")]
#[test]
fn a_source_that_is_no_subvolume_names_no_origin() {
    let root: PathBuf = crate::test_support::fixture("origin-no-subvolume");
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join(".git").as_path()).expect("a plain checkout");
    assert_eq!(origin_of_the_source(source.as_path(), source.join(".git").as_path()), None);
    remove(root.as_path());
}
