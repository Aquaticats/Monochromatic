//! What:
//!  A stop's signal and the process the kernel named as its sender,
//!  kept as one word.
//! Why:
//!  A signal handler may run on any thread at any instruction,
//!  and the record of a stopped commit states which stop ended the command and who sent it as one fact.
//!  Two words would let a delivery or a kill fall between them and pair one stop's signal with another stop's sender,
//!  which names a process that did not send what ended the command;
//!  one word cannot be read half written.
//!  The relay of the preparation and the wait of a commit both keep their stop this way,
//!  and neither has a version of it of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pack = (signal: number, sender?: number): bigint => BigInt(signal) | (sender === undefined ? 0n : BigInt(sender + 1) << 32n);
//! const unpack = (word: bigint): [number, number?] | undefined => ...;
//! ```

/// `c_int` is the C integer type signal numbers use.
#[cfg(unix)]
use std::ffi::c_int;

/// What:
///  A stop's signal and its sender as one word,
///  and back.
/// Why:
///  Two words let a process killed or a signal delivered between them pair one stop's signal with another stop's sender,
///  and the record of a stopped commit states both as one fact;
///  so every place this program keeps a signal a handler wrote holds it in this word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const packed = (signal: number, sender?: number): bigint => BigInt(signal) | (sender === undefined ? 0n : BigInt(sender + 1) << 32n);
/// ```
#[cfg(unix)]
pub(crate) fn packed(signal: c_int, sender: Option<i32>) -> u64 {
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
pub(crate) fn unpacked(word: u64) -> Option<(c_int, Option<i32>)> {
    let signal: c_int = c_int::try_from(word & u64::from(u32::MAX)).unwrap_or(0);
    if signal == 0 {
        return None;
    }
    let high: u64 = word >> 32;
    return Some((signal, if high == 0 { None } else { i32::try_from(high - 1).ok() }));
}

/// The packing is controlled beside this file,
/// because the pairing it keeps is what the record's sender depends on.
#[cfg(all(unix, test))]
#[path = "stop_word_tests.rs"]
mod tests;
