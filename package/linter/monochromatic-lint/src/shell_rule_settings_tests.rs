//! What:
//!  Validated JSONC to typed shell rule dispatch controls.
//! Why:
//!  Omission,
//!  explicit off,
//!  selected severity and the two kinds of default `max` have different configuration meanings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('shell rule settings', () => { /* off, defaults, max, errors */ });
//! ```

/// Import the real conversion and the shared types.
use super::{ShellBudget, ShellRuleSettings, shell_rule_settings};
use crate::diagnostic::Severity;
use crate::shell_dialect::Construct;
/// Import the repository's exact JSONC parser.
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};

/// Convert one owned fixture without replacing configuration validation.
fn settings(source: &str) -> Result<ShellRuleSettings, String> {
    let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
    return shell_rule_settings(&value).map_err(|error| error.to_string());
}

/// Absent and explicitly disabled rules stay disabled.
#[test]
fn disabled_and_missing_rules_remain_disabled() {
    assert_eq!(settings("{}"), Ok(ShellRuleSettings::default()));
    assert!(settings("{}").expect("settings").is_empty());
    let off: Result<ShellRuleSettings, String> = settings(
        r#"{"shell/max-lines":{"severity":"off"},"shell/no-pipe":{"severity":"off"}}"#,
    );
    assert_eq!(off, Ok(ShellRuleSettings::default()));
}

/// The line budget defaults to two lines and each construct rule to none.
#[test]
fn defaults_are_two_lines_and_no_occurrences() {
    let found: ShellRuleSettings = settings(
        r#"{"shell/max-lines":{"severity":"error"},"shell/no-semicolon":{"severity":"warn"}}"#,
    )
    .expect("settings");
    assert_eq!(
        found.max_lines,
        Some(ShellBudget {
            severity: Severity::Error,
            max: 2
        })
    );
    assert_eq!(
        found.constructs,
        [(
            Construct::Semicolon,
            ShellBudget {
                severity: Severity::Warn,
                max: 0
            }
        )]
    );
    assert!(!found.is_empty());
}

/// An explicit `max` replaces the default.
#[test]
fn explicit_max_is_read() {
    let found: ShellRuleSettings = settings(
        r#"{"shell/max-lines":{"severity":"error","max":7},"shell/no-pipe":{"severity":"error","max":3}}"#,
    )
    .expect("settings");
    assert_eq!(found.max_lines.expect("budget").max, 7);
    assert_eq!(found.constructs[0].1.max, 3);
}

/// Every construct rule has an adapter,
///  and non-shell rules are ignored here.
#[test]
fn every_construct_rule_is_selectable() {
    let found: ShellRuleSettings = settings(
        r#"{"shell/no-pipe":{"severity":"error"},"shell/no-and-chain":{"severity":"error"},"shell/no-or-chain":{"severity":"error"},"shell/no-pipe-stderr":{"severity":"error"},"shell/no-semicolon":{"severity":"error"},"shell/no-dollar-paren":{"severity":"error"},"shell/no-backtick":{"severity":"error"},"shell/no-ampersand":{"severity":"error"},"rust/max-lines":{"severity":"error"}}"#,
    )
    .expect("settings");
    assert_eq!(found.constructs.len(), 8);
    assert_eq!(found.max_lines, None);
}

/// Misspelled rules,
///  bad numbers and unknown options are configuration errors that name the rule.
#[test]
fn invalid_settings_are_errors() {
    let unknown: String = settings(r#"{"shell/no-such":{"severity":"error"}}"#).expect_err("unknown");
    assert!(unknown.contains("shell/no-such"), "{unknown}");
    let text: String =
        settings(r#"{"shell/no-pipe":{"severity":"error","max":"1"}}"#).expect_err("string");
    assert!(text.contains("shell/no-pipe max"), "{text}");
    let negative: String =
        settings(r#"{"shell/max-lines":{"severity":"error","max":-1}}"#).expect_err("negative");
    assert!(negative.contains("shell/max-lines max"), "{negative}");
    let option: String =
        settings(r#"{"shell/no-pipe":{"severity":"error","exclude":[]}}"#).expect_err("option");
    assert!(option.contains("unknown option"), "{option}");
    let missing: String = settings(r#"{"shell/no-pipe":{"max":1}}"#).expect_err("severity");
    assert!(missing.contains("severity"), "{missing}");
}
