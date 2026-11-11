//! What:
//!  The words a service serves from a stand-in runtime with,
//! and the words a task that cannot get a worker ends it with.
//! Why:
//!  A service that ends because its runtime came out as a stand-in fails a person's commit,
//! so the line it leaves must name the cause and the wait
//! (`LATE_BUILD_PATIENCE` in `foundation/lazy_runtime.rs`),
//! and a process that never asked for a runtime must not be reported as one that built a bad one.

use super::*;
use crate::lazy_runtime::{
    BuiltFlavor, LateRuntimeForTheControl, RefusalForTheControl, a_control_refuses_every_task, a_control_stands_in_for_a_late_runtime, await_a_worker_within, built_flavor, late_build_arrived, run_on_runtime,
    tasks_can_run,
};

/// What:
///  A runtime this process built as the multi-thread one says nothing about its build.
/// Why:
///   The line exists for the one state a person must diagnose;
/// a healthy service builds in milliseconds and would only add noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runOnRuntime(async () => {}); expect(standInWords(runtimeBuild())).toBeUndefined();
/// ```
#[test]
fn a_multi_thread_runtime_says_nothing_about_its_build() {
    run_on_runtime(async {}).expect("the runtime runs one task");
    assert!(tasks_can_run(), "this process built a runtime whose workers run a task");
    let Some(build) = runtime_build() else { panic!("the runtime this process just built reports its build") };
    assert!(matches!(build.flavor, BuiltFlavor::MultiThread { .. }), "the runtime of this control is the multi-thread one: {:?}", build.flavor);
    assert_eq!(stand_in_words(build), None, "a healthy build says nothing");
}

/// What:
///  A build that missed its deadline is named with that cause and the wait the service gives it.
/// Why:
///   The gate of 2026-11-11T10:29-05:00 ended a service whose build had outlasted its deadline,
/// and the only line it left named neither the cause nor the wait
///  (`fault_trace::a_service_killed_at_each_ftruncate_leaves_a_recoverable_repository`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runOnRuntime(async () => {}); with aStandInForALateBuild: expect(standInWords(runtimeBuild())).toContain('outlasted')
/// ```
#[test]
fn a_stand_in_runtime_is_named_with_its_cause_and_the_wait() {
    run_on_runtime(async {}).expect("the runtime runs one task");
    let late: LateRuntimeForTheControl = a_control_stands_in_for_a_late_runtime();
    assert_eq!(built_flavor(), Some(BuiltFlavor::CurrentThreadAfterDeadline), "the control stands in for a build that missed its deadline");
    let Some(build) = runtime_build() else { panic!("the runtime this process built reports its build") };
    let words: String = stand_in_words(build).expect("a stand-in is named");
    drop(late);
    assert!(words.contains("outlasted"), "the line names the deadline as the cause: {words}");
    assert!(words.contains(&format!("{} ms", LATE_BUILD_PATIENCE.as_millis())), "the line names the wait the service gives the late build: {words}");
}

/// What:
///  The words of a task that must have a worker name the task, the cause, the wait and what to check.
/// Why:
///   A person whose commit failed because a service ended here reads this line and nothing else,
/// so it names the limit that may have refused the threads and the two that may have made them slow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(noWorkerWords('the acceptor of connections')).toContain('the acceptor of connections')
/// ```
#[test]
fn the_words_of_a_task_with_no_worker_name_it_the_wait_and_what_to_check() {
    run_on_runtime(async {}).expect("the runtime runs one task");
    let late: LateRuntimeForTheControl = a_control_stands_in_for_a_late_runtime();
    let words: String = no_worker_words("the acceptor of connections");
    drop(late);
    assert!(words.contains("the acceptor of connections"), "the line names the task it could not start: {words}");
    assert!(words.contains("no worker thread"), "the line says what this process lacked: {words}");
    assert!(words.contains("outlasted"), "the line names the cause of the stand-in: {words}");
    assert!(words.contains(&format!("{} ms", LATE_BUILD_PATIENCE.as_millis())), "the line names how long the wait lasted: {words}");
    assert!(words.contains("`ulimit -u`") && words.contains("pids.max") && words.contains("memory.max"), "the line names what to check: {words}");
}

/// What:
///  A process that never asked for a runtime is reported as one that never built one,
/// and a wait that finds no worker ends at its bound.
/// Why:
///   A service can reach its acceptor before anything needed the runtime,
/// and then no build exists to describe;
/// the wait that has nothing coming must end at `LATE_BUILD_PATIENCE` rather than wait on a build that will never be there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runtimeBuild()).toBeUndefined(); expect(awaitWorkerWithin(50ms)).toBe(false);
/// ```
#[test]
fn a_process_that_never_built_a_runtime_says_so_and_a_wait_with_nothing_coming_ends() {
    assert!(runtime_build().is_none(), "no runtime was asked for yet, so none is described");
    assert!(!late_build_arrived(), "no build missed a deadline here");
    let words: String = no_worker_words("the after-commit work");
    assert!(words.contains("the after-commit work") && words.contains("never built"), "the line names the task and that no build exists: {words}");
    let refused: RefusalForTheControl = a_control_refuses_every_task();
    let began: std::time::Instant = std::time::Instant::now();
    let arrived: bool = await_a_worker_within(THE_LONGEST_A_WAIT_THIS_CONTROL_RUNS);
    let waited: std::time::Duration = began.elapsed();
    drop(refused);
    assert!(!arrived, "a system that refuses every thread gives no worker");
    assert!(waited < THE_LONGEST_A_WAIT_THIS_CONTROL_RUNS * 10, "the wait ended at its bound and not later: {waited:?}");
}

/// The wait this control gives a system that refuses every thread:
/// short enough that the control ends at once,
/// since the bound itself is what it measures.
const THE_LONGEST_A_WAIT_THIS_CONTROL_RUNS: std::time::Duration = std::time::Duration::from_millis(50);
