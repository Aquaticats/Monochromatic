//! What:     Reading `toolchain.plan.jsonc`: the hand-written file where people declare which tools the toolchain
//!           holds, with comments explaining each pin.
//! Why:      The file is JSONC (comments and trailing commas allowed), so it is parsed with the repository's own
//!           `monochromatic-jsonc-edit` crate and converted to plain JSON values, then decoded once by `model.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // plan.ts
//! export function parsePlan(source: string, label: string): Plan { ... }
//! export function readPlan(path: string): Plan { ... }
//! ```

/// Imports the JSON value types (see `model.rs`).
use serde_json::{Map, Value};

/// What:     Imports the JSONC parser and the model types from the repository's `monochromatic-jsonc-edit` crate.
/// Why:      Comments are data to this parser, so a commented plan parses where `serde_json` would reject it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc, type JsoncValue, JsoncKind } from '@monochromatic-dev/jsonc-edit-core-ts';
/// ```
use monochromatic_jsonc_edit::{JsoncKind, JsoncValue, parse_jsonc, units_to_string};

use std::fs;
use std::path::Path;

use super::error::ToolchainError;
use super::model::{Declaration, decode_plan_entry};

/// The deepest nesting accepted in a plan or lock value; a real file nests four levels, so this only stops hostile
/// input from deepening the recursive conversion below.
const MAX_DEPTH: usize = 32;

/// What:     The plan: its tools in the order they were written (the sync order and the lock order follow it).
/// Why:      A `Vec` and not a map because order is part of the data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plan = { tools: Declaration[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    /// Declarations in written order, ids unique.
    pub tools: Vec<Declaration>,
}

/// What:     Converts a parsed JSONC value into a plain JSON `Value`, dropping comments and rejecting duplicate keys.
/// Why:      Two entries with one key would silently collapse in a map; for a plan that means a tool vanishing, so
///           the conversion refuses it and names the key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toJson(value: JsoncValue, path: string, depth: number): unknown
/// ```
fn to_json(value: &JsoncValue, path: &str, depth: usize) -> Result<Value, String> {
    if depth > MAX_DEPTH {
        return Err(format!("{path} nests deeper than {MAX_DEPTH} levels"));
    }
    match &value.kind {
        JsoncKind::Null => return Ok(Value::Null),
        JsoncKind::Boolean { value: flag } => return Ok(Value::Bool(*flag)),
        JsoncKind::Number { raw, .. } => {
            return match raw.parse::<serde_json::Number>() {
                Ok(number) => Ok(Value::Number(number)),
                Err(error) => Err(format!("{path} is not a usable number: {error}")),
            };
        }
        JsoncKind::Text { units, .. } => {
            return match units_to_string(units) {
                Ok(text) => Ok(Value::String(text)),
                Err(error) => Err(format!("{path} is not valid text: {error}")),
            };
        }
        JsoncKind::Array { elements } => {
            let mut items: Vec<Value> = Vec::new();
            for (index, element) in elements.iter().enumerate() {
                items.push(to_json(element, &format!("{path}[{index}]"), depth + 1)?);
            }
            return Ok(Value::Array(items));
        }
        JsoncKind::Record { entries } => {
            let mut map: Map<String, Value> = Map::new();
            for entry in entries {
                let key: String = match units_to_string(&entry.key.units) {
                    Ok(text) => text,
                    Err(error) => return Err(format!("a key of {path} is not valid text: {error}")),
                };
                if map.contains_key(&key) {
                    return Err(format!("{path} has the key {key:?} twice"));
                }
                let child: Value = to_json(&entry.value, &format!("{path}.{key}"), depth + 1)?;
                map.insert(key, child);
            }
            return Ok(Value::Object(map));
        }
    }
}

/// What:     Parses JSONC text into a JSON `Value`.
/// Why:      `label` is the file name shown in the diagnostic, so a person knows which file to open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function jsoncToValue(source: string, label: string): unknown
/// ```
pub fn jsonc_to_value(source: &str, label: &str) -> Result<Value, ToolchainError> {
    let parsed: JsoncValue = match parse_jsonc(source) {
        Ok(value) => value,
        Err(error) => return Err(ToolchainError::config(format!("{label} is not valid JSONC: {error}"))),
    };
    return match to_json(&parsed, "$", 0) {
        Ok(value) => Ok(value),
        Err(message) => Err(ToolchainError::config(format!("{label}: {message}"))),
    };
}

/// What:     Decodes plan text (`{"schemaVersion": 1, "tools": {"<id>": {...}}}`) into a `Plan`.
/// Why:      Every command starts from the plan; the checks here (known fields, unique ids, each bin name declared
///           once) make a later stage's assumptions true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parsePlan(source: string, label: string): Plan
/// ```
pub fn parse_plan(source: &str, label: &str) -> Result<Plan, ToolchainError> {
    let value: Value = jsonc_to_value(source, label)?;
    let Some(root) = value.as_object() else {
        return Err(ToolchainError::config(format!("{label} must hold a JSON object at its top level")));
    };
    for key in root.keys() {
        if key != "schemaVersion" && key != "tools" && key != "$schema" {
            return Err(ToolchainError::config(format!(
                "{label}: $.{key} is not a known field; the known fields are $schema, schemaVersion, tools"
            )));
        }
    }
    if root.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err(ToolchainError::config(format!("{label}: $.schemaVersion must be 1")));
    }
    let Some(tools_value) = root.get("tools").and_then(Value::as_object) else {
        return Err(ToolchainError::config(format!("{label}: $.tools must be an object keyed by tool id")));
    };
    let mut tools: Vec<Declaration> = Vec::new();
    for (id, entry) in tools_value {
        match decode_plan_entry(id, entry, &format!("tools.{id}")) {
            Ok(declaration) => tools.push(declaration),
            Err(message) => return Err(ToolchainError::config(format!("{label}: {message}"))),
        }
    }
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for tool in &tools {
        for bin in &tool.bins {
            for (owner, name) in &seen {
                if name == bin {
                    return Err(ToolchainError::config(format!(
                        "{label}: the binary name {bin:?} is declared by both {owner} and {}",
                        tool.id
                    )));
                }
            }
            seen.push((tool.id.as_str(), bin.as_str()));
        }
    }
    return Ok(Plan { tools });
}

/// What:     Reads and decodes the plan file at `path`.
/// Why:      A missing file is a configuration error naming the path, not a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readPlan(path: string): Plan
/// ```
pub fn read_plan(path: &Path) -> Result<Plan, ToolchainError> {
    let source: String = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            return Err(ToolchainError::config(format!("cannot read the plan {}: {error}", path.display())));
        }
    };
    return parse_plan(&source, &path.display().to_string());
}
