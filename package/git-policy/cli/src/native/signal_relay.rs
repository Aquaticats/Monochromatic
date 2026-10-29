//! What:
//!  Catching `SIGINT`, `SIGTERM`, `SIGHUP` and `SIGQUIT` while the wrapper waits for a Git child,
//!  relaying a signal to that child only when a process sent it,
//!  and reporting how the child ended.
//!  `SignalGuard` owns the handlers for its lifetime and restores the previous dispositions on drop.
//! Why:
//!  A wrapper that dies on a signal leaves its commit transaction for recovery and Git running;
//!  one that catches the signal can end Git, release its locks and remove the transaction first.
//!  A signal from the terminal already reached Git as part of the foreground process group,
//!  so relaying it again would deliver it twice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.once('SIGTERM', () => { if (!fromTerminal) child.kill('SIGTERM'); });
//! ```
//!
//! Integration (for the commit path and wrapped commands):
//! ```ts
//! // const guard = SignalGuard.install(); const outcome = guard.runChild(command); // cleanup; then exit(exitCode(outcome))
//! ```
//!
//! Limits:
//!  Handlers are process-wide, so one guard may be active at a time;
//!  `SA_RESTART` keeps other system calls from failing with `EINTR`.
//!  Only Linux reads the child's exit without reaping it, so the PID is cleared before reaping there;
//!  elsewhere a signal that arrives in the instant between the child's exit and its reaping
//!  could reach a recycled PID.

/// What:
///  The typed reason a second signal guard is refused while one is active.
/// Why:
///   Callers that wait for the guard to be free must recognize this refusal by its type, not by its wording
///  (issue 1346).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SignalGuardActiveError extends Error {}
/// ```
#[derive(Debug)]
pub struct SignalGuardActive;

/// The wording of the refusal.
impl std::fmt::Display for SignalGuardActive {
    /// Writes the wording of the refusal.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str("a signal guard is already active in this process");
    }
}

impl std::error::Error for SignalGuardActive {}

/// What:
///  Whether an installation failure is the refusal of a second guard.
/// Why:
///   Only this refusal clears by itself when the other holder lets go; every other failure is final.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error instanceof SignalGuardActiveError
/// ```
pub fn is_guard_active_refusal(error: &io::Error) -> bool {
    return error
        .get_ref()
        .is_some_and(|inner| { return inner.is::<SignalGuardActive>(); });
}

/// `c_int` is the C integer type signal numbers and status codes use.
#[cfg(unix)]
use std::ffi::c_int;
/// `io` is the input and output error type the spawn and wait calls return.
use std::io;
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// `Command` builds the child, and `ExitStatus` is how it ended.
use std::process::{Command, ExitStatus};

/// The Git child's end, with the exit-code mapping the wrapper's exit uses.
use super::forwarding::ChildOutcome;

/// What:
///  Whether the kernel recorded a signal as sent by a process rather than by the terminal or the kernel.
/// Why:
///   `si_code` values `SI_USER`, `SI_QUEUE` and (Linux) `SI_TKILL` come from `kill`, `sigqueue` and `tkill`;
///  a terminal interrupt carries `SI_KERNEL`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fromProcess = info.si_code === SI_USER || info.si_code === SI_QUEUE;
/// ```
#[cfg(unix)]
pub fn is_process_sender(code: c_int) -> bool {
    return PROCESS_SENDER_CODES.contains(&code);
}

/// Sender codes that mean a process sent the signal, on Linux.
#[cfg(target_os = "linux")]
const PROCESS_SENDER_CODES: [c_int; 3] = [libc::SI_USER, libc::SI_QUEUE, libc::SI_TKILL];

/// Sender codes that mean a process sent the signal, on other Unix systems.
#[cfg(all(unix, not(target_os = "linux")))]
const PROCESS_SENDER_CODES: [c_int; 2] = [libc::SI_USER, libc::SI_QUEUE];

/// What:
///  The signals the guard catches and relays.
/// Why:
///   The four signals that end an interactive or service wrapper in the incumbent's contract.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CAUGHT = ['SIGINT', 'SIGTERM', 'SIGHUP', 'SIGQUIT'] as const;
/// ```
#[cfg(unix)]
pub const RELAYED_SIGNALS: [c_int; 4] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT];

/// What:
///  PID of the Git child being waited for, or zero when none is.
/// Why:
///   The handler runs asynchronously, so it can only read state through atomics.
#[cfg(unix)]
static CHILD_PID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// What:
///  One slot per relayed signal, holding a signal number that has not been relayed yet, or zero.
/// Why:
///   A signal that arrives before the child exists is relayed as soon as its PID is recorded.
#[cfg(unix)]
static PENDING: [std::sync::atomic::AtomicI32; 4] =
    [const { std::sync::atomic::AtomicI32::new(0) }; 4];

