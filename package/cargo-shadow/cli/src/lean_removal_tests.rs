//! The tests of the removal primitive:
//! the identity is checked before the move and again before the unlink,
//! a swapped entry is never deleted,
//! and a directory this user may not write is made writable first when this user owns it,
//! or left alone and named when it does not.

use std::{fs, os::unix::fs::{MetadataExt, PermissionsExt}, path::{Path, PathBuf}};

use rustix::process::geteuid;

use crate::lean_identity::Deadline;
use crate::lean_removal::{Deleted, Item, Refusal, claim_into_trash, delete_claimed, delete_claimed_as, item_of, modes_named, widened_mode};

/// The modes a claimed tree of this issue holds:
/// searchable but not writable,
/// readable but not writable,
/// and closed to its owner altogether.
const CLOSED_MODES: [u32; 3] = [0o500, 0o555, 0o000];

/// How deep below the claimed entry the closed directory sits,
/// one case per depth.
const DEPTHS: [usize; 3] = [1, 2, 3];

use crate::test_scratch::scratch;

#[test]
fn an_item_is_claimed_and_deleted_and_its_bytes_are_reported() {
    let root = scratch();
    let path: PathBuf = root.path().join("unit");
    fs::create_dir_all(path.join("inner")).expect("directory");
    fs::write(path.join("inner").join("file"), vec![1_u8; 8192]).expect("file");
    let item: Item = item_of(&path).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
    assert!(!path.exists());
    let deleted: Deleted = delete_claimed(&claimed, item.identity, &Deadline::after(std::time::Duration::from_secs(60))).expect("delete");
    assert!(deleted.bytes >= 8192, "{}", deleted.bytes);
    assert!(deleted.modes_added.is_empty(), "a writable tree needs no mode change: {deleted:?}");
    assert!(!claimed.exists());
}

#[test]
fn a_path_swapped_for_another_entry_is_not_claimed() {
    let root = scratch();
    let path: PathBuf = root.path().join("unit");
    fs::create_dir_all(&path).expect("directory");
    let item: Item = item_of(&path).expect("identity");
    fs::remove_dir(&path).expect("remove the judged entry");
    fs::create_dir_all(&path).expect("put another entry in its place");
    fs::write(path.join("precious"), b"user data").expect("data");
    let outcome = claim_into_trash(&item, &root.path().join("trash"), 1);
    assert!(matches!(outcome, Err(Refusal::Changed { .. })), "{outcome:?}");
    assert!(path.join("precious").exists(), "the other entry stays where it is");
}

#[test]
fn a_claimed_entry_that_changed_before_the_unlink_is_not_deleted() {
    let root = scratch();
    let path: PathBuf = root.path().join("unit");
    fs::create_dir_all(&path).expect("directory");
    let item: Item = item_of(&path).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
    fs::remove_dir(&claimed).expect("another owner removes it");
    fs::create_dir_all(&claimed).expect("and a different entry takes the name");
    fs::write(claimed.join("precious"), b"user data").expect("data");
    let outcome = delete_claimed(&claimed, item.identity, &Deadline::after(std::time::Duration::from_secs(60)));
    assert!(matches!(outcome, Err(Refusal::Changed { .. })), "{outcome:?}");
    assert!(claimed.join("precious").exists());
}

#[test]
fn an_item_another_owner_already_removed_is_gone_not_an_error() {
    let root = scratch();
    let path: PathBuf = root.path().join("unit");
    fs::create_dir_all(&path).expect("directory");
    let item: Item = item_of(&path).expect("identity");
    fs::remove_dir(&path).expect("another owner");
    assert!(matches!(claim_into_trash(&item, &root.path().join("trash"), 1), Err(Refusal::Gone(_))));
}

#[test]
fn a_symbolic_link_is_removed_as_a_link_and_its_target_stays() {
    let root = scratch();
    let target: PathBuf = root.path().join("target-dir");
    fs::create_dir_all(&target).expect("target");
    fs::write(target.join("data"), b"keep").expect("data");
    let link: PathBuf = root.path().join("link");
    std::os::unix::fs::symlink(&target, &link).expect("link");
    let item: Item = item_of(&link).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
    delete_claimed(&claimed, item.identity, &Deadline::after(std::time::Duration::from_secs(60))).expect("delete");
    assert!(target.join("data").exists(), "the link's target is untouched");
}

