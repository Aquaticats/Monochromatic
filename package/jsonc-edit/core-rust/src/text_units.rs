//! What:
//!      Conversion between quoted JSON string tokens and UTF-16 code units.
//!           A JSON string may escape an unpaired UTF-16 surrogate such as `\uD800`,
//!  which Rust
//!           `String` cannot hold,
//!  so this crate stores decoded text as `Vec<u16>` (an owned sequence
//!           of 16-bit code units) and converts to `String` only when the caller asks and the units
//!           allow it.
//! Why:
//!       The port must retain escaped lone surrogates instead of rejecting or replacing them,
//!  and
//!           must be able to write a replacement value back as a legal JSON string literal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module textUnits: decodeQuoted(raw), unitsToString(units), encodeQuoted(units).
//! ```

/// What:
///      Import the source-positioned parse failure type.
/// Why:
///       A malformed quoted token must report the byte offset where scanning stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncParseError } from './error';
/// ```
use crate::error::JsoncParseError;
/// What:
///      Import the byte scanner that already implements JSON string escapes.
/// Why:
///       Decoding a standalone quoted token must agree exactly with decoding inside a document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Scanner } from './scan';
/// ```
use crate::scan::Scanner;

/// What:
///      Decode one complete quoted JSON string token into UTF-16 code units.
/// Why:
///       Callers that hold a raw token (from an edit or a fixture) need the same decoding the
///           parser performs,
///  including escaped lone surrogates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeQuoted(raw: string): number[] { return JSON.parse(raw); }
/// ```
///
/// # Errors
/// Returns the scanner's failure when the token is unterminated,
///  holds an invalid escape,
///  or is
/// followed by extra bytes.
pub fn decode_quoted(raw: &str) -> Result<Vec<u16>, JsoncParseError> {
    // What:     `Scanner::new` borrows the token text without copying it.
    // Why:      Decoding reads the token once and returns owned code units.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const scanner = { source: raw, offset: 0 };
    // ```
    let mut scanner = Scanner::new(raw);
    let (units, _) = scanner.string()?;
    if scanner.offset != raw.len() {
        return Err(scanner.error("unexpected content after quoted JSON string token"));
    }
    return Ok(units);
}

/// What:
///      Convert stored code units into a Rust `String` when they form valid UTF-16 text.
/// Why:
///       Most documents contain no lone surrogate,
///  and callers want ordinary Rust text;
///  the
///           conversion must fail rather than substitute a replacement character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unitsToString(units: number[]): string { return String.fromCharCode(...units); }
/// ```
///
/// # Errors
/// Returns the standard library's `FromUtf16Error` when a code unit sequence holds an unpaired
/// surrogate.
pub fn units_to_string(units: &[u16]) -> Result<String, std::string::FromUtf16Error> {
    // What:     `String::from_utf16` validates every code unit pair.
    // Why:      Validation is the whole point: an unpaired surrogate has no UTF-8 encoding.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return String.fromCharCode(...units);
    // ```
    return String::from_utf16(units);
}

/// What:
///      Write code units as a quoted JSON string literal.
/// Why:
///       An edit that replaces a string value must emit legal JSON text,
///  escaping quotes,
///           backslashes,
///  control characters and any unpaired surrogate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encodeQuoted(units: number[]): string { return JSON.stringify(String.fromCharCode(...units)); }
/// ```
pub fn encode_quoted(units: &[u16]) -> String {
    // What:     `String` accumulates the emitted literal.
    // Why:      The result is owned text the caller can splice into a document.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let out = '"';
    // ```
    let mut output = String::from("\"");
    let mut cursor: usize = 0;
    while cursor < units.len() {
        let unit = units[cursor];
        cursor += 1;
        // What:     A high surrogate followed by a low surrogate is one scalar value.
        // Why:      Emitting the real character keeps output readable and matches well-formed
        //           `JSON.stringify` behavior.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (unit >= 0xd800 && unit <= 0xdbff && next >= 0xdc00 && next <= 0xdfff) { emitPair(); }
        // ```
        if (0xD800..=0xDBFF).contains(&unit) {
            let next = units.get(cursor).copied().unwrap_or(0);
            if (0xDC00..=0xDFFF).contains(&next) {
                cursor += 1;
                let scalar = 0x1_0000 + ((u32::from(unit) - 0xD800) << 10) + (u32::from(next) - 0xDC00);
                if let Some(character) = char::from_u32(scalar) {
                    output.push(character);
                    continue;
                }
            }
        }
        if unit == u16::from(b'"') {
            output.push_str("\\\"");
            continue;
        }
        if unit == u16::from(b'\\') {
            output.push_str("\\\\");
            continue;
        }
        // What:     Short escapes cover the five control characters JSON names directly.
        // Why:      Matching `JSON.stringify` keeps output familiar and round-trippable.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const short = { 8: '\\b', 9: '\\t', 10: '\\n', 12: '\\f', 13: '\\r' }[unit];
        // ```
        let short_escape = match unit {
            0x08 => Some("\\b"),
            0x09 => Some("\\t"),
            0x0A => Some("\\n"),
            0x0C => Some("\\f"),
            0x0D => Some("\\r"),
            _ => None,
        };
        if let Some(escape) = short_escape {
            output.push_str(escape);
            continue;
        }
        if unit < 0x20 || (0xD800..=0xDFFF).contains(&unit) {
            // What:     `format!` writes a lowercase four-digit hexadecimal escape.
            // Why:      Control characters and lone surrogates have no literal JSON representation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // out += '\\u' + unit.toString(16).padStart(4, '0');
            // ```
            output.push_str(format!("\\u{unit:04x}").as_str());
            continue;
        }
        // What:     `char::from_u32` converts a non-surrogate code unit to a Rust character.
        // Why:      Every remaining unit below 0x10000 outside the surrogate range is a valid scalar.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // out += String.fromCharCode(unit);
        // ```
        match char::from_u32(u32::from(unit)) {
            Some(character) => output.push(character),
            None => output.push_str(format!("\\u{unit:04x}").as_str()),
        }
    }
    output.push('"');
    return output;
}
