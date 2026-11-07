//! What:
//!  A watcher for a wait that blocks inside a read or a wait on a child process:
//!  it prints a notice after the grace and at every interval while the wait lasts,
//!  and ends the child when it has shown no sign of work for the quiet window.
//! Why:
//!  The wrapper reads Git objects from a long-lived `git cat-file --batch` and runs Git steps of a commit
//!  with their output captured.
//!  Both block in the operating system with nothing printed and no end,
//!  so a stalled child looks the same as a slow one and holds the commit for good (issue 1811).
//!  A watcher task, run on the process's runtime beside the blocked thread, sees the wait from outside:
//!  the waiting thread marks where a wait begins and ends,
//!  the watcher prints, and when the child has shown no sign of work for the window it ends the child so the blocked call
//!  returns.
//!  Elapsed time never ends a child (owner, 2026-11-03: "Stop only on no progress"; pick 4 of
//!  `doc/planning/cli-git-shipped-limits-audit.md`, the coordinator's under the owner's delegation and open to the owner's
//!  veto). The signs of work are those the push reads (`push_progress.rs`): bytes moved, processor time, a task able to run or
//!  waiting for the disk. A child that is slow and works is waited for however long it takes.
//!  The window is not measured: no log held a quiet gap before this change, so the class numbers stay as labelled
//!  unmeasured windows (`child_class.rs`), and the longest quiet gap of every watched step is kept for the timing records
//!  (`quiet_ledger.rs`) so a measured interval can replace them.
//!  A person's hook is never ended: every Git the wrapper starts carries `core.hooksPath` below `/dev/null` and the hook
//!  switches (`hooks_off.rs`), so no hook of the person is ever below a watched child, and the stop signals the child and what
//!  it started, never a process beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = startWatch({ waitingFor, pid, limitMs, cadence, notices });
//! watch.begin(); try { await child.read(); } finally { watch.end(); }
//! if (watch.expired) throw new Error(`${waitingFor} showed no sign of work for ${limit} seconds`);
//! ```

/// The bound variable shared by every class of step, and the classes themselves.
use crate::child_class::{CHILD_WAIT_VARIABLE, ChildClass};
/// The longest one step runs.
use crate::wait_ceilings::OBJECT_STEP_CEILING;
/// The captured output of a step that is never ended, in files that outlive the command.
use crate::output_capture::Capture;
/// Starts that ride out a short shortage of processes.
use crate::start_retry::RetryStart;
/// The notice cadence, its ticker and the child-wait line.
use crate::wait_progress::{ChildWaitFacts, Ticker, child_wait_line};
/// The patience of a run that ends on a stall, and the stop it reports.
use crate::bounded_push::StopCause;
/// The clock that reads the signs of work of a process tree, and how often it reads.
use crate::push_progress::{Clock, SAMPLE_INTERVAL};
/// The longest quiet gap of each step, kept for the timing records.
use crate::quiet_ledger;
/// The child and command types of the one-shot run.
use std::process::{Child, Command, Output, Stdio};
/// The sink a notice goes to.
use std::io::Write;
/// The flag the watcher raises when a wait crossed the bound, read by the waiting thread.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
/// The marks the waiting thread sends the watcher task; the task owns every other piece of state.
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
/// The lock files a stopped step leaves.
use crate::stopped_git_locks::LockLedger;
/// The watcher task, started on the process's runtime.
use crate::lazy_runtime::spawn_on_runtime;
use tokio::task::JoinHandle;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};
pub use crate::child_watch_clock_marks_and_stopping::{WatchTerms, grace_for};
use crate::child_watch_clock_marks_and_stopping::{Mark, Marks, SHORTEST_SLEEP, WatchFlags, stop_process};
pub use crate::child_watch_step_bounds_and_errors::{CeilingReached, NotKnown, StepBound, is_ceiling, is_not_known, not_known_text};
use crate::child_watch_step_bounds_and_errors::{output_of};

