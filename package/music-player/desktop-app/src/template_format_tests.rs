// Unit tests for `template_format.rs`, pulled in by `#[cfg(test)] #[path = "template_format_tests.rs"] mod tests;`.
//
// Expected strings and numbers come from running the JavaScript reference with node
// (`String(x)` and `Number(text)`), not from memory of JavaScript's rules.

use super::*;

// What: `fn number_text_matches_javascript_string(...)` checks each sample against node's `String()` output.
// Why: Whole numbers, fractions, exponent forms, and the smallest subnormal all have distinct layouts.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(String(275)).toBe("275");
// ```
#[test]
fn number_text_matches_javascript_string() {
    assert_eq!(number_text(275.0), "275");
    assert_eq!(number_text(-1.5), "-1.5");
    assert_eq!(number_text(0.1 + 0.2), "0.30000000000000004");
    assert_eq!(number_text(1e21), "1e+21");
    assert_eq!(number_text(1e-7), "1e-7");
    assert_eq!(number_text(1.5e-7), "1.5e-7");
    assert_eq!(number_text(123_456_789_012_345_680_000.0), "123456789012345680000");
    assert_eq!(number_text(5e-324), "5e-324");
    assert_eq!(number_text(1e20), "100000000000000000000");
    assert_eq!(number_text(0.000_001), "0.000001");
    assert_eq!(number_text(59.9), "59.9");
    assert_eq!(number_text(-0.0), "0");
    assert_eq!(number_text(9_007_199_254_740_992.0), "9007199254740992");
    assert_eq!(number_text(f64::MAX), "1.7976931348623157e+308");
    assert_eq!(number_text(100.0), "100");
    assert_eq!(number_text(1e301), "1e+301");
}

// What: `fn unknown_format_letter_finds_letters_outside_apostrophes(...)` checks each rule of the format letter scan.
// Why: TE5: only h, m and s are time letters, apostrophes keep other words, and every other character is a separator.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(unknownFormatLetter("m min")).toBe("i");
// ```
#[test]
fn unknown_format_letter_finds_letters_outside_apostrophes() {
    assert_eq!(unknown_format_letter("m:ss"), None);
    assert_eq!(unknown_format_letter("hh:mm:ss"), None);
    assert_eq!(unknown_format_letter("m min"), Some('i'));
    assert_eq!(unknown_format_letter("MM"), Some('M'));
    assert_eq!(unknown_format_letter("q:q"), Some('q'));
    assert_eq!(unknown_format_letter("m' min'"), None);
    assert_eq!(unknown_format_letter("'h'm:ss"), None);
    assert_eq!(unknown_format_letter("m \u{e9}"), Some('\u{e9}'));
    assert_eq!(unknown_format_letter("m / s, 1.5"), None);
    assert_eq!(unknown_format_letter(""), None);
    assert_eq!(unknown_format_letter("m'unclosed"), None, "an unclosed apostrophe keeps the rest as text, as format_seconds does");
}

