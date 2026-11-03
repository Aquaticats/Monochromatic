//! Controls for the directory-relative primitives: a link is never followed, a mount-crossing directory is refused, an
//!  existing name is never overwritten, and entries are created, timed and removed below a held handle.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The root of a walk for a fixture directory, by its canonical path.
fn open_directory(path: &Path) -> std::io::Result<File> {
    return open_root(std::fs::canonicalize(path)?.as_path());
}

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
    assert!(open_root(root.join("real/linked").as_path()).is_err(), "a link as the last component of a path");
    let through = std::fs::canonicalize(&root).expect("canonical").join("real/linked/inner");
    assert!(open_root(through.as_path()).is_err(), "a link among the ancestors is refused, not followed");
    assert!(open_root(Path::new("relative/path")).is_err(), "a root is an absolute path");
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

/// `.` and `..` are not names: no call takes them, whatever it does with the name.
#[test]
fn dot_and_dot_dot_are_not_names() {
    let root: PathBuf = fixture("beneath-dots");
    std::fs::create_dir_all(root.join("inner")).expect("inner");
    let inner: File = open_directory(root.join("inner").as_path()).expect("inner handle");
    for refused in [".", ".."] {
        assert!(open_directory_at(&inner, OsStr::new(refused)).is_err(), "{refused}");
        assert!(open_directory_beneath(&inner, &[OsString::from(refused)]).is_err(), "{refused}");
        assert!(entry_metadata_at(&inner, OsStr::new(refused)).is_err(), "{refused}");
        assert!(read_link_at(&inner, OsStr::new(refused)).is_err(), "{refused}");
        assert!(identity_at(&inner, OsStr::new(refused)).is_err(), "{refused}");
        assert!(remove_at(&inner, OsStr::new(refused), true).is_err(), "{refused}");
        assert!(set_times_at(&inner, OsStr::new(refused), [libc::timespec { tv_sec: 0, tv_nsec: 0 }; 2]).is_err(), "{refused}");
    }
    remove(root.as_path());
}

/// A name with a slash or an absolute name is refused by the lookups that went through a path before.
#[test]
fn a_lookup_takes_one_plain_name() {
    let root: PathBuf = fixture("beneath-lookup");
    std::fs::create_dir_all(root.join("inner")).expect("inner");
    std::fs::write(root.join("inner/file"), b"x").expect("file");
    std::os::unix::fs::symlink("file", root.join("inner/link")).expect("link");
    let held: File = open_directory(root.as_path()).expect("root");
    assert!(entry_metadata_at(&held, OsStr::new("inner")).is_ok(), "positive control");
    assert!(entry_metadata_at(&held, OsStr::new("inner/file")).is_err(), "several components");
    assert!(entry_metadata_at(&held, OsStr::new("/etc/passwd")).is_err(), "an absolute name");
    assert!(read_link_at(&held, OsStr::new("inner/link")).is_err());
    assert!(read_link_at(&held, OsStr::new("/proc/self/exe")).is_err());
    remove(root.as_path());
}

/// The identity of an object round-trips through its record text, and differs between two objects.
#[test]
fn an_identity_round_trips_and_tells_two_objects_apart() {
    let root: PathBuf = fixture("beneath-identity");
    std::fs::write(root.join("one"), b"1").expect("one");
    std::fs::write(root.join("two"), b"2").expect("two");
    let held: File = open_directory(root.as_path()).expect("root");
    let one: ObjectId = identity_at(&held, OsStr::new("one")).expect("one");
    let two: ObjectId = identity_at(&held, OsStr::new("two")).expect("two");
    assert_ne!(one, two);
    assert_eq!(ObjectId::decode(one.encode().as_str()), Some(one));
    assert_eq!(ObjectId::decode(ObjectId { device: 1, inode: 2, birth: None }.encode().as_str()), Some(ObjectId { device: 1, inode: 2, birth: None }));
    for broken in ["", "1:2", "1:2:3", "a:b:c:d", "1:2:3:4:5"] {
        assert_eq!(ObjectId::decode(broken), None, "{broken}");
    }
    assert_eq!(identity_of(&File::open(root.join("one")).expect("open")).expect("by descriptor"), one);
    remove(root.as_path());
}

