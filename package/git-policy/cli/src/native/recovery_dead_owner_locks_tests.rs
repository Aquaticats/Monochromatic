//! Controls for finding a dead owner's real index and for the release of what it held.

use super::*;
use crate::real_index_lock::real_lock_path;
use crate::recovery_files::lock_pid_path;
use crate::recovery_unrecorded_lock::tests::{DEAD, ID, Scene, scene};
use crate::test_support::remove;

/// The PID file announces an owner only by its exact text.
#[test]
fn a_pid_file_announces_its_owner_by_exact_text() {
    let scene: Scene = scene("unrecorded-announces");
    assert!(announces_owner(scene.index.as_path(), DEAD));
    assert!(!announces_owner(scene.index.as_path(), DEAD + 1));
    assert!(!announces_owner(scene.index.as_path(), 0));
    std::fs::write(lock_pid_path(scene.index.as_path()), format!("pid {DEAD}")).expect("no newline");
    assert!(!announces_owner(scene.index.as_path(), DEAD));
    std::fs::remove_file(lock_pid_path(scene.index.as_path())).expect("remove");
    assert!(!announces_owner(scene.index.as_path(), DEAD));
    remove(scene.root.as_path());
}

/// Without `preparing.json` the real index is the worktree's own, found from the registry's place.
#[test]
fn the_real_index_defaults_to_the_worktrees_own() {
    let scene: Scene = scene("unrecorded-default-index");
    assert_eq!(real_index_of_transaction(scene.directory.as_path()), scene.index);
    std::fs::write(scene.directory.join(PREPARING_FILENAME), b"not a record").expect("garbage");
    assert_eq!(real_index_of_transaction(scene.directory.as_path()), scene.index);
    remove(scene.root.as_path());
}

/// A dead owner with no lock record and no announcing PID file progressed nowhere, and nothing is touched.
#[test]
fn a_transaction_that_announced_nothing_is_not_progress() {
    let scene: Scene = scene("unrecorded-no-progress");
    std::fs::remove_file(lock_pid_path(scene.index.as_path())).expect("remove");
    let progressed: bool =
        release_without_preparing(scene.root.join("objects").as_path(), scene.directory.as_path(), &[], ID, DEAD).expect("judged");
    assert!(!progressed);
    assert!(real_lock_path(scene.index.as_path()).exists());
    remove(scene.root.as_path());
}

/// A transaction that announced a lock progressed past its preparing record: the PID file naming the dead owner is
/// released with the rest, whether or not the lock itself is proven.
#[test]
fn a_transaction_that_announced_a_lock_progressed_and_loses_its_pid_file() {
    let scene: Scene = scene("dead-owner-progress");
    let progressed: bool =
        release_without_preparing(scene.root.join("objects").as_path(), scene.directory.as_path(), &[], ID, DEAD).expect("judged");
    assert!(progressed);
    assert!(!lock_pid_path(scene.index.as_path()).exists());
    remove(scene.root.as_path());
}
