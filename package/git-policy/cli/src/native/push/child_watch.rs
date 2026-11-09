//! What:
//!  A watcher for a wait that blocks inside a read or a wait on a child process:
//!  it prints a notice after the grace and at every interval while the wait lasts,
//!  and ends the child when it has shown no sign of work for the quiet window.
//! Why:
//!  The wrapper reads Git objects from a long-lived `git cat-file --batch` and runs Git steps of a commit
//!  with their output captured.
//!  Both block in the operating system with nothing printed and no end,
//!  so a stalled child looks the same as a slow one and holds the commit for good (issue 1811).
//!  A watcher task,
//! run on the process's runtime beside the blocked thread,
//! sees the wait from outside:
//!  the waiting thread marks where a wait begins and ends,
//!  the watcher prints,
//! and when the child has shown no sign of work for the window it ends the child so the blocked call
//!  returns.
//!  Elapsed time never ends a child (owner,
//! 2026-11-03:
//! "Stop only on no progress";
//! pick 4 of
//!  `doc/planning/cli-git-shipped-limits-audit.md`,
//! the coordinator's under the owner's delegation and open to the owner's
//!  veto).
//! The signs of work are those the push reads (`push_progress.rs`):
//! bytes moved,
//! processor time,
//! a task able to run or
//!  waiting for the disk.
//! A child that is slow and works is waited for however long it takes.
//!  The window is not measured:
//! no log held a quiet gap before this change,
//! so the class numbers stay as labelled
//!  unmeasured windows (`child_class.rs`),
//! and the longest quiet gap of every watched step is kept for the timing records
//!  (`quiet_ledger.rs`) so a measured interval can replace them.
//!  A person's hook is never below a watched child the wrapper starts for itself:
//! every Git the wrapper starts carries `core.hooksPath` below `/dev/null` and the hook
//!  switches (`hooks_off.rs`),
//! so the stop signals the child and what
//!  it started,
//! never a process beside it.
//!  A push to a local path or a `file://` remote is the one place a hook of the person runs below a watched child:
//! the setting does not follow the push into the remote,
//! so its own `receive-pack` runs `pre-receive`,
//! `update` and `post-receive` in the push's tree,
//! and a stop that ends that tree ends those with it (`bounded_push.rs`,
//! `SPEC.md`,
//! "A local remote's hooks run in the push's tree").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = startWatch({ waitingFor, pid, limitMs, cadence, notices });
//! watch.begin(); try { await child.read(); } finally { watch.end(); }
//! if (watch.expired) throw new Error(`${waitingFor} showed no sign of work for ${limit} seconds`);
//! ```

/// The bound variable shared by every class of step,
/// and the classes themselves.
use crate::child_class::ChildClass;
/// The longest one step runs.
use crate::wait_ceilings::OBJECT_STEP_CEILING;
/// The captured output of a step that is never ended,
/// in files that outlive the command.
use crate::output_capture::Capture;
/// The pipes a step that is ended at its ceiling is read through,
/// whole,
/// on the runtime.
use crate::step_output_read_whole_through_pipes::{StepPipes, attach_pipes, release_write_ends};
/// Starts that ride out a short shortage of processes.
use crate::start_retry::RetryStart;
/// The longest quiet gap of each step,
/// kept for the timing records.
use crate::quiet_ledger;
/// The child and command types of the one-shot run.
use std::process::{Child, Command, Output};
/// The flag the watcher raises when a wait crossed the bound,
/// read by the waiting thread.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
/// The marks the waiting thread sends the watcher task;
/// the task owns every other piece of state.
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
/// The lock files a stopped step leaves.
use crate::stop_of_the_service_noticed_by_its_children::{StopArm, Unfinished, begin_unfinished_work};
use crate::stopped_git_locks::LockLedger;
/// The watcher task,
/// started on the process's runtime.
use crate::lazy_runtime::spawn_on_runtime;
use tokio::task::JoinHandle;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};
pub use crate::child_watch_clock_marks_and_stopping::{WatchTerms, grace_for};
use crate::child_watch_clock_marks_and_stopping::{Mark, WatchFlags};
use crate::child_watch_task_loop_and_notice::watch_loop;
pub use crate::child_watch_step_bounds_and_errors::{CeilingReached, EndedByStop, NotKnown, StepBound, ended_by_the_stop, is_ceiling, is_ended_by_stop, is_not_known, not_known_text, refused_while_stopping};
/// The sentence a quiet step is reported with,
/// kept beside the other wordings of a watch's ending.
pub use crate::child_watch_step_bounds_and_errors::watched_timeout_text;
/// The sentence for a step whose watcher could not exist,
/// kept beside the other wordings of a watch's ending.
pub use crate::child_watch_step_bounds_and_errors::no_watcher_text;
pub(crate) use crate::child_watch_step_bounds_and_errors::{Outlived, ending_of_a_watched_step};
use crate::child_watch_step_bounds_and_errors::{output_of};

