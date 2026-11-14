//! What:
//!  Controls for the look that finds every volume below a tree,
//!  and for the pass that empties a tree around each one it did not make.
//! Why:
//!  The look decides and the pass deletes,
//!  and both must leave what they did not make exactly as it stands:
//!  a volume's bytes,
//! its mode,
//! the mode of every directory that leads to it,
//! and a volume that takes a directory's name while the pass runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(treeOf(volume)).toEqual(before); expect(look.found).toEqual([volume]);
//! ```

use super::*;
use crate::fs_beneath::open_root;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::{MetadataExt, PermissionsExt};

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
