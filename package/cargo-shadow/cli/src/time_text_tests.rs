//! The unit tests of the timestamp reading and writing.

use crate::time_text::{MS_PER_MINUTE, format_iso_millis, parse_rfc3339_millis, publish_time_cutoff};

/// 2026-10-29T12:00:00Z in milliseconds,
/// derived once from the same calendar the code under test implements,
/// checked
/// against JavaScript's `Date.parse('2026-10-29T12:00:00Z')` when the tests were written.
const NOON_2026_10_29: i64 = 1_793_275_200_000;

#[test]
fn the_sparse_index_shape_without_a_fraction_reads() {
    assert_eq!(parse_rfc3339_millis("2026-10-29T12:00:00Z"), Some(NOON_2026_10_29));
}

#[test]
fn the_api_shape_with_microseconds_and_an_offset_reads() {
    assert_eq!(parse_rfc3339_millis("2026-10-29T12:00:00.123456+00:00"), Some(NOON_2026_10_29 + 123));
}

#[test]
fn a_positive_offset_means_an_earlier_instant() {
    assert_eq!(parse_rfc3339_millis("2026-10-29T14:00:00+02:00"), Some(NOON_2026_10_29));
    assert_eq!(parse_rfc3339_millis("2026-10-29T10:30:00-01:30"), Some(NOON_2026_10_29));
}

#[test]
fn text_that_is_not_a_timestamp_is_none() {
    for text in ["", "yesterday", "2026-10-29", "2026-13-29T12:00:00Z", "2026-10-29T12:00:00", "2026-10-29T12:00:00.Z"] {
        assert_eq!(parse_rfc3339_millis(text), None, "{text:?}");
    }
}

#[test]
fn the_epoch_and_a_leap_day_round_trip() {
    for millis in [0, 951_782_400_000, NOON_2026_10_29, -86_400_000] {
        assert_eq!(parse_rfc3339_millis(&format_iso_millis(millis)), Some(millis), "{millis}");
    }
}

#[test]
fn formatting_matches_the_javascript_iso_string() {
    assert_eq!(format_iso_millis(NOON_2026_10_29 + 7), "2026-10-29T12:00:00.007Z");
    assert_eq!(format_iso_millis(0), "1970-01-01T00:00:00.000Z");
}

#[test]
fn the_cutoff_is_now_minus_the_age_in_cargos_shape() {
    let one_day_minutes: u64 = 1440;
    assert_eq!(publish_time_cutoff(NOON_2026_10_29, one_day_minutes), "2026-10-28T12:00:00Z");
    assert_eq!(publish_time_cutoff(NOON_2026_10_29 + MS_PER_MINUTE, 1), "2026-10-29T12:00:00Z");
}