/// The running watcher for a task that started,
/// or nothing when no thread could run a task.
fn running_watcher_of(spawned: Option<JoinHandle<()>>, finished: std::sync::mpsc::Receiver<()>) -> Option<RunningWatcher> {
    let handle: JoinHandle<()> = spawned?;
    return Some(RunningWatcher { handle, finished });
}

/// What:
///  The watcher of one child process.
/// Why:
///   One watch serves every wait on the same child:
///  a long-lived reader marks one wait per request,
/// a one-shot command marks one wait.
///  Dropping the watch ends its thread and hands the longest quiet gap it saw to the ledger,
/// if the watch was named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChildWatch implements Disposable { begin(); end(); get expired(): boolean }
/// ```
pub struct ChildWatch {
    /// Where the marks go;
    /// dropped first when the watch is dropped,
    /// which tells the watcher task to end.
    marks: Option<UnboundedSender<Mark>>,
    /// Whether a wait showed no sign of work for the window and the child was ended;
    /// raised by the watcher before it signals
    ///  the child.
    expired: Arc<AtomicBool>,
    /// The watcher task and the signal that it ended,
    /// waited for when the watch is dropped;
    /// nothing where no worker thread could
    ///  run a task,
    /// and the waits then run unwatched.
    task: Option<RunningWatcher>,
    /// The quiet window.
    limit: Duration,
    /// Whether a child that shows no sign of work for the window may be ended;
    /// cleared for a step that is not safe to run
    ///  twice,
    /// which is announced but only waited for.
    stoppable: Arc<AtomicBool>,
    /// The longest quiet gap of the waits so far,
    /// in microseconds,
    /// written by the watcher.
    longest: Arc<AtomicU64>,
    /// The step name the longest quiet gap is kept under in the ledger when the watch is dropped;
    /// nothing keeps no figure.
    ledger_step: Option<String>,
    /// How long one wait may last in all (shared with the watcher),
    /// in microseconds.
    ceiling: Arc<AtomicU64>,
    /// Whether the child was ended because the ceiling passed.
    ceiling_hit: Arc<AtomicBool>,
    /// Raised by the watcher task when it ended the child because the service was stopped.
    stopped: Arc<AtomicBool>,
    /// Why nothing watches this child (no worker thread of the runtime),
    /// in the words the person reads,
    /// or nothing when a task watches it.
    unwatched: Option<String>,
}

impl ChildWatch {
    /// Start watching `pid`,
    /// a child that holds no Git lock:
    /// see `start_with_locks`.
    pub fn start(waiting_for: &str, pid: u32, terms: WatchTerms) -> Self {
        return Self::start_with_locks(waiting_for, pid, terms, LockLedger::none());
    }