/// What:
///  The watcher of one child process.
/// Why:
///   One watch serves every wait on the same child:
///  a long-lived reader marks one wait per request, a one-shot command marks one wait.
///  Dropping the watch ends its thread and hands the longest quiet gap it saw to the ledger, if the watch was named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChildWatch implements Disposable { begin(); end(); get expired(): boolean }
/// ```
pub struct ChildWatch {
    /// Where the marks go; dropped first when the watch is dropped, which tells the watcher task to end.
    marks: Option<UnboundedSender<Mark>>,
    /// Whether a wait showed no sign of work for the window and the child was ended; raised by the watcher before it signals
    ///  the child.
    expired: Arc<AtomicBool>,
    /// The watcher task and the signal that it ended, waited for when the watch is dropped; nothing where no worker thread could
    ///  run a task, and the waits then run unwatched.
    task: Option<RunningWatcher>,
    /// The quiet window.
    limit: Duration,
    /// Whether a child that shows no sign of work for the window may be ended; cleared for a step that is not safe to run
    ///  twice, which is announced but only waited for.
    stoppable: Arc<AtomicBool>,
    /// The longest quiet gap of the waits so far, in microseconds, written by the watcher.
    longest: Arc<AtomicU64>,
    /// The step name the longest quiet gap is kept under in the ledger when the watch is dropped; nothing keeps no figure.
    ledger_step: Option<String>,
    /// How long one wait may last in all (shared with the watcher), in microseconds.
    ceiling: Arc<AtomicU64>,
    /// Whether the child was ended because the ceiling passed.
    ceiling_hit: Arc<AtomicBool>,
}

impl ChildWatch {
    /// Start watching `pid`, a child that holds no Git lock: see `start_with_locks`.
    pub fn start(waiting_for: &str, pid: u32, terms: WatchTerms) -> Self {
        return Self::start_with_locks(waiting_for, pid, terms, LockLedger::none());
    }

    /// Start watching `pid`:
    ///  waits marked with `begin` and `end` are announced as waiting for `waiting_for`, the child is ended when it shows no
    ///  sign of work for the terms' window, and the lock files the child leaves when it is ended are swept by `locks`.
    ///  A watch whose task cannot run (the runtime has no worker thread) watches nothing, and the waits it marks run unannounced and
    ///  unended.
    pub fn start_with_locks(waiting_for: &str, pid: u32, terms: WatchTerms, locks: LockLedger) -> Self {
        let (marks, incoming): (UnboundedSender<Mark>, UnboundedReceiver<Mark>) = unbounded_channel();
        let expired: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let stoppable: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let longest: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
        let ceiling: Arc<AtomicU64> = Arc::new(AtomicU64::new(u64::MAX));
        let ceiling_hit: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let flags: WatchFlags = WatchFlags {
            expired: Arc::clone(&expired),
            stoppable: Arc::clone(&stoppable),
            longest: Arc::clone(&longest),
            ceiling: Arc::clone(&ceiling),
            ceiling_hit: Arc::clone(&ceiling_hit),
        };
        let subject: String = String::from(waiting_for);
        let limit: Duration = terms.limit;
        let (ended, finished): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel();
        // The task holds `ended` and drops it however it ends (a panic or an abort included), which `finished` reads as a
        // disconnection.
        let spawned: Option<JoinHandle<()>> = spawn_on_runtime(async move {
            let _ends_with_the_task: std::sync::mpsc::Sender<()> = ended;
            watch_loop(incoming, flags, subject, pid, terms, locks).await;
        });
        let task: Option<RunningWatcher> = spawned.map(|handle| { return RunningWatcher { handle, finished }; });
        return Self { marks: Some(marks), expired, task, limit, stoppable, longest, ledger_step: None, ceiling, ceiling_hit };
    }

    /// Keep the longest quiet gap of this watch in the ledger under `step` when the watch is dropped.
    pub fn kept_as(mut self, step: &str) -> Self {
        self.ledger_step = Some(String::from(step));
        return self;
    }

    /// Announce the waits of this child but never end it for being quiet: a step that is not safe to run twice (a reference
    ///  update, a commit Git builds) is ended in the middle of its write only by the person.
    pub fn never_stop(&self) {
        self.stoppable.store(false, Ordering::Release);
    }

