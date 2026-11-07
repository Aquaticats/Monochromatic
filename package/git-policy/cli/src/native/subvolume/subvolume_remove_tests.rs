//! Controls for the removal of trees:
//! files,
//! links,
//! ordinary trees and trees with directories the owner cannot write.

use super::*;
use std::path::PathBuf;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;

/// A file,
/// a link and a missing path are handled;
/// a link is unlinked,
/// never followed.
#[test]
fn a_file_and_a_link_are_unlinked_and_a_missing_path_is_an_error() {
    let parent: PathBuf = fixture("remove-tree-simple");
    std::fs::write(parent.join("file"), b"x").expect("file");
    std::os::unix::fs::symlink(parent.join("file"), parent.join("link")).expect("link");
    remove_tree(parent.join("link").as_path()).expect("link removed");
    assert!(parent.join("file").exists() && !parent.join("link").exists());
    remove_tree(parent.join("file").as_path()).expect("file removed");
    assert_eq!(remove_tree(parent.join("file").as_path()).expect_err("gone").kind(), std::io::ErrorKind::NotFound);
    remove(parent.as_path());
}

/// Directories without owner write or search permission,
/// at several depths,
/// do not stop the removal;
/// a link inside is
///  unlinked and its target is untouched.
#[test]
fn directories_the_owner_cannot_write_are_removed_with_their_content() {
    let parent: PathBuf = fixture("remove-tree-read-only");
    let tree: PathBuf = parent.join("tree");
    std::fs::create_dir_all(tree.join("a/b/c")).expect("directories");
    std::fs::write(tree.join("a/b/c/file"), b"x").expect("file");
    std::fs::write(tree.join("a/file"), b"x").expect("file");
    std::fs::write(parent.join("outside"), b"keep").expect("outside");
    std::os::unix::fs::symlink(parent.join("outside"), tree.join("a/link")).expect("link");
    for (path, mode) in [("a/b/c", 0o555), ("a/b", 0o000), ("a", 0o500)] {
        std::fs::set_permissions(tree.join(path), std::fs::Permissions::from_mode(mode)).expect("mode");
    }
    remove_tree(tree.as_path()).expect("removed");
    assert!(!tree.exists());
    assert_eq!(std::fs::read(parent.join("outside")).expect("kept"), b"keep");
    remove(parent.as_path());
}

/// A nested subvolume is another tree's data,
/// not part of the tree being removed:
/// the removal refuses before it deletes
///  anything,
/// and names the nested subvolume.
#[test]
fn btrfs_a_nested_subvolume_stops_the_removal_before_anything_is_deleted() {
    let Some(root) = btrfs_fixture("remove-tree-nested") else { return };
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.join("a/b")).expect("directories");
    std::fs::write(tree.join("a/first.txt"), b"first").expect("file");
    std::fs::write(tree.join("a/b/second.txt"), b"second").expect("file");
    subvolume_create_at(tree.join("z-inner").as_path()).expect("nested subvolume");
    std::fs::write(tree.join("z-inner/keep.txt"), b"independent data").expect("nested file");
    let refused: std::io::Error = remove_tree(tree.as_path()).expect_err("a nested subvolume is a boundary");
    assert!(refused.to_string().contains("z-inner"), "{refused}");
    assert_eq!(std::fs::read(tree.join("z-inner/keep.txt")).expect("kept"), b"independent data");
    assert_eq!(std::fs::read(tree.join("a/first.txt")).expect("nothing was deleted before the refusal"), b"first");
    assert_eq!(std::fs::read(tree.join("a/b/second.txt")).expect("nothing was deleted before the refusal"), b"second");
    // Once the nested subvolume is gone the same tree is removed whole.
    crate::fs_btrfs_ioctl::subvolume_destroy_at(tree.join("z-inner").as_path()).or_else(|_refused| { return std::fs::remove_dir_all(tree.join("z-inner")); }).ok();
    remove_tree(tree.as_path()).expect("removed");
    assert!(!tree.exists());
    remove(root.as_path());
}
