//! Controls for the walk to the split depth of a stage removal:
//! a start the system refuses leaves the pool's closure queued,
//! and what that closure may do is what these controls pin.

use crate::no_threads_child_test_support::{let_threads_start_again, ran_in_child, refuse_threads_again};
use crate::test_support::{fixture, remove};
use super::remove_tree_split;
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
