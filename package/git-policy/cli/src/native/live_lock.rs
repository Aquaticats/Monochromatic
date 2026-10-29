//! What:
//!  Wait for a lock another live process holds, with a visible notice and a generous backstop.
//! Why:
//!  A holder that is alive but stuck (a hung hook, a stalled tool) would otherwise stall every later command
//!  with no output at all.
//!  After a short silence the waiter says whom it waits for and when it will give up,
//!  and past the backstop it fails closed: the command stops with the holder named and the lock left in place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await acquireOwnerLock({ lockDirectory, signal: AbortSignal.timeout(limitMs), onSlow: announce });
//! ```

/// Positive-seconds parsing shared with the push deadline.
use super::bounded_push::positive_seconds;
/// The owner lock, its bounded acquire and the notice interface.
use super::owner_lock::{
    DEFAULT_POLL_DELAY, OwnerLock, OwnerLockError, WaitNotice, acquire_owner_lock_for,
};
/// The lock record whose owner the notice names.
use super::owner_lock_record::{PublishedOwner, read_owner_lock_record};
/// A commit's place in line, and how it reads.
use super::wait_progress::{QueuePlace, queue_place_text};
/// Borrowed OS text of the environment value.
use std::ffi::OsStr;
/// `Write` is the sink the notice goes to.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// Time spans.
use std::time::Duration;

/// Environment variable that sets, in whole seconds, how long a command waits for a lock held by a live process.
pub const LIVE_WAIT_VARIABLE: &str = "CLI_GIT_LOCK_WAIT_SECONDS";

/// How long a command waits for a live holder when the variable is absent or unusable:
///  longer than any ordinary hook run, short enough that a stuck holder is noticed within the hour.
pub const DEFAULT_LIVE_WAIT: Duration = Duration::from_secs(30 * 60);

/// What:
///  The wait limit for a value of the variable.
/// Why:
///   A missing, zero or malformed value falls back to the default,
///  never to "no limit".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveSeconds(value) ?? DEFAULT_LIVE_WAIT_MS
/// ```
pub fn live_wait_limit_from(value: Option<&OsStr>) -> Duration {
    let Some(text) = value else {
        return DEFAULT_LIVE_WAIT;
    };
    return positive_seconds(LIVE_WAIT_VARIABLE, text).unwrap_or(DEFAULT_LIVE_WAIT);
}

/// The wait limit this process runs under: the variable in its environment, or the default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// liveWaitLimitMs(process.env.CLI_GIT_LOCK_WAIT_SECONDS)
/// ```
pub fn live_wait_limit() -> Duration {
    return live_wait_limit_from(std::env::var_os(LIVE_WAIT_VARIABLE).as_deref());
}

/// What:
///  What one notice of a wait for a lock says.
/// Why:
///   Every wait for a lock names the same facts:
///  what it waits for, who holds it, how long it has waited, where it gives up, and its place when commits queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockWaitFacts = { subject; lockDirectory; holderPid?; waitedMs; limitMs; place?; repeat };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LockWaitFacts<'facts> {
    /// What the lock protects, in words.
    pub subject: &'facts str,
    /// The lock being waited for.
    pub lock_directory: &'facts Path,
    /// The live holder's process, when its record can be read.
    pub holder_pid: Option<i64>,
    /// How long the wait has lasted.
    pub waited: Duration,
    /// How long the wait lasts before it gives up.
    pub limit: Duration,
    /// This commit's place among the commits that queue for the same thing, when the registry knows it.
    pub place: Option<QueuePlace>,
    /// Whether an earlier notice of this wait already printed the hint.
    pub repeat: bool,
}

