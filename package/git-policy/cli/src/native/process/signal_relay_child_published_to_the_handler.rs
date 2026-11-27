//! What:
//!  The child this process publishes to the relay's handler:
//!  the slots a signal that arrives before the child exists waits in,
//!  the publishing and retiring of the child the handler signals,
//!  and the wait that starts the child under its ceiling.
//! Why:
//!  The handler runs asynchronously,
//!  so the only state it may read is an atomic and the one descriptor this file publishes through,
//!  and a signal caught before the child exists is relayed the moment it is published.
//!  `rust/max-lines` splits the relay by role,
//!  and what a signal arrives at is the role on this side of the handler.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! publish(child); relayPending(); // and afterwards retire(child)
//! ```

use crate::child_awaited_through_a_pidfd::adopt;
use crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::CeilingStop;
use crate::forwarding::ChildOutcome;
use crate::signal_relay_descriptor::{
    RELAY_FD,
    ensure_relay_descriptor,
    pidfd_of,
    point_relay_at,
};
use crate::start_retry::spawn_retrying_without_blocking;
use std::ffi::c_int;
use std::io;
use std::process::Command;

/// What:
///  Whether a child is published to the handler:
/// on Linux the descriptor `RELAY_FD` stands for it,
/// elsewhere `CHILD_PID` names it.
/// Why:
///   The handler runs asynchronously,
/// so it can only read state through atomics.
#[cfg(unix)]
pub(super) static CHILD_PUBLISHED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

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
///  One slot per relayed signal,
/// holding a signal number that has not been relayed yet,
/// or zero.
/// Why:
///   A signal that arrives before the child exists is relayed as soon as its PID is recorded.
#[cfg(unix)]
pub(super) static PENDING: [std::sync::atomic::AtomicI32; 4] =
    [const { std::sync::atomic::AtomicI32::new(0) }; 4];

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
pub(super) fn relay_pending() {
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
    let borrowed: std::os::fd::BorrowedFd<'_> =
        unsafe { std::os::fd::BorrowedFd::borrow_raw(*descriptor) };
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
    /// nothing when no notice thread watches this step,
    /// and the wait then decides on its own.
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
pub(super) async fn run_relayed(
    command: &mut Command,
    relaying: bool,
    duty: CeilingDuty<'_>,
) -> io::Result<(ChildOutcome, Option<CeilingStop>)> {
    // In a service the shutdown waits for this child and ends it when the service is stopped; elsewhere there is no stop to wait for.
    let (_unfinished, stop) =
        crate::child_awaited_until_it_exits_or_the_service_is_stopped::begin_an_awaited_child(
            "a Git step of a commit",
        )?;
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
pub(super) fn publish(child_id: u32) -> bool {
    let opened: Option<std::os::fd::OwnedFd> = i32::try_from(child_id)
        .ok()
        .and_then(rustix::process::Pid::from_raw)
        .and_then(pidfd_of);
    if let Err(error) = ensure_relay_descriptor() {
        // The commit must go on: without the descriptor the child is not relayed to, and the log says so.
        crate::diagnostic_log::warn(
            "signalRelay",
            format!(
                "no descriptor for the relay ({error}), so signals will not be relayed to the child"
            )
            .as_str(),
        );
        return false;
    }
    let Some(pidfd) = opened else {
        crate::diagnostic_log::warn(
            "signalRelay",
            "the child could not be given a pidfd, so signals sent to this process will not be relayed to it",
        );
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
pub(super) fn publish(child_id: u32) -> bool {
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
pub(super) fn retire(published: bool) {
    if !published {
        return;
    }
    CHILD_PUBLISHED.store(false, std::sync::atomic::Ordering::SeqCst);
    if let Err(error) = point_relay_at(None) {
        crate::diagnostic_log::warn(
            "signalRelay",
            format!("the relay could not be pointed away from the ended child: {error}").as_str(),
        );
    }
}

/// Stop publishing the child,
/// on the systems that have no pidfd.
#[cfg(all(unix, not(target_os = "linux")))]
pub(super) fn retire(published: bool) {
    if !published {
        return;
    }
    CHILD_PUBLISHED.store(false, std::sync::atomic::Ordering::SeqCst);
    CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);
}
