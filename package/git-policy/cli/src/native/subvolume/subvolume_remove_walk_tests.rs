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

/// What:
///  A nested subvolume that appears in a directory after the removal pass listed it stops the emptying there,
///  by name,
/// with the ways forward,
///  and what it holds is untouched.
/// Why:
///   The emptying lists a directory once and unlinks what it listed,
/// so a boundary made after that listing is met only
///  when the kernel refuses the directory as not empty;
/// `SPEC.md` promises that a boundary appearing after the first
///  pass is refused at the directory where it is met,
/// which means naming it there,
/// never the kernel's words alone.
#[test]
fn btrfs_a_subvolume_made_after_a_directory_was_listed_stops_the_emptying_there_by_name() {
    let Some(root) = btrfs_fixture("walk-subvolume-after-listing") else { return };
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.join("a")).expect("directory");
    std::fs::write(tree.join("a/file"), b"x").expect("file");
    let boundary: PathBuf = tree.join("a/boundary-made-after-the-listing");
    let made: PathBuf = boundary.clone();
    crate::subvolume_remove_listing_test_support::act_after_each_listing(move |listed: &Path| {
        if listed.file_name() != Some(std::ffi::OsStr::new("a")) {
            return;
        }
        subvolume_create_at(made.as_path()).expect("a nested subvolume made while the emptying runs");
        std::fs::write(made.join("inside"), b"independent").expect("a file inside it");
    });
    let held: File = open_root(tree.as_path()).expect("open the top");
    let error: std::io::Error = empty_directory(&held, tree.as_path()).expect_err("a nested subvolume is a boundary");
    crate::subvolume_remove_listing_test_support::stop_acting();
    assert!(error.to_string().contains(boundary.to_str().expect("UTF-8")), "the reason names the boundary the emptying met: {error}");
    assert!(error.to_string().contains("btrfs subvolume delete"), "the ways forward are named: {error}");
    assert_eq!(std::fs::read(boundary.join("inside")).expect("kept"), b"independent");
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

/// What:
///  The look that decides a removal goes around a volume finds every one of them and deletes nothing.
/// Why:
///  The removal decides on this look,
///  and a look that widened a directory or removed an entry would change what it judges before the
///  decision is taken.
#[test]
fn btrfs_the_look_reports_every_volume_and_changes_nothing() {
    let Some(root) = btrfs_fixture("walk-look") else { return };
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.join("a/b")).expect("directories");
    std::fs::write(tree.join("a/b/file"), b"x").expect("file");
    subvolume_create_at(tree.join("a/b/one").as_path()).expect("one volume");
    subvolume_create_at(tree.join("two").as_path()).expect("another volume");
    std::fs::write(tree.join("a/b/one/keep"), b"independent").expect("a file in a volume");
    std::fs::set_permissions(tree.join("a/b").as_path(), std::fs::Permissions::from_mode(0o500)).expect("a read-only directory");
    let held: File = open_root(tree.as_path()).expect("open the top");
    let look: BoundaryLook = look_for_boundaries(&held, tree.as_path());
    assert!(look.unreadable.is_none(), "the look covered the whole tree: {look:?}");
    let mut found: Vec<String> = look.found.iter().map(|one| return one.path.display().to_string()).collect();
    found.sort();
    let mut wanted: Vec<String> = vec![tree.join("a/b/one").display().to_string(), tree.join("two").display().to_string()];
    wanted.sort();
    assert_eq!(found, wanted, "both volumes are reported");
    assert_eq!(std::fs::metadata(tree.join("a/b").as_path()).expect("the directory").mode() & 0o7777, 0o500, "the look widened nothing");
    assert_eq!(std::fs::read(tree.join("a/b/file")).expect("the file"), b"x", "and removed nothing");
    assert_eq!(std::fs::read(tree.join("a/b/one/keep")).expect("the file in the volume"), b"independent");
    remove(root.as_path());
}