    /// Give every wait of this child a ceiling: the child is ended when one wait has lasted this long, whatever it shows and
    ///  whether or not a quiet child may be ended (owner, 2026-11-03: "nothing should ever wait forever or indefinitely").
    pub fn within(self, ceiling: Duration) -> Self {
        self.ceiling.store(u64::try_from(ceiling.as_micros()).unwrap_or(u64::MAX - 1), Ordering::Release);
        return self;
    }

    /// Whether the child was ended because its ceiling passed.
    pub fn ceiling_hit(&self) -> bool {
        return self.ceiling_hit.load(Ordering::Acquire);
    }

    /// Send one mark; a watcher that is gone has nothing to be told.
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

/// Stops the watcher task, waits for it, and keeps the longest quiet gap it saw.
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
        if let Some(step) = self.ledger_step.take() {
            let gap: Duration = Duration::from_micros(self.longest.load(Ordering::Acquire));
            quiet_ledger::note(step.as_str(), gap, self.expired());
        }
    }
}

/// How long dropping a watch waits for its task to end: the bound of "nothing should ever wait forever" on a wait that is over in
///  a few milliseconds (the task reads the closed marks at its next look, at most `STOP_POLL` after, then sweeps the lock files
///  of a step it ended). No measurement picked it; it is the budget the start retry gives a refused start
///  (`start_retry.rs`, `START_RETRY_BUDGET`), open to the owner's veto.
const WATCHER_END_BOUND: Duration = Duration::from_secs(5);

/// A watcher task and the signal that it has ended.
struct RunningWatcher {
    /// The task, aborted when it does not end by `WATCHER_END_BOUND`.
    handle: JoinHandle<()>,
    /// Disconnects when the task ends.
    finished: std::sync::mpsc::Receiver<()>,
}

/// What:
///  The watcher task's loop: wait until the next notice or the next look at the child's work is due, print or end, repeat.
/// Why:
///   Between waits the task awaits the next mark without a timeout, so an idle watch costs nothing, and a dropped
///  watch ends it because its sender is gone. During a wait it wakes at the notice cadence and at the sampling interval of
///  the push meter, whichever is sooner, and at the window itself when that is shorter. The notice is written on the blocking
///  pool, because a terminal that does not take a line must not hold a worker; where no thread can start the notices stop and the
///  judgment goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const mark of marksUntilTimeout()) { if (since === undefined) continue; if (stoppable && clock.stalled()) await stop(pid); else if (ticker.due(elapsed)) await notice(); }
/// ```
async fn watch_loop(mut incoming: UnboundedReceiver<Mark>, flags: WatchFlags, waiting_for: String, pid: u32, terms: WatchTerms, locks: LockLedger) {
    let expired: &AtomicBool = flags.expired.as_ref();
    let stoppable: &AtomicBool = flags.stoppable.as_ref();
    let longest: Arc<AtomicU64> = Arc::clone(&flags.longest);
    let (limit, cadence): (Duration, crate::wait_progress::Cadence) = (terms.limit, terms.cadence);
    let mut notices: Option<Box<dyn Write + Send>> = Some(terms.notices);
    let mut marks: Marks = Marks { since: None, ticker: Ticker::new(cadence), printed: 0, clock: None, pid, window: limit, longest };
    loop {
        let Some(since) = marks.since else {
            match incoming.recv().await {
                Some(mark) => marks.apply(mark, cadence),
                None => return,
            }
            continue;
        };
        let waited: Duration = since.elapsed();
        // A notice that is due goes out before the stall is judged, so a child ended for being quiet was announced.
        if marks.ticker.due(waited) {
            let line: String = child_wait_line(&ChildWaitFacts {
                waiting_for: waiting_for.as_str(),
                pid,
                waited,
                limit,
                limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
                repeat: marks.printed > 0,
                ends_when_quiet: stoppable.load(Ordering::Acquire),
            });
            marks.printed += 1;
            if let Some(sink) = notices.take() {
                notices = write_notice(sink, line).await;
            }
            // The wait may have ended, or another begun, while the line was written.
            while let Ok(mark) = incoming.try_recv() {
                marks.apply(mark, cadence);
            }
            if marks.since != Some(since) {
                continue;
            }
        }
        // The clock is read at every look, a step that may not be run twice included, so its quiet gap is measured even though
        // the step is never ended. Printing can never keep a child that shows no sign of work past its window, because the
        // judgment follows the notice however long the notice took.
        // The ceiling ends the child whatever it shows and whether or not a quiet child may be ended.
        let ceiling: Duration = Duration::from_micros(flags.ceiling.load(Ordering::Acquire));
        if waited >= ceiling {
            flags.ceiling_hit.store(true, Ordering::Release);
            expired.store(true, Ordering::Release);
            marks.keep_gap();
            marks.since = None;
            marks.clock = None;
            stop_process(pid, &locks, &mut incoming).await;
            continue;
        }
        let verdict: Option<StopCause> = marks.clock.as_mut().and_then(Clock::check);
        let quiet: bool = match verdict {
            Some(cause) => matches!(cause, StopCause::Stalled(_) | StopCause::Deadline),
            None => false,
        };
        if quiet && stoppable.load(Ordering::Acquire) {
            // Raised before the child is signalled, so the waiting thread that its end releases reads it as true.
            expired.store(true, Ordering::Release);
            marks.keep_gap();
            marks.since = None;
            marks.clock = None;
            stop_process(pid, &locks, &mut incoming).await;
            continue;
        }
        let until_notice: Duration = marks.ticker.next_in(since.elapsed());
        let until_ceiling: Duration = Duration::from_micros(flags.ceiling.load(Ordering::Acquire)).saturating_sub(since.elapsed());
        let sleep: Duration = until_notice.min(SAMPLE_INTERVAL).min(limit).min(until_ceiling).max(SHORTEST_SLEEP);
        match tokio::time::timeout(sleep, incoming.recv()).await {
            Ok(Some(mark)) => marks.apply(mark, cadence),
            Ok(None) => return,
            Err(_slept) => {}
        }
    }
}

