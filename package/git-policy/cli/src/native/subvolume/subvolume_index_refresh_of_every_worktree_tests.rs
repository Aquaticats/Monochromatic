//! The claim one index rewrite takes:
//! a rewrite the report gave up on does not run when the pool's next thread finds it queued,
//! and one that took the claim first makes the report say its result is not known.

use super::*;
use crate::no_threads_child_test_support::{let_threads_start_again, ran_in_child, refuse_threads_again};
use crate::parallel_runner_on_the_runtime::LinesToTheConsumer;

/// What:
///  A rewrite the report gave up on does not run afterwards,
///  although the pool had already queued it and its next thread runs what it queued.
/// Why:
///  The pool queues a job before it asks for a thread (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`),
/// so a report that says "given up on before it could start,
/// so those stay as they were" is only true
///  when the job finds the claim taken and does nothing.
/// The control runs in a child whose process limit is one,
/// because only a refused thread start queues a job this way.
#[test]
fn an_index_rewrite_reported_as_never_started_does_not_run_afterwards() {
    if !ran_in_child(module_path!(), "an_index_rewrite_reported_as_never_started_does_not_run_afterwards") {
        return;
    }
    // A runtime with workers of its own, built while a thread may start.
    let_threads_start_again().expect("the process limit rises");
    crate::lazy_runtime::run_on_runtime(async {}).expect("the runtime");
    // Every thread this process starts is refused now, so the rewrite is queued and cannot start.
    refuse_threads_again().expect("the process limit falls");
    let ran: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let counter: Arc<AtomicUsize> = Arc::clone(&ran);
    let rewrite: Arc<RewriteOne> = Arc::new(move |_index: &Path, _tree: &Path, _options: &RefreshOptions<'_>| {
        counter.fetch_add(1, Ordering::SeqCst);
        return String::from("rewritten");
    });
    let gave_no_result: Arc<RewritesThatGaveNoResult> = Arc::new(RewritesThatGaveNoResult::default());
    let item: RewriteEachIndex =
        RewriteEachIndex { rewrite, lock: IndexLock::Acquire, hash_length: Some(20), flushes_to_disk: false, gave_no_result: Arc::clone(&gave_no_result) };
    let target: IndexTarget = IndexTarget { index: PathBuf::from("/no/such/index"), tree: PathBuf::from("/no/such/tree"), references: Vec::new() };
    let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel::<crate::parallel_runner::Message<String>>();
    let line: String = crate::lazy_runtime::run_on_runtime(async { return item.run(0, &target, LinesToTheConsumer::new(&sender, 0)).await; })
        .expect("the runtime");
    assert_eq!(gave_no_result.never_started.load(Ordering::SeqCst), 1, "the item says its work was given up on before it could start: {line}");
    assert_eq!(ran.load(Ordering::SeqCst), 0, "a rewrite ran although no thread could start: {line}");
    // A thread may start again: the pool's first new thread runs what it queued, and the claim stops it.
    let_threads_start_again().expect("the process limit rises again");
    let drained = crate::lazy_runtime::run_on_runtime(async { return crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(|| ()).await; });
    assert!(matches!(drained, Ok(Ok(()))), "the job that starts the pool's first thread ran: {drained:?}");
    assert_eq!(
        ran.load(Ordering::SeqCst),
        0,
        "the rewrite the report gave up on ran afterwards, so the indexes are not as they were: {line}"
    );
}
