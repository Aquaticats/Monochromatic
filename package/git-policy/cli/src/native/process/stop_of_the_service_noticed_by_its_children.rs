//! What:
//!  How a stop of the commit service reaches the Git processes it waits for:
//! the signal handler records the stop and writes one byte to a pipe,
//! a task of the service's runtime reads the byte and tells every waiter,
//! each waiter ends the one child it owns,
//! and the same task ends the process by the signal once the work in flight has settled or its bound has passed.
//! Why:
//!  The owner's pick for open choice 12 (2026-11-07T22:19:43-05:00, verbatim "C: shutdown stops its Gits (Recommended)"):
//! the stop handler only records the stop,
//! the service's own shutdown signals the Gits it waits on,
//! every wait after the signal is bounded and then `SIGKILL`,
//! and a commit whose Git is stopped keeps its work on its side ref.
//! The owner's rule "Use as few locks as possible." is why there is no table of children:
//! each wait owns its child and ends it,
//! and the shutdown only counts the work that has not finished.
//! Before this step a stop of the service killed the service at once and left the Git of a landing running,
//! and the lock the service held on the index stayed behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.on('SIGTERM', (signal) => { stopped.set(signal); wake(); });
//! // in the service's runtime: await wake(); stopped.broadcast(); await timeout(settleBound, unfinished.reachesZero()); process.kill(process.pid, signal);
//! // in each waiter: await Promise.race([child.exited, stopped.broadcasted]) // then end the child it owns
//! ```
//!
//! Limits:
//!  A process that never called `prepare_to_notice` (every command of the wrapper) counts nothing,
//! refuses nothing and hands out no arm,
//! so a stop of one of those is the relay's business (`signal_relay.rs`).

use std::io;
use std::os::fd::{FromRawFd, IntoRawFd, OwnedFd, RawFd};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::{Notify, watch};

/// How long the shutdown waits for the work in flight to settle after the stop was told:
/// the longest delay measured on this host times the owner's margin (`STOP_CEILING`),
/// the grace a stopped child gets before `SIGKILL` (`STOP_GRACE`),
/// and one second for the commit's own unwinding (the side ref that keeps its work and the lock it gives up).
/// A step in uninterruptible sleep is waited for up to `STOP_CEILING` before the kill (the owner's "3x worst case",
/// `stop_waits_while_a_member_cannot_be_interrupted.rs`),
/// so this bound is that wait,
/// the grace of the kill that follows it,
/// and the unwinding.
/// The service manager's own stop timeout is longer (ninety seconds unless the unit says otherwise;
/// the unit of this crate says nothing),
/// so the shutdown ends before the manager would kill the unit.
pub const SETTLE_CEILING: Duration = crate::stop_waits_while_a_member_cannot_be_interrupted::STOP_CEILING
    .saturating_add(crate::bounded_push::STOP_GRACE)
    .saturating_add(Duration::from_secs(1));

/// The seconds after the stop at which the kernel ends the process whatever the runtime is doing:
/// the settle bound and one more grace.
/// It is the last resort for a runtime whose workers are all blocked,
/// where no task could end the process,
/// and it is armed by the handler with `alarm`,
/// which `signal-safety(7)` lists.
pub const LAST_RESORT: Duration = SETTLE_CEILING.saturating_add(crate::bounded_push::STOP_GRACE);

/// The signal that began the stop,
/// or zero while none has.
static STOP_SIGNAL: AtomicI32 = AtomicI32::new(0);

/// Whether this process is a service that notices a stop.
static NOTICING: AtomicBool = AtomicBool::new(false);

/// The write end of the pipe the handler writes one byte to,
/// or -1.
static WAKE_WRITE: AtomicI32 = AtomicI32::new(-1);

/// The read end of the pipe,
/// until the shutdown task takes it,
/// or -1.
static WAKE_READ: AtomicI32 = AtomicI32::new(-1);

