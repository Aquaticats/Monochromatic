//! Controls for the snapshot mechanics on btrfs:
//! the stage becomes a private snapshot with the new worktree's pointer,
//! no
//!  stubs and the source's index;
//! a source that changed after the clean check is refused;
//! and the swap leaves the
//!  stage name holding what Git made.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::repo_read_index::parse_index;
use crate::repo_read_layout::ObjectFormat;
use crate::test_support::{age, git, git_context, remove};
use crate::worktree_ladder_test_support::{main_identity, plan_to, subvolume_repository};
use crate::worktree_snapshot_source::choose_snapshot_source;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use crate::worktree_add_plan::AddPlan;

/// Take the snapshot with the administrative directory opened as the ladder holds it.
fn take(source: &SnapshotSource, plan: &AddPlan, stage: &Path, admin: &Path, pointer: &[u8]) -> Result<(), String> {
    let held: File = open_root(admin).expect("administrative directory");
    return take_snapshot(source, plan, stage, &held, pointer).map(|_made: Snapshotted| {}).map_err(|failure: SnapshotFailure| { return failure.reason; });
}

/// The identity of a directory by its canonical path.
fn id_of_directory(path: &Path) -> ObjectId {
    let parent: File = open_root(path.parent().expect("parent")).expect("parent handle");
    return identity_at(&parent, path.file_name().expect("name")).expect("identity");
}

/// Git's own `--no-checkout` worktree beside the repository:
/// the empty directory with a `.git` file,
/// and its
///  administrative directory.
fn no_checkout_worktree(repo: &Path, name: &str) -> (PathBuf, PathBuf, Vec<u8>) {
    git(repo, &["worktree", "add", "--quiet", "--no-checkout", "-b", name, format!("../{name}").as_str()]);
    let destination: PathBuf = std::fs::canonicalize(repo.join("..").join(name)).expect("destination");
    let pointer: Vec<u8> = std::fs::read(destination.join(".git")).expect("pointer");
    let admin: PathBuf = repo.join(".git/worktrees").join(name);
    return (destination, admin, pointer);
}

/// The snapshot is a private subvolume that holds the files,
/// the new pointer,
/// the index,
/// and no stub of the destination.
#[test]
fn btrfs_the_stage_becomes_a_worktree_with_the_new_pointer_and_the_sources_index() {
    let Some(root) = btrfs_fixture("snapshot-stage") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination.clone());
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    take(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect("snapshot");
    assert!(is_subvolume_root(stage.as_path()).expect("probe"));
    assert_eq!(std::fs::metadata(stage.as_path()).expect("metadata").mode() & 0o777, 0o700, "private until placed");
    assert_eq!(std::fs::read(stage.join(".git")).expect("pointer"), pointer);
    assert_eq!(std::fs::read(stage.join("ignored/pkg/index.js")).expect("ignored state"), b"module.exports = 1;\n");
    let index: Vec<u8> = std::fs::read(admin.join("index")).expect("index");
    let entries = parse_index(index.as_slice(), ObjectFormat::Sha1).expect("a plain index");
    assert_eq!(entries.len(), source.clean.tracked);
    git(stage.as_path(), &["--git-dir", admin.to_str().expect("utf8"), "--work-tree", stage.to_str().expect("utf8"), "status", "--porcelain=v1"]);
    remove(root.as_path());
}

