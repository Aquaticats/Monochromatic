//! One-line text for client-chosen strings in control answers.
//!
//! Notification and job answers carry text that hosted apps choose.
//! The answer grammar separates fields with a space,
//!  list items with `,`,
//!  parts of an item with `:`,
//!  and records with `; `,
//!  so every byte outside a small safe set is written as `%XX`.

/// Tells whether a byte passes through the encoding unchanged.
///
/// What:
///      `fn is_plain(byte: u8) -> bool`.
///  ASCII letters,
///  digits,
///  and the marks `-`,
///  `.`,
///  `_`,
///  `~`.
/// Why:
///       Space,
///  `;`,
///  `,`,
///  `:`,
///  `=`,
///  `%`,
///  and line breaks are the grammar's separators,
///  so none of them may appear raw inside a value.
fn is_plain(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~');
}

/// Percent-encodes `text` so it holds no separator of the answer grammar.
///
/// What:
///      `pub fn encode_text(text: &str) -> String`.
///  Each byte outside the plain set becomes `%` and two uppercase hexadecimal digits.
///  Multi-byte UTF-8 is escaped byte by byte,
///  and an empty string stays empty.
/// Why:
///       A summary or body from the hosted app could otherwise end its record or open a new field.
///
/// @example
/// ```ts
/// encodeText("Done; 100 %"); // => "Done%3B%20100%20%25"
/// ```
pub fn encode_text(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if is_plain(byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    return encoded;
}

/// Joins already-formatted records the way `launcher-log` does:
///  a count,
///  then the records.
///
/// What:
///      `pub fn format_records(records: &[String]) -> String`.
///  No records give `0`.
///  Otherwise the count,
///  one space,
///  and the records joined by `; `.
/// Why:
///       The count tells the reader how many records follow without counting separators.
pub fn format_records(records: &[String]) -> String {
    if records.is_empty() {
        return "0".to_string();
    }
    return format!("{} {}", records.len(), records.join("; "));
}

/// Unit tests for the encoding and the record list.
#[cfg(test)]
#[path = "wire_text_tests.rs"]
mod tests;
