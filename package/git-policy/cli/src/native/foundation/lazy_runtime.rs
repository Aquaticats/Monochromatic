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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // There is no TS counterpart of choosing a thread pool: Node has one event loop.
//! const result = await runOnRuntime(async () => { ... });
//! ```

use std::time::Duration;
use tokio::runtime::Runtime;

use crate::runtime_plan::RuntimePlan;

/// How the runtime that was built came to be what it is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltFlavor {
    /// The plan was the current-thread runtime.
    CurrentThread,
    /// The multi-thread runtime was built with this many workers.
    MultiThread {
        /// Worker threads.
        workers: usize,
    },
    /// The multi-thread build did not finish by the deadline;
    /// the current-thread runtime stands in.
    CurrentThreadAfterDeadline,
    /// The multi-thread build failed,
    /// panicked or could not get its helper thread;
    /// the current-thread runtime stands in.
    CurrentThreadAfterFailure,
}

/// A runtime and how it came to be.
pub struct BuiltRuntime {
    /// The runtime itself;
    /// it is never dropped.
    pub runtime: Runtime,
    /// How it was built.
    pub flavor: BuiltFlavor,
}

/// Why a future could not be run on the runtime.
#[derive(Debug)]
pub enum RuntimeError {
    /// Called from inside the runtime,
    /// where `block_on` would panic.
    Nested,
    /// No runtime could be built,
    /// with the system's reason.
    CouldNotStart(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            RuntimeError::Nested => write!(
                formatter,
                "the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; \
                 nothing was run. This is a defect of monochromatic-git-policy-cli: report it with the command that was run"
            ),
            RuntimeError::CouldNotStart(reason) => write!(
                formatter,
                "cannot start the runtime that runs this command's waits: {reason}. Hint: the system refused a thread or memory, so \
                 check the process limit (`ulimit -u`), the address-space limit (`ulimit -v`) and the control group's `pids.max` and \
                 `memory.max`, free some, and run the command again; no Git command ran"
            ),
        };
    }
}

/// How long the build of the multi-thread runtime may take before the current-thread runtime stands in.
/// A build takes well under
///  a few milliseconds when it works (the probe of `doc/planning/git-policy-cli-full-tokio.md` measured +0.3 to +0.5 ms for the
///  whole eager build with 28 workers);
/// a build the system refuses can hang inside tokio's unwind with no end.
/// One second is
///  more than two hundred times the working build and a fifth of the owner's "should not take 5s longer" for a commit,
/// so a hung
///  build costs a commit part of its slack and never all of it.
/// The coordinator's pick under the owner's choice 9
///  ("Helper thread + deadline"),
/// open to the owner's veto.
pub const BUILD_DEADLINE: Duration = Duration::from_secs(1);

/// The most malloc arenas the process may create,
/// set before the first worker starts.
/// Each arena past the first reserves 64 MiB
///  of address space per thread that uses it,
/// and 28 workers with the default arenas hung the build under an address-space limit
///  of 900 to 1150 MiB where the same workers with one arena ran at 100 MiB (`doc/planning/git-policy-cli-full-tokio.md`,
///  "Why the address space runs out");
/// the plan's per-thread figures assume this value.
/// The owner's choice 3 ("Cap pools + read
///  limits").
/// Allocation contention is measured in the same document's progress notes.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
const MALLOC_ARENA_LIMIT: libc::c_int = 1;

/// Caps the malloc arenas of the process;
/// a no-op where the allocator has no arenas.
fn cap_malloc_arenas() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        // SAFETY: `mallopt` takes two integers and reads no memory; it runs before this process starts its first worker.
        let _ = unsafe { libc::mallopt(libc::M_ARENA_MAX, MALLOC_ARENA_LIMIT) };
    }
}

/// The current-thread runtime:
/// no worker,
/// the calling thread runs every task.
fn current_thread() -> std::io::Result<Runtime> {
    return tokio::runtime::Builder::new_current_thread().enable_all().build();
}

/// The multi-thread runtime with `workers` workers and at most `blocking` blocking-pool threads.
fn multi_thread(workers: usize, blocking: usize) -> std::io::Result<Runtime> {
    return tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .max_blocking_threads(blocking)
        .thread_stack_size(crate::runtime_plan::THREAD_STACK_BYTES)
        .thread_name("git-policy-worker")
        .enable_all()
        .build();
}

/// Runs a runtime builder on its two sizes,
/// the state and function a guarded call takes.
fn run_multi_thread_builder<B>(state: (B, usize, usize)) -> std::io::Result<Runtime>
where
    B: FnOnce(usize, usize) -> std::io::Result<Runtime>,
{
    let (builder, workers, blocking_threads): (B, usize, usize) = state;
    return builder(workers, blocking_threads);
}

/// The text of a build error,
/// the reason a process has no runtime.
fn error_text(error: std::io::Error) -> String {
    return error.to_string();
}