    /// Start watching `pid`:
    ///  waits marked with `begin` and `end` are announced as waiting for `waiting_for`,
    /// the child is ended when it shows no
    ///  sign of work for the terms' window,
    /// and the lock files the child leaves when it is ended are swept by `locks`.
    ///  A watch whose task cannot run (the runtime has no worker thread) watches nothing,
    /// and the waits it marks run unannounced and
    ///  unended.
    pub fn start_with_locks(waiting_for: &str, pid: u32, terms: WatchTerms, locks: LockLedger) -> Self {
        return Self::start_under_stop(waiting_for, pid, terms, locks, None);
    }

    /// What:
    ///  `start_with_locks`,
    /// and the child is ended when the service's stop is told through `stop`.
    /// Why:
    ///  A step the service waits for must not outlive the service (the owner's pick for open choice 12);
    /// the watcher already knows how to end a child and sweep the locks it leaves,
    /// so the stop is one more reason for it to do that.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ChildWatch.startUnderStop(waitingFor, pid, terms, locks, work.takeArm())
    /// ```
    pub fn start_under_stop(waiting_for: &str, pid: u32, terms: WatchTerms, locks: LockLedger, stop: Option<crate::stop_of_the_service_noticed_by_its_children::StopArm>) -> Self {
        return Self::start_when_a_task_may_run(waiting_for, pid, terms, locks, stop, crate::lazy_runtime::tasks_can_run());
    }

    /// What:
    ///  `start_under_stop` where the caller says whether a watcher task can run at all.
    /// Why:
    ///  A task needs a worker thread of its own;
    /// where the runtime has none (the current-thread stand-in,
    /// a build the system refused)
    ///  nothing watches the child,
    /// and the wait must not be unbounded because of it
    /// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
    /// finding 8).
    ///  The control hands in `false` instead of exhausting this process's threads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ChildWatch.startWhenATaskMayRun(waitingFor, pid, terms, locks, work.takeArm(), tasksCanRun())
    /// ```
    pub(crate) fn start_when_a_task_may_run(
        waiting_for: &str,
        pid: u32,
        terms: WatchTerms,
        locks: LockLedger,
        stop: Option<crate::stop_of_the_service_noticed_by_its_children::StopArm>,
        a_task_may_run: bool,
    ) -> Self {
        let (marks, incoming): (UnboundedSender<Mark>, UnboundedReceiver<Mark>) = unbounded_channel();
        let expired: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let stoppable: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let longest: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
        let ceiling: Arc<AtomicU64> = Arc::new(AtomicU64::new(u64::MAX));
        let ceiling_hit: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let stopped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let flags: WatchFlags = WatchFlags {
            expired: Arc::clone(&expired),
            stoppable: Arc::clone(&stoppable),
            longest: Arc::clone(&longest),
            ceiling: Arc::clone(&ceiling),
            ceiling_hit: Arc::clone(&ceiling_hit),
            stopped: Arc::clone(&stopped),
        };
        let subject: String = String::from(waiting_for);
        let limit: Duration = terms.limit;
        // The child is this process's own unreaped child at this moment, so its number is certainly its own: the pin taken here
        //  names that child for good, and every signal of the stop goes through it (the third review's finding 1).
        let root_pin: crate::child_watch_clock_marks_and_stopping::RootPin = crate::child_watch_clock_marks_and_stopping::pin_at_start(pid);
        let (ended, finished): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel();
        // The task holds `ended` and drops it however it ends (a panic or an abort included), which `finished` reads as a
        // disconnection.
        let spawned: Option<JoinHandle<()>> = if a_task_may_run {
            spawn_on_runtime(async move {
                let _ends_with_the_task: std::sync::mpsc::Sender<()> = ended;
                watch_loop(incoming, flags, subject, pid, terms, locks, stop, root_pin).await;
            })
        } else {
            None
        };
        let task: Option<RunningWatcher> = running_watcher_of(spawned, finished);
        // Nothing watches: the refusal is said once, here, because no task is left to say it, and the wait reads it from the
        // watch to carry the ceiling the task would have given it. The line goes to the diagnostics of this command, which is
        // where every other refusal of the wrapper goes: no blocking pool is entered because none may be reachable, and the
        // line is written once.
        let unwatched: Option<String> = match task.is_some() {
            true => None,
            false => {
                let refusal: String = no_watcher_text(waiting_for);
                crate::diagnostic_log::warn("childWatch", refusal.as_str());
                Some(refusal)
            }
        };
        return Self { marks: Some(marks), expired, task, limit, stoppable, longest, ledger_step: None, ceiling, ceiling_hit, stopped, unwatched };
    }

