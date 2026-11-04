//! Controls for the filesystem kind and the subvolume-root check.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// The two magic numbers map to their kinds, in the signed and the unsigned spelling, and others are `Other`.
#[test]
fn the_magic_numbers_map_to_their_kinds() {
    assert_eq!(classify_magic(libc::BTRFS_SUPER_MAGIC as u64), FilesystemKind::Btrfs);
    assert_eq!(classify_magic(libc::BTRFS_SUPER_MAGIC as u32 as u64), FilesystemKind::Btrfs);
    assert_eq!(classify_magic(libc::OVERLAYFS_SUPER_MAGIC as u64), FilesystemKind::Overlayfs);
    assert_eq!(classify_magic(libc::TMPFS_MAGIC as u64), FilesystemKind::Other);
    assert_eq!(classify_magic(0), FilesystemKind::Other);
}

/// Bits above the low 32 of a sign-extended `f_type` do not hide the kind.
#[test]
fn a_sign_extended_f_type_still_classifies() {
    let extended: u64 = u64::from(libc::BTRFS_SUPER_MAGIC as u32) | 0xffff_ffff_0000_0000;
    assert_eq!(classify_magic(extended), FilesystemKind::Btrfs);
}

/// A real directory answers with some kind and free space; a missing path is a typed error that names `statfs`.
#[test]
fn the_kind_of_a_real_directory_is_read_and_a_missing_path_is_a_typed_error() {
    let root: PathBuf = fixture("fs-kind-real");
    let (_kind, free) = filesystem_facts(root.as_path()).expect("the fixture root answers");
    assert!(free > 0, "a fixture root with no free byte cannot hold a fixture");
    let error: PrimitiveError = filesystem_kind(root.join("absent").as_path()).expect_err("a missing path has no filesystem");
    assert_eq!((error.operation, error.errno), ("statfs", libc::ENOENT));
    remove(root.as_path());
}

/// Whatever the fixture filesystem is, a plain directory made inside it is not a subvolume root unless it is on btrfs
///  with inode 256, and on a filesystem other than btrfs inode 256 never counts.
#[test]
fn a_plain_directory_is_not_a_subvolume_root() {
    let root: PathBuf = fixture("fs-kind-plain");
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    assert_eq!(is_subvolume_root(plain.as_path()), Ok(false));
    remove(root.as_path());
}

/// The positive control on btrfs: a created subvolume is a root, its parent directory is not, and a file is not.
#[test]
fn btrfs_a_created_subvolume_is_a_root_and_a_plain_directory_beside_it_is_not() {
    let Some(root) = btrfs_fixture("fs-kind-btrfs") else { return };
    assert_eq!(filesystem_kind(root.as_path()), Ok(FilesystemKind::Btrfs));
    let subvolume: PathBuf = root.join("sub");
    crate::fs_btrfs_ioctl::subvolume_create_at(subvolume.as_path()).expect("subvolume create");
    assert_eq!(is_subvolume_root(subvolume.as_path()), Ok(true));
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    assert_eq!(is_subvolume_root(plain.as_path()), Ok(false));
    std::fs::write(root.join("file"), b"x").expect("file");
    assert_eq!(is_subvolume_root(root.join("file").as_path()), Ok(false));
    remove(root.as_path());
}
