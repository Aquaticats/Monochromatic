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
use crate::process_tree::{MemberRead, TreeMember, descendants_of, member_read, still_running};
use crate::member_signalled_through_its_pidfd::{Delivered, Pinned, PinnedMember, kill_members, terminate_members};
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
            Mark::End => self.end(),
        }
    }

    /// What:
    ///  Ends the wait the watcher holds,
    /// as the waiting thread's end mark does,
    /// and keeps the quiet gap it measured.
    /// Why:
    ///   A wait can end without a mark:
    /// the pin of the child reports it gone,
    /// a stop of the service ends it,
    ///  and the watcher itself ends it after a quiet window or a ceiling.
    /// Every one of those ends the same way,
    /// so the watcher
    ///  waits for the next mark instead of taking the same branch again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// marks.end(); // keepGap(); since = undefined; clock = undefined;
    /// ```
    pub(crate) fn end(&mut self) {
        self.keep_gap();
        self.since = None;
        self.clock = None;
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
///  The pin of a step's root,
/// taken when the child started,
/// while it was this process's own unreaped child.
/// Why:
///  A number is a safe address only while the process it names is this process's own unreaped child,
/// and the thread
///  that waits for the child reaps it as soon as it ends.
/// A pin taken at stopping time therefore pins whatever process
///  holds the number *then*:
/// the step can have ended and been reaped while its grace ran,
/// the number can
///  have passed to another command,
/// and that command would receive the stop
/// (the third review's finding 1).
///  Taken at the start,
/// the number certainly names the child,
/// and the pin names that child for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = pinAtStart(child.pid); // held only if the record read then named the child
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) type RootPin = Pinned;

/// Without a descriptor to take a pin with,
/// nothing is ever signalled to a step's root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
#[derive(Debug)]
pub(crate) struct RootPin;

/// What:
///  The pin of the child that has just started,
/// taken while its number is certainly its own.
/// Why:
///   The child is this process's own unreaped child right after the start,
/// so the record read here describes it and
///  no other process;
/// the descriptor that record is proven against then names that child for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = pinAtStart(child.pid);
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn pin_at_start(pid: u32) -> RootPin {
    let Ok(raw) = i32::try_from(pid) else {
        return Pinned::Unproven;
    };
    let MemberRead::Found { member, .. } = member_read(raw) else {
        return Pinned::Unproven;
    };
    return PinnedMember::open(&member);
}

/// Without `/proc` nothing is known about the child's number,
/// so no pin is made and nothing is signalled.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn pin_at_start(_pid: u32) -> RootPin {
    return RootPin;
}

/// What:
///  Send `signal` to the step's root through its pin,
/// and answer what the signal came to.
/// Why:
///   A record that could not be read proves nothing about the process,
/// and a number that was not pinned may
///  name an unrelated process by now,
/// so no number is signalled;
/// the step is still waited for,
/// and one line says it
///  was not stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sendToTheRoot(pinned: Pinned, pid: number, signal: number): Delivered;
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn send_to_the_root(pinned: &RootPin, raw: i32, signal: libc::c_int) -> Delivered {
    let delivered: Delivered = match pinned {
        Pinned::Held(member) => member.signal(signal),
        Pinned::Gone => Delivered::Gone,
        Pinned::Unproven => Delivered::Unproven,
    };
    if delivered == Delivered::Unproven {
        crate::diagnostic_log::warn(
            "childWatch",
            format!(
                "the step {raw} was not sent {}: nothing proved which process its number named, so no number was signalled. The \
                 step is still waited for; check with `ps` whether it runs and what it waits for.",
                signal_name(signal)
            )
            .as_str(),
        );
    }
    return delivered;
}

