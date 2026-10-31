//! What:
//!  A linear-time parser for exactly the JavaScript literal subset that caniuse-lite's packed data files use.
//! Why:
//!  `agents.js`, `browsers.js` and `browserVersions.js` are `module.exports=<literal>;` files,
//!  not JSON, and running JavaScript to read them is out of the question.
//!  Anything outside the subset is an error naming the file and byte offset.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parseModuleExports(file: string, source: string): JsValue; // throws on anything outside the subset
//! ```

use crate::error::{Outcome, data_error};
use std::collections::HashSet;

/// What:
///  The deepest container nesting the parser accepts.
///  `usize` (not `u32` or `u64`) because it is compared with a recursion counter used as a length.
/// Why:
///  The real files nest three levels;
///  a hard cap keeps the recursion's stack use constant for hostile input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_DEPTH = 8;
/// ```
const MAX_DEPTH: usize = 8;

/// What:
///  The exact text every packed data file starts with.
/// Why:
///  The envelope is part of the subset: a file wrapped any other way is not one this parser vouches for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PREFIX = 'module.exports=';
/// ```
const PREFIX: &str = "module.exports=";

/// What:
///  A parsed literal: object, array, string, number or `null`.
///  Numbers keep their source text because no consumer needs their value.
/// Why:
///  The decoders pick out strings and arrays only;
///  keeping number text avoids pulling in float parsing for data the queries never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsValue = { [key: string]: JsValue } | JsValue[] | string | { number: string } | null;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsValue {
    /// `{ key: value, ... }` with keys in source order and no duplicates.
    Object(Vec<(String, JsValue)>),
    /// `[ value, ... ]`.
    Array(Vec<JsValue>),
    /// A double-quoted string without escapes.
    Text(String),
    /// A non-negative integer or plain decimal, as written.
    Number(String),
    /// The `null` keyword.
    Null,
}

/// What:
///  Look up a key in an object value;
///  returns `None` (the "absent" variant of `Option`) for other value kinds or a missing key.
/// Why:
///  The decoders walk `agents.js` by key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function get(value: JsValue, key: string): JsValue | undefined;
/// ```
pub fn object_get<'a>(value: &'a JsValue, key: &str) -> Option<&'a JsValue> {
    if let JsValue::Object(entries) = value {
        for (entry_key, entry_value) in entries {
            if entry_key == key {
                return Some(entry_value);
            }
        }
    }
    return None;
}