/// What:
///  Whether a guard is installed in this process.
/// Why:
///   Handlers and their saved dispositions are process-wide, so two guards would overwrite each other's record.
#[cfg(unix)]
static GUARD_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  The first three fields of `siginfo_t` on every Unix system: signal number, error number and code.
/// Why:
///   The C struct's fields differ in name and union layout across systems; these three share one layout,
///  and reading them through this view avoids the platform accessors.
#[cfg(unix)]
#[repr(C)]
struct SignalHeader {
    /// Signal number.
    signal: c_int,
    /// Error number, unused here.
    error: c_int,
    /// Sender code, `SI_USER` for `kill` and so on.
    code: c_int,
}

/// What:
///  The handler for each relayed signal.
/// Why:
///   Relaying inside the handler delivers the signal at once, even while the wrapper is blocked in a wait,
///  where a flag would not wake it;
///  `kill` is async-signal-safe, and the handler touches only atomics and `kill`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.on('SIGTERM', (info) => { if (info.fromProcess) child.kill('SIGTERM'); });
/// ```
#[cfg(unix)]
extern "C" fn relay_handler(
    signal: c_int,
    info: *mut libc::siginfo_t,
    _context: *mut std::ffi::c_void,
) {
    if info.is_null() {
        return;
    }
    // SAFETY: with `SA_SIGINFO` the kernel passes a valid `siginfo_t`; its first three fields are the header.
    let code: c_int = unsafe { (*info.cast::<SignalHeader>()).code };
    if !is_process_sender(code) {
        return;
    }
    let Some(slot) = RELAYED_SIGNALS.iter().position(|candidate| return *candidate == signal) else {
        return;
    };
    PENDING[slot].store(signal, std::sync::atomic::Ordering::SeqCst);
    relay_pending();
}

/// What:
///  Send every pending signal to the recorded child, claiming each slot so it is sent once.
/// Why:
///   Both the handler and the waiting code call this; the atomic swap decides which one sends a given signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const signal of pending.drain()) child.kill(signal);
/// ```
#[cfg(unix)]
fn relay_pending() {
    let pid: i32 = CHILD_PID.load(std::sync::atomic::Ordering::SeqCst);
    if pid <= 0 {
        return;
    }
    for slot in &PENDING {
        let signal: c_int = slot.swap(0, std::sync::atomic::Ordering::SeqCst);
        if signal != 0 {
            // SAFETY: `kill` sends one signal to a PID this module recorded; a failure is ignored because the child may have ended.
            unsafe { libc::kill(pid, signal) };
        }
    }
}

/// What:
///  Install one relaying handler, returning the disposition it replaced.
///  A signal the parent ignores stays ignored, so `None` is returned and nothing is saved.
/// Why:
///   Git inherits ignored signals, so replacing an ignore with a handler would change Git's environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.listenerCount(signal) === 0) install(signal);
/// ```
#[cfg(unix)]
fn install_one(signal: c_int) -> io::Result<Option<libc::sigaction>> {
    // SAFETY: a zeroed `sigaction` is a valid starting value; every field used below is set first.
    let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
    // SAFETY: as above; `previous` receives the current disposition.
    let mut previous: libc::sigaction = unsafe { std::mem::zeroed() };
    // SAFETY: a null new action only reads the current disposition into `previous`.
    let read: c_int = unsafe { libc::sigaction(signal, std::ptr::null(), &mut previous) };
    if read != 0 {
        return Err(io::Error::last_os_error());
    }
    if previous.sa_sigaction == libc::SIG_IGN {
        return Ok(None);
    }
    action.sa_sigaction = relay_handler as *const () as libc::sighandler_t;
    action.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
    // SAFETY: `sigemptyset` initializes the mask in place.
    unsafe { libc::sigemptyset(&mut action.sa_mask) };
    // SAFETY: `action` is fully initialized; the old disposition is written to `previous`.
    let set: c_int = unsafe { libc::sigaction(signal, &action, &mut previous) };
    if set != 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(Some(previous));
}

/// What:
///  Put back each saved disposition.
/// Why:
///   Restoring is the drop's job and also the rollback when an installation fails midway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [signal, previous] of saved) process.removeListener(signal, previous);
/// ```
#[cfg(unix)]
fn restore(saved: &[(c_int, libc::sigaction)]) {
    for (signal, previous) in saved {
        // SAFETY: `previous` is a disposition this module read earlier; restoring it reads no memory.
        unsafe { libc::sigaction(*signal, previous, std::ptr::null_mut()) };
    }
}

/// What:
///  Wait for the child to exit without reaping it, so its PID still names it.
/// Why:
///   Clearing the recorded PID before reaping stops a relay to a PID that the system may have reused.
///  `waitid` with `WNOWAIT` reports the exit and leaves the child waitable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForExitWithoutReaping(pid);
/// ```
#[cfg(target_os = "linux")]
fn wait_without_reaping(pid: libc::pid_t) -> io::Result<()> {
    loop {
        // SAFETY: `waitid` writes into the zeroed buffer this function owns, and only for a child it names.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        // SAFETY: `P_PID` names one child; `WEXITED | WNOWAIT` only observes its exit.
        let result: c_int = unsafe {
            libc::waitid(
                libc::P_PID,
                libc::id_t::try_from(pid).unwrap_or(0),
                &mut info,
                libc::WEXITED | libc::WNOWAIT,
            )
        };
        if result == 0 {
            return Ok(());
        }
        let error: io::Error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::EINTR) => continue,
            Some(libc::ECHILD) => return Ok(()),
            _ => return Err(error),
        }
    }
}

