//! Controls for the least traffic a push's connection must carry to count as progress:
//! the variable that sets it,
//! and the value a run gets when the variable says nothing usable.
//! Why:
//!  A typo must not remove the rule,
//! so a zero,
//! negative,
//! fractional,
//! empty or non-text value is no setting at all and leaves the measured default,
//! as every bounded value of this crate does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(minimumRateBytes([['CLI_GIT_PUSH_MINIMUM_RATE_BYTES', '0']])).toBe(1024);
//! ```

use crate::bounded_push::{DEFAULT_PUSH_MINIMUM_RATE_BYTES, PUSH_MINIMUM_RATE_VARIABLE, push_minimum_rate_bytes};
use std::ffi::OsString;

/// The same setting in the environment's shape.
fn setting(value: &str) -> Vec<(OsString, OsString)> {
    return vec![(OsString::from(PUSH_MINIMUM_RATE_VARIABLE), OsString::from(value))];
}

/// A run that sets nothing is judged on the measured default.
#[test]
fn a_run_without_the_variable_gets_the_measured_default() {
    assert_eq!(push_minimum_rate_bytes(&[]), DEFAULT_PUSH_MINIMUM_RATE_BYTES);
    let other: Vec<(OsString, OsString)> = vec![(OsString::from("CLI_GIT_PUSH_DEADLINE_SECONDS"), OsString::from("7"))];
    assert_eq!(push_minimum_rate_bytes(&other), DEFAULT_PUSH_MINIMUM_RATE_BYTES, "another variable sets nothing");
}

/// A positive whole number of bytes is the floor,
/// and the last of repeated entries wins,
/// as in an environment block.
#[test]
fn a_positive_whole_number_of_bytes_sets_the_floor_and_the_last_one_wins() {
    assert_eq!(push_minimum_rate_bytes(&setting("64")), 64);
    let repeated: Vec<(OsString, OsString)> = [
        (OsString::from(PUSH_MINIMUM_RATE_VARIABLE), OsString::from("64")),
        (OsString::from(PUSH_MINIMUM_RATE_VARIABLE), OsString::from("4096")),
    ]
    .to_vec();
    assert_eq!(push_minimum_rate_bytes(&repeated), 4096);
}

/// No unusable value is a floor of nothing:
/// each leaves the measured default,
/// so a typo cannot remove the rule.
#[test]
fn no_unusable_value_takes_the_rule_away() {
    for value in ["0", "-1", "1.5", "", "  ", "many", "1e6", "0x10"] {
        assert_eq!(
            push_minimum_rate_bytes(&setting(value)),
            DEFAULT_PUSH_MINIMUM_RATE_BYTES,
            "the value {value:?} is no setting at all"
        );
    }
}

/// A value that is not text is no setting either.
#[test]
fn a_value_that_is_not_text_is_no_setting() {
    use std::os::unix::ffi::OsStringExt;
    let undecodable: Vec<(OsString, OsString)> = vec![(OsString::from(PUSH_MINIMUM_RATE_VARIABLE), OsString::from_vec(vec![0xff, 0xfe]))];
    assert_eq!(push_minimum_rate_bytes(&undecodable), DEFAULT_PUSH_MINIMUM_RATE_BYTES);
}
