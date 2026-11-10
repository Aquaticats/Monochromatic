//! What:
//! Unit tests of the output cap in `output_cap.rs`:
//! the decision that turns it on,
//! and the copy loop that stops at a byte count.
//! Why:
//! The byte boundary is the whole rule,
//! and only a unit test can hand the loop an input of exactly the cap,
//! one byte more,
//! and a writer that refuses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // output-cap.unit.test.ts
//! describe('forward', () => { ... });
//! ```

/// What:
/// Imports the decision and the copy loop under test.
/// Why:
/// The tests call them directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { capApplies, forward } from './output_cap.ts';
/// ```
use crate::output_cap::{Forwarding, cap_applies, forward};

/// Imports `Cursor`,
/// a byte slice that can be read,
/// and the traits that make one writable and readable.
use std::io::{self, Cursor, Read, Write};

/// The cap these cases use:
/// a handful of bytes,
/// so the boundary is readable in a failure message.
const CAP: u64 = 8;

/// A writer that always refuses,
/// the shape of a reader that has gone away.
struct RefusingWriter;

/// Writing anything to it fails with the error a closed pipe gives.
impl Write for RefusingWriter {
    fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
        return Err(io::Error::from(io::ErrorKind::BrokenPipe));
    }

    fn flush(&mut self) -> io::Result<()> {
        return Ok(());
    }
}

/// A writer that keeps every byte and fails to hand them over.
struct UnflushableWriter {
    /// What was written before the flush.
    kept: Vec<u8>,
}

/// Writing succeeds, so only the flush can fail.
impl Write for UnflushableWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.kept.extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        return Err(io::Error::other("the caller is gone"));
    }
}

/// A reader that fails on its first read.
struct BrokenReader;

/// Reading anything from it fails with an input/output error.
impl Read for BrokenReader {
    fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        return Err(io::Error::other("the pipe broke"));
    }
}

/// The cap turns on an output nothing is watching and never on a terminal's.
#[test]
fn the_cap_applies_only_to_an_output_that_is_not_a_terminal() {
    assert!(!cap_applies(true));
    assert!(cap_applies(false));
}

/// An output shorter than the cap is passed on whole and reported complete.
#[test]
fn an_output_under_the_cap_is_complete() {
    let input: &[u8] = b"short";
    let mut from: Cursor<&[u8]> = Cursor::new(input);
    let mut to: Vec<u8> = Vec::new();

    let outcome = forward(&mut from, &mut to, CAP);

    assert!(matches!(outcome, Forwarding::Complete), "{outcome:?}");
    assert_eq!(to, input);
}

/// An output of exactly the cap is not past it:
/// the tool printed everything it had.
#[test]
fn an_output_of_exactly_the_cap_is_complete() {
    let input: &[u8] = b"12345678";
    let mut from: Cursor<&[u8]> = Cursor::new(input);
    let mut to: Vec<u8> = Vec::new();

    let outcome = forward(&mut from, &mut to, CAP);

    assert!(matches!(outcome, Forwarding::Complete), "{outcome:?}");
    assert_eq!(to, input);
}

/// One byte past the cap stops the copy there:
/// exactly the cap reaches the caller and the rest is dropped.
#[test]
fn one_byte_past_the_cap_stops_at_the_cap() {
    let input: &[u8] = b"123456789";
    let mut from: Cursor<&[u8]> = Cursor::new(input);
    let mut to: Vec<u8> = Vec::new();

    let outcome = forward(&mut from, &mut to, CAP);

    assert!(matches!(outcome, Forwarding::Capped), "{outcome:?}");
    assert_eq!(to, &input[..CAP as usize]);
}

/// A caller that stops listening is told apart from the cap,
/// and the bytes it already took stay written.
#[test]
fn a_writer_that_refuses_ends_the_copy_with_its_error() {
    let input: &[u8] = b"short";
    let mut from: Cursor<&[u8]> = Cursor::new(input);
    let mut to: RefusingWriter = RefusingWriter;

    let outcome = forward(&mut from, &mut to, CAP);

    assert!(matches!(outcome, Forwarding::Failed(_)), "{outcome:?}");
}

/// A flush that fails is an output that did not arrive,
/// even when every byte was accepted.
#[test]
fn a_flush_that_fails_is_a_failed_output() {
    let input: &[u8] = b"short";
    let mut from: Cursor<&[u8]> = Cursor::new(input);
    let mut to: UnflushableWriter = UnflushableWriter { kept: Vec::new() };

    let outcome = forward(&mut from, &mut to, CAP);

    assert!(matches!(outcome, Forwarding::Failed(_)), "{outcome:?}");
    assert_eq!(to.kept, input);
}

/// A pipe that cannot be read fails the copy rather than passing for a complete output.
#[test]
fn a_reader_that_fails_is_a_failed_output() {
    let mut from: BrokenReader = BrokenReader;
    let mut to: Vec<u8> = Vec::new();

    let outcome = forward(&mut from, &mut to, CAP);

    assert!(matches!(outcome, Forwarding::Failed(_)), "{outcome:?}");
    assert!(to.is_empty());
}
