//! What:
//!  Typed execution settings for the shipped Markdown rules.
//! Why:
//!  File matching yields validated JSONC;
//!  rule dispatch should read a typed selection once
//! instead of reinterpreting option objects for every rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MarkdownRuleSettings = { headingIncrement?: Severity; /* ... */ lfsImageUrl?: { severity: Severity; exclude: PathPatterns } };
//! ```

/// Import the existing schema readers and validation;
///  no second configuration grammar is added.
/// Import the diagnostic severity,
///  which intentionally has no Off variant.
/// Import the gitignore-syntax matcher the `exclude` option compiles to.
use crate::{
    config_data::{
        key_text,
        strings,
        text,
        validate_data,
    },
    config_error::ConfigError,
    configuration_rules::{
        image_byte_bound,
        validate_rules,
    },
    diagnostic::Severity,
    markdown_lfs_patterns::PathPatterns,
};
/// Import the exact JSONC representation.
use monochromatic_jsonc_edit::JsoncValue;

/// What:
///  The most bytes of one LFS-tracked image the rule reads to compute its object id
///  when configuration names no `max-image-bytes`:
///  sixteen mebibytes.
/// Why:
///  An object id is the SHA-256 of the whole file,
///  so the file must be read,
///  and a bound keeps a mistyped destination or a file that grew far past its kind
///  from filling memory in the middle of a lint run.
///  The bound is the rule's `max-image-bytes` option,
/// defaulted here,
///  because a repository with a larger image must raise it without editing this crate,
///  which ships as a published binary.
///  This repository's largest tracked image is 3,380,183 bytes
///  (a screenshot under `package/ide/app/design/screenshots`,
///  measured 2026-11-25),
///  and the next largest are between 2.1 and 2.8 mebibytes,
///  so sixteen mebibytes leaves room for about five times that.
///  The siblings a reader might expect are an unbounded read and a streamed hash;
///  a bound on the bytes read is the one that also ends on a file that keeps growing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_MAX_IMAGE_BYTES = 16 * 1024 * 1024;
/// ```
pub const DEFAULT_MAX_IMAGE_BYTES: u64 = 16 * 1024 * 1024;

/// What:
///  The selected `markdown/lfs-image-url` rule with its compiled `exclude` option.
/// Why:
///  Patterns are compiled once per resolved configuration,
///  and an invalid pattern is a setup
/// error before any file is linted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LfsSetting = { severity: Severity; exclude: PathPatterns; maxImageBytes: number };
/// ```
#[derive(Clone, Debug)]
pub struct LfsSetting {
    /// Severity selected by matching configuration blocks.
    pub severity: Severity,
    /// Gitignore-syntax patterns,
    ///  relative to the repository root,
    ///  of files the rule leaves alone.
    pub exclude: PathPatterns,
    /// The most bytes of one tracked image the rule reads to compute its object id,
    ///  from the `max-image-bytes` option and `DEFAULT_MAX_IMAGE_BYTES` when it is absent.
    pub max_image_bytes: u64,
}

/// What:
///  Immutable Markdown rule selection;
///  absence and explicit `off` both leave a rule disabled.
/// Why:
///  `Option<Severity>` makes "not selected" unrepresentable as a severity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MarkdownRuleSettings = Partial<Record<MarkdownRuleName, Severity>> & { lfsImageUrl?: LfsSetting };
/// ```
#[derive(Clone, Debug, Default)]
pub struct MarkdownRuleSettings {
    /// `markdown/heading-increment`.
    pub heading_increment: Option<Severity>,
    /// `markdown/commands-show-output`.
    pub commands_show_output: Option<Severity>,
    /// `markdown/no-duplicate-heading`.
    pub no_duplicate_heading: Option<Severity>,
    /// `markdown/single-h1`.
    pub single_h1: Option<Severity>,
    /// `markdown/no-trailing-punctuation`.
    pub no_trailing_punctuation: Option<Severity>,
    /// `markdown/no-bare-urls`.
    pub no_bare_urls: Option<Severity>,
    /// `markdown/no-emphasis-as-heading`.
    pub no_emphasis_as_heading: Option<Severity>,
    /// `markdown/fenced-code-language`.
    pub fenced_code_language: Option<Severity>,
    /// `markdown/link-image-reference-definitions`.
    pub reference_definitions: Option<Severity>,
    /// `markdown/link-image-style`.
    pub link_image_style: Option<Severity>,
    /// `markdown/no-pipe-tables`.
    pub no_pipe_tables: Option<Severity>,
    /// `markdown/semantic-line-breaks`.
    pub semantic_line_breaks: Option<Severity>,
    /// `markdown/lfs-image-url`,
    ///  with its compiled exclusions.
    pub lfs_image_url: Option<LfsSetting>,
}

