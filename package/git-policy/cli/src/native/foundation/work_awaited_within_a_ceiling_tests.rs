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

/// What:
///  A thread the system refuses is told apart from a job that did not answer,
///  and the job is not run at all.
/// Why:
///  A caller that holds something until the work ends must give it back when nothing ran,
/// and only this answer says that nothing ran:
/// a job that was left waiting may still do its work and still holds what it was given.
#[test]
fn a_refused_thread_is_not_a_job_that_did_not_answer() {
    let refused: ThreadsRefusedForTheControl = a_control_refuses_every_thread();
    let ran: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let touched: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&ran);
    let answered: TheAnswerOfAThreadOfWork<u8> = work_within_reporting(Duration::from_secs(5), move || {
        touched.store(true, std::sync::atomic::Ordering::SeqCst);
        return 7;
    });
    assert!(matches!(answered, TheAnswerOfAThreadOfWork::NoThreadToStart), "a refused thread is its own answer");
    assert!(!ran.load(std::sync::atomic::Ordering::SeqCst), "the job of a refused thread never ran");
    drop(refused);
    assert_eq!(work_within(Duration::from_secs(5), || return 7), Some(7), "with threads the job runs and answers");
    assert!(matches!(work_within_reporting(Duration::from_secs(5), || return 7), TheAnswerOfAThreadOfWork::Ran(7)), "and the same call reports that it ran");
}