/// What:
///  Write one notice line to `sink` on the blocking pool and hand the sink back; nothing where the write could not run or failed.
/// Why:
///   A notice is advisory, so a failed write is ignored as it always was, and a terminal that never takes the line holds a pool
///  thread, not a worker of the runtime; with no thread to start the notices of this watch stop (the owner, 2026-11-07, "Fail
///  plainly": no inline run of a call that can wait).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sink = await runBlockingJob(() => { sink.write(line + '\n'); return sink; }).catch(() => undefined);
/// ```
async fn write_notice(mut sink: Box<dyn Write + Send>, line: String) -> Option<Box<dyn Write + Send>> {
    return match crate::blocking_job::run_blocking_job(move || {
        // Terminal output error: a notice is advisory; a failed write must not stop the watch.
        let _written = writeln!(sink, "{line}");
        return sink;
    })
    .await
    {
        Ok(kept) => Some(kept),
        Err(error) => {
            crate::diagnostic_log::debug("childWatch", format!("a notice was not written: {error}").as_str());
            None
        }
    };
}

/// What:
///  How the object reader's wait names itself in notices and in the timeout sentence.
/// Why:
///   The reader is one long-lived `cat-file --batch` whose single request can name a large blob, so it runs under the
///  bulk class on purpose (a `cat-file` that answers one query through `run_git` is a quick query step), and its
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
///  The sentence for a watched wait that showed no sign of work for the quiet window.
/// Why:
///   It names what was waited for, the window, where to change it and what to do next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} showed no sign of work for ${limit} seconds and was stopped ...`
/// ```
pub fn watched_timeout_text(waiting_for: &str, limit: Duration) -> String {
    return format!(
        "{waiting_for} showed no sign of work for {} seconds and was stopped ({CHILD_WAIT_VARIABLE} changes this). \
         If the step belonged to a commit, the next Git command of this wrapper finishes or undoes that commit, so \
         run the command again. If the step is quiet while it works, raise the window with {CHILD_WAIT_VARIABLE}=<seconds>; \
         if it stalls again, check with `ps` what that process waits for.",
        limit.as_secs()
    );
}