/// What:
///  Builds the runtime `plan` names.
/// The multi-thread runtime is built by `multi_thread` on a helper thread that the caller waits
///  for until `deadline`;
/// when the helper fails,
/// panics,
/// cannot be started or is late,
/// the current-thread runtime stands in and
///  the helper is left to finish or hang without the caller.
/// Why:
///  tokio's multi-thread build panics when the system refuses a thread and can hang inside the unwind,
/// where no `catch_unwind`
///  reaches;
/// only a wait with a deadline from outside bounds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = await Promise.race([buildMulti(workers, blocking), sleep(deadline).then(() => 'late')]);
/// return built === 'late' ? currentThreadRuntime() : built;
/// ```
pub fn build_with_deadline<B>(plan: RuntimePlan, deadline: Duration, multi_thread_builder: B) -> std::io::Result<BuiltRuntime>
where
    B: FnOnce(usize, usize) -> std::io::Result<Runtime> + Send + 'static,
{
    return build_with_deadline_and(plan, deadline, multi_thread_builder, current_thread);
}

/// What:
///  `build_with_deadline` with the stand-in given as `fallback`.
/// Why:
///   The window in which a late build can be lost is the caller's own stand-in:
/// a runtime sent after the deadline but before
///  the channel is closed is neither received nor kept.
/// A control can only order a build into that window if the caller can be held inside
///  the stand-in,
/// which is what this parameter is for;
/// the process builds its own runtime through `build_with_deadline`,
///  whose stand-in is the current-thread runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = buildWithDeadlineAnd(plan, deadline, multiThread, fallback);
/// ```
fn build_with_deadline_and<B, F>(plan: RuntimePlan, deadline: Duration, multi_thread_builder: B, fallback: F) -> std::io::Result<BuiltRuntime>
where
    B: FnOnce(usize, usize) -> std::io::Result<Runtime> + Send + 'static,
    F: FnOnce() -> std::io::Result<Runtime>,
{
    let RuntimePlan::MultiThread { workers, blocking_threads } = plan else {
        return Ok(BuiltRuntime { runtime: current_thread()?, flavor: BuiltFlavor::CurrentThread });
    };
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<Runtime>>();
    let started = std::thread::Builder::new().name(String::from("git-policy-runtime-build")).spawn(move || {
        // A refused thread panics inside tokio's build; the panic is caught here and reported as a replaced runtime.
        let outcome: std::thread::Result<std::io::Result<Runtime>> =
            crate::panic_notice::while_panics_are_expected((multi_thread_builder, workers, blocking_threads), run_multi_thread_builder::<B>);
        let result: std::io::Result<Runtime> = match outcome {
            Ok(built) => built,
            Err(_payload) => Err(std::io::Error::other("the build of the multi-thread runtime panicked")),
        };
        if let Err(unsent) = sender.send(result)
            && let Ok(late) = unsent.0
        {
            // The caller moved on: a working runtime that arrived after the deadline is kept for the work it was built for
            //  (`spawn_on_runtime`), and one that cannot be kept is never dropped here (dropping waits for its tasks
            //  without end).
            if let Err(unkept) = LATE_RUNTIME.set(late) {
                std::mem::forget(unkept);
            }
        }
    });
    let flavor_when_replaced: BuiltFlavor = match started {
        Err(_no_thread) => BuiltFlavor::CurrentThreadAfterFailure,
        Ok(_helper) => match receiver.recv_timeout(deadline) {
            Ok(Ok(runtime)) => return Ok(BuiltRuntime { runtime, flavor: BuiltFlavor::MultiThread { workers } }),
            Ok(Err(_refused)) => BuiltFlavor::CurrentThreadAfterFailure,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // A build that finished between the deadline and here is already in the channel, and one that finishes
                //  after the channel is closed is kept by its own helper: measure the two, so neither is dropped.
                let _kept: bool = keep_a_late_build(receiver.try_recv());
                drop(receiver);
                BuiltFlavor::CurrentThreadAfterDeadline
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => BuiltFlavor::CurrentThreadAfterFailure,
        },
    };
    return Ok(BuiltRuntime { runtime: fallback()?, flavor: flavor_when_replaced });
}

/// What:
///  Keeps a runtime that finished after its build deadline for the work it was built for,
///  and gives whether it was one.
/// Why:
///   A runtime that arrived while the caller was still in its stand-in is in the channel when the caller times out,
/// and the
///  channel is closed a moment later:
/// reading it at the deadline is what keeps that runtime,
/// and a receiver already closed
///  makes the build that arrives after it the helper's to keep.
/// A build that failed keeps nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = keepALateBuild(receiver.tryRecv()); // true when a runtime was kept
/// ```
fn keep_a_late_build(outcome: Result<std::io::Result<Runtime>, std::sync::mpsc::TryRecvError>) -> bool {
    let Ok(Ok(late)) = outcome else { return false };
    if let Err(unkept) = LATE_RUNTIME.set(late) {
        std::mem::forget(unkept);
    }
    return true;
}

