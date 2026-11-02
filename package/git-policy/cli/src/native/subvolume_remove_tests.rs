//! Controls for the removal of trees: files, links, ordinary trees and trees with directories the owner cannot write.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;

/// A file, a link and a missing path are handled; a link is unlinked, never followed.
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

/// Directories without owner write or search permission, at several depths, do not stop the removal; a link inside is
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
