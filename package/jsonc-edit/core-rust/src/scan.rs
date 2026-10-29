//! What:
//!  Linear JSONC token and comment scanning over a borrowed source string.
//! Why:
//!  The published parser models required by this port either lose surrogate data or use regex.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Scanner = { source: string; offset: number };
//! ```

/// What:
///  Import comment and parse-error values from the owned value model.
/// Why:
///  Scanner errors and trivia must use the same interface as parser results.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncComment, JsoncCommentKind, JsoncParseError } from './value';
/// ```
use crate::error::JsoncParseError;
/// What:
///      Import the exact-number identity that a scanned token must produce.
/// Why:
///       A number token is only usable when its mathematical identity is canonical and exact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncNumberIdentity } from './number';
/// ```
use crate::number::JsoncNumberIdentity;
/// What:
///      Import the comment types the scanner returns for trivia.
/// Why:
///       Comments are data in this crate,
///  so scanning produces the same types the document stores.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncComment, JsoncCommentKind } from './value';
/// ```
use crate::value::{JsoncComment, JsoncCommentKind};

/// What:
///  Borrow a source string and track its next UTF-8 byte offset.
/// Why:
///  The scanner never copies whole input merely to navigate it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scanner = { source: string; offset: number };
/// ```
pub struct Scanner<'a> {
    /// What:
    ///  Borrowed UTF-8 source,
    ///  unlike an owned `String` that would copy it.
    /// Why:
    ///  Parsed nodes retain only their required raw scalar slices.
    pub source: &'a str,
    /// What:
    ///  The next source byte,
    ///  not a UTF-16 index.
    /// Why:
    ///  Errors and raw slices point directly at Rust UTF-8 input.
    pub offset: usize,
}

/// What:
///  Same-line comments and whether the one separating comma was consumed.
/// Why:
///  A value's trailing comment attaches to that value,
///  not the next entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Trailing = { comments: JsoncComment[]; comma: boolean };
/// ```
pub struct Trailing {
    /// Comments seen on the current line.
    pub comments: Vec<JsoncComment>,
    /// Whether one comma was consumed.
    pub comma: bool,
}

/// What:
///  Convert an ASCII hexadecimal digit to its nibble value.
/// Why:
///  `\u` escapes represent UTF-16 code units,
///  including lone surrogates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hexDigit(byte: number): number | undefined;
/// ```
fn hex_digit(byte: u8) -> Option<u16> {
    if byte.is_ascii_digit() {
        // What: `Some` is the present variant of `Option`.
        // Why: The digit is known valid and contributes a nibble to the code unit.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return byte - 48;
        // ```
        return Some(u16::from(byte - b'0'));
    }
    let lower = byte.to_ascii_lowercase();
    if (b'a'..=b'f').contains(&lower) {
        return Some(u16::from(lower - b'a') + 10);
    }
    // What: `None` means the source byte cannot be a hex digit.
    // Why: The string scanner must reject malformed `\u` text before emission.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return undefined;
    // ```
    return None;
}

/// What:
///      The scanner's token,
///  trivia and error operations.
/// Why:
///       One cursor type owns source navigation,
///  so the parser never indexes the document itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Scanner { byte(): number | undefined; consume(expected: number): boolean; string(): [number[], string] }
/// ```
impl<'a> Scanner<'a> {
    /// What:
    ///  Create a source-borrowing scanner positioned at the start.
    /// Why:
    ///  The parser owns its scanner and never mutates caller-owned input.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function scanner(source: string): Scanner { return { source, offset: 0 }; }
    /// ```
    pub fn new(source: &'a str) -> Self {
        return Self { source, offset: 0 };
    }

    /// What:
    ///  Read the current byte without changing the scanner offset.
    /// Why:
    ///  Parser branches can select syntax before committing input consumption.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function byte(state: Scanner): number | undefined { return source.charCodeAt(offset); }
    /// ```
    pub fn byte(&self) -> Option<u8> {
        // What: `get` yields an `Option<&u8>` borrowed from UTF-8 bytes; `copied` returns an owned byte.
        // Why: Out-of-range reads stay explicit absence rather than an invalid slice.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return sourceBytes[offset];
        // ```
        return self.source.as_bytes().get(self.offset).copied();
    }

    /// What:
    ///  Consume one expected ASCII byte when it is present.
    /// Why:
    ///  Container delimiters and punctuation move the cursor exactly once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function consume(expected: number): boolean;
    /// ```
    pub fn consume(&mut self, expected: u8) -> bool {
        if self.byte() != Some(expected) {
            return false;
        }
        self.offset += 1;
        return true;
    }

