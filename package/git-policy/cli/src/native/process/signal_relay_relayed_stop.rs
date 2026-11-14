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

/// The two halves of `RELAYED_STOP` as one word.
/// Why:
///  The pairing is what makes the record honest, so the packing is named and stated once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const packed = (signal: number, sender?: number): bigint => BigInt(signal) | (sender === undefined ? 0n : BigInt(sender + 1) << 32n);
/// ```
#[cfg(unix)]
fn packed(signal: c_int, sender: Option<i32>) -> u64 {
    let low: u64 = u64::from(u32::try_from(signal).unwrap_or(0));
    let high: u64 = match sender {
        Some(pid) => u64::from(u32::try_from(pid).unwrap_or(0)) + 1,
        None => 0,
    };
    return low | (high << 32);
}

/// What:
///  The signal and sender a packed word holds,
///  and nothing when it holds no stop.
/// Why:
///  The wait reads the pair the handler wrote as one value,
///  so it can never read half of one stop beside half of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [signal, sender] = unpacked(word); // [SIGTERM, 4242] or []
/// ```
#[cfg(unix)]
fn unpacked(word: u64) -> Option<(c_int, Option<i32>)> {
    let signal: c_int = c_int::try_from(word & u64::from(u32::MAX)).unwrap_or(0);
    if signal == 0 {
        return None;
    }
    let high: u64 = word >> 32;
    return Some((signal, if high == 0 { None } else { i32::try_from(high - 1).ok() }));
}

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

/// The packing and unpacking are controlled here,
/// because the pairing they keep is what the record's sender depends on.
#[cfg(all(unix, test))]
mod tests {
    use super::*;

    /// What:
    ///  A packed stop gives back the signal and the sender it was written with,
    ///  and a word that holds no sender is read as none.
    /// Why:
    ///  The two live in one word so that a process killed between two stores cannot pair one stop's signal with
    ///  another stop's sender;
    ///  the packing is therefore the thing that has to be right,
    ///  and each shape a stop can have is read back here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// expect(unpacked(packed(SIGTERM, 4242))).toEqual([SIGTERM, 4242]);
    /// expect(unpacked(packed(SIGINT, undefined))).toEqual([SIGINT, undefined]);
    /// expect(unpacked(0)).toBeUndefined();
    /// ```
    #[test]
    fn a_relayed_stop_is_read_back_with_the_sender_it_was_written_with() {
        assert_eq!(unpacked(packed(libc::SIGTERM, Some(4_242))), Some((libc::SIGTERM, Some(4_242))), "a process's stop keeps both halves");
        assert_eq!(unpacked(packed(libc::SIGINT, None)), Some((libc::SIGINT, None)), "a terminal's stop names no process");
        assert_eq!(unpacked(0), None, "a word no stop was written to holds none");
        // A sender alone is no stop: the signal half is what says there is one.
        assert_eq!(unpacked(packed(0, Some(4_242))), None, "a word with a sender and no signal holds no stop");
    }
}