/// What:
/// `fn closed_tree(root: &Path, mode: u32, depth: usize) -> (PathBuf, PathBuf)` builds a tree whose directory at the given
/// depth has the given mode,
/// and returns the tree and that directory's path below it.
/// Why:
/// The cases of unlink failing with `EACCES` are a directory this user may not write,
/// at the top of what the walk enters and below other directories,
/// and one that holds both a file and a directory,
/// so both the unlink and the descent are refused without the mode change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function closedTree(root: string, mode: number, depth: number): [string, string]
/// ```
fn closed_tree(root: &Path, mode: u32, depth: usize) -> (PathBuf, PathBuf) {
    let tree: PathBuf = root.join("tree");
    fs::create_dir_all(&tree).expect("tree");
    let mut below: PathBuf = PathBuf::new();
    let mut directory: PathBuf = tree.clone();
    for level in 0..depth {
        below = below.join(format!("level-{level}"));
        directory = tree.join(&below);
        fs::create_dir_all(&directory).expect("directory");
        fs::write(directory.join("file"), b"x").expect("file");
    }
    fs::create_dir_all(directory.join("under")).expect("under");
    fs::write(directory.join("under").join("deep-file"), b"x").expect("deep file");
    fs::set_permissions(&directory, fs::Permissions::from_mode(mode)).expect("mode");
    return (tree, below);
}

#[test]
fn a_claimed_tree_whose_directory_this_user_may_not_write_is_still_removed_whole() {
    for mode in CLOSED_MODES {
        for depth in DEPTHS {
            let root = scratch();
            let (tree, below) = closed_tree(root.path(), mode, depth);
            let item: Item = item_of(&tree).expect("identity");
            let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
            let deleted: Deleted = match delete_claimed(&claimed, item.identity, &Deadline::after(std::time::Duration::from_secs(60))) {
                Ok(deleted) => deleted,
                Err(refusal) => panic!("mode {mode:04o} at depth {depth}: {refusal}"),
            };
            assert!(!claimed.exists(), "mode {mode:04o} at depth {depth}: the whole tree is gone");
            assert_eq!(deleted.modes_added, vec![claimed.join(&below)], "mode {mode:04o} at depth {depth}: the one directory this removal changed the mode of is named");
            let named: String = modes_named(&deleted.modes_added);
            assert!(named.contains("mode changed on 1 directories"), "mode {mode:04o} at depth {depth}: {named}");
        }
    }
}

#[test]
fn a_symbolic_link_inside_a_claimed_tree_that_names_a_closed_directory_outside_it_leaves_it_untouched() {
    let root = scratch();
    let outside: PathBuf = root.path().join("outside");
    fs::create_dir_all(&outside).expect("outside directory");
    fs::write(outside.join("kept"), b"a person's file").expect("file");
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o500)).expect("mode");
    let tree: PathBuf = root.path().join("tree");
    fs::create_dir_all(tree.join("inner")).expect("tree");
    let link: PathBuf = tree.join("inner").join("to-outside");
    std::os::unix::fs::symlink(&outside, &link).expect("link");
    let item: Item = item_of(&tree).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
    let deleted: Deleted = delete_claimed(&claimed, item.identity, &Deadline::after(std::time::Duration::from_secs(60))).expect("delete");
    assert!(!claimed.exists(), "the claimed tree is gone");
    assert_eq!(fs::symlink_metadata(&outside).expect("the outside directory").mode() & 0o7777, 0o500, "the mode of a directory outside the claimed tree is as it was");
    assert!(outside.join("kept").exists(), "what lies outside the claimed tree is untouched");
    assert!(deleted.modes_added.is_empty(), "nothing outside the claimed tree was made writable: {deleted:?}");
}

#[test]
fn a_closed_directory_of_another_user_stays_and_the_removal_says_so() {
    // The removal runs as a user id the fixture's directories do not have,
    // which is what a directory of another user looks like to the walk.
    let root = scratch();
    let somebody_else: u32 = geteuid().as_raw() ^ 1;
    let tree: PathBuf = root.path().join("tree");
    fs::create_dir_all(tree.join("mine")).expect("tree");
    fs::write(tree.join("mine").join("file"), b"x").expect("file");
    let theirs: PathBuf = tree.join("theirs");
    fs::create_dir_all(&theirs).expect("their directory");
    fs::write(theirs.join("file"), b"x").expect("file");
    fs::set_permissions(&theirs, fs::Permissions::from_mode(0o500)).expect("mode");
    let item: Item = item_of(&tree).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &root.path().join("trash"), 1).expect("claim");
    let deleted: Deleted = match delete_claimed_as(&claimed, item.identity, somebody_else, &Deadline::after(std::time::Duration::from_secs(60))) {
        Err(Refusal::PartlyRemoved { deleted, .. }) => deleted,
        other => panic!("a directory of another user is a partial removal: {other:?}"),
    };
    assert_eq!(fs::symlink_metadata(claimed.join("theirs")).expect("their directory").mode() & 0o7777, 0o500, "their mode is not changed");
    assert!(claimed.join("theirs").join("file").exists(), "what lies under their directory stays");
    assert!(!claimed.join("mine").exists(), "what this user owns is still removed");
    assert!(deleted.modes_added.is_empty(), "nothing was made writable: {deleted:?}");
    assert!(
        deleted.remaining.iter().any(|reason| {
            return reason.contains("is owned by uid") && reason.contains("was not changed");
        }),
        "the reason names the owner and says the mode was not changed: {deleted:?}"
    );
}

