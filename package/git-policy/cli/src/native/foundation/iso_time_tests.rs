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

/// The owner's ruling of 2026-11-07 ("Time stamps:
/// jiff") replaces the hand-written calendar with the published crate,
/// so neither
///  file that held calendar arithmetic holds its constants:
/// the days of an era (146,097) and the shift to the epoch (719,468).
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

/// Known instants,
/// formatted:
/// the epoch,
/// a leap day,
/// the end of a year,
/// and a time before 1970.
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

/// The version of the package `name` in the crate's lockfile text.
fn locked_version<'a>(lock: &'a str, name: &str) -> Option<&'a str> {
    let header: String = format!("name = \"{name}\"\nversion = \"");
    let start: usize = lock.find(header.as_str())? + header.len();
    return lock[start..].split('"').next();
}

/// The owner approved the publisher change behind the newer `jiff` releases (2026-11-07:
/// "I'm approving the publisher change for
///  jiff."),
/// so the lock no longer holds `jiff` at the 0.2.24 it was kept at while the trust policy refused `defmt-parser` 1.1.0.
#[test]
fn the_lock_no_longer_holds_jiff_at_the_release_kept_while_the_trust_policy_refused_the_newer_ones() {
    let lock: &str = include_str!("../../../Cargo.lock");
    let version: &str = locked_version(lock, "jiff").expect("jiff is locked");
    assert_ne!(version, "0.2.24", "jiff is still held at the pin; move it to the newest release the policy allows");
}

/// What:
///  Two stamps asked for in one millisecond are different moments,
///  and each is a `toISOString` of a moment this process has not written before.
/// Why:
///   A stamp can name a record,
///   and a name that repeats inside one millisecond drops the second record:
///   the queue answers a name it already knows as known.
/// The clock is a millisecond wide,
/// so the guarantee belongs to the stamp writer,
/// not to the caller that hashes it.
#[test]
fn a_stamp_written_again_in_one_millisecond_is_still_a_moment_of_its_own() {
    /// How many stamps one control writes;
    /// each takes far less than a millisecond,
    /// so this many cannot all fall in different milliseconds.
    const STAMPS: usize = 1000;
    let stamps: Vec<String> = (0..STAMPS).map(|_unused| return now_iso_never_the_same_twice()).collect();
    for stamp in &stamps {
        assert_eq!(stamp.len(), 24, "{stamp} is not a `toISOString` stamp");
        assert!(stamp.ends_with('Z'), "{stamp} is not a `toISOString` stamp");
    }
    let mut distinct: Vec<String> = stamps.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), stamps.len(), "each of the {} stamps is a moment of its own", stamps.len());
    // A stamp is a moment this process asks for, never one before it: the newest reading is not older than this test.
    let newest: &String = distinct.last().expect("the stamps");
    assert!(newest.as_str() > "2026-10-06T00:00:00.000Z", "{newest} is a moment before this control was written");
}
