//! What:
//!  Catching `SIGINT`,
//! `SIGTERM`,
//! `SIGHUP` and `SIGQUIT` while the wrapper waits for a Git child,
//!  relaying a signal to that child only when a process sent it,
//!  and reporting how the child ended.
//!  `SignalGuard` owns the handlers for its lifetime and restores the previous dispositions on drop.
//! Why:
//!  A wrapper that dies on a signal leaves its commit transaction for recovery and Git running;
//!  one that catches the signal can end Git,
//! release its locks and remove the transaction first.
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
//!  Handlers are process-wide,
//! so the first guard of a process owns them and relays to its own child;
//! a guard made while
//!  another owns them,
//! or in a process whose service owns its stop signals,
//! relays nothing and refuses nothing
//!  (owner,
//! 2026-11-07,
//! "Registry of children" with "Only that operation's children":
//! a signal reaches only the children of
//!  the operation it belongs to,
//! and a stop of the service is an operation of its own).
//!  `SA_RESTART` keeps other system calls from failing with `EINTR`.
//!  On Linux the relay signals the child through a pidfd that stands on one descriptor number kept for the life of the
//!  process,
//! so a signal after the child was reaped finds nothing,
//! and a recycled PID is never reached.
//!  Elsewhere a signal that arrives in the instant between the child's exit and its reaping
//!  could reach a recycled PID.

/// `c_int` is the C integer type signal numbers and status codes use.
#[cfg(unix)]
use std::ffi::c_int;
/// `io` is the input and output error type the spawn and wait calls return.
use std::io;
/// The start of the child,
/// pausing on the runtime's timer when the system refuses a process.
use crate::start_retry::spawn_retrying_without_blocking;
/// The process's runtime,
/// which the synchronous face enters.
use crate::lazy_runtime::run_on_runtime;
/// A child owned until it is reaped,
/// awaited through its pidfd.
use crate::child_awaited_through_a_pidfd::adopt;
/// `Command` builds the child,
/// and `ExitStatus` is how it ended.
use std::process::{Command, ExitStatus};

/// The Git child's end,
/// with the exit-code mapping the wrapper's exit uses.
use crate::forwarding::ChildOutcome;
/// What:
///  The publication of the child to the handler,
/// the pending signal that arrived before it existed,
/// and the sending of a
///  caught signal through the one descriptor number.
/// Why:
///   A file of its own keeps this one under its line limit;
/// the tests below read the names through these re-exports.
#[cfg(unix)]
use crate::signal_relay_to_the_child::{PENDING, Publisher, publish, relay_pending, retire};
/// Whether a child is published to the handler,
/// for the controls of the relay.
#[cfg(all(test, unix))]
use crate::signal_relay_to_the_child::CHILD_PUBLISHED;
/// The descriptor number and its making,
/// for the controls of the relay.
#[cfg(all(test, target_os = "linux"))]
use crate::signal_relay_to_the_child::{RELAY_FD, ensure_relay_descriptor};

/// What:
///  Whether the kernel recorded a signal as sent by a process rather than by the terminal or the kernel.
/// Why:
///   `si_code` values `SI_USER`,
/// `SI_QUEUE` and (Linux) `SI_TKILL` come from `kill`,
/// `sigqueue` and `tkill`;
///  a terminal interrupt carries `SI_KERNEL`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fromProcess = info.si_code === SI_USER || info.si_code === SI_QUEUE;
/// ```
#[cfg(all(unix, not(target_os = "macos")))]
pub fn is_process_sender(code: c_int) -> bool {
    return PROCESS_SENDER_CODES.contains(&code);
}

/// What:
///  Whether a signal counts as sent by a process;
/// on macOS every signal does.
/// Why:
///   The macOS kernel reports `si_code` 0 for `kill`,
/// for a signal from a forked child and for a Ctrl-C written to a
///  pseudo-terminal master (measured on Darwin arm64),
/// so the code cannot tell a terminal sender from a process one.
///  Relaying all of them is the safe side:
/// a missed relay orphans the Git child,
///  while a second `SIGINT`,
/// `SIGTERM`,
/// `SIGHUP` or `SIGQUIT` reaching Git is harmless.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fromProcess = (_siCode: number) => true;
/// ```
#[cfg(target_os = "macos")]
pub fn is_process_sender(_code: c_int) -> bool {
    return true;
}

/// Sender codes that mean a process sent the signal,
/// on Linux.
#[cfg(target_os = "linux")]
const PROCESS_SENDER_CODES: [c_int; 3] = [libc::SI_USER, libc::SI_QUEUE, libc::SI_TKILL];

