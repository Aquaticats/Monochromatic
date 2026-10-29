//! What:
//!  Typed execution settings for the shipped `shell/` rules.
//! Why:
//!  File matching yields validated JSONC,
//!  but rule dispatch should not reinterpret loose option objects for every run body.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ShellRuleSettings = { maxLines?: Budget; constructs: [Construct, Budget][] };
//! type Budget = { severity: Severity; max: number };
//! ```

/// Import existing schema readers and validation instead of adding another configuration grammar.
use crate::config_data::{key_text, text, validate_data};
/// Import the configuration error type returned when a shell rule setting is invalid.
use crate::config_error::ConfigError;
/// Import the validators that read the count limit and the rule table from configuration.
use crate::configuration_rules::{count_limit, validate_rules};
/// Import the diagnostic severity,
///  which intentionally has no Off variant.
use crate::diagnostic::Severity;
/// Import the construct list that gives each `shell/no-*` rule its identifier.
use crate::shell_dialect::{CONSTRUCTS, Construct};
/// Import the exact JSONC representation.
use monochromatic_jsonc_edit::JsoncValue;

/// The default `max` of `shell/max-lines`:
///  a step runs one command,
///  or at most two lines.
pub const DEFAULT_MAX_LINES: usize = 2;

/// A selected shell rule's complete execution parameters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellBudget {
    /// Severity selected by matching configuration blocks.
    pub severity: Severity,
    /// Allowed code lines for `shell/max-lines`,
    ///  allowed occurrences per run body for a `shell/no-*` rule.
    pub max: usize,
}

/// Immutable rule selection;
///  absence and explicit off both leave the rule disabled.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ShellRuleSettings {
    /// The line budget of one run body.
    pub max_lines: Option<ShellBudget>,
    /// The occurrence budget of each selected construct rule.
    pub constructs: Vec<(Construct, ShellBudget)>,
}

/// What:
///  Whether any rule is selected.
/// Why:
///  A run body under no selected rule needs no scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settings.isEmpty()
/// ```
impl ShellRuleSettings {
    /// True when no `shell/` rule is selected.
    pub fn is_empty(&self) -> bool {
        return self.max_lines.is_none() && self.constructs.is_empty();
    }
}

/// Read a rule's severity (`None` for off) and its optional `max`.
fn options(
    id: &str,
    setting: &JsoncValue,
) -> Result<(Option<Severity>, Option<usize>), ConfigError> {
    let mut selected: Option<Option<Severity>> = None;
    let mut max: Option<usize> = None;
    for entry in setting
        .entries()
        .expect("rule settings validated as a record")
    {
        let key: String = key_text(&entry.key.units)?;
        if key == "severity" {
            let value: String = text(&entry.value, "severity")?;
            selected = Some(match value.as_str() {
                "off" => None,
                "warn" => Some(Severity::Warn),
                "error" => Some(Severity::Error),
                _ => {
                    return Err(ConfigError::new(
                        "Rule severity must be off, warn, or error.",
                    ));
                }
            });
        } else if key == "max" {
            max = Some(count_limit(id, &entry.value)?);
        }
    }
    let Some(severity): Option<Option<Severity>> = selected else {
        return Err(ConfigError::new(
            "A selected rule requires severity off, warn, or error before execution.",
        ));
    };
    return Ok((severity, max));
}

/// Convert merged rules without enabling unselected shell checks or ignoring misspelled settings.
pub fn shell_rule_settings(rules: &JsoncValue) -> Result<ShellRuleSettings, ConfigError> {
    validate_data(rules)?;
    validate_rules(rules)?;
    let mut result: ShellRuleSettings = ShellRuleSettings::default();
    for entry in rules.entries().expect("rule record validated") {
        let id: String = key_text(&entry.key.units)?;
        if !id.starts_with("shell/") {
            continue;
        }
        let (severity, max): (Option<Severity>, Option<usize>) = options(id.as_str(), &entry.value)?;
        let Some(selected): Option<Severity> = severity else {
            continue;
        };
        if id == "shell/max-lines" {
            result.max_lines = Some(ShellBudget {
                severity: selected,
                max: max.unwrap_or(DEFAULT_MAX_LINES),
            });
            continue;
        }
        let mut matched: bool = false;
        for construct in CONSTRUCTS {
            if construct.rule_id() == id {
                result.constructs.push((
                    construct,
                    ShellBudget {
                        severity: selected,
                        max: max.unwrap_or(0),
                    },
                ));
                matched = true;
            }
        }
        if !matched {
            return Err(ConfigError::new(
                format!("Shell rule {id} has no execution adapter.").as_str(),
            ));
        }
    }
    return Ok(result);
}

/// Keep configuration execution controls outside release code.
#[cfg(test)]
#[path = "shell_rule_settings_tests.rs"]
mod tests;
