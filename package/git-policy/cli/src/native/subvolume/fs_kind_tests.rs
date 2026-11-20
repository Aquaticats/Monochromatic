//! Controls for the filesystem kind and the subvolume-root check.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// The two magic numbers map to their kinds,
/// in the signed and the unsigned spelling,
/// and others are `Other`.
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

/// A real directory answers with some kind and free space;
/// a missing path is a typed error that names `statfs`.
#[test]
fn the_kind_of_a_real_directory_is_read_and_a_missing_path_is_a_typed_error() {
    let root: PathBuf = fixture("fs-kind-real");
    let (_kind, free) = filesystem_facts(root.as_path()).expect("the fixture root answers");
    assert!(free > 0, "a fixture root with no free byte cannot hold a fixture");
    let error: PrimitiveError = filesystem_kind(root.join("absent").as_path()).expect_err("a missing path has no filesystem");
    assert_eq!((error.operation, error.errno), ("statfs", libc::ENOENT));
    remove(root.as_path());
}

/// Whatever the fixture filesystem is,
/// a plain directory made inside it is not a subvolume root unless it is on btrfs
///  with inode 256,
/// and on a filesystem other than btrfs inode 256 never counts.
#[test]
fn a_plain_directory_is_not_a_subvolume_root() {
    let root: PathBuf = fixture("fs-kind-plain");
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    assert_eq!(is_subvolume_root(plain.as_path()), Ok(false));
    remove(root.as_path());
}

/// The positive control on btrfs:
/// a created subvolume is a root,
/// its parent directory is not,
/// and a file is not.
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

/// What:
///  Every kind of filesystem answers whether it may share data blocks:
/// btrfs,
/// overlayfs and ZFS may,
/// and any other
///  kind does not,
/// because the one filesystem whose clone support the kind does not hold is XFS,
/// which the
///  reader asks the kernel about.
/// Why:
///   A ZFS path the clone rung never tries would leave the same-dataset proof unreachable,
/// and a kind the rung tries on
///  another filesystem would recast a byte copy as a reflink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(['btrfs', 'overlayfs', 'zfs'].every(kindMayClone)).toBe(true); expect(kindMayClone('other')).toBe(false);
/// ```
#[test]
fn every_kind_answers_whether_it_may_clone() {
    assert!(kind_may_clone(FilesystemKind::Btrfs));
    assert!(kind_may_clone(FilesystemKind::Overlayfs));
    assert!(kind_may_clone(FilesystemKind::Zfs));
    assert!(!kind_may_clone(FilesystemKind::Other));
}

/// What:
///  A ZFS mount is its own kind,
/// in the signed and the unsigned spelling of `f_type`.
/// Why:
///   Rung 2 of the cheap-worktree ladder may reflink only inside one ZFS dataset,
/// so the ladder has to tell a ZFS mount from btrfs,
/// overlayfs
///  and everything else before it can ask which dataset a path is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(classifyMagic(0x2fc12fc1n)).toBe('zfs');
/// ```
#[test]
fn zfs_magic_number_maps_to_its_kind() {
    assert_eq!(ZFS_SUPER_MAGIC, 0x2fc1_2fc1, "the value `include/sys/fs/zfs.h` of OpenZFS `zfs-2.4.4` defines as `ZFS_SUPER_MAGIC` and writes into `statfs`, which the libc crate does not define");
    assert_eq!(classify_magic(ZFS_SUPER_MAGIC), FilesystemKind::Zfs);
    let extended: u64 = ZFS_SUPER_MAGIC | 0xffff_ffff_0000_0000;
    assert_eq!(classify_magic(extended), FilesystemKind::Zfs);
}

/// The environment variable that acknowledges "this run has no ZFS fixture root".
/// The container gate image sets it,
/// because its fixture root is a tmpfs and never a ZFS dataset,
/// and the btrfs host task sets it too,
/// because its fixture root is btrfs by definition;
/// a run that believes it is on ZFS leaves it unset,
/// so this control fails instead of
///  recording a pass for coverage it never had.
const ZFS_ABSENT_ACKNOWLEDGED_VARIABLE: &str = "CLI_GIT_ZFS_ABSENT_ACKNOWLEDGED";

/// What:
///  A directory on a real ZFS mount answers with the ZFS kind.
/// Why:
///   The magic number is only as good as the `statfs` answer it classifies,
/// and this host's checkout sits on a ZFS dataset;
///  a container whose fixture root is not ZFS prints the line that says this control did not run,
/// and
///  fails unless the run acknowledged that it has no ZFS fixture root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (kindOf(fixtureRoot) !== 'zfs') expectAcknowledged('CLI_GIT_ZFS_ABSENT_ACKNOWLEDGED'); else expect(kindOf(fixtureRoot)).toBe('zfs');
/// ```
#[test]
fn zfs_a_real_mount_answers_with_its_kind() {
    let root: PathBuf = fixture("fs-kind-zfs");
    let kind: FilesystemKind = filesystem_kind(root.as_path()).expect("the fixture root answers");
    if kind != FilesystemKind::Zfs {
        let acknowledged: bool = std::env::var_os(ZFS_ABSENT_ACKNOWLEDGED_VARIABLE).is_some_and(|value| { return !value.is_empty(); });
        assert!(
            acknowledged,
            "the control zfs_a_real_mount_answers_with_its_kind needs a ZFS fixture root, but {} is on {kind:?}; set CLI_GIT_TEST_ROOT to a directory on ZFS, or set {ZFS_ABSENT_ACKNOWLEDGED_VARIABLE}=1 to state that this run has none",
            root.display()
        );
        eprintln!("ZFS CONTROL NOT RUN: zfs_a_real_mount_answers_with_its_kind (fixture root {} is on {kind:?}; the guest fixture of the ZFS plan's step 2 runs it)", root.display());
        remove(root.as_path());
        return;
    }
    assert_eq!(kind, FilesystemKind::Zfs);
    let inside: PathBuf = root.join("plain");
    std::fs::create_dir(&inside).expect("plain directory");
    assert_eq!(filesystem_kind(inside.as_path()), Ok(FilesystemKind::Zfs), "a directory made in a ZFS mount is on that dataset");
    remove(root.as_path());
}
