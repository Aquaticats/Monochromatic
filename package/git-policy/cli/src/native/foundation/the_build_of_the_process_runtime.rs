//! What:
//!  The build of the process's multi-thread runtime:
//!  the deadline it is given,
//!  the arena cap set before its first worker starts,
//!  the helper thread that builds it,
//!  the hand-over that keeps a runtime the caller was too late for,
//!  and the pause that makes a control's build outlast its deadline.
//! Why:
//!  `lazy_runtime.rs` asks for one runtime in a process's life and never drops it;
//!  the build is here,
//! so the file that answers questions about a runtime carries neither the thread
//!  nor the pauses a build needs,
//!  and a reader of either file reads one job.
//!  A build takes milliseconds where it works and can hang where the system refuses a thread,
//! so the caller stops waiting at `BUILD_DEADLINE`,
//!  and a runtime that arrives after that moment is kept here for the work it was built for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const built = await Promise.race([buildMulti(workers, blocking), sleep(deadline).then(() => 'late')]);
//! if (built === 'late') keepItForLater(built);
//! ```

/// The sizes the plan gives the multi-thread runtime.
use crate::runtime_plan::RuntimePlan;
/// How a built runtime came out,
/// and the runtime itself.
use crate::what_a_built_runtime_is::{
    BuiltFlavor,
    BuiltRuntime,
};
/// The clock the deadline is measured on.
use std::time::Duration;
/// The runtime the helper thread builds.
use tokio::runtime::Runtime;

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
pub(crate) fn cap_malloc_arenas() {
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
    return tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
}

/// The variable that makes the build of the multi-thread runtime wait before it starts its workers,
/// so a control can produce a build that outlasts `BUILD_DEADLINE`:
/// a whole number of milliseconds.
/// Why:
///  A real build takes milliseconds even under load,
/// so no control could reach the one state the gate of 2026-11-11T10:29-05:00 reached on a machine slow to give a process its threads;
///  the pause stands in for that machine.
/// It is compiled into the release executable and inert unless the variable names a pause,
///  as `CLI_GIT_TEST_ONLY_SERVICE_FAULT` is.
/// The read is compiled out of any build the release ships,
/// so only a build made for a control can name a pause.
pub const SLOW_BUILD_VARIABLE: &str = "CLI_GIT_TEST_ONLY_RUNTIME_BUILD_MILLIS";

/// The pause `SLOW_BUILD_VARIABLE` names,
/// or none.
/// A control asks this of a build made without the switches,
/// where it answers no pause.
pub(crate) fn the_pause_a_control_asks_for() -> Duration {
    // The milliseconds a control named, or nothing in every shipped build.
    let named: Option<u64> =
        crate::test_only_switches::text(SLOW_BUILD_VARIABLE).and_then(milliseconds_of);
    return named.map_or(Duration::ZERO, Duration::from_millis);
}

/// The whole number of milliseconds `text` names,
/// or nothing when it names none.
fn milliseconds_of(text: String) -> Option<u64> {
    return text.trim().parse::<u64>().ok();
}

