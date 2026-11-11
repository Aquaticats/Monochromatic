//! Removal beneath a held parent handle:
//! what is removed,
//! what is refused,
//! and that no link is ever followed.

use super::*;
use crate::fs_beneath::{identity_at, open_root};
use crate::fs_btrfs_ioctl::{snapshot_create_at, subvolume_create_at};
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};

/// Write a file,
/// making its directories.
fn put(root: &Path, path: &str, bytes: &[u8]) {
    let full: PathBuf = root.join(path);
    std::fs::create_dir_all(full.parent().expect("parent")).expect("directories");
    std::fs::write(full, bytes).expect("file");
}

/// A tree with files,
/// a read-only directory holding a file,
/// and a link to a directory outside it is removed whole;
/// the
///  link is unlinked and what it points at is untouched.
#[test]
fn a_tree_with_a_read_only_directory_and_a_link_is_removed_without_touching_the_link_target() {
    let root: PathBuf = fixture("trash-remove-tree");
    put(root.as_path(), "top/a/file", b"a");
    put(root.as_path(), "top/a/b/deep", b"b");
    put(root.as_path(), "top/locked/file", b"locked");
    put(root.as_path(), "precious/data", b"keep");
    std::os::unix::fs::symlink(root.join("precious"), root.join("top/a/link")).expect("link");
    std::fs::set_permissions(root.join("top/locked"), std::os::unix::fs::PermissionsExt::from_mode(0o555)).expect("read-only directory");
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let removal: Removal = remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("top"), None).expect("removed");
    assert_eq!(removal, Removal::Removed);
    assert!(!root.join("top").exists(), "the whole tree is gone");
    assert!(root.join("precious/data").exists(), "what the link pointed at is untouched");
    remove(root.as_path());
}

/// A name with nothing at it is absent,
/// not an error.
#[test]
fn a_name_with_nothing_at_it_is_absent() {
    let root: PathBuf = fixture("trash-remove-absent");
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    assert_eq!(remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("nothing"), None), Ok(Removal::Absent));
    remove(root.as_path());
}

/// A link at the name is refused and its target is untouched,
/// whether or not an identity is expected.
#[test]
fn a_link_at_the_name_is_refused_and_not_followed() {
    let root: PathBuf = fixture("trash-remove-link");
    put(root.as_path(), "precious/data", b"keep");
    std::os::unix::fs::symlink(root.join("precious"), root.join("top")).expect("link");
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let reason: String = remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("top"), None).expect_err("a link is no directory to empty");
    assert!(reason.contains("top"), "{reason}");
    assert!(root.join("precious/data").exists());
    remove(root.as_path());
}

/// A directory that is not the expected object is left alone with a reason naming both identities.
#[test]
fn a_directory_that_is_not_the_expected_object_is_left_alone() {
    let root: PathBuf = fixture("trash-remove-identity");
    put(root.as_path(), "first/file", b"first");
    put(root.as_path(), "second/file", b"second");
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let first = identity_at(&held, std::ffi::OsStr::new("first")).expect("identity");
    let reason: String = remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("second"), Some(first)).expect_err("another object");
    assert!(reason.contains("second") && reason.contains(first.encode().as_str()), "{reason}");
    assert!(root.join("second/file").exists(), "the other directory is untouched");
    assert_eq!(remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("first"), Some(first)), Ok(Removal::Removed));
    remove(root.as_path());
}

/// Depth costs no recursion:
/// a chain of directories far deeper than a thread's stack could recurse through is removed.
#[test]
fn a_very_deep_chain_of_directories_is_removed() {
    let root: PathBuf = fixture("trash-remove-deep");
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let mut current: std::fs::File = held.try_clone().expect("clone");
    crate::fs_beneath::make_directory_at(&current, std::ffi::OsStr::new("top"), 0o755).expect("top");
    current = crate::fs_beneath::open_directory_at(&current, std::ffi::OsStr::new("top")).expect("open top");
    for _ in 0..300 {
        crate::fs_beneath::make_directory_at(&current, std::ffi::OsStr::new("d"), 0o755).expect("level");
        current = crate::fs_beneath::open_directory_at(&current, std::ffi::OsStr::new("d")).expect("open level");
    }
    drop(current);
    assert_eq!(remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("top"), None), Ok(Removal::Removed));
    assert!(!root.join("top").exists());
    remove(root.as_path());
}

