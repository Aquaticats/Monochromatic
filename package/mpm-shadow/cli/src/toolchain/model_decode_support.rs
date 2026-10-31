//! What:
//! The small checks the decoders share:
//! a value is an object,
//! a text,
//! a non-empty text or a list of texts,
//! and
//!           an object has no field outside an allowed set;
//! each failure names the path of the wrong field.
//! Why:
//! Keeps `model_decode.rs` about the shape of the plan and lock,
//! and these checks in one tested place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // model-decode-support.ts
//! export function asObject(value: unknown, path: string): Record<string, unknown>
//! export function rejectUnknownKeys(map: Record<string, unknown>, allowed: string[], path: string): void
//! ```

/// Imports the JSON value types.
use serde_json::{Map, Value};

/// A decoding failure message that already names the path of the wrong field (`tools.node.bins[1]`).
pub type DecodeResult<T> = Result<T, String>;

/// The JSON kind name used in messages (`"an object"`).
pub(super) fn kind_name(value: &Value) -> &'static str {
    match value {
        Value::Null => return "null",
        Value::Bool(_) => return "a boolean",
        Value::Number(_) => return "a number",
        Value::String(_) => return "a string",
        Value::Array(_) => return "an array",
        Value::Object(_) => return "an object",
    }
}

/// `value` as an object,
/// or a message naming `path`.
pub(super) fn as_object<'a>(value: &'a Value, path: &str) -> DecodeResult<&'a Map<String, Value>> {
    if let Some(map) = value.as_object() {
        return Ok(map);
    }
    return Err(format!("{path} must be an object, found {}", kind_name(value)));
}

/// `value` as text,
/// or a message naming `path`.
pub(super) fn as_text(value: &Value, path: &str) -> DecodeResult<String> {
    if let Some(text) = value.as_str() {
        return Ok(text.to_string());
    }
    return Err(format!("{path} must be a string, found {}", kind_name(value)));
}

/// `value` as a non-empty text,
/// or a message naming `path`.
pub(super) fn as_nonempty_text(value: &Value, path: &str) -> DecodeResult<String> {
    let text: String = as_text(value, path)?;
    if text.is_empty() {
        return Err(format!("{path} must not be empty"));
    }
    return Ok(text);
}

/// `value` as a list of texts,
/// or a message naming `path` and the index of the first wrong element.
pub(super) fn as_text_list(value: &Value, path: &str) -> DecodeResult<Vec<String>> {
    let Some(items) = value.as_array() else {
        return Err(format!("{path} must be an array of strings, found {}", kind_name(value)));
    };
    let mut texts: Vec<String> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        texts.push(as_nonempty_text(item, &format!("{path}[{index}]"))?);
    }
    return Ok(texts);
}

/// Fails when `map` has a key outside `allowed`,
/// naming the key and the allowed ones.
pub(super) fn reject_unknown_keys(map: &Map<String, Value>, allowed: &[&str], path: &str) -> DecodeResult<()> {
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{path}.{key} is not a known field; the known fields are {}", allowed.join(", ")));
        }
    }
    return Ok(());
}
