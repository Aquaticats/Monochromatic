//! Controls for the margins of the reference-lock judge:
//! each must clear the measurement it was set from.

use super::*;

/// A lock younger than the longest live hold measured may still be live,
/// so the margin must exceed that hold.
#[test]
fn the_margin_exceeds_the_longest_live_reference_step_measured() {
    assert!(MARGIN > LONGEST_LIVE_REF_STEP, "{MARGIN:?} must exceed {LONGEST_LIVE_REF_STEP:?}");
}

/// The margin is the multiple of the measurement,
/// rounded up to whole seconds and no further.
#[test]
fn the_margin_is_the_multiple_of_the_measurement_rounded_up_to_a_whole_second() {
    let wanted: Duration = LONGEST_LIVE_REF_STEP * u32::try_from(MARGIN_MULTIPLE).expect("a small multiple");
    assert_eq!(MARGIN.subsec_nanos(), 0);
    assert!(MARGIN >= wanted, "{MARGIN:?} must cover {wanted:?}");
    assert!(MARGIN - wanted < Duration::from_secs(1), "{MARGIN:?} is a second or more over {wanted:?}");
}
