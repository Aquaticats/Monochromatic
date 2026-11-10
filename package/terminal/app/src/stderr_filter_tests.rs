// What:     Unit tests for `stderr_filter.rs`, pulled in by
//           `#[cfg(test)] #[path = "stderr_filter_tests.rs"] mod tests;` at
//           the bottom of `stderr_filter.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of stderr_filter.
// Why:      Keep the tests beside the code without inflating
//           `stderr_filter.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::{forward_filtered_stderr, should_suppress_stderr_line};`
//           imports the private predicate and the forwarding worker from the
//           parent module.
// Why:      Tests check the suppression rule directly and drive the worker with
//           scripted streams instead of the process-wide pipe.
use super::{forward_filtered_stderr, should_suppress_stderr_line};

// What:     `use std::io::{self, Read};` imports the I/O error type and the
//           byte-stream trait the scripted reader implements.
// Why:      A read failure is what a test must place in the stream; a real pipe
//           cannot be made to fail on demand.
use std::io::{self, Read};

// What:     `const FORWARDED_LINE: &[u8] = b"kept diagnostic\n";` is the one
//           record the worker must pass to the original stderr untouched.
// Why:      Every case asserts on the same kept record, so one name serves both.
const FORWARDED_LINE: &[u8] = b"kept diagnostic\n";

// What:     `const READ_FAILURE_MESSAGE: &str = "pipe read failed";` is the text
//           the scripted reader reports when it fails.
// Why:      The failure report must quote the error it caught, so the test needs
//           text it can look for verbatim.
const READ_FAILURE_MESSAGE: &str = "pipe read failed";

// What:     `const LINE_AFTER_FAILURE: &[u8] = b"never forwarded\n";` is a record
//           scripted after the read failure.
// Why:      The worker stops at a read error, so this record proves nothing is
//           forwarded afterwards.
const LINE_AFTER_FAILURE: &[u8] = b"never forwarded\n";

// What:     `struct ScriptedReader { steps: ... }` replays a fixed sequence of
//           read outcomes.
// Why:      Injecting an I/O error into the worker needs a stream the test
//           controls byte by byte.
struct ScriptedReader {
    // What:     `steps` holds the queued outcomes; a spent queue reads as EOF.
    // Why:      The worker ends on EOF, so the queue needs no sentinel record.
    steps: std::vec::IntoIter<io::Result<Vec<u8>>>,
}

impl Read for ScriptedReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self.steps.next() {
            None => return Ok(0),
            Some(Err(error)) => return Err(error),
            Some(Ok(bytes)) => {
                let copied = bytes.len().min(buffer.len());
                buffer[..copied].copy_from_slice(&bytes[..copied]);
                return Ok(copied);
            }
        }
    }
}

#[test]
fn suppresses_ghostty_unimplemented_osc_callback() {
    assert!(should_suppress_stderr_line(
        b"debug(stream): unimplemented OSC callback: .{ .context_signal = .{} }\n",
    ));
}

#[test]
fn keeps_other_ghostty_debug_lines() {
    assert!(!should_suppress_stderr_line(
        b"debug(stream): some other Ghostty diagnostic\n",
    ));
}

#[test]
fn keeps_non_utf8_lines_without_marker() {
    assert!(!should_suppress_stderr_line(b"\xff\xfe\xfd\n"));
}

#[test]
fn reports_a_read_error_on_the_original_stderr() {
    let mut scripted_reader = ScriptedReader {
        steps: vec![
            Ok(FORWARDED_LINE.to_vec()),
            Err(io::Error::other(READ_FAILURE_MESSAGE)),
            Ok(LINE_AFTER_FAILURE.to_vec()),
        ]
        .into_iter(),
    };
    let mut forwarded = Vec::new();
    forward_filtered_stderr(&mut scripted_reader, &mut forwarded);
    let forwarded_line = std::str::from_utf8(FORWARDED_LINE).expect("the kept line is UTF-8");
    let line_after_failure =
        std::str::from_utf8(LINE_AFTER_FAILURE).expect("the post-failure line is UTF-8");
    let report = String::from_utf8_lossy(forwarded.as_slice()).into_owned();
    assert!(
        report.starts_with(forwarded_line),
        "the kept line should still reach the original stderr first, got: {report}"
    );
    assert!(
        report.contains(READ_FAILURE_MESSAGE),
        "the report should name the read error, got: {report}"
    );
    assert!(
        !report.contains(line_after_failure),
        "no record may be forwarded after the read error, got: {report}"
    );
}

#[test]
fn keeps_forwarding_after_an_interrupted_read() {
    let mut scripted_reader = ScriptedReader {
        steps: vec![
            Err(io::Error::from(io::ErrorKind::Interrupted)),
            Ok(FORWARDED_LINE.to_vec()),
        ]
        .into_iter(),
    };
    let mut forwarded = Vec::new();
    forward_filtered_stderr(&mut scripted_reader, &mut forwarded);
    assert_eq!(
        forwarded.as_slice(),
        FORWARDED_LINE,
        "an interrupted read must not end forwarding"
    );
}
