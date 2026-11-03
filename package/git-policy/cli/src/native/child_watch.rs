//! What:
//!  A watcher for a wait that blocks inside a read or a wait on a child process:
//!  it prints a notice after the grace and at every interval while the wait lasts,
//!  and stops the child when the wait crosses its bound.
//! Why:
//!  The wrapper reads Git objects from a long-lived `git cat-file --batch` and runs Git steps of a commit
//!  with their output captured.
//!  Both block in the operating system with nothing printed and no end,
//!  so a stalled child looks the same as a slow one and holds the commit for good (issue 1811).
//!  A watcher thread sees the wait from outside:
//!  the waiting thread marks where a wait begins and ends,
//!  the watcher prints, and at the bound it ends the child so the blocked call returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = startWatch({ waitingFor, pid, limitMs, cadence, notices });
//! watch.begin(); try { await child.read(); } finally { watch.end(); }
//! if (watch.expired) throw new Error(`${waitingFor} did not finish within ${limit} seconds`);
//! ```

/// The bound variable shared by every class of step, and the classes themselves.
use super::child_class::{CHILD_WAIT_VARIABLE, ChildClass};
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// The notice cadence, its ticker and the child-wait line.
use super::wait_progress::{Cadence, ChildWaitFacts, Ticker, child_wait_line};
/// The child and command types of the one-shot run.
use std::process::{Child, Command, Output, Stdio};
/// The sink a notice goes to.
use std::io::Write;
/// The flag the watcher raises when a wait crossed the bound, read by the waiting thread.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
/// The marks the waiting thread sends the watcher; the watcher owns every other piece of state.
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
/// The lock files a stopped step leaves.
use super::stopped_git_locks::LockLedger;
/// How long a stopped step may take to end after the polite request.
use super::bounded_push::STOP_GRACE;
/// The processes below the step, and the identity checks that make signalling them safe.
use super::process_tree::{TreeMember, descendants_of, kill_members, member_of, still_running, terminate_members};
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
    /// How long one wait may last before the child is stopped.
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
}

