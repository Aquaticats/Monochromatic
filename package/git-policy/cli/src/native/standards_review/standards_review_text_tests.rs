//! Controls for the JavaScript text behaviors the standards-review log depends on.
//! Expectations come from the input or from the standard library's own UTF-16 encoder, never from copied counts.

use super::*;

/// Text with one character of each UTF-8 width, the last one outside the basic plane.
const MIXED: &str = "a\u{e9}\u{4e2d}\u{1f600}z";

/// The length is the UTF-16 encoder's count, not the byte count.
#[test]
fn length_counts_utf16_units() {
    assert_eq!(utf16_length(MIXED), MIXED.encode_utf16().count());
    assert_ne!(utf16_length(MIXED), MIXED.len());
    assert_eq!(utf16_length(""), 0);
}

/// A prefix at every limit is the longest whole-character prefix that fits, and a limit past the end keeps all.
#[test]
fn prefix_never_exceeds_the_limit_and_never_splits_a_pair() {
    let total: usize = utf16_length(MIXED);
    for limit in 0..=total + 1 {
        let kept: String = utf16_prefix(MIXED, limit);
        assert!(MIXED.starts_with(kept.as_str()));
        assert!(utf16_length(kept.as_str()) <= limit);
        let next: Option<char> = MIXED[kept.len()..].chars().next();
        if let Some(character) = next {
            assert!(utf16_length(kept.as_str()) + character.len_utf16() > limit);
        }
    }
    assert_eq!(utf16_prefix(MIXED, total), MIXED);
}

/// Every printed number parses back to the same value, and the layout switches follow ECMAScript's thresholds.
#[test]
fn numbers_round_trip_and_follow_the_exponent_thresholds() {
    for value in [0.43_f64, 0.02, 1.0, 0.5, 123.456, 1e-6, 1e-7, 1.5e-7, 1e20, 1e21, 1.5e21, 123_456_789.0, -0.25] {
        let text: String = js_number_text(value);
        assert_eq!(text.parse::<f64>().ok(), Some(value), "{text}");
    }
    assert_eq!(js_number_text(0.0), "0");
    // 1e21 is the first magnitude ECMAScript prints with an exponent; 1e-7 is the first small one.
    assert!(js_number_text(1e21).contains("e+"));
    assert!(!js_number_text(1e20).contains('e'));
    assert!(js_number_text(1e-7).contains("e-"));
    assert!(!js_number_text(1e-6).contains('e'));
    assert!(js_number_text(0.43).starts_with("0.43"));
    assert_eq!(js_number_text(1.0), "1");
}

/// The tenth-of-a-second text agrees with the nearest tenth everywhere except ties, and a tie goes up.
#[test]
fn seconds_round_to_the_nearest_tenth_with_ties_up() {
    for milliseconds in 0..=3000_u64 {
        let text: String = seconds_with_tenth(milliseconds);
        let printed: f64 = text.parse::<f64>().unwrap_or(f64::NAN);
        let exact: f64 = milliseconds as f64 / 1000.0;
        assert!((printed - exact).abs() <= 0.05 + 1e-9, "{milliseconds} -> {text}");
        if milliseconds % 500 == 250 {
            assert!(printed > exact, "{milliseconds} -> {text}");
        }
        assert_eq!(text.split_once('.').map(|(_, tenth)| return tenth.len()), Some(1));
    }
}

/// Nothing that could end a line survives, text without such characters is unchanged, and the escapes are visible.
#[test]
fn one_line_text_has_no_line_breaks() {
    let hostile: String = format!("a\nb\r\nc\td\u{0}e\u{2028}f\u{2029}g\u{85}h\u{7f}i");
    let escaped: String = on_one_line(hostile.as_str());
    assert!(!escaped.chars().any(|character| return character.is_control() || character == '\u{2028}' || character == '\u{2029}'));
    assert!(escaped.contains("\\n") && escaped.contains("\\r") && escaped.contains("\\t"));
    assert!(escaped.contains("\\u{0}") && escaped.contains("\\u{2028}"));
    assert_eq!(on_one_line(MIXED), MIXED);
    assert_eq!(on_one_line("plain: `x` \\ done"), "plain: `x` \\ done");
}

/// Blank lines before the subject are skipped, a subject wrapped over lines joins with spaces, and the body is dropped.
#[test]
fn subject_is_the_first_paragraph_folded() {
    assert_eq!(commit_subject("fix(a): one\n\nbody line\n"), "fix(a): one");
    assert_eq!(commit_subject("\n\nfix(a): one\ntwo   \n\nbody"), "fix(a): one two");
    assert_eq!(commit_subject("only"), "only");
    assert_eq!(commit_subject(""), "");
    assert_eq!(commit_subject("  \n \n"), "");
}
