//! The lazy runtime: one runtime for the process, a build that ends by its deadline whatever the multi-thread build does, and a
//! refusal instead of a panic when called from inside the runtime.

use super::*;
use std::time::Instant;

/// A multi-thread builder as production writes it, with a small pool.
fn real_multi_thread(workers: usize, blocking: usize) -> std::io::Result<Runtime> {
    return tokio::runtime::Builder::new_multi_thread().worker_threads(workers).max_blocking_threads(blocking).enable_all().build();
}

/// A multi-thread plan with two workers and two blocking threads.
const TWO: RuntimePlan = RuntimePlan::MultiThread { workers: 2, blocking_threads: 2 };

/// A plan for the current-thread runtime builds it and never calls the multi-thread builder.
#[test]
fn a_current_thread_plan_builds_the_current_thread_runtime() {
    let built: BuiltRuntime =
        build_with_deadline(RuntimePlan::CurrentThread, BUILD_DEADLINE, |_workers, _blocking| { panic!("the multi-thread builder ran") })
            .expect("a current-thread runtime");
    assert_eq!(built.flavor, BuiltFlavor::CurrentThread);
    assert_eq!(built.runtime.block_on(async { return 7; }), 7);
}

/// A multi-thread plan whose build finishes in time gives the multi-thread runtime with the planned workers.
#[test]
fn a_multi_thread_plan_that_builds_in_time_gives_the_multi_thread_runtime() {
    let built: BuiltRuntime = build_with_deadline(TWO, BUILD_DEADLINE, real_multi_thread).expect("a runtime");
    assert_eq!(built.flavor, BuiltFlavor::MultiThread { workers: 2 });
    assert_eq!(built.runtime.block_on(async { return 7; }), 7);
}

/// A build that outlasts the deadline is left running and the current-thread runtime stands in, by the deadline.
#[test]
fn a_build_that_outlasts_the_deadline_is_replaced_by_the_current_thread_runtime() {
    let deadline: Duration = Duration::from_millis(200);
    let started: Instant = Instant::now();
    let built: BuiltRuntime = build_with_deadline(TWO, deadline, move |workers, blocking| {
        std::thread::sleep(deadline * 5);
        return real_multi_thread(workers, blocking);
    })
    .expect("a runtime");
    let took: Duration = started.elapsed();
    assert_eq!(built.flavor, BuiltFlavor::CurrentThreadAfterDeadline);
    assert!(took >= deadline, "the deadline was not waited for: {took:?}");
    assert!(took < deadline * 3, "the build waited far past its deadline: {took:?}");
    assert_eq!(built.runtime.block_on(async { return 7; }), 7);
}

/// A build that panics, as tokio's does when the system refuses a thread, is replaced at once.
#[test]
fn a_build_that_panics_is_replaced_by_the_current_thread_runtime() {
    let built: BuiltRuntime = build_with_deadline(TWO, BUILD_DEADLINE, |_workers, _blocking| { panic!("OS can't spawn worker thread") })
        .expect("a runtime");
    assert_eq!(built.flavor, BuiltFlavor::CurrentThreadAfterFailure);
}

/// A build that reports an error is replaced at once.
#[test]
fn a_build_that_fails_is_replaced_by_the_current_thread_runtime() {
    let built: BuiltRuntime =
        build_with_deadline(TWO, BUILD_DEADLINE, |_workers, _blocking| { return Err(std::io::Error::other("no thread")) })
            .expect("a runtime");
    assert_eq!(built.flavor, BuiltFlavor::CurrentThreadAfterFailure);
}

/// The one runtime serves every call: a second call sees the flavor the first built.
#[test]
fn every_call_runs_on_the_same_runtime() {
    assert_eq!(built_flavor(), None, "nothing is built before the first need");
    assert_eq!(run_on_runtime(async { return 1; }).expect("runs"), 1);
    let first: Option<BuiltFlavor> = built_flavor();
    assert!(first.is_some());
    assert_eq!(run_on_runtime(async { return 2; }).expect("runs"), 2);
    assert_eq!(built_flavor(), first);
}

/// A call from inside the runtime is refused with an error where `block_on` would panic.
#[test]
fn a_call_from_inside_the_runtime_is_refused() {
    let inner = run_on_runtime(async { return run_on_runtime(async { return 1; }).map_err(|error| { return matches!(error, RuntimeError::Nested); }); })
        .expect("the outer call runs");
    assert_eq!(inner, Err(true));
}
