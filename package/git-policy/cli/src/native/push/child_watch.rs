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

/// The longest quiet gap of each step,
/// kept for the timing records.
use crate::quiet_ledger;
/// The flag the watcher raises when a wait crossed the bound,
/// read by the waiting thread.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
/// The marks the waiting thread sends the watcher task;
/// the task owns every other piece of state.
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
/// The lock files a stopped step leaves.
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
/// The one-shot run of a watched step,
/// which is where a caller starts a command under a watch.
pub use crate::child_watch_one_shot_run::{output_watched, output_watched_stopping, output_watched_within, output_watched_within_for_a_stop, reader_subject};
/// The sentence a quiet step is reported with,
/// kept beside the other wordings of a watch's ending.
pub use crate::child_watch_step_bounds_and_errors::watched_timeout_text;
/// The sentence for a step whose watcher could not exist,
/// kept beside the other wordings of a watch's ending.
pub use crate::child_watch_step_bounds_and_errors::no_watcher_text;
use crate::child_watch_running_task::{RunningWatcher, WATCHER_END_BOUND, running_watcher_of};
/// The refusal a control arms,
/// which only a control build can reach.
#[cfg(test)]
pub(crate) use crate::child_watch_running_task::refuse_the_next_left_running_waiter;


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
    /// How long the command waits for a step it never ends before it stops waiting and leaves it running (shared with the
    /// watcher),
    /// in microseconds,
    /// or zero when it waits for as long as the step lives.
    left_running_after: Arc<AtomicU64>,
    /// Whether the child was ended because the ceiling passed.
    ceiling_hit: Arc<AtomicBool>,
    /// Raised by the watcher task when it ended the child because the service was stopped.
    stopped: Arc<AtomicBool>,
    /// Why nothing watches this child (no worker thread of the runtime),
    /// in the words the person reads,
    /// or nothing when a task watches it.
    unwatched: Option<String>,
    /// Whether the pin taken when the child started proves which process to stop,
    /// which decides whether this watch can end the step at its ceiling at all.
    could_end_the_step: bool,
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
        let left_running_after: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
        let ceiling_hit: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let stopped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let flags: WatchFlags = WatchFlags {
            expired: Arc::clone(&expired),
            stoppable: Arc::clone(&stoppable),
            longest: Arc::clone(&longest),
            ceiling: Arc::clone(&ceiling),
            ceiling_hit: Arc::clone(&ceiling_hit),
            stopped: Arc::clone(&stopped),
            left_running_after: Arc::clone(&left_running_after),
        };
        let subject: String = String::from(waiting_for);
        let limit: Duration = terms.limit;
        // The child is this process's own unreaped child at this moment, so its number is certainly its own: the pin taken here
        //  names that child for good, and every signal of the stop goes through it (the third review's finding 1).
        let root_pin: crate::child_watch_clock_marks_and_stopping::RootPin = crate::child_watch_clock_marks_and_stopping::pin_at_start(pid);
        // Whether that pin proves the process decides, later, whether this watch can end the step it watches at all.
        let could_end_the_step: bool = crate::child_watch_clock_marks_and_stopping::the_root_is_proven(&root_pin);
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
        return Self { marks: Some(marks), expired, task, limit, stoppable, longest, ledger_step: None, ceiling, left_running_after, ceiling_hit, stopped, unwatched, could_end_the_step };
    }

    /// Whether nothing watches this child,
    /// with the reason in the words the person reads.
    pub fn unwatched(&self) -> Option<&str> {
        return self.unwatched.as_deref();
    }

    /// What:
    ///  Whether a watch runs for this child,
    /// and whether it can end the child at its ceiling.
    /// Why:
    ///   The wait for a step's output keeps the step's ceiling as a backstop of its own,
    /// so how much later than the watch it
    ///  may fire depends on what the watch can do:
    /// a watch whose pin proves the process ends the step itself and is given `THE_WATCH_HAS_ITS_ENDING`
    ///  to do it,
    /// while one that can prove nothing ends nothing and leaves the ceiling to that wait.
    pub fn what_watches_the_step(&self) -> crate::step_output_read_whole_through_pipes::TheWatchOfAStep {
        return match self.unwatched {
            Some(_) => crate::step_output_read_whole_through_pipes::TheWatchOfAStep::NotWatched,
            None if self.could_end_the_step => crate::step_output_read_whole_through_pipes::TheWatchOfAStep::WatchedAndProven,
            None => crate::step_output_read_whole_through_pipes::TheWatchOfAStep::WatchedAndUnproven,
        };
    }

    /// Whether the pin this watch took proves which process to stop,
    /// for the ending a caller words after the wait.
    pub fn could_end_the_step(&self) -> bool {
        return self.could_end_the_step;
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

    /// Say the bound a notice of this child names:
    /// the command waits `after` for a step it never ends and then leaves it running,
    /// with the result not known.
    /// Why:
    ///   A step the wrapper never ends is not ended for being slow,
    /// but nothing waits without a bound either:
    ///  a notice that said such a step is waited for as long as it lives said so of a write whose wait ended at 127
    ///  seconds (2026-11-11,
    ///  the release gate of 0.6.1-alpha.11).
    ///  The watcher says what the waiting thread does,
    /// and it is told here.
    pub fn stops_waiting_after(&self, after: Duration) {
        self.left_running_after.store(u64::try_from(after.as_micros()).unwrap_or(u64::MAX - 1), Ordering::Release);
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
