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
//!
//! Files beside this one,
//!  and why the relay is not one file:
//!  `signal_relay_child_published_to_the_handler.rs` holds the child the handler signals,
//!  the slots a signal caught before that child exists waits in,
//!  and the publishing the handler reaches it through.
//!  `rust/max-lines` splits the relay by role,
//!  and the guard with its handler is the role that catches and owns the signals.

pub(crate) use super::signal_relay_child_published_to_the_handler::{
    CeilingDuty,
    publish_child,
    retire_child,
};
use super::signal_relay_child_published_to_the_handler::{
    PENDING,
    relay_pending,
    run_relayed,
};
use crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::CeilingStop;
use crate::forwarding::ChildOutcome;
use crate::lazy_runtime::run_on_runtime;
use crate::signal_report::read_report;
#[cfg(unix)]
use std::ffi::c_int;
use std::io;
use std::process::Command;

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
const PROCESS_SENDER_CODES: &[c_int] = &[libc::SI_USER, libc::SI_QUEUE, libc::SI_TKILL];

/// Sender codes that mean a process sent the signal,
/// on other Unix systems.
#[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
const PROCESS_SENDER_CODES: &[c_int] = &[libc::SI_USER, libc::SI_QUEUE];

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
pub const RELAYED_SIGNALS: &[c_int] = &[libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT];

/// What:
///  Whether a service owns this process's stop signals,
/// so no guard relays them.
/// Why:
///   A stop of the service is an operation of its own (owner,
/// 2026-11-07):
/// it must not be taken for the signal of a commit.
#[cfg(unix)]
static SERVICE_OWNS_SIGNALS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

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
    let (from_process, sender): (bool, Option<i32>) = read_report(info);
    if !from_process {
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
    // The two signals a waiting commit answers are noted as well, because the child this one is relayed to may be gone, and
    //  the command that waits must still learn that the person stopped it (`signal_relay_relayed_stop.rs`); the process the
    //  kernel named is kept with the signal, so the record of that commit can tell a tool's bound from a keyboard's interrupt.
    if signal == libc::SIGINT || signal == libc::SIGTERM {
        crate::signal_relay_relayed_stop::note(signal, sender);
    }
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
    ///  Whether this guard owns the process's handlers,
    /// and so relays to the child it publishes.
    /// Why:
    ///   A guard made while another owns them,
    /// or in a process whose service owns its stop signals,
    /// publishes nothing;
    /// the sibling module that starts and publishes a child asks this rather than reading the field.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// relays(): boolean
    /// ```
    pub(crate) fn relays(&self) -> bool {
        return self.relaying;
    }

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
            return Ok(Self {
                saved: Vec::new(),
                relaying: false,
            });
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
            return Ok(Self {
                saved: Vec::new(),
                relaying: false,
            });
        }
        // `mut` lets each installed disposition be recorded as it is replaced.
        let mut saved: Vec<(c_int, libc::sigaction)> = Vec::with_capacity(RELAYED_SIGNALS.len());
        for &signal in RELAYED_SIGNALS {
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
        return Ok(Self {
            saved,
            relaying: true,
        });
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
    pub fn run_child(
        &self,
        command: &mut Command,
    ) -> io::Result<ChildOutcome> {
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
    pub async fn run_child_awaiting(
        &self,
        command: &mut Command,
    ) -> io::Result<ChildOutcome> {
        let unheeded: crate::child_number_told_at_start::ChildNumber =
            crate::child_number_told_at_start::ChildNumber::not_started();
        let duty: CeilingDuty<'_> = CeilingDuty {
            told: &unheeded,
            bound: None,
            while_held: None,
            stall: None,
        };
        let (outcome, _no_ceiling): (ChildOutcome, Option<CeilingStop>) =
            run_relayed(command, self.relaying, duty).await?;
        return Ok(outcome);
    }

    /// What:
    ///  Start `command`,
    /// wait for it while relaying signals,
    /// and return how it ended,
    /// with the ceiling's stop when the ceiling came first;
    /// the child's number is published into `duty.told` as soon as it exists.
    /// Why:
    ///   A ceiling that ends a step must name that step and no other child of this process,
    /// and the only moment its number can be had is the moment it starts.
    ///  This wait ends the child it owns when the ceiling comes,
    /// whose number cannot be reused before it is reaped,
    /// so the ceiling depends on no thread and on no name a reuse can reach.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const [outcome, ceilingStop] = guard.runChildNaming(command, { told: notice.stepNumber(), bound });
    /// ```
    pub fn run_child_telling_its_number(
        &self,
        command: &mut Command,
        duty: CeilingDuty<'_>,
    ) -> io::Result<(ChildOutcome, Option<CeilingStop>)> {
        return match run_on_runtime(run_relayed(command, self.relaying, duty)) {
            Ok(outcome) => outcome,
            Err(refusal) => Err(io::Error::other(refusal.to_string())),
        };
    }
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

/// The guard of a system with no signal relay,
/// which keeps the same name at every call site.
/// Why:
///   The two implementations are the same API and only one is compiled,
/// so the call sites name `SignalGuard` and never a platform.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { SignalGuard } from './signal-relay-without-relay';
/// ```
#[cfg(not(unix))]
pub use crate::signal_relay_without_relay::SignalGuard;

/// Signal controls live beside the guard.
#[cfg(test)]
#[path = "signal_relay_tests.rs"]
mod tests;
