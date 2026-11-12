//! What:
//!  The stop signal this process relayed to a child,
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

/// What:
///  Records that `signal` was relayed to a child.
/// Why:
///  The handler runs asynchronously,
///  so it writes one word and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteRelayedStop(signal)
/// ```
#[cfg(unix)]
pub(crate) fn note(signal: c_int) {
    RELAYED_STOP.store(signal, std::sync::atomic::Ordering::SeqCst);
}

/// The recorder of a system that relays no signal.
#[cfg(not(unix))]
pub(crate) fn note(_signal: std::os::raw::c_int) {}

/// What:
///  The stop signal this process relayed to a child,
///  taken once so the answer belongs to one wait.
/// Why:
///  The wait of a commit asks once per step,
///  and a signal a Git child already received must not be applied twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stop = relayedStop; relayedStop = 0; stop
/// ```
#[cfg(unix)]
pub(crate) fn take() -> Option<c_int> {
    let signal: c_int = RELAYED_STOP.swap(0, std::sync::atomic::Ordering::SeqCst);
    return if signal == 0 { None } else { Some(signal) };
}

/// The taken stop of a system that relays no signal.
#[cfg(not(unix))]
pub(crate) fn take() -> Option<std::os::raw::c_int> {
    return None;
}
