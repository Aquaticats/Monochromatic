//! What:
//!  The process's one tokio runtime,
//! built on the first need and never before,
//! sized from the budget and the limits,
//! built on a
//!  helper thread under a deadline,
//! and never dropped.
//! Why:
//!  The owner's rulings of 2026-11-07 (`doc/planning/git-policy-cli-full-tokio.md`,
//! "The owner's answers"):
//! "Full on tokio.
//!  Multi-thread."
//! with the worker count "sized to budget";
//! built "Lazily,
//! on first need" so a forwarded command pays nothing
//!  (measured +0.3 to +0.5 ms and about 125 minor page faults for an eager build);
//! "Helper thread + deadline" so a build the
//!  system refuses cannot hold a commit ("nothing should ever wait forever");
//! "Fail plainly" where no thread can start.
//! The shapes a built runtime has are in `what_a_built_runtime_is.rs`,
//! and the build of one is in `the_build_of_the_process_runtime.rs`:
//! the helper thread it runs on,
//! the deadline it is given,
//! and the runtime a build that missed that deadline keeps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // There is no TS counterpart of choosing a thread pool: Node has one event loop.
//! const result = await runOnRuntime(async () => { ... });
//! ```

/// The build of the process's runtime:
/// the deadline it is given,
/// the pause a control gives that build,
/// and the call that builds one and hands it over.
pub use crate::the_build_of_the_process_runtime::{
    BUILD_DEADLINE,
    SLOW_BUILD_VARIABLE,
    build_with_deadline,
};
/// The runtime a build that missed its deadline keeps,
/// the arena cap that build sets,
/// and the three parts of it this file asks for.
pub(crate) use crate::the_build_of_the_process_runtime::{
    LATE_RUNTIME,
    cap_malloc_arenas,
    error_text,
    multi_thread,
};
/// The pause a control asks a build to wait before it starts,
/// the build a control orders to finish after its deadline,
/// and the two moments a control holds a build and its caller in.
#[cfg(test)]
pub(crate) use crate::the_build_of_the_process_runtime::{
    THE_BUILD_STOPS_BEFORE_KEEPING,
    THE_CALLER_STOPS_WAITING,
    build_with_deadline_and,
    the_pause_a_control_asks_for,
};
/// The runtime and how it came to be what it is,
/// as a build produces them and as a caller reads them.
pub use crate::what_a_built_runtime_is::{
    BuiltFlavor,
    BuiltRuntime,
    RuntimeBuild,
    RuntimeError,
};
/// The clock a wait for a worker is measured on.
use std::time::Duration;
/// The runtime a late build keeps and the caller starts work on.
use tokio::runtime::Runtime;

/// The sizes a plan gives the multi-thread runtime.
use crate::runtime_plan::RuntimePlan;

/// What:
///  The runtime a build that outlasted `BUILD_DEADLINE` left for the work it was built for,
///  once it arrived,
///  which is how every question about a worker reads it.
/// Why:
///   This cell is the only state here that changes under a running process,
/// and a caller that reads it twice can describe two different moments:
/// "no worker yet" and then "no build coming" is a build that arrived between the two reads,
///  and a caller that cannot do its work without a worker then gives up while one is there
/// (the review of 2026-11-12T09:34-05:00 found that pair ending a commit service with status 4).
/// A control reads through this function too,
///  so it can answer as the moment before the arrival.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const late = lateRuntime(); // undefined until the build that missed its deadline arrives
/// ```
fn the_late_runtime() -> Option<&'static Runtime> {
    #[cfg(test)]
    if tests::this_read_of_the_late_runtime_is_held_back() {
        return None;
    }
    return LATE_RUNTIME.get();
}

/// The process's runtime,
/// built by the first call that needs it;
/// the error is the reason no runtime could be built.
static RUNTIME: std::sync::OnceLock<Result<BuiltRuntime, String>> = std::sync::OnceLock::new();

/// The most workers (and blocking-pool threads) the runtime may have,
/// where the process asked for a cap before the runtime was built;
/// `usize::MAX` for no cap.
static WORKER_CAP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(usize::MAX);

