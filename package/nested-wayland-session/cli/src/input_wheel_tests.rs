//! Unit tests for the notch conversion.

use super::notches_to_v120;

/// Whole notches are 120 units each,
///  with the sign kept.
#[test]
fn whole_notches_are_120_units() {
    assert_eq!(notches_to_v120(1.0), 120);
    assert_eq!(notches_to_v120(-3.0), -360);
    assert_eq!(notches_to_v120(0.0), 0);
}

/// Fractions become proportional units.
#[test]
fn fractional_notches_are_proportional_units() {
    assert_eq!(notches_to_v120(0.25), 30);
    assert_eq!(notches_to_v120(-1.5), -180);
    assert_eq!(notches_to_v120(1.0 / 120.0), 1);
}

/// An amount below half a unit rounds to nothing.
#[test]
fn amounts_below_half_a_unit_round_to_zero() {
    assert_eq!(notches_to_v120(0.004), 0);
    assert_eq!(notches_to_v120(-0.004), 0);
}
