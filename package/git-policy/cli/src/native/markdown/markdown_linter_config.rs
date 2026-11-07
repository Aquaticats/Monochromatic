//! What:
//!  The linter configuration the `markdown/autofix` policy hands `monochromatic-lint`:
//!  one block that selects the policy's rules for Markdown and MDX files,
//!  as JSONC text for the linter library's in-process entry.
//! Why:
//!  The plan's contract (`doc/planning/unified-linter.md`,
//! section "Distribution and consumers")
//!  is a one-rule configuration of the policy's own,
//!  so a configuration the repository keeps for its own linting never changes what the commit-time policy does.
//!  Each rule runs at `warn`,
//!  so a finding the fixes leave is a warning finding and the run counts as usable,
//!  and only a run that could not finish fails.
//!  The document is built as a value and printed by the JSONC emitter,
//!  so no pattern text can break out of its string.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const document = oneRuleConfiguration({ rules, exclude }); // '[{"files":["**/*.md","**/*.mdx"],"rules":{...}}]'
//! ```

/// Import the selectable rules and their configuration spelling.
use crate::config_schema::{MarkdownRule, markdown_rule_name};
/// Import the JSONC value model and its emitter.
use monochromatic_jsonc_edit::{JsoncEntry, JsoncKey, JsoncValue, emit_jsonc_value};
/// The files the block selects,
///  relative to the linter's working directory,
///  the top level.
pub const MARKDOWN_FILE_PATTERNS: [&str; 2] = ["**/*.md", "**/*.mdx"];

/// What:
///  A JSON string value holding `text`.
///  `&str` borrows the text.
/// Why:
///   The emitter quotes and escapes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = (value: string) => value;
/// ```
fn text(value: &str) -> JsoncValue {
    // `.encode_utf16().collect()` turns the text into the UTF-16 units the model stores.
    return JsoncValue::text_from_units(value.encode_utf16().collect());
}

/// What:
///  A record member named `key`.
/// Why:
///   Short name for the members of the block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const member = (key: string, value: unknown) => [key, value];
/// ```
fn member(key: &str, value: JsoncValue) -> JsoncEntry {
    return JsoncEntry {
        key: JsoncKey::from_text(key),
        value,
    };
}

/// What:
///  The settings of one rule.
///  `&[String]` borrows the policy's exclude patterns.
/// Why:
///   The LFS rule takes the patterns as its own `exclude` option,
///  as the installed
///       wrapper passed them to the linter it ran (`--lfs-image-exclude`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ruleSettings = (rule, exclude) => ({ severity: 'warn', exclude });
/// ```
fn rule_settings(rule: MarkdownRule, exclude: &[String]) -> JsoncValue {
    // `match` must name every rule, so a new rule cannot be given the wrong options.
    match rule {
        MarkdownRule::LfsImageUrl => {
            let mut patterns: Vec<JsoncValue> = Vec::new();
            for pattern in exclude {
                patterns.push(text(pattern.as_str()));
            }
            return JsoncValue::record(vec![
                member("severity", text("warn")),
                member("exclude", JsoncValue::array(patterns)),
            ]);
        }
    }
}

/// What:
///  The configuration document for `rules` and `exclude`,
///  as JSONC text.
/// Why:
///   One block,
///  the policy's rules only,
///  every other rule off by omission.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function oneRuleConfiguration(rules: MarkdownRule[], exclude: string[]): string;
/// ```
pub fn one_rule_configuration(rules: &[MarkdownRule], exclude: &[String]) -> String {
    let mut files: Vec<JsoncValue> = Vec::new();
    for pattern in MARKDOWN_FILE_PATTERNS {
        files.push(text(pattern));
    }
    let mut settings: Vec<JsoncEntry> = Vec::new();
    for rule in rules {
        settings.push(member(
            format!("markdown/{}", markdown_rule_name(*rule)).as_str(),
            rule_settings(*rule, exclude),
        ));
    }
    let block: JsoncValue = JsoncValue::record(vec![
        member("files", JsoncValue::array(files)),
        member("rules", JsoncValue::record(settings)),
    ]);
    return emit_jsonc_value(&JsoncValue::array(vec![block]));
}

/// Document controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_config_tests.rs"]
mod tests;