/// What:
///  The pass that removes around a volume leaves the volume and the directories that lead to it,
///  gives those directories back the mode they had,
///  and removes every other entry of the tree.
/// Why:
///  This is what a removal does with what it did not make,
///  and the modes are the state nobody asked it to change.
#[test]
fn btrfs_the_tree_is_emptied_around_a_volume_and_the_folders_that_lead_to_it_keep_their_modes() {
    let Some(root) = btrfs_fixture("walk-around") else { return };
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.join("a/b")).expect("the directories the volume stands in");
    std::fs::write(tree.join("a/b/beside"), b"removed").expect("a file beside the volume");
    std::fs::write(tree.join("top-file"), b"removed").expect("a file at the top");
    std::fs::create_dir_all(tree.join("gone/inner")).expect("a directory that goes");
    std::fs::write(tree.join("gone/inner/file"), b"removed").expect("a file in it");
    subvolume_create_at(tree.join("a/b/deep").as_path()).expect("a volume");
    std::fs::write(tree.join("a/b/deep/keep"), b"independent").expect("a file in the volume");
    for (path, mode) in [("a/b", 0o500), ("a", 0o750)] {
        std::fs::set_permissions(tree.join(path), std::fs::Permissions::from_mode(mode)).expect("mode");
    }
    let held: File = open_root(tree.as_path()).expect("open the top");
    empty_directory_around(&held, tree.as_path()).expect("the tree is emptied around the volume");
    assert_eq!(std::fs::read(tree.join("a/b/deep/keep")).expect("the file in the volume"), b"independent", "the volume is untouched");
    assert_eq!(std::fs::read_dir(tree.as_path()).expect("the top").flatten().map(|found| return found.file_name()).collect::<Vec<_>>(), vec![std::ffi::OsString::from("a")], "nothing but the volume's chain is left at the top");
    assert!(!tree.join("top-file").exists() && !tree.join("gone").exists(), "every other entry is removed");
    assert_eq!(std::fs::metadata(tree.join("a/b").as_path()).expect("the directory").mode() & 0o7777, 0o500, "the read-only directory keeps its mode");
    assert_eq!(std::fs::metadata(tree.join("a").as_path()).expect("the directory").mode() & 0o7777, 0o750, "and so does the one above it");
    let look: BoundaryLook = look_for_boundaries(&held, tree.as_path());
    assert_eq!(look.found.len(), 1, "the volume is still there and named: {look:?}");
    remove(root.as_path());
}

/// What:
///  A volume whose own mode forbids this user access is never widened to look inside it.
/// Why:
///  The pass must not change what it does not own,
///  and a directory of mode 000 is where a widening attempt would show.
#[test]
fn btrfs_a_volume_is_left_with_the_mode_it_had() {
    let Some(root) = btrfs_fixture("walk-around-mode") else { return };
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.as_path()).expect("the tree");
    subvolume_create_at(tree.join("volume").as_path()).expect("a volume");
    std::fs::set_permissions(tree.join("volume").as_path(), std::fs::Permissions::from_mode(0o000)).expect("a mode that forbids access");
    let held: File = open_root(tree.as_path()).expect("open the top");
    empty_directory_around(&held, tree.as_path()).expect("the tree is emptied around the volume");
    assert_eq!(std::fs::symlink_metadata(tree.join("volume")).expect("the volume").mode() & 0o7777, 0o000, "the volume's mode is what it was");
    assert!(tree.join("volume").exists(), "and it stands where it stood");
    std::fs::set_permissions(tree.join("volume").as_path(), std::fs::Permissions::from_mode(0o700)).expect("a mode the removal of the fixture needs");
    remove(root.as_path());
}

/// What:
///  A volume that appears in a directory after the pass listed that directory is left,
///  and the pass goes on with what it listed beside it.
/// Why:
///  The kernel then refuses the directory as not empty,
///  and a removal that refused there would leave the worktree half emptied with
///  a volume in it;
/// the decision is that such a volume is left and the removal finishes.
#[test]
fn btrfs_a_volume_made_after_a_directory_was_listed_is_left_and_the_pass_goes_on() {
    let Some(root) = btrfs_fixture("walk-around-late") else { return };
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.join("a")).expect("the directory the volume appears in");
    std::fs::write(tree.join("a/file"), b"removed").expect("a file beside it");
    let planting: PathBuf = tree.join("a/late");
    let acted_on: PathBuf = tree.join("a");
    crate::subvolume_remove_listing_test_support::act_after_each_listing(move |listed: &Path| {
        if listed == acted_on.as_path() {
            subvolume_create_at(planting.as_path()).expect("a volume that appeared after the listing");
        }
    });
    let held: File = open_root(tree.as_path()).expect("open the top");
    let passed: std::io::Result<()> = empty_directory_around(&held, tree.as_path());
    crate::subvolume_remove_listing_test_support::stop_acting();
    passed.expect("the pass leaves the volume and goes on");
    assert!(tree.join("a/late").is_dir(), "the volume that appeared is left");
    assert!(!tree.join("a/file").exists(), "and the entry the pass listed beside it is removed");
    remove(root.as_path());
}

