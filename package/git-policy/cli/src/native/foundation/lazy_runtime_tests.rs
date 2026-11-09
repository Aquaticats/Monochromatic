//! The lazy runtime:
//! one runtime for the process,
//! a build that ends by its deadline whatever the multi-thread build does,
//! and a
//! refusal instead of a panic when called from inside the runtime.

use super::*;
use std::time::Instant;

/// A multi-thread builder as production writes it,
/// with a small pool.
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

/// A build that outlasts the deadline is left running and the current-thread runtime stands in,
/// by the deadline.
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

/// A build that panics,
/// as tokio's does when the system refuses a thread,
/// is replaced at once.
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

/// The one runtime serves every call:
/// a second call sees the flavor the first built.
#[test]
fn every_call_runs_on_the_same_runtime() {
    // Whether one is built before the first need is the control of the child below: a sibling test of this process may have
    //  built it already, and that is no part of what this one says.
    assert_eq!(run_on_runtime(async { return 1; }).expect("runs"), 1);
    let first: Option<BuiltFlavor> = built_flavor();
    assert!(first.is_some());
    assert_eq!(run_on_runtime(async { return 2; }).expect("runs"), 2);
    assert_eq!(built_flavor(), first);
}

/// What:
///  The path this file's child test `name` has when the test binary is run directly.
/// Why:
///   A test binary names its tests from the module path without the crate name,
/// and a name that matches no test runs
///  nothing and succeeds,
/// which the control below rules out by reading what the child said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = `${module_path.split('::').slice(1).join('::')}::${name}`;
/// ```
fn child_path(name: &str) -> String {
    let module: &str = module_path!();
    let without_crate: &str = module.split_once("::").map_or(module, |(_, rest)| { return rest; });
    return format!("{without_crate}::{name}");
}

/// Child of the control below:
/// a process that has needed no runtime yet.
#[test]
#[ignore = "started by nothing_is_built_before_the_first_need in a process of its own"]
fn child_that_has_needed_no_runtime_yet() {
    let before: Option<BuiltFlavor> = built_flavor();
    let ran: u32 = run_on_runtime(async { return 7; }).expect("runs");
    let after: Option<BuiltFlavor> = built_flavor();
    assert!(ran == 7 && before.is_none() && after.is_some(), "before {before:?}, after {after:?}");
}

/// What:
///  Nothing is built before the first need,
///  and the first need builds the one runtime.
/// Why:
///   Laziness is measurable only in a process that has needed nothing,
/// and a sibling test of the same process can have built the runtime
///  before this one runs,
/// which made the assertion an order of the runner's rather than a fact about the code.
/// The child is a process of its own, so the fact is the code's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = spawnSync(exe, ['--exact', childPath('childThatHasNeededNoRuntimeYet'), '--ignored']);
/// expect(output.status).toBe(0);
/// ```
#[test]
fn nothing_is_built_before_the_first_need() {
    let output: std::process::Output = std::process::Command::new(crate::test_support::this_test_binary().expect("the test binary"))
        .args(["--exact", child_path("child_that_has_needed_no_runtime_yet").as_str(), "--ignored", "--nocapture", "--test-threads=1"])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the child test runs");
    let ran: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(ran.contains(child_path("child_that_has_needed_no_runtime_yet").as_str()), "the child ran the test named, not nothing:\n{ran}");
    assert!(output.status.success(), "a process that has needed no runtime builds one at its first need:\n{ran}\n{said}");
}

/// A call from inside the runtime is refused with an error where `block_on` would panic.
#[test]
fn a_call_from_inside_the_runtime_is_refused() {
    let inner = run_on_runtime(async { return run_on_runtime(async { return 1; }).map_err(|error| { return matches!(error, RuntimeError::Nested); }); })
        .expect("the outer call runs");
    assert_eq!(inner, Err(true));
}

/// A task spawned on the runtime runs while the thread that spawned it is blocked in a plain wait,
/// which is what a watcher does
///  beside a blocked reader;
/// the control would fail if the task ran only when the caller drove the runtime.
#[test]
fn a_spawned_task_runs_while_the_caller_is_blocked() {
    let (sender, receiver) = std::sync::mpsc::channel::<u8>();
    let task = spawn_on_runtime(async move { let _sent = sender.send(7); });
    assert!(task.is_some(), "this host's plan has workers, so the runtime is multi-thread");
    assert_eq!(receiver.recv_timeout(Duration::from_secs(30)).expect("the task ran on a worker"), 7);
}

/// How long the builder of the control below takes:
/// four times the deadline it is given.
const SLOW_BUILD: Duration = Duration::from_millis(200);

/// How many workers the control's runtime is planned for:
/// one is all a late build has to bring, because one worker is what a watcher of a wait needs.
const ONE_WORKER: usize = 1;

/// What:
///  A build that finishes after its deadline is kept,
///  and a task runs on it.
/// Why:
///   The current-thread stand-in runs tasks only while a caller drives it,
/// so a process left on it has no watcher of any wait:
///  `spawn_on_runtime` answers nothing there,
/// and a child that stopped showing work was never ended.
/// Discarding the late build left the process
///  that way for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = await buildWithDeadline({ kind: 'multi', workers: 1 }, 50, () => { sleep(200); return buildMultiThread(1) });
/// expect(built.flavor.kind).toBe('currentThreadAfterDeadline'); await lateArrived; lateRuntime.spawn(task);
/// ```
#[test]
fn a_build_that_arrives_after_its_deadline_takes_the_work_it_was_built_for() {
    let built: BuiltRuntime = build_with_deadline(RuntimePlan::MultiThread { workers: ONE_WORKER, blocking_threads: ONE_WORKER }, SLOW_BUILD / 4, |workers: usize, blocking: usize| -> std::io::Result<Runtime> {
        std::thread::sleep(SLOW_BUILD);
        return tokio::runtime::Builder::new_multi_thread().worker_threads(workers).max_blocking_threads(blocking).enable_all().build();
    })
    .expect("a runtime stands in");
    assert_eq!(built.flavor, BuiltFlavor::CurrentThreadAfterDeadline, "the deadline passed before the build finished");
    let arrived: bool = crate::test_support::wait_until(&|| { return late_build_arrived(); });
    assert!(arrived, "the build that missed its deadline never arrived, so its runtime was thrown away");
    let late: &Runtime = LATE_RUNTIME.get().expect("the late build is kept");
    let ran: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mark = std::sync::Arc::clone(&ran);
    let task = late.spawn(async move {
        mark.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    // Driven on the late runtime itself, so this control neither builds nor uses this process's own runtime.
    late.block_on(task).expect("the task of the late runtime runs");
    assert!(ran.load(std::sync::atomic::Ordering::SeqCst), "a task of the late runtime ran without a caller driving it");
}
