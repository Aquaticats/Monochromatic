//! What:
//!  The bytes the signal handler writes.
//! Why:
//!  The handler cannot be tested by sending the test process a signal, and it may not allocate; the line is built apart from it,
//!  so its text and its handling of large numbers can be checked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(signalLine(15, uptimeNanos)).toBe('cli-git-service: exiting reason=signal-15 uptime_ms=...\n');
//! ```

/// The line and the handled signals.
use super::{HANDLED, LINE_CAPACITY, signal_line};
use crate::service_timing::LOG_PREFIX;

/// The line states the signal and the uptime in whole milliseconds, ends in a newline, and starts like the service's other lines.
#[test]
fn the_line_states_the_signal_and_the_uptime_in_milliseconds() {
    let uptime_millis: u64 = 12_345;
    let (bytes, length) = signal_line(libc::SIGTERM, uptime_millis * 1_000_000 + 999_999);
    let text: &str = std::str::from_utf8(&bytes[..length]).expect("text");
    assert_eq!(text, format!("{LOG_PREFIX}exiting reason=signal-{} uptime_ms={uptime_millis}\n", libc::SIGTERM));
}

/// A zero uptime and the largest one both fit the buffer.
#[test]
fn the_extremes_fit_the_buffer() {
    let (zero, zero_length) = signal_line(libc::SIGHUP, 0);
    assert!(std::str::from_utf8(&zero[..zero_length]).expect("text").ends_with("uptime_ms=0\n"));
    let (largest, largest_length) = signal_line(libc::c_int::MAX, u64::MAX);
    assert!(largest_length < LINE_CAPACITY);
    assert!(std::str::from_utf8(&largest[..largest_length]).expect("text").ends_with(format!("uptime_ms={}\n", u64::MAX / 1_000_000).as_str()));
}

/// The signals that end the service are the three that a manager or a terminal sends.
#[test]
fn the_handled_signals_are_termination_interrupt_and_hangup() {
    assert_eq!(HANDLED, [libc::SIGTERM, libc::SIGINT, libc::SIGHUP]);
}
