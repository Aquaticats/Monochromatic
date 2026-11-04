//! Controls for `FICLONE` and the atomic exchange.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// Two directories swap atomically: each path then holds what the other held.
#[test]
fn an_exchange_swaps_two_directories() {
    let root: PathBuf = fixture("fs-exchange");
    let (first, second) = (root.join("first"), root.join("second"));
    std::fs::create_dir(&first).expect("first");
    std::fs::create_dir(&second).expect("second");
    std::fs::write(first.join("mark"), b"one").expect("mark one");
    std::fs::write(second.join("mark"), b"two").expect("mark two");
    exchange_paths(first.as_path(), second.as_path()).expect("exchange");
    assert_eq!(std::fs::read(first.join("mark")).expect("first mark"), b"two");
    assert_eq!(std::fs::read(second.join("mark")).expect("second mark"), b"one");
    remove(root.as_path());
}

/// An exchange with a path that does not exist is a typed error and changes nothing.
#[test]
fn an_exchange_with_a_missing_path_fails_and_changes_nothing() {
    let root: PathBuf = fixture("fs-exchange-missing");
    let present: PathBuf = root.join("present");
    std::fs::create_dir(&present).expect("present");
    let error: PrimitiveError = exchange_paths(present.as_path(), root.join("absent").as_path()).expect_err("one path is missing");
    assert_eq!(error.errno, libc::ENOENT);
    assert!(present.is_dir());
    assert!(!root.join("absent").exists());
    remove(root.as_path());
}

/// A path with a NUL byte is refused before the call.
#[test]
fn an_exchange_path_with_a_nul_byte_is_refused() {
    use std::os::unix::ffi::OsStrExt;
    let bad: PathBuf = PathBuf::from(std::ffi::OsStr::from_bytes(b"a\0b"));
    let error: PrimitiveError = exchange_paths(bad.as_path(), Path::new("/")).expect_err("NUL");
    assert_eq!(error.errno, libc::EINVAL);
}

/// A clone either succeeds with equal content (a reflink filesystem) or fails with a typed feature-missing error and
///  leaves the destination empty; it never panics and never half-copies.
#[test]
fn a_clone_succeeds_or_reports_a_missing_feature() {
    let root: PathBuf = fixture("fs-clone-any");
    std::fs::write(root.join("source"), b"clone me").expect("source");
    let source: File = File::open(root.join("source")).expect("open source");
    let destination: File = std::fs::OpenOptions::new().write(true).create_new(true).open(root.join("dest")).expect("dest");
    match clone_file(&source, &destination, root.join("dest").as_path()) {
        Ok(()) => assert_eq!(std::fs::read(root.join("dest")).expect("read dest"), b"clone me"),
        Err(error) => {
            assert!(error.means_feature_missing(), "{error}");
            assert_eq!(error.operation, "FICLONE");
            assert_eq!(std::fs::read(root.join("dest")).expect("read dest"), b"");
        }
    }
    remove(root.as_path());
}

/// On btrfs the clone always succeeds inside one filesystem and shares nothing observable but content; a clone of a
///  directory descriptor is refused.
#[test]
fn btrfs_a_clone_succeeds_with_equal_content_and_a_directory_is_refused() {
    let Some(root) = btrfs_fixture("fs-clone-btrfs") else { return };
    std::fs::write(root.join("source"), vec![7u8; 200_000]).expect("source");
    let source: File = File::open(root.join("source")).expect("open source");
    let destination: File = std::fs::OpenOptions::new().write(true).create_new(true).open(root.join("dest")).expect("dest");
    clone_file(&source, &destination, root.join("dest").as_path()).expect("clone on btrfs");
    assert_eq!(std::fs::read(root.join("dest")).expect("read"), vec![7u8; 200_000]);
    let directory: File = File::open(&root).expect("directory");
    let error: PrimitiveError = clone_file(&directory, &destination, root.join("dest").as_path()).expect_err("a directory is no clone source");
    assert_eq!(error.operation, "FICLONE");
    remove(root.as_path());
}
