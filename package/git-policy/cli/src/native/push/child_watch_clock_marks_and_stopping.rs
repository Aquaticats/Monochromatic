//! What:
//!  What a child watch is made of below its public face:
//! the terms a watch runs under,
//! the marks the waiting thread sends,
//!  the shared flags,
//! and the stopping of a child and the processes it started.
//! Why:
//!  `child_watch.rs` holds the watch itself and the running of watched steps;
//! these pieces do not use it,
//! so they sit in a
//!  file of their own and `child_watch.rs` re-exports the public ones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type WatchTerms = { class: ChildClass; interval: number; limit: number };
//! function stopProcess(pid: number, locks: LockLedger, incoming: Receiver<Mark>): Promise<void>; // ask, wait the grace, kill, sweep
//! ```

use crate::child_class::ChildClass;
use crate::wait_progress::{Cadence, Ticker};
use crate::bounded_push::Patience;
use crate::push_progress::Clock;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::mpsc::UnboundedReceiver;
use crate::stopped_git_locks::LockLedger;
use crate::bounded_push::STOP_GRACE;
use crate::stop_waits_while_a_member_cannot_be_interrupted::escalation_waits;
use crate::pinned_member_signalled_through_a_pidfd::Delivered;
use crate::process_tree::{ProcessLook, TreeMember, descendants_of, kill_members, look_of, still_running, terminate_members};
use std::time::{Duration, Instant};

/// The shortest sleep of the watcher,
/// so a due moment that already passed cannot spin it.
pub(crate) const SHORTEST_SLEEP: Duration = Duration::from_millis(1);

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
    /// The quiet window:
    /// how long one wait may show no sign of work before the child is ended.
    pub limit: Duration,
    /// When the notices of a wait are due.
    pub cadence: Cadence,
    /// Where the notices go.
    pub notices: Box<dyn Write + Send>,
}

impl WatchTerms {
    /// The terms a step of `class` runs under:
    /// the class's bound (or the variable's) and the class's grace before
    ///  the first notice,
    /// the notice interval of the environment,
    /// notices on standard error.
    ///  The grace never exceeds the interval,
    /// so a short interval still announces a wait quickly,
    /// nor half the bound,
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

    /// The terms of a step that reads objects through a long-lived reader:
    /// those of an object-writing step,
    /// the longer
    ///  class,
    /// because one request can name a large blob.
    pub fn from_environment() -> Self {
        return Self::for_class(ChildClass::Object);
    }
}

/// What:
///  How long a step of `class` runs before its first notice,
/// for a notice `interval` and a `limit`.
/// Why:
///   The class's grace,
/// but never past the interval (a short interval still announces a wait quickly) and never past
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
///  What the waiting thread tells the watcher:
/// a wait began at a moment,
/// or the wait ended.
/// Why:
///   The watcher owns the marks it acts on (when the wait began,
/// when its next notice is due,
/// how many notices went out),
///  so no state is shared and no lock is held:
/// the waiting thread only sends,
/// and the watcher reads at its own pace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mark = { kind: 'begin'; at: number } | { kind: 'end' };
/// ```
pub(crate) enum Mark {
    /// A wait on the child began at this moment.
    Begin(Instant),
    /// The wait ended.
    End,
}

/// How often a stopped step is looked at while it ends.
pub(crate) const STOP_POLL: Duration = Duration::from_millis(10);

/// The marks of the wait the watcher is watching,
/// kept by the watcher task alone.
pub(crate) struct Marks {
    /// When the current wait began,
    /// or nothing between waits.
    pub(crate) since: Option<Instant>,
    /// When the current wait's next notice is due.
    pub(crate) ticker: Ticker,
    /// How many notices the current wait printed.
    pub(crate) printed: u32,
    /// The clock that reads the signs of work of the child during the current wait;
    /// nothing between waits.
    pub(crate) clock: Option<Clock>,
    /// The child.
    pub(crate) pid: u32,
    /// The quiet window.
    pub(crate) window: Duration,
    /// The longest quiet gap of any wait on this child,
    /// in microseconds,
    /// read by the waiting thread.
    pub(crate) longest: Arc<AtomicU64>,
}