/// What:
///  A cursor over the source bytes plus the file name used in diagnostics.
/// Why:
///  Tracking the offset in one place lets every error name the exact byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cursor = { file: string; bytes: Uint8Array; offset: number };
/// ```
struct Cursor<'a> {
    file: &'a str,
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    /// Build the error at the current offset.
    fn fail(&self, message: &str) -> crate::error::BrowserslistError {
        return data_error(self.file, self.offset, message);
    }

    /// Return the current byte without consuming it.
    fn peek(&self) -> Option<u8> {
        return self.bytes.get(self.offset).copied();
    }

    /// Consume one expected byte or fail.
    fn expect(&mut self, wanted: u8, what: &str) -> Outcome<()> {
        if self.peek() == Some(wanted) {
            self.offset += 1;
            return Ok(());
        }
        return Err(self.fail(&format!("expected {what}")));
    }

    /// Parse a double-quoted string: printable ASCII only, no backslash escapes.
    fn string(&mut self) -> Outcome<String> {
        self.expect(b'"', "a double-quoted string")?;
        let start = self.offset;
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.fail("unterminated string"));
            };
            if byte == b'"' {
                break;
            }
            if !(0x20..=0x7e).contains(&byte) || byte == b'\\' {
                return Err(self.fail("string holds a byte outside printable ASCII or an escape"));
            }
            self.offset += 1;
        }
        let text = String::from_utf8_lossy(&self.bytes[start..self.offset]).into_owned();
        self.offset += 1;
        return Ok(text);
    }

    /// Parse an object key: a double-quoted string, or an identifier that starts with a letter.
    fn key(&mut self) -> Outcome<String> {
        if self.peek() == Some(b'"') {
            return self.string();
        }
        let start = self.offset;
        if !self.peek().is_some_and(|byte| byte.is_ascii_alphabetic()) {
            return Err(self.fail("expected an object key"));
        }
        while self.peek().is_some_and(|byte| byte.is_ascii_alphanumeric()) {
            self.offset += 1;
        }
        return Ok(String::from_utf8_lossy(&self.bytes[start..self.offset]).into_owned());
    }

    /// Parse `0` or a digit run without a leading zero, then an optional `.` and one or more digits.
    fn number(&mut self) -> Outcome<JsValue> {
        let start = self.offset;
        if self.peek() == Some(b'0') {
            self.offset += 1;
        } else {
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.offset += 1;
            }
        }
        if self.peek() == Some(b'.') {
            self.offset += 1;
            let digits_start = self.offset;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.offset += 1;
            }
            if self.offset == digits_start {
                return Err(self.fail("a decimal point must be followed by digits"));
            }
        }
        if self.peek().is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'.') {
            return Err(self.fail("number followed by a byte outside the subset"));
        }
        return Ok(JsValue::Number(String::from_utf8_lossy(&self.bytes[start..self.offset]).into_owned()));
    }

    /// Parse the `null` keyword.
    fn null(&mut self) -> Outcome<JsValue> {
        if self.bytes[self.offset..].starts_with(b"null") {
            self.offset += 4;
            if self.peek().is_some_and(|byte| byte.is_ascii_alphanumeric()) {
                return Err(self.fail("identifier continues after null"));
            }
            return Ok(JsValue::Null);
        }
        return Err(self.fail("unknown keyword; only null is in the subset"));
    }

    /// Parse any value; `depth` counts the containers already open.
    fn value(&mut self, depth: usize) -> Outcome<JsValue> {
        match self.peek() {
            Some(b'{') => return self.object(depth + 1),
            Some(b'[') => return self.array(depth + 1),
            Some(b'"') => return Ok(JsValue::Text(self.string()?)),
            Some(byte) if byte.is_ascii_digit() => return self.number(),
            Some(b'n') => return self.null(),
            _ => return Err(self.fail("unexpected byte; not in the literal subset")),
        }
    }

    /// Parse `{ key: value, ... }`, rejecting trailing commas and duplicate keys.
    fn object(&mut self, depth: usize) -> Outcome<JsValue> {
        if depth > MAX_DEPTH {
            return Err(self.fail("containers nest deeper than the subset allows"));
        }
        self.expect(b'{', "an object")?;
        let mut entries: Vec<(String, JsValue)> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        if self.peek() == Some(b'}') {
            self.offset += 1;
            return Ok(JsValue::Object(entries));
        }
        loop {
            let key_offset = self.offset;
            let key = self.key()?;
            if !seen.insert(key.clone()) {
                self.offset = key_offset;
                return Err(self.fail("duplicate object key"));
            }
            self.expect(b':', "a colon after the key")?;
            let value = self.value(depth)?;
            entries.push((key, value));
            match self.peek() {
                Some(b',') => self.offset += 1,
                Some(b'}') => {
                    self.offset += 1;
                    return Ok(JsValue::Object(entries));
                }
                _ => return Err(self.fail("expected a comma or a closing brace")),
            }
        }
    }

    /// Parse `[ value, ... ]`, rejecting trailing commas and holes.
    fn array(&mut self, depth: usize) -> Outcome<JsValue> {
        if depth > MAX_DEPTH {
            return Err(self.fail("containers nest deeper than the subset allows"));
        }
        self.expect(b'[', "an array")?;
        let mut items: Vec<JsValue> = Vec::new();
        if self.peek() == Some(b']') {
            self.offset += 1;
            return Ok(JsValue::Array(items));
        }
        loop {
            items.push(self.value(depth)?);
            match self.peek() {
                Some(b',') => self.offset += 1,
                Some(b']') => {
                    self.offset += 1;
                    return Ok(JsValue::Array(items));
                }
                _ => return Err(self.fail("expected a comma or a closing bracket")),
            }
        }
    }
}

/// What:
///  Parse `module.exports=<literal>;` plus a final newline into a value.
/// Why:
///  This is the single entry point for the three caniuse-lite files.
/// Gotcha:
///  The envelope is exact: no space after `=`, a `;`, and exactly one trailing `\n`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseModuleExports(file: string, source: string): JsValue;
/// ```
pub fn parse_module_exports(file: &str, source: &str) -> Outcome<JsValue> {
    let mut cursor = Cursor { file, bytes: source.as_bytes(), offset: 0 };
    if !source.starts_with(PREFIX) {
        return Err(cursor.fail("file does not start with `module.exports=`"));
    }
    cursor.offset = PREFIX.len();
    let value = cursor.value(0)?;
    cursor.expect(b';', "a semicolon after the value")?;
    cursor.expect(b'\n', "a newline after the semicolon")?;
    if cursor.offset != cursor.bytes.len() {
        return Err(cursor.fail("bytes remain after the final newline"));
    }
    return Ok(value);
}

#[cfg(test)]
#[path = "js_literal_tests.rs"]
mod tests;
