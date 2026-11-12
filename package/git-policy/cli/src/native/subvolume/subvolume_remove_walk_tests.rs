//! Controls for the descriptor-relative emptying of a directory:
//! a plain tree,
//! directories the owner cannot use,
//! links that
//!  stay unfollowed,
//! and,
//! on btrfs,
//! a nested subvolume that stops the walk before anything is deleted.

use super::*;
use crate::fs_beneath::open_root;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;

/// The directory emptied,
/// the entries below it gone,
/// a link unlinked and its target kept,
/// and the directory itself left.
#[test]
fn a_tree_is_emptied_and_its_top_directory_stays() {
    let parent: PathBuf = fixture("walk-plain");
    let tree: PathBuf = parent.join("tree");
    std::fs::create_dir_all(tree.join("a/b/c")).expect("directories");
    std::fs::write(tree.join("a/b/c/file"), b"x").expect("file");
    std::fs::write(tree.join("top-file"), b"x").expect("file");
    std::fs::write(parent.join("outside"), b"keep").expect("outside");
    std::os::unix::fs::symlink(parent.join("outside"), tree.join("a/link")).expect("link");
    std::os::unix::fs::symlink(parent.join("tree"), tree.join("a/b/loop")).expect("loop");
    for (path, mode) in [("a/b/c", 0o555), ("a/b", 0o000), ("a", 0o500)] {
        std::fs::set_permissions(tree.join(path), std::fs::Permissions::from_mode(mode)).expect("mode");
    }
    let held: File = open_root(tree.as_path()).expect("open the top");
    verify_below(&held, tree.as_path()).expect("nothing in the way");
    assert!(tree.join("a/b/c/file").exists(), "the check deletes nothing");
    empty_directory(&held, tree.as_path()).expect("emptied");
    assert_eq!(std::fs::read_dir(tree.as_path()).expect("listing").count(), 0);
    assert_eq!(std::fs::read(parent.join("outside")).expect("kept"), b"keep");
    remove(parent.as_path());
}

/// A nested subvolume stops the check and the removal at its path,
/// with the data beside it and inside it untouched.
#[test]
fn btrfs_a_nested_subvolume_stops_the_walk_and_names_its_path() {
    let Some(root) = btrfs_fixture("walk-nested") else { return };
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.join("a")).expect("directory");
    std::fs::write(tree.join("a/file"), b"x").expect("file");
    subvolume_create_at(tree.join("a/inner").as_path()).expect("nested subvolume");
    std::fs::write(tree.join("a/inner/keep"), b"independent").expect("nested file");
    let held: File = open_root(tree.as_path()).expect("open the top");
    for refused in [verify_below(&held, tree.as_path()), empty_directory(&held, tree.as_path())] {
        let error: std::io::Error = refused.expect_err("a nested subvolume is a boundary");
        assert!(error.to_string().contains(tree.join("a/inner").to_str().expect("UTF-8")), "{error}");
        assert!(error.to_string().contains("btrfs subvolume delete"), "the ways forward are named: {error}");
    }
    assert_eq!(std::fs::read(tree.join("a/inner/keep")).expect("kept"), b"independent");
    assert_eq!(std::fs::read(tree.join("a/file")).expect("kept"), b"x");
    remove(root.as_path());
}

/// The empty directory a snapshot holds for a nested subvolume is part of the snapshot:
/// it is removed,
/// and the nested
///  subvolume of the original,
/// with its data,
/// stays.
#[test]
fn btrfs_a_snapshot_is_emptied_and_the_original_nested_subvolume_stays() {
    let Some(root) = btrfs_fixture("walk-snapshot") else { return };
    let original: PathBuf = root.join("original");
    subvolume_create_at(original.as_path()).expect("original");
    subvolume_create_at(original.join("inner").as_path()).expect("nested subvolume");
    std::fs::write(original.join("inner/keep"), b"independent").expect("nested file");
    std::fs::write(original.join("file"), b"x").expect("file");
    let copy: PathBuf = root.join("copy");
    crate::fs_btrfs_ioctl::snapshot_create_at(original.as_path(), copy.as_path()).expect("snapshot");
    assert!(copy.join("inner").is_dir(), "the snapshot holds a stand-in for the nested subvolume");
    let held: File = open_root(copy.as_path()).expect("open the top");
    verify_below(&held, copy.as_path()).expect("a stand-in is not a boundary");
    empty_directory(&held, copy.as_path()).expect("emptied");
    assert_eq!(std::fs::read_dir(copy.as_path()).expect("listing").count(), 0);
    assert_eq!(std::fs::read(original.join("inner/keep")).expect("the nested subvolume of the original is untouched"), b"independent");
    assert_eq!(std::fs::read(original.join("file")).expect("kept"), b"x");
    remove(root.as_path());
}

/// What:
///  The folder the caller names as its own is left out of the walk,
///  a volume inside it is no boundary,
///  and one
///  beside it still is.
/// Why:
///   A caller that keeps its own state in a folder below the top removes that folder itself,
/// and a volume a fill made
///  there in the moments since it was cleared is the caller's own:
/// a walk that refused it as somebody else's would
///  stop a removal over state this program made.
/// The boundary beside it is what proves the walk still looks at everything else.
#[test]
fn btrfs_the_folder_the_caller_names_is_left_out_and_a_volume_beside_it_is_not() {
    let Some(root) = btrfs_fixture("walk-except") else { return };
    let tree: PathBuf = root.join("tree");
    let own: PathBuf = tree.join("monochromatic-git-policy-cli");
    std::fs::create_dir_all(own.join("ahead")).expect("the caller's own folder");
    subvolume_create_at(own.join("ahead/member").as_path()).expect("a volume the caller made inside its own folder");
    std::fs::write(tree.join("file"), b"x").expect("file");
    let held: File = open_root(tree.as_path()).expect("open the top");
    let nothing: Option<Boundary> = find_boundary_below_except(&held, tree.as_path(), &[own.as_path()]).expect("walk");
    assert!(nothing.is_none(), "the caller's own folder is not a boundary: {nothing:?}");
    // The same walk without the exclusion refuses the volume inside that folder, which is what makes the exclusion a decision.
    let refused: Boundary = find_boundary_below(&held, tree.as_path()).expect("walk").expect("the volume inside the caller's folder is a boundary");
    assert!(refused.path.ends_with("ahead/member"), "{}", refused.path.display());
    subvolume_create_at(tree.join("beside").as_path()).expect("a volume the caller did not make");
    let found: Boundary = find_boundary_below_except(&held, tree.as_path(), &[own.as_path()]).expect("walk").expect("a volume beside the caller's folder is a boundary");
    assert!(found.path.ends_with("beside"), "{}", found.path.display());
    remove(root.as_path());
}