/// A platform without a descriptor cannot signal the step's root;
/// the wait it announces is all it has.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn send_to_the_root(_pinned: &RootPin, raw: i32, signal: libc::c_int) -> Delivered {
    crate::diagnostic_log::warn(
        "childWatch",
        format!(
            "the step {raw} was not sent {}: this platform gives no descriptor to name a process with, so no number is signalled. \
             Check with `ps` whether it runs and what it waits for.",
            signal_name(signal)
        )
        .as_str(),
    );
    return Delivered::Unproven;
}

/// What:
///  Whether the step's root is still there,
/// as the pin taken when the child started tells it.
/// Why:
///   The pin names the child for good,
/// so a pin that reports the process gone means the step itself ended;
/// the status its wait
///  collected is then the answer,
/// and nothing of a stop may replace it
/// (the fourth review's finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillThere = (pin) => pin.kind === 'held' || pin.kind === 'unproven';
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn the_root_is_still_there(pin: &RootPin) -> bool {
    return match pin {
        // The descriptor becomes readable when the process it names exits, which answers this without a signal.
        Pinned::Held(member) => !member.has_ended(),
        Pinned::Gone => false,
        // Nothing proved which process the number names, so the stop goes on as if the root were there: a bound the step keeps
        //  is never lost to a record that could not be read.
        Pinned::Unproven => true,
    };
}

/// What:
///  Whether the pin of a step's root proves which process to signal.
/// Why:
///   A stop signals the root only through the descriptor taken when the child started,
/// and a pin that proves nothing signals
///  nothing:
/// the watch then cannot end the step at its ceiling,
/// and the wait for the step's output is what does,
///  at the ceiling itself rather than after the watch has had its ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proven = (pin) => pin.kind === 'held';
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn the_root_is_proven(pin: &RootPin) -> bool {
    // A pin that reports the child gone names no process either, so nothing is signalled for it and the step is the wait's to end.
    return matches!(pin, Pinned::Held(_));
}

/// Without a descriptor nothing is proven,
/// so no watch can end the step it watches.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn the_root_is_proven(_pin: &RootPin) -> bool {
    return false;
}

/// What:
///  The processes below a step's root,
/// read only while its pin proves the root is still the process it names.
/// Why:
///  Once the root is reaped its number may name another process,
/// and the children of that process are not this
///  step's:
/// enumerating them would signal a command of the person's that has nothing to do with the stop
///  (the fourth review's finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = stillThere(pin) ? descendantsOf(pid) : [];
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn descendants_of_a_root_that_still_runs(pin: &RootPin, raw: i32) -> Vec<TreeMember> {
    if !the_root_is_still_there(pin) {
        return Vec::new();
    }
    return descendants_of(raw);
}

/// Without a descriptor nothing is known about the root,
/// so the stop goes on as if it were there.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn the_root_is_still_there(_pin: &RootPin) -> bool {
    return true;
}

/// Without `/proc` no process is listed,
/// so there is no tree to read.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub(crate) fn descendants_of_a_root_that_still_runs(_pin: &RootPin, _raw: i32) -> Vec<TreeMember> {
    return Vec::new();
}

