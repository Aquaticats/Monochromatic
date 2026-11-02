//! Controls for the directory-relative primitives: a link is never followed, a mount-crossing directory is refused, an
//!  existing name is never overwritten, and entries are created, timed and removed below a held handle.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// `..`, an absolute path and `.` are refused, plain names are kept in order.
#[test]
fn only_plain_components_are_names() {
    assert_eq!(plain_components(Path::new("a/b/c")).expect("plain"), vec![OsString::from("a"), OsString::from("b"), OsString::from("c")]);
    for refused in ["../x", "/abs", "a/../b", "./a"] {
        assert!(plain_components(Path::new(refused)).is_err(), "{refused}");
    }
}

/// A directory that is a link is an error at any depth, and the same path through real directories opens.
#[test]
fn a_link_in_the_way_is_an_error_and_never_a_detour() {
    let root: PathBuf = fixture("beneath-link");
    std::fs::create_dir_all(root.join("real/inner")).expect("real");
    std::fs::create_dir_all(root.join("elsewhere/inner")).expect("elsewhere");
    std::os::unix::fs::symlink(root.join("elsewhere"), root.join("real/linked")).expect("link");
    let held: File = open_directory(root.join("real").as_path()).expect("root");
    assert!(open_directory_beneath(&held, &[OsString::from("inner")]).is_ok(), "positive control");
    assert!(open_directory_beneath(&held, &[OsString::from("linked"), OsString::from("inner")]).is_err(), "a link at the first component");
    assert!(open_directory_at(&held, OsStr::new("linked")).is_err(), "a link as the directory itself");
    assert!(open_directory(root.join("real/linked").as_path()).is_err(), "a link as the last component of a path");
    remove(root.as_path());
}

/// A file that is a link is not opened for reading, and an existing name is not created over, a link included.
#[test]
fn a_file_link_is_not_opened_and_an_existing_name_is_not_replaced() {
    let root: PathBuf = fixture("beneath-file");
    std::fs::write(root.join("target.txt"), b"target").expect("target");
    std::os::unix::fs::symlink("target.txt", root.join("link")).expect("link");
    let held: File = open_directory(root.as_path()).expect("root");
    assert!(open_file_at(&held, OsStr::new("target.txt")).is_ok(), "positive control");
    assert!(open_file_at(&held, OsStr::new("link")).is_err(), "a link is not followed");
    assert!(create_file_at(&held, OsStr::new("link"), 0o600).is_err(), "a link is not written through");
    assert!(create_file_at(&held, OsStr::new("target.txt"), 0o600).is_err(), "a file is not replaced");
    assert_eq!(std::fs::read(root.join("target.txt")).expect("kept"), b"target");
    remove(root.as_path());
}

/// A name with a slash, a NUL byte or nothing is refused whatever the call.
#[test]
fn a_name_is_one_plain_component() {
    let root: PathBuf = fixture("beneath-name");
    let held: File = open_directory(root.as_path()).expect("root");
    for refused in [OsStr::new(""), OsStr::new("a/b"), OsStr::from_bytes(b"nul\0byte")] {
        assert!(create_file_at(&held, refused, 0o600).is_err());
        assert!(open_directory_at(&held, refused).is_err());
        assert!(remove_at(&held, refused, false).is_err());
    }
    remove(root.as_path());
}

/// A directory is made once, found again without error, and a link or a file with its name is an error.
#[test]
fn a_directory_is_made_once_and_a_link_with_its_name_is_an_error() {
    let root: PathBuf = fixture("beneath-mkdir");
    std::fs::create_dir(root.join("outside")).expect("outside");
    std::os::unix::fs::symlink(root.join("outside"), root.join("linked")).expect("link");
    std::fs::write(root.join("file"), b"").expect("file");
    let held: File = open_directory(root.as_path()).expect("root");
    assert!(make_directory_at(&held, OsStr::new("new"), 0o755).expect("made"));
    assert!(!make_directory_at(&held, OsStr::new("new"), 0o755).expect("found"));
    assert!(make_directory_at(&held, OsStr::new("linked"), 0o755).is_err(), "a link to a directory is not a directory here");
    assert!(make_directory_at(&held, OsStr::new("file"), 0o755).is_err());
    remove(root.as_path());
}

/// A link is created and read below the handle, its own times are set without touching the target, and removal takes the
///  link, not the target.
#[test]
fn links_are_created_timed_and_removed_below_the_handle() {
    let root: PathBuf = fixture("beneath-symlink");
    std::fs::write(root.join("target.txt"), b"target").expect("target");
    let held: File = open_directory(root.as_path()).expect("root");
    symlink_at(Path::new("target.txt"), &held, OsStr::new("link")).expect("link");
    assert_eq!(read_link_at(&held, OsStr::new("link")).expect("read"), PathBuf::from("target.txt"));
    let times: [libc::timespec; 2] = [libc::timespec { tv_sec: 1_000_000, tv_nsec: 5 }, libc::timespec { tv_sec: 2_000_000, tv_nsec: 7 }];
    set_times_at(&held, OsStr::new("link"), times).expect("times");
    let metadata = entry_metadata_at(&held, OsStr::new("link")).expect("metadata");
    assert!(metadata.file_type().is_symlink());
    assert_eq!((metadata.mtime(), metadata.mtime_nsec()), (2_000_000, 7));
    assert_ne!(std::fs::metadata(root.join("target.txt")).expect("target").mtime(), 2_000_000, "the target keeps its time");
    remove_at(&held, OsStr::new("link"), false).expect("remove");
    assert!(root.join("target.txt").exists() && !root.join("link").exists());
    assert!(remove_at(&held, OsStr::new("link"), false).is_err(), "a name that is gone is an error the caller may ignore");
    remove(root.as_path());
}

/// A directory with something in it is not removed as an empty one, and a nested subvolume is another device.
#[test]
fn a_full_directory_stays_and_a_subvolume_is_another_device() {
    let Some(root) = btrfs_fixture("beneath-device") else { return };
    std::fs::create_dir_all(root.join("full")).expect("full");
    std::fs::write(root.join("full/file"), b"x").expect("file");
    subvolume_create_at(root.join("sub").as_path()).expect("subvolume");
    let held: File = open_directory(root.as_path()).expect("root");
    assert!(remove_at(&held, OsStr::new("full"), true).is_err(), "not empty");
    assert!(open_directory_beneath(&held, &[OsString::from("full")]).is_ok(), "positive control: the same device");
    assert!(open_directory_beneath(&held, &[OsString::from("sub")]).is_err(), "a subvolume is not crossed");
    assert!(device_of(&held).is_ok());
    remove(root.as_path());
}
