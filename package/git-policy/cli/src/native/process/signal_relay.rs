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
/// The descriptor the handler sends through,
/// and the pidfd that names the child it points at.
use crate::signal_relay_descriptor::{ensure_relay_descriptor, pidfd_of, point_relay_at};
/// A child owned until it is reaped,
/// awaited through its pidfd.
use crate::child_awaited_through_a_pidfd::adopt;
/// `Command` builds the child,
/// and `ExitStatus` is how it ended.
use std::process::Command;

/// The Git child's end,
/// with the exit-code mapping the wrapper's exit uses.
use crate::forwarding::ChildOutcome;
/// The record of a step the wait had to end itself at its ceiling.
use crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::CeilingStop;

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
///  Whether a child is published to the handler:
/// on Linux the descriptor `RELAY_FD` stands for it,
/// elsewhere `CHILD_PID` names it.
/// Why:
///   The handler runs asynchronously,
/// so it can only read state through atomics.
#[cfg(unix)]
static CHILD_PUBLISHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  PID of the Git child being waited for,
/// or zero when none is,
/// on the systems that have no pidfd.
/// Why:
///   Without a pidfd the handler signals by number,
/// with the recycled-number window the module's limits name.
#[cfg(all(unix, not(target_os = "linux")))]
static CHILD_PID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// What:
///  The one descriptor number the relay signals through,
/// made by the first guard and never closed.
/// Why:
///   A descriptor that is closed while a handler reads its number can be reused by an unrelated pidfd and then signal the wrong
///  process.
/// This number is never closed,
/// so it always names either the published child or `/dev/null`:
/// publishing and
///  retiring replace what it refers to with `dup3`,
/// atomically,
/// and no other code can be handed this number.
/// A signal that finds
///  `/dev/null` fails harmlessly.
#[cfg(target_os = "linux")]
pub(crate) static RELAY_FD: std::sync::OnceLock<std::os::fd::RawFd> = std::sync::OnceLock::new();

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
///  One slot per relayed signal,
/// holding a signal number that has not been relayed yet,
/// or zero.
/// Why:
///   A signal that arrives before the child exists is relayed as soon as its PID is recorded.
#[cfg(unix)]
static PENDING: [std::sync::atomic::AtomicI32; 4] =
    [const { std::sync::atomic::AtomicI32::new(0) }; 4];

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
    // The two signals a waiting commit answers are noted as well, because the child this one is relayed to may be gone, and
    //  the command that waits must still learn that the person stopped it (`signal_relay_relayed_stop.rs`).
    if signal == libc::SIGINT || signal == libc::SIGTERM {
        crate::signal_relay_relayed_stop::note(signal);
    }
    relay_pending();
}

/// What:
///  Send every pending signal to the published child,
/// claiming each slot so it is sent once.
/// Why:
///   Both the handler and the waiting code call this;
/// the atomic swap decides which one sends a given signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const signal of pending.drain()) child.kill(signal);
/// ```
#[cfg(unix)]
fn relay_pending() {
    if !CHILD_PUBLISHED.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    for slot in &PENDING {
        let signal: c_int = slot.swap(0, std::sync::atomic::Ordering::SeqCst);
        if signal != 0 {
            send_to_published_child(signal);
        }
    }
}

/// What:
///  Send `signal` to the published child through its pidfd.
/// Why:
///   The call is a raw system call of `rustix`'s Linux backend,
/// which neither allocates nor touches `errno`,
/// so the handler leaves
///  the interrupted code's `errno` as it found it.
/// A child that was reaped,
/// or the `/dev/null` that stands in when none is
///  published,
/// makes the call fail,
/// and the failure is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.kill(signal); // by the pidfd, never by a number that may have been reused
/// ```
#[cfg(target_os = "linux")]
fn send_to_published_child(signal: c_int) {
    let Some(descriptor) = RELAY_FD.get() else {
        return;
    };
    let Some(named) = rustix::process::Signal::from_named_raw(signal) else {
        return;
    };
    // SAFETY: the descriptor number is never closed (see `RELAY_FD`), so it stays open for the whole process.
    let borrowed: std::os::fd::BorrowedFd<'_> = unsafe { std::os::fd::BorrowedFd::borrow_raw(*descriptor) };
    let _failed_harmlessly = rustix::process::pidfd_send_signal(borrowed, named);
}