/// What:
///  The multi-thread runtime of a build that finished after `BUILD_DEADLINE`,
/// once it arrived.
/// Why:
///   A build that misses the deadline is a working runtime that arrived after the caller stopped waiting.
/// Discarding it left the
///  process on the current-thread stand-in for ever,
/// and every watcher of a wait with it (`spawn_on_runtime` answers nothing there,
///  so a stuck child was never ended),
/// where the late runtime can still take that work.
/// The slot is filled by the helper thread of the build that was late,
/// and read by `spawn_on_runtime`,
/// which is why a
///  process whose build was late keeps answering with nothing until it arrives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lateRuntime: Runtime | undefined = undefined; // set by the build that missed its deadline
/// ```
static LATE_RUNTIME: std::sync::OnceLock<Runtime> = std::sync::OnceLock::new();

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

/// Plans from this process's budget and limits and builds.
fn build_for_this_process() -> Result<BuiltRuntime, String> {
    cap_malloc_arenas();
    let host: crate::parallel_budget::HostResources = crate::parallel_budget::HostResources::read();
    let limits: crate::runtime_limits::RuntimeLimits = crate::runtime_limits::RuntimeLimits::read();
    let plan: RuntimePlan = crate::runtime_plan::capped_at(crate::runtime_plan::plan_runtime(&host, &limits), WORKER_CAP.load(std::sync::atomic::Ordering::SeqCst));
    return build_with_deadline(plan, BUILD_DEADLINE, multi_thread).map_err(error_text);
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
    #[cfg(test)]
    if REFUSED.load(std::sync::atomic::Ordering::SeqCst) {
        return false;
    }
    return match RUNTIME.get_or_init(build_for_this_process) {
        Ok(built) => matches!(built.flavor, BuiltFlavor::MultiThread { .. }) || LATE_RUNTIME.get().is_some(),
        Err(_no_runtime) => LATE_RUNTIME.get().is_some(),
    };
}

/// What:
///  Whether a control stands in for a system that refuses every thread,
/// so `tasks_can_run` answers false.
/// Why:
///  A control cannot exhaust this process's threads;
/// the flag is the seam,
/// and nextest gives every
///  control its own process,
/// so it cannot reach another.
#[cfg(test)]
static REFUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  Makes `tasks_can_run` answer false until the returned value is dropped,
/// standing in for a system that refused a thread.
/// Why:
///  The seam of the control above;
/// the flag is cleared however the control ends,
/// a panic included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = refuseTasks(); // tasksCanRun() === false until refused.release()
/// ```
#[cfg(test)]
pub fn a_control_refuses_every_task() -> RefusalForTheControl {
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
    if let Ok(built) = RUNTIME.get_or_init(build_for_this_process)
        && matches!(built.flavor, BuiltFlavor::MultiThread { .. })
    {
        return Some(built.runtime.spawn(future));
    }
    // A stand-in runtime has no worker of its own, but a build that arrived after its deadline does: the work it was built
    // for runs there.
    if let Some(late) = LATE_RUNTIME.get() {
        return Some(late.spawn(future));
    }
    return None;
}

/// What:
///  Whether this process can run a task beside a blocked thread now:
/// its own runtime came up multi-threaded,
/// or the build that missed
///  its deadline has arrived since.
/// Why:
///   A caller that needs a worker asks once and gets one,
/// and a control of what needs a worker can tell a runtime the system refused
///  from one that was merely late.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ready = builtFlavor()?.kind === 'multi' || lateRuntime !== undefined;
/// ```
pub fn has_workers() -> bool {
    return match RUNTIME.get_or_init(build_for_this_process) {
        Ok(built) => matches!(built.flavor, BuiltFlavor::MultiThread { .. }) || LATE_RUNTIME.get().is_some(),
        Err(_) => LATE_RUNTIME.get().is_some(),
    };
}

/// Whether the build that missed its deadline has arrived;
/// a control of the late build reads it.
#[cfg(test)]
pub(crate) fn late_build_arrived() -> bool {
    return LATE_RUNTIME.get().is_some();
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
#[cfg(test)]
static THE_BUILD_WAS_LATE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Makes `built_flavor` answer the stand-in of a build that missed its deadline until the returned value is dropped.
#[cfg(test)]
pub fn a_control_stands_in_for_a_late_runtime() -> LateRuntimeForTheControl {
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
    let built: &BuiltRuntime = RUNTIME.get()?.as_ref().ok()?;
    return Some(built.flavor);
}

/// Runtime controls stay out of the release executable.
#[cfg(test)]
#[path = "lazy_runtime_tests.rs"]
mod tests;
