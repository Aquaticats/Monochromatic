//! What:
//!  Reads the Node.js release list out of node-releases' `data/processed/envs.json`.
//! Why:
//!  browserslist's `last N node major versions` uses only the `version` field of each record,
//! in file order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const versions = JSON.parse(envsJson).map((release) => release.version);
//! ```

/// What:
///  Import `BrowserslistError, Outcome` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome } from './error';
/// ```
use crate::error::{BrowserslistError, Outcome};
/// What:
///  Import `Value` from `serde_json::Value`.
/// Why:
///  JSON documents are read and written through serde_json.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Value } from 'serde_json';
/// ```
use serde_json::Value;

/// Diagnostic name of the release file.
const ENVS_FILE: &str = "node-releases data/processed/envs.json";

/// Build a data error for the release file without a byte offset.
fn envs_error(message: &str) -> BrowserslistError {
    return BrowserslistError::Data { file: ENVS_FILE.to_string(), offset: None, message: message.to_string() };
}

/// What:
///  Parse `envs.json` into the `version` strings of its records,
/// keeping file order.
/// Why:
///  The selector relies on the file being ascending and must not sort or dedupe it.
/// Gotcha:
///  `lts` is a boolean or a codename string,
/// so the records are read as untyped JSON values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseNodeVersions(text: string): string[];
/// ```
pub fn parse_node_versions(text: &str) -> Outcome<Vec<String>> {
    let parsed: Value = match serde_json::from_str(text) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Err(BrowserslistError::Data {
                file: ENVS_FILE.to_string(),
                offset: Some(0),
                message: format!("not valid JSON: {error}"),
            });
        }
    };
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

/// What:
///  The unit tests of `node_releases.rs`,
/// kept in the sidecar file `node_releases_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './node_releases.test';
/// ```
#[cfg(test)]
#[path = "node_releases_tests.rs"]
mod tests;
