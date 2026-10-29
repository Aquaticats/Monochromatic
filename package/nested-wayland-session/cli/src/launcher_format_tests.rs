//! Answer text for taskbar progress, including adversarial URIs.

/// Brings the formatters and the update record into scope.
use super::*;

/// Builds an update with the given URI and progress for formatting checks.
fn update(app_uri: &str, at_ms: u64, progress: f64) -> LauncherUpdate {
    return LauncherUpdate {
        at_ms,
        app_uri: app_uri.to_owned(),
        progress,
        progress_visible: true,
        count: 3,
        count_visible: false,
        urgent: true,
    }
}

/// Ordinary URIs keep their letters and separators, while every delimiter is escaped.
#[test]
fn uri_encoding_escapes_separators_and_keeps_plain_uris() {
    assert_eq!(encode_uri("application://org.example.app.desktop"), "application://org.example.app.desktop");
    assert_eq!(encode_uri("a b;c=d\ne\rf%g"), "a%20b%3Bc%3Dd%0Ae%0Df%25g");
    assert_eq!(encode_uri(""), "");
}

/// Multi-byte UTF-8 is escaped byte by byte, so every byte of the character is written out.
#[test]
fn uri_encoding_escapes_utf8_bytes() {
    assert_eq!(encode_uri("é"), "%C3%A9");
}

/// The latest answer lists every field in order and clamps a negative age to zero.
#[test]
fn latest_answer_has_every_field_in_order() {
    let record = update("application://app.desktop", 1000, 0.25);
    assert_eq!(
        format_latest(&record, 1500),
        "uri=application://app.desktop progress=0.2500 visible=true count=3 count-visible=false urgent=true age-ms=500",
    );
    assert!(format_latest(&record, 900).ends_with("age-ms=0"));
}

/// A URI with delimiters cannot add fields to the latest answer.
#[test]
fn latest_answer_keeps_hostile_uri_on_one_field() {
    let record = update("x y;progress=9", 0, 0.5);
    let answer = format_latest(&record, 0);
    assert!(answer.starts_with("uri=x%20y%3Bprogress%3D9 progress=0.5000 "), "{answer}");
    assert_eq!(answer.matches(" progress=").count(), 1);
    assert!(!answer.contains('\n'));
}

/// Records render the time, progress with four decimals, and visibility.
#[test]
fn record_has_time_progress_and_visibility() {
    let record = update("application://app.desktop", 42, 1.0);
    assert_eq!(format_record(&record), "t=42 progress=1.0000 visible=true");
}

/// An empty log is exactly `0`, with no trailing space.
#[test]
fn empty_log_is_zero() {
    assert_eq!(format_log(&[]), "0");
}

/// A log lists its count and then the records, oldest first, separated by semicolons.
#[test]
fn log_lists_count_then_records_in_order() {
    let first = update("application://app.desktop", 10, 0.1);
    let second = update("application://app.desktop", 20, 0.2);
    assert_eq!(
        format_log(&[&first, &second]),
        "2 t=10 progress=0.1000 visible=true; t=20 progress=0.2000 visible=true",
    );
}
