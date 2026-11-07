//! Unit tests of the retry of a refused start:
//! every branch with an injected refusal and a recording pause,
//! so nothing waits.

use std::io::{Error, ErrorKind};
use std::time::Duration;

use crate::start_retry::{START_RETRY_BUDGET, is_refusal, retry_refused_start_with};

/// The refusal the system reports when it cannot create another process.
fn refusal() -> Error {
    return Error::from(ErrorKind::WouldBlock);
}

/// A start that is refused `times` times and then succeeds with its own attempt number.
fn refused_then_ok(times: u32) -> impl FnMut() -> std::io::Result<u32> {
    let mut attempts: u32 = 0;
    return move || {
        attempts += 1;
        if attempts <= times {
            return Err(refusal());
        }
        return Ok(attempts);
    };
}

#[test]
fn a_start_that_works_the_first_time_pauses_never_and_reports_no_refusal() {
    let mut pauses: Vec<Duration> = Vec::new();
    let (outcome, report) =
        retry_refused_start_with(START_RETRY_BUDGET, &mut |delay: Duration| pauses.push(delay), &mut refused_then_ok(0));
    assert_eq!(outcome.expect("the start works"), 1);
    assert!(pauses.is_empty());
    assert_eq!(report.refusals, 0);
    assert_eq!(report.paused, Duration::ZERO);
}

#[test]
fn a_refused_start_is_tried_again_after_pauses_that_double() {
    let mut pauses: Vec<Duration> = Vec::new();
    let (outcome, report) =
        retry_refused_start_with(START_RETRY_BUDGET, &mut |delay: Duration| pauses.push(delay), &mut refused_then_ok(3));
    assert_eq!(outcome.expect("the fourth try works"), 4);
    assert_eq!(pauses.len(), 3);
    assert_eq!(pauses[1], pauses[0] * 2, "{pauses:?}");
    assert_eq!(pauses[2], pauses[1] * 2, "{pauses:?}");
    assert_eq!(report.refusals, 3);
    assert_eq!(report.paused, pauses.iter().sum::<Duration>());
}

#[test]
fn pauses_stop_growing_at_the_longest_pause_and_the_total_stays_within_the_budget() {
    let mut pauses: Vec<Duration> = Vec::new();
    let (outcome, report) = retry_refused_start_with(
        START_RETRY_BUDGET,
        &mut |delay: Duration| pauses.push(delay),
        &mut || -> std::io::Result<()> { return Err(refusal()) },
    );
    let error = outcome.expect_err("the shortage outlasts the budget");
    assert!(is_refusal(&error), "{error}");
    assert!(pauses.iter().sum::<Duration>() <= START_RETRY_BUDGET, "{pauses:?}");
    let longest = pauses.iter().max().copied().expect("at least one pause");
    assert!(pauses.iter().rev().take(2).all(|pause| return *pause == longest), "{pauses:?}");
    assert_eq!(report.refusals as usize, pauses.len() + 1, "the last refusal is not followed by a pause");
    assert_eq!(report.paused, pauses.iter().sum::<Duration>());
}

#[test]
fn a_start_refused_with_no_budget_fails_at_once_without_pausing() {
    let mut pauses: Vec<Duration> = Vec::new();
    let (outcome, report) = retry_refused_start_with(
        Duration::ZERO,
        &mut |delay: Duration| pauses.push(delay),
        &mut || -> std::io::Result<()> { return Err(refusal()) },
    );
    assert!(outcome.is_err());
    assert!(pauses.is_empty());
    assert_eq!(report.refusals, 1);
}

#[test]
fn any_other_start_error_is_returned_at_once_with_its_kind_and_no_pause() {
    for kind in [ErrorKind::NotFound, ErrorKind::PermissionDenied, ErrorKind::InvalidInput] {
        let mut pauses: Vec<Duration> = Vec::new();
        let mut attempts: u32 = 0;
        let (outcome, report) = retry_refused_start_with(
            START_RETRY_BUDGET,
            &mut |delay: Duration| pauses.push(delay),
            &mut || -> std::io::Result<()> {
                attempts += 1;
                return Err(Error::from(kind));
            },
        );
        assert_eq!(outcome.expect_err("not retried").kind(), kind);
        assert_eq!(attempts, 1, "{kind:?}");
        assert!(pauses.is_empty(), "{kind:?}");
        assert_eq!(report.refusals, 0, "{kind:?}");
    }
}

/// `EAGAIN` on Linux is errno 11 (named here,
/// not in the assertions).
#[cfg(target_os = "linux")]
const LINUX_EAGAIN: i32 = 11;

#[cfg(target_os = "linux")]
#[test]
fn the_systems_own_eagain_is_the_refusal_that_is_retried() {
    assert!(is_refusal(&Error::from_raw_os_error(LINUX_EAGAIN)));
    let mut pauses: Vec<Duration> = Vec::new();
    let mut attempts: u32 = 0;
    let (outcome, _report) = retry_refused_start_with(START_RETRY_BUDGET, &mut |delay: Duration| pauses.push(delay), &mut || {
        attempts += 1;
        if attempts == 1 {
            return Err(Error::from_raw_os_error(LINUX_EAGAIN));
        }
        return Ok(attempts);
    });
    assert_eq!(outcome.expect("the second try works"), 2);
    assert_eq!(pauses.len(), 1);
}
