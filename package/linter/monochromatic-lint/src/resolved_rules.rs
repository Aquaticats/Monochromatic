//! What:
//!  Complete validated rule settings after ordered block merging.
//! Why:
//!  Partial blocks can supply options independently,
//!  but execution requires an explicit severity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Finalize merged settings without enabling any absent rule.
//! ```

/// Import schema readers and typed setup failures.
use crate::config_data::{key_text, text};
/// Import the configuration error type returned when completed rules fail validation.
use crate::config_error::ConfigError;
/// Import the same exact JSONC representation used by parsing and merging.
use monochromatic_jsonc_edit::{JsoncEntry, JsoncKey, JsoncKind, JsoncValue};

/// What:
///  Fill defaults only for rules already present in the merged record.
/// Why:
///  No compiled-in preset may turn on a rule the configuration did not select.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function completeRules(rules: JsoncValue): JsoncValue;
/// ```
pub(crate) fn complete_rules(mut rules: JsoncValue) -> Result<JsoncValue, ConfigError> {
    // What: Borrow the owned output's record payload mutably.
    // Why: Defaults change only the fresh merge result, never any source configuration.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [id, setting] of entries(mergedRules)) { fillDefaults(setting); }
    // ```
    let JsoncKind::Record { entries } = &mut rules.kind else {
        return Err(ConfigError::new("Merged rule settings must be an object."));
    };
    for entry in entries {
        let id = key_text(&entry.key.units)?;
        let JsoncKind::Record { entries: options } = &mut entry.value.kind else {
            return Err(ConfigError::new(
                format!("Merged rule {id} settings must be an object.").as_str(),
            ));
        };
        let mut severity_present = false;
        let mut max_present = false;
        let mut exclude_present = false;
        for option in options.iter() {
            let name = key_text(&option.key.units)?;
            if name == "severity" {
                let severity = text(&option.value, "severity")?;
                if severity != "off" && severity != "warn" && severity != "error" {
                    return Err(ConfigError::new(
                        format!("Rule {id} severity must be off, warn, or error.").as_str(),
                    ));
                }
                severity_present = true;
            } else if name == "max" {
                max_present = true;
            } else if name == "exclude" {
                exclude_present = true;
            }
        }
        if !severity_present {
            return Err(ConfigError::new(format!("Rule {id} needs severity off, warn, or error in a matching configuration block.").as_str()));
        }
        if id == "rust/max-lines" && !max_present {
            // The literal is a build invariant, not unvalidated external number text.
            let value =
                JsoncValue::number_from_token("300").expect("default line limit is a JSON number");
            options.push(JsoncEntry {
                key: JsoncKey::from_text("max"),
                value,
            });
        }
        if id.starts_with("shell/") && !max_present {
            // A step runs one command or at most two lines; every construct rule allows none.
            let default: &str = if id == "shell/max-lines" { "2" } else { "0" };
            let value = JsoncValue::number_from_token(default)
                .expect("default shell limit is a JSON number");
            options.push(JsoncEntry {
                key: JsoncKey::from_text("max"),
                value,
            });
        }
        if id == "markdown/lfs-image-url" && !exclude_present {
            options.push(JsoncEntry {
                key: JsoncKey::from_text("exclude"),
                value: JsoncValue::array(Vec::new()),
            });
        }
    }
    return Ok(rules);
}
