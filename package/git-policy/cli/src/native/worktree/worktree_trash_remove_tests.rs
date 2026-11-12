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

/// What:
///  A subdirectory that another command removes after its parent was listed counts as removed,
///  and the tree is removed whole.
/// Why:
///   A directory is listed once and each listed entry is popped once,
/// and an entry of the listing may be gone when the loop reaches it:
/// a concurrent `git clean -X` under an ignored trash does exactly that,
/// as does the person emptying the trash by hand.
/// The file branch and the final `rmdir` already answer a vanished entry as removed,
/// and the contract of this removal says so for every entry,
/// while the directory branch turned it into a failure the service settles for good.
#[test]
fn a_subdirectory_that_vanishes_after_its_parent_was_listed_counts_as_removed() {
    let root: PathBuf = fixture("trash-remove-vanished");
    put(root.as_path(), "top/outer/kept/file", b"kept");
    put(root.as_path(), "top/outer/vanishing/deep/file", b"deep");
    let vanishing: PathBuf = root.join("top/outer/vanishing");
    crate::subvolume_remove_listing_test_support::act_after_each_listing(move |listed: &Path| {
        if listed.file_name() != Some(std::ffi::OsStr::new("outer")) {
            return;
        }
        std::fs::remove_dir_all(vanishing.as_path()).expect("another command removes a directory of the listing");
    });
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let removal: Result<Removal, String> = remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("top"), None);
    crate::subvolume_remove_listing_test_support::stop_acting();
    assert_eq!(removal, Ok(Removal::Removed), "a directory that is already gone is removed");
    assert!(!root.join("top").exists(), "and the rest of the tree goes with it");
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
///  A nested subvolume that appears in a directory after the removal listed that directory is refused there,
///  by name,
/// with the ways forward,
///  and it and what it holds are untouched.
/// Why:
///   A removal lists a directory once and unlinks what it listed,
/// so a subvolume made after that listing is not
///  among them;
/// the kernel then refuses the directory as not empty,
/// and the removal has to say what it met and what
///  to do about it,
/// which `SPEC.md` promises (a boundary that appears after the first pass is refused again at the
///  directory where it is met).
/// The entries that listing held are already removed,
/// which is the residual race
///  `SPEC.md` accepts by name;
/// this control pins the refusal and its words,
/// never that the listing's entries
///  survived.
#[test]
fn btrfs_a_nested_subvolume_made_after_a_directory_was_listed_is_refused_there_by_name() {
    let Some(root) = btrfs_fixture("trash-remove-subvolume-after-listing") else { return };
    let top: PathBuf = root.join("top");
    put(root.as_path(), "top/locked/file", b"a file the listing held");
    let boundary: PathBuf = top.join("locked/boundary-made-after-the-listing");
    let made: PathBuf = boundary.clone();
    crate::subvolume_remove_listing_test_support::act_after_each_listing(move |listed: &Path| {
        if listed.file_name() != Some(std::ffi::OsStr::new("locked")) {
            return;
        }
        subvolume_create_at(made.as_path()).expect("a nested subvolume made while the removal runs");
        std::fs::write(made.join("inside"), b"data the subvolume holds").expect("a file inside it");
    });
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let reason: String = remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("top"), None).expect_err("a nested subvolume is no boundary a removal crosses");
    crate::subvolume_remove_listing_test_support::stop_acting();
    assert!(reason.contains(boundary.to_str().expect("UTF-8")), "the reason names the boundary the removal met: {reason}");
    assert!(reason.contains("btrfs subvolume delete"), "the ways forward are named: {reason}");
    assert!(boundary.join("inside").is_file(), "the subvolume and what it holds are untouched");
    assert!(top.is_dir(), "the tree stays where it is");
    drop(held);
    remove(root.as_path());
}

/// What:
///  Entries that appear in a directory after the removal listed it are named,
///  and they are left as they are.
/// Why:
///   The removal deletes what it listed and nothing else,
/// so an entry that appeared after the listing is no
///  part of what it was asked to delete;
/// the kernel's `Directory not empty` alone tells the person nothing about
///  what stands there or what to do.
#[test]
fn entries_that_appeared_after_a_directory_was_listed_are_named_and_left_as_they_are() {
    let root: PathBuf = fixture("trash-remove-entry-after-listing");
    put(root.as_path(), "top/locked/file", b"a file the listing held");
    let appeared: PathBuf = root.join("top/locked/appeared-after-the-listing");
    let written: PathBuf = appeared.clone();
    crate::subvolume_remove_listing_test_support::act_after_each_listing(move |listed: &Path| {
        if listed.file_name() != Some(std::ffi::OsStr::new("locked")) {
            return;
        }
        std::fs::write(written.as_path(), b"written while the removal ran").expect("an entry that appears after the listing");
    });
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let reason: String = remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("top"), None).expect_err("a directory that gained an entry is not the one the removal listed");
    crate::subvolume_remove_listing_test_support::stop_acting();
    assert!(reason.contains("appeared-after-the-listing"), "the reason names what appeared: {reason}");
    assert!(appeared.is_file(), "what appeared is left as it is: {reason}");
    drop(held);
    remove(root.as_path());
}

/// What:
///  A directory whose name no longer leads to it when the removal asks for it is named as left as it is,
///  and the directory the removal held is left where the removal put it.
/// Why:
///   The removal holds the directory it emptied and removes the name in the held parent,
/// so a name that another
///  directory took removes nothing of the held one;
/// the person has to be told what the removal met there,
///  not left with a bare `Directory not empty`.
#[test]
fn a_directory_whose_name_was_taken_while_it_was_removed_is_named_as_left_as_it_is() {
    let root: PathBuf = fixture("trash-remove-name-taken");
    put(root.as_path(), "top/locked/file", b"a file the listing held");
    let moved: PathBuf = root.join("top/moved-aside");
    let taken: PathBuf = root.join("top/locked");
    let (from, to, another): (PathBuf, PathBuf, PathBuf) = (taken.clone(), moved.clone(), taken.clone());
    crate::subvolume_remove_listing_test_support::act_after_each_listing(move |listed: &Path| {
        if listed.file_name() != Some(std::ffi::OsStr::new("locked")) {
            return;
        }
        std::fs::rename(from.as_path(), to.as_path()).expect("the directory the removal holds leaves its name");
        std::fs::create_dir(another.as_path()).expect("another directory takes the name");
        std::fs::write(another.join("someone-elses"), b"not the tree the removal was asked to remove").expect("an entry of that other directory");
    });
    let held: std::fs::File = open_root(root.as_path()).expect("root");
    let reason: String = remove_tree_beneath(root.as_path(), &held, std::ffi::OsStr::new("top"), None).expect_err("a name that leads to another object is not the directory the removal emptied");
    crate::subvolume_remove_listing_test_support::stop_acting();
    assert!(reason.contains(taken.to_str().expect("UTF-8")), "the reason names the name that was taken: {reason}");
    assert!(reason.contains("led to a directory that was not empty"), "the reason says what the removal met at the name: {reason}");
    assert!(reason.contains("left as it is"), "the reason says what happens to it: {reason}");
    assert!(taken.join("someone-elses").is_file(), "the other directory and its entry are left as they are");
    drop(held);
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
