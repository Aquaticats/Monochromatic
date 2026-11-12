//! Controls for the evidence a record written before a snapshot leaves:
//! the source it is taken from,
//! and the instant its creation cannot be older than.

use super::*;
use crate::test_support::remove;
use std::path::PathBuf;

/// How long a control waits so that a creation and a record fall in different ticks of the coarse clock the filesystem
/// stamps a subvolume's creation with.
/// Why a number this large:
///  one tick is a millisecond at the coarsest,
/// and the gap only has to be wider than the clock's own resolution.
const ONE_COARSE_TICK: std::time::Duration = std::time::Duration::from_millis(20);

/// A subvolume that stands for a checkout,
/// with one durable write in it,
/// as the source of a snapshot.
#[cfg(target_os = "linux")]
fn a_source_subvolume(label: &str) -> Option<(PathBuf, PathBuf)> {
    let root: PathBuf = crate::fs_btrfs_test_support::btrfs_fixture(label)?;
    let source: PathBuf = root.join("source");
    crate::fs_btrfs_ioctl::subvolume_create_at(source.as_path()).expect("the source subvolume");
    std::fs::write(source.join("tracked.txt"), b"tracked").expect("a file");
    return Some((root, source));
}

/// The snapshot this command's own creation makes is proved by the source it was taken from and by the instant the
/// record holds.
#[cfg(target_os = "linux")]
#[test]
fn a_snapshot_made_from_the_recorded_source_after_the_record_is_proved() {
    let Some((root, source)) = a_source_subvolume("origin-after") else { return };
    let origin: SnapshotOrigin = origin_of_the_source(source.as_path()).expect("the source is a subvolume");
    let stage: PathBuf = root.join("stage");
    crate::fs_btrfs_ioctl::snapshot_create_at(source.as_path(), stage.as_path()).expect("the snapshot");
    assert_eq!(prove_made_from_the_recorded_source(stage.as_path(), &origin), Ok(()), "the filesystem says this snapshot was made from the recorded source, after the recorded instant");
    remove(root.as_path());
}

/// A snapshot a person made before the record is not proved,
/// because the filesystem says it came into being before the instant the record names.
/// Why the control reads this way:
///  a reaping that proves a folder by the transaction that created it deletes the person's snapshot and the only copy
///  of a file in it,
///  because the source's generation is older than the last committed transaction;
///  this control holds the state in which that misreading happens, without a transaction anywhere in the proof.
#[cfg(target_os = "linux")]
#[test]
fn a_snapshot_made_before_the_record_is_not_proved() {
    let Some((root, source)) = a_source_subvolume("origin-before") else { return };
    let person: PathBuf = root.join("a-persons-snapshot");
    crate::fs_btrfs_ioctl::snapshot_create_at(source.as_path(), person.as_path()).expect("the person's snapshot");
    std::thread::sleep(ONE_COARSE_TICK);
    let origin: SnapshotOrigin = origin_of_the_source(source.as_path()).expect("the source is a subvolume");
    let reason: String = prove_made_from_the_recorded_source(person.as_path(), &origin).expect_err("the person's snapshot was made before the record");
    assert!(reason.contains("came into being before this command wrote the record"), "the reason names when the filesystem says it was made: {reason}");
    remove(root.as_path());
}

/// A snapshot of another source made after the record is not proved,
/// because the filesystem says it was taken from elsewhere.
#[cfg(target_os = "linux")]
#[test]
fn a_snapshot_of_another_source_is_not_proved() {
    let Some((root, source)) = a_source_subvolume("origin-elsewhere") else { return };
    let elsewhere: PathBuf = root.join("elsewhere");
    crate::fs_btrfs_ioctl::subvolume_create_at(elsewhere.as_path()).expect("another subvolume");
    let origin: SnapshotOrigin = origin_of_the_source(source.as_path()).expect("the source is a subvolume");
    let stage: PathBuf = root.join("stage");
    crate::fs_btrfs_ioctl::snapshot_create_at(elsewhere.as_path(), stage.as_path()).expect("a snapshot of another source");
    let reason: String = prove_made_from_the_recorded_source(stage.as_path(), &origin).expect_err("the snapshot is of another source");
    assert!(reason.contains("was not taken from the source"), "the reason names what the snapshot was taken from: {reason}");
    remove(root.as_path());
}

/// A plain directory is not proved,
/// whatever it holds,
/// because it is no snapshot at all.
#[cfg(target_os = "linux")]
#[test]
fn a_plain_directory_is_not_proved() {
    let Some((root, source)) = a_source_subvolume("origin-plain") else { return };
    let origin: SnapshotOrigin = origin_of_the_source(source.as_path()).expect("the source is a subvolume");
    let plain: PathBuf = root.join("a-person's-directory");
    std::fs::create_dir_all(plain.as_path()).expect("a plain directory");
    std::fs::write(plain.join("notes.txt"), b"a person's only copy").expect("a person's file");
    let reason: String = prove_made_from_the_recorded_source(plain.as_path(), &origin).expect_err("a plain directory is no snapshot");
    assert!(reason.contains("is not a btrfs subvolume root"), "the reason names the shape it found: {reason}");
    remove(root.as_path());
}

/// A source whose record cannot be read names no origin,
/// so the record keeps its folder rather than delete it.
/// Why a plain directory is not this case:
///  `BTRFS_IOC_GET_SUBVOL_INFO` answers for the subvolume that holds any directory,
///  so a plain directory names the subvolume it sits in and never itself,
///  and a plain directory takes no snapshot at all (the kernel refuses it, as the control below shows),
///  so no folder ever exists for such a source to be judged by.
#[cfg(target_os = "linux")]
#[test]
fn a_source_that_cannot_be_read_names_no_origin() {
    let root: PathBuf = crate::test_support::fixture("origin-no-subvolume");
    assert_eq!(origin_of_the_source(root.join("absent").as_path()), None);
    let plain: PathBuf = root.join("a-plain-directory");
    std::fs::create_dir_all(plain.join(".git").as_path()).expect("a plain checkout");
    let refused = crate::fs_btrfs_ioctl::snapshot_create_at(plain.as_path(), root.join("stage").as_path());
    assert!(refused.is_err(), "a plain directory is no source of a snapshot, so nothing is ever created at the stage name");
    remove(root.as_path());
}
