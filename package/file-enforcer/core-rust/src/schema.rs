//! What:
//!  The JSON Schema of the config,
//! derived from the config types.
//! Why:
//!  `file-enforcer.schema.json` is committed so editors can read it;
//! a golden test keeps the committed file equal
//!  to this output,
//! and a corpus test keeps the hand-written decoder and the schema agreeing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function configSchema(): object { return zodToJsonSchema(ConfigSchema); }
//! ```

/// Imports `Config` from `crate::config_types`.
use crate::config_types::Config;
/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// What:
///  Derive the schema as a JSON value.
/// Why:
///  One function serves the golden test,
/// the corpus test and the `write-schema` example.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configSchema(): object
/// ```
pub fn config_schema() -> Value {
    let schema = schemars::schema_for!(Config);
    let mut value = schema.to_value();
    flatten_descriptions(&mut value);
    return value;
}

/// What:
///  Turn every schema `description` string into one line of prose,
/// in place.
/// `&mut Value` is a borrow this function may change (TS:
/// a parameter it mutates).
/// Why:
///  The rustdoc on the config types breaks lines at semantic boundaries,
/// and schemars copies those line breaks into `description`,
/// where editors show them as raw line breaks.
/// The rustdoc keeps its breaks and the schema gets one line of prose per description.
/// Only string values under the key `description` change,
/// so a config property that is itself named `description` (an object) is untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flattenDescriptions(node: unknown): void {
///   if (Array.isArray(node)) { node.forEach(flattenDescriptions); return; }
///   if (typeof node !== 'object' || node === null) return;
///   for (const [key, child] of Object.entries(node)) {
///     if (key === 'description' && typeof child === 'string') node[key] = oneLine(child);
///     else flattenDescriptions(child);
///   }
/// }
/// ```
pub fn flatten_descriptions(node: &mut Value) {
    match node {
        Value::Array(items) => {
            for item in items {
                flatten_descriptions(item);
            }
        }
        Value::Object(members) => {
            for (key, child) in members.iter_mut() {
                match child {
                    Value::String(text) if key == "description" => *text = one_line(text),
                    _ => flatten_descriptions(child),
                }
            }
        }
        _ => {}
    }
}

/// What:
///  Join the lines of a doc comment with single spaces:
/// trim each line,
/// drop blank ones.
/// Why:
///  A blank line in a doc comment separates paragraphs;
/// a schema description is one run of prose,
/// so paragraphs also join with a space.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function oneLine(text: string): string { return text.split(/\r?\n/).map((l) => l.trim()).filter(Boolean).join(' '); }
/// ```
pub fn one_line(text: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            lines.push(trimmed);
        }
    }
    return lines.join(" ");
}

/// What:
///  Render the schema the way the committed file stores it:
/// pretty-printed,
/// one trailing newline.
/// Why:
///  Byte equality with the committed file is the golden test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configSchemaText(): string { return JSON.stringify(configSchema(), null, 2) + '\n'; }
/// ```
pub fn config_schema_text() -> String {
    let mut text = serde_json::to_string_pretty(&config_schema()).unwrap_or_default();
    text.push('\n');
    return text;
}
