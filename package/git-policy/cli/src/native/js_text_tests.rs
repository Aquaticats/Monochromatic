//! Controls for the JavaScript whitespace table.

use super::*;

/// Every ECMAScript whitespace and line terminator is trimmed at both ends.
#[test]
fn every_javascript_whitespace_is_trimmed() {
    let whitespace: [char; 25] = [
        '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0020}', '\u{00a0}',
        '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
        '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}',
        '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}',
    ];
    for character in whitespace {
        assert!(is_javascript_whitespace(character), "{character:?}");
        let text: String = format!("{character}x y{character}");
        assert_eq!(trim_javascript(text.as_str()), "x y");
    }
}

/// Characters Rust or older Unicode count as whitespace,
///  and ordinary text,
///  are kept.
#[test]
fn other_characters_are_kept() {
    for character in ['\u{0085}', '\u{180e}', '\u{200b}', 'x', '\u{0}', '\u{1f}'] {
        assert!(!is_javascript_whitespace(character), "{character:?}");
    }
    assert_eq!(trim_javascript("\u{85}x\u{85}"), "\u{85}x\u{85}");
    assert_eq!(trim_javascript(""), "");
    assert_eq!(trim_javascript(" \t\n"), "");
}

/// Trailing whitespace goes and leading whitespace stays,
///  as `trimEnd` does.
#[test]
fn trailing_whitespace_only_is_trimmed() {
    assert_eq!(trim_javascript_end("  x y \t\n"), "  x y");
    assert_eq!(trim_javascript_end("\u{feff}pid 1\u{2029}"), "\u{feff}pid 1");
    assert_eq!(trim_javascript_end("\u{85}"), "\u{85}");
    assert_eq!(trim_javascript_end(""), "");
}
