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
    return schema.to_value();
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