impl Marks {
    /// Take one mark from the waiting thread into account;
    /// a wait that begins starts a clock on the child with the window as its
    ///  stall and no wall-clock fallback,
    /// because elapsed time alone never ends a step.
    pub(crate) fn apply(&mut self, mark: Mark, cadence: Cadence) {
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

    /// Keep the longest quiet gap of the wait that is ending,
    /// the one still running included.
    pub(crate) fn keep_gap(&self) {
        // A clock that reads nothing (no `/proc`) has no quiet gap to keep: its figure would be the elapsed time.
        if let Some(clock) = self.clock.as_ref()
            && clock.measures()
        {
            let micros: u64 = u64::try_from(clock.longest_quiet().as_micros()).unwrap_or(u64::MAX);
            self.longest.fetch_max(micros, Ordering::AcqRel);
        }
    }
}

/// The name of `signal` as the person knows it,
/// for a line that says which signal was not sent.
#[cfg(unix)]
fn signal_name(signal: libc::c_int) -> &'static str {
    return match signal {
        libc::SIGTERM => "SIGTERM",
        libc::SIGKILL => "SIGKILL",
        _ => "a signal",
    };
}

/// What:
///  The pin of the step's root,
/// taken from the identity the stop read before any wait,
/// or nothing when that look could not establish one.
/// Why:
///  A signal must reach the process the stop read and never a process that took its number afterwards.
/// Reading the
///  record again at the signal proves only which process holds the number *then*:
/// the root can have ended and been
///  reaped while the grace ran,
/// and an unrelated process that received its number would be the one signalled.
/// This pin
///  names its process for good,
/// so both signals of the stop reach that process or nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rootPin = pinOfTheRoot(lookOf(pid)); // undefined when that look proved nothing
/// ```
#[cfg(unix)]
pub(crate) fn pin_of_the_root(at_start: &ProcessLook) -> Option<crate::pinned_member_signalled_through_a_pidfd::PinnedProcess> {
    use crate::pinned_member_signalled_through_a_pidfd::{PinOutcome, pin_of};
    let ProcessLook::Alive(found) = at_start else {
        return None;
    };
    return match pin_of(found) {
        PinOutcome::Held(pinned) => Some(pinned),
        PinOutcome::Gone | PinOutcome::Unreadable => None,
    };
}

/// What:
///  Send `signal` to the step's root through its pin,
/// or nothing when nothing proved which process the root was,
/// and say which it was.
/// Why:
///   A record that could not be read proves nothing about the process,
/// and a number that was not pinned may name an
///  unrelated process by now,
/// so no number is signalled;
/// the step is still waited for,
/// and the line says it was not
///  stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sendToTheRoot(pin: Pin | undefined, pid: number, signal: number): void;
/// ```
#[cfg(unix)]
pub(crate) fn send_to_the_root(
    pin: Option<&crate::pinned_member_signalled_through_a_pidfd::PinnedProcess>,
    raw: i32,
    signal: libc::c_int,
) {
    let Some(pinned) = pin else {
        crate::diagnostic_log::warn(
            "childWatch",
            format!(
                "the step {raw} was not sent {}: nothing proved which process its number named, so no number was signalled. The \
                 step is still waited for; check with `ps` whether it runs and what it waits for.",
                signal_name(signal)
            )
            .as_str(),
        );
        return;
    };
    let _delivered: Delivered = pinned.signal(signal);
}

/// What:
///  Whether the root of a stopped step is still running.
/// Why:
///   The record read at the start of the stop judges a root it found,
/// by the start time that proves the number still names it;
///  a record that could not be read is read again here,
/// because the start's read is older than the lock claim,
/// and a refusal is
///  not an exit.
/// A root that was unknown at the start therefore stays in the wait,
/// so it gets the grace it would have had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rootStillRunning = (look) => look.kind === 'unknown' ? lookOf(pid).kind !== 'gone' : look.kind === 'alive' && sameStart(look);
/// ```
#[cfg(unix)]
fn root_still_running(raw: i32, at_start: &ProcessLook) -> bool {
    return match at_start {
        ProcessLook::Alive(found) => still_running(found),
        ProcessLook::Gone => false,
        ProcessLook::Unknown => !matches!(look_of(raw), ProcessLook::Gone),
    };
}

