//! What:
//!  Controls of the deadline's arithmetic:
//!  a run inside the bound has time left,
//!  the bound itself has none,
//!  and the margin is the part of the value that protects against noise.
//! Why:
//!  The mode's wait and its kill must agree on where the boundary is,
//!  and a margin larger than the value would be a bound that fires at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(remaining(start, start)).toBeGreaterThan(0); expect(deadlinePassed(start, start + PROBE_DEADLINE)).toBe(true);
//! ```

use super::{PROBE_DEADLINE, PROBE_DEADLINE_MARGIN, deadline_from_words, deadline_passed, remaining, the_bound};
use std::time::{Duration, Instant};

/// A just-started run has time left,
/// and the bound itself has none.
#[test]
fn the_bound_is_where_the_wait_ends() {
    let started: Instant = Instant::now();
    assert!(!deadline_passed(started, started, PROBE_DEADLINE));
    assert!(remaining(started, started, PROBE_DEADLINE).is_some_and(|left| return left == PROBE_DEADLINE));
    assert!(deadline_passed(started, started + PROBE_DEADLINE, PROBE_DEADLINE));
    assert_eq!(remaining(started, started + PROBE_DEADLINE, PROBE_DEADLINE), None);
    assert_eq!(remaining(started, started + PROBE_DEADLINE + Duration::from_millis(1), PROBE_DEADLINE), None);
}

/// The margin is part of the value,
/// never larger than it.
#[test]
fn the_margin_is_part_of_the_value() {
    assert!(PROBE_DEADLINE > Duration::ZERO);
    assert!(PROBE_DEADLINE_MARGIN < PROBE_DEADLINE);
}

/// The test-only deadline override takes a positive number of milliseconds
/// and leaves the shipped bound alone otherwise.
#[test]
fn the_test_only_deadline_is_a_positive_number() {
    assert_eq!(deadline_from_words(None), PROBE_DEADLINE);
    assert_eq!(deadline_from_words(Some("900000")), std::time::Duration::from_millis(900000));
    assert_eq!(deadline_from_words(Some("0")), PROBE_DEADLINE);
    assert_eq!(deadline_from_words(Some("soon")), PROBE_DEADLINE);
}

/// A build the release ships uses the shipped bound,
/// whatever the environment says,
/// because the variable's read sits behind the feature a control's build turns
/// on.
#[test]
fn the_shipped_bound_is_what_this_build_uses() {
    assert_eq!(the_bound(), PROBE_DEADLINE, "the release's probe path is the shipped bound; only `--features start-probe-raised-deadline` reads the variable");
}