/// What:
///  The notice a slow wait prints.
/// Why:
///   It names the holder, the lock and both durations so the person can look at the holder or raise the limit;
///  the first notice of a wait adds what to do, and the repeats stay short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: still waiting for ${holder} to release ${subject} ...`
/// ```
pub fn slow_wait_line(facts: &LockWaitFacts<'_>) -> String {
    let holder: String = match facts.holder_pid {
        Some(pid) => format!("process {pid}"),
        None => String::from("another cli-git process"),
    };
    let place: String = match facts.place {
        Some(place) => format!(" {}", queue_place_text(place)),
        None => String::new(),
    };
    let hint: &str = if facts.repeat {
        ""
    } else {
        " Hint: if you started another Git command in a second terminal or session, let it finish and run this one \
         again; if no such command is running, check with `ps` and report the stuck process."
    };
    return format!(
        "cli-git: still waiting for {holder} to release {} ({}) after {} seconds; \
         giving up after {} seconds ({LIVE_WAIT_VARIABLE} changes this), leaving the lock in place.{place}{hint}",
        facts.subject,
        facts.lock_directory.display(),
        facts.waited.as_secs(),
        facts.limit.as_secs()
    );
}

/// What:
///  A wait notice that prints a line to a sink once the wait is slow, and again at every interval.
/// Why:
///   Brief contention stays silent, as the incumbent's waits were;
///  a long wait stops looking like a hang, and a wait that is still going keeps saying so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const onSlow = (waitedMs) => sink.write(slowWaitLine(...))
/// ```
pub struct HolderNotice<Sink: Write> {
    /// The lock being waited for.
    lock_directory: PathBuf,
    /// What the lock protects, in words.
    subject: &'static str,
    /// The limit the wait gives up at.
    limit: Duration,
    /// Where the line goes.
    sink: Sink,
    /// How many lines this wait has printed.
    printed: u32,
}

impl<Sink: Write> HolderNotice<Sink> {
    /// A notice for one lock, subject and limit.
    pub fn new(lock_directory: &Path, subject: &'static str, limit: Duration, sink: Sink) -> Self {
        return Self {
            lock_directory: lock_directory.to_path_buf(),
            subject,
            limit,
            sink,
            printed: 0,
        };
    }
}

impl<Sink: Write> WaitNotice for HolderNotice<Sink> {
    /// The first busy attempt says nothing: most waits end within the grace.
    fn waiting(&mut self) {}

    /// Print the wait line naming the holder when its record can be read;
    ///  the wait calls this after the grace and at every interval.
    fn slow(&mut self, waited: Duration) {
        let holder_pid: Option<i64> = match read_owner_lock_record(&self.lock_directory) {
            Ok(PublishedOwner::Owner(record)) => Some(record.owner_pid),
            Ok(PublishedOwner::Busy) | Err(_) => None,
        };
        let line: String = slow_wait_line(&LockWaitFacts {
            subject: self.subject,
            lock_directory: &self.lock_directory,
            holder_pid,
            waited,
            limit: self.limit,
            place: None,
            repeat: self.printed > 0,
        });
        self.printed += 1;
        // Terminal output error: the notice is advisory; a failed write must not stop the wait.
        let _written = writeln!(self.sink, "{line}");
    }
}

/// What:
///  Take an owner lock, waiting for a live holder with a slow-wait notice on standard error,
///  and giving up at this process's wait limit.
/// Why:
///   One call replaces the silent unbounded wait at every lock a command may queue behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireLiveOwnerLock({ lockDirectory, subject });
/// ```
pub fn acquire_live_owner_lock(
    lock_directory: &Path,
    subject: &'static str,
) -> Result<OwnerLock, OwnerLockError> {
    let limit: Duration = live_wait_limit();
    let mut notice: HolderNotice<std::io::Stderr> =
        HolderNotice::new(lock_directory, subject, limit, std::io::stderr());
    return acquire_owner_lock_for(lock_directory, DEFAULT_POLL_DELAY, &mut notice, limit);
}

/// Wait notices and limits are controlled in the test module.
#[cfg(test)]
#[path = "live_lock_tests.rs"]
mod tests;