/// A lock file is removed only while the name still holds the file that was made; a replacement stays, a directory is never
///  removed, and a name that is gone is nothing.
#[test]
fn only_the_recorded_file_is_removed() {
    let root: PathBuf = fixture("beneath-identical");
    let held: File = open_directory(root.as_path()).expect("root");
    std::fs::write(root.join("made"), b"mine").expect("made");
    let made: ObjectId = identity_at(&held, OsStr::new("made")).expect("identity");
    // Another process replaces the name with its own file of the same name.
    std::fs::rename(root.join("made"), root.join("moved-away")).expect("moved");
    std::fs::write(root.join("made"), b"theirs").expect("a replacement");
    let outcome: Owned = remove_if_identical(&held, OsStr::new("made"), made).expect("outcome");
    assert!(matches!(outcome, Owned::Kept(ref reason) if reason.contains("is not the object that was made")), "{outcome:?}");
    assert_eq!(std::fs::read(root.join("made")).expect("the replacement stays"), b"theirs");
    let moved: ObjectId = identity_at(&held, OsStr::new("moved-away")).expect("identity");
    assert_eq!(remove_if_identical(&held, OsStr::new("moved-away"), moved).expect("removed"), Owned::Removed);
    assert!(!root.join("moved-away").exists());
    assert_eq!(remove_if_identical(&held, OsStr::new("moved-away"), moved).expect("absent"), Owned::Absent);
    std::fs::create_dir(root.join("directory")).expect("directory");
    let directory: ObjectId = identity_at(&held, OsStr::new("directory")).expect("identity");
    let refused: Owned = remove_if_identical(&held, OsStr::new("directory"), directory).expect("outcome");
    assert!(matches!(refused, Owned::Kept(ref reason) if reason.contains("is a directory")), "{refused:?}");
    assert!(root.join("directory").is_dir());
    remove(root.as_path());
}

/// An open file is published under a new name without replacing anything: the file published is the one the descriptor holds,
///  whatever a name that was checked earlier refers to now.
#[test]
fn an_open_file_is_linked_under_a_new_name_and_never_over_an_existing_one() {
    let root: PathBuf = fixture("beneath-link-open");
    let held: File = open_directory(root.as_path()).expect("root");
    let written: File = create_file_at(&held, OsStr::new("lock"), 0o600).expect("lock");
    std::io::Write::write_all(&mut &written, b"the index we wrote").expect("content");
    // Another process takes the lock name for its own file; the descriptor still holds ours.
    std::fs::rename(root.join("lock"), root.join("lock.moved")).expect("lock moved away");
    std::fs::write(root.join("lock"), b"theirs").expect("their lock");
    link_open_file_at(&written, &held, OsStr::new("index")).expect("published");
    assert_eq!(std::fs::read(root.join("index")).expect("index"), b"the index we wrote");
    assert_eq!(std::fs::read(root.join("lock")).expect("lock"), b"theirs");
    assert!(link_open_file_at(&written, &held, OsStr::new("index")).is_err(), "an existing name is not replaced");
    std::fs::remove_file(root.join("lock.moved")).expect("the last name of the file goes");
    std::fs::remove_file(root.join("index")).expect("and the published one");
    assert!(link_open_file_at(&written, &held, OsStr::new("again")).is_err(), "a file with no name left cannot be linked: the publication fails closed");
    std::fs::write(root.join("other"), b"x").expect("other");
    assert!(link_open_file_at(&written, &held, OsStr::new("other")).is_err());
    assert_eq!(std::fs::read(root.join("other")).expect("kept"), b"x");
    remove(root.as_path());
}

/// Two names are exchanged atomically, and a rename without `replace` leaves an existing name alone.
#[test]
fn names_are_exchanged_and_renamed_without_replacing() {
    let root: PathBuf = fixture("beneath-exchange");
    std::fs::write(root.join("a"), b"a").expect("a");
    std::fs::write(root.join("b"), b"b").expect("b");
    let held: File = open_directory(root.as_path()).expect("root");
    exchange_at(&held, OsStr::new("a"), OsStr::new("b")).expect("exchange");
    assert_eq!((std::fs::read(root.join("a")).expect("a"), std::fs::read(root.join("b")).expect("b")), (b"b".to_vec(), b"a".to_vec()));
    assert!(rename_at(&held, OsStr::new("a"), &held, OsStr::new("b"), false).is_err(), "no replace");
    assert_eq!(std::fs::read(root.join("b")).expect("kept"), b"a");
    rename_at(&held, OsStr::new("a"), &held, OsStr::new("c"), false).expect("rename to a free name");
    assert!(root.join("c").exists() && !root.join("a").exists());
    remove(root.as_path());
}
