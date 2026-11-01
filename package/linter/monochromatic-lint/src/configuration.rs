//! What:
//!  Parsing of the linter's ordered,
//!  data-only JSONC configuration.
//! Why:
//!  Syntax,
//!  schema and rule-option errors must be rejected before file matching or merging.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseConfiguration(source) returns validated ordered blocks.
//! ```

/// Import shared validation and the built-in rule schema.
use crate::config_data::{key_text, strings, text, validate_data};
/// Import the configuration error type that parsing and validation report failures with.
use crate::config_error::ConfigError;
/// Import the rule validator that runs after the block structure has been parsed.
use crate::configuration_rules::validate_rules;
/// Import the repository's parser and exact value representation.
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};

/// What:
///  One immutable configuration block in source order.
/// Why:
///  File matching and rule merging need the same validated settings without rereading source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigBlock = { name?: string; files: string[]; ignores: string[];
///   rules: JsoncValue; globalIgnore: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigBlock {
    /// Optional author-facing name,
    ///  not a policy identity.
    pub name: Option<String>,
    /// Patterns deciding where this block applies.
    pub files: Vec<String>,
    /// Exclusions relative to the configuration's base directory.
    pub ignores: Vec<String>,
    /// Validated partial settings merged only after file matching.
    pub rules: JsoncValue,
    /// An ignores-only block excludes files from linting entirely.
    pub global_ignore: bool,
}

/// What:
///  Interpret one object after whole-document ambiguity validation.
/// Why:
///  The block has one shape regardless of how comments and trailing commas were written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseBlock(value: JsoncValue): ConfigBlock;
/// ```
fn parse_block(value: &JsoncValue) -> Result<ConfigBlock, ConfigError> {
    // What: Extract a borrowed record view or return a typed schema failure.
    // Why: Array entries cannot be silently skipped when they are malformed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!isRecord(value)) throw new ConfigError('...');
    // ```
    let Some(entries) = value.entries() else {
        return Err(ConfigError::new(
            "Each configuration block must be an object.",
        ));
    };
    // What: Start with absent optional fields and empty owned pattern vectors.
    // Why: Presence of files/rules, not their length, distinguishes global ignores.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let name; const files = []; const ignores = []; let rules = {};
    // ```
    let mut name = None;
    let mut files = Vec::new();
    let mut ignores = Vec::new();
    let mut rules = JsoncValue::record(Vec::new());
    let mut has_files = false;
    let mut has_ignores = false;
    let mut has_rules = false;
    for entry in entries {
        // Borrow the decoded key and propagate invalid textual schema keys.
        let key = key_text(&entry.key.units)?;
        if key == "name" {
            // Some records actual presence, unlike an empty or omitted name.
            name = Some(text(&entry.value, "name")?);
        } else if key == "files" {
            files = strings(&entry.value, "files")?;
            has_files = true;
        } else if key == "ignores" {
            ignores = strings(&entry.value, "ignores")?;
            has_ignores = true;
        } else if key == "rules" {
            validate_rules(&entry.value)?;
            // Own a copy so blocks can outlive the parser's outer document.
            rules = entry.value.clone();
            has_rules = true;
        } else {
            return Err(ConfigError::new(
                format!("Unknown configuration block field {key}.").as_str(),
            ));
        }
    }
    let global_ignore = has_ignores && !has_files && !has_rules;
    if !global_ignore && !has_files {
        return Err(ConfigError::new(
            "Configuration blocks need files unless they contain only ignores and an optional name.",
        ));
    }
    return Ok(ConfigBlock {
        name,
        files,
        ignores,
        rules,
        global_ignore,
    });
}

/// What:
///  Parse and validate an ordered array of configuration blocks.
/// Why:
///  Duplicate keys and nulls must be rejected before any settings reach the merge function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConfiguration(source: string): ConfigBlock[];
/// ```
pub fn parse_configuration(source: &str) -> Result<Vec<ConfigBlock>, ConfigError> {
    // What: Narrow the parser's success or failure result without discarding its byte offset.
    // Why: A syntax failure must identify its original source position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const document = parseJsonc(source); // syntax errors retain offsets
    // ```
    let document = match parse_jsonc(source) {
        Ok(document) => document,
        Err(error) => {
            return Err(ConfigError::new(
                format!(
                    "JSONC syntax error at byte {}: {}",
                    error.offset, error.message
                )
                .as_str(),
            ));
        }
    };
    // Validate every nested object before interpreting a block.
    validate_data(&document)?;
    let Some(elements) = document.elements() else {
        return Err(ConfigError::new(
            "Linter configuration must be an ordered array of blocks.",
        ));
    };
    let mut result = Vec::with_capacity(elements.len());
    for element in elements {
        result.push(parse_block(element)?);
    }
    return Ok(result);
}

/// Compile schema regressions only into the verification build.
#[cfg(test)]
#[path = "configuration_tests.rs"]
mod tests;
