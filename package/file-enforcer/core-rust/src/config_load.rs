//! What:
//!  Read a JSONC config file into a plain JSON value,
//! with positioned errors.
//! Why:
//!  The decoder works on `serde_json::Value`,
//! the schema corpus check feeds the same value to the JSON Schema
//!  validator,
//! and syntax errors need `path:line:col` instead of a byte offset.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function loadJsonc(path: string): unknown; // throws a Diagnostic naming path:line:col
//! ```

/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `JsoncKind`,
/// `JsoncValue`,
/// `parse_jsonc` from `monochromatic_jsonc_edit`.
use monochromatic_jsonc_edit::{JsoncKind, JsoncValue, parse_jsonc};
/// Imports `Map`,
/// `Number`,
/// `Value` from `serde_json`.
use serde_json::{Map, Number, Value};
/// Imports `Path` from `std::path`.
use std::path::Path;

/// What:
///  Turn a byte offset in `source` into a one-based line and column.
///  `(usize, usize)` is a pair of unsigned machine-size integers (TS:
/// `[number, number]`).
/// Why:
///  `JsoncParseError` only carries an offset;
/// people read `path:line:col`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineAndColumn(source: string, offset: number): [number, number];
/// ```
pub fn line_and_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line: usize = 1;
    let mut column: usize = 1;
    for (index, character) in source.char_indices() {
        if index >= offset {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    return (line, column);
}

/// What:
///  Parse JSONC text into a plain JSON value;
/// comments are dropped,
/// trailing commas are accepted.
/// Why:
///  Duplicate object keys and unpaired UTF-16 surrogates cannot survive in a `serde_json::Value`,
///  so they are rejected here with the key named instead of silently keeping the last one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConfigText(source: string, displayPath: string): unknown;
/// ```
pub fn parse_config_text(source: &str, display_path: &str) -> Result<Value, Diagnostic> {
    let parsed = match parse_jsonc(source) {
        Ok(value) => value,
        Err(error) => {
            let (line, column) = line_and_column(source, error.offset);
            let message = format!("{display_path}:{line}:{column}: {}", error.message);
            return Err(Diagnostic::new("config-syntax", &message).with_path(display_path));
        }
    };
    return convert_value(&parsed, "", display_path);
}

/// What:
///  Read a JSONC file from disk and parse it.
/// Why:
///  A missing or unreadable config is a command-line level failure with its own code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadConfigFile(path: string): unknown;
/// ```
pub fn load_config_file(path: &Path) -> Result<Value, Diagnostic> {
    let display_path = path.display().to_string();
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("cannot read the config file {display_path}: {error}");
            return Err(Diagnostic::new("config-unreadable", &message).with_path(&display_path));
        }
    };
    let source = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            let message = format!("{display_path} is not valid UTF-8");
            return Err(Diagnostic::new("config-syntax", &message).with_path(&display_path));
        }
    };
    return parse_config_text(&source, &display_path);
}

/// What:
///  Convert one JSONC value (and its children) into a `serde_json::Value`,
/// recording a JSON pointer for errors.
/// Why:
///  A recursive walk is fine here:
/// the parser already refuses more than 512 nested containers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convert(value: JsoncValue, pointer: string): unknown;
/// ```
fn convert_value(value: &JsoncValue, pointer: &str, display_path: &str) -> Result<Value, Diagnostic> {
    match &value.kind {
        JsoncKind::Null => return Ok(Value::Null),
        JsoncKind::Boolean { value: flag } => return Ok(Value::Bool(*flag)),
        JsoncKind::Text { units, .. } => {
            let text = match String::from_utf16(units) {
                Ok(text) => text,
                Err(_) => {
                    let message = format!("{display_path}: the string at {pointer} holds an unpaired surrogate");
                    return Err(Diagnostic::new("config-syntax", &message).with_path(display_path));
                }
            };
            return Ok(Value::String(text));
        }
        JsoncKind::Number { raw, .. } => {
            return convert_number(raw, pointer, display_path);
        }
        JsoncKind::Array { elements } => {
            let mut items: Vec<Value> = Vec::new();
            for (index, element) in elements.iter().enumerate() {
                let child_pointer = format!("{pointer}/{index}");
                items.push(convert_value(element, &child_pointer, display_path)?);
            }
            return Ok(Value::Array(items));
        }
        JsoncKind::Record { entries } => {
            let mut object: Map<String, Value> = Map::new();
            for entry in entries {
                let key = match String::from_utf16(&entry.key.units) {
                    Ok(key) => key,
                    Err(_) => {
                        let message = format!("{display_path}: a key under {pointer} holds an unpaired surrogate");
                        return Err(Diagnostic::new("config-syntax", &message).with_path(display_path));
                    }
                };
                if object.contains_key(&key) {
                    let message = format!("{display_path}: the key {key:?} appears twice under {pointer}");
                    return Err(Diagnostic::new("config-syntax", &message).with_path(display_path));
                }
                let child_pointer = format!("{pointer}/{key}");
                object.insert(key, convert_value(&entry.value, &child_pointer, display_path)?);
            }
            return Ok(Value::Object(object));
        }
    }
}

/// What:
///  Convert a number token into a JSON number.
/// Why:
///  The config only uses small integers;
/// anything else is reported with its pointer instead of being rounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertNumber(raw: string, pointer: string): number;
/// ```
fn convert_number(raw: &str, pointer: &str, display_path: &str) -> Result<Value, Diagnostic> {
    if let Ok(integer) = raw.parse::<i64>() {
        return Ok(Value::Number(Number::from(integer)));
    }
    if let Ok(float) = raw.parse::<f64>()
        && let Some(number) = Number::from_f64(float)
    {
        return Ok(Value::Number(number));
    }
    let message = format!("{display_path}: the number {raw} at {pointer} is not representable");
    return Err(Diagnostic::new("config-syntax", &message).with_path(display_path));
}