/// How many pieces of work the shutdown waits for:
/// the children being waited on and the groups being landed.
static UNFINISHED: AtomicUsize = AtomicUsize::new(0);

/// Told whenever a piece of work finishes,
/// so the shutdown looks at the count again.
static ONE_FINISHED: Notify = Notify::const_new();

/// The level every waiter reads:
/// true from the moment the shutdown task has told the stop.
static TOLD: OnceLock<watch::Sender<bool>> = OnceLock::new();

/// The sender of `TOLD`,
/// made on first use.
fn told() -> &'static watch::Sender<bool> {
    return TOLD.get_or_init(new_level);
}

/// A level that starts false.
fn new_level() -> watch::Sender<bool> {
    return watch::channel(false).0;
}

/// Whether a value is true,
/// the test a waiter's `wait_for` applies.
fn is_true(value: &bool) -> bool {
    return *value;
}

/// What:
///  Makes the pipe the handler wakes the shutdown through,
/// and turns the process into one that notices a stop.
/// Why:
///  The handler may call only what `signal-safety(7)` lists,
/// and a `write` to a non-blocking pipe is on the list;
/// a task of the runtime reads the other end,
/// where a lock-free wake of a task from a handler is not possible.
/// Both ends do not block,
/// so a full pipe loses a byte after the first one that already wakes the task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = pipe({ nonBlocking: true }); noticing = true;
/// ```
pub fn prepare_to_notice() -> io::Result<()> {
    if NOTICING.load(Ordering::SeqCst) {
        return Ok(());
    }
    let (reader, writer): (std::io::PipeReader, std::io::PipeWriter) = std::io::pipe()?;
    let read_end: OwnedFd = OwnedFd::from(reader);
    let write_end: OwnedFd = OwnedFd::from(writer);
    make_non_blocking(&read_end)?;
    make_non_blocking(&write_end)?;
    WAKE_READ.store(read_end.into_raw_fd(), Ordering::SeqCst);
    WAKE_WRITE.store(write_end.into_raw_fd(), Ordering::SeqCst);
    NOTICING.store(true, Ordering::SeqCst);
    return Ok(());
}