/// What:
///  Holds the runtime this process builds to at most `cap` workers and `cap` blocking-pool threads.
/// Why:
///  The commit service lives for hours and one runs per worktree,
/// so its idle threads are a cost of the budget-sized default (`service/service_main.rs`,
/// `SERVICE_WORKER_CAP`);
/// it asks before it needs the runtime.
/// A runtime that is already built is not resized (tokio cannot),
/// so a call after the first use changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// capWorkers(2)
/// ```
pub fn cap_workers(cap: usize) {
    WORKER_CAP.store(cap, std::sync::atomic::Ordering::SeqCst);
}

/// Plans from this process's budget and limits and builds,
/// and keeps how long the build took.
fn build_for_this_process() -> Result<BuiltRuntime, String> {
    cap_malloc_arenas();
    let host: crate::parallel_budget::HostResources = crate::parallel_budget::HostResources::read();
    let limits: crate::runtime_limits::RuntimeLimits = crate::runtime_limits::RuntimeLimits::read();
    // What:     the workers this process builds with: the room a control gave it, or the budget bounded by the limits.
    // Why:      the room is this process's own fact, applied here at the one moment the runtime is made, so that a
    //           control which plans a fixed host of its own reads only that host.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const workers = workersForThisProcess(host, limits);
    // ```
    let workers: usize = crate::runtime_plan::workers_for_this_process(&host, &limits);
    let plan: RuntimePlan = crate::runtime_plan::capped_at(
        crate::runtime_plan::plan_with_workers(workers),
        WORKER_CAP.load(std::sync::atomic::Ordering::SeqCst),
    );
    let began: std::time::Instant = std::time::Instant::now();
    let built: Result<BuiltRuntime, String> =
        build_with_deadline(plan, BUILD_DEADLINE, multi_thread).map_err(error_text);
    let _kept: bool = THE_BUILD_SPENT.set(began.elapsed()).is_ok();
    return built;
}

/// How long the build of this process's runtime took,
/// once it was asked for.
static THE_BUILD_SPENT: std::sync::OnceLock<Duration> = std::sync::OnceLock::new();

/// What:
///  How the process's runtime was built and how long that took,
///  or nothing before the first need.
/// Why:
///   A service that finds no worker must be able to say which way its build went,
/// and a person reading its log
///  must be able to tell a build the system refused from one that was merely slower than its deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const build = runtimeBuild(); // { flavor, spent } | undefined
/// ```
pub fn runtime_build() -> Option<RuntimeBuild> {
    let flavor: BuiltFlavor = built_flavor()?;
    let spent: Duration = *THE_BUILD_SPENT.get()?;
    return Some(RuntimeBuild {
        flavor,
        spent,
    });
}

/// What:
///  How the build named by `flavor` came out,
/// in the words a service's log and its refusal use.
/// Why:
///   The two stand-ins differ in what the person can do about them:
/// a build that missed its deadline was slow and may still arrive,
/// one the system refused or that failed has nothing coming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// flavorWords(flavor) // 'the multi-thread runtime with 4 workers'
/// ```
pub fn flavor_words(flavor: BuiltFlavor) -> String {
    return match flavor {
        BuiltFlavor::CurrentThread => String::from(
            "the current-thread runtime, because not one worker fitted under this process's thread and address-space limits",
        ),
        BuiltFlavor::MultiThread {
            workers,
        } => format!("the multi-thread runtime with {workers} workers"),
        BuiltFlavor::CurrentThreadAfterDeadline => {
            format!(
                "a current-thread stand-in, because the multi-thread build outlasted its {BUILD_DEADLINE:?} deadline"
            )
        }
        BuiltFlavor::CurrentThreadAfterFailure => String::from(
            "a current-thread stand-in, because the multi-thread build failed or could not get its own helper thread",
        ),
    };
}

/// What:
///  What a caller can do about a worker here and now:
///  whether one can run a task,
///  and whether a build that outlasted its deadline is still on its way.
/// Why:
///   One cell in this module changes under a running process,
/// the build that missed its deadline and arrived late,
/// and a caller that reads it once per question can describe an arrival it has passed:
/// "no worker yet" and then "no build coming" is a build that arrived between the two reads,
///  so a caller that cannot do its work without a worker gives up while one is there
/// (the review of 2026-11-12T09:34-05:00 found that pair ending a commit service with status 4).
/// Both answers read at one moment always agree about the arrival.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = workerAnswers(); // { canRun, aBuildIsComing }, read at one moment
/// ```
pub(crate) struct WorkerAnswers {
    /// Whether a task of this process's runtime runs on a worker thread of its own.
    can_run: bool,
    /// Whether a build that outlasted its deadline is still on its way,
    /// which is the one state a caller can usefully wait in.
    pub(crate) a_build_is_coming: bool,
}