    /// What:
    ///  Build a parse error at the current UTF-8 byte offset.
    /// Why:
    ///  Callers receive a source position and neutral operation message.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function error(message: string): JsoncParseError { return { offset, message }; }
    /// ```
    pub fn error(&self, message: &str) -> JsoncParseError {
        return JsoncParseError { offset: self.offset, message: message.to_string() };
    }

    /// What:
    ///  Consume one C-style line or block comment,
    ///  without its delimiters.
    /// Why:
    ///  The parser attaches the body to a key or value instead of dropping it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function scanComment(): JsoncComment;
    /// ```
    fn comment(&mut self) -> Result<JsoncComment, JsoncParseError> {
        let start = self.offset + 2;
        if self.source.as_bytes().get(self.offset + 1) == Some(&b'/') {
            self.offset = start;
            // JSONC line comments stop before either CR or LF, so CRLF is not part of the body.
            while self.byte().is_some() && self.byte() != Some(b'\n') && self.byte() != Some(b'\r') {
                self.offset += 1;
            }
            // What: `Ok` wraps the parsed comment as a successful result.
            // Why: The content remains untrimmed for deterministic round trips.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { kind: 'inline', text: source.slice(start, offset) };
            // ```
            return Ok(JsoncComment { kind: JsoncCommentKind::Line, text: self.source[start..self.offset].to_string() });
        }
        if self.source.as_bytes().get(self.offset + 1) == Some(&b'*') {
            self.offset = start;
            while self.offset + 1 < self.source.len() {
                if self.byte() == Some(b'*') && self.source.as_bytes().get(self.offset + 1) == Some(&b'/') {
                    let text = self.source[start..self.offset].to_string();
                    self.offset += 2;
                    return Ok(JsoncComment { kind: JsoncCommentKind::Block, text });
                }
                self.offset += 1;
            }
            return Err(self.error("unterminated block comment"));
        }
        return Err(self.error("expected JSONC comment"));
    }

    /// What:
    ///  Skip JSON whitespace and gather comments before the next token.
    /// Why:
    ///  Leading comments must be assigned to the following key or value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function trivia(): JsoncComment[];
    /// ```
    pub fn trivia(&mut self) -> Result<Vec<JsoncComment>, JsoncParseError> {
        // What: `Vec<JsoncComment>` is an owned growable list, unlike borrowed `&[JsoncComment]` or fixed arrays.
        // Why: The parser will merge this ordered list into one attached comment.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const comments: JsoncComment[] = [];
        // ```
        let mut comments: Vec<JsoncComment> = Vec::new();
        loop {
            if self.byte() == Some(b' ') || self.byte() == Some(b'\t')
                || self.byte() == Some(b'\r') || self.byte() == Some(b'\n') {
                self.offset += 1;
            } else if self.byte() == Some(b'/')
                && (self.source.as_bytes().get(self.offset + 1) == Some(&b'/')
                    || self.source.as_bytes().get(self.offset + 1) == Some(&b'*')) {
                // What: `?` returns a scanner error immediately or unwraps its successful comment.
                // Why: Unterminated comments cannot silently disappear during parsing.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // comments.push(scanComment());
                // ```
                comments.push(self.comment()?);
            } else {
                break;
            }
        }
        return Ok(comments);
    }

    /// What:
    ///  Gather at most one comma and following comments on a value's line.
    /// Why:
    ///  Inline comments belong to the value just parsed even after its comma.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function captureTrailing(): { comments: JsoncComment[]; comma: boolean };
    /// ```
    pub fn capture_trailing(&mut self) -> Result<Trailing, JsoncParseError> {
        let mut comments = Vec::new();
        let mut comma = false;
        loop {
            if self.byte() == Some(b' ') || self.byte() == Some(b'\t') || self.byte() == Some(b'\r') {
                self.offset += 1;
            } else if self.byte() == Some(b',') && !comma {
                comma = true;
                self.offset += 1;
            } else if self.byte() == Some(b'/')
                && (self.source.as_bytes().get(self.offset + 1) == Some(&b'/')
                    || self.source.as_bytes().get(self.offset + 1) == Some(&b'*')) {
                comments.push(self.comment()?);
            } else {
                break;
            }
        }
        return Ok(Trailing { comments, comma });
    }

