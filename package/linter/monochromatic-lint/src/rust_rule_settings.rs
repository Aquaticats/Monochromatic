//! What:
//!  Typed execution settings for the shipped Rust rules.
//! Why:
//!  File matching yields validated JSONC,
//!  but rule dispatch should not reinterpret loose option objects repeatedly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RustRuleSettings = { maxLines?: { severity: Severity; max: number }; rustdoc?: Severity; explicit?: Severity; sidecars?: Severity };
//! ```

/// Import existing schema readers and validation instead of adding another configuration grammar.
use crate::config_data::{key_text, text, validate_data};
/// Import the configuration error type returned when a Rust rule setting is invalid.
use crate::config_error::ConfigError;
/// Import the validators that read the line limit and the rule table from configuration.
use crate::configuration_rules::{line_limit, validate_rules};
/// Import the diagnostic severity,
///  which intentionally has no Off variant.
use crate::diagnostic::Severity;
/// Import the exact JSONC representation.
use monochromatic_jsonc_edit::JsoncValue;

/// A selected line-budget rule's complete execution parameters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineBudget {
    /// Severity selected by matching configuration blocks.
    pub severity: Severity,
    /// Exact nonnegative code-line limit.
    pub max: usize,
}

/// A selected closure-lint rule's complete execution parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureLint {
    /// Severity selected by matching configuration blocks.
    pub severity: Severity,
    /// The Dylint library directory as written in configuration,
    ///  relative to the configuration file's directory.
    pub library: String,
}

/// Immutable rule selection;
///  absence and explicit off both leave the rule disabled.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RustRuleSettings {
    /// Optional code-line budget.
    pub max_lines: Option<LineBudget>,
    /// Documentation requirement,
    ///  including private declarations.
    pub rustdoc: Option<Severity>,
    /// Full semantic annotation policy,
    ///  which requires a workspace context.
    pub explicit_types: Option<Severity>,
    /// Require unit tests to live in sidecar files.
    pub tests_in_sidecars: Option<Severity>,
    /// Type-aware closure check that runs the Dylint library once per owning Cargo package.
    pub avoidable_closure: Option<ClosureLint>,
}

/// The options of one rule after merging:
///  a severity (`None` for off),
///  the line limit and the Dylint library path.
type Options = (Option<Severity>, usize, Option<String>);

/// Read a selected severity,
///  line limit and library path after ordinary schema validation.
fn options(setting: &JsoncValue) -> Result<Options, ConfigError> {
    let mut selected: Option<Option<Severity>> = None;
    let mut max: usize = 300;
    let mut library: Option<String> = None;
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
            max = line_limit(&entry.value)?;
        } else if key == "library" {
            library = Some(text(&entry.value, "library")?);
        }
    }
    let Some(severity): Option<Option<Severity>> = selected else {
        return Err(ConfigError::new(
            "A selected rule requires severity off, warn, or error before execution.",
        ));
    };
    return Ok((severity, max, library));
}

/// Convert merged rules without enabling unselected Rust checks or ignoring misspelled settings.
pub fn rust_rule_settings(rules: &JsoncValue) -> Result<RustRuleSettings, ConfigError> {
    validate_data(rules)?;
    validate_rules(rules)?;
    let mut result: RustRuleSettings = RustRuleSettings::default();
    for entry in rules.entries().expect("rule record validated") {
        let id: String = key_text(&entry.key.units)?;
        if !id.starts_with("rust/") {
            continue;
        }
        let (severity, max, library): Options = options(&entry.value)?;
        match id.as_str() {
            "rust/max-lines" => {
                if let Some(value) = severity {
                    result.max_lines = Some(LineBudget {
                        severity: value,
                        max,
                    });
                }
            }
            "rust/require-rustdoc" => result.rustdoc = severity,
            "rust/require-explicit-types" => result.explicit_types = severity,
            "rust/tests-in-sidecars" => result.tests_in_sidecars = severity,
            "rust/avoidable-closure" => {
                if let Some(value) = severity {
                    let Some(path) = library else {
                        return Err(ConfigError::new(
                            "Rule rust/avoidable-closure needs the option library, the directory of the Dylint library, in a matching configuration block.",
                        ));
                    };
                    result.avoidable_closure = Some(ClosureLint {
                        severity: value,
                        library: path,
                    });
                }
            }
            _ => {
                return Err(ConfigError::new(
                    format!("Rust rule {id} has no execution adapter.").as_str(),
                ));
            }
        }
    }
    return Ok(result);
}

/// Keep configuration execution controls outside release code.
#[cfg(test)]
#[path = "rust_rule_settings_tests.rs"]
mod tests;
