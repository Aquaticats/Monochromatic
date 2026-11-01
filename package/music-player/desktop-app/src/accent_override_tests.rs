// What:     Unit tests for `accent_override.rs`, pulled in by `#[cfg(test)] #[path = "accent_override_tests.rs"] mod tests;`.
// Why:      Each accepted spelling maps to its color, and a typo is an error naming the text, not a silent system accent.

use super::*;

#[test]
fn unset_and_empty_follow_the_system() {
    assert_eq!(accent_override(None), Ok(None));
    assert_eq!(accent_override(Some("")), Ok(None));
}

#[test]
fn six_digits_with_or_without_a_hash_give_the_color() {
    let expected: Option<Srgb8> = Some(Srgb8 { red: 0x60, green: 0x60, blue: 0xC0 });
    assert_eq!(accent_override(Some("6060C0")), Ok(expected));
    assert_eq!(accent_override(Some("#6060c0")), Ok(expected));
}

#[test]
fn other_text_is_an_error_naming_the_value() {
    for text in ["red", "#12345", "1234567", "12345G", "##123456", " 123456"] {
        let error = accent_override(Some(text)).expect_err("malformed values are not accepted");
        assert_eq!(error, AccentError::Malformed(text.to_string()));
        assert!(error.to_string().contains(&format!("\"{text}\"")));
    }
}

#[test]
fn multibyte_text_of_six_bytes_is_rejected_without_a_panic() {
    // Positive control for the slicing: six bytes that are not six hex digits must not panic in the byte slice.
    assert!(accent_override(Some("ééé")).is_err());
}
