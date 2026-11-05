//! What:
//!  Validation of the options object of the `markdown/autofix` policy and of its `rules` list.
//! Why:
//!  The policy reader in `config_policies.rs` validates every policy's options and stays under the file-size limit;
//!  this policy's options carry a rule list of their own, so their validation lives in a file of its own that the
//!  reader imports and that imports nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // markdownAutofixOptions(value, path) returns MarkdownAutofixOptions or throws ConfigError naming the key.
//! ```

/// The error type that names the offending configuration key.
use crate::config_error::ConfigError;
/// The typed settings of the policy, the rule names it accepts, and the defaults a missing key keeps.
use crate::config_schema::{MarkdownAutofixOptions, MarkdownRule, PolicyConfig, markdown_rule_from_name};
/// The shared value readers.
use crate::config_values::{member_keys, member_path, strings, wrong_kind};
/// The repository's JSONC value type, which the reader walks directly.
use monochromatic_jsonc_edit::JsoncValue;

/// What:
///  Validate the `rules` list of `markdown/autofix`.
///       `Vec<MarkdownRule>` is an owned growable list (sibling `&[T]` borrows one).
/// Why:
///   Only shipped rules exist,
///  an empty list would enable a policy that does
///       nothing,
///  and a repeated rule is an ambiguous duplicate setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markdownRules(value: JsoncValue, path: string): MarkdownRule[];
/// ```
fn markdown_rules(value: &JsoncValue, path: &str) -> Result<Vec<MarkdownRule>, ConfigError> {
    let names: Vec<String> = strings(value, path)?;
    if names.is_empty() {
        return Err(ConfigError::new(
            format!(
                "Configuration key {path} must name at least one rule; set the policy's \
                 severity to \"off\" to disable it."
            )
            .as_str(),
        ));
    }
    // `Vec::<T>::with_capacity(n)` is an empty list with room for `n` items.
    let mut rules: Vec<MarkdownRule> = Vec::<MarkdownRule>::with_capacity(names.len());
    for (index, name) in names.iter().enumerate() {
        let element_path: String = format!("{path}[{index}]");
        let Some(rule) = markdown_rule_from_name(name.as_str()) else {
            return Err(ConfigError::new(
                format!(
                    "Configuration key {element_path} names an unknown Markdown rule {name:?}; \
                     the shipped rule is \"lfs-image-url\"."
                )
                .as_str(),
            ));
        };
        // `.contains(&rule)` borrows the new rule to compare it with earlier ones.
        if rules.contains(&rule) {
            return Err(ConfigError::new(
                format!(
                    "Configuration key {element_path} repeats the Markdown rule {name:?}; \
                     list each rule once."
                )
                .as_str(),
            ));
        }
        rules.push(rule);
    }
    return Ok(rules);
}

/// What:
///  Validate the options object of `markdown/autofix`.
/// Why:
///   cli-git starts its own coordinated linter;
///  a `command` array would let
///       repository data choose a program and its arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markdownAutofixOptions(value: JsoncValue, path: string): MarkdownAutofixOptions;
/// ```
pub fn markdown_autofix_options(
    value: &JsoncValue,
    path: &str,
) -> Result<MarkdownAutofixOptions, ConfigError> {
    let Some(entries) = value.entries() else {
        return Err(wrong_kind(value, path, "an options object"));
    };
    let keys: Vec<String> = member_keys(entries, path)?;
    let mut options: MarkdownAutofixOptions = PolicyConfig::defaults().markdown_autofix;
    for (index, key) in keys.iter().enumerate() {
        let key_path: String = member_path(path, key.as_str());
        let member: &JsoncValue = &entries[index].value;
        if key == "rules" {
            options.rules = markdown_rules(member, key_path.as_str())?;
        } else if key == "exclude" {
            options.exclude = strings(member, key_path.as_str())?;
        } else if key == "command" {
            return Err(ConfigError::new(
                format!(
                    "Configuration key {key_path} is retired: cli-git runs its own coordinated \
                     Markdown linter and configuration cannot select a command. Remove the key."
                )
                .as_str(),
            ));
        } else {
            return Err(ConfigError::new(
                format!("Unknown configuration key: {key_path}. Accepted options: rules, exclude.")
                    .as_str(),
            ));
        }
    }
    return Ok(options);
}
