//! What:
//!      Tests for quoted-string decoding,
//!  the fallible conversion to Rust text,
//!  and re-encoding.
//! Why:
//!       Escaped unpaired surrogates must survive as code units,
//!  and any replacement value must be
//!           written back as a legal JSON string literal that decodes to the same units.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('textUnits', () => { /* decode, convert, encode round trip */ });
//! ```

/// What:
///      Import the conversions under test.
/// Why:
///       These tests use the same public functions a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { decodeQuoted, encodeQuoted, unitsToString } from './textUnits';
/// ```
use crate::text_units::{decode_quoted, encode_quoted, units_to_string};

/// Check that ordinary text and JSON escapes decode to the expected UTF-16 code units.
#[test]
fn decodes_plain_text_and_escapes() {
    // What:     Each pair is one quoted token and the code units it must produce.
    // Why:      Decoding is the parser's only view of string content, so escapes need exact coverage.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cases: [string, number[]][] = [['"ab"', [97, 98]]];
    // ```
    let cases: [(&str, &[u16]); 6] = [
        (r#""ab""#, &[97, 98]),
        (r#""a\nb""#, &[97, 10, 98]),
        (r#""\u0041""#, &[65]),
        (r#""\"\\\u002F""#, &[34, 92, 47]),
        (r#""\uD83D\uDE00""#, &[0xD83D, 0xDE00]),
        (r#""\uD800""#, &[0xD800]),
    ];
    for (raw, expected) in cases {
        let decoded = decode_quoted(raw).expect("valid quoted token decodes");
        assert_eq!(decoded, expected, "unexpected units for {raw}");
    }
}

/// Check that malformed quoted tokens are rejected with a source-positioned failure.
#[test]
fn rejects_malformed_quoted_tokens() {
    // What:     Each entry is one token shape the grammar does not admit.
    // Why:      A rejected token must never reach the document model as partial text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const invalid = ['"abc', '"abc"x', '"\\uZZZZ"'];
    // ```
    let invalid = [r#""abc"#, r#""abc"x"#, r#""\uZZZZ""#, r#""\u00""#, "abc\"", r#""\q""#];
    for raw in invalid {
        assert!(decode_quoted(raw).is_err(), "accepted malformed token {raw}");
    }
}

/// Check the fallible conversion to Rust text,
///  including the lone-surrogate rejection.
#[test]
fn converts_units_to_rust_text_only_when_valid() {
    // Ordinary text converts directly.
    assert_eq!(units_to_string(&[97, 98]).expect("valid units convert"), "ab");
    // A surrogate pair converts to the scalar it encodes.
    assert_eq!(
        units_to_string(&[0xD83D, 0xDE00]).expect("paired surrogates convert"),
        "😀"
    );
    // A lone surrogate has no UTF-8 encoding, so the conversion must fail rather than substitute.
    assert!(units_to_string(&[0xD800]).is_err(), "lone surrogate converted");
    assert!(units_to_string(&[0xDC00]).is_err(), "lone low surrogate converted");
}

/// Check that encoding produces a legal JSON literal that decodes back to the same units.
#[test]
fn encodes_and_decodes_the_same_units() {
    // What:     Each entry is one unit sequence to round-trip through encoding and decoding.
    // Why:      An edited value must survive emission and a later reparse unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cases: number[][] = [[97, 98], [0xD800], [0xD83D, 0xDE00]];
    // ```
    let cases: [&[u16]; 6] = [
        &[97, 98],
        &[34, 92, 47],
        &[10, 9, 13, 8, 12],
        &[0, 31],
        &[0xD800],
        &[0xD83D, 0xDE00],
    ];
    for units in cases {
        let encoded = encode_quoted(units);
        assert!(encoded.starts_with('"') && encoded.ends_with('"'), "unquoted literal {encoded}");
        let decoded = decode_quoted(&encoded).expect("encoded literal must decode");
        assert_eq!(decoded, units, "round trip changed units for {encoded}");
    }
}

/// Check the exact spelling of each escape class the encoder emits.
#[test]
fn encodes_expected_escape_spellings() {
    // Short escapes match the JSON specification names.
    assert_eq!(encode_quoted(&[8]), r#""\b""#);
    assert_eq!(encode_quoted(&[9]), r#""\t""#);
    assert_eq!(encode_quoted(&[10]), r#""\n""#);
    assert_eq!(encode_quoted(&[12]), r#""\f""#);
    assert_eq!(encode_quoted(&[13]), r#""\r""#);
    // Quote and backslash use their two-character forms.
    assert_eq!(encode_quoted(&[34]), r#""\"""#);
    assert_eq!(encode_quoted(&[92]), r#""\\""#);
    // Other control characters and lone surrogates use lowercase four-digit escapes.
    assert_eq!(encode_quoted(&[0]), r#""\u0000""#);
    assert_eq!(encode_quoted(&[31]), r#""\u001f""#);
    assert_eq!(encode_quoted(&[0xD800]), r#""\ud800""#);
    assert_eq!(encode_quoted(&[0xDC00]), r#""\udc00""#);
    // A valid pair is emitted as the character itself, the way well-formed JSON.stringify does.
    assert_eq!(encode_quoted(&[0xD83D, 0xDE00]), "\"😀\"");
    // Printable text is emitted without escapes.
    assert_eq!(encode_quoted(&[97, 233, 0x4E2D]), "\"aé中\"");
}
