//! What:
//!  A watcher for a wait that blocks inside a read or a wait on a child process:
//!  it prints a notice after the grace and at every interval while the wait lasts,
//!  and ends the child when it has shown no sign of work for the quiet window.
//! Why:
//!  The wrapper reads Git objects from a long-lived `git cat-file --batch` and runs Git steps of a commit
//!  with their output captured.
//!  Both block in the operating system with nothing printed and no end,
//!  so a stalled child looks the same as a slow one and holds the commit for good (issue 1811).
//!  A watcher thread sees the wait from outside:
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
use crate::wait_progress::{Cadence, ChildWaitFacts, Ticker, child_wait_line};
/// The patience of a run that ends on a stall, and the stop it reports.
use crate::bounded_push::{Patience, StopCause};
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
/// The marks the waiting thread sends the watcher; the watcher owns every other piece of state.
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
/// The lock files a stopped step leaves.
use crate::stopped_git_locks::LockLedger;
/// How long a stopped step may take to end after the polite request.
use crate::bounded_push::STOP_GRACE;
/// The processes below the step, and the identity checks that make signalling them safe.
use crate::process_tree::{TreeMember, descendants_of, kill_members, member_of, still_running, terminate_members};
/// The watcher thread.
use std::thread::JoinHandle;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// The shortest sleep of the watcher, so a due moment that already passed cannot spin it.
const SHORTEST_SLEEP: Duration = Duration::from_millis(1);

/// What:
///  When a watched wait prints and where it stops the child.
/// Why:
///   The production entry reads these from the environment and writes to standard error;
///  tests choose short ones and read the notices from a buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WatchTerms = { limitMs: number; cadence: Cadence; notices: Writable };
/// ```
pub struct WatchTerms {
    /// The quiet window: how long one wait may show no sign of work before the child is ended.
    pub limit: Duration,
    /// When the notices of a wait are due.
    pub cadence: Cadence,
    /// Where the notices go.
    pub notices: Box<dyn Write + Send>,
}

impl WatchTerms {
    /// The terms a step of `class` runs under: the class's bound (or the variable's) and the class's grace before
    ///  the first notice, the notice interval of the environment, notices on standard error.
    ///  The grace never exceeds the interval, so a short interval still announces a wait quickly, nor half the bound,
    ///  so a bound set below the class's grace still gets a notice before the step is stopped.
    pub fn for_class(class: ChildClass) -> Self {
        let interval: Cadence = Cadence::from_environment();
        let limit: Duration = class.limit();
        return Self {
            limit,
            cadence: Cadence { grace: grace_for(class, interval.interval, limit), interval: interval.interval },
            notices: Box::new(std::io::stderr()),
        };
    }

    /// The terms of a step that reads objects through a long-lived reader: those of an object-writing step, the longer
    ///  class, because one request can name a large blob.
    pub fn from_environment() -> Self {
        return Self::for_class(ChildClass::Object);
    }
}

/// What:
///  How long a step of `class` runs before its first notice, for a notice `interval` and a `limit`.
/// Why:
///   The class's grace, but never past the interval (a short interval still announces a wait quickly) and never past
///  half the bound (a bound set below the grace still gets a notice before the step is stopped).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.min(class.noticeGrace, interval, limit / 2)
/// ```
pub fn grace_for(class: ChildClass, interval: Duration, limit: Duration) -> Duration {
    return class.notice_grace().min(interval).min(limit / 2);
}

/// What:
///  What the waiting thread tells the watcher: a wait began at a moment, or the wait ended.
/// Why:
///   The watcher owns the marks it acts on (when the wait began, when its next notice is due, how many notices went out),
///  so no state is shared and no lock is held: the waiting thread only sends, and the watcher reads at its own pace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mark = { kind: 'begin'; at: number } | { kind: 'end' };
/// ```
enum Mark {
    /// A wait on the child began at this moment.
    Begin(Instant),
    /// The wait ended.
    End,
}

/// How often a stopped step is looked at while it ends.
const STOP_POLL: Duration = Duration::from_millis(10);

/// The marks of the wait the watcher is watching, kept by the watcher thread alone.
struct Marks {
    /// When the current wait began, or nothing between waits.
    since: Option<Instant>,
    /// When the current wait's next notice is due.
    ticker: Ticker,
    /// How many notices the current wait printed.
    printed: u32,
    /// The clock that reads the signs of work of the child during the current wait; nothing between waits.
    clock: Option<Clock>,
    /// The child.
    pid: u32,
    /// The quiet window.
    window: Duration,
    /// The longest quiet gap of any wait on this child, in microseconds, read by the waiting thread.
    longest: Arc<AtomicU64>,
}

