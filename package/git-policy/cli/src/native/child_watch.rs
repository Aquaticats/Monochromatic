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

/// Positive-seconds parsing shared with the lock wait limits.
use super::bounded_push::positive_seconds;
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// The notice cadence, its ticker and the child-wait line.
use super::wait_progress::{Cadence, ChildWaitFacts, Ticker, child_wait_line};
/// The child and command types of the one-shot run.
use std::process::{Child, Command, Output, Stdio};
/// Borrowed OS text of the environment value.
use std::ffi::OsStr;
/// The sink a notice goes to.
use std::io::Write;
/// Shared state between the waiting thread and the watcher.
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
/// The watcher thread.
use std::thread::JoinHandle;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// Environment variable that sets, in whole seconds, how long one Git process the wrapper starts for itself
///  (the object reader and the steps of a commit) may run before it is stopped.
pub const CHILD_WAIT_VARIABLE: &str = "CLI_GIT_CHILD_WAIT_SECONDS";

/// How long such a process may run when the variable is absent or unusable:
///  longer than any ordinary step, short enough that a stalled process is noticed within the hour.
pub const DEFAULT_CHILD_WAIT: Duration = Duration::from_secs(30 * 60);

/// The shortest sleep of the watcher, so a due moment that already passed cannot spin it.
const SHORTEST_SLEEP: Duration = Duration::from_millis(1);

/// What:
///  The child wait limit for a value of the variable.
/// Why:
///   A missing, zero or malformed value falls back to the default, never to "no limit".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveSeconds(value) ?? DEFAULT_CHILD_WAIT_MS
/// ```
pub fn child_wait_limit_from(value: Option<&OsStr>) -> Duration {
    let Some(text) = value else {
        return DEFAULT_CHILD_WAIT;
    };
    return positive_seconds(CHILD_WAIT_VARIABLE, text).unwrap_or(DEFAULT_CHILD_WAIT);
}

/// The child wait limit this process runs under: the variable in its environment, or the default.
pub fn child_wait_limit() -> Duration {
    return child_wait_limit_from(std::env::var_os(CHILD_WAIT_VARIABLE).as_deref());
}

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
    /// The terms this process runs under: the variables of its environment, notices on standard error.
    pub fn from_environment() -> Self {
        return Self {
            limit: child_wait_limit(),
            cadence: Cadence::from_environment(),
            notices: Box::new(std::io::stderr()),
        };
    }
}

/// What:
///  What the watcher thread shares with the waiting thread, behind one lock.
/// Why:
///   The waiting thread marks a wait's start and end; the watcher reads the marks and the clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WatchState = { since?: number; ticker: Ticker; printed: number; stopped: boolean };
/// ```
struct WatchState {
    /// When the current wait began, or nothing between waits.
    since: Option<Instant>,
    /// When the current wait's next notice is due.
    ticker: Ticker,
    /// How many notices the current wait printed.
    printed: u32,
    /// Whether the watch was dropped.
    stopped: bool,
    /// Whether a wait crossed the bound and the child was stopped.
    expired: bool,
}

/// The lock and the signal the two threads share.
struct Shared {
    /// The marks.
    state: Mutex<WatchState>,
    /// Signalled when a wait begins or ends or the watch stops.
    wake: Condvar,
}

/// Take the lock, ignoring poison: the state is plain marks that stay valid after a panic elsewhere.
fn locked(shared: &Shared) -> MutexGuard<'_, WatchState> {
    return shared.state.lock().unwrap_or_else(PoisonError::into_inner);
}

/// What:
///  Ends a process the way a watched wait does when it crosses its bound.
/// Why:
///   The blocked read or wait on that process then returns, and the waiting thread reports the bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(pid, 'SIGKILL');
/// ```
#[cfg(unix)]
fn stop_process(pid: u32) {
    let Ok(raw) = i32::try_from(pid) else {
        return;
    };
    // SAFETY: `kill` takes no pointers. The watcher fires only while a wait is marked, and the waiting thread reaps
    // the child only after it ends the mark, except for the window between a one-shot run's `wait_with_output`
    // returning and its `end` call (microseconds), where a reaped identifier could in principle be reused by another
    // process before the signal lands; the bound is far longer than that window, so a stop in it needs a wait that
    // crossed the bound in the same instant.
    unsafe {
        libc::kill(raw, libc::SIGKILL);
    }
}

