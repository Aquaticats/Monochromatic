//! What:
//!  The JSON half of the JetBrains Harper policy:
//! the flat server configuration and the settings schema embedded in the IDE's XML.
//! Why:
//!  The IDE stores each language server's configuration as a JSON document inside an XML attribute.
//!  The policy patches that document with `serde_json`
//!  (strict output,
//! insertion-ordered objects),
//! so unrelated keys keep their position and a conforming document is byte-stable.
//!  The two-space layout matches `JSON.stringify(value, null, 2)`,
//! which the retired TypeScript tool wrote.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function patchedConfig(existing: string, disabled: string[], exclude: string[]): string
//! ```

/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{Map, Value};

/// The configuration key that lists the files the server ignores.
pub const EXCLUDE_KEY: &str = "harper-ls.excludePatterns";

/// The schema property the policy defaults when the IDE's schema lacks it.
pub const TITLE_CASE_PROPERTY: &str = "harper-ls.linters.UseTitleCase";

/// The description the retired TypeScript policy gave the title-case property.
pub const TITLE_CASE_DESCRIPTION: &str = "Prompts you to use title case in relevant headings.";

/// What:
///  Parse JSON text that must be an object.
/// Why:
///  A document that is not an object is not Harper configuration,
/// and rewriting it would destroy the user's data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseObject(text: string, label: string): Record<string, unknown>
/// ```
pub fn parse_object(text: &str, label: &str, path: &str) -> Result<Map<String, Value>, Diagnostic> {
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(object)) => return Ok(object),
        Ok(_) => {
            let message = format!("{path}: {label} must be a JSON object");
            return Err(Diagnostic::new("harper-json-invalid", &message).with_path(path));
        }
        Err(error) => {
            let message = format!("{path}: {label} is not valid JSON: {error}");
            return Err(Diagnostic::new("harper-json-invalid", &message).with_path(path));
        }
    }
}

/// What:
///  The key that turns one Harper linter off.
/// Why:
///  Harper reads `harper-ls.linters.<Name>` booleans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linterKey(name: string): string
/// ```
fn linter_key(name: &str) -> String {
    return format!("harper-ls.linters.{name}");
}

/// What:
///  Set each named linter to `false`,
/// keeping an existing key's position.
/// Why:
///  A repeated run must not reorder the document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function disableLinters(config: Config, names: string[]): void
/// ```
pub fn disable_linters(config: &mut Map<String, Value>, names: &[String]) {
    for name in names {
        config.insert(linter_key(name), Value::Bool(false));
    }
}

/// What:
///  Union string patterns into the exclude list:
/// existing strings first,
/// then the additions,
/// without duplicates.
/// Why:
///  The user's own patterns stay;
/// a non-string member cannot be a pattern and is dropped.
///  An empty addition list leaves the document untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unionExcludes(config: Config, additions: string[]): void
/// ```
pub fn union_excludes(config: &mut Map<String, Value>, additions: &[String]) {
    if additions.is_empty() {
        return;
    }
    let mut members: Vec<String> = Vec::new();
    if let Some(Value::Array(existing)) = config.get(EXCLUDE_KEY) {
        for member in existing {
            if let Value::String(text) = member
                && !members.contains(text)
            {
                members.push(text.clone());
            }
        }
    }
    for addition in additions {
        if !members.contains(addition) {
            members.push(addition.clone());
        }
    }
    config.insert(EXCLUDE_KEY.to_string(), Value::Array(members.into_iter().map(Value::String).collect()));
}

/// What:
///  Derive a scoped server's configuration from the patched base configuration:
/// drop the exclude list,
/// then disable the scoped linters.
/// Why:
///  A server scoped to named files must not inherit the base server's file exclusions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scopedConfig(base: Config, disabled: string[]): Config
/// ```
pub fn scoped_config(base: &Map<String, Value>, disabled: &[String]) -> Map<String, Value> {
    let mut scoped = base.clone();
    scoped.shift_remove(EXCLUDE_KEY);
    disable_linters(&mut scoped, disabled);
    return scoped;
}

/// What:
///  Add the title-case default to the schema's `properties`,
/// only where the schema has no such property.
/// Why:
///  The policy disables that linter,
/// and the IDE's settings page needs the property declared to show it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withSchemaDefaults(schema: Schema): Schema
/// ```
pub fn with_schema_defaults(mut schema: Map<String, Value>) -> Map<String, Value> {
    let mut properties = match schema.get("properties") {
        Some(Value::Object(existing)) => existing.clone(),
        _ => Map::new(),
    };
    if !properties.contains_key(TITLE_CASE_PROPERTY) {
        let mut property = Map::new();
        property.insert("type".to_string(), Value::String("boolean".to_string()));
        property.insert("default".to_string(), Value::Bool(true));
        property.insert("description".to_string(), Value::String(TITLE_CASE_DESCRIPTION.to_string()));
        properties.insert(TITLE_CASE_PROPERTY.to_string(), Value::Object(property));
    }
    schema.insert("properties".to_string(), Value::Object(properties));
    return schema;
}

/// What:
///  Print an object the way the IDE stores embedded JSON:
/// two-space indentation,
/// no trailing newline.
/// Why:
///  Matches `JSON.stringify(value, null, 2)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function formatObject(object: Record<string, unknown>): string
/// ```
pub fn format_object(object: Map<String, Value>) -> String {
    let value = Value::Object(object);
    // A `Value` always serializes; the compact text is only a defensive fallback.
    let Ok(pretty) = serde_json::to_string_pretty(&value) else { return value.to_string() };
    return pretty;
}

/// Tests for the JSON helpers.
#[cfg(test)]
#[path = "harper_json_tests.rs"]
mod tests;
