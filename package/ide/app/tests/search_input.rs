//! Search overlay parsing and double-Shift timing are deterministic without a native event loop.

/// The same input rules are used by native capture and query-edit callbacks.
use ide_app::search_input::{DoubleShift, SearchInput};
/// Inject elapsed times so the gesture boundary is measured exactly rather than through sleeps.
use std::time::Duration;

/// Percent applies only at the raw input's first character,
///  and empty patterns do not start work.
#[test]
fn query_prefix_and_trimming_follow_the_reference_order() {
    assert_eq!(
        SearchInput::parse("  needle  "),
        Some(SearchInput {
            query: "needle".to_string(),
            content_only: false
        })
    );
    assert_eq!(
        SearchInput::parse("%  needle\n"),
        Some(SearchInput {
            query: "needle".to_string(),
            content_only: true
        })
    );
    assert_eq!(
        SearchInput::parse(" %needle"),
        Some(SearchInput {
            query: "%needle".to_string(),
            content_only: false
        })
    );
    assert_eq!(
        SearchInput::parse("%%literal"),
        Some(SearchInput {
            query: "%literal".to_string(),
            content_only: true
        })
    );
    for empty in ["", " \t\n", "%", "%  "] {
        assert!(SearchInput::parse(empty).is_none());
    }
}

/// JavaScript trims the byte-order mark but retains NEXT LINE,
///  unlike Rust's default whitespace predicate.
#[test]
fn query_trimming_retains_javascript_whitespace_semantics() {
    assert_eq!(
        SearchInput::parse("\u{feff}needle\u{feff}")
            .expect("BOM-padded input")
            .query,
        "needle"
    );
    assert_eq!(
        SearchInput::parse("\u{0085}")
            .expect("NEXT LINE remains a pattern")
            .query,
        "\u{0085}"
    );
}

/// The second release must precede the 400ms boundary;
///  successful gestures reset the pair.
#[test]
fn double_shift_uses_a_strict_timing_boundary_and_resets() {
    let mut gesture = DoubleShift::default();
    gesture.press(true);
    assert!(!gesture.release(true, Duration::ZERO));
    gesture.press(true);
    assert!(gesture.release(true, Duration::from_millis(399)));
    assert!(!gesture.release(true, Duration::from_millis(400)));
    assert!(!gesture.release(true, Duration::from_millis(800)));
    assert!(gesture.release(true, Duration::from_millis(900)));
}

/// Other keypresses break the gesture,
///  while unrelated releases do not manufacture a Shift release.
#[test]
fn intervening_keys_and_reversed_time_do_not_open_search() {
    let mut gesture = DoubleShift::default();
    assert!(!gesture.release(false, Duration::from_millis(1)));
    assert!(!gesture.release(true, Duration::from_millis(10)));
    gesture.press(false);
    assert!(!gesture.release(false, Duration::from_millis(20)));
    assert!(!gesture.release(true, Duration::from_millis(30)));
    assert!(gesture.release(true, Duration::from_millis(40)));
    assert!(!gesture.release(true, Duration::from_millis(100)));
    assert!(!gesture.release(true, Duration::from_millis(90)));
}