    /// Whether nothing watches this child,
    /// with the reason in the words the person reads.
    pub fn unwatched(&self) -> Option<&str> {
        return self.unwatched.as_deref();
    }

    /// Keep the longest quiet gap of this watch in the ledger under `step` when the watch is dropped.
    pub fn kept_as(mut self, step: &str) -> Self {
        self.ledger_step = Some(String::from(step));
        return self;
    }

    /// Announce the waits of this child but never end it for being quiet:
    /// a step that is not safe to run twice (a reference
    ///  update,
    /// a commit Git builds) is ended in the middle of its write only by the person.
    pub fn never_stop(&self) {
        self.stoppable.store(false, Ordering::Release);
    }

    /// Give every wait of this child a ceiling:
    /// the child is ended when one wait has lasted this long,
    /// whatever it shows and
    ///  whether or not a quiet child may be ended (owner,
    /// 2026-11-03:
    /// "nothing should ever wait forever or indefinitely").
    pub fn within(self, ceiling: Duration) -> Self {
        self.ceiling.store(u64::try_from(ceiling.as_micros()).unwrap_or(u64::MAX - 1), Ordering::Release);
        return self;
    }

    /// Whether the watcher ended the child because the service was stopped.
    pub fn ended_by_the_stop(&self) -> bool {
        return self.stopped.load(Ordering::Acquire);
    }

    /// Whether the child was ended because its ceiling passed.
    pub fn ceiling_hit(&self) -> bool {
        return self.ceiling_hit.load(Ordering::Acquire);
    }

    /// Send one mark;
    /// a watcher that is gone has nothing to be told.
    fn send(&self, mark: Mark) {
        if let Some(marks) = &self.marks {
            // A watcher whose thread did not start, or that panicked, dropped its receiver; the wait runs unwatched.
            let _sent = marks.send(mark);
        }
    }

    /// Mark the start of a wait on the child.
    pub fn begin(&self) {
        self.send(Mark::Begin(Instant::now()));
    }

    /// Mark the end of the wait.
    pub fn end(&self) {
        self.send(Mark::End);
    }

    /// Whether a wait showed no sign of work for the window and the child was ended.
    pub fn expired(&self) -> bool {
        return self.expired.load(Ordering::Acquire);
    }

    /// The quiet window.
    pub fn limit(&self) -> Duration {
        return self.limit;
    }
}

/// Stops the watcher task,
/// waits for it,
/// and keeps the longest quiet gap it saw.
impl Drop for ChildWatch {
    fn drop(&mut self) {
        drop(self.marks.take());
        if let Some(running) = self.task.take() {
            // The task ends as soon as it reads that the marks are gone (it checks at least every `STOP_POLL`); `finished` reports
            // that as a disconnection. A task that does not end by the bound is stopped, so that nothing waits for it for good.
            if let Err(std::sync::mpsc::RecvTimeoutError::Timeout) = running.finished.recv_timeout(WATCHER_END_BOUND) {
                running.handle.abort();
            }
        }
        self.keep_the_longest_gap();
    }
}

