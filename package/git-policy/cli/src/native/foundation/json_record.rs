//! What:
//!  Reading and writing the compact JSON records both wrappers share,
//!  with the
//!       incumbent's `JSON.parse` and `JSON.stringify` semantics.
//! Why:
//!  Lock,
//!  owner,
//!  journal and capture records written by either wrapper are read by the
//!      other.
//!  A record must be read the way the incumbent reads it (numbers are doubles,
//!  a
//!      whole number in safe-integer range is an integer,
//!  the last duplicate key wins) and
//!      written with the same field order and escaping.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const value: unknown = JSON.parse(text); const bytes = `${JSON.stringify(record)}\n`;
//! ```

/// What:
///  `Map` is a JSON object's key/value table and `Value` any JSON value.
/// Why:
///   Records are untrusted until each field is checked,
///  so they are parsed into the
///       general form first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::{Map, Value};

/// The largest whole number a double holds exactly,
///  JavaScript's `Number.MAX_SAFE_INTEGER`.
pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

/// What:
///  Parse record text into a JSON object,
///  or nothing for any other document.
///       `Option<Map<String, Value>>` is "an object,
///  or nothing".
/// Why:
///   Every record is an object;
///  an array,
///  a scalar or text that is not JSON is malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseObject(text: string): Record<string, unknown> | undefined;
/// ```
pub fn parse_object(text: &str) -> Option<Map<String, Value>> {
    // `match` unpacks the parse result; any parse failure is "no object".
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(map)) => return Some(map),
        Ok(_) | Err(_) => return None,
    }
}

/// What:
///  Decode record bytes the way a `fatal` `TextDecoder` does:
///  valid UTF-8 only,
///  with one
///       leading byte-order mark removed.
/// Why:
///   The incumbent decodes owner and journal records with `new TextDecoder('utf-8',
///       { fatal: true })`,
///  which rejects invalid bytes and drops a leading mark.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder('utf-8', { fatal: true }).decode(bytes)
/// ```
pub fn decode_fatal(bytes: &[u8]) -> Option<&str> {
    let text: &str = std::str::from_utf8(bytes).ok()?;
    // `strip_prefix` returns the rest after the mark, when the text starts with it.
    return Some(text.strip_prefix('\u{feff}').unwrap_or(text));
}

/// What:
///  Read a field as text,
///  or nothing when it is absent or not a string.
/// Why:
///   `typeof value.key === 'string'` is the incumbent's check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typeof value[key] === 'string' ? value[key] : undefined
/// ```
pub fn string_field<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    return map.get(key)?.as_str();
}

/// What:
///  Read a field as a JavaScript safe integer,
///  or nothing.
///       `i64` is a signed 64-bit integer.
/// Why:
///   `typeof value === 'number' && Number.isSafeInteger(value)` accepts `1`,
///  `1.0` and
///       `1e0` alike,
///  because JavaScript has only doubles;
///  anything with a fraction or beyond
///       2^53 - 1 is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typeof value[key] === 'number' && Number.isSafeInteger(value[key]) ? value[key] : undefined
/// ```
pub fn safe_integer_field(map: &Map<String, Value>, key: &str) -> Option<i64> {
    return safe_integer(map.get(key)?);
}

/// What:
///  The safe-integer reading of one JSON value.
/// Why:
///   Shared by object fields and array items.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function safeInteger(value: unknown): number | undefined;
/// ```
pub fn safe_integer(value: &Value) -> Option<i64> {
    // `let Value::Number(number) = value else { ... }` keeps only numbers.
    let Value::Number(number) = value else {
        return None;
    };
    if let Some(whole) = number.as_i64() {
        return in_safe_range(whole);
    }
    if number.as_u64().is_some() {
        // Any `u64` that does not fit `i64` is far beyond the safe range.
        return None;
    }
    let double: f64 = number.as_f64()?;
    // A double is a safe integer when it has no fraction and lies within ±(2^53 - 1).
    if double.fract() != 0.0 || double.abs() > MAX_SAFE_INTEGER as f64 {
        return None;
    }
    // The cast is exact: the value is whole and within range.
    return Some(double as i64);
}