/// A platform without signals cannot end the child from the watcher; the wait is only announced.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn stop_process(_pid: u32) {}

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
    /// The state shared with the watcher thread.
    shared: Arc<Shared>,
    /// The watcher thread, joined when the watch is dropped.
    thread: Option<JoinHandle<()>>,
    /// The bound a wait may last.
    limit: Duration,
    /// When the notices of each wait are due.
    cadence: Cadence,
}

impl ChildWatch {
    /// Start watching `pid`:
    ///  waits marked with `begin` and `end` are announced as waiting for `waiting_for` and bounded by the terms' limit.
    ///  A watch whose thread cannot start watches nothing, and the waits it marks run unannounced and unbounded.
    pub fn start(waiting_for: &str, pid: u32, terms: WatchTerms) -> Self {
        let shared: Arc<Shared> = Arc::new(Shared {
            state: Mutex::new(WatchState {
                since: None,
                ticker: Ticker::new(terms.cadence),
                printed: 0,
                stopped: false,
                expired: false,
            }),
            wake: Condvar::new(),
        });
        let watcher: Arc<Shared> = Arc::clone(&shared);
        let subject: String = String::from(waiting_for);
        let limit: Duration = terms.limit;
        let cadence: Cadence = terms.cadence;
        let thread: Option<JoinHandle<()>> = std::thread::Builder::new()
            .name(String::from("cli-git-child-watch"))
            .spawn(move || {
                watch_loop(&watcher, subject.as_str(), pid, terms);
            })
            .ok();
        return Self { shared, thread, limit, cadence };
    }

    /// Mark the start of a wait on the child.
    pub fn begin(&self) {
        let mut state: MutexGuard<'_, WatchState> = locked(&self.shared);
        state.since = Some(Instant::now());
        state.ticker = Ticker::new(self.cadence);
        state.printed = 0;
        drop(state);
        self.shared.wake.notify_all();
    }

    /// Mark the end of the wait.
    pub fn end(&self) {
        let mut state: MutexGuard<'_, WatchState> = locked(&self.shared);
        state.since = None;
        drop(state);
        self.shared.wake.notify_all();
    }

    /// Whether a wait crossed the bound and the child was stopped.
    pub fn expired(&self) -> bool {
        return locked(&self.shared).expired;
    }

    /// The bound a wait may last.
    pub fn limit(&self) -> Duration {
        return self.limit;
    }
}

/// Stops the watcher thread and waits for it.
impl Drop for ChildWatch {
    fn drop(&mut self) {
        locked(&self.shared).stopped = true;
        self.shared.wake.notify_all();
        if let Some(thread) = self.thread.take() {
            // A watcher that panicked has nothing left to report; the wait it watched is over.
            let _joined = thread.join();
        }
    }
}

/// What:
///  The watcher thread's loop: sleep until the next notice or the bound is due, print or stop, repeat.
/// Why:
///   Between waits the thread sleeps without a timeout, so an idle watch costs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (!stopped) { if (since === undefined) await signal(); else if (elapsed >= limit) stop(pid); else if (ticker.due(elapsed)) notice(); else await sleep(next); }
/// ```
fn watch_loop(shared: &Shared, waiting_for: &str, pid: u32, mut terms: WatchTerms) {
    let mut state: MutexGuard<'_, WatchState> = locked(shared);
    loop {
        if state.stopped {
            return;
        }
        let Some(since) = state.since else {
            state = shared.wake.wait(state).unwrap_or_else(PoisonError::into_inner);
            continue;
        };
        let waited: Duration = since.elapsed();
        if waited >= terms.limit {
            state.expired = true;
            state.since = None;
            stop_process(pid);
            continue;
        }
        if state.ticker.due(waited) {
            let line: String = child_wait_line(&ChildWaitFacts {
                waiting_for,
                pid,
                waited,
                limit: terms.limit,
                limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
                repeat: state.printed > 0,
            });
            state.printed += 1;
            // Terminal output error: a notice is advisory; a failed write must not stop the watch.
            let _written = writeln!(terms.notices, "{line}");
            continue;
        }
        let until_notice: Duration = state.ticker.next_in(waited);
        let until_bound: Duration = terms.limit.saturating_sub(waited);
        let sleep: Duration = until_notice.min(until_bound).max(SHORTEST_SLEEP);
        state = shared
            .wake
            .wait_timeout(state, sleep)
            .unwrap_or_else(PoisonError::into_inner)
            .0;
    }
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
         Run the command again; if it stalls again, check with `ps` what that process waits for.",
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
    let child: Child = command.spawn_retrying()?;
    let watch: ChildWatch = ChildWatch::start(waiting_for, child.id(), terms);
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
