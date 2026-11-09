//! Controls for the bounded job:
//! a value that comes in time,
//! one that never comes,
//! and the ceiling as the only bound.

use super::*;

/// A value the job answers at once is the caller's value.
#[test]
fn a_job_that_answers_at_once_gives_its_value() {
    assert_eq!(work_within(Duration::from_secs(5), || return 7), Some(7));
}

/// What:
///  A job that never answers gives up at the ceiling,
/// and the caller's wait is that ceiling.
/// Why:
///  This is the whole bound:
/// the job is left behind rather than waited for,
/// so the call ends even though the
///  work does not.
#[test]
fn a_job_that_never_answers_gives_up_at_its_ceiling() {
    let ceiling: Duration = Duration::from_millis(50);
    let (_hold, waiting) = channel::<()>();
    let started = std::time::Instant::now();
    let answer: Option<u8> = work_within(ceiling, move || {
        let _never = waiting.recv();
        return 1;
    });
    let waited: Duration = started.elapsed();
    assert_eq!(answer, None);
    // The ceiling is a floor for how long this may take and a loose roof for how long it does: a loaded host adds
    //  scheduling time to the wait, never work.
    assert!(waited >= ceiling, "the wait ended before the ceiling: {waited:?}");
    assert!(waited < ceiling + Duration::from_secs(10), "the wait outlived its ceiling: {waited:?}");
}

/// What:
///  A line whose stream never accepts it gives no answer at the ceiling,
/// and a line a stream accepts is delivered.
/// Why:
///  The write is the way a warning and a notice reach the person,
/// so both answers must be values the caller
///  can act on.
#[test]
fn a_line_is_delivered_or_left_waiting_at_the_ceiling() {
    let delivered: Option<std::io::Result<()>> = write_within(Duration::from_secs(5), Vec::<u8>::new(), String::from("a line\n"));
    assert!(matches!(delivered, Some(Ok(()))), "a stream that accepts the line answers with it: {delivered:?}");
    let (sender, receiver) = channel::<()>();
    let blocked: Option<std::io::Result<()>> = write_within(Duration::from_millis(50), BlockedStream { _sender: sender, waiting: receiver }, String::from("a line\n"));
    assert!(blocked.is_none(), "a stream that never accepts the line gives no answer: {blocked:?}");
}

/// A stream whose write waits for an answer that never comes.
struct BlockedStream {
    /// Kept alive so the wait below has no end.
    _sender: std::sync::mpsc::Sender<()>,
    /// Waited on,
    /// never answered.
    waiting: std::sync::mpsc::Receiver<()>,
}

impl Write for BlockedStream {
    /// Waits for an answer that never comes.
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let _never = self.waiting.recv();
        return Ok(buffer.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}
