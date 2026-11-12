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
/// The child is a process of its own,
/// so the fact is the code's.
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

/// How long the builder of the control below takes,
/// counted from the moment it is released:
/// a stated span,
/// so the control's ordering is its own and never the scheduler's.
const A_LATE_BUILD: Duration = Duration::from_millis(100);

/// How long the caller's stand-in is held in the control below:
/// longer than the late build needs,
/// so the build is certainly inside the window the control orders.
const THE_STAND_IN_IS_HELD: Duration = Duration::from_millis(400);

/// How long the control below gives the build after the deadline:
/// shorter than the build's own span,
/// so the deadline passes first and the ordering is the control's.
const BEFORE_THE_DEADLINE: Duration = Duration::from_millis(50);

/// How many workers the controls' runtime is planned for:
/// one is all a late build has to bring,
/// because one worker is what a watcher of a wait needs.
const ONE_WORKER: usize = 1;

/// How long a caller that must have a worker is given in the pair of controls below:
/// a stated span,
/// short enough that the control ends in a moment and long enough that the wait inside it is measurable.
const A_STATED_WAIT: Duration = Duration::from_millis(200);

/// How long the control that has nothing coming gives a wait that must not happen:
/// long enough that waiting the whole of it would be unmistakable in the control's own measurement.
const A_WAIT_NOTHING_ANSWERS: Duration = Duration::from_secs(5);

/// What:
///  A caller that must have a worker waits for a build that outlasted its deadline and is still coming.
/// Why:
///   The positive control of the pair below:
/// a caller's wait for a build that is on its way must not become no wait at all,
/// and the stand-in for a late build is how this process reaches that state,
/// because the runtime this process builds is the multi-thread one on any host that gives it a thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const late = standInForALateBuild(); const began = now(); expect(spawnOnRuntimeWithin(200, async () => {})).toBeUndefined(); expect(now() - began).toBeGreaterThanOrEqual(200)
/// ```
#[test]
fn a_task_that_needs_a_worker_waits_for_a_build_that_is_still_coming() {
    let refused: RefusalForTheControl = a_control_refuses_every_task();
    let late: LateRuntimeForTheControl = a_control_stands_in_for_a_late_runtime();
    let began: Instant = Instant::now();
    let started: Option<tokio::task::JoinHandle<()>> = spawn_on_runtime_within(A_STATED_WAIT, async {});
    let waited: Duration = began.elapsed();
    drop(late);
    drop(refused);
    assert!(started.is_none(), "a system that refuses every thread gives no worker, so nothing was started");
    assert!(waited >= A_STATED_WAIT, "the wait for a build that is still coming lasted its whole bound: {waited:?}");
}

/// What:
///  A caller that must have a worker does not wait when no build is on its way.
/// Why:
///   The other half of the pair:
/// a system that refuses every thread has nothing coming,
/// so a wait there spends the person's command on a state no wait can change,
/// and the service must end at once with the line that says which way its build went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = refuseEveryTask(); const began = now(); expect(spawnOnRuntimeWithin(5000, async () => {})).toBeUndefined(); expect(now() - began).toBeLessThan(1000)
/// ```
#[test]
fn a_task_that_needs_a_worker_does_not_wait_for_a_runtime_that_nothing_brings() {
    let refused: RefusalForTheControl = a_control_refuses_every_task();
    let began: Instant = Instant::now();
    let started: Option<tokio::task::JoinHandle<()>> = spawn_on_runtime_within(A_WAIT_NOTHING_ANSWERS, async {});
    let waited: Duration = began.elapsed();
    drop(refused);
    assert!(started.is_none(), "a system that refuses every thread starts nothing");
    assert!(
        waited < A_WAIT_NOTHING_ANSWERS / 2,
        "nothing was coming, so the caller did not wait the bound it was given: {waited:?} of {A_WAIT_NOTHING_ANSWERS:?}"
    );
}

/// What:
///  A runtime that arrives after the deadline and while the caller is in its stand-in is kept.
/// Why:
///   The window between the deadline and the closing of the channel is the caller's own stand-in:
/// a runtime that arrives in it is
///  neither received nor kept unless the caller reads the channel at the deadline,
/// and only the helper keeps a build that
///  arrives after the channel is closed.
/// The control orders the three steps through channels rather than by their length:
/// the stand-in tells a driver thread that
///  the deadline has passed,
/// the driver releases the build only then,
/// and the stand-in is held until the control says the build
///  has finished,
/// so no ordering here depends on how the scheduler ran the three threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = buildWithDeadlineAnd(plan, 50ms, () => { await released; return buildMultiThread(1) }, () => { began(); await builtTheBuild; return buildCurrentThread() });
/// expect(built.flavor).toBe('currentThreadAfterDeadline'); expect(lateArrived()).toBe(true);
/// ```
#[test]
fn a_build_that_arrives_while_the_caller_stands_in_is_kept() {
    let (began_sender, began) = std::sync::mpsc::channel::<()>();
    let (release_sender, release) = std::sync::mpsc::channel::<()>();
    let (built_sender, built) = std::sync::mpsc::channel::<()>();
    let driver = std::thread::spawn(move || {
        began.recv().expect("the caller reached its stand-in, so the deadline passed");
        release_sender.send(()).expect("the build is released after the deadline");
        // The build says it has sent its runtime; the stand-in is only then allowed to end, so the send lands inside it.
        built.recv().expect("the build sent its runtime");
    });
    let outcome: BuiltRuntime = build_with_deadline_and(
        RuntimePlan::MultiThread { workers: ONE_WORKER, blocking_threads: ONE_WORKER },
        BEFORE_THE_DEADLINE,
        move |workers: usize, blocking: usize| -> std::io::Result<Runtime> {
            release.recv().expect("the driver releases the build");
            let runtime: Runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(workers).max_blocking_threads(blocking).enable_all().build()?;
            built_sender.send(()).expect("the build says it sent its runtime");
            return Ok(runtime);
        },
        move || -> std::io::Result<Runtime> {
            began_sender.send(()).expect("the stand-in says the deadline passed");
            // Hold the caller here until the build has sent: the send lands while this stand-in is still running.
            std::thread::sleep(THE_STAND_IN_IS_HELD);
            return tokio::runtime::Builder::new_current_thread().enable_all().build();
        },
    )
    .expect("a runtime stands in");
    driver.join().expect("the driver ends");
    assert_eq!(outcome.flavor, BuiltFlavor::CurrentThreadAfterDeadline, "the deadline passed before the build was released");
    assert!(late_build_arrived(), "the runtime that arrived while the caller was in its stand-in was dropped with the channel");
}

/// What:
///  A build that arrives after the channel is closed
///  is kept by its own helper,
///  and a task runs on it.
/// Why:
///   The other half of the window:
/// a build that finishes once the caller has returned owns its runtime,
/// and a runtime dropped
///  here would never be dropped anywhere else without waiting for its tasks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = buildWithDeadlineAnd(plan, 50ms, () => { sleep(400ms); return buildMultiThread(1) }, () => currentThread());
/// await lateArrived; lateRuntime.spawn(task);
/// ```
#[test]
fn a_build_that_arrives_after_the_channel_is_closed_takes_the_work_it_was_built_for() {
    let built: BuiltRuntime = build_with_deadline(RuntimePlan::MultiThread { workers: ONE_WORKER, blocking_threads: ONE_WORKER }, BEFORE_THE_DEADLINE, |workers: usize, blocking: usize| -> std::io::Result<Runtime> {
        std::thread::sleep(A_LATE_BUILD);
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