/// The multi-thread runtime with `workers` workers and at most `blocking` blocking-pool threads.
pub(crate) fn multi_thread(
    workers: usize,
    blocking: usize,
) -> std::io::Result<Runtime> {
    let pause: Duration = the_pause_a_control_asks_for();
    if !pause.is_zero() {
        std::thread::sleep(pause);
    }
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
pub(crate) fn error_text(error: std::io::Error) -> String {
    return error.to_string();
}

/// What:
///  The one moment in which the build of a runtime hands it over and the caller of that build stops waiting for it.
/// Why:
///   `std::sync::mpsc` cannot say whether a value sent a moment before the caller gives up has been read:
/// a send that lands between the caller's last read and its closing of the channel succeeds into a channel that is then dropped,
///  and the runtime goes with it.
/// A runtime lost that way left the process for ever believing a build was still coming,
///  so every later task spent the whole patience on it,
///  and the service ended with status 4 (the review of 2026-11-12T09:34-05:00 found this one too).
/// Held by both,
/// the send happens either before the caller's read,
/// which then takes the runtime,
/// or after the channel is closed,
/// which makes the build keep it;
/// the lock covers that pair of statements and nothing else,
/// and one runtime is built in a process's life,
/// so nothing else waits on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handOff = new Mutex(); // held around the build's send, and around the caller's last read and close
/// ```
static THE_HANDOFF_OF_A_LATE_RUNTIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// What:
///  A control's own code at the moment between the caller's last read of the channel and its closing of it,
///  which is the moment the hand-over above closes.
/// Why:
///   The two statements cannot be ordered from outside,
/// so the control that shows a runtime sent into them is kept releases the build from here;
///  the slot is empty in every process that sets no control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let theCallersMoment: (() => void) | undefined; // run once between the caller's last read and its close
/// ```
#[cfg(test)]
pub(crate) static THE_CALLER_STOPS_WAITING: std::sync::Mutex<Option<Box<dyn Fn() + Send>>> =
    std::sync::Mutex::new(None);

/// What:
///  A control's own code at the build's moment between a hand-over the caller was too late for and the keeping of the runtime it carries,
///  which is the moment the hand-over above hands the keep to the build.
/// Why:
///   The keep happens on the build's own thread after the caller has already returned,
/// so nothing outside can order it with the answer the caller got;
///  a control that holds the build here sees what that answer says without the keep and then lets it happen.
///  The slot is empty in every process that sets no control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let theBuildsMomentBeforeKeeping: (() => void) | undefined; // run once between a failed send and the keep
/// ```
#[cfg(test)]
pub(crate) static THE_BUILD_STOPS_BEFORE_KEEPING: std::sync::Mutex<Option<Box<dyn Fn() + Send>>> =
    std::sync::Mutex::new(None);

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
pub fn build_with_deadline<B>(
    plan: RuntimePlan,
    deadline: Duration,
    multi_thread_builder: B,
) -> std::io::Result<BuiltRuntime>
where
    B: FnOnce(usize, usize) -> std::io::Result<Runtime> + Send + 'static,
{
    return build_with_deadline_and(plan, deadline, multi_thread_builder, current_thread);
}

/// What:
///  `build_with_deadline` with the stand-in given as `fallback`.
/// Why:
///   A control orders a build that finishes after the deadline by holding the caller in the caller's own stand-in,
/// and only this parameter lets it hold the caller there;
/// the process builds its own runtime through `build_with_deadline`,
///  whose stand-in is the current-thread runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const built = buildWithDeadlineAnd(plan, deadline, multiThread, fallback);
/// ```
pub(crate) fn build_with_deadline_and<B, F>(
    plan: RuntimePlan,
    deadline: Duration,
    multi_thread_builder: B,
    fallback: F,
) -> std::io::Result<BuiltRuntime>
where
    B: FnOnce(usize, usize) -> std::io::Result<Runtime> + Send + 'static,
    F: FnOnce() -> std::io::Result<Runtime>,
{
    let RuntimePlan::MultiThread {
        workers,
        blocking_threads,
    } = plan
    else {
        return Ok(BuiltRuntime {
            runtime: current_thread()?,
            flavor: BuiltFlavor::CurrentThread,
        });
    };
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<Runtime>>();
    let started = std::thread::Builder::new()
        .name(String::from("git-policy-runtime-build"))
        .spawn(move || {
            // A refused thread panics inside tokio's build; the panic is caught here and reported as a replaced runtime.
            let outcome: std::thread::Result<std::io::Result<Runtime>> =
                crate::panic_notice::while_panics_are_expected(
                    (multi_thread_builder, workers, blocking_threads),
                    run_multi_thread_builder::<B>,
                );
            let result: std::io::Result<Runtime> = match outcome {
                Ok(built) => built,
                Err(_payload) => Err(std::io::Error::other(
                    "the build of the multi-thread runtime panicked",
                )),
            };
            // The caller reads the channel once more and closes it under this same moment, so a runtime sent just before it gives up is
            //  taken there rather than dropped with the channel.
            let _handing_over: std::sync::MutexGuard<'_, ()> = THE_HANDOFF_OF_A_LATE_RUNTIME
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Err(unsent) = sender.send(result)
                && let Ok(late) = unsent.0
            {
                // The caller moved on: a working runtime that arrived after the deadline is kept for the work it was built for
                //  (`spawn_on_runtime`), and one that cannot be kept is never dropped here (dropping waits for its tasks
                //  without end).
                #[cfg(test)]
                if let Some(moment) = THE_BUILD_STOPS_BEFORE_KEEPING
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                {
                    moment();
                }
                if let Err(unkept) = LATE_RUNTIME.set(late) {
                    std::mem::forget(unkept);
                }
            }
        });
    let flavor_when_replaced: BuiltFlavor = match started {
        Err(_no_thread) => BuiltFlavor::CurrentThreadAfterFailure,
        Ok(_helper) => match receiver.recv_timeout(deadline) {
            Ok(Ok(runtime)) => {
                return Ok(BuiltRuntime {
                    runtime,
                    flavor: BuiltFlavor::MultiThread {
                        workers,
                    },
                });
            }
            Ok(Err(_refused)) => BuiltFlavor::CurrentThreadAfterFailure,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // A build that finished between the deadline and here is already in the channel, and one that finishes
                //  after the channel is closed is kept by its own helper: measure the two, so neither is dropped.
                // The read and the close are one moment with the build's send, so a runtime sent as the caller gives up is
                //  taken here rather than dropped with the channel.
                let _handing_over: std::sync::MutexGuard<'_, ()> = THE_HANDOFF_OF_A_LATE_RUNTIME
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let _kept: bool = keep_a_late_build(receiver.try_recv());
                #[cfg(test)]
                if let Some(moment) = THE_CALLER_STOPS_WAITING
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                {
                    moment();
                }
                drop(receiver);
                BuiltFlavor::CurrentThreadAfterDeadline
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                BuiltFlavor::CurrentThreadAfterFailure
            }
        },
    };
    return Ok(BuiltRuntime {
        runtime: fallback()?,
        flavor: flavor_when_replaced,
    });
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
fn keep_a_late_build(
    outcome: Result<std::io::Result<Runtime>, std::sync::mpsc::TryRecvError>
) -> bool {
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
pub(crate) static LATE_RUNTIME: std::sync::OnceLock<Runtime> = std::sync::OnceLock::new();