/// What:
///  Reads both questions about a worker,
///  building the process's runtime at the first need as the first caller always did.
/// Why:
///   The two answers belong to one reading,
/// and `spawn_on_runtime_within` decides with both of them,
/// so the reading lives here and no caller asks the two questions one after the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = workerAnswers();
/// ```
pub(crate) fn the_worker_answers() -> WorkerAnswers {
    // The build is the first need of a runtime, as it always was for `tasks_can_run`.
    let _built: &Result<BuiltRuntime, String> = RUNTIME.get_or_init(build_for_this_process);
    // The one cell that can change under this call, read once: an arrival is seen by both answers or by neither.
    let arrived_late: bool = the_late_runtime().is_some();
    // What:
    //  Whether a task of this process runs on a worker thread of its own,
    //  which is a fact about the runtime the process really has.
    // Why:
    //  A control stands in for a build that missed its deadline in `built_flavor`, which is what a caller is told;
    // a control that stood in here too would report no worker for a process whose workers run,
    // and every watcher of a wait would be refused in the very state the control exists to measure.
    let can_run: bool = arrived_late
        || matches!(
            the_flavor_of_the_runtime_this_process_built(),
            Some(BuiltFlavor::MultiThread { .. })
        );
    // The flavor a caller is told, which a control may stand in for.
    let a_build_is_coming: bool =
        !arrived_late && built_flavor() == Some(BuiltFlavor::CurrentThreadAfterDeadline);
    #[cfg(test)]
    if REFUSED.load(std::sync::atomic::Ordering::SeqCst) {
        return WorkerAnswers {
            can_run: false,
            a_build_is_coming,
        };
    }
    return WorkerAnswers {
        can_run,
        a_build_is_coming,
    };
}

/// What:
///  Whether a task of this process's runtime runs on a worker thread of its own.
/// Why:
///  A watcher runs beside a thread that is blocked in a read or a wait,
/// so it needs a worker that runs without a caller driving
///  it;
/// a current-thread runtime runs tasks only while some thread calls `block_on`,
/// so nothing watches there,
///  and the caller is told that by this answer.
/// `spawn_on_runtime` asks it,
/// and so does every caller that must say why nothing will watch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tasksRun = runtime.workerCount > 0;
/// ```
pub fn tasks_can_run() -> bool {
    return the_worker_answers().can_run;
}

/// What:
///  Whether a control stands in for a system that refuses every thread,
/// so `tasks_can_run` answers false.
/// Why:
///  A control cannot exhaust this process's threads,
/// so the flag is the seam.
///  It answers for the whole process,
/// because `tasks_can_run` does:
/// a threaded run consults it from every control's threads,
///  so the control that arms it runs its body in a child of its own
/// (`crate::process_wide_state_child_test_support`),
///  and `a_control_refuses_every_task` refuses in any other process.
#[cfg(test)]
static REFUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  Makes `tasks_can_run` answer false until the returned value is dropped,
/// standing in for a system that refused a thread.
/// Why:
///  The seam of the control above;
/// the flag is cleared however the control ends,
/// a panic included.
/// Gotcha:
///  The flag belongs to the whole process,
/// so this refuses outside a child the control started,
/// which names the starter;
///  an answered value here reaches every other test of a threaded run,
/// where their watchers are refused with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = refuseTasks(); // tasksCanRun() === false until refused.release()
/// ```
#[cfg(test)]
pub fn a_control_refuses_every_task() -> RefusalForTheControl {
    crate::process_wide_state_child_test_support::refuse_outside_the_child(
        "the stand-in for a system that refuses every thread of this process's runtime",
    );
    REFUSED.store(true, std::sync::atomic::Ordering::SeqCst);
    return RefusalForTheControl;
}

/// The stand-in of `a_control_refuses_every_task`,
/// cleared when it is dropped.
#[cfg(test)]
pub struct RefusalForTheControl;

