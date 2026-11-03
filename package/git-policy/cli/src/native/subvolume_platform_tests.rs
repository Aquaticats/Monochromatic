//! Controls for the Linux platform's primitives on a btrfs fixture: they are the kernel's answers, not constants.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::remove;

/// A subvolume made through the platform is a subvolume root, a directory beside it is not, and a path that is not there is
///  an error; the platform reports what the kernel did, so a platform that always said yes or no would fail here.
#[test]
fn btrfs_the_platform_creates_a_subvolume_and_tells_it_from_a_directory() {
    let Some(root) = btrfs_fixture("platform-subvolume") else { return };
    let platform: LinuxPlatform = LinuxPlatform;
    std::fs::create_dir(root.join("plain")).expect("directory");
    assert_eq!(platform.is_subvolume_root(root.join("plain").as_path()), Ok(false));
    assert!(!root.join("made").exists());
    platform.create_subvolume(root.join("made").as_path()).expect("create");
    assert!(root.join("made").is_dir(), "the subvolume exists");
    assert_eq!(platform.is_subvolume_root(root.join("made").as_path()), Ok(true));
    assert!(platform.create_subvolume(root.join("made").as_path()).is_err(), "a second creation is refused by the kernel");
    assert!(platform.is_subvolume_root(root.join("absent").as_path()).is_err());
    remove(root.as_path());
}

/// The platform returns the kernel's error for a path that is not there, on any filesystem: asking whether it is a subvolume
///  root is an error, and creating a subvolume below a parent that does not exist is an error that creates nothing. A platform
///  that answered `Ok` to either would fail here without a btrfs fixture.
#[test]
fn the_platform_reports_the_error_for_a_path_that_is_not_there() {
    let root = crate::test_support::fixture("platform-absent");
    let platform: LinuxPlatform = LinuxPlatform;
    assert!(platform.is_subvolume_root(root.join("absent").as_path()).is_err());
    let created = platform.create_subvolume(root.join("absent/below").as_path());
    assert!(created.is_err(), "{created:?}");
    assert!(!root.join("absent").exists(), "nothing was created");
    remove(root.as_path());
}

/// A directory made the ordinary way is not a subvolume root, on any filesystem; a platform that answered `true` would fail
///  here, and the creation of a subvolume where the filesystem has none is refused by the kernel and leaves nothing behind.
#[test]
fn an_ordinary_directory_is_no_subvolume_root_whatever_the_filesystem() {
    let root = crate::test_support::fixture("platform-ordinary");
    let platform: LinuxPlatform = LinuxPlatform;
    std::fs::create_dir(root.join("plain")).expect("directory");
    assert_eq!(platform.is_subvolume_root(root.join("plain").as_path()), Ok(false));
    std::fs::write(root.join("file"), b"x").expect("file");
    assert_eq!(platform.is_subvolume_root(root.join("file").as_path()), Ok(false), "a file is not a subvolume root either");
    let existing = platform.create_subvolume(root.join("plain").as_path());
    assert!(existing.is_err(), "a name that is taken is refused: {existing:?}");
    assert!(root.join("plain").is_dir(), "the directory is untouched");
    remove(root.as_path());
}
