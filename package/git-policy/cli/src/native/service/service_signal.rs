//! What:
//!  The service's last line when a signal ends it:
//! the service manager stops a unit with `SIGTERM`,
//! and a terminal sends
//!  `SIGINT` or `SIGHUP`;
//! the handler writes one line with the signal and the uptime,
//! then lets the signal end the process as
//!  it would have without a handler.
//! Why:
//!  The journal of the unit with the 17-second commit had no line for the first service's end,
//! and the manager's own lines
//!  about the unit cannot be relied on.
//! A signal gives the process no chance to log from ordinary code,
//! so the line is
//!  built and written with nothing but calls that `signal-safety(7)` lists (`clock_gettime`,
//! `sendto`,
//! `send`,
//! `poll`,
//! `write`,
//! `signal`,
//! `raise`),
//! and the write never waits for room: a standard error that is a full pipe or stream loses the line, because a service
//!  that cannot end for want of room to say so is worse than a missing line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.on('SIGTERM', () => { console.error(`cli-git-service: exiting reason=signal-15 uptime_ms=${uptime()}`); process.exit(143); });
//! ```

use crate::journald_native::{IDENTIFIER, Priority, raw_address, send_without_waiting};
use crate::service_timing::{LOG_PREFIX, sink};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};

/// The signals that end the service without a word otherwise.
pub const HANDLED: [libc::c_int; 3] = [libc::SIGTERM, libc::SIGINT, libc::SIGHUP];

/// The words before the signal number in the line.
const BEFORE_SIGNAL: &str = "exiting reason=signal-";

/// The words between the signal number and the uptime.
const BEFORE_UPTIME: &str = " uptime_ms=";

/// The words between the uptime and the wall-clock time of the event,
/// as every other line of the service states it.
const BEFORE_TIME: &str = " at_ms=";

/// The room the line needs:
/// the prefix,
/// the words,
/// three numbers of at most twenty digits,
/// and the newline.
const LINE_CAPACITY: usize = 256;

/// The monotonic clock,
/// in nanoseconds,
/// when the handlers were installed;
/// zero before that.
static STARTED_NANOS: AtomicU64 = AtomicU64::new(0);

/// The native journal socket's descriptor, or -1 when the service's entries go to standard error.
static NATIVE_DESCRIPTOR: AtomicI32 = AtomicI32::new(-1);

/// The journal socket's address, made before any signal so the handler only reads it.
static NATIVE_ADDRESS: OnceLock<(libc::sockaddr_un, libc::socklen_t)> = OnceLock::new();

/// Whether standard error is the journal's stream, decided before any signal: the fallback line then carries the priority
/// prefix.
static ON_STREAM: AtomicBool = AtomicBool::new(false);

/// The words before the message in a native entry: the priority and the identifier.
const NATIVE_HEAD: &str = "SYSLOG_IDENTIFIER=";

/// The field that carries the time of the event beside the message.
const NATIVE_TIME_FIELD: &str = "\nCLI_GIT_AT_MS=";

/// The wall clock in whole milliseconds since the epoch,
/// zero when the clock cannot be read.
fn wall_millis_signal_safe() -> u64 {
    let mut now: libc::timespec = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `now` is valid for the call to write one `timespec`, and `clock_gettime` is safe in a signal handler.
    let status: libc::c_int = unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &raw mut now) };
    if status != 0 {
        return 0;
    }
    return u64::try_from(now.tv_sec).unwrap_or(0).saturating_mul(1_000).saturating_add(u64::try_from(now.tv_nsec).unwrap_or(0) / 1_000_000);
}

/// The monotonic clock in nanoseconds,
/// zero when the clock cannot be read.
fn monotonic_nanos() -> u64 {
    let mut now: libc::timespec = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `now` is valid for the call to write one `timespec`, and `clock_gettime` is safe in a signal handler.
    let status: libc::c_int = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &raw mut now) };
    if status != 0 {
        return 0;
    }
    return u64::try_from(now.tv_sec).unwrap_or(0).saturating_mul(1_000_000_000).saturating_add(u64::try_from(now.tv_nsec).unwrap_or(0));
}