impl Marks {
    /// Take one mark from the waiting thread into account; a wait that begins starts a clock on the child with the window as its
    ///  stall and no wall-clock fallback, because elapsed time alone never ends a step.
    fn apply(&mut self, mark: Mark, cadence: Cadence) {
        match mark {
            Mark::Begin(at) => {
                self.since = Some(at);
                self.ticker = Ticker::new(cadence);
                self.printed = 0;
                self.clock = Some(Clock::start(self.pid, Patience::Progress { stall: self.window }).without_wall_fallback());
            }
            Mark::End => {
                self.keep_gap();
                self.since = None;
                self.clock = None;
            }
        }
    }

    /// Keep the longest quiet gap of the wait that is ending, the one still running included.
    fn keep_gap(&self) {
        // A clock that reads nothing (no `/proc`) has no quiet gap to keep: its figure would be the elapsed time.
        if let Some(clock) = self.clock.as_ref().filter(|measuring| { return measuring.measures(); }) {
            let micros: u64 = u64::try_from(clock.longest_quiet().as_micros()).unwrap_or(u64::MAX);
            self.longest.fetch_max(micros, Ordering::AcqRel);
        }
    }
}

/// What:
///  Ends a process the way a watched wait does when it crosses its bound: a catchable request first, a bounded grace,
///  then the kill, and then the removal of the lock files it could not remove itself.
/// Why:
///   Git removes its lock files only when it is asked to end with a signal it catches, so a step that is killed at once
///  leaves its `.lock` files and every later Git command on the same files fails. A step that answers the request ends
///  in the grace; one that does not (it was stopped by the kernel in an uninterruptible call) is killed, and its locks are
///  swept after it is gone. The blocked read or wait on that process then returns, and the waiting thread reports the
///  bound. The processes the child started end with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(pid, 'SIGTERM'); await Promise.race([exited, delay(graceMs)]) || process.kill(pid, 'SIGKILL'); locks.sweep(tree.as_slice());
/// ```
#[cfg(unix)]
fn stop_process(pid: u32, locks: &LockLedger, incoming: &Receiver<Mark>) {
    let Ok(raw) = i32::try_from(pid) else {
        return;
    };
    // The processes below the child are read before it is stopped, because a process's link to its parent is gone once the
    // parent has exited; a step's helpers (a transport, a hook) would otherwise outlive it and hold its output open, and the
    // wait for that output would outlast the bound by as long as they live.
    let below: Vec<TreeMember> = descendants_of(raw);
    let root: Option<TreeMember> = member_of(raw);
    let mut tree: Vec<TreeMember> = root.iter().cloned().collect();
    tree.extend(below.iter().cloned());
    // A process number can be reused once the waiting thread has reaped the child, and the child is reaped as soon as it
    // ends, which may be well before the grace is over (a helper below it can outlive it). The signals therefore go to the
    // root only while it is still the process that was found, by its recorded start time; where no start time can be read
    // (a platform without `/proc`) the root is signalled without that check, and with nothing to look at the grace ends at once,
    // as the kill alone did before.
    let identified: bool = cfg!(target_os = "linux");
    let signal_root = |signal: libc::c_int| {
        if root.as_ref().map_or(!identified, still_running) {
            // SAFETY: `kill` takes no pointers; the process was checked a moment ago to be the one that was found.
            unsafe {
                libc::kill(raw, signal);
            }
        }
    };
    signal_root(libc::SIGTERM);
    terminate_members(below.as_slice());
    let asked: Instant = Instant::now();
    while asked.elapsed() < STOP_GRACE && tree.iter().any(still_running) {
        std::thread::sleep(STOP_POLL);
    }
    signal_root(libc::SIGKILL);
    kill_members(below.as_slice());
    // The waiting thread reaps the child, so the watcher only looks. A step in an uninterruptible call ends when the kernel
    // lets it go, which can be long after the bound; the locks it creates on the way out are swept then, or when the watch
    // is dropped, whichever comes first.
    while tree.iter().any(still_running) {
        match incoming.recv_timeout(STOP_POLL) {
            Err(RecvTimeoutError::Disconnected) => break,
            Ok(_) | Err(RecvTimeoutError::Timeout) => {}
        }
    }
    locks.sweep(tree.as_slice());
}

/// A platform without signals cannot end the child from the watcher; the wait is only announced.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn stop_process(_pid: u32, _locks: &LockLedger, _incoming: &Receiver<Mark>) {}