/// What:
///  Keep a whole number only when it lies within the safe range.
/// Why:
///   `Number.isSafeInteger` rejects magnitudes beyond 2^53 - 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.abs(whole) <= Number.MAX_SAFE_INTEGER ? whole : undefined
/// ```
fn in_safe_range(whole: i64) -> Option<i64> {
    if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&whole) {
        return None;
    }
    return Some(whole);
}

/// What:
///  Whether a field holds exactly the JSON number `expected`.
/// Why:
///   `value.schemaVersion !== 1` compares doubles,
///  so `1.0` matches `1`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value[key] === expected
/// ```
pub fn number_equals(map: &Map<String, Value>, key: &str, expected: i64) -> bool {
    return safe_integer_field(map, key) == Some(expected);
}

/// What:
///  Builder of one compact JSON object in insertion order,
///  as `JSON.stringify` writes it.
///       A `struct` groups named fields;
///  this one holds the text written so far.
/// Why:
///   Records must list their fields in the incumbent's order so the bytes match for the
///       same values;
///  escaping comes from `serde_json`,
///  which escapes the same characters
///       `JSON.stringify` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ a: 1, b: 'x' })
/// ```
#[derive(Debug, Default)]
pub struct ObjectWriter {
    /// The object text without its closing brace.
    text: String,
}

/// Methods that append one field each and finish the object.
impl ObjectWriter {
    /// What:
    ///  Start an empty object.
    /// Why:
    ///   Every record starts the same way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const record = {};
    /// ```
    pub fn new() -> Self {
        return Self {
            text: String::from("{"),
        };
    }

    /// What:
    ///  Append the separator and the quoted key of the next field.
    /// Why:
    ///   Every field kind starts the same way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// text += (first ? '' : ',') + JSON.stringify(key) + ':';
    /// ```
    fn key(&mut self, key: &str) {
        if self.text.len() > 1 {
            self.text.push(',');
        }
        self.text.push_str(quote(key).as_str());
        self.text.push(':');
    }

    /// What:
    ///  Append a text field.
    /// Why:
    ///   Most record fields are text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record[key] = value;
    /// ```
    pub fn string(&mut self, key: &str, value: &str) -> &mut Self {
        self.key(key);
        self.text.push_str(quote(value).as_str());
        return self;
    }

    /// What:
    ///  Append a whole-number field.
    /// Why:
    ///   Schema versions,
    ///  process IDs and attempt numbers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record[key] = value;
    /// ```
    pub fn integer(&mut self, key: &str, value: i64) -> &mut Self {
        self.key(key);
        self.text.push_str(value.to_string().as_str());
        return self;
    }

    /// What:
    ///  Append a boolean field.
    /// Why:
    ///   Flags such as whether a prepared commit is signed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record[key] = value;
    /// ```
    pub fn boolean(&mut self, key: &str, value: bool) -> &mut Self {
        self.key(key);
        self.text.push_str(if value { "true" } else { "false" });
        return self;
    }

    /// What:
    ///  Append a field whose value is already complete JSON text.
    /// Why:
    ///   Nested objects and arrays are built by their own writers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record[key] = nested;
    /// ```
    pub fn raw(&mut self, key: &str, json: &str) -> &mut Self {
        self.key(key);
        self.text.push_str(json);
        return self;
    }

    /// What:
    ///  The finished object text.
    /// Why:
    ///   Callers add the record's line terminator themselves.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// JSON.stringify(record)
    /// ```
    pub fn finish(&self) -> String {
        return format!("{}}}", self.text);
    }
}

/// What:
///  Quote text as a JSON string literal.
/// Why:
///   `serde_json` escapes `"`,
///  `\` and control characters as `JSON.stringify` does;
///       serializing a `&str` cannot fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(text)
/// ```
pub fn quote(text: &str) -> String {
    // `unwrap_or_default` is never reached: a `&str` always serializes.
    return serde_json::to_string(text).unwrap_or_default();
}

/// What:
///  A JSON array of text items.
/// Why:
///   Pathspec and path lists in journal records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(items)
/// ```
pub fn string_array(items: &[String]) -> String {
    // `mut` allows appending one item at a time.
    let mut text: String = String::from("[");
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            text.push(',');
        }
        text.push_str(quote(item.as_str()).as_str());
    }
    text.push(']');
    return text;
}

/// Parsing and writing controls stay out of the release executable.
#[cfg(test)]
#[path = "json_record_tests.rs"]
mod tests;
