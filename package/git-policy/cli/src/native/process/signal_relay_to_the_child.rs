//! What:
//!  The one descriptor a relayed child is published on:
//!  the number a caught signal is sent through,
//!  the pending signal that arrived before its child existed,
//!  and the end of that publication.
//! Why:
//!  `signal_relay.rs` holds the handlers,
//! the guard and the wait under its line limit,
//! as
//!  `child_awaited_until_it_exits_or_the_service_is_stopped.rs` was split out for the same reason;
//! these pieces
//!  use none of that,
//! so they sit in a file of their own and `signal_relay.rs` owns the names the tests use.
//!  The number is never closed and is repointed with `dup3`,
//! so a signal cannot reach a process number that was reused while
//!  the handler read it,
//! and no other code of this process can be handed that number.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const published = publish(child); // dup3(pidfdOf(child), RELAY_FD); drainPending();
//! retire(published);                // dup3(openDevNull(), RELAY_FD)
//! ```

/// Signals are C integers,
/// and so are the status codes and the sender codes beside them.
#[cfg(unix)]
use std::ffi::c_int;

/// What:
///  Whether a child is published to the handler:
/// on Linux the descriptor `RELAY_FD` stands for it,
/// elsewhere `CHILD_PID` names it.
/// Why:
///   The handler runs asynchronously,
/// so it can only read state through atomics.
#[cfg(unix)]
pub(crate) static CHILD_PUBLISHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  PID of the Git child being waited for,
/// or zero when none is,
/// on the systems that have no pidfd.
/// Why:
///   Without a pidfd the handler signals by number,
/// with the recycled-number window `signal_relay.rs` names.
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
///  One slot per relayed signal,
/// holding a signal number that has not been relayed yet,
/// or zero.
/// Why:
///   A signal that arrives before the child exists is relayed as soon as the child is published.
#[cfg(unix)]
pub(crate) static PENDING: [std::sync::atomic::AtomicI32; 4] =
    [const { std::sync::atomic::AtomicI32::new(0) }; 4];

/// What:
///  How a child is published to the handler:
/// true when the handler can reach it.
/// Why:
///  A control cannot refuse this process's descriptors,
/// so it hands in a publisher that refuses,
/// which is what a
///  descriptor limit does
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 6).
#[cfg(unix)]
pub(crate) type Publisher = fn(u32) -> bool;

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
///   The limit named on `signal_relay.rs`:
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
pub(crate) fn relay_pending() {
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
///  Make the descriptor number the relay signals through,
/// once for the process.
/// Why:
///   `/dev/null` stands on the number until a child is published;
/// the number is never closed (see `RELAY_FD`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// relayDescriptor ??= openDevNull();
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn ensure_relay_descriptor() -> std::io::Result<()> {
    if RELAY_FD.get().is_some() {
        return Ok(());
    }
    let nothing: std::os::fd::OwnedFd = rustix::fs::open("/dev/null", rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
        .map_err(std::io::Error::from)?;
    let number: std::os::fd::RawFd = std::os::fd::IntoRawFd::into_raw_fd(nothing);
    // A guard that lost the race closes its own descriptor; the winner's number is the only one that is kept.
    if RELAY_FD.set(number).is_err() {
        // SAFETY: the number was just made here and was not published, so nothing else holds or reads it.
        drop(unsafe { <std::os::fd::OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(number) });
    }
    return Ok(());
}

/// What:
///  Make the relay descriptor name `child` (a pidfd opened for it),
/// or `/dev/null` when `child` is nothing.
/// Why:
///   `dup3` replaces what the number refers to in one step,
/// so the handler never sees a closed number and never a descriptor of
///  another process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointRelayAt(child);
/// ```
#[cfg(target_os = "linux")]
fn point_relay_at(child: Option<&std::os::fd::OwnedFd>) -> std::io::Result<()> {
    let Some(target) = RELAY_FD.get() else {
        return Err(std::io::Error::other("the relay descriptor was never made"));
    };
    let nothing: std::os::fd::OwnedFd;
    let source: std::os::fd::RawFd = match child {
        Some(pidfd) => std::os::fd::AsRawFd::as_raw_fd(pidfd),
        None => {
            nothing = rustix::fs::open("/dev/null", rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
                .map_err(std::io::Error::from)?;
            std::os::fd::AsRawFd::as_raw_fd(&nothing)
        }
    };
    // SAFETY: `dup3` takes two descriptor numbers and no memory; `source` is open for this call and `target` is the process's own
    //  never-closed number. `O_CLOEXEC` keeps the child from inheriting the copy.
    let replaced: c_int = unsafe { libc::dup3(source, *target, libc::O_CLOEXEC) };
    if replaced < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// A pidfd for the process with this number,
/// or nothing when the system gives none.
#[cfg(target_os = "linux")]
fn pidfd_of(pid: rustix::process::Pid) -> Option<std::os::fd::OwnedFd> {
    return rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).ok();
}

/// What:
///  Publish the child with this id to the handler and relay what was waiting for it.
///  A failure is logged and leaves the child unrelayed,
/// so the caller knows to put the handlers back.
/// Why:
///  The commit must go on:
/// without the descriptor the child is not relayed to,
/// and the log says so,
/// but a handler
///  that swallows the signal because nothing is published is no way out of a stuck command
/// (`signal_relay.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publish(child) { const pidfd = pidfdOf(child); if (!pidfd) return false; pointRelayAt(pidfd); CHILD_PUBLISHED = true; relayPending(); return true; }
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn publish(child_id: u32) -> bool {
    let opened: Option<std::os::fd::OwnedFd> = i32::try_from(child_id)
        .ok()
        .and_then(rustix::process::Pid::from_raw)
        .and_then(pidfd_of);
    if let Err(error) = ensure_relay_descriptor() {
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
pub(crate) fn publish(child_id: u32) -> bool {
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
pub(crate) fn retire(published: bool) {
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
pub(crate) fn retire(published: bool) {
    if !published {
        return;
    }
    CHILD_PUBLISHED.store(false, std::sync::atomic::Ordering::SeqCst);
    CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);
}