impl Marks {
    /// Take one mark from the waiting thread into account.
    fn apply(&mut self, mark: Mark, cadence: Cadence) {
        match mark {
            Mark::Begin(at) => {
                self.since = Some(at);
                self.ticker = Ticker::new(cadence);
                self.printed = 0;
            }
            Mark::End => self.since = None,
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
/// process.kill(pid, 'SIGTERM'); await Promise.race([exited, delay(graceMs)]) || process.kill(pid, 'SIGKILL'); locks.sweep(tree);
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
///  The watcher of one child process.
/// Why:
///   One watch serves every wait on the same child:
///  a long-lived reader marks one wait per request, a one-shot command marks one wait.
///  Dropping the watch ends its thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ChildWatch implements Disposable { begin(); end(); get expired(): boolean }
/// ```
pub struct ChildWatch {
    /// Where the marks go; dropped first when the watch is dropped, which tells the watcher thread to end.
    marks: Option<Sender<Mark>>,
    /// Whether a wait crossed the bound and the child was stopped; raised by the watcher before it signals the child.
    expired: Arc<AtomicBool>,
    /// The watcher thread, joined when the watch is dropped.
    thread: Option<JoinHandle<()>>,
    /// The bound a wait may last.
    limit: Duration,
}

impl ChildWatch {
    /// Start watching `pid`, a child that holds no Git lock: see `start_with_locks`.
    pub fn start(waiting_for: &str, pid: u32, terms: WatchTerms) -> Self {
        return Self::start_with_locks(waiting_for, pid, terms, LockLedger::none());
    }

    /// Start watching `pid`:
    ///  waits marked with `begin` and `end` are announced as waiting for `waiting_for` and bounded by the terms' limit,
    ///  and the lock files the child leaves when it is stopped are swept by `locks`.
    ///  A watch whose thread cannot start watches nothing, and the waits it marks run unannounced and unbounded.
    pub fn start_with_locks(waiting_for: &str, pid: u32, terms: WatchTerms, locks: LockLedger) -> Self {
        let (marks, incoming): (Sender<Mark>, Receiver<Mark>) = channel();
        let expired: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let raised: Arc<AtomicBool> = Arc::clone(&expired);
        let subject: String = String::from(waiting_for);
        let limit: Duration = terms.limit;
        let thread: Option<JoinHandle<()>> = std::thread::Builder::new()
            .name(String::from("cli-git-child-watch"))
            .spawn(move || {
                watch_loop(&incoming, raised.as_ref(), subject.as_str(), pid, terms, &locks);
            })
            .ok();
        return Self { marks: Some(marks), expired, thread, limit };
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

    /// Whether a wait crossed the bound and the child was stopped.
    pub fn expired(&self) -> bool {
        return self.expired.load(Ordering::Acquire);
    }

    /// The bound a wait may last.
    pub fn limit(&self) -> Duration {
        return self.limit;
    }
}

/// Stops the watcher thread and waits for it.
impl Drop for ChildWatch {
    fn drop(&mut self) {
        drop(self.marks.take());
        if let Some(thread) = self.thread.take() {
            // A watcher that panicked has nothing left to report; the wait it watched is over.
            let _joined = thread.join();
        }
    }
}

/// What:
///  The watcher thread's loop: sleep until the next notice or the bound is due, print or stop, repeat.
/// Why:
///   Between waits the thread blocks on the next mark without a timeout, so an idle watch costs nothing, and a dropped
///  watch ends it because its sender is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const mark of marksUntilTimeout()) { if (since === undefined) continue; if (elapsed >= limit) stop(pid); else if (ticker.due(elapsed)) notice(); }
/// ```
fn watch_loop(incoming: &Receiver<Mark>, expired: &AtomicBool, waiting_for: &str, pid: u32, mut terms: WatchTerms, locks: &LockLedger) {
    let mut marks: Marks = Marks { since: None, ticker: Ticker::new(terms.cadence), printed: 0 };
    loop {
        let Some(since) = marks.since else {
            match incoming.recv() {
                Ok(mark) => marks.apply(mark, terms.cadence),
                Err(_) => return,
            }
            continue;
        };
        let mut waited: Duration = since.elapsed();
        // A notice that is due goes out before the bound is checked, so a child stopped at its bound was announced.
        if marks.ticker.due(waited) {
            let line: String = child_wait_line(&ChildWaitFacts {
                waiting_for,
                pid,
                waited,
                limit: terms.limit,
                limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
                repeat: marks.printed > 0,
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
            // The bound is checked right after a notice, however long the notice took, so printing can never keep a
            // child past its bound.
            waited = since.elapsed();
        }
        if waited >= terms.limit {
            // Raised before the child is signalled, so the waiting thread that its end releases reads it as true.
            expired.store(true, Ordering::Release);
            marks.since = None;
            stop_process(pid, locks, incoming);
            continue;
        }
        let until_notice: Duration = marks.ticker.next_in(waited);
        let until_bound: Duration = terms.limit.saturating_sub(waited);
        let sleep: Duration = until_notice.min(until_bound).max(SHORTEST_SLEEP);
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
///  The sentence for a watched wait that crossed its bound.
/// Why:
///   It names what was waited for, the bound, where to change it and what to do next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} did not finish within ${limit} seconds and was stopped ...`
/// ```
pub fn watched_timeout_text(waiting_for: &str, limit: Duration) -> String {
    return format!(
        "{waiting_for} did not finish within {} seconds and was stopped ({CHILD_WAIT_VARIABLE} changes this). \
         If the step belonged to a commit, the next Git command of this wrapper finishes or undoes that commit, so \
         run the command again. If the step was only slow, raise the bound with {CHILD_WAIT_VARIABLE}=<seconds>; \
         if it stalls again, check with `ps` what that process waits for.",
        limit.as_secs()
    );
}

/// What:
///  Run a command to its end and collect its output, announcing the wait and stopping the child at the bound.
///  The command's input must be set by the caller;
///  its output streams are replaced by pipes.
/// Why:
///   `Command::output` blocks until the child ends and prints nothing meanwhile.
///  A Git step that stalls under load or behind a filter must say so, and must not hold a commit for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, terms }); // throws on the bound
/// ```
pub fn output_watched(command: &mut Command, waiting_for: &str, terms: WatchTerms) -> std::io::Result<Output> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    // Taken before the child starts, so a lock born earlier is never taken for the child's.
    let locks: LockLedger = LockLedger::before_start(command);
    let child: Child = command.spawn_retrying()?;
    let watch: ChildWatch = ChildWatch::start_with_locks(waiting_for, child.id(), terms, locks);
    watch.begin();
    let output: std::io::Result<Output> = child.wait_with_output();
    watch.end();
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
