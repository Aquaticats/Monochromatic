//! Controls for Git's lock-retry schedule.

use super::*;

/// Git's formula:
///  `(750 + random%500) * multiplier / 1000`,
///  rounded down.
#[test]
fn jittered_waits_follow_gits_formula() {
    let state: BackoffState = BackoffState { step: 3, multiplier: 9 };
    assert_eq!(jittered_wait(state, 0.5), 9);
    assert_eq!(jittered_wait(state, 0.0), 6);
    assert_eq!(jittered_wait(state, 0.999), 11);
}

/// The multiplier grows by `2n + 1` (so it is the square of the step) and stops at the cap.
#[test]
fn backoff_squares_then_stops_at_the_cap() {
    let first: BackoffState = next_backoff(INITIAL_BACKOFF);
    assert_eq!(first, BackoffState { step: 2, multiplier: 4 });
    let second: BackoffState = next_backoff(first);
    assert_eq!(second, BackoffState { step: 3, multiplier: 9 });
    let near_cap: BackoffState = BackoffState { step: 31, multiplier: 961 };
    assert_eq!(next_backoff(near_cap), BackoffState { step: 31, multiplier: BACKOFF_MAX_MULTIPLIER });
    // A step that lands on the cap exactly is still a step; only one that would pass it is held back.
    let lands_on_cap: BackoffState = BackoffState { step: 5, multiplier: 989 };
    assert_eq!(next_backoff(lands_on_cap), BackoffState { step: 6, multiplier: BACKOFF_MAX_MULTIPLIER });
    let at_cap: BackoffState = BackoffState { step: 31, multiplier: BACKOFF_MAX_MULTIPLIER };
    assert_eq!(next_backoff(at_cap), at_cap);
}