/// What:
///  Read a rule's severity,
///  its `exclude` list and its `max-image-bytes` bound after ordinary schema validation.
/// Why:
///  The outer `Option` distinguishes a missing severity (a configuration error) from `off`,
///  and the bound stays absent here so `markdown_rule_settings` decides its default once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function options(setting: JsoncValue): { severity?: Severity; exclude: string[]; maxImageBytes?: number };
/// ```
fn options(
    setting: &JsoncValue
) -> Result<(Option<Severity>, Vec<String>, Option<u64>), ConfigError> {
    let mut selected: Option<Option<Severity>> = None;
    let mut exclude: Vec<String> = Vec::<String>::new();
    let mut max_image_bytes: Option<u64> = None;
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
        } else if key == "exclude" {
            exclude = strings(&entry.value, "exclude")?;
        } else if key == "max-image-bytes" {
            // The byte-bound reader counts the option's own unit,
            // and refuses a value the reader's one-byte probe past the bound cannot hold.
            max_image_bytes = Some(image_byte_bound(&entry.value)?);
        }
    }
    let Some(severity): Option<Option<Severity>> = selected else {
        return Err(ConfigError::new(
            "A selected rule requires severity off, warn, or error before execution.",
        ));
    };
    return Ok((severity, exclude, max_image_bytes));
}

/// What:
///  Convert merged rules into a typed Markdown selection.
/// Why:
///  Rules of other languages are skipped,
///  unselected rules stay disabled,
///  and a Markdown rule
/// without an execution adapter is an error instead of a silently unchecked rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markdownRuleSettings(rules: JsoncValue): MarkdownRuleSettings;
/// ```
pub fn markdown_rule_settings(rules: &JsoncValue) -> Result<MarkdownRuleSettings, ConfigError> {
    validate_data(rules)?;
    validate_rules(rules)?;
    let mut result: MarkdownRuleSettings = MarkdownRuleSettings::default();
    for entry in rules.entries().expect("rule record validated") {
        let id: String = key_text(&entry.key.units)?;
        if !id.starts_with("markdown/") {
            continue;
        }
        let (severity, exclude, max_image_bytes): (Option<Severity>, Vec<String>, Option<u64>) =
            options(&entry.value)?;
        match id.as_str() {
            "markdown/heading-increment" => result.heading_increment = severity,
            "markdown/commands-show-output" => result.commands_show_output = severity,
            "markdown/no-duplicate-heading" => result.no_duplicate_heading = severity,
            "markdown/single-h1" => result.single_h1 = severity,
            "markdown/no-trailing-punctuation" => result.no_trailing_punctuation = severity,
            "markdown/no-bare-urls" => result.no_bare_urls = severity,
            "markdown/no-emphasis-as-heading" => result.no_emphasis_as_heading = severity,
            "markdown/fenced-code-language" => result.fenced_code_language = severity,
            "markdown/link-image-reference-definitions" => result.reference_definitions = severity,
            "markdown/link-image-style" => result.link_image_style = severity,
            "markdown/no-pipe-tables" => result.no_pipe_tables = severity,
            "markdown/semantic-line-breaks" => result.semantic_line_breaks = severity,
            "markdown/lfs-image-url" => {
                // Patterns are compiled even when the rule is off, so a disabled rule cannot hide an invalid option.
                let compiled: PathPatterns = match PathPatterns::new(exclude.as_slice()) {
                    Ok(patterns) => patterns,
                    Err(error) => {
                        return Err(ConfigError::new(
                            format!("Rule markdown/lfs-image-url exclude: {error}").as_str(),
                        ));
                    }
                };
                if let Some(value) = severity {
                    // What: the bound the rule reads one tracked image under, defaulted when configuration names none.
                    // Why: a repository with an image over the default raises it in its own configuration.
                    let bound: u64 = match max_image_bytes {
                        Some(bytes) => bytes,
                        None => DEFAULT_MAX_IMAGE_BYTES,
                    };
                    result.lfs_image_url = Some(LfsSetting {
                        severity: value,
                        exclude: compiled,
                        max_image_bytes: bound,
                    });
                }
            }
            _ => {
                return Err(ConfigError::new(
                    format!("Markdown rule {id} has no execution adapter.").as_str(),
                ));
            }
        }
    }
    return Ok(result);
}

/// Configuration execution controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_rule_settings_tests.rs"]
mod tests;