/// What:
///  A tree that holds a nested subvolume is refused whole:
///  every ordinary directory in it is still there when the reason comes back.
/// Why:
///   The nested subvolume is no part of the tree being removed,
/// so the removal cannot go on;
/// the person decides what happens to the tree,
/// and a removal that unlinked the ordinary directories
///  first destroyed what `never discard user data` keeps until that decision.
/// `SPEC.md` states the refusal comes before anything is deleted.
#[test]
fn btrfs_a_nested_subvolume_is_refused_before_anything_else_in_the_tree_is_deleted() {
    let Some(root) = btrfs_fixture("trash-remove-nested-subvolume") else { return };
    let top: PathBuf = root.join("top");
    std::fs::create_dir(top.as_path()).expect("the top directory");
    subvolume_create_at(top.join("boundary").as_path()).expect("a nested subvolume");
    put(root.as_path(), "top/boundary/inside", b"data the subvolume holds");
    put(root.as_path(), "top/ordinary-a/sentinel", b"a file the person has not committed");
    put(root.as_path(), "top/ordinary-b/deep/sentinel", b"another file the person has not committed");
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let reason: String = remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("top"), None).expect_err("a nested subvolume is no boundary a removal crosses");
    assert!(reason.contains("boundary"), "the reason names the path it stopped at: {reason}");
    assert!(top.join("ordinary-a/sentinel").exists(), "nothing was deleted before the refusal came: {reason}");
    assert!(top.join("ordinary-b/deep/sentinel").exists(), "nothing was deleted before the refusal came: {reason}");
    assert!(top.join("boundary/inside").exists(), "the subvolume and what it holds are untouched");
    drop(held);
    // The walk that removes a fixture goes through the subvolume like any directory, and removes it once it is empty.
    remove(root.as_path());
}

/// What:
///  A folder holding the stand-ins a snapshot shows where its source holds nested subvolumes is removed whole.
/// Why:
///   The stand-in step takes a whole worktree folder off an add's waiting path by moving it into the trash,
/// so the
///  service's emptying meets the stand-ins of every worktree the repository had;
/// they hold nothing and belong to the tree
///  being removed,
/// although btrfs reports a device of their own for each.
#[test]
fn btrfs_a_folder_holding_the_stand_ins_of_nested_subvolumes_is_removed_whole() {
    let Some(root) = btrfs_fixture("trash-remove-stand-ins") else { return };
    let source: PathBuf = root.join("source");
    subvolume_create_at(source.as_path()).expect("the source subvolume");
    std::fs::create_dir_all(source.join("folder")).expect("the folder");
    for name in ["w0", "w1"] {
        subvolume_create_at(source.join("folder").join(name).as_path()).expect("a nested subvolume");
    }
    let snapshot: PathBuf = root.join("snapshot");
    snapshot_create_at(source.as_path(), snapshot.as_path()).expect("a snapshot");
    let held: std::fs::File = open_root(snapshot.as_path()).expect("the snapshot");
    let stand_ins: Vec<String> = std::fs::read_dir(snapshot.join("folder").as_path()).expect("listing").flatten().map(|entry: std::fs::DirEntry| { return entry.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(stand_ins.len(), 2, "the snapshot shows a stand-in for each nested subvolume: {stand_ins:?}");
    assert_eq!(remove_tree_beneath(snapshot.as_path(), &held, std::ffi::OsStr::new("folder"), None), Ok(Removal::Removed));
    assert!(!snapshot.join("folder").exists(), "the folder and the stand-ins in it are gone");
    remove(root.as_path());
}
