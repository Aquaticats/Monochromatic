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

/// What:
///  Imports the converter and the location table.
/// Why:
///  Loading a config means parsing it,
/// converting it,
/// and keeping where every node sits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { convertDocument } from './configConvert';
/// import { ConfigLocations } from './configLocations';
/// ```
use crate::config_convert::convert_document;
/// Imports `ConfigLocations` from `crate::config_locations`.
use crate::config_locations::ConfigLocations;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// What:
///  Imports the span-reporting parse and the shared line and column rule.
/// Why:
///  The config's positions come from the same parse that builds the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { jsoncPositionAt, parseJsoncWithSpans } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{jsonc_position_at, parse_jsonc_with_spans};
/// What:
///  Imports `Value` from `serde_json`.
/// Why:
///  The decoder reads a plain JSON value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // JSON values are plain JS objects in TS
/// ```
use serde_json::Value;
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
    let position = jsonc_position_at(source, offset);
    return (position.line, position.column);
}

/// What:
///  A parsed config:
/// the plain JSON value and where every node of it sits in the file.
/// Why:
///  The decoder reads `value` and asks `locations` for the `file:line:col` of every problem it reports,
///  so the two can never be paired with different files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigDocument = { value: unknown; locations: ConfigLocations };
/// ```
#[derive(Clone, Debug)]
pub struct ConfigDocument {
    /// The config as plain JSON (comments dropped,
    /// trailing commas accepted).
    pub value: Value,
    /// Where each node of `value` sits in the file.
    pub locations: ConfigLocations,
}

/// What:
///  Parse JSONC text into a plain JSON value plus the location of every node;
/// comments are dropped,
/// trailing commas are accepted.
/// Why:
///  Duplicate object keys and unpaired UTF-16 surrogates cannot survive in a `serde_json::Value`,
///  so they are rejected here with the key named instead of silently keeping the last one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConfigText(source: string, displayPath: string): ConfigDocument;
/// ```
pub fn parse_config_text(source: &str, display_path: &str) -> Result<ConfigDocument, Diagnostic> {
    let parsed = match parse_jsonc_with_spans(source) {
        Ok(value) => value,
        Err(error) => {
            let (line, column) = line_and_column(source, error.offset);
            let message = format!("{display_path}:{line}:{column}: {}", error.message);
            return Err(Diagnostic::new("config-syntax", &message).with_path(display_path));
        }
    };
    let (value, locations) = convert_document(&parsed.value, &parsed.spans, display_path)?;
    return Ok(ConfigDocument { value, locations });
}

/// What:
///  Read a JSONC file from disk and parse it.
/// Why:
///  A missing or unreadable config is a command-line level failure with its own code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadConfigFile(path: string): ConfigDocument;
/// ```
pub fn load_config_file(path: &Path) -> Result<ConfigDocument, Diagnostic> {
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
