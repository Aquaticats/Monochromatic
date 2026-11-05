//! What:
//!  The bytes the signal handler writes.
//! Why:
//!  The handler cannot be tested by sending the test process a signal,
//! and it may not allocate;
//! the line is built apart from it,
//!  so its text and its handling of large numbers can be checked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(signalLine(15, uptimeNanos)).toBe('monochromatic-git-policy-cli-service: exiting reason=signal-15 uptime_ms=...\n');
//! ```

/// The line and the handled signals.
use super::{HANDLED, LINE_CAPACITY, signal_datagram, signal_line};
use crate::service_timing::LOG_PREFIX;

/// The line states the signal and the uptime in whole milliseconds,
/// ends in a newline,
/// and starts like the service's other lines.
#[test]
fn the_line_states_the_signal_and_the_uptime_in_milliseconds() {
    let uptime_millis: u64 = 12_345;
    let at_ms: u64 = 1_793_648_794_987;
    let (bytes, length) = signal_line(libc::SIGTERM, uptime_millis * 1_000_000 + 999_999, at_ms, false);
    let text: &str = std::str::from_utf8(&bytes[..length]).expect("text");
    assert_eq!(text, format!("{LOG_PREFIX}exiting reason=signal-{} uptime_ms={uptime_millis} at_ms={at_ms}\n", libc::SIGTERM));
}

/// A zero uptime and the largest one both fit the buffer.
#[test]
fn the_extremes_fit_the_buffer() {
    let (zero, zero_length) = signal_line(libc::SIGHUP, 0, 0, false);
    assert!(std::str::from_utf8(&zero[..zero_length]).expect("text").ends_with("uptime_ms=0 at_ms=0\n"));
    let (largest, largest_length) = signal_line(libc::c_int::MAX, u64::MAX, u64::MAX, true);
    assert!(largest_length < LINE_CAPACITY);
    assert!(std::str::from_utf8(&largest[..largest_length]).expect("text").ends_with(format!("uptime_ms={} at_ms={}\n", u64::MAX / 1_000_000, u64::MAX).as_str()));
}

/// The signals that end the service are the three that a manager or a terminal sends.
#[test]
fn the_handled_signals_are_termination_interrupt_and_hangup() {
    assert_eq!(HANDLED, [libc::SIGTERM, libc::SIGINT, libc::SIGHUP]);
}

/// The entry the handler sends on the native socket is a journal entry with the message of the line, the priority and the
/// time of the event as a field, and it fits the buffer at the extremes.
#[test]
fn the_native_entry_carries_the_message_the_priority_and_the_time() {
    let at_ms: u64 = 1_793_648_794_987;
    let (bytes, length) = signal_datagram(libc::SIGTERM, 12_345 * 1_000_000, at_ms);
    let text: &str = std::str::from_utf8(&bytes[..length]).expect("text");
    let fields: Vec<&str> = text.lines().collect();
    assert_eq!(
        fields,
        vec![
            "PRIORITY=6".to_string(),
            "SYSLOG_IDENTIFIER=monochromatic-git-policy-cli-service".to_string(),
            format!("MESSAGE=exiting reason=signal-{} uptime_ms=12345 at_ms={at_ms}", libc::SIGTERM),
            format!("CLI_GIT_AT_MS={at_ms}"),
        ]
    );
    let (largest, largest_length) = signal_datagram(libc::c_int::MAX, u64::MAX, u64::MAX);
    assert!(largest_length < LINE_CAPACITY);
    assert!(std::str::from_utf8(&largest[..largest_length]).expect("text").ends_with(format!("CLI_GIT_AT_MS={}\n", u64::MAX).as_str()));
}

/// On the journal's stream the fallback line starts with the priority the native entry has, so a unit with a level filter
/// treats the two paths alike; elsewhere there is no prefix.
#[test]
fn the_line_on_the_journal_stream_carries_the_info_priority() {
    let (on_stream, on_length) = signal_line(libc::SIGTERM, 0, 1, true);
    let (plain, plain_length) = signal_line(libc::SIGTERM, 0, 1, false);
    let streamed: &[u8] = &on_stream[..on_length];
    assert!(streamed.starts_with(b"<6>monochromatic-git-policy-cli-service: exiting"), "{:?}", String::from_utf8_lossy(streamed));
    assert_eq!(&streamed[3..], &plain[..plain_length]);
}
