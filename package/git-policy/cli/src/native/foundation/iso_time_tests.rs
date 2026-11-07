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

/// The current time has the shape and lies after this test was written.
#[test]
fn now_has_the_shape() {
    let now: String = now_iso();
    assert_eq!(now.len(), 24);
    assert!(now.ends_with('Z'));
    assert!(now.as_str() > "2026-10-06T00:00:00.000Z");
}

/// The owner's ruling of 2026-11-07 ("Time stamps: jiff") replaces the hand-written calendar with the published crate, so neither
///  file that held calendar arithmetic holds its constants: the days of an era (146,097) and the shift to the epoch (719,468).
#[test]
fn no_file_holds_hand_written_calendar_arithmetic() {
    let sources: [(&str, &str); 2] = [
        ("foundation/iso_time.rs", include_str!("iso_time.rs")),
        ("conflict/discarded_prune_stamps.rs", include_str!("../conflict/discarded_prune_stamps.rs")),
    ];
    for (name, source) in sources {
        let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
        for needle in ["146_097", "719_468", "days_from_civil", "civil_from_days"] {
            assert!(!code.contains(needle), "{name} still holds `{needle}`: the calendar comes from the jiff crate");
        }
    }
}

/// Known instants, formatted: the epoch, a leap day, the end of a year, and a time before 1970.
#[test]
fn known_instants_format_as_the_wrapper_always_wrote_them() {
    assert_eq!(format_iso_milliseconds(0), "1970-01-01T00:00:00.000Z");
    assert_eq!(format_iso_milliseconds(1_709_210_096_789), "2024-02-29T12:34:56.789Z");
    assert_eq!(format_iso_milliseconds(-1), "1969-12-31T23:59:59.999Z");
    assert_eq!(format_iso_milliseconds(4_102_444_800_000), "2100-01-01T00:00:00.000Z");
    // The crate's range ends a little before the end of year 9999; a later instant is clamped to that end, not misprinted.
    assert_eq!(format_iso_milliseconds(i64::MAX), "9999-12-30T22:00:00.999Z");
    assert_eq!(format_iso_milliseconds(i64::MIN), "-9999-01-02T01:59:59.000Z");
}
