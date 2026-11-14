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
/// A stop's signal and its sender as one word,
/// kept in the one place that encodes them.
#[cfg(unix)]
use crate::stop_word::{packed, unpacked};

/// The stop signal last relayed to a child and the process the kernel named as its sender,
/// in one word:
/// the signal in the low half,
/// and one more than the sender in the high half,
/// so zero means no sender.
/// Why:
///  The handler runs asynchronously and may write only one such word,
///  and the wait reads it once:
///  a process killed at any instruction is ordinary here,
///  and two separate words would let a kill fall between them and pair one stop's signal with another's sender.
///  One word cannot be half written.
#[cfg(unix)]
static RELAYED_STOP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

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
    RELAYED_STOP.store(packed(signal, sender), std::sync::atomic::Ordering::SeqCst);
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
    return unpacked(RELAYED_STOP.swap(0, std::sync::atomic::Ordering::SeqCst));
}

/// The taken stop of a system that relays no signal.
#[cfg(not(unix))]
pub(crate) fn take() -> Option<(std::os::raw::c_int, Option<i32>)> {
    return None;
}

/// The packing and unpacking are controlled beside this file,
/// because the pairing they keep is what the record's sender depends on.
#[cfg(all(unix, test))]
#[path = "signal_relay_relayed_stop_tests.rs"]
mod tests;
