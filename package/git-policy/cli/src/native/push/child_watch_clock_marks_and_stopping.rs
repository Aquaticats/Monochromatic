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
///  Send `signal` to the process `raw`,
/// through a descriptor taken from a look made right here,
/// or nothing when its record could not be read.
/// Why:
///  A number is a safe address only while its process is this process's own unreaped child,
/// and the root of a step
///  is reaped by the thread that waits for it as soon as it ends,
/// so a look taken before an awaited step (the lock
///  claim) says nothing about what the number names now.
/// The look is therefore made here,
/// and the descriptor taken
///  inside `signal_member` carries its answer through to the signal.
/// A record that cannot be read proves nothing about the
///  process,
/// so no number is signalled that may name an unrelated one,
/// and one line says the step was not sent that
///  signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function signalRoot(pid: number, signal: number): void; // fresh look, pidfd, or a line saying nothing was sent
/// ```
#[cfg(unix)]
pub(crate) fn signal_root(raw: i32, signal: libc::c_int) {
    // The record is read here, at the signal, and not reused from the start of the stop.
    let mut look = || return look_of(raw);
    return signal_root_through(&mut look, raw, signal);
}

/// What:
///  `signal_root` over a look the caller supplies,
/// so a control can read a record that refuses without starving the whole test process.
/// Why:
///   The whole decision of a signal is what the look answered,
/// and the refusal (`ProcessLook::Unknown`) is a branch no
///  real `/proc` gives a control on demand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function signalRootThrough(look: () => ProcessLook, pid: number, signal: number): void;
/// ```
#[cfg(unix)]
pub(crate) fn signal_root_through(look: &mut dyn FnMut() -> ProcessLook, raw: i32, signal: libc::c_int) {
    match look() {
        ProcessLook::Alive(found) => {
            // The member is held by a pidfd where the kernel gives one, so a number reused between the look and the signal is
            //  never reached.
            let _delivered: Delivered = crate::pinned_member_signalled_through_a_pidfd::signal_member(&found, signal);
        }
        // A refusal is not an exit, and it is not a proof of which process the number names either: nothing is signalled,
        //  and the person is told the step was not stopped.
        ProcessLook::Unknown => crate::diagnostic_log::warn(
            "childWatch",
            format!(
                "the step {raw} was not sent {}: its record could not be read, so no number is signalled that may name another \
                 process. The step is still waited for; check with `ps` whether it runs and what it waits for.",
                signal_name(signal)
            )
            .as_str(),
        ),
        ProcessLook::Gone => {}
    }
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
    let mut tree: Vec<TreeMember> = match root_at_start.clone() {
        ProcessLook::Alive(found) => vec![found],
        ProcessLook::Gone | ProcessLook::Unknown => Vec::new(),
    };
    tree.extend(below.iter().cloned());
    // A process number can be reused once the waiting thread has reaped the child, and the child is reaped as soon as it
    // ends, which may be well before the grace is over (a helper below it can outlive it). The signals therefore go through a
    // descriptor taken from a look made at the signal itself, and never to a number alone; where the record could not be read
    // nothing is signalled (`signal_root`), because a refusal is not a proof of which process the number names.
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
    signal_root(raw, libc::SIGTERM);
    terminate_members(below.as_slice());
    let asked: Instant = Instant::now();
    while escalation_waits(asked.elapsed(), STOP_GRACE, tree.as_slice()) && the_stop_waits(tree.as_slice(), raw, &root_at_start) {
        tokio::time::sleep(STOP_POLL).await;
    }
    signal_root(raw, libc::SIGKILL);
    kill_members(below.as_slice());
    // The waiting thread reaps the child, so the watcher only looks. A step in an uninterruptible call ends when the kernel
    // lets it go, which can be long after the bound; the locks it creates on the way out are swept then, or when the watch
    // is dropped, whichever comes first.
    while the_stop_waits(tree.as_slice(), raw, &root_at_start) {
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