/// A link to a directory is unlinked and its target kept,
/// by the pass that empties around a volume as by the one that empties everything.
#[test]
fn a_link_is_unlinked_and_its_target_kept_by_the_pass_that_goes_around() {
    let parent: PathBuf = fixture("walk-around-link");
    let tree: PathBuf = parent.join("tree");
    std::fs::create_dir_all(tree.as_path()).expect("the tree");
    std::fs::write(parent.join("outside"), b"keep").expect("a file outside the tree");
    std::os::unix::fs::symlink(parent.join("outside"), tree.join("link")).expect("a link");
    let held: File = open_root(tree.as_path()).expect("open the top");
    empty_directory_around(&held, tree.as_path()).expect("the tree is emptied");
    assert!(!tree.join("link").exists(), "the link is gone");
    assert_eq!(std::fs::read(parent.join("outside")).expect("kept"), b"keep", "its target is not touched");
    remove(parent.as_path());
}

/// What:
///  A volume put at the name of a directory the pass has just asked about is kept,
///  and nothing inside it is removed.
/// Why:
///  The pass asks what an entry is and then opens it for reading,
/// and the two are not one step:
/// a subvolume renamed over a plain directory between them was opened,
/// emptied and removed,
///  which is exactly what a removal that "goes around" a volume promises never to do.
///  The control states the exchange at that moment, so it is the same window every time rather than a race.
#[test]
fn btrfs_a_volume_put_at_the_name_after_the_check_is_kept() {
    let Some(root) = btrfs_fixture("walk-around-swap") else { return };
    let tree: PathBuf = root.join("tree");
    std::fs::create_dir_all(tree.join("d")).expect("the directory the swap takes");
    std::fs::write(tree.join("d/beside.txt"), b"removed").expect("a file of the worktree");
    subvolume_create_at(tree.join("racer").as_path()).expect("a volume the pass did not make");
    std::fs::write(tree.join("racer/precious.txt"), b"RACER PRECIOUS\n").expect("a file the person put there");
    let held: File = open_root(tree.as_path()).expect("open the top");
    let swapped: File = held.try_clone().expect("a handle of the top");
    let at: PathBuf = tree.join("d");
    let once: std::cell::Cell<bool> = std::cell::Cell::new(false);
    crate::subvolume_remove_listing_test_support::act_after_each_check(move |checked: &Path| {
        if checked != at.as_path() || once.replace(true) {
            return;
        }
        // The volume takes the name of the plain directory the pass has just judged, and the directory takes the volume's
        //  name: the pass's next open of that name reaches the volume.
        crate::fs_beneath::exchange_at(&swapped, std::ffi::OsStr::new("d"), std::ffi::OsStr::new("racer")).expect("exchange the two");
    });
    let passed: std::io::Result<()> = empty_directory_around(&held, tree.as_path());
    crate::subvolume_remove_listing_test_support::stop_acting();
    passed.expect("the pass finishes");
    let survived: bool = tree.join("d/precious.txt").is_file() || tree.join("racer/precious.txt").is_file();
    assert!(survived, "the volume and the file the person put there are where they stand: {:?}", std::fs::read_dir(tree.as_path()).expect("the tree").flatten().map(|found| return found.file_name()).collect::<Vec<_>>());
    let bytes: Vec<u8> = std::fs::read(tree.join("d/precious.txt")).or_else(|_| return std::fs::read(tree.join("racer/precious.txt"))).expect("the file the person put there");
    assert_eq!(bytes, b"RACER PRECIOUS\n", "and hold the bytes they held");
    remove(root.as_path());
}