#[test]
fn the_display_of_a_partial_removal_says_what_went_and_what_stayed() {
    let refusal: Refusal = Refusal::PartlyRemoved {
        path: PathBuf::from("/base/.cargo-shadow-trash/1-2-0-tree"),
        deleted: Deleted {
            bytes: 4096,
            modes_added: vec![PathBuf::from("/base/.cargo-shadow-trash/1-2-0-tree/closed")],
            remaining: vec![String::from("/base/.cargo-shadow-trash/1-2-0-tree/theirs is owned by uid 70 and the removal runs as uid 1000, so its mode 0500 was not changed")],
            cut: false,
        },
    };
    let text: String = refusal.to_string();
    assert!(text.contains("was only partly removed"), "{text}");
    assert!(text.contains("4096 disk bytes are gone"), "{text}");
    assert!(text.contains("mode changed on 1 directories"), "{text}");
    assert!(text.contains("is owned by uid 70"), "{text}");
    assert!(!text.contains("skipped"), "{text}");
}

#[test]
fn a_widened_directory_gets_this_users_read_write_and_search_and_keeps_every_other_bit() {
    for (found, widened) in [(0o500, 0o700), (0o555, 0o755), (0o000, 0o700), (0o540, 0o740), (0o2750, 0o2750 | 0o700), (0o1755, 0o1755)] {
        assert_eq!(widened_mode(found), widened, "mode {found:04o}");
    }
}

/// A deletion the deadline stops leaves the rest of the claimed entry in the trash,
/// and the sweep the next pass runs finishes it.
/// Why:
/// A tree can hold hundreds of thousands of files,
/// and the run that ended 2026-11-09T08:00:00-05:00 was killed by its unit thirty minutes past its own deadline
/// while a single tree's deletion was still going.
/// What a deadline leaves is not a failure:
/// the entry was judged and claimed already,
/// so the next pass only has to finish the unlink.
#[test]
fn a_deletion_the_deadline_stops_leaves_the_rest_for_the_next_pass_to_finish() {
    let root = scratch();
    let path: PathBuf = root.path().join("tree");
    for index in 0..8 {
        let directory: PathBuf = path.join(format!("unit-{index}")).join("inner");
        fs::create_dir_all(&directory).expect("directory");
        fs::write(directory.join("file"), vec![3_u8; 4096]).expect("file");
    }
    let trash: PathBuf = root.path().join("trash");
    let item: Item = item_of(&path).expect("identity");
    let claimed: PathBuf = claim_into_trash(&item, &trash, 1).expect("claim");
    let cut = delete_claimed(&claimed, item.identity, &Deadline::after(std::time::Duration::ZERO));
    match cut {
        Err(Refusal::PartlyRemoved { deleted, .. }) => {
            assert!(deleted.cut, "the walk stopped at the deadline: {deleted:?}");
            assert!(deleted.remaining.iter().any(|reason| { return reason.contains("the deadline passed while deleting it"); }), "{deleted:?}");
        }
        other => panic!("expected the deadline to stop the walk, got {other:?}"),
    }
    assert!(claimed.exists(), "what is left of it is still the claimed entry in the trash");
    let sweep: crate::lean_removal::TrashSweep = crate::lean_removal::empty_trash(&trash, &Deadline::after(std::time::Duration::from_secs(60)));
    assert!(!sweep.cut, "{sweep:?}");
    assert!(sweep.failures.is_empty(), "{sweep:?}");
    assert!(sweep.bytes >= 8 * 4096, "the next pass frees what the stopped one claimed: {sweep:?}");
    assert!(!claimed.exists(), "the next pass finished the deletion");
}

/// A sweep of the trash that reaches its bound deletes nothing it did not finish and says it was cut.
#[test]
fn a_trash_sweep_reports_the_bound_it_stopped_at_and_deletes_nothing_without_finishing() {
    let root = scratch();
    let trash: PathBuf = root.path().join("trash");
    fs::create_dir_all(trash.join("1-2-0-one")).expect("a claimed entry");
    fs::write(trash.join("1-2-0-one").join("file"), vec![4_u8; 4096]).expect("file");
    let cut: crate::lean_removal::TrashSweep = crate::lean_removal::empty_trash(&trash, &Deadline::after(std::time::Duration::ZERO));
    assert!(cut.cut, "{cut:?}");
    assert!(trash.join("1-2-0-one").exists(), "the claimed entry stays for the next pass");
    let finished: crate::lean_removal::TrashSweep = crate::lean_removal::empty_trash(&trash, &Deadline::after(std::time::Duration::from_secs(60)));
    assert!(!finished.cut, "{finished:?}");
    assert!(finished.bytes >= 4096, "{finished:?}");
    assert!(!trash.join("1-2-0-one").exists());
}