/// What:
///  Whether the stop keeps waiting after it has killed what it could.
/// Why:
///   The wait after the kill ends with a grace of its own,
/// extended only while a member is in uninterruptible
///  sleep and up to the stop's ceiling,
/// as the wait before the kill does,
/// so nothing waits for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitAfterTheKill = (elapsed, tree) => escalationWaits(elapsed, grace, tree) && tree.some(stillRunning);
/// ```
#[cfg(unix)]
pub(crate) fn the_wait_after_the_kill(elapsed: Duration, tree: &[TreeMember]) -> bool {
    return escalation_waits(elapsed, STOP_GRACE, tree) && tree.iter().any(still_running);
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
///  Its root is pinned when the child starts,
/// while it is this process's own unreaped child,
/// so a signal reaches that
///  process and never one that took its number afterwards,
/// and every wait of the stop is bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(pid, 'SIGTERM'); await Promise.race([exited, delay(graceMs)]) || process.kill(pid, 'SIGKILL'); locks.sweep(tree.as_slice());
/// ```
#[cfg(unix)]
pub(crate) async fn stop_process(pid: u32, locks: &LockLedger, incoming: &mut UnboundedReceiver<Mark>, root_pin: &RootPin) {
    let Ok(raw) = i32::try_from(pid) else {
        return;
    };
    // The processes below the child are read before it is stopped, because a process's link to its parent is gone once the
    // parent has exited; a step's helpers (a transport, a hook) would otherwise outlive it and hold its output open, and the
    // wait for that output would outlast the bound by as long as they live.
    // Nothing about the tree is read once the pin says the root ended: the number may name another process by then, and the
    //  children of that process are not this step's (the fourth review's finding 1).
    let below: Vec<TreeMember> = descendants_of_a_root_that_still_runs(root_pin, raw);
    // Three answers, not two: a record that could not be read is not a process that has gone, so nothing is signalled for it
    //  and nothing is concluded either.
    let root: MemberRead = if the_root_is_still_there(root_pin) { member_read(raw) } else { MemberRead::Gone };
    let mut tree: Vec<TreeMember> = match &root {
        MemberRead::Found { member, .. } => vec![member.clone()],
        MemberRead::Gone | MemberRead::Unreadable => Vec::new(),
    };
    tree.extend(below.iter().cloned());
    // The signals go through the pin the watch took when this child started, not one taken from a number now: by this moment
    //  the child can have ended and been reaped, and the number can name another command (the third review's finding 1).
    // A process number can be reused once the waiting thread has reaped the child, and the child is reaped as soon as it
    // ends, which may be well before the grace is over (a helper below it can outlive it). Both signals therefore go through
    // the one pin taken when the child started, and never to a number read now: where that start proved nothing, nothing is
    // signalled, and one line says so.
    // Nothing is claimed before the signals: a number in a pid file does not say which PID namespace wrote it
    //  (cross-vendor review of 2026-11-09, finding 1).
    let claiming: LockLedger = locks.clone();
    let _asked_delivered: Delivered = send_to_the_root(root_pin, raw, libc::SIGTERM);
    let _below_delivered: Vec<Delivered> = terminate_members(below.as_slice());
    let asked: Instant = Instant::now();
    while escalation_waits(asked.elapsed(), STOP_GRACE, tree.as_slice()) && tree.iter().any(still_running) {
        tokio::time::sleep(STOP_POLL).await;
    }
    let _killed_delivered: Delivered = send_to_the_root(root_pin, raw, libc::SIGKILL);
    let _killed_below: Vec<Delivered> = kill_members(below.as_slice());
    // The waiting thread reaps the child, so the watcher only looks. A member in an uninterruptible call ends when the kernel
    // lets it go, which can be after the grace; that wait is bounded as the grace before the kill was, so a member whose record
    // cannot be read ends the stop here rather than keeping it waiting. The locks the tree leaves are swept then, or when the
    // watch is dropped, whichever comes first.
    let killed: Instant = Instant::now();
    while the_wait_after_the_kill(killed.elapsed(), tree.as_slice()) {
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
    if let Err(error) = crate::blocking_job::run_blocking_job(move || { sweeping.sweep(); }).await {
        crate::diagnostic_log::debug("childWatch", format!("the lock files of the stopped step {pid} were not swept: {error}").as_str());
    }
}

/// A platform without signals cannot end the child from the watcher;
/// the wait is only announced.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub(crate) async fn stop_process(_pid: u32, _locks: &LockLedger, _incoming: &mut UnboundedReceiver<Mark>, _root_pin: &RootPin) {}

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
    /// How long the command waits for a step it never ends before it stops waiting and leaves it running,
    /// in microseconds,
    /// or zero when it waits for as long as the step lives.
    /// The watcher says it,
    /// because that is the bound its notices name.
    pub(crate) left_running_after: Arc<AtomicU64>,
}