// What: `fn number_text_prints_special_values(...)` checks NaN, both infinities, and zero.
// Why: A duration that is not finite still has to print, so the text must match JavaScript's words for them.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(String(NaN)).toBe("NaN");
// ```
#[test]
fn number_text_prints_special_values() {
    assert_eq!(number_text(f64::NAN), "NaN");
    assert_eq!(number_text(f64::INFINITY), "Infinity");
    assert_eq!(number_text(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(number_text(0.0), "0");
}

// What: `fn as_number_reads_javascript_spellings(...)` checks the number spellings `Number()` accepts.
// Why: `tf` and `tc` read their arguments as text, so every accepted spelling must yield the same value as node.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(Number(" 0x1F ")).toBe(31);
// ```
#[test]
fn as_number_reads_javascript_spellings() {
    assert_eq!(as_number(" 0x1F "), Some(31.0));
    assert_eq!(as_number("5."), Some(5.0));
    assert_eq!(as_number(".5"), Some(0.5));
    assert_eq!(as_number("0b101"), Some(5.0));
    assert_eq!(as_number("0o17"), Some(15.0));
    assert_eq!(as_number("0O17"), Some(15.0));
    assert_eq!(as_number("+.5e-3"), Some(0.0005));
    assert_eq!(as_number("  12  "), Some(12.0));
    assert_eq!(as_number("-0"), Some(0.0));
    assert_eq!(as_number("275"), Some(275.0));
    assert_eq!(as_number("-1.5"), Some(-1.5));
}

// What: `fn as_number_refuses_what_javascript_refuses(...)` checks the spellings that give NaN or an infinity.
// Why: Only finite numbers are durations, and a signed hexadecimal or a stray character is not a number at all.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(Number("-0x1")).toBeNaN();
// ```
#[test]
fn as_number_refuses_what_javascript_refuses() {
    assert_eq!(as_number("-0x1"), None);
    assert_eq!(as_number("+0x1"), None);
    assert_eq!(as_number("1e400"), None);
    assert_eq!(as_number("-1e400"), None);
    assert_eq!(as_number("1_0"), None);
    assert_eq!(as_number(""), None);
    assert_eq!(as_number(" "), None);
    assert_eq!(as_number("Infinity"), None);
    assert_eq!(as_number("-Infinity"), None);
    assert_eq!(as_number("1e"), None);
    assert_eq!(as_number("NaN"), None);
    assert_eq!(as_number("0x"), None);
    assert_eq!(as_number("1 2"), None);
    assert_eq!(as_number("5d"), None);
    assert_eq!(as_number("+"), None);
    assert_eq!(as_number("."), None);
    assert_eq!(as_number("0.0e+0x"), None);
}

// What: `fn as_number_trims_javascript_white_space_only(...)` checks the characters `trim` removes in JavaScript.
// Why: Rust's `trim` keeps the byte order mark and removes U+0085, so the JavaScript set must be applied by hand.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(Number("﻿5")).toBe(5);
// ```
#[test]
fn as_number_trims_javascript_white_space_only() {
    assert_eq!(as_number("\u{FEFF}5"), Some(5.0));
    assert_eq!(as_number("\u{2028}5"), Some(5.0));
    assert_eq!(as_number("\u{A0}5"), Some(5.0));
    assert_eq!(as_number("\u{1680} 5"), Some(5.0));
    assert_eq!(as_number("\u{3000}50"), Some(50.0));
    assert_eq!(as_number("\u{85}5"), None);
    assert_eq!(as_number("\u{200B}5"), None);
}

// What: `fn as_number_reads_exactly_within_128_bits(...)` checks that large radix literals convert with exact rounding.
// Why: The exact path keeps `0xFFFFFFFFFFFFFFFF` at JavaScript's value, which accumulation in `f64` would miss.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(Number("0xFFFFFFFFFFFFFFFF")).toBe(18446744073709552000);
// ```
#[test]
fn as_number_reads_exactly_within_128_bits() {
    assert_eq!(as_number("0xFFFFFFFFFFFFFFFF"), Some(18_446_744_073_709_551_615.0));
    assert_eq!(number_text(as_number("0xFFFFFFFFFFFFFFFF").unwrap_or(0.0)), "18446744073709552000");
}

// What: `fn format_seconds_follows_the_format_letters(...)` checks the duration formats from the reference.
// Why: Letters, doubling, and apostrophes each change the output, so every rule gets a case.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(formatSeconds({ seconds: 275, format: "mm:ss" })).toBe("04:35");
// ```
#[test]
fn format_seconds_follows_the_format_letters() {
    assert_eq!(format_seconds(275.0, "m:ss"), "4:35");
    assert_eq!(format_seconds(275.0, "mm:ss"), "04:35");
    assert_eq!(format_seconds(275.0, "h:mm:ss"), "0:04:35");
    assert_eq!(format_seconds(4000.0, "m:ss"), "66:40");
    assert_eq!(format_seconds(4000.0, "h:mm:ss"), "1:06:40");
    assert_eq!(format_seconds(4000.0, "hh:mm:ss"), "01:06:40");
    assert_eq!(format_seconds(59.9, "m:ss"), "0:59");
    assert_eq!(format_seconds(5.0, "ss"), "05");
    assert_eq!(format_seconds(0.0, "hh"), "00");
}

// What: `fn format_seconds_keeps_apostrophe_text_and_bare_letters(...)` checks the text kept between apostrophes.
// Why: A bare letter is always a time part, and only text between apostrophes is kept as written.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(formatSeconds({ seconds: 275, format: "m min" })).toBe("4 4in");
// ```
#[test]
fn format_seconds_keeps_apostrophe_text_and_bare_letters() {
    assert_eq!(format_seconds(275.0, "m min"), "4 4in");
    assert_eq!(format_seconds(275.0, "m' min'"), "4 min");
    assert_eq!(format_seconds(4000.0, "h' hours' and m' minutes'"), "1 hours and 6 minutes");
    assert_eq!(format_seconds(4000.0, "'h'm:ss"), "h6:40");
}

// What: `fn format_seconds_clamps_and_prints_large_values(...)` checks negative seconds and values that print in exponent form.
// Why: A negative duration reads as zero, and a huge duration prints with JavaScript's exponent layout.
//
// In TS you'd write (pseudocode):
// ```ts
// expect(formatSeconds({ seconds: -5, format: "m:ss" })).toBe("0:00");
// ```
#[test]
fn format_seconds_clamps_and_prints_large_values() {
    assert_eq!(format_seconds(-5.0, "m:ss"), "0:00");
    assert_eq!(format_seconds(1e300, "h"), "2.777777777777778e+296");
    assert_eq!(format_seconds(1e21, "m:ss"), "16666666666666666000:40");
    assert_eq!(format_seconds(0.0, "m:ss"), "0:00");
}

// What: `fn keep_code_points_counts_code_points(...)` checks the cut by code points and its edges.
// Why: The reference keeps whole code points, so an astral character counts once and a count past the end keeps all of the text.
//
// In TS you'd write (pseudocode):
// ```ts
// expect([..."𐐨abc"].slice(0, 1).join("")).toBe("𐐨");
// ```
#[test]
fn keep_code_points_counts_code_points() {
    assert_eq!(keep_code_points("Another Xronixle", 7.0), "Another");
    assert_eq!(keep_code_points("かめりあ(Camellia)", 4.0), "かめりあ");
    assert_eq!(keep_code_points("Another", 0.0), "");
    assert_eq!(keep_code_points("Another", 99.0), "Another");
    assert_eq!(keep_code_points("Another", 2.9), "An");
    assert_eq!(keep_code_points("\u{10428}abc", 1.0), "\u{10428}");
    assert_eq!(keep_code_points("", 0.0), "");
}

// What: `fn capitalize_first_unit_follows_the_slice_rule(...)` checks `cap` on ordinary, empty, multi-letter, and astral text.
// Why: `cap` uppercases one UTF-16 unit, so an astral first character is left unchanged, as in the reference.
//
// In TS you'd write (pseudocode):
// ```ts
// expect("hello there".slice(0, 1).toUpperCase() + "hello there".slice(1)).toBe("Hello there");
// ```
#[test]
fn capitalize_first_unit_follows_the_slice_rule() {
    assert_eq!(capitalize_first_unit("hello there"), "Hello there");
    assert_eq!(capitalize_first_unit(""), "");
    assert_eq!(capitalize_first_unit("été"), "Été");
    assert_eq!(capitalize_first_unit("ßa"), "SSa");
    assert_eq!(capitalize_first_unit("\u{10428}abc"), "\u{10428}abc");
}

// What: `fn utf16_length_counts_code_units(...)` checks that astral characters count two units.
// Why: Template offsets are UTF-16 offsets, so the length of text must count surrogate pairs as two.
//
// In TS you'd write (pseudocode):
// ```ts
// expect("\u{10428}".length).toBe(2);
// ```
#[test]
fn utf16_length_counts_code_units() {
    assert_eq!(utf16_length("abc"), 3);
    assert_eq!(utf16_length("\u{10428}"), 2);
    assert_eq!(utf16_length(""), 0);
}