/// What:
///  Run a command to its end and collect its output, announcing the wait and ending the child when it shows no sign of work
///  for the window.
///  The command's input must be set by the caller;
///  its output streams are replaced by pipes.
/// Why:
///   `Command::output` blocks until the child ends and prints nothing meanwhile.
///  A Git step that stalls under load or behind a filter must say so, and must not hold a commit for good; a step that is slow
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
///  `output_watched`, where `stoppable` says whether a child that shows no sign of work for the window is ended, and
///  `ledger_step` names the step under which its longest quiet gap is kept.
/// Why:
///   A step that is not safe to run twice is announced at the usual interval and waited for as long as it lives; the caller
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
///  `output_watched_stopping` where the step is ended at `ceiling` whatever it shows, or never ended for its length when there is
///  none.
/// Why:
///   The owner's rule is that nothing waits for ever; the ceiling is what ends a step that keeps working and one that may not be
///  ended for being quiet. A step with no ceiling is a write in place that is not safe to run twice (owner, 2026-11-04: "Never cut
///  a branch write (Recommended)"; `transaction_git.rs`, `step_bound`): ending it in the middle of its write could cost the
///  person's data, so only the person ends it.
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
    let ceiling: Option<Duration> = match bound {
        StepBound::EndAt(limit) => Some(limit),
        StepBound::LeaveRunning { .. } | StepBound::Unbounded => None,
    };
    // A step that is never ended writes into files that outlive this command, not into pipes this command holds: when the command
    // stops waiting and exits, the step must find its output open, or it dies of SIGPIPE at its next write, which is a cut.
    let captured: Option<Capture> = if matches!(bound, StepBound::EndAt(_)) { None } else { Some(Capture::new()?) };
    match captured.as_ref() {
        Some(files) => {
            let (out, err) = files.stdio()?;
            command.stdout(out).stderr(err);
        }
        None => {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        }
    }
    // Taken before the child starts, so a lock born earlier is never taken for the child's.
    let locks: LockLedger = LockLedger::before_start(command);
    let child: Child = command.spawn_retrying()?;
    let started: ChildWatch = ChildWatch::start_with_locks(waiting_for, child.id(), terms, locks).kept_as(ledger_step);
    // A step with no ceiling is waited for as long as it lives: the watch starts without one.
    let watch: ChildWatch = match ceiling {
        Some(limit) => started.within(limit),
        None => started,
    };
    if !stoppable {
        watch.never_stop();
    }
    watch.begin();
    let output: std::io::Result<Output> = match bound {
        StepBound::LeaveRunning { after, extra } => {
            // The child moves to a thread that waits for it and collects its output, so the command can stop waiting and the child
            // goes on; the thread reaps it when it ends.
            let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::ExitStatus>>();
            let waiter = std::thread::Builder::new().name(String::from("cli-git-left-running-step")).spawn(move || {
                let mut leaving: Child = child;
                // The command may have stopped waiting; a closed channel is that case, and there is nobody to tell.
                let _abandoned = sender.send(leaving.wait());
            });
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
        StepBound::EndAt(_) => child.wait_with_output(),
        StepBound::Unbounded => {
            let mut waited_for: Child = child;
            match waited_for.wait() {
                Ok(status) => Ok(output_of(status, captured.as_ref())),
                Err(error) => Err(error),
            }
        }
    };
    watch.end();
    if let Some(limit) = ceiling
        && watch.ceiling_hit()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            CeilingReached(format!("{waiting_for} ran for {} seconds, the longest the wrapper lets one step run, and was ended", limit.as_secs())),
        ));
    }
    if watch.expired() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            watched_timeout_text(waiting_for, watch.limit()),
        ));
    }
    return output;
}

/// Watch controls stay out of the release executable.
#[cfg(test)]
#[path = "child_watch_tests.rs"]
mod tests;