/// What:
///  The three flags the waiting thread and the watcher thread share.
/// Why:
///   They travel together to the watcher, which writes `expired` and `longest` and reads `stoppable`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WatchFlags = { expired: AtomicBoolean; stoppable: AtomicBoolean; longest: AtomicNumber };
/// ```
struct WatchFlags {
    /// Raised by the watcher when it ended the child for being quiet.
    expired: Arc<AtomicBool>,
    /// Whether a quiet child may be ended.
    stoppable: Arc<AtomicBool>,
    /// The longest quiet gap so far, in microseconds.
    longest: Arc<AtomicU64>,
    /// How long one wait may last in all, in microseconds, whatever the child shows; the largest value means no ceiling.
    ceiling: Arc<AtomicU64>,
    /// Raised by the watcher when the child was ended because the ceiling passed, not because it was quiet.
    ceiling_hit: Arc<AtomicBool>,
}

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
    /// Where the marks go; dropped first when the watch is dropped, which tells the watcher thread to end.
    marks: Option<Sender<Mark>>,
    /// Whether a wait showed no sign of work for the window and the child was ended; raised by the watcher before it signals
    ///  the child.
    expired: Arc<AtomicBool>,
    /// The watcher thread, joined when the watch is dropped.
    thread: Option<JoinHandle<()>>,
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
    ///  A watch whose thread cannot start watches nothing, and the waits it marks run unannounced and unended.
    pub fn start_with_locks(waiting_for: &str, pid: u32, terms: WatchTerms, locks: LockLedger) -> Self {
        let (marks, incoming): (Sender<Mark>, Receiver<Mark>) = channel();
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
        let thread: Option<JoinHandle<()>> = std::thread::Builder::new()
            .name(String::from("cli-git-child-watch"))
            .spawn(move || {
                watch_loop(&incoming, &flags, subject.as_str(), pid, terms, &locks);
            })
            .ok();
        return Self { marks: Some(marks), expired, thread, limit, stoppable, longest, ledger_step: None, ceiling, ceiling_hit };
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

/// Stops the watcher thread, waits for it, and keeps the longest quiet gap it saw.
impl Drop for ChildWatch {
    fn drop(&mut self) {
        drop(self.marks.take());
        if let Some(thread) = self.thread.take() {
            // A watcher that panicked has nothing left to report; the wait it watched is over.
            let _joined = thread.join();
        }
        if let Some(step) = self.ledger_step.take() {
            let gap: Duration = Duration::from_micros(self.longest.load(Ordering::Acquire));
            quiet_ledger::note(step.as_str(), gap, self.expired());
        }
    }
}

/// What:
///  The watcher thread's loop: sleep until the next notice or the next look at the child's work is due, print or end, repeat.
/// Why:
///   Between waits the thread blocks on the next mark without a timeout, so an idle watch costs nothing, and a dropped
///  watch ends it because its sender is gone. During a wait it wakes at the notice cadence and at the sampling interval of
///  the push meter, whichever is sooner, and at the window itself when that is shorter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const mark of marksUntilTimeout()) { if (since === undefined) continue; if (stoppable && clock.stalled()) stop(pid); else if (ticker.due(elapsed)) notice(); }
/// ```
fn watch_loop(incoming: &Receiver<Mark>, flags: &WatchFlags, waiting_for: &str, pid: u32, mut terms: WatchTerms, locks: &LockLedger) {
    let expired: &AtomicBool = flags.expired.as_ref();
    let stoppable: &AtomicBool = flags.stoppable.as_ref();
    let longest: Arc<AtomicU64> = Arc::clone(&flags.longest);
    let mut marks: Marks = Marks { since: None, ticker: Ticker::new(terms.cadence), printed: 0, clock: None, pid, window: terms.limit, longest };
    loop {
        let Some(since) = marks.since else {
            match incoming.recv() {
                Ok(mark) => marks.apply(mark, terms.cadence),
                Err(_) => return,
            }
            continue;
        };
        let waited: Duration = since.elapsed();
        // A notice that is due goes out before the stall is judged, so a child ended for being quiet was announced.
        if marks.ticker.due(waited) {
            let line: String = child_wait_line(&ChildWaitFacts {
                waiting_for,
                pid,
                waited,
                limit: terms.limit,
                limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
                repeat: marks.printed > 0,
                ends_when_quiet: stoppable.load(Ordering::Acquire),
            });
            marks.printed += 1;
            // Terminal output error: a notice is advisory; a failed write must not stop the watch.
            let _written = writeln!(terms.notices, "{line}");
            // The wait may have ended, or another begun, while the line was written.
            while let Ok(mark) = incoming.try_recv() {
                marks.apply(mark, terms.cadence);
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
            stop_process(pid, locks, incoming);
            continue;
        }
        let verdict: Option<StopCause> = marks.clock.as_mut().and_then(Clock::check);
        let quiet: bool = verdict.is_some_and(|cause| { return matches!(cause, StopCause::Stalled(_) | StopCause::Deadline); });
        if quiet && stoppable.load(Ordering::Acquire) {
            // Raised before the child is signalled, so the waiting thread that its end releases reads it as true.
            expired.store(true, Ordering::Release);
            marks.keep_gap();
            marks.since = None;
            marks.clock = None;
            stop_process(pid, locks, incoming);
            continue;
        }
        let until_notice: Duration = marks.ticker.next_in(since.elapsed());
        let until_ceiling: Duration = Duration::from_micros(flags.ceiling.load(Ordering::Acquire)).saturating_sub(since.elapsed());
        let sleep: Duration = until_notice.min(SAMPLE_INTERVAL).min(terms.limit).min(until_ceiling).max(SHORTEST_SLEEP);
        match incoming.recv_timeout(sleep) {
            Ok(mark) => marks.apply(mark, terms.cadence),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
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
///  How long a watched step is waited for, and what happens at the end.
/// Why:
///   A step that can be run twice is ended at a ceiling (owner, 2026-11-03: "nothing should ever wait forever or indefinitely"). A
///  write in place that cannot be run twice is never ended (owner, 2026-11-04: "Never cut a branch write (Recommended)"): the
///  command that started it returns at its own bound plus a short extra wait with an error that says the result is not known,
///  and the step is left running (owner, commit protocol revision 12, "Short extra wait (Recommended)"); only the commit service's
///  landing thread, whose clients are told through the protocol, waits for it without a bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StepBound = { kind: 'end-at'; ms } | { kind: 'leave-running'; afterMs; extraMs } | { kind: 'unbounded' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepBound {
    /// The step is ended, with its whole process tree, when it has run this long, whatever it shows.
    EndAt(Duration),
    /// The command stops waiting `after` plus `extra` after the step began and returns an error saying the result is not known;
    ///  the step is left running and reaped when it ends.
    LeaveRunning {
        /// The command's own bound.
        after: Duration,
        /// The short extra wait for the write the step started.
        extra: Duration,
    },
    /// The step is waited for as long as it lives.
    Unbounded,
}

/// The error of a step ended at its ceiling, which tells it apart from a step ended for being quiet: a step that may be run again
///  is run again after the second and not after the first.
#[derive(Debug)]
pub struct CeilingReached(
    /// The sentence that names the step and the ceiling.
    pub String,
);

impl std::fmt::Display for CeilingReached {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

impl std::error::Error for CeilingReached {}

/// The output of a step that ended: its status and what it wrote to the captured files.
fn output_of(status: std::process::ExitStatus, captured: Option<&Capture>) -> Output {
    let (stdout, stderr): (Vec<u8>, Vec<u8>) = captured.map_or_else(|| { return (Vec::new(), Vec::new()); }, Capture::read);
    return Output { status, stdout, stderr };
}

/// What:
///  The sentence for a step left running: what it was, how long it was waited for, and what to check.
/// Why:
///   The owner's rule for a write the command started and cannot confirm (commit protocol revision 12, "Short extra wait"): exit
///  non-zero saying the result is not known and what to look at. The step is a write in place, so the repository is what to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} was still running ${after + extra} seconds after it began, and was left running ...`
/// ```
pub fn not_known_text(waiting_for: &str, after: Duration, extra: Duration) -> String {
    return format!(
        "{waiting_for} was still running {} seconds after it began (its bound of {} seconds and {} more for the write it started), so it \
         was left running and not ended: it is not known whether it finished. Check what it changes with `git status` and `git log -1`, \
         look at the process with `ps` and at the disk it writes to, and run the command again if it did not finish.",
        after.saturating_add(extra).as_secs(),
        after.as_secs(),
        extra.as_secs()
    );
}

/// The error of a step left running: the command stopped waiting for it, so it is not known whether it finished.
#[derive(Debug)]
pub struct NotKnown(
    /// The sentence that names the step, how long it was waited for, and what to check.
    pub String,
);

impl std::fmt::Display for NotKnown {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

impl std::error::Error for NotKnown {}

/// Whether `error` is the error of a step left running because the command stopped waiting for it.
pub fn is_not_known(error: &std::io::Error) -> bool {
    return error.get_ref().is_some_and(|inner| { return inner.is::<NotKnown>(); });
}

/// Whether `error` is the error of a step ended at its ceiling.
pub fn is_ceiling(error: &std::io::Error) -> bool {
    return error.get_ref().is_some_and(|inner| { return inner.is::<CeilingReached>(); });
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
                Ok(finished) => finished.map(|status| { return output_of(status, captured.as_ref()); }),
                Err(_timeout_or_gone) => {
                    watch.end();
                    return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, NotKnown(not_known_text(waiting_for, after, extra))));
                }
            }
        }
        StepBound::EndAt(_) => child.wait_with_output(),
        StepBound::Unbounded => {
            let mut waited_for: Child = child;
            waited_for.wait().map(|status| { return output_of(status, captured.as_ref()); })
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
