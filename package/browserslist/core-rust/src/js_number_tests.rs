//! What:
//!  Tests for the JavaScript number-reader ports against the values JavaScript gives.
//! Why:
//!  The expected values were taken from `parseInt` and `parseFloat` semantics, so the ports cannot drift silently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('number readers', () => { /* parseInt, parseFloat, parseLatestFloat */ });
//! ```

use crate::js_number::{parse_float, parse_int_prefix, parse_latest_float};

/// `parseInt` keeps the digit prefix and gives NaN (`None`) without one.
#[test]
fn int_prefix_cases() {
    assert_eq!(parse_int_prefix("26"), Some(26));
    assert_eq!(parse_int_prefix("4abc"), Some(4));
    assert_eq!(parse_int_prefix("007"), Some(7));
    assert_eq!(parse_int_prefix(""), None);
    assert_eq!(parse_int_prefix("TP"), None);
    assert_eq!(parse_int_prefix("-3"), None);
    assert_eq!(parse_int_prefix("99999999999999999999999"), None);
}

/// `parseFloat` reads the longest numeric prefix.
#[test]
fn float_prefix_cases() {
    assert_eq!(parse_float("5.5"), 5.5);
    assert_eq!(parse_float("12.1"), 12.1);
    assert_eq!(parse_float("11"), 11.0);
    assert_eq!(parse_float("4.4.3"), 4.4);
    assert_eq!(parse_float("15.2-15.3"), 15.2);
    assert_eq!(parse_float("13.52"), 13.52);
    assert_eq!(parse_float("1e3x"), 1000.0);
    assert_eq!(parse_float("1e"), 1.0);
    assert_eq!(parse_float("2e-1"), 0.2);
    assert_eq!(parse_float("-2.5"), -2.5);
    assert_eq!(parse_float("+7"), 7.0);
    assert_eq!(parse_float(".5"), 0.5);
    assert_eq!(parse_float("5."), 5.0);
}

/// Text without a numeric prefix is NaN, which compares false with everything.
#[test]
fn float_nan_cases() {
    for text in ["", "TP", ".", "-", "+.", "e5", "abc"] {
        assert_eq!(parse_float(text).is_nan(), true, "{text:?}");
    }
}

/// `parseLatestFloat` reads the part after the first dash, falling back to the whole text.
#[test]
fn latest_float_cases() {
    assert_eq!(parse_latest_float("15.2-15.3"), 15.3);
    assert_eq!(parse_latest_float("4.4.3-4.4.4"), 4.4);
    assert_eq!(parse_latest_float("10-12"), 12.0);
    assert_eq!(parse_latest_float("13.52"), 13.52);
    assert_eq!(parse_latest_float("9-"), 9.0);
    assert_eq!(parse_latest_float("TP").is_nan(), true);
}