impl ChildWatch {
    /// What:
    ///  End the watch from a caller inside the runtime:
    /// the task is awaited,
    /// not waited for on this thread,
    /// and the longest quiet gap is kept as a drop keeps it.
    /// Why:
    ///   Dropping a watch waits for its task on the calling thread,
    /// which is the right wait for a synchronous caller and a wrong one for
    ///  a task:
    /// a worker that waits there for the bound of `WATCHER_END_BOUND` (nineteen seconds) is a worker no
    ///  other task of the process can use,
    /// and while the workers of a small runtime are all inside such a wait,
    /// the very task that
    ///  would end them cannot run,
    /// so every watch of a round pays the whole bound.
    /// Measured on 2026-11-08 (`foundation/parallel_runner_on_the_runtime_awaiting_tests.rs`,
    /// `an_item_may_await_a_git_child`):
    /// four awaited Git steps took 38.2 s with the drop and 10.8 ms with this call,
    /// and one awaited step alone took 19.1 s
    ///  with the drop,
    /// which is the whole of `WATCHER_END_BOUND`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await watch.close(); // the synchronous face's `drop` waits on the calling thread instead
    /// ```
    pub async fn close_awaiting(&mut self) {
        drop(self.marks.take());
        if let Some(running) = self.task.take() {
            // The task ends as soon as it reads that the marks are gone; a task that does not end by the bound is stopped, so that
            // nothing waits for it for good.
            let abort: tokio::task::AbortHandle = running.handle.abort_handle();
            if tokio::time::timeout(WATCHER_END_BOUND, running.handle).await.is_err() {
                abort.abort();
            }
        }
        self.keep_the_longest_gap();
    }

    /// What:
    ///  Keep the longest quiet gap of the waits this watch saw for the timing records,
    /// once.
    /// Why:
    ///   Both endings of a watch (its drop and `close_awaiting`) hand the figure to the ledger,
    /// and only one of them ever runs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (ledgerStep) note(ledgerStep, longest, expired);
    /// ```
    fn keep_the_longest_gap(&mut self) {
        if let Some(step) = self.ledger_step.take() {
            let gap: Duration = Duration::from_micros(self.longest.load(Ordering::Acquire));
            quiet_ledger::note(step.as_str(), gap, self.expired());
        }
    }
}