/// What:
///  Whether a stop waits for anything still running:
/// the root of the step and every member of its tree.
/// Why:
///   The root is not always a member of the tree:
/// a record that could not be read leaves no start time to name it by,
///  and a stop that then looked only at the tree ended nothing and waited for nothing,
/// so a live Git got no grace between
///  the request and the kill.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theStopWaits = (tree, root) => rootStillRunning(root) || tree.some(stillRunning);
/// ```
#[cfg(unix)]
pub(crate) fn the_stop_waits(tree: &[TreeMember], raw: i32, at_start: &ProcessLook) -> bool {
    return root_still_running(raw, at_start) || tree.iter().any(still_running);
}

/// What:
///  Whether the stop keeps waiting after it has killed what it could.
/// Why:
///   The wait after the kill ends with a grace of its own,
/// extended only while a member is in uninterruptible
///  sleep and up to the stop's ceiling,
/// as the wait before the kill does.
/// A root whose record cannot be read stays
///  in that wait (`the_stop_waits`),
/// and nothing else bounds it,
/// so without this the stop waited for as long as the
///  process lived,
/// against a kill that was never sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitAfterTheKill = (elapsed, tree) => escalationWaits(elapsed, grace, tree) && theStopWaits(tree);
/// ```
#[cfg(unix)]
pub(crate) fn the_wait_after_the_kill(elapsed: Duration, tree: &[TreeMember], raw: i32, at_start: &ProcessLook) -> bool {
    return escalation_waits(elapsed, STOP_GRACE, tree) && the_stop_waits(tree, raw, at_start);
}

