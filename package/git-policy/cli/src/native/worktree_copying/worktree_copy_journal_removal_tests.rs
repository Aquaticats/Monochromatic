//! Controls for the walk to the split depth of a stage removal:
//! a start the system refuses leaves the pool's closure queued,
//! and what that closure may do is what these controls pin.

use crate::no_threads_child_test_support::{let_threads_start_again, ran_in_child, refuse_threads_again};
use crate::test_support::{fixture, remove};
use super::{remove_tree_if_present, remove_tree_split};
use std::path::PathBuf;

/// What a later writer put at the stage path survives the walk that was queued before its start was refused,
/// and the removal that was refused its thread says that it did not run.
/// Why:
///  The pool queues a blocking closure before it asks the system for a thread
///  (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
///  `spawn_task`),
///  so a refused start leaves that closure queued and it runs when a thread becomes free.
///  The removal answers `NoPlaceWithinTheCeiling` and removes nothing, while the queued walk must do nothing when its
///  thread comes: the claim on the job's work is what promises that, and an entry a later writer put at that path is not
///  that walk's to delete.
#[test]
fn a_walk_queued_before_its_start_was_refused_never_deletes_data_written_after_the_fallback() {
    if ran_in_child(
        module_path!(),
        "a_walk_queued_before_its_start_was_refused_never_deletes_data_written_after_the_fallback",
    ) {
        return;
    }
    // The runtime is built while threads are allowed, and the refusal that follows is this control's own.
    let_threads_start_again().expect("threads start again while the runtime is built");
    // One blocking place, so the one job that starts after the refusal has the queued walk to run.
    crate::lazy_runtime::cap_workers(1);
    crate::lazy_runtime::run_on_runtime(async {}).expect("the runtime builds");
    let root: PathBuf = fixture("journal-removal-queued-walk");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.as_path()).expect("stage");
    std::fs::write(stage.join("old"), b"old").expect("old stage entry");

    refuse_threads_again().expect("refuse the walk its thread");
    let refused: std::io::Result<()> = remove_tree_split(stage.as_path());
    assert!(refused.is_err(), "a removal with no thread to walk on says it did not run: {refused:?}");
    assert!(stage.exists(), "nothing ran, so nothing of the stage was removed");

    std::fs::create_dir_all(stage.as_path()).expect("a later writer's directory");
    let later: PathBuf = stage.join("written-after-the-removal-returned");
    std::fs::write(later.as_path(), b"what a later writer put there").expect("a later writer's file");

    let_threads_start_again().expect("a thread becomes free for the queued walk");
    let drained = crate::lazy_runtime::run_on_runtime(crate::blocking_job::run_blocking_job(|| ()));
    assert!(matches!(drained, Ok(Ok(()))), "one blocking job ran after the refusal, so the queue was drained: {drained:?}");
    assert!(
        later.exists(),
        "the walk the pool queued before the refusal deleted data a later writer put there"
    );
    remove(root.as_path());
}

/// What:
///  A removal whose walk found no place removes nothing, on the calling task or anywhere else, and says so.
/// Why:
///  `run_a_job_or_no_place` answers that no place came without running the walk, and the plain recursive removal
///  used to run in its place: on the calling task, with no bound, so a tree whose storage stopped answering would hold
///  that task for ever (the review of `a7a6958f9d`, F3).
///  A stage that is left where it is, is left for a later look, and its journal still names it.
/// The body runs in a child that is refused every thread, which is the one way to reach the answer without waiting for
/// `PLACE_WAIT_CEILING`.
#[test]
fn a_removal_whose_walk_found_no_place_removes_nothing_and_says_so() {
    if ran_in_child(module_path!(), "a_removal_whose_walk_found_no_place_removes_nothing_and_says_so") {
        return;
    }
    let root: PathBuf = fixture("journal-removal-no-place");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("deep")).expect("a stage");
    std::fs::write(stage.join("deep/a-file"), b"a tree of a stage").expect("a file of the stage");
    let refused: crate::worktree_copy_error::WorktreeCopyError = remove_tree_if_present(stage.as_path()).expect_err("a walk that found no place removed nothing, so the removal says so");
    let said: String = refused.0.clone();
    assert!(said.contains("no place"), "the removal names the place that never came: {said}");
    assert!(
        stage.join("deep/a-file").exists(),
        "the removal ran the plain recursive deletion on the calling task although no place came: the stage is gone"
    );
    remove(root.as_path());
}