/// What:
///  Send `signal` to the child whose number is published,
/// on the systems that have no pidfd.
/// Why:
///   The limit named on the module:
/// a number reused between the child's exit and its reaping would be signalled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(pid, signal);
/// ```
#[cfg(all(unix, not(target_os = "linux")))]
fn send_to_published_child(signal: c_int) {
    let pid: i32 = CHILD_PID.load(std::sync::atomic::Ordering::SeqCst);
    if pid > 0 {
        // SAFETY: `kill` sends one signal to a PID this module recorded; a failure is ignored because the child may have ended.
        unsafe { libc::kill(pid, signal) };
    }
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
        let unheeded: crate::child_number_told_at_start::ChildNumber = crate::child_number_told_at_start::ChildNumber::not_started();
        let duty: CeilingDuty<'_> = CeilingDuty { told: &unheeded, bound: None, while_held: None, stall: None };
        let (outcome, _no_ceiling): (ChildOutcome, Option<CeilingStop>) = run_relayed(command, self.relaying, duty).await?;
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

/// What:
///  Who ends a Git step that reaches its ceiling.
/// Why:
///  The wait that owns a child is the only place its number cannot be reused,
/// so it is the enforcement a ceiling always has (`child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`).
///  The notice thread ends the same step through the descriptor taken at its start,
/// which names that step for good,
/// so one bound has two enforcers and its stop is recorded once.
///  A commit that may open an editor is under no ceiling at all:
/// the person is the wait,
/// and a stop would lose what they typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CeilingDuty = { told: ChildNumber; bound?: Ceilings; lines?: Queue; stall?: TheStallDecidedOnce };
/// ```
pub struct CeilingDuty<'a> {
    /// The spot the step's number is published into the moment it starts.
    pub told: &'a crate::child_number_told_at_start::ChildNumber,
    /// The two moments that end the step,
    /// both counted from its start;
    /// nothing when a message may be typed into an editor in it.
    pub bound: Option<crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::Ceilings>,
    /// The queue the command says through the writer thread it hands the step's lines to,
    /// and nothing when the command has no line to say.
    pub while_held: Option<&'a std::sync::mpsc::SyncSender<String>>,
    /// The one decision this ceiling shares with the notice's thread,
    /// so neither enforcer can end the step at the usual ceiling after the other has stood aside;
    /// nothing when no notice thread watches this step, and the wait then decides on its own.
    pub stall: Option<&'a crate::scratch_step_stall_decided_once::TheStallDecidedOnce>,
}

/// What:
///  Publish a child this process started to the relay's handler,
/// for the sibling module that owns the token.
/// Why:
///   The descriptor is this module's business alone,
/// so the token asks here rather than reaching for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishChild(childId): boolean
/// ```
pub(crate) fn publish_child(child_id: u32) -> bool {
    return publish(child_id);
}

/// What:
///  Stop the handler pointing at the child that token published,
/// for the sibling module that owns the token.
/// Why:
///   The same reason as `publish_child`:
/// one module points the descriptor,
/// and one stops it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// retireChild(published: boolean): void
/// ```
pub(crate) fn retire_child(published: bool) {
    retire(published);
}

/// What:
///  Stop publishing the child,
/// on a system that has no relay to point.
/// Why:
///   The relayed-child token drops on every platform,
/// and there is nothing to point back on this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retire(published: boolean): void {}
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub(crate) fn retire_child(_published: bool) {}

