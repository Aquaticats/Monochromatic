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
