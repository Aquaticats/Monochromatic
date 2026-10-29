//! Plain-text summaries of D-Bus variant maps (notification hints and job properties).
//!
//! The stores hold these summaries instead of D-Bus values,
//!  so their records are comparable,
//!  printable,
//!  and testable without a bus.

/// Imports `HashMap` from `std::collections`.
use std::collections::HashMap;

/// Imports `OwnedValue`,
///  `Value` from `zbus::zvariant`.
use zbus::zvariant::{OwnedValue, Value};

/// Imports `encode_text` from `crate::wire_text`.
use crate::wire_text::encode_text;

/// One map entry reduced to its key,
///  its D-Bus type,
///  and a printable value.
///
/// What:
///      `pub struct ValueSummary { key, signature, text }`.
///  `text` is the value for plain types,
///  `array-N` or `dict-N` for containers with `N` items,
///  and `struct` for a structure.
///  Byte arrays such as `image-data` show only their length,
///  never their pixels.
/// Why:
///       A test needs to see the type a client sent (an `int32` where `uint32` was required
///  is the classic wire mistake) and the value,
///  on one line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueSummary {
    /// The map key,
    ///  such as `urgency` or `percent`.
    pub key: String,
    /// The D-Bus type signature of the value,
    ///  such as `u` or `s`.
    pub signature: String,
    /// The printable value,
    ///  not yet percent-encoded.
    pub text: String,
}

/// Renders one value as printable text.
///
/// What:
///      `pub fn describe_value(value: &Value<'_>) -> String`.
///  A variant inside a variant is unwrapped to the innermost value.
/// Why:
///       Doubles,
///  booleans,
///  and integers print as a reader expects.
#[must_use]
pub fn describe_value(value: &Value<'_>) -> String {
    return match value {
        Value::U8(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::I16(number) => number.to_string(),
        Value::U16(number) => number.to_string(),
        Value::I32(number) => number.to_string(),
        Value::U32(number) => number.to_string(),
        Value::I64(number) => number.to_string(),
        Value::U64(number) => number.to_string(),
        Value::F64(number) => number.to_string(),
        Value::Str(text) => text.to_string(),
        Value::Signature(signature) => signature.to_string(),
        Value::ObjectPath(path) => path.to_string(),
        Value::Value(inner) => describe_value(inner),
        Value::Array(array) => format!("array-{}", array.len()),
        Value::Dict(dict) => format!("dict-{}", dict.iter().count()),
        Value::Structure(_) => "struct".to_string(),
        _ => "other".to_string(),
    };
}

/// Summarizes a variant map,
///  sorted by key so the order never depends on hashing.
///
/// What:
///      `pub fn summarize_map(map: &HashMap<String, OwnedValue>) -> Vec<ValueSummary>`.
/// Why:
///       A `HashMap` iterates in an unstable order,
///  and a log line must read the same every run.
#[must_use]
pub fn summarize_map(map: &HashMap<String, OwnedValue>) -> Vec<ValueSummary> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    let mut summaries = Vec::with_capacity(keys.len());
    for key in keys {
        if let Some(value) = map.get(key) {
            summaries.push(ValueSummary {
                key: key.clone(),
                signature: value.value_signature().to_string(),
                text: describe_value(value),
            });
        }
    }
    return summaries;
}

/// Writes summaries as `key:signature:value` items separated by `,`.
///
/// What:
///      `pub fn format_summaries(summaries: &[ValueSummary]) -> String`.
///  Every part is percent-encoded,
///  so no part can contain `,` or `:`.
///  No entries give an empty string.
/// Why:
///       The list sits inside a `key=value` field of a record.
#[must_use]
pub fn format_summaries(summaries: &[ValueSummary]) -> String {
    let mut items = Vec::with_capacity(summaries.len());
    for summary in summaries {
        items.push(format!(
            "{}:{}:{}",
            encode_text(&summary.key),
            encode_text(&summary.signature),
            encode_text(&summary.text),
        ));
    }
    return items.join(",");
}

/// Unit tests for the summaries.
#[cfg(test)]
#[path = "value_summary_tests.rs"]
mod tests;