/// Spawns the child,
/// publishes it to the handler when `relaying`,
/// relays any signal that was caught before the child existed,
/// and waits for it on its pidfd.
///
/// A handler that runs before the spawn keeps its signal in `PENDING` and finds no child.
/// The child is published before the drain,
/// so that signal is relayed here,
/// not lost.
#[cfg(unix)]
async fn run_relayed(command: &mut Command, relaying: bool, duty: CeilingDuty<'_>) -> io::Result<(ChildOutcome, Option<CeilingStop>)> {
    // In a service the shutdown waits for this child and ends it when the service is stopped; elsewhere there is no stop to wait for.
    let (_unfinished, stop) = crate::child_awaited_until_it_exits_or_the_service_is_stopped::begin_an_awaited_child("a Git step of a commit")?;
    let started: std::process::Child = spawn_retrying_without_blocking(command).await?;
    duty.told.publish(i32::try_from(started.id()).unwrap_or(0));
    let owned = adopt(started);
    let published: bool = relaying && publish(owned.id());
    // What: The wait that owns the child is the one that ends it at the ceiling.
    // Why:  A ceiling that depends on the notice's thread depends on a thread the system may refuse to make,
    //       and on one that has not died since it started.
    //       This wait needs neither:
    //       the number of an unreaped child cannot be reused, and only this path reaps it,
    //       so its signal reaches the step and never a successor.
    //       The notice's own ceiling stays as the second enforcer of the same bound,
    //       and a stop is recorded once, by whichever of the two ended the step.
    let waited: (ChildOutcome, Option<CeilingStop>) = match duty.bound {
        Some(ceilings) => crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::wait_until_it_exits_the_stop_or_the_ceiling(
            owned,
            stop,
            ceilings,
            "a Git step of a commit",
            crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::WhatTheCeilingIsTold { stall: duty.stall, lines: duty.while_held },
        )
        .await?,
        None => (
            crate::forwarding::outcome_of(crate::child_awaited_until_it_exits_or_the_service_is_stopped::wait_until_exit_or_stop(owned, stop, "a Git step of a commit").await?),
            None,
        ),
    };
    // The child is reaped, so a signal that finds the descriptor still pointing at it fails with `ESRCH`; the descriptor is
    //  repointed before the next child all the same.
    retire(published);
    return Ok(waited);
}

/// Publish the child with this id to the handler and relay what was waiting for it;
/// a failure to publish leaves the child
/// unrelayed and is logged,
/// because the commit must go on.
/// True when the child was published.
#[cfg(target_os = "linux")]
fn publish(child_id: u32) -> bool {
    let opened: Option<std::os::fd::OwnedFd> = i32::try_from(child_id)
        .ok()
        .and_then(rustix::process::Pid::from_raw)
        .and_then(pidfd_of);
    if let Err(error) = ensure_relay_descriptor() {
        // The commit must go on: without the descriptor the child is not relayed to, and the log says so.
        crate::diagnostic_log::warn("signalRelay", format!("no descriptor for the relay ({error}), so signals will not be relayed to the child").as_str());
        return false;
    }
    let Some(pidfd) = opened else {
        crate::diagnostic_log::warn("signalRelay", "the child could not be given a pidfd, so signals sent to this process will not be relayed to it");
        return false;
    };
    if let Err(error) = point_relay_at(Some(&pidfd)) {
        crate::diagnostic_log::warn("signalRelay", format!("the relay could not be pointed at the child ({error}), so signals will not be relayed to it").as_str());
        return false;
    }
    CHILD_PUBLISHED.store(true, std::sync::atomic::Ordering::SeqCst);
    relay_pending();
    return true;
}

/// Publish the child with this id by its number,
/// on the systems that have no pidfd.
#[cfg(all(unix, not(target_os = "linux")))]
fn publish(child_id: u32) -> bool {
    let Ok(pid) = libc::pid_t::try_from(child_id) else {
        return false;
    };
    CHILD_PID.store(pid, std::sync::atomic::Ordering::SeqCst);
    CHILD_PUBLISHED.store(true, std::sync::atomic::Ordering::SeqCst);
    relay_pending();
    return true;
}

/// Stop publishing the child and point the relay at nothing.
#[cfg(target_os = "linux")]
fn retire(published: bool) {
    if !published {
        return;
    }
    CHILD_PUBLISHED.store(false, std::sync::atomic::Ordering::SeqCst);
    if let Err(error) = point_relay_at(None) {
        crate::diagnostic_log::warn("signalRelay", format!("the relay could not be pointed away from the ended child: {error}").as_str());
    }
}

/// Stop publishing the child,
/// on the systems that have no pidfd.
#[cfg(all(unix, not(target_os = "linux")))]
fn retire(published: bool) {
    if !published {
        return;
    }
    CHILD_PUBLISHED.store(false, std::sync::atomic::Ordering::SeqCst);
    CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);
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
