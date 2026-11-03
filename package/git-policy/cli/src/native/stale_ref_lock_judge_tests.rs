//! Controls for the margins of the reference-lock judge: each must clear the measurement it was set from.

use super::*;

/// A lock younger than the longest live hold measured may still be live, so the margin must exceed that hold.
#[test]
fn the_margin_exceeds_the_longest_live_reference_step_measured() {
    assert!(MARGIN > LONGEST_LIVE_REF_STEP, "{MARGIN:?} must exceed {LONGEST_LIVE_REF_STEP:?}");
}

/// The margin is whole seconds, the smallest whole number of seconds above the measurement.
#[test]
fn the_margin_is_the_next_whole_second_above_the_measurement() {
    assert_eq!(MARGIN.subsec_nanos(), 0);
    assert!(MARGIN - LONGEST_LIVE_REF_STEP <= Duration::from_secs(1), "{MARGIN:?} is more than a second over the measurement");
}
