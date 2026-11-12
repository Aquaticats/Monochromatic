//! Controls of the relink a removal settles:
//! the record that names the worktree goes,
//! a record that names another worktree stays,
//! and a record the service already holds is left to the service.

use super::cancel_the_relink_of;
use crate::package_repair_relink::{RelinkEntry, RelinkTool, SearchPathOfTheAdd, item_of};
use crate::service_work::work_directory;
use crate::test_support::{fixture, remove};
use crate::work_item::{DONE, PENDING, WorkItem, enqueue, state_directory};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// One repository's common directory and the worktree a record names.
struct Scene {
    /// The fixture root,
    /// removed at the end.
    root: PathBuf,
    /// The common Git directory the queue lives under.
    common: PathBuf,
    /// The worktree a removal takes.
    worktree: PathBuf,
    /// Another worktree of the same repository.
    other: PathBuf,
}

/// A scene with two worktrees,
/// and a queue under the repository's common directory.
fn scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("worktree");
    let other: PathBuf = root.join("other");
    for directory in [&common, &worktree, &other] {
        std::fs::create_dir_all(directory).expect("directory");
    }
    return Scene { root, common, worktree, other };
}

/// The record a relink of `worktree` is queued as.
fn record_of(common: &Path, worktree: &Path) -> WorkItem {
    let entry: RelinkEntry = RelinkEntry { root: worktree.to_path_buf(), search_path: SearchPathOfTheAdd::Set(OsString::from("/usr/bin")), tools: vec![RelinkTool::Pnpm] };
    return item_of(common, worktree, &entry).expect("the record");
}

/// The names in one state directory of the scene's queue.
fn names_in(common: &Path, state: &str) -> Vec<String> {
    let listing = std::fs::read_dir(state_directory(work_directory(common).as_path(), state).as_path());
    let Ok(entries) = listing else { return Vec::new() };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    return names;
}

/// What:
///  A relink record waiting for a worktree the removal took is settled,
///  and the person reads why.
/// Why:
///   The add that asked for the relink is over and the directory the tools rewrite is gone,
///  so the record is work nothing can finish usefully,
///  and it would otherwise be claimed by a service and skipped later,
///  with a line about a worktree somebody already removed.
#[test]
fn a_record_a_removal_preceded_is_settled_and_said() {
    let scene: Scene = scene("relink-cancel-pending");
    let item: WorkItem = record_of(scene.common.as_path(), scene.worktree.as_path());
    enqueue(work_directory(scene.common.as_path()).as_path(), &item).expect("the record is queued");
    assert_eq!(names_in(scene.common.as_path(), PENDING), vec![item.id()], "the record waits before the removal");
    let lines: Vec<String> = cancel_the_relink_of(scene.common.as_path(), scene.worktree.as_path());
    assert_eq!(lines.len(), 1, "the settled record is said once: {lines:?}");
    assert!(lines[0].contains("relinking the package state of"), "{lines:?}");
    assert!(lines[0].contains(scene.worktree.to_str().expect("utf8")), "the line names the worktree: {lines:?}");
    assert!(names_in(scene.common.as_path(), PENDING).is_empty(), "nothing waits for a worktree that is gone");
    assert_eq!(names_in(scene.common.as_path(), DONE), vec![item.id()], "the record is settled, not dropped");
    remove(scene.root.as_path());
}

/// What:
///  A relink record of another worktree is left exactly where it is.
/// Why:
///   One repository's queue holds the relinks of every worktree a person added,
///  and a removal takes one of them;
///  settling another worktree's record would leave that worktree's links as the copy left them.
#[test]
fn a_record_of_another_worktree_stays() {
    let scene: Scene = scene("relink-cancel-other");
    let mine: WorkItem = record_of(scene.common.as_path(), scene.worktree.as_path());
    let theirs: WorkItem = record_of(scene.common.as_path(), scene.other.as_path());
    for item in [&mine, &theirs] {
        enqueue(work_directory(scene.common.as_path()).as_path(), item).expect("the record is queued");
    }
    let lines: Vec<String> = cancel_the_relink_of(scene.common.as_path(), scene.worktree.as_path());
    assert_eq!(lines.len(), 1, "one removal settles one record: {lines:?}");
    assert_eq!(names_in(scene.common.as_path(), PENDING), vec![theirs.id()], "the other worktree's record waits");
    assert_eq!(names_in(scene.common.as_path(), DONE), vec![mine.id()]);
    remove(scene.root.as_path());
}

/// What:
///  A record the service already holds is not touched,
///  and nothing is said about it.
/// Why:
///   A claim is the service's own work,
///  and the run it describes stops itself when the directory it works in is gone
///  and settles with its own words;
///  a removal that moved that file would take the claim away from the run holding it.
#[test]
fn a_record_the_service_holds_is_left_to_the_service() {
    let scene: Scene = scene("relink-cancel-claimed");
    let item: WorkItem = record_of(scene.common.as_path(), scene.worktree.as_path());
    enqueue(work_directory(scene.common.as_path()).as_path(), &item).expect("the record is queued");
    let pending: PathBuf = state_directory(work_directory(scene.common.as_path()).as_path(), PENDING).join(item.id());
    let claimed: PathBuf = state_directory(work_directory(scene.common.as_path()).as_path(), crate::work_item::CLAIMED).join(item.id());
    std::fs::create_dir_all(claimed.parent().expect("the claimed directory")).expect("directory");
    std::fs::rename(pending.as_path(), claimed.as_path()).expect("the service claims it");
    let lines: Vec<String> = cancel_the_relink_of(scene.common.as_path(), scene.worktree.as_path());
    assert!(lines.is_empty(), "a record the service holds is not the removal's to settle: {lines:?}");
    assert!(claimed.is_file(), "the claim stays where the service put it");
    remove(scene.root.as_path());
}

/// What:
///  A queue that was never written is one failed listing and nothing else.
/// Why:
///   Most removals happen in a repository whose service has never queued a relink,
///  and a removal must not report a state that was never there.
#[test]
fn a_repository_with_no_queue_says_nothing() {
    let scene: Scene = scene("relink-cancel-none");
    assert!(cancel_the_relink_of(scene.common.as_path(), scene.worktree.as_path()).is_empty());
    remove(scene.root.as_path());
}
