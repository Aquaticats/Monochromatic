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