/// Sender codes that mean a process sent the signal,
/// on other Unix systems.
#[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
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
///  Whether a service owns this process's stop signals,
/// so no guard relays them.
/// Why:
///   A stop of the service is an operation of its own (owner,
/// 2026-11-07):
/// it must not be taken for the signal of a commit.
#[cfg(unix)]
static SERVICE_OWNS_SIGNALS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  Whether a guard owns the handlers of this process.
/// Why:
///   Handlers and their saved dispositions are process-wide,
/// so two guards would overwrite each other's record:
/// the first owns them,
///  and a guard made meanwhile relays nothing.
#[cfg(unix)]
static RELAYING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  The first three fields of `siginfo_t` on every Unix system:
/// signal number,
/// error number and code.
/// Why:
///   The C struct's fields differ in name and union layout across systems;
/// these three share one layout,
///  and reading them through this view avoids the platform accessors.
#[cfg(unix)]
#[repr(C)]
struct SignalHeader {
    /// Signal number.
    signal: c_int,
    /// Error number,
    /// unused here.
    error: c_int,
    /// Sender code,
    /// `SI_USER` for `kill` and so on.
    code: c_int,
}

/// What:
///  The handler for each relayed signal.
/// Why:
///   Relaying inside the handler delivers the signal at once,
/// even while the wrapper is blocked in a wait,
///  where a flag would not wake it;
///  `kill` is async-signal-safe,
/// and the handler touches only atomics and `kill`.
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
    // A plain loop, because this is a signal handler: it must not allocate or call anything that is not async-signal-safe.
    let mut found: Option<usize> = None;
    for (index, candidate) in RELAYED_SIGNALS.iter().enumerate() {
        if *candidate == signal {
            found = Some(index);
            break;
        }
    }
    let Some(slot) = found else {
        return;
    };
    PENDING[slot].store(signal, std::sync::atomic::Ordering::SeqCst);
    relay_pending();
}

/// What:
///  Install one relaying handler,
/// returning the disposition it replaced.
///  A signal the parent ignores stays ignored,
/// so `None` is returned and nothing is saved.
/// Why:
///   Git inherits ignored signals,
/// so replacing an ignore with a handler would change Git's environment.
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
///  Owns the relaying handlers for its lifetime when it is the first guard of the process;
/// otherwise it relays nothing.
///  Install one guard,
/// run the Git child through it,
/// then drop it after cleanup.
/// Why:
///   Dropping the owning guard restores the previous dispositions,
/// so a signal after cleanup behaves as it did before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using guard = SignalGuard.install(); const outcome = await guard.runChild(command);
/// ```
#[cfg(unix)]
#[must_use = "the handlers last only while the guard lives"]
pub struct SignalGuard {
    /// Dispositions to restore on drop,
    /// one per signal that was replaced;
    /// empty for a guard that owns no handlers.
    saved: Vec<(c_int, libc::sigaction)>,
    /// Whether this guard owns the process's handlers and so relays to its child.
    relaying: bool,
}

/// What:
///  Records that a service owns this process's stop signals,
/// so no guard installs handlers or relays from here on.
/// Why:
///   The owner's rule (2026-11-07,
/// "Only that operation's children"):
/// a stop of the service is an operation of its own and does
///  not reach the Git child of a commit that the service happens to be preparing through this relay;
/// the service's own shutdown ends the Gits it waits on (owner,
/// 2026-11-07T22:19:43-05:00,
/// "C:
/// shutdown stops its Gits (Recommended)";
/// `stop_of_the_service_noticed_by_its_children.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leaveSignalsToTheService();
/// ```
#[cfg(unix)]
pub fn leave_signals_to_the_service() {
    SERVICE_OWNS_SIGNALS.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Methods over an installed guard.
#[cfg(unix)]
impl SignalGuard {
    /// What:
    ///  Install the relaying handlers when no other guard owns them and no service owns the stop signals;
    /// otherwise make a guard
    ///  that relays nothing.
    /// It does not fail for another guard's presence.
    /// Why:
    ///   The handlers are process-wide,
    /// so a second owner would record the first one's handler as the old disposition.
    /// Refusing or
    ///  waiting for the first is the loop the registry replaces:
    /// the second operation simply has no process signal of its own.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static install(): SignalGuard // never waits for another guard
    /// ```
    pub fn install() -> io::Result<Self> {
        if SERVICE_OWNS_SIGNALS.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(Self { saved: Vec::new(), relaying: false });
        }
        if RELAYING
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_err()
        {
            return Ok(Self { saved: Vec::new(), relaying: false });
        }
        // `mut` lets each installed disposition be recorded as it is replaced.
        let mut saved: Vec<(c_int, libc::sigaction)> = Vec::with_capacity(RELAYED_SIGNALS.len());
        for signal in RELAYED_SIGNALS {
            match install_one(signal) {
                Ok(Some(previous)) => saved.push((signal, previous)),
                Ok(None) => {}
                Err(error) => {
                    restore(&saved);
                    RELAYING.store(false, std::sync::atomic::Ordering::SeqCst);
                    return Err(error);
                }
            }
        }
        return Ok(Self { saved, relaying: true });
    }

    /// What:
    ///  Start `command`,
    /// wait for it while relaying signals,
    /// and return how it ended;
    /// the synchronous face of
    ///  `run_child_awaiting`,
    /// which refuses from inside the runtime.
    ///  The guard does not exit the process;
    /// the caller releases its locks and removes its state,
    /// then exits.
    /// Why:
    ///   The callers are synchronous until they become asynchronous themselves (step T11).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// runChild(command: SpawnOptions): ChildOutcome // only until every caller is async
    /// ```
    pub fn run_child(&self, command: &mut Command) -> io::Result<ChildOutcome> {
        return match run_on_runtime(self.run_child_awaiting(command)) {
            Ok(outcome) => outcome,
            Err(refusal) => Err(io::Error::other(refusal.to_string())),
        };
    }

    /// What:
    ///  Start `command`,
    /// wait for it on its pidfd while relaying signals,
    /// and return how it ended.
    /// Why:
    ///   Signals that arrived before the child started are relayed as soon as it is published.
    /// A guard that owns no handlers
    ///  starts and waits the same way and publishes nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await guard.runChildAwaiting(command): Promise<ChildOutcome>
    /// ```
    pub async fn run_child_awaiting(&self, command: &mut Command) -> io::Result<ChildOutcome> {
        return run_relayed_with(command, self.relaying, self.saved.as_slice(), publish).await;
    }
}