/// What:
///  Ends a process the way a watched wait does when it crosses its bound:
/// a catchable request first,
/// a bounded grace,
///  then the kill,
/// and then the removal of the lock files it could not remove itself.
/// Why:
///   Git removes its lock files only when it is asked to end with a signal it catches,
/// so a step that is killed at once
///  leaves its `.lock` files and every later Git command on the same files fails.
/// A step that answers the request ends
///  in the grace;
/// one that does not (it was stopped by the kernel in an uninterruptible call) is killed,
/// and its locks are
///  swept after it is gone.
/// The blocked read or wait on that process then returns,
/// and the waiting thread reports the
///  bound.
/// The processes the child started end with it.
///  Its root is pinned once,
/// from the look before any wait,
/// so a signal reaches that process and never one that took its
///  number afterwards,
/// and every wait of the stop is bounded:
/// a process whose record cannot be read is signalled
///  nothing and waited for no longer than the grace and the stop's ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(pid, 'SIGTERM'); await Promise.race([exited, delay(graceMs)]) || process.kill(pid, 'SIGKILL'); locks.sweep(tree.as_slice());
/// ```
#[cfg(unix)]
pub(crate) async fn stop_process(pid: u32, locks: &LockLedger, incoming: &mut UnboundedReceiver<Mark>) {
    let Ok(raw) = i32::try_from(pid) else {
        return;
    };
    // The processes below the child are read before it is stopped, because a process's link to its parent is gone once the
    // parent has exited; a step's helpers (a transport, a hook) would otherwise outlive it and hold its output open, and the
    // wait for that output would outlast the bound by as long as they live.
    let below: Vec<TreeMember> = descendants_of(raw);
    // Three answers, not two: a record that could not be read is not a process that has gone, so the root is still waited
    //  for (`root_still_running`).
    let root_at_start: ProcessLook = look_of(raw);
    // The pin is taken here, while nothing has had the chance to reap the root, and it names that process for good.
    let root_pin: Option<crate::pinned_member_signalled_through_a_pidfd::PinnedProcess> = pin_of_the_root(&root_at_start);
    let mut tree: Vec<TreeMember> = match root_at_start.clone() {
        ProcessLook::Alive(found) => vec![found],
        ProcessLook::Gone | ProcessLook::Unknown => Vec::new(),
    };
    tree.extend(below.iter().cloned());
    // A process number can be reused once the waiting thread has reaped the child, and the child is reaped as soon as it
    // ends, which may be well before the grace is over (a helper below it can outlive it). Both signals therefore go through
    // one pin taken from the look made here, before any wait, and never to a number read again; where that look proved nothing
    // nothing is signalled (`send_to_the_root`), because a refusal is not a proof of which process the number names.
    // The locks the tree holds are claimed while their pid files still name the holders: the signals below can cut Git's cleanup
    //  between the removal of a pid file and the removal of its lock.
    let seen: Vec<TreeMember> = tree.clone();
    let before_claims: LockLedger = locks.clone();
    let claiming: LockLedger = match crate::blocking_job::run_blocking_job(move || { return before_claims.claiming(seen.as_slice()); }).await {
        Ok(claimed) => claimed,
        Err(error) => {
            crate::diagnostic_log::debug("childWatch", format!("the locks of the step {pid} were not claimed: {error}").as_str());
            locks.clone()
        }
    };
    send_to_the_root(root_pin.as_ref(), raw, libc::SIGTERM);
    terminate_members(below.as_slice());
    let asked: Instant = Instant::now();
    while escalation_waits(asked.elapsed(), STOP_GRACE, tree.as_slice()) && the_stop_waits(tree.as_slice(), raw, &root_at_start) {
        tokio::time::sleep(STOP_POLL).await;
    }
    send_to_the_root(root_pin.as_ref(), raw, libc::SIGKILL);
    kill_members(below.as_slice());
    // The waiting thread reaps the child, so the watcher only looks. A member in an uninterruptible call ends when the kernel
    // lets it go, which can be after the grace; that wait is bounded as the grace before the kill was, so a stop whose record
    // cannot be read, or whose member cannot be interrupted, ends here rather than waiting for good. The locks the tree leaves
    // are swept then, or when the watch is dropped, whichever comes first.
    let killed: Instant = Instant::now();
    while the_wait_after_the_kill(killed.elapsed(), tree.as_slice(), raw, &root_at_start) {
        // A mark that arrives is ignored, and a timeout is the next look; a closed channel means the watch was dropped, so nobody
        // is left to tell and the locks are swept below.
        if let Ok(None) = tokio::time::timeout(STOP_POLL, incoming.recv()).await {
            break;
        }
    }
    // The sweep reads and removes files in the repository, which can wait on a disk, so it runs on the blocking pool; where no
    // thread can start it is not run (the owner, 2026-11-07, "Fail plainly": no inline run of a call that can wait on storage),
    // and the next Git command that meets such a lock names it.
    let sweeping: LockLedger = claiming;
    if let Err(error) = crate::blocking_job::run_blocking_job(move || { sweeping.sweep(tree.as_slice()); }).await {
        crate::diagnostic_log::debug("childWatch", format!("the lock files of the stopped step {pid} were not swept: {error}").as_str());
    }
}

/// A platform without signals cannot end the child from the watcher;
/// the wait is only announced.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub(crate) async fn stop_process(_pid: u32, _locks: &LockLedger, _incoming: &mut UnboundedReceiver<Mark>) {}

/// What:
///  The flags the waiting thread and the watcher task share.
/// Why:
///   They travel together to the watcher,
/// which writes `expired` and `longest` and reads `stoppable`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WatchFlags = { expired: AtomicBoolean; stoppable: AtomicBoolean; longest: AtomicNumber };
/// ```
pub(crate) struct WatchFlags {
    /// Raised by the watcher when it ended the child for being quiet.
    pub(crate) expired: Arc<AtomicBool>,
    /// Whether a quiet child may be ended.
    pub(crate) stoppable: Arc<AtomicBool>,
    /// The longest quiet gap so far,
    /// in microseconds.
    pub(crate) longest: Arc<AtomicU64>,
    /// How long one wait may last in all,
    /// in microseconds,
    /// whatever the child shows;
    /// the largest value means no ceiling.
    pub(crate) ceiling: Arc<AtomicU64>,
    /// Raised by the watcher when the child was ended because the ceiling passed,
    /// not because it was quiet.
    pub(crate) ceiling_hit: Arc<AtomicBool>,
    /// Raised by the watcher when the child was ended because the service was stopped,
    /// before the child is signalled,
    /// so the thread that its end releases reads it as true.
    pub(crate) stopped: Arc<AtomicBool>,
}
