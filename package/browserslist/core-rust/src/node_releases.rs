//! What:
//!  Reads the Node.js release list out of node-releases' `data/processed/envs.json`.
//! Why:
//!  browserslist's `last N node major versions` uses only the `version` field of each record, in file order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const versions = JSON.parse(envsJson).map((release) => release.version);
//! ```

use crate::error::{BrowserslistError, Outcome};
use serde_json::Value;

/// Diagnostic name of the release file.
const ENVS_FILE: &str = "node-releases data/processed/envs.json";

/// Build a data error for the release file without a byte offset.
fn envs_error(message: &str) -> BrowserslistError {
    return BrowserslistError::Data { file: ENVS_FILE.to_string(), offset: None, message: message.to_string() };
}

/// What:
///  Parse `envs.json` into the `version` strings of its records, keeping file order.
/// Why:
///  The selector relies on the file being ascending and must not sort or dedupe it.
/// Gotcha:
///  `lts` is a boolean or a codename string, so the records are read as untyped JSON values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseNodeVersions(text: string): string[];
/// ```
pub fn parse_node_versions(text: &str) -> Outcome<Vec<String>> {
    let parsed: Value = serde_json::from_str(text).map_err(|error| {
        return BrowserslistError::Data {
            file: ENVS_FILE.to_string(),
            offset: Some(0),
            message: format!("not valid JSON: {error}"),
        };
    })?;
    let Value::Array(records) = parsed else {
        return Err(envs_error("top level is not an array"));
    };
    let mut versions: Vec<String> = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        let Some(Value::String(version)) = record.get("version") else {
            return Err(envs_error(&format!("record {index} has no string version")));
        };
        versions.push(version.clone());
    }
    return Ok(versions);
}

#[cfg(test)]
#[path = "node_releases_tests.rs"]
mod tests;