/// What:
///  Non-Linux Unix placeholder for the wait that leaves the child unreaped.
/// Why:
///   `waitid` is not in the `libc` crate for these systems; the window it closes is documented on the module.
#[cfg(all(unix, not(target_os = "linux")))]
fn wait_without_reaping(_pid: libc::pid_t) -> io::Result<()> {
    return Ok(());
}

/// What:
///  Owns the relaying handlers for its lifetime.
///  Install one guard, run the Git child through it, then drop it after cleanup.
/// Why:
///   Dropping it restores the previous dispositions, so a signal after cleanup behaves as it did before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using guard = SignalGuard.install(); const outcome = await guard.runChild(command);
/// ```
#[cfg(unix)]
#[must_use = "the handlers last only while the guard lives"]
pub struct SignalGuard {
    /// Dispositions to restore on drop, one per signal that was replaced.
    saved: Vec<(c_int, libc::sigaction)>,
}

/// Methods over an installed guard.
#[cfg(unix)]
impl SignalGuard {
    /// What:
    ///  Install the relaying handlers.
    ///  Fails when another guard is already active in this process.
    /// Why:
    ///   The handlers are process-wide, so a second guard would record the first one's handler as the old disposition.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static install(): SignalGuard // throws when already installed
    /// ```
    pub fn install() -> io::Result<Self> {
        if GUARD_ACTIVE
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_err()
        {
            return Err(io::Error::other(SignalGuardActive));
        }
        // `mut` lets each installed disposition be recorded as it is replaced.
        let mut saved: Vec<(c_int, libc::sigaction)> = Vec::with_capacity(RELAYED_SIGNALS.len());
        for signal in RELAYED_SIGNALS {
            match install_one(signal) {
                Ok(Some(previous)) => saved.push((signal, previous)),
                Ok(None) => {}
                Err(error) => {
                    restore(&saved);
                    GUARD_ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);
                    return Err(error);
                }
            }
        }
        return Ok(Self { saved });
    }

    /// What:
    ///  Start `command`, wait for it while relaying signals, and return how it ended.
    ///  The guard does not exit the process; the caller releases its locks and removes its state, then exits.
    /// Why:
    ///   Signals that arrived before the child started are relayed as soon as it has a PID.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async runChild(command: SpawnOptions): Promise<ChildOutcome>
    /// ```
    pub fn run_child(&self, command: &mut Command) -> io::Result<ChildOutcome> {
        return run_relayed(command);
    }
}

/// Spawns the child, then relays any signal that was caught before the child existed.
///
/// A handler that runs before the spawn keeps its signal in `PENDING` and finds no child.
/// The PID is published before the drain, so that signal is relayed here, not lost.
#[cfg(unix)]
fn run_relayed(command: &mut Command) -> io::Result<ChildOutcome> {
    let mut child: std::process::Child = command.spawn_retrying()?;
    let pid: libc::pid_t = libc::pid_t::try_from(child.id())
        .map_err(|error| return io::Error::other(error))?;
    CHILD_PID.store(pid, std::sync::atomic::Ordering::SeqCst);
    relay_pending();
    let waited: io::Result<()> = wait_without_reaping(pid);
    CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);
    let status: ExitStatus = child.wait()?;
    waited?;
    return Ok(super::forwarding::outcome_of(status));
}

/// Restore the previous dispositions and release the process-wide slot.
#[cfg(unix)]
impl Drop for SignalGuard {
    /// What:
    ///  Restore the saved dispositions and clear the active flag.
    /// Why:
    ///   The next guard, or the process default after the guard, must see the original handlers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { restore(saved); }
    /// ```
    fn drop(&mut self) {
        restore(&self.saved);
        GUARD_ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// What:
///  Non-Unix guard: no signal relay exists, so the child runs and the wrapper waits.
/// Why:
///   Callers keep one API; Windows ends a console child differently and is not covered by these controls.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub struct SignalGuard;

/// Methods over the non-Unix guard.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
impl SignalGuard {
    /// What:
    ///  Return a guard that relays nothing.
    /// Why:
    ///   Keeps the call site identical on every platform.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static install(): SignalGuard
    /// ```
    pub fn install() -> io::Result<Self> {
        return Ok(Self);
    }

    /// What:
    ///  Start `command` and wait for it.
    /// Why:
    ///   Without relaying, the wait is the whole job.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async runChild(command): Promise<ChildOutcome>
    /// ```
    pub fn run_child(&self, command: &mut Command) -> io::Result<ChildOutcome> {
        let mut child: std::process::Child = command.spawn_retrying()?;
        let status: ExitStatus = child.wait()?;
        return Ok(super::forwarding::outcome_of(status));
    }
}

/// Signal controls live beside the guard.
#[cfg(test)]
#[path = "signal_relay_tests.rs"]
mod tests;
