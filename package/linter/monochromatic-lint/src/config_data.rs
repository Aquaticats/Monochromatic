//! What:
//!  Validation shared by configuration blocks and their rule options.
//! Why:
//!  Generic JSONC permits nulls and duplicate members that the linter deliberately rejects.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Validate the data model before interpreting or merging settings.
//! ```

/// Import the linter's typed setup failure.
use crate::config_error::ConfigError;
/// What:
///  Import the actual JSONC model and string decoder.
/// Why:
///  Schema validation must preserve the parser's exact representation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { unitsToString } from 'jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{JsoncKind, JsoncValue, units_to_string};
/// What:
///  Import a set for decoded object-property identities.
/// Why:
///  Duplicate detection must compare string values,
///  not their different JSON escape spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Set<string> for each visited object.
/// ```
use std::collections::BTreeSet;

/// What:
///  Reject nulls and duplicate decoded keys throughout a configuration tree.
/// Why:
///  The merge layer only receives unambiguous settings;
///  nested options have the same requirement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function validateData(root: JsoncValue): void;
/// ```
pub(crate) fn validate_data(root: &JsoncValue) -> Result<(), ConfigError> {
    // What: Keep borrowed nodes in an explicit growable work stack, not recursive calls.
    // Why: The parser supports 512 levels, and validation must not add call-stack growth.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pending = [root];
    // ```
    let mut pending = Vec::new();
    pending.push(root);
    // What: Extract one pending node while an Option contains a value.
    // Why: Empty work is normal completion, not an error or an implicit missing node.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (pending.length !== 0) { const value = pending.pop(); }
    // ```
    while let Some(value) = pending.pop() {
        // What: Inspect only the variant of this borrowed payload.
        // Why: JSON null is valid syntax but has no meaning in the accepted linter schema.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value.kind === 'null') throw new ConfigError('...');
        // ```
        if let JsoncKind::Null = value.kind {
            // What: Return the failure variant instead of panicking on user input.
            // Why: The CLI must report a configuration error with its normal setup exit code.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw new ConfigError('Configuration values must not be null.');
            // ```
            return Err(ConfigError::new("Configuration values must not be null."));
        }
        // Borrow record members when this is an object; arrays are handled independently.
        if let Some(entries) = value.entries() {
            // A sorted set owns decoded key copies; unlike a vector it avoids repeated linear scans.
            let mut keys = BTreeSet::new();
            for entry in entries {
                // Clone decoded units, not raw spelling, so escape aliases are detected.
                if !keys.insert(entry.key.units.clone()) {
                    return Err(ConfigError::new(
                        "Configuration contains duplicate object properties after JSON escape decoding.",
                    ));
                }
                // Lend the member value to the traversal without cloning its subtree.
                pending.push(&entry.value);
            }
        }
        // Borrow array elements and add their references to the same structural traversal.
        if let Some(elements) = value.elements() {
            pending.extend(elements);
        }
    }
    // What: Return successful completion with no payload.
    // Why: Callers retain the original tree; validation creates no alternate representation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return;
    // ```
    return Ok(());
}

/// What:
///  Decode one property name into Rust's owned UTF-8 string.
/// Why:
///  Rule IDs and schema keys are text,
///  not arbitrary unpaired UTF-16 code units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keyText(units: readonly number[]): string;
/// ```
pub(crate) fn key_text(units: &[u16]) -> Result<String, ConfigError> {
    // What: Separate a successful decode from the decoder's error result.
    // Why: Unpaired surrogates get an actionable configuration failure rather than replacement text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return decodeUtf16(units); } catch (error) { throw new ConfigError(String(error)); }
    // ```
    match units_to_string(units) {
        Ok(value) => return Ok(value),
        Err(error) => {
            return Err(ConfigError::new(
                format!("Configuration text has an unpaired UTF-16 surrogate: {error}").as_str(),
            ));
        }
    }
}

/// What:
///  Require one textual field and preserve its decoded value.
/// Why:
///  Schema callers name the field,
///  so failure messages identify the affected input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function text(value: JsoncValue, field: string): string;
/// ```
pub(crate) fn text(value: &JsoncValue, field: &str) -> Result<String, ConfigError> {
    // Extract a borrowed string view without changing the parsed document.
    if let Some(units) = value.text_units() {
        return key_text(units);
    }
    return Err(ConfigError::new(
        format!("Configuration field {field} must be a string.").as_str(),
    ));
}

/// What:
///  Require an array containing only decoded strings.
/// Why:
///  File patterns and exclusions must not coerce booleans or numbers into path text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function strings(value: JsoncValue, field: string): string[];
/// ```
pub(crate) fn strings(value: &JsoncValue, field: &str) -> Result<Vec<String>, ConfigError> {
    // Extract the array view; absence is a schema error rather than an empty list.
    let Some(elements) = value.elements() else {
        return Err(ConfigError::new(
            format!("Configuration field {field} must be an array of strings.").as_str(),
        ));
    };
    // Own output strings rather than borrowing UTF-16 data from the syntax tree.
    let mut result = Vec::with_capacity(elements.len());
    for element in elements {
        // What: Propagate a typed validation failure with ? instead of returning a partial list.
        // Why: A malformed pattern invalidates the whole configuration.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // result.push(text(element, field));
        // ```
        result.push(text(element, field)?);
    }
    return Ok(result);
}