/// Clears the control's stand-in.
#[cfg(test)]
impl Drop for RefusalForTheControl {
    fn drop(&mut self) {
        REFUSED.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// What:
///  Starts `future` on the runtime a build that missed its deadline left behind,
/// or answers nothing when that build never arrived.
/// Why:
///   A stand-in runtime has no worker,
/// so the work a late build was made for runs on the build's own runtime;
/// the task is taken by a function rather than a closure,
/// so what is started and what it is started on are both named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const late = lateRuntime(); if (late === undefined) return undefined; return late.spawn(future);
/// ```
fn spawn_on_the_late_runtime<F>(future: F) -> Option<tokio::task::JoinHandle<F::Output>>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let late: &Runtime = the_late_runtime()?;
    return Some(late.spawn(future));
}

/// What:
///  Starts `future` as a task of the process's runtime,
/// building the runtime on the first call,
/// and returns nothing where that
///  runtime has no worker thread of its own (the current-thread stand-in,
/// or a build that failed).
/// Why:
///  A watcher runs beside a thread that is blocked in a read or a wait,
/// so it needs a worker that runs without a caller driving
///  it;
/// `tasks_can_run` is that question,
/// and `None` tells the caller that nothing watches.
/// Callable from any context,
/// the runtime itself
///  included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const task = setImmediate(() => future); // Node's loop is always there; a blocked caller there would block the task too.
/// ```
pub fn spawn_on_runtime<F>(future: F) -> Option<tokio::task::JoinHandle<F::Output>>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    if !tasks_can_run() {
        return None;
    }
    return match RUNTIME.get_or_init(build_for_this_process) {
        Ok(built) if matches!(built.flavor, BuiltFlavor::MultiThread { .. }) => {
            Some(built.runtime.spawn(future))
        }
        // A stand-in runtime has no worker of its own, but a build that arrived after its deadline does: the work it was
        //  built for runs there.
        Ok(_) | Err(_) => spawn_on_the_late_runtime(future),
    };
}

/// How often `await_a_worker_within` asks whether the late build arrived:
/// short enough that a worker that arrives is used at once,
/// long enough that the wait costs the scheduler nothing.
const A_WORKER_WAIT_STEP: Duration = Duration::from_millis(10);

/// What:
///  Whether a worker can run a task now,
/// waiting at most `within` for a build that missed its deadline to arrive.
/// Why:
///   A caller that must have a worker (`spawn_on_runtime_within`) asks once and gets an answer that is either a worker
/// or a wait that has run out;
/// the wait is bounded,
/// so nothing here waits without an end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ready = awaitWorkerWithin(5_000);
/// ```
pub fn await_a_worker_within(within: Duration) -> bool {
    let deadline: std::time::Instant = std::time::Instant::now() + within;
    loop {
        if tasks_can_run() {
            return true;
        }
        let left: Duration = deadline.saturating_duration_since(std::time::Instant::now());
        if left.is_zero() {
            return false;
        }
        std::thread::sleep(left.min(A_WORKER_WAIT_STEP));
    }
}

/// What:
///  `spawn_on_runtime`,
/// after waiting up to `within` for a build that outlasted its deadline and is still coming,
/// so a caller that cannot do its work without a worker does not give up at `BUILD_DEADLINE` itself.
/// Why:
///   A build that outlasted its deadline is kept for the work it was built for,
/// and that work includes the tasks a service cannot serve without;
/// ending the service at the deadline
///  failed the commits of a person whose machine was slow to give the process its threads.
/// A runtime nothing will bring is never waited for:
/// whether a worker runs now and whether a build is still on its way are one reading of the state,
/// so a build that arrives as the caller asks cannot make the first answer say no worker yet and the second say no build coming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const task = spawnOnRuntimeWithin(60_000, work); // undefined when no worker arrived in time
/// ```
pub fn spawn_on_runtime_within<F>(
    within: Duration,
    future: F,
) -> Option<tokio::task::JoinHandle<F::Output>>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    let answers: WorkerAnswers = the_worker_answers();
    if !answers.can_run && !(answers.a_build_is_coming && await_a_worker_within(within)) {
        return None;
    }
    return spawn_on_runtime(future);
}

/// Whether the build that missed its deadline has arrived;
/// a control of the late build reads it.
#[cfg(test)]
pub(crate) fn late_build_arrived() -> bool {
    return the_late_runtime().is_some();
}

