//! Controls for `toISOString` formatting,
//!  checked against values Node 24 prints.

use super::*;

/// Known instants format as JavaScript formats them.
#[test]
fn known_instants_match_to_iso_string() {
    // new Date(0).toISOString()
    assert_eq!(format_iso_milliseconds(0), "1970-01-01T00:00:00.000Z");
    // new Date(1791344041123).toISOString()
    assert_eq!(
        format_iso_milliseconds(1_791_344_041_123),
        "2026-10-07T03:34:01.123Z"
    );
    // new Date(951782400000).toISOString(): a leap day of a century leap year.
    assert_eq!(
        format_iso_milliseconds(951_782_400_000),
        "2000-02-29T00:00:00.000Z"
    );
    // new Date(-1).toISOString()
    assert_eq!(format_iso_milliseconds(-1), "1969-12-31T23:59:59.999Z");
    // new Date(4102444799999).toISOString()
    assert_eq!(
        format_iso_milliseconds(4_102_444_799_999),
        "2099-12-31T23:59:59.999Z"
    );
    // new Date(-62135596800000).toISOString(): the first day of year 1.
    assert_eq!(
        format_iso_milliseconds(-62_135_596_800_000),
        "0001-01-01T00:00:00.000Z"
    );
}

/// Every day of four years,
///  crossing a leap year,
///  maps to consecutive dates.
#[test]
fn consecutive_days_are_consecutive_dates() {
    let mut previous: (i64, u32, u32) = civil_from_days(10_957);
    // new Date(10957 * 86400000).toISOString() is 2000-01-01, and day 10956 is 1999-12-31.
    assert_eq!(previous, (2000, 1, 1));
    assert_eq!(civil_from_days(10_956), (1999, 12, 31));
    for days in 10_958..10_958 + 4 * 366 {
        let current: (i64, u32, u32) = civil_from_days(days);
        let same_month: bool = current.0 == previous.0 && current.1 == previous.1;
        if same_month {
            assert_eq!(current.2, previous.2 + 1, "{days}");
        } else {
            assert_eq!(current.2, 1, "{days}");
            assert!(previous.2 >= 28, "{days}");
        }
        previous = current;
    }
}

/// The current time has the shape and lies after this test was written.
#[test]
fn now_has_the_shape() {
    let now: String = now_iso();
    assert_eq!(now.len(), 24);
    assert!(now.ends_with('Z'));
    assert!(now.as_str() > "2026-10-06T00:00:00.000Z");
}
