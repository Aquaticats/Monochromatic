//! The encoding keeps every separator of the answer grammar out of client text.

/// Brings the encoder and the record joiner into scope.
use super::*;

/// Plain text is untouched and every separator is escaped.
#[test]
fn separators_never_survive_encoding() {
    assert_eq!(encode_text("plain-text_1.0~"), "plain-text_1.0~");
    assert_eq!(encode_text(""), "");
    assert_eq!(encode_text("Done; 100 %"), "Done%3B%20100%20%25");
    for hostile in ["a b", "a;b", "a,b", "a:b", "a=b", "a%b", "a\nb", "a\rb", "a\0b", "a\tb", "ok\nerr x"] {
        let encoded = encode_text(hostile);
        assert!(
            !encoded.contains([' ', ';', ',', ':', '=', '\n', '\r', '\0', '\t']),
            "{hostile:?} -> {encoded:?}",
        );
    }
}

/// Multi-byte characters are escaped byte by byte.
#[test]
fn utf8_is_escaped_per_byte() {
    assert_eq!(encode_text("é"), "%C3%A9");
    assert_eq!(encode_text("🎵"), "%F0%9F%8E%B5");
}

/// The record list starts with its count and uses `; ` between records.
#[test]
fn record_lists_match_the_launcher_log_shape() {
    assert_eq!(format_records(&[]), "0");
    assert_eq!(format_records(&["a=1".to_string()]), "1 a=1");
    assert_eq!(format_records(&["a=1".to_string(), "b=2".to_string()]), "2 a=1; b=2");
}