/// What:
///  Whether a control stands in for a build that missed its deadline and arrived after it.
/// Why:
///   That state asks a caller to tell a process with no worker of its own from one whose workers arrived late,
///   and it cannot be produced on demand here:
///   the runtime is built once,
///   and only a build the helper thread finished after `BUILD_DEADLINE` reaches it.
///  The flag makes `built_flavor` answer the stand-in,
/// while `tasks_can_run` and `spawn_on_runtime` keep answering for the runtime this process really has,
///  which is the whole shape of that state.
///  The answer belongs to the whole process,
/// so the control that arms it runs its body in a child of its own.
#[cfg(test)]
static THE_BUILD_WAS_LATE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// What:
///  `fn a_control_stands_in_for_a_late_runtime() -> LateRuntimeForTheControl` makes `built_flavor` answer the stand-in
///  of a build that missed its deadline,
///  until the returned value is dropped.
///  The sibling a reader might expect is a stand-in taken from the runtime itself,
///  which no control can produce:
///  this process builds one runtime,
///  and only a build that outlasted `BUILD_DEADLINE` reaches it.
/// Why:
///  A caller that must tell a process with no worker of its own from one whose workers arrived late needs that state
///  reachable in a control.
/// Gotcha:
///  The flag belongs to the whole process,
/// so this refuses outside a child the control started,
/// which names the starter;
///  a `built_flavor` answered here is answered for every other test of a threaded run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const late = standInForALateBuild(); // builtFlavor() answers the stand-in until late is dropped
/// ```
#[cfg(test)]
pub fn a_control_stands_in_for_a_late_runtime() -> LateRuntimeForTheControl {
    crate::process_wide_state_child_test_support::refuse_outside_the_child(
        "the stand-in for a runtime build that outlasted its deadline",
    );
    THE_BUILD_WAS_LATE.store(true, std::sync::atomic::Ordering::SeqCst);
    return LateRuntimeForTheControl;
}

/// The stand-in of `a_control_stands_in_for_a_late_runtime`,
/// cleared when it is dropped.
#[cfg(test)]
pub struct LateRuntimeForTheControl;

/// Clears the control's stand-in.
#[cfg(test)]
impl Drop for LateRuntimeForTheControl {
    fn drop(&mut self) {
        THE_BUILD_WAS_LATE.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// What:
///  Runs `future` to its end on the process's runtime,
/// building the runtime on the first call.
/// Why:
///  One owner builds the only runtime,
/// lazily,
/// so a command that needs none never pays for one.
/// A call from inside the runtime
///  returns `RuntimeError::Nested` where `Runtime::block_on` would panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = await future; // Node's loop is always there; here the loop is built on first need.
/// ```
pub fn run_on_runtime<F: std::future::Future>(future: F) -> Result<F::Output, RuntimeError> {
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(RuntimeError::Nested);
    }
    return match RUNTIME.get_or_init(build_for_this_process) {
        Ok(built) => Ok(built.runtime.block_on(future)),
        Err(reason) => Err(RuntimeError::CouldNotStart(reason.clone())),
    };
}

/// How the process's runtime was built,
/// or nothing before the first need.
pub fn built_flavor() -> Option<BuiltFlavor> {
    #[cfg(test)]
    if THE_BUILD_WAS_LATE.load(std::sync::atomic::Ordering::SeqCst) {
        return Some(BuiltFlavor::CurrentThreadAfterDeadline);
    }
    return the_flavor_of_the_runtime_this_process_built();
}

/// What:
///  How the runtime this process really built came out,
///  or nothing before anything asked for it,
///  and never a control's stand-in.
/// Why:
///   `built_flavor` answers what a caller is told,
/// where a control stands in for a build that missed its deadline;
/// whether a task can run now is a question about the runtime the process has,
/// so it is asked here and not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const flavorTheProcessHas = built?.ok ? built.value.flavor : undefined;
/// ```
fn the_flavor_of_the_runtime_this_process_built() -> Option<BuiltFlavor> {
    // `?` answers nothing before the first need, and `ok` turns a build the system refused into nothing as well.
    let built: &BuiltRuntime = RUNTIME.get()?.as_ref().ok()?;
    // `Some` carries the flavor out.
    return Some(built.flavor);
}

/// Runtime controls stay out of the release executable.
#[cfg(test)]
#[path = "lazy_runtime_tests.rs"]
mod tests;
