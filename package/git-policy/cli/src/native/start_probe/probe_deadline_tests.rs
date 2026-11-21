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

use super::{PROBE_DEADLINE, PROBE_DEADLINE_MARGIN, deadline_passed, remaining};
use std::time::{Duration, Instant};

/// A just-started run has time left,
/// and the bound itself has none.
#[test]
fn the_bound_is_where_the_wait_ends() {
    let started: Instant = Instant::now();
    assert!(!deadline_passed(started, started));
    assert!(remaining(started, started).is_some_and(|left| return left == PROBE_DEADLINE));
    assert!(deadline_passed(started, started + PROBE_DEADLINE));
    assert_eq!(remaining(started, started + PROBE_DEADLINE), None);
    assert_eq!(remaining(started, started + PROBE_DEADLINE + Duration::from_millis(1)), None);
}

/// The margin is part of the value,
/// never larger than it.
#[test]
fn the_margin_is_part_of_the_value() {
    assert!(PROBE_DEADLINE > Duration::ZERO);
    assert!(PROBE_DEADLINE_MARGIN < PROBE_DEADLINE);
}