/// A source that changed after the clean check is refused by the comparison of the snapshot with the copied index.
#[test]
fn btrfs_a_source_that_changed_after_the_check_is_refused() {
    let Some(root) = btrfs_fixture("snapshot-changed") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    // The same size, a different time: Git's fast path sees a change.
    std::fs::write(repo.join("shared.txt"), b"SECOND, LONGER\n").expect("edit after the check");
    age(repo.join("shared.txt").as_path());
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let reason: String = take(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect_err("refused");
    assert!(reason.contains("the source changed while it was copied"), "{reason}");
    assert!(!admin.join("index").exists(), "no index is installed for a refused snapshot");
    remove(root.as_path());
}

/// The swap puts the stage at the destination with the source's mode and leaves Git's directory at the stage name,
///  which `settle_displaced` removes.
#[test]
fn btrfs_the_swap_puts_the_stage_at_the_destination() {
    let Some(root) = btrfs_fixture("snapshot-swap") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination.clone());
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    take(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect("snapshot");
    let made: ObjectId = id_of_directory(destination.as_path());
    put_in_place(stage.as_path(), destination.as_path(), 0o750, made).expect("swap");
    assert!(is_subvolume_root(destination.as_path()).expect("probe"));
    assert_eq!(std::fs::metadata(destination.as_path()).expect("metadata").mode() & 0o7777, 0o750);
    assert_eq!(std::fs::read(destination.join("shared.txt")).expect("tracked"), b"second, longer\n");
    assert!(stage.join(".git").is_file() && !stage.join("shared.txt").exists(), "Git's directory is at the stage name");
    assert_eq!(settle_displaced(stage.as_path(), destination.as_path(), made, pointer.as_slice()), Vec::<String>::new());
    assert!(!stage.exists());
    assert_eq!(git(destination.as_path(), &["status", "--porcelain=v1"]).stdout, b"");
    remove(root.as_path());
}

/// A stage on any filesystem,
/// with the source root `/src` as the paths of the stubs name it.
fn stage_for_stubs(name: &str) -> (PathBuf, File) {
    let root: PathBuf = crate::test_support::fixture(name);
    std::fs::create_dir_all(root.join("stage")).expect("stage");
    let held: File = open_root(std::fs::canonicalize(root.join("stage")).expect("canonical").as_path()).expect("handle");
    return (root, held);
}

/// A path that reaches the snapshot through a link left where a directory was moved removes nothing in the tree the link
///  points at,
/// for a nested stub and for the destination's own directory,
/// and a real path through real directories is
///  removed as before.
#[test]
fn a_stub_reached_through_a_link_removes_nothing_the_link_points_at() {
    let (root, stage) = stage_for_stubs("snapshot-stub-link");
    let victim: PathBuf = root.join("elsewhere/moved/wt");
    std::fs::create_dir_all(victim.as_path()).expect("the original nested worktree");
    std::fs::write(victim.join(".git"), b"gitdir: /original\n").expect("its pointer");
    std::fs::create_dir_all(root.join("stage/ignored")).expect("ignored");
    std::os::unix::fs::symlink(root.join("elsewhere/moved"), root.join("stage/ignored/group")).expect("the link the snapshot carries");
    let source_root: &Path = Path::new("/src");
    for pointer in [false, true] {
        let error: String = remove_stub(&stage, source_root, Path::new("/src/ignored/group/wt"), pointer).expect_err("refused");
        assert!(error.contains("link"), "{error}");
        assert_eq!(std::fs::read(victim.join(".git")).expect("the original's pointer"), b"gitdir: /original\n", "pointer {pointer}");
    }
    // Positive control: the same shape through real directories.
    std::fs::create_dir_all(root.join("stage/ignored/real/wt")).expect("a real stub");
    std::fs::create_dir_all(root.join("stage/ignored/real/dest")).expect("a real destination");
    std::fs::write(root.join("stage/ignored/real/dest/.git"), b"gitdir: x\n").expect("Git's pointer");
    remove_stub(&stage, source_root, Path::new("/src/ignored/real/wt"), false).expect("empty stub removed");
    remove_stub(&stage, source_root, Path::new("/src/ignored/real/dest"), true).expect("destination removed");
    assert!(!root.join("stage/ignored/real/wt").exists() && !root.join("stage/ignored/real/dest").exists());
    remove(root.as_path());
}

/// A nested stub that holds anything is not an empty stub:
/// it stays and the rung ends,
/// so no nested worktree's files are
///  deleted one by one.
#[test]
fn a_nested_stub_is_removed_only_as_an_empty_directory() {
    let (root, stage) = stage_for_stubs("snapshot-stub-full");
    std::fs::create_dir_all(root.join("stage/nested")).expect("nested");
    std::fs::write(root.join("stage/nested/.git"), b"gitdir: y\n").expect("a pointer file in it");
    let error: String = remove_stub(&stage, Path::new("/src"), Path::new("/src/nested"), false).expect_err("not empty");
    assert!(error.contains("empty directory"), "{error}");
    assert!(root.join("stage/nested/.git").exists());
    // A path outside the source, or one the snapshot never held, is nothing to remove.
    remove_stub(&stage, Path::new("/src"), Path::new("/elsewhere/nested"), false).expect("outside the source");
    remove_stub(&stage, Path::new("/src"), Path::new("/src/missing/deep"), true).expect("absent");
    remove(root.as_path());
}

/// A destination that was swapped for another directory after Git made it is not exchanged,
/// and the stage is untouched.
#[test]
fn btrfs_a_destination_swapped_before_the_exchange_is_refused() {
    let Some(root) = btrfs_fixture("snapshot-swap-destination") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination.clone());
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    take(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect("snapshot");
    let identity: ObjectId = id_of_directory(destination.as_path());
    std::fs::rename(destination.as_path(), root.join("topic.moved")).expect("moved away");
    std::fs::create_dir(destination.as_path()).expect("another directory");
    let reason: String = put_in_place(stage.as_path(), destination.as_path(), 0o750, identity).expect_err("refused").reason;
    assert!(reason.contains("not the directory Git made"), "{reason}");
    assert!(stage.join("shared.txt").exists() && std::fs::read_dir(destination.as_path()).expect("listing").count() == 0, "nothing moved");
    remove(root.as_path());
}

/// A link where the destination should be is not a directory Git made.
#[test]
fn btrfs_a_destination_replaced_by_a_link_is_refused() {
    let Some(root) = btrfs_fixture("snapshot-link-destination") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination.clone());
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    take(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect("snapshot");
    let identity: ObjectId = id_of_directory(destination.as_path());
    std::fs::rename(destination.as_path(), root.join("topic.moved")).expect("moved away");
    std::os::unix::fs::symlink(root.join("topic.moved"), destination.as_path()).expect("a link, the very same inode behind it");
    assert!(put_in_place(stage.as_path(), destination.as_path(), 0o750, identity).is_err(), "a link is not the directory, whatever it points at");
    remove(root.as_path());
}

/// A held `index.lock` in the new worktree's administrative directory ends the rung before anything is written:
/// the index
///  is neither created nor changed.
#[test]
fn btrfs_a_held_index_lock_leaves_the_index_alone() {
    let Some(root) = btrfs_fixture("snapshot-index-lock") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    std::fs::write(admin.join("index.lock"), b"held by another Git").expect("a lock");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let reason: String = take(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect_err("refused");
    assert!(reason.contains("index.lock"), "{reason}");
    assert!(!admin.join("index").exists(), "no index was written before the lock was held");
    assert_eq!(std::fs::read(admin.join("index.lock")).expect("lock"), b"held by another Git", "the other's lock stays");
    // The same with an index already there: it keeps its bytes.
    std::fs::write(admin.join("index"), b"sentinel").expect("an index");
    take(&source, &plan, stage.with_file_name(".cli-git-worktree-copy-test-two").as_path(), admin.as_path(), pointer.as_slice()).expect_err("refused again");
    assert_eq!(std::fs::read(admin.join("index")).expect("index"), b"sentinel");
    remove(root.as_path());
}

/// The snapshot is checked against the index that was proven clean,
/// not a second read of the file:
/// a source that moved to
///  another commit after the check is refused.
#[test]
fn btrfs_the_snapshot_is_checked_against_the_frozen_index() {
    let Some(root) = btrfs_fixture("snapshot-frozen") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    crate::worktree_ladder_test_support::tracked(repo.as_path(), "later.txt", b"later\n");
    crate::worktree_ladder_test_support::commit_all(repo.as_path(), "later");
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    git(repo.as_path(), &["reset", "--quiet", "--hard", "HEAD~1"]);
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let reason: String = take(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect_err("refused");
    assert!(reason.contains("later.txt"), "{reason}");
    assert!(!admin.join("index").exists());
    remove(root.as_path());
}

/// A snapshot taken and ready to swap,
/// with Git's destination and the identity of the directory Git made.
fn ready_to_swap(root: &Path, name: &str) -> (PathBuf, PathBuf, ObjectId, Vec<u8>) {
    let repo: PathBuf = subvolume_repository(root);
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), name);
    let plan = plan_to(destination.clone());
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    take(&source, &plan, stage.as_path(), admin.as_path(), pointer.as_slice()).expect("snapshot");
    let made: ObjectId = id_of_directory(destination.as_path());
    return (destination, stage, made, pointer);
}

/// A file another process adds to the destination after it was looked at,
/// and before the exchange,
/// is not deleted:
/// it stays in
///  the displaced directory,
/// which stays,
/// and the notes name each file.
/// (A snapshot is a subvolume of its own,
/// and a rename
///  into another subvolume is refused,
/// so nothing is moved on btrfs;
/// where the rename is allowed the file is moved.)
#[test]
fn btrfs_files_added_to_the_destination_before_the_exchange_are_kept() {
    let Some(root) = btrfs_fixture("snapshot-added-before-swap") else { return };
    let (destination, stage, made, pointer) = ready_to_swap(root.as_path(), "topic");
    let mut add = || {
        std::fs::write(destination.join("notes"), b"written while the worktree was made").expect("a new file");
        std::fs::write(destination.join("shared.txt"), b"a file the snapshot also has").expect("a colliding file");
    };
    put_in_place_with(stage.as_path(), destination.as_path(), 0o750, made, &mut PlaceSeams { after_check: &mut add, before_swap_back: &mut || {} }).expect("swap");
    let notes: Vec<String> = settle_displaced(stage.as_path(), destination.as_path(), made, pointer.as_slice());
    assert_eq!(std::fs::read(destination.join("shared.txt")).expect("tracked"), b"second, longer\n", "the snapshot's file was not replaced");
    assert_eq!(std::fs::read(stage.join("shared.txt")).expect("kept in place"), b"a file the snapshot also has");
    let kept_notes: bool = stage.join("notes").exists();
    assert_eq!(kept_notes, !destination.join("notes").exists(), "the file is in exactly one place");
    assert_eq!(std::fs::read(if kept_notes { stage.join("notes") } else { destination.join("notes") }).expect("kept"), b"written while the worktree was made");
    assert!(notes.iter().any(|note: &String| { return note.contains("shared.txt") && note.contains("stays"); }), "{notes:?}");
    assert!(stage.exists() && !stage.join(".git").exists(), "the displaced directory stays with the files, and Git's pointer went");
    remove(root.as_path());
}

/// With nothing added,
/// the displaced directory is Git's pointer alone:
/// the pointer and the directory are removed,
/// and nothing
///  else is looked for.
#[test]
fn btrfs_a_displaced_directory_with_only_gits_pointer_is_removed() {
    let Some(root) = btrfs_fixture("snapshot-displaced-clean") else { return };
    let (destination, stage, made, pointer) = ready_to_swap(root.as_path(), "topic");
    put_in_place(stage.as_path(), destination.as_path(), 0o750, made).expect("swap");
    assert_eq!(settle_displaced(stage.as_path(), destination.as_path(), made, pointer.as_slice()), Vec::<String>::new());
    assert!(!stage.exists());
    remove(root.as_path());
}

/// A displaced directory that is not the one Git made,
/// or holds a `.git` other than Git's pointer,
/// is left alone.
#[test]
fn btrfs_a_displaced_directory_that_is_not_gits_is_left_alone() {
    let Some(root) = btrfs_fixture("snapshot-displaced-foreign") else { return };
    let (destination, stage, made, pointer) = ready_to_swap(root.as_path(), "topic");
    put_in_place(stage.as_path(), destination.as_path(), 0o750, made).expect("swap");
    std::fs::write(stage.join(".git"), b"gitdir: somewhere else\n").expect("another pointer");
    let notes: Vec<String> = settle_displaced(stage.as_path(), destination.as_path(), made, pointer.as_slice());
    assert!(notes.iter().any(|note: &String| { return note.contains("is not the pointer Git wrote"); }), "{notes:?}");
    assert!(stage.join(".git").exists(), "the file stays");
    let other: ObjectId = ObjectId { device: made.device, inode: made.inode + 1_000_000, birth: made.birth };
    let refused: Vec<String> = settle_displaced(stage.as_path(), destination.as_path(), other, pointer.as_slice());
    assert!(refused.iter().any(|note: &String| { return note.contains("is not the directory Git made"); }), "{refused:?}");
    assert!(stage.exists());
    remove(root.as_path());
}

/// A destination replaced between the check and the exchange is swapped back:
/// the replacement is where it was,
/// the snapshot is
///  at the stage name,
/// and the error says the stage is the wrapper's own.
#[test]
fn btrfs_a_destination_replaced_at_the_exchange_is_swapped_back() {
    let Some(root) = btrfs_fixture("snapshot-replaced-at-swap") else { return };
    let (destination, stage, made, _pointer) = ready_to_swap(root.as_path(), "topic");
    let mut replace = || {
        std::fs::rename(destination.as_path(), root.join("topic.moved")).expect("moved away");
        std::fs::create_dir(destination.as_path()).expect("another directory");
        std::fs::write(destination.join("theirs"), b"someone's directory").expect("their file");
    };
    let error: PlaceError = put_in_place_with(stage.as_path(), destination.as_path(), 0o750, made, &mut PlaceSeams { after_check: &mut replace, before_swap_back: &mut || {} }).expect_err("refused");
    assert!(error.stage_is_ours, "{error:?}");
    assert_eq!(std::fs::read(destination.join("theirs")).expect("their directory is back"), b"someone's directory");
    assert!(stage.join("shared.txt").exists(), "the snapshot is at the stage name again");
    remove(root.as_path());
}

/// When the swap back fails too,
/// the stage name holds somebody else's directory and the error says it is not the wrapper's:
///  the caller must not destroy it.
#[test]
fn btrfs_a_failed_swap_back_does_not_claim_the_stage() {
    let Some(root) = btrfs_fixture("snapshot-failed-swap-back") else { return };
    let (destination, stage, made, _pointer) = ready_to_swap(root.as_path(), "topic");
    let mut replace = || {
        std::fs::rename(destination.as_path(), root.join("topic.moved")).expect("moved away");
        std::fs::create_dir(destination.as_path()).expect("another directory");
        std::fs::write(destination.join("theirs"), b"someone's directory").expect("their file");
    };
    let mut remove_published = || {
        std::fs::rename(destination.as_path(), root.join("topic.gone")).expect("the published destination moves away");
    };
    let error: PlaceError = put_in_place_with(stage.as_path(), destination.as_path(), 0o750, made, &mut PlaceSeams { after_check: &mut replace, before_swap_back: &mut remove_published }).expect_err("refused");
    assert!(!error.stage_is_ours, "{error:?}");
    assert_eq!(std::fs::read(stage.join("theirs")).expect("their directory is at the stage name"), b"someone's directory");
    remove(root.as_path());
}

/// A snapshot that is taken reports the identity of the stage it made;
/// a snapshot that fails after the stage exists reports it
///  too,
/// so that the stage the failed run made is the one it may remove.
#[test]
fn btrfs_the_stage_a_snapshot_made_is_identified_even_when_the_snapshot_fails() {
    let Some(root) = btrfs_fixture("snapshot-stage-identity") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let (destination, admin, pointer) = no_checkout_worktree(repo.as_path(), "topic");
    let plan = plan_to(destination);
    let source = choose_snapshot_source(&git_context(), &main_identity(repo.as_path()), &plan).expect("a source");
    std::fs::write(admin.join("index.lock"), b"held by another Git").expect("a lock");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-test");
    let held: File = open_root(admin.as_path()).expect("administrative directory");
    let failure: SnapshotFailure = take_snapshot(&source, &plan, stage.as_path(), &held, pointer.as_slice()).expect_err("refused");
    assert_eq!(failure.stage, Some(id_of_directory(stage.as_path())), "the failed run names the stage it made");
    std::fs::remove_file(admin.join("index.lock")).expect("lock gone");
    let stage_two: PathBuf = root.join(".cli-git-worktree-copy-test-two");
    let made: ObjectId = take_snapshot(&source, &plan, stage_two.as_path(), &held, pointer.as_slice()).expect("snapshot").stage;
    assert_eq!(made, id_of_directory(stage_two.as_path()));
    remove(root.as_path());
}