    /// What:
    ///  Consume exactly four hexadecimal digits after a JSON `\u` marker.
    /// Why:
    ///  Every code unit remains valid even when it represents a lone surrogate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function codeUnit(): number { /* four hex digits */ }
    /// ```
    fn code_unit(&mut self) -> Result<u16, JsoncParseError> {
        let mut value: u16 = 0;
        for _ in 0..4 {
            // What: `let Some` extracts a present source byte; absence returns a parse error.
            // Why: A short escape must not silently become a different code unit.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const byte = sourceBytes[offset]; if (byte === undefined) throw Error('short escape');
            // ```
            let Some(byte) = self.byte() else { return Err(self.error("incomplete Unicode escape")); };
            let Some(digit) = hex_digit(byte) else { return Err(self.error("invalid Unicode escape digit")); };
            value = value * 16 + digit;
            self.offset += 1;
        }
        return Ok(value);
    }

    /// What:
    ///  Parse a JSON quoted token into UTF-16 units and unchanged raw spelling.
    /// Why:
    ///  Rust `String` alone cannot represent escaped unpaired surrogate values.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function scanString(): { units: number[]; raw: string };
    /// ```
    pub fn string(&mut self) -> Result<(Vec<u16>, String), JsoncParseError> {
        let start = self.offset;
        if !self.consume(b'"') {
            return Err(self.error("expected double-quoted JSON string"));
        }
        let mut units = Vec::new();
        loop {
            let Some(byte) = self.byte() else { return Err(self.error("unterminated JSON string")); };
            if byte == b'"' {
                self.offset += 1;
                return Ok((units, self.source[start..self.offset].to_string()));
            }
            if byte == b'\\' {
                self.offset += 1;
                let Some(escape) = self.byte() else { return Err(self.error("unterminated JSON escape")); };
                self.offset += 1;
                if escape == b'u' {
                    units.push(self.code_unit()?);
                    continue;
                }
                let simple_escape = if escape == b'"' { Some(u16::from(b'"')) }
                    else if escape == b'\\' { Some(u16::from(b'\\')) }
                    else if escape == b'/' { Some(u16::from(b'/')) }
                    else if escape == b'b' { Some(8) }
                    else if escape == b'f' { Some(12) }
                    else if escape == b'n' { Some(10) }
                    else if escape == b'r' { Some(13) }
                    else if escape == b't' { Some(9) }
                    else { None };
                let Some(decoded) = simple_escape else { return Err(self.error("invalid JSON string escape")); };
                units.push(decoded);
                continue;
            }
            if byte < 0x20 {
                return Err(self.error("unescaped JSON control character"));
            }
            // What: A valid UTF-8 source character can encode one or two UTF-16 code units.
            // Why: Unescaped Unicode must agree with JavaScript string-value semantics.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // units.push(...utf16Encode(nextUnicodeScalar(source)));
            // ```
            let Some(ch) = self.source[self.offset..].chars().next() else { return Err(self.error("unterminated JSON string")); };
            let mut buffer = [0u16; 2];
            units.extend_from_slice(ch.encode_utf16(&mut buffer));
            self.offset += ch.len_utf8();
        }
    }

    /// What:
    ///  Parse an unrounded JSON number and retain its exact literal spelling.
    /// Why:
    ///  Large integers and exponents must not pass through `f64`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function scanNumber(): { raw: string; identity: ExactNumber };
    /// ```
    pub fn number(&mut self) -> Result<(String, JsoncNumberIdentity), JsoncParseError> {
        let start = self.offset;
        while let Some(byte) = self.byte() {
            if byte.is_ascii_digit() || byte == b'-' || byte == b'+'
                || byte == b'.' || byte == b'e' || byte == b'E' {
                self.offset += 1;
            } else {
                break;
            }
        }
        let raw = self.source[start..self.offset].to_string();
        // What: Exact-number validation returns `Result`, whose `Err` becomes a source-positioned parse error.
        // Why: No malformed numeric token or silently rounded value can enter the node tree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const identity = parseExactNumber(raw);
        // ```
        // What: `match` extracts success or returns a source-positioned error.
        // Why: A failed number scan must not be treated as a parsed value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let identity; try { identity = parseExactNumber(raw); } catch (error) { throw at(start, error); }
        // ```
        let identity = match JsoncNumberIdentity::from_token(&raw) {
            Ok(value) => value,
            Err(error) => {
                return Err(JsoncParseError {
                    offset: start,
                    message: error.to_string(),
                });
            }
        };
        return Ok((raw, identity));
    }
}