/// What:
///  `run_relayed` with the dispositions to put back and the way the child is published.
/// Why:
///   A child that cannot be published is not relayed to,
/// and a guard whose child is not relayed to must not keep the
///  handlers:
/// the signal would reach neither the child nor the wrapper,
/// and only `SIGKILL` would end the command.
///  The dispositions therefore go back for the life of the guard,
/// so the next `SIGINT`,
/// `SIGTERM`,
/// `SIGHUP` or
///  `SIGQUIT` ends the command as it would have without a guard.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const published = relaying && publisher(child.id); if (relaying && !published) { restore(saved); warn('the signals are put back'); }
/// ```
#[cfg(unix)]
async fn run_relayed_with(command: &mut Command, relaying: bool, saved: &[(c_int, libc::sigaction)], publisher: Publisher) -> io::Result<ChildOutcome> {
    // In a service the shutdown waits for this child and ends it when the service is stopped; elsewhere there is no stop to wait for.
    let (_unfinished, stop) = crate::child_awaited_until_it_exits_or_the_service_is_stopped::begin_an_awaited_child("a Git step of a commit")?;
    let started: std::process::Child = spawn_retrying_without_blocking(command).await?;
    let owned = adopt(started);
    let published: bool = relaying && publisher(owned.id());
    if relaying && !published {
        // Nothing was published, so a handler would drop every signal it caught: `relay_pending` returns at its first line.
        // The dispositions go back, so the next signal ends this command as it would have without the guard, and the child is
        //  left to end by itself. `publish` has already said why the child was not published.
        restore(saved);
        crate::diagnostic_log::warn(
            "signalRelay",
            "the handlers of SIGINT, SIGTERM, SIGHUP and SIGQUIT are put back because the child is not relayed to, so a signal \
             now ends this command instead of being relayed; the child is left running and ends by itself. Hint: check the descriptor \
             limit (`ulimit -n`) and the control group's `pids.max`, free some, and run the command again",
        );
    }
    let waited: io::Result<ExitStatus> = crate::child_awaited_until_it_exits_or_the_service_is_stopped::wait_until_exit_or_stop(owned, stop, "a Git step of a commit").await;
    // The child is reaped, so a signal that finds the descriptor still pointing at it fails with `ESRCH`; the descriptor is
    //  repointed before the next child all the same.
    retire(published);
    return Ok(crate::forwarding::outcome_of(waited?));
}

/// Restore the previous dispositions and release the process-wide slot.
#[cfg(unix)]
impl Drop for SignalGuard {
    /// What:
    ///  Restore the saved dispositions and release the process's handlers when this guard owned them.
    /// Why:
    ///   The next guard,
    /// or the process default after the guard,
    /// must see the original handlers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { restore(saved); }
    /// ```
    fn drop(&mut self) {
        if !self.relaying {
            return;
        }
        restore(&self.saved);
        RELAYING.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// What:
///  Non-Unix guard:
/// no signal relay exists,
/// so the child runs and the wrapper waits.
/// Why:
///   Callers keep one API;
/// Windows ends a console child differently and is not covered by these controls.
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
    ///   Without relaying,
    /// the wait is the whole job.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async runChild(command): Promise<ChildOutcome>
    /// ```
    pub fn run_child(&self, command: &mut Command) -> io::Result<ChildOutcome> {
        let mut child: std::process::Child = crate::start_retry::RetryStart::spawn_retrying(command)?;
        let status: ExitStatus = child.wait()?;
        return Ok(crate::forwarding::outcome_of(status));
    }
}

/// Signal controls live beside the guard.
#[cfg(test)]
#[path = "signal_relay_tests.rs"]
mod tests;
