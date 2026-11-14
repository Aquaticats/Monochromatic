//! What:
//!  The stop signal this process relayed to a child,
//!  and the process the kernel named as its sender,
//!  kept for the command that waits after it.
//! Why:
//!   A person's interrupt that arrives while a Git child of a commit runs is relayed to that child
//!  (`signal_relay.rs`),
//!  and the child may already be gone or may not act on it,
//!  so nothing else would ever tell the command that the person stopped it:
//!  the commit would be made although the person interrupted it.
//!  The wait of a commit takes this beside its own flag
//!  (`service_client_interrupted.rs`),
//!  and no other command reads it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.on('SIGINT', () => { relayedStop = SIGINT; child.kill('SIGINT'); });
//! const stop = takeRelayedStop();
//! ```

/// `c_int` is the C integer type signal numbers use.
#[cfg(unix)]
use std::ffi::c_int;

/// The stop signal last relayed to a child,
/// or zero while none was.
#[cfg(unix)]
static RELAYED_STOP: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// The process the kernel named as that signal's sender,
/// or zero while none was named.
/// Why:
///  The handler runs asynchronously and may write only these words;
///  the wait reads them once,
///  and a signal the kernel attributes to no process keeps zero so the record says so.
#[cfg(unix)]
static RELAYED_SENDER: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// What:
///  Records that `signal` was relayed to a child,
///  and the process the kernel named as its sender.
/// Why:
///  The handler runs asynchronously,
///  so it writes these words and does nothing else;
///  the sender is there beside the signal because the record of the commit this stop ends names it,
///  and a stop with no sender states none rather than the one a previous stop named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteRelayedStop(signal, sender)
/// ```
#[cfg(unix)]
pub(crate) fn note(signal: c_int, sender: Option<i32>) {
    RELAYED_SENDER.store(sender.unwrap_or(0), std::sync::atomic::Ordering::SeqCst);
    RELAYED_STOP.store(signal, std::sync::atomic::Ordering::SeqCst);
}

/// The recorder of a system that relays no signal.
#[cfg(not(unix))]
pub(crate) fn note(_signal: std::os::raw::c_int, _sender: Option<i32>) {}

/// What:
///  The stop signal this process relayed to a child and the sender the kernel named,
///  taken once so the answer belongs to one wait.
/// Why:
///  The wait of a commit asks once per step,
///  and a signal a Git child already received must not be applied twice;
///  the sender is taken with the signal so it can never be read beside another stop's number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stop = [relayedStop, relayedSender]; relayedStop = 0; relayedSender = 0; stop
/// ```
#[cfg(unix)]
pub(crate) fn take() -> Option<(c_int, Option<i32>)> {
    let signal: c_int = RELAYED_STOP.swap(0, std::sync::atomic::Ordering::SeqCst);
    if signal == 0 {
        return None;
    }
    let sender: c_int = RELAYED_SENDER.swap(0, std::sync::atomic::Ordering::SeqCst);
    return Some((signal, if sender > 0 { Some(sender) } else { None }));
}

/// The taken stop of a system that relays no signal.
#[cfg(not(unix))]
pub(crate) fn take() -> Option<(std::os::raw::c_int, Option<i32>)> {
    return None;
}