// What:
//  A control that refuses the thread that waits for a step left running,
//  as a process at its thread limit does with `EAGAIN` from `pthread_create`.
// Why:
//   The refusal cannot be produced on demand here:
//   a thread limit low enough to refuse this thread refuses the control's own threads too,
//   so the system's answer is injected
//   (`native_commit_waiting_notice.rs` injects the same refusal the same way).
//  The thread is started by the thread that runs the step, so the control is of that thread.
#[cfg(test)]
thread_local! {
    /// How many waiter threads the next starts are refused.
    static WAITER_THREAD_REFUSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Refuse the next `count` threads that would wait for a step left running.
#[cfg(test)]
pub(crate) fn refuse_the_next_left_running_waiter(count: usize) {
    WAITER_THREAD_REFUSALS.with(|left| { left.set(count); });
}

/// What:
///  The refusal a control armed,
///  standing in for a system that cannot make another thread.
/// Why:
///  The start of the thread that waits for a step left running answers a refused thread the way it answers any other,
/// and only a control tells the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusedNow = () => armedRefusal-- > 0 ? new Error('refused') : undefined;
/// ```
fn a_waiter_thread_is_refused_now() -> Option<std::io::Error> {
    #[cfg(test)]
    if WAITER_THREAD_REFUSALS.with(|left| {
        let armed: usize = left.get();
        left.set(armed.saturating_sub(1));
        return armed > 0;
    }) {
        return Some(std::io::Error::other("a control refused the thread that waits for a step left running"));
    }
    return None;
}

/// How long dropping a watch waits for its task to end:
/// the bound of "nothing should ever wait forever" on a wait that is over in
///  a few hundred microseconds.
/// Measured on 2026-11-07 (`push/child_watch_drop_time_measurement_tests.rs`,
/// three runs by hand under
///  a load average of 15 to 40):
/// dropping a watch between waits took at most 505 us and in the middle of a wait at most 389 us,
/// the
///  lock sweep alone at most 569 us,
/// and a whole stop of a child that answers the request at most 41680 us past the window.
/// A task
///  can legitimately outlast those only inside a stop that waits for a member in uninterruptible sleep,
/// which is at most
///  `STOP_CEILING` after the request (the owner's "3x worst case" of the longest delay measured;
/// a descendant that ignores the request
///  is killed when that wait ends),
/// so the bound is that ceiling plus `STOP_TAIL_ALLOWANCE`:
/// aborting earlier would cancel the kill of a
///  stubborn descendant that holds no pipe.
const WATCHER_END_BOUND: Duration = crate::stop_waits_while_a_member_cannot_be_interrupted::STOP_CEILING.saturating_add(STOP_TAIL_ALLOWANCE);

/// What a stop needs beyond its grace:
/// the kill,
/// the look at the processes,
/// the lock sweep and the end of the task.
/// One second is
///  twenty-four times the longest tail measured (41680 us),
/// see `WATCHER_END_BOUND`.
const STOP_TAIL_ALLOWANCE: Duration = Duration::from_secs(1);

/// A watcher task and the signal that it has ended.
struct RunningWatcher {
    /// The task,
    /// aborted when it does not end by `WATCHER_END_BOUND`.
    handle: JoinHandle<()>,
    /// Disconnects when the task ends.
    finished: std::sync::mpsc::Receiver<()>,
}

/// What:
///  How the object reader's wait names itself in notices and in the timeout sentence.
/// Why:
///   The reader is one long-lived `cat-file --batch` whose single request can name a large blob,
/// so it runs under the
///  bulk class on purpose (a `cat-file` that answers one query through `run_git` is a quick query step),
/// and its
///  subject says so like every other step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `git cat-file --batch, ${labelOf(BULK)}`
/// ```
pub fn reader_subject() -> String {
    return format!("git cat-file --batch, {}", ChildClass::Object.label());
}

/// What:
///  Run a command to its end and collect its output,
/// announcing the wait and ending the child when it shows no sign of work
///  for the window.
///  The command's input must be set by the caller;
///  its output streams are replaced by pipes.
/// Why:
///   `Command::output` blocks until the child ends and prints nothing meanwhile.
///  A Git step that stalls under load or behind a filter must say so,
/// and must not hold a commit for good;
/// a step that is slow
///  and works is waited for however long it takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, terms }); // throws on a stall
/// ```
pub fn output_watched(command: &mut Command, waiting_for: &str, terms: WatchTerms) -> std::io::Result<Output> {
    return output_watched_stopping(command, waiting_for, waiting_for, terms, true);
}

/// What:
///  `output_watched`,
/// where `stoppable` says whether a child that shows no sign of work for the window is ended,
/// and
///  `ledger_step` names the step under which its longest quiet gap is kept.
/// Why:
///   A step that is not safe to run twice is announced at the usual interval and waited for as long as it lives;
/// the caller
///  starts a stoppable step again when this returns the `TimedOut` error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, ledgerStep, terms, stoppable }); // throws on a stall
/// ```
pub fn output_watched_stopping(
    command: &mut Command,
    waiting_for: &str,
    ledger_step: &str,
    terms: WatchTerms,
    stoppable: bool,
) -> std::io::Result<Output> {
    return output_watched_within(command, waiting_for, ledger_step, terms, stoppable, StepBound::EndAt(OBJECT_STEP_CEILING));
}

/// What:
///  `output_watched_stopping` where the step is ended at `ceiling` whatever it shows,
/// or never ended for its length when there is
///  none.
/// Why:
///   The owner's rule is that nothing waits for ever;
/// the ceiling is what ends a step that keeps working and one that may not be
///  ended for being quiet.
/// A step with no ceiling is a write in place that is not safe to run twice (owner,
/// 2026-11-04:
/// "Never cut
///  a branch write (Recommended)";
/// `transaction_git.rs`,
/// `step_bound`):
/// ending it in the middle of its write could cost the
///  person's data,
/// so only the person ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, ledgerStep, terms, stoppable, ceilingMs }); // throws on a stall or the ceiling
/// ```
pub fn output_watched_within(
    command: &mut Command,
    waiting_for: &str,
    ledger_step: &str,
    terms: WatchTerms,
    stoppable: bool,
    bound: StepBound,
) -> std::io::Result<Output> {
    return output_watched_within_for_a_stop(command, waiting_for, ledger_step, terms, stoppable, bound, false);
}

/// What:
///  `output_watched_within` where the service's stop may be told to let the step finish.
/// Why:
///   The owner's pick of 2026-11-08 ("B:
/// let a branch write finish (Recommended)"):
/// a stop does not end the branch write.
/// Its count stays,
/// so the shutdown waits for it up to its own bound and the service ends after it;
/// what is still running then is left to the manager's final kill (`KillMode=mixed` in the service unit),
/// and the batch's open intent and the commit on its scratch branch are what recovery reads.
/// Every other step is ended by the stop as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, ledgerStep, terms, stoppable, bound, aStopWaits });
/// ```
pub fn output_watched_within_for_a_stop(
    command: &mut Command,
    waiting_for: &str,
    ledger_step: &str,
    terms: WatchTerms,
    stoppable: bool,
    bound: StepBound,
    a_stop_lets_it_finish: bool,
) -> std::io::Result<Output> {
    let ceiling: Option<Duration> = match bound {
        StepBound::EndAt(limit) => Some(limit),
        StepBound::LeaveRunning { .. } | StepBound::Unbounded => None,
    };
    // A step that is never ended writes into files that outlive this command, not into pipes this command holds: when the command
    // stops waiting and exits, the step must find its output open, or it dies of SIGPIPE at its next write, which is a cut. A step
    // that is ended at its ceiling is read to its end by this command through pipes on the runtime, every byte of it; the pipes are
    // made before the child starts, so a refusal starts nothing.
    let (captured, pipes): (Option<Capture>, Option<StepPipes>) = match bound {
        StepBound::EndAt(_) => (None, Some(attach_pipes(command)?)),
        StepBound::LeaveRunning { .. } | StepBound::Unbounded => (Some(Capture::new()?), None),
    };
    if let Some(files) = captured.as_ref() {
        let (out, err) = files.stdio()?;
        command.stdout(out).stderr(err);
    }
    // The service's shutdown waits for this step and ends it when the service is stopped. A landing that is already unwinding after a stop
    //  may start the steps that undo what the stop cut (the side ref that keeps its commit); every other step begun after the stop is refused.
    let mut unfinished: Unfinished = match begin_unfinished_work(crate::service_busy_step_record_on_this_thread::on_landing_thread(), waiting_for) {
        Ok(counted) => counted,
        Err(refusal) => return Err(refused_while_stopping(&refusal)),
    };
    // Taken before the child starts, so a lock born earlier is never taken for the child's.
    let locks: LockLedger = LockLedger::before_start(command);
    let spawned: std::io::Result<Child> = command.spawn_retrying();
    // The command holds the write ends of the pipes until they are replaced, and the readers see no end of output while it does.
    if pipes.is_some() {
        release_write_ends(command);
    }
    let child: Child = spawned?;
    // A step that a stop lets finish keeps its count and takes no arm, so nothing ends it when the stop is told.
    let arm: Option<StopArm> = if a_stop_lets_it_finish { None } else { unfinished.take_arm() };
    let started: ChildWatch = ChildWatch::start_under_stop(waiting_for, child.id(), terms, locks, arm).kept_as(ledger_step);
    // A step with no ceiling is waited for as long as it lives: the watch starts without one.
    let watch: ChildWatch = match ceiling {
        Some(limit) => started.within(limit),
        None => started,
    };
    if !stoppable {
        watch.never_stop();
    }
    // Whatever the watch could not do, the wait for the output carries the step's own ceiling: a watch that cannot run, and one
    // that may announce a wait but must not end it, both leave this wait as the only thing that ends the step at its ceiling
    // (the third review's finding 2). A step with no ceiling is waited for as before.
    let carried: Option<crate::step_output_read_whole_through_pipes::CarriedCeiling> =
        crate::step_output_read_whole_through_pipes::carried_for(ceiling, watch.unwatched().is_none());
    watch.begin();
    let output: std::io::Result<Output> = match bound {
        StepBound::LeaveRunning { after, extra } => {
            // The child moves to a thread that waits for it and collects its output, so the command can stop waiting and the child
            // goes on; the thread reaps it when it ends.
            let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::ExitStatus>>();
            // The child is handed over through a channel rather than moved into the closure, so a refused start leaves it here
            //  (`a_waiter_thread_is_refused_now`, and a system at its thread limit below).
            let (hand_over, taken) = std::sync::mpsc::channel::<Child>();
            let waiter = match a_waiter_thread_is_refused_now() {
                Some(refusal) => {
                    // Nothing was given the receiver, so it is dropped here: the channel closes, and the child comes back to this
                    //  command below to be reaped rather than left in a queue nobody reads.
                    drop(taken);
                    Err(refusal)
                }
                None => std::thread::Builder::new().name(String::from("cli-git-left-running-step")).spawn(move || {
                    // A refused start drops this receiver with the closure, and the child is handed back to the caller below.
                    let Ok(mut leaving) = taken.recv() else { return };
                    // The command may have stopped waiting; a closed channel is that case, and there is nobody to tell.
                    let _abandoned = sender.send(leaving.wait());
                }),
            };
            // A child dropped with a refused start would run on unreaped and stay a zombie of this process once it ended, so it is
            //  kept by whoever can still collect it: the thread above, or a reaper of its own where no thread could start.
            if let Err(std::sync::mpsc::SendError(unwaited)) = hand_over.send(child) {
                crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon(unwaited);
            }
            if let Err(error) = waiter {
                watch.end();
                return Err(std::io::Error::other(format!("{waiting_for} was started but no thread could be started to wait for it: {error}")));
            }
            match receiver.recv_timeout(after.saturating_add(extra)) {
                Ok(finished) => match finished {
                    Ok(status) => Ok(output_of(status, captured.as_ref())),
                    Err(error) => Err(error),
                },
                Err(_timeout_or_gone) => {
                    watch.end();
                    // Nobody will read what the step writes from here on, and it is held in memory until the step ends.
                    if let Some(files) = captured.as_ref() {
                        files.discard();
                    }
                    return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, NotKnown(not_known_text(waiting_for, after, extra))));
                }
            }
        }
        StepBound::EndAt(_) => match pipes {
            Some(open) => open.read_until_the_child_ends(child, waiting_for, carried),
            None => Err(std::io::Error::other("a step that is ended at its ceiling had no pipes to read: a defect of monochromatic-git-policy-cli")),
        },
        StepBound::Unbounded => {
            let mut waited_for: Child = child;
            match waited_for.wait() {
                Ok(status) => Ok(output_of(status, captured.as_ref())),
                Err(error) => Err(error),
            }
        }
    };
    watch.end();
    // The stop ended the step: the caller is told so, whatever the step's own end looked like.
    let outlived: Outlived = Outlived {
        ended_by_the_stop: watch.ended_by_the_stop(),
        ceiling,
        ceiling_hit: watch.ceiling_hit(),
        expired: watch.expired(),
        limit: watch.limit(),
    };
    return match ending_of_a_watched_step(waiting_for, outlived) {
        Some(error) => Err(error),
        None => output,
    };
}

/// Watch controls stay out of the release executable.
#[cfg(test)]
#[path = "child_watch_tests.rs"]
mod tests;

/// One-shot run controls stay out of the release executable.
#[cfg(test)]
#[path = "child_watch_one_shot_through_pipes_tests.rs"]
mod one_shot_through_pipes_tests;

/// The measurement of how long dropping a watch takes,
/// kept out of the release executable.
#[cfg(test)]
#[path = "child_watch_drop_time_measurement_tests.rs"]
mod drop_time_measurement_tests;
