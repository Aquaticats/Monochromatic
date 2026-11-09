//! What:
//!  The counters a command's record states for the time it spent outside its phases,
//!  and the writer that measures one of them.
//! Why:
//!  A counter that adds the wrong span,
//!  or forgets a removal that never finished,
//!  makes `otherMs` look explained when it is not;
//! a writer that measures nothing
//!  would show every commit as paying nothing for what it prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(outsideThePhases().scratchRemoval).toBe(10);
//! ```

/// The counters and the measured stream.
use super::{MeasuredStandardError, note_scratch_removal, note_standard_error, outside_the_phases};
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::time::Duration;

/// A stream that spends time writing,
/// so a control can make the measured span reach a whole millisecond,
/// and that keeps what it was given.
#[derive(Clone, Debug)]
struct Slow {
    /// The bytes it was given,
    /// shared with the control that reads them.
    written: Rc<RefCell<Vec<u8>>>,
}

impl Slow {
    /// A stream that has written nothing yet.
    fn new() -> Self {
        return Self { written: Rc::new(RefCell::new(Vec::new())) };
    }
}

impl Write for Slow {
    /// Keeps the bytes after spending a moment,
    /// as a pipe whose reader is slow does.
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        std::thread::sleep(Duration::from_millis(3));
        self.written.borrow_mut().extend_from_slice(buffer);
        return Ok(buffer.len());
    }

    /// Spends a moment,
    /// as a flush of a slow pipe does.
    fn flush(&mut self) -> std::io::Result<()> {
        std::thread::sleep(Duration::from_millis(3));
        return Ok(());
    }
}

/// Each removal adds its own span,
/// and one that never finished is counted apart.
#[test]
fn every_removal_adds_its_span_and_an_unfinished_one_is_counted() {
    note_scratch_removal(Duration::from_millis(7), true);
    note_scratch_removal(Duration::from_micros(900), true);
    note_scratch_removal(Duration::from_millis(3), false);
    let outside = outside_the_phases();
    assert_eq!(outside.scratch_removal, Duration::from_millis(10), "7 ms plus 0.9 ms, each rounded down");
    assert_eq!(outside.removals_that_did_not_finish, 1);
    assert_eq!(outside.standard_error, Duration::ZERO);
}

/// A write and a flush each add the time they took,
/// and the bytes still reach the stream underneath.
#[test]
fn a_write_and_a_flush_add_the_time_they_took() {
    let slow: Slow = Slow::new();
    let kept: Rc<RefCell<Vec<u8>>> = Rc::clone(&slow.written);
    let mut stream: MeasuredStandardError<Slow> = MeasuredStandardError::over(slow);
    stream.write_all(b"a line the person reads\n").expect("written");
    stream.flush().expect("flushed");
    let outside = outside_the_phases();
    assert!(outside.standard_error >= Duration::from_millis(6), "one write and one flush of 3 ms: {outside:?}");
    assert_eq!(kept.borrow().as_slice(), b"a line the person reads\n");
    assert_eq!(outside.scratch_removal, Duration::ZERO);
    assert_eq!(outside.removals_that_did_not_finish, 0);
}

/// A span the counter cannot hold is held at its end instead of wrapping around it.
#[test]
fn a_span_larger_than_the_counter_is_held_at_its_end() {
    note_standard_error(Duration::from_secs(u64::MAX / 2));
    note_standard_error(Duration::from_secs(u64::MAX / 2));
    assert_eq!(outside_the_phases().standard_error, Duration::from_millis(u64::MAX / 1000));
}