/// What:
///  Appends the decimal digits of `value` to `line` at `used`,
/// and returns the new length.
/// Why:
///   Formatting in a signal handler may not allocate,
/// so the digits go into a fixed buffer from the least significant end and
///  are copied in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// line.write(String(value), used)
/// ```
fn push_number(line: &mut [u8; LINE_CAPACITY], used: usize, value: u64) -> usize {
    let mut digits: [u8; 20] = [0; 20];
    let mut count: usize = 0;
    let mut rest: u64 = value;
    loop {
        digits[count] = b'0' + u8::try_from(rest % 10).unwrap_or(0);
        count += 1;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    let mut end: usize = used;
    for index in (0..count).rev() {
        line[end] = digits[index];
        end += 1;
    }
    return end;
}

/// Appends `text` to `line` at `used`,
/// and returns the new length.
fn push_text(line: &mut [u8; LINE_CAPACITY], used: usize, text: &str) -> usize {
    let bytes: &[u8] = text.as_bytes();
    line[used..used + bytes.len()].copy_from_slice(bytes);
    return used + bytes.len();
}

/// What:
///  The line the handler writes for `signal` after `uptime_nanos` of running,
/// and its length.
/// Why:
///   Kept apart from the handler so a test can read the bytes without sending a signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git-service: exiting reason=signal-${signal} uptime_ms=${uptimeMs} at_ms=${Date.now()}\n`
/// ```
pub fn signal_line(signal: libc::c_int, uptime_nanos: u64, at_ms: u64, on_stream: bool) -> ([u8; LINE_CAPACITY], usize) {
    let mut line: [u8; LINE_CAPACITY] = [0; LINE_CAPACITY];
    // On the journal's stream a `<6>` prefix keeps the `info` level the line has on the native socket.
    let mut used: usize = push_text(&mut line, 0, if on_stream { "<6>" } else { "" });
    used = push_text(&mut line, used, LOG_PREFIX);
    used = push_text(&mut line, used, BEFORE_SIGNAL);
    used = push_number(&mut line, used, u64::try_from(signal).unwrap_or(0));
    used = push_text(&mut line, used, BEFORE_UPTIME);
    used = push_number(&mut line, used, uptime_nanos / 1_000_000);
    used = push_text(&mut line, used, BEFORE_TIME);
    used = push_number(&mut line, used, at_ms);
    line[used] = b'\n';
    return (line, used + 1);
}

/// What:
///  The native journal entry the handler sends for `signal`, and its length: the priority, the identifier, the message
///  (the line without its prefix and newline) and the time of the event as a field.
/// Why:
///   The kernel stamps a datagram when it is sent, so the exit of a service that is stopped is dated by the stop and not by
///  the moment the journal gets round to reading standard error. Built in a fixed buffer like `signal_line`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `PRIORITY=6\nSYSLOG_IDENTIFIER=monochromatic-git-policy-cli-service\nMESSAGE=exiting reason=signal-${signal} uptime_ms=${u} at_ms=${t}\nCLI_GIT_AT_MS=${t}\n`
/// ```
pub fn signal_datagram(signal: libc::c_int, uptime_nanos: u64, at_ms: u64) -> ([u8; LINE_CAPACITY], usize) {
    let mut line: [u8; LINE_CAPACITY] = [0; LINE_CAPACITY];
    let mut used: usize = push_text(&mut line, 0, "PRIORITY=");
    used = push_number(&mut line, used, u64::from(Priority::Info.number()));
    used = push_text(&mut line, used, "\n");
    used = push_text(&mut line, used, NATIVE_HEAD);
    used = push_text(&mut line, used, IDENTIFIER);
    used = push_text(&mut line, used, "\nMESSAGE=");
    used = push_text(&mut line, used, BEFORE_SIGNAL);
    used = push_number(&mut line, used, u64::try_from(signal).unwrap_or(0));
    used = push_text(&mut line, used, BEFORE_UPTIME);
    used = push_number(&mut line, used, uptime_nanos / 1_000_000);
    used = push_text(&mut line, used, BEFORE_TIME);
    used = push_number(&mut line, used, at_ms);
    used = push_text(&mut line, used, NATIVE_TIME_FIELD);
    used = push_number(&mut line, used, at_ms);
    line[used] = b'\n';
    return (line, used + 1);
}

/// The handler:
/// one `write` of the line to standard error,
/// then the signal's default action.
extern "C" fn on_signal(signal: libc::c_int) {
    let started: u64 = STARTED_NANOS.load(Ordering::Relaxed);
    let uptime: u64 = monotonic_nanos().saturating_sub(started);
    let at_ms: u64 = wall_millis_signal_safe();
    let on_stream: bool = ON_STREAM.load(Ordering::Relaxed);
    let (line, length) = signal_line(signal, uptime, at_ms, on_stream);
    let (entry, entry_length) = signal_datagram(signal, uptime, at_ms);
    let descriptor: libc::c_int = NATIVE_DESCRIPTOR.load(Ordering::Relaxed);
    // SAFETY: the buffers hold `length` and `entry_length` initialized bytes, the address was made before any signal and is
    //  only read, and `sendto`, `write`, `signal` and `raise` are safe in a signal handler. The results are dropped on
    //  purpose: a closed standard error is no reason to keep the process alive. The datagram goes first and the line is
    //  written only when it could not be sent, so the exit is in the journal once.
    unsafe {
        let sent: bool = match NATIVE_ADDRESS.get() {
            Some((address, address_length)) if descriptor >= 0 => {
                libc::sendto(descriptor, entry.as_ptr().cast(), entry_length, libc::MSG_DONTWAIT, std::ptr::from_ref(address).cast(), *address_length) >= 0
            }
            _ => false,
        };
        if !sent {
            // Without waiting for room (a full pipe or stream loses the line): `send_without_waiting` calls only `send`,
            //  `poll` and `write`.
            let _written = send_without_waiting(libc::STDERR_FILENO, &line[..length]);
        }
        libc::signal(signal, libc::SIG_DFL);
        libc::raise(signal);
    }
}

/// What:
///  Installs the handler for the signals that end the service.
/// Why:
///   Without it the process dies from `SIGTERM` with no line in the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// installExitLines()
/// ```
pub fn install() {
    STARTED_NANOS.store(monotonic_nanos(), Ordering::Relaxed);
    ON_STREAM.store(sink().on_stream(), Ordering::Relaxed);
    // The sink is decided here, before any signal, and the handler sends on its descriptor when the entries are native.
    if let Some((descriptor, address)) = sink().signal_target()
        && let Some(raw) = raw_address(address)
        && NATIVE_ADDRESS.set(raw).is_ok()
    {
        NATIVE_DESCRIPTOR.store(descriptor, Ordering::Relaxed);
    }
    for signal in HANDLED {
        // SAFETY: `on_signal` has the signature a handler needs and only calls functions that are safe in a handler.
        unsafe {
            libc::signal(signal, on_signal as *const () as libc::sighandler_t);
        }
    }
}

/// The line's bytes.
#[cfg(test)]
#[path = "service_signal_tests.rs"]
mod tests;
