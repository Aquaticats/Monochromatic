//! Controls for the btrfs ioctl wrappers:
//! the request-number encoding and structure sizes (everywhere),
//! and one real
//!  call of each ioctl on a btrfs fixture (where btrfs exists;
//! elsewhere reported by the fixture helper).

use super::*;
use crate::fs_btrfs_test_support::{btrfs_fixture, removal_allowed};
use crate::fs_kind::{is_subvolume_root, BTRFS_SUBVOLUME_ROOT_INODE};
use crate::test_support::{fixture, remove};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The three requests as `_IOW` computes them (`include/uapi/linux/btrfs.h`:
/// type 0x94,
/// numbers 14,
/// 15 and 23,
/// a 4096-byte
///  argument) under the asm-generic encoding that x86,
/// arm,
/// arm64,
/// riscv,
/// s390 and loongarch use:
/// direction 1 at bit 30.
const GENERIC_VALUES: [u64; 3] = [0x5000_940e, 0x5000_940f, 0x5000_9417];

/// The same requests under the encoding of powerpc,
/// mips,
/// sparc and alpha (direction 4 at bit 29) and of parisc
///  (direction 2 at bit 30):
/// both give the value the kernel reports for `BTRFS_IOC_SUBVOL_CREATE` on powerpc64le.
const WIDE_VALUES: [u64; 3] = [0x9000_940e, 0x9000_940f, 0x9000_9417];

/// The positive control of the encoding:
/// the same function reproduces `libc::FICLONE`,
/// which `libc` defines as
///  `_IOW(0x94, 9, int)` for each architecture,
/// before the btrfs numbers are trusted.
#[test]
fn the_encoding_reproduces_the_clone_request() {
    assert_eq!(ioc_write(9, std::mem::size_of::<libc::c_int>()), libc::FICLONE);
}

/// Each encoding gives the numbers the kernel headers give under it,
/// and the one for the architecture this is built for
///  gives the constants the code uses.
#[test]
fn the_request_numbers_equal_the_header_values_of_every_encoding() {
    for (layout, expected) in [(GENERIC_IOC_LAYOUT, GENERIC_VALUES), (WIDE_DIRECTION_IOC_LAYOUT, WIDE_VALUES), (PARISC_IOC_LAYOUT, WIDE_VALUES)] {
        let computed: [u64; 3] = [
            u64::from(ioc_write_with(layout, 14, VOLUME_ARGUMENT_BYTES)),
            u64::from(ioc_write_with(layout, 15, VOLUME_ARGUMENT_BYTES)),
            u64::from(ioc_write_with(layout, 23, VOLUME_ARGUMENT_BYTES)),
        ];
        assert_eq!(computed, expected, "{layout:?}");
    }
    let on_this_architecture: [u64; 3] = [BTRFS_IOC_SUBVOL_CREATE as u64, BTRFS_IOC_SNAP_DESTROY as u64, BTRFS_IOC_SNAP_CREATE_V2 as u64];
    let wide: bool = HOST_IOC_LAYOUT == WIDE_DIRECTION_IOC_LAYOUT;
    assert_eq!(on_this_architecture, if wide { WIDE_VALUES } else { GENERIC_VALUES });
}

/// Both argument structures are 4096 bytes,
/// as the request numbers state,
/// and the name starts where the header says.
#[test]
fn the_argument_structures_have_the_size_the_requests_state() {
    assert_eq!(std::mem::size_of::<VolArgs>(), VOLUME_ARGUMENT_BYTES);
    assert_eq!(std::mem::size_of::<VolArgsV2>(), VOLUME_ARGUMENT_BYTES);
    assert_eq!(std::mem::offset_of!(VolArgs, name), 8);
    assert_eq!(std::mem::offset_of!(VolArgsV2, name), 8 + 8 + 8 + 32);
    // The fields of the subvolume's own record start where the header puts them: the tree id, the 256-byte name,
    // parent id, dir id, generation and flags, then the three UUIDs and the transactions.
    assert_eq!(std::mem::size_of::<SubvolumeInfoArgs>(), SUBVOLUME_INFO_BYTES);
    assert_eq!(std::mem::offset_of!(SubvolumeInfoArgs, uuid), 8 + 256 + 8 * 4);
    assert_eq!(std::mem::offset_of!(SubvolumeInfoArgs, parent_uuid), 8 + 256 + 8 * 4 + 16);
    assert_eq!(std::mem::offset_of!(SubvolumeInfoArgs, otransid), 8 + 256 + 8 * 4 + 16 * 3 + 8);
}

/// The request number of the subvolume's own record equals the header's `_IOR(0x94, 60, 504)` under each encoding,
/// and the one this build uses is the value this host's kernel answered.
#[test]
fn the_subvolume_info_request_equals_the_header_values_of_every_encoding() {
    for (layout, expected) in [(GENERIC_IOC_LAYOUT, 0x81f8_943cu64), (WIDE_DIRECTION_IOC_LAYOUT, 0x41f8_943c), (PARISC_IOC_LAYOUT, 0x41f8_943c)] {
        assert_eq!(u64::from(ioc_read_with(layout, 60, SUBVOLUME_INFO_BYTES)), expected, "{layout:?}");
    }
    let wide: bool = HOST_IOC_LAYOUT == WIDE_DIRECTION_IOC_LAYOUT;
    assert_eq!(u64::from(BTRFS_IOC_GET_SUBVOL_INFO as u32), if wide { 0x41f8_943c } else { 0x81f8_943c });
}

/// A name that is not one plain component is refused with `EINVAL` before any call,
/// for every wrapper.
#[test]
fn a_name_that_is_not_one_component_is_refused_before_any_call() {
    let root: PathBuf = fixture("fs-ioctl-names");
    let long: String = "n".repeat(300);
    for text in ["", ".", "..", "a/b", "a\0b", long.as_str()] {
        let name = std::ffi::OsStr::new(text);
        for error in [
            subvolume_create(root.as_path(), name).expect_err("create refuses"),
            snapshot_create(root.as_path(), root.as_path(), name).expect_err("snapshot refuses"),
            snapshot_destroy(root.as_path(), name).expect_err("destroy refuses"),
        ] {
            assert_eq!(error.errno, libc::EINVAL, "{error}");
        }
    }
    remove(root.as_path());
}

/// A parent that does not exist is a typed error with the missing parent's error number,
/// never a panic.
#[test]
fn a_missing_parent_is_a_typed_error() {
    let root: PathBuf = fixture("fs-ioctl-parent");
    let error: PrimitiveError = subvolume_create(root.join("absent").as_path(), std::ffi::OsStr::new("x")).expect_err("no parent");
    assert_eq!(error.errno, libc::ENOENT);
    remove(root.as_path());
}

/// A path with no last component cannot be split.
#[test]
fn a_path_without_a_name_cannot_be_split() {
    let error: PrimitiveError = split_parent(Path::new("/"), "BTRFS_IOC_SUBVOL_CREATE").expect_err("the root has no name");
    assert_eq!(error.errno, libc::EINVAL);
    let (parent, name) = split_parent(Path::new("relative"), "x").expect("a bare name");
    assert_eq!((parent, name), (PathBuf::from("."), std::ffi::OsString::from("relative")));
}

/// On a filesystem that is not btrfs the ioctls answer with a feature-missing error,
/// which a ladder falls through on.
/// On btrfs this control has nothing to check,
/// and it says so in one line rather than passing without a word.
#[test]
fn off_btrfs_the_ioctls_report_a_missing_feature() {
    let root: PathBuf = fixture("fs-ioctl-off");
    if crate::fs_kind::filesystem_kind(root.as_path()) == Ok(crate::fs_kind::FilesystemKind::Btrfs) {
        eprintln!("BTRFS CONTROL NOT RUN: off-btrfs-ioctls (fixture root {} is on btrfs, and this control needs a filesystem that is not btrfs)", root.display());
        remove(root.as_path());
        return;
    }
    let error: PrimitiveError = subvolume_create_at(root.join("sub").as_path()).expect_err("not btrfs");
    assert!(error.means_feature_missing(), "{error}");
    let unread = subvolume_info(root.as_path()).expect_err("no subvolume record is readable here");
    assert!(unread.means_feature_missing(), "{unread}");
    remove(root.as_path());
}

/// One real call of each ioctl:
/// create makes a subvolume root,
/// snapshot makes a second one that holds the source's
///  files and shares its data,
/// and destroy removes it (by the ioctl where the mount allows it,
/// else the ioctl is refused
///  with `EPERM` and `rmdir` removes the empty subvolume).
#[test]
fn btrfs_create_snapshot_and_destroy_each_work_once() {
    let Some(root) = btrfs_fixture("fs-ioctl-real") else { return };
    let source: PathBuf = root.join("source");
    subvolume_create_at(source.as_path()).expect("create");
    assert_eq!(std::fs::metadata(&source).expect("stat").ino(), BTRFS_SUBVOLUME_ROOT_INODE);
    std::fs::write(source.join("file"), b"content").expect("file in the source");

    let copy: PathBuf = root.join("copy");
    snapshot_create_at(source.as_path(), copy.as_path()).expect("snapshot");
    assert_eq!(is_subvolume_root(copy.as_path()), Ok(true));
    assert_eq!(std::fs::read(copy.join("file")).expect("file in the snapshot"), b"content");

    // The record the kernel writes into the snapshot names the subvolume it was taken from, and a later transaction.
    let facts: SubvolumeInfo = subvolume_info(copy.as_path()).expect("the facts of the snapshot");
    let origin: SubvolumeInfo = subvolume_info(source.as_path()).expect("the facts of the source");
    assert_eq!(facts.parent_uuid, origin.uuid, "the snapshot names the subvolume it was taken from");
    // The source is no snapshot of anything, so its own parent is nobody, and the snapshot's record carries a transaction.
    assert_eq!(origin.parent_uuid, [0u8; 16], "a plain subvolume was taken from no source");
    assert_ne!(facts.transid_at_creation, 0, "the snapshot's record names the transaction that created it");
    let holding: PathBuf = root.join("plain-for-facts");
    std::fs::create_dir(holding.as_path()).expect("plain directory");
    assert_ne!(
        subvolume_info(holding.as_path()).expect("the facts of the holding subvolume").uuid,
        origin.uuid,
        "a plain directory answers with the subvolume that holds it, not with the source"
    );

    let again = snapshot_create_at(source.as_path(), copy.as_path()).expect_err("the name exists");
    assert_eq!(again.errno, libc::EEXIST);
    let plain: PathBuf = root.join("plain");
    std::fs::create_dir(&plain).expect("plain directory");
    let not_a_subvolume = snapshot_create_at(plain.as_path(), root.join("from-plain").as_path()).expect_err("a plain directory is no source");
    assert!(not_a_subvolume.means_feature_missing(), "{not_a_subvolume}");

    std::fs::remove_file(copy.join("file")).expect("empty the snapshot");
    if removal_allowed(root.as_path()) {
        subvolume_destroy_at(copy.as_path()).expect("destroy where the mount allows it");
    } else {
        let refused = subvolume_destroy_at(copy.as_path()).expect_err("the mount option is absent");
        assert_eq!(refused.errno, libc::EPERM, "{refused}");
        std::fs::remove_dir(&copy).expect("rmdir removes an empty subvolume");
    }
    assert!(!copy.exists());
    remove(root.as_path());
}

/// The destroy ioctl on a subvolume that holds a nested subvolume is refused,
/// so removal goes deepest first.
#[test]
fn btrfs_a_subvolume_with_a_nested_subvolume_is_not_destroyed_in_one_step() {
    let Some(root) = btrfs_fixture("fs-ioctl-nested") else { return };
    if !removal_allowed(root.as_path()) {
        remove(root.as_path());
        return;
    }
    let outer: PathBuf = root.join("outer");
    subvolume_create_at(outer.as_path()).expect("outer");
    subvolume_create_at(outer.join("inner").as_path()).expect("inner");
    let refused = subvolume_destroy_at(outer.as_path()).expect_err("a nested subvolume blocks the destroy");
    assert_eq!(refused.errno, libc::ENOTEMPTY, "{refused}");
    subvolume_destroy_at(outer.join("inner").as_path()).expect("inner first");
    subvolume_destroy_at(outer.as_path()).expect("then outer");
    remove(root.as_path());
}

/// The two requests added for the destroy by id,
/// as `_IOW` and `_IOWR` compute them under each encoding (type 0x94,
/// numbers
///  63 and 18,
/// a 4096-byte argument):
/// the asm-generic encoding,
/// the wide one,
/// and parisc's.
#[test]
fn the_requests_for_the_destroy_by_id_equal_the_header_values_of_every_encoding() {
    const GENERIC_DESTROY_V2: u32 = 0x5000_943f;
    const WIDE_DESTROY_V2: u32 = 0x9000_943f;
    const LOOKUP: u32 = 0xd000_9412;
    for (layout, destroy) in [(GENERIC_IOC_LAYOUT, GENERIC_DESTROY_V2), (WIDE_DIRECTION_IOC_LAYOUT, WIDE_DESTROY_V2), (PARISC_IOC_LAYOUT, WIDE_DESTROY_V2)] {
        assert_eq!(ioc_write_with(layout, 63, VOLUME_ARGUMENT_BYTES), destroy, "{layout:?}");
        assert_eq!(ioc_read_write_with(layout, 18, VOLUME_ARGUMENT_BYTES), LOOKUP, "{layout:?}");
    }
    assert_eq!(BTRFS_IOC_INO_LOOKUP as u64, u64::from(LOOKUP));
    let wide: bool = HOST_IOC_LAYOUT == WIDE_DIRECTION_IOC_LAYOUT;
    assert_eq!(BTRFS_IOC_SNAP_DESTROY_V2 as u64, u64::from(if wide { WIDE_DESTROY_V2 } else { GENERIC_DESTROY_V2 }));
}

/// A subvolume's id names it for good:
/// two subvolumes have two ids;
/// the destroy by id removes the one whose id it is given,
///  through a directory of the filesystem that is not its parent,
/// and leaves the other (where the mount allows it;
/// else
///  the kernel refuses with `EPERM`,
/// which also shows the request was understood and the id resolved).
#[test]
fn btrfs_the_destroy_by_id_removes_the_subvolume_whose_id_it_is_given() {
    let Some(root) = btrfs_fixture("fs-ioctl-by-id") else { return };
    let (first, second) = (root.join("first"), root.join("second"));
    subvolume_create_at(first.as_path()).expect("first");
    subvolume_create_at(second.as_path()).expect("second");
    let first_id: u64 = subvolume_id(&std::fs::File::open(first.as_path()).expect("open"), first.as_path()).expect("id of the first");
    let second_id: u64 = subvolume_id(&std::fs::File::open(second.as_path()).expect("open"), second.as_path()).expect("id of the second");
    assert_ne!(first_id, second_id);
    // A file inside a subvolume belongs to that subvolume's id.
    std::fs::write(first.join("file"), b"x").expect("file");
    assert_eq!(subvolume_id(&std::fs::File::open(first.join("file")).expect("open"), first.as_path()).expect("id of a file"), first_id);
    // The path moves; the id still leads to the same subvolume.
    std::fs::remove_file(first.join("file")).expect("empty it");
    std::fs::rename(first.as_path(), root.join("renamed")).expect("rename");
    let through: std::fs::File = std::fs::File::open(root.as_path()).expect("any directory of the filesystem");
    if removal_allowed(root.as_path()) {
        snapshot_destroy_by_id(&through, first_id, first.as_path()).expect("destroy by id");
        assert!(!root.join("renamed").exists(), "the subvolume with that id is gone, under its new name");
        assert!(second.is_dir(), "the other one is untouched");
    } else {
        let refused = snapshot_destroy_by_id(&through, first_id, first.as_path()).expect_err("the mount option is absent");
        assert_eq!(refused.errno, libc::EPERM, "{refused}");
        assert!(root.join("renamed").is_dir() && second.is_dir());
    }
    remove(root.as_path());
}