/// Sets `O_NONBLOCK` on a descriptor.
fn make_non_blocking(descriptor: &OwnedFd) -> io::Result<()> {
    let raw: RawFd = std::os::fd::AsRawFd::as_raw_fd(descriptor);
    // SAFETY: `fcntl` with `F_GETFL` reads the flags of a descriptor this function borrows for the call.
    let flags: libc::c_int = unsafe { libc::fcntl(raw, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `fcntl` with `F_SETFL` sets the flags of the same open descriptor.
    let set: libc::c_int = unsafe { libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    if set < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Records `signal` as the stop,
/// and says whether it was the first.
/// Why:
///  Called by the signal handler,
/// so it only touches an atomic.
/// A second stop signal is the operator insisting,
/// and the handler ends the process at once for it.
/// Work started after this call sees the stop (`begin_unfinished_work`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = stopSignal.compareExchange(0, signal);
/// ```
pub fn record_from_a_signal_handler(signal: libc::c_int) -> bool {
    return STOP_SIGNAL.compare_exchange(0, signal, Ordering::SeqCst, Ordering::SeqCst).is_ok();
}

/// What:
///  Wakes the shutdown task.
/// Why:
///  Called by the signal handler after its last line is written,
/// so the shutdown can never end the process before the line is out.
/// Returns false when there is nothing to wake,
/// so the handler ends the process as it did before this step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!wakePipe) return false; wakePipe.write(1); return true;
/// ```
pub fn wake_the_shutdown_from_a_signal_handler() -> bool {
    let descriptor: RawFd = WAKE_WRITE.load(Ordering::SeqCst);
    if descriptor < 0 {
        return false;
    }
    let byte: u8 = 1;
    // SAFETY: `write` is on `signal-safety(7)`'s list; the buffer is one valid byte for the call, and a failed or short write is no
    //  reason to keep the handler from arming the bound.
    let _written: isize = unsafe { libc::write(descriptor, std::ptr::from_ref(&byte).cast(), 1) };
    return true;
}

/// What:
///  Gives the kernel the last-resort bound:
/// `SIGALRM` ends the process `LAST_RESORT` after this call.
/// Why:
///  A runtime whose workers are all blocked runs no task,
/// so nothing else would end a service that was told to stop.
/// `alarm` is on `signal-safety(7)`'s list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// alarm(LAST_RESORT_SECONDS);
/// ```
pub fn arm_the_last_resort_from_a_signal_handler() {
    // SAFETY: `alarm` takes a number of seconds and touches no memory.
    let _previous: libc::c_uint = unsafe { libc::alarm(u32::try_from(LAST_RESORT.as_secs()).unwrap_or(u32::MAX)) };
}

/// The signal that began the stop,
/// or nothing while none has.
pub fn stopping_signal() -> Option<libc::c_int> {
    return match STOP_SIGNAL.load(Ordering::SeqCst) {
        0 => None,
        signal => Some(signal),
    };
}

/// Whether a stop has been recorded.
pub fn is_stopping() -> bool {
    return STOP_SIGNAL.load(Ordering::SeqCst) != 0;
}

/// What:
///  Ends this process by the signal that began the stop,
/// as it would have ended without a handler.
/// Why:
///  The manager and the person read the way the service ended:
/// "killed by SIGTERM" is a stop,
/// and an exit code of zero would say the service chose to leave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.removeAllListeners(signal); process.kill(process.pid, signal);
/// ```
pub fn end_by_the_signal(signal: libc::c_int) {
    // SAFETY: restoring the default disposition and raising the signal read no memory of this process.
    unsafe {
        libc::signal(signal, libc::SIG_DFL);
        libc::raise(signal);
    }
    // A signal that did not end the process (blocked by something unusual) is followed by a plain exit with the shell's code for it.
    std::process::exit(128 + signal);
}

/// What:
///  What a waiter reads to learn that the stop was told.
/// Why:
///  A waiter awaits it beside its child's end and then ends the child it owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([child.exited, arm.stopTold()]);
/// ```
#[derive(Debug)]
pub struct StopArm {
    /// The level the shutdown task raises.
    level: watch::Receiver<bool>,
}

impl StopArm {
    /// Resolves when the stop was told;
    /// resolves at once when it already was.
    pub async fn stop_told(&mut self) {
        // The sender lives in a static and is never dropped, so an error would mean a defect and a wait that ends at once is the safe side.
        let _told: Result<watch::Ref<'_, bool>, watch::error::RecvError> = self.level.wait_for(is_true).await;
    }
}

/// What:
///  One piece of work the shutdown waits for,
/// counted from `begin_unfinished_work` to its drop,
/// and the arm of the child it owns.
/// Why:
///  The count is all the shutdown needs;
/// the arm is how the owner of a child learns to end it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using work = beginUnfinishedWork(); await Promise.race([child.exited, work.arm.stopTold()]);
/// ```
#[derive(Debug)]
pub struct Unfinished {
    /// What ends the child this work owns when the stop is told;
    /// nothing for work that began after the stop,
    /// which is allowed to finish.
    arm: Option<StopArm>,
    /// Whether this work is in the count.
    counted: bool,
}

impl Unfinished {
    /// Hands over the arm,
    /// once.
    pub fn take_arm(&mut self) -> Option<StopArm> {
        return self.arm.take();
    }
}

impl Drop for Unfinished {
    fn drop(&mut self) {
        if self.counted {
            UNFINISHED.fetch_sub(1, Ordering::SeqCst);
            ONE_FINISHED.notify_waiters();
        }
    }
}

/// The error of work that was not started because the service is stopping.
#[derive(Debug)]
pub struct RefusedWhileStopping(
    /// What was not started,
    /// in words for the person.
    pub String,
);

impl std::fmt::Display for RefusedWhileStopping {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "{} was not started because the service is stopping", self.0);
    }
}

impl std::error::Error for RefusedWhileStopping {}

/// What:
///  Counts one piece of work the shutdown waits for,
/// and refuses it when the service is already stopping and the work is not one that must finish.
/// Why:
///  The count is raised before the stop is read,
/// so either this call sees the stop or the shutdown sees the count:
/// no work starts unseen between the two.
/// Work that began before the stop gets an arm,
/// and work that must finish after it (the landing's own unwinding:
/// the side ref that keeps a stopped commit,
/// the lock it gives up) does not,
/// so the stop does not end the steps that undo what the stop cut.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unfinished += 1; if (stopping && !mustFinish) { unfinished -= 1; throw new RefusedWhileStopping(what); }
/// ```
pub fn begin_unfinished_work(allowed_after_the_stop: bool, what: &str) -> Result<Unfinished, RefusedWhileStopping> {
    if !NOTICING.load(Ordering::SeqCst) {
        return Ok(Unfinished { arm: None, counted: false });
    }
    UNFINISHED.fetch_add(1, Ordering::SeqCst);
    let stopping: bool = is_stopping();
    let arm: Option<StopArm> = if stopping { None } else { Some(StopArm { level: told().subscribe() }) };
    // Dropped on the refusal below, which gives the count back.
    let work: Unfinished = Unfinished { arm, counted: true };
    if stopping && !allowed_after_the_stop {
        return Err(RefusedWhileStopping(String::from(what)));
    }
    return Ok(work);
}

/// How many pieces of work are unfinished.
pub fn unfinished_work() -> usize {
    return UNFINISHED.load(Ordering::SeqCst);
}

/// Resolves when no work is unfinished.
async fn all_work_settled() {
    loop {
        let finished = ONE_FINISHED.notified();
        tokio::pin!(finished);
        // Registered before the count is read, so a finish between the read and the wait is not missed.
        finished.as_mut().enable();
        if UNFINISHED.load(Ordering::SeqCst) == 0 {
            return;
        }
        finished.await;
    }
}

/// How the shutdown ended its wait.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Settled {
    /// No work was unfinished.
    Completely,
    /// The bound passed with work unfinished.
    AtTheBound,
}

/// What:
///  Waits for the wake of the handler,
/// tells every waiter,
/// waits for the work in flight to settle for at most `bound`,
/// and says how the wait ended.
/// Why:
///  This is the service's own shutdown:
/// the handler did nothing but record,
/// and everything that waits is here and bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await wake.read(); stopped.broadcast(); return await timeout(bound, unfinished.reachesZero()) ? 'completely' : 'at the bound';
/// ```
pub async fn tell_the_stop_and_wait_for_the_work(bound: Duration) -> Settled {
    wait_for_the_wake().await;
    told().send_replace(true);
    return match tokio::time::timeout(bound, all_work_settled()).await {
        Ok(()) => Settled::Completely,
        Err(_elapsed) => Settled::AtTheBound,
    };
}

/// Waits until the handler has written its byte.
/// A pipe that cannot be read counts as woken:
/// a shutdown that never starts is worse than one that starts early.
async fn wait_for_the_wake() {
    let raw: RawFd = WAKE_READ.swap(-1, Ordering::SeqCst);
    if raw < 0 {
        return;
    }
    // SAFETY: the descriptor was made by `prepare_to_notice`, is open, and this swap handed it to this call alone.
    let owned: OwnedFd = unsafe { OwnedFd::from_raw_fd(raw) };
    let Ok(reader) = tokio::net::unix::pipe::Receiver::from_owned_fd(owned) else {
        return;
    };
    let mut byte: [u8; 1] = [0];
    loop {
        if reader.readable().await.is_err() {
            return;
        }
        match reader.try_read(&mut byte) {
            Ok(_read) => return,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(_unreadable) => return,
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "stop_of_the_service_noticed_by_its_children_tests.rs"]
mod tests;
