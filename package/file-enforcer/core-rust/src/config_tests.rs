//! What:
//!  Unit tests for JSONC loading and config decoding.
//! Why:
//!  Positioned syntax errors,
//! pointer-named decode errors and scope checks are the contract the CLI reports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('config', () => { it('decodes a concat rule', () => { ... }); });
//! ```

use crate::config_decode::{ConfigScope, decode_config};
use crate::config_load::{line_and_column, parse_config_text};
use crate::config_types::RuleConfig;

/// What:
///  Parse then decode a config text for the repository scope.
/// Why:
///  Shared setup for the tests below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decode(text: string) { return decodeConfig(parseConfigText(text, 'c.jsonc'), 'repository', 'c.jsonc'); }
/// ```
fn decode_text(text: &str) -> Result<crate::config_types::Config, Vec<crate::diagnostic::Diagnostic>> {
    let document = parse_config_text(text, "c.jsonc").map_err(|diagnostic| return vec![diagnostic])?;
    return decode_config(&document, ConfigScope::Repository, "c.jsonc");
}

#[test]
fn decodes_a_concat_rule_with_comments_and_trailing_commas() {
    let text = r#"{
        // comment
        "$schema": "./file-enforcer.schema.json",
        "rules": [
            { "kind": "concat", "id": "claude", "output": "CLAUDE.md",
              "parts": ["x", { "file": "AGENTS.md" }, { "concat": [{ "var": "dir" }, "/y"] }], "mode": "0600", },
        ],
    }"#;
    let config = decode_text(text).expect("config decodes");
    assert_eq!(config.rules.len(), 1);
    assert_eq!(config.rules[0].kind(), "concat");
    assert!(matches!(&config.rules[0], RuleConfig::Concat { mode: Some(mode), .. } if mode == "0600"));
}

#[test]
fn syntax_errors_name_path_line_and_column() {
    let error = parse_config_text("{\n  \"rules\": [,]\n}", "c.jsonc").expect_err("syntax error");
    assert_eq!(error.code, "config-syntax");
    assert!(error.message.starts_with("c.jsonc:2:"), "{}", error.message);
}

#[test]
fn line_and_column_counts_from_one() {
    assert_eq!(line_and_column("ab\ncd", 0), (1, 1));
    assert_eq!(line_and_column("ab\ncd", 4), (2, 2));
}

#[test]
fn duplicate_keys_are_rejected() {
    let error = parse_config_text("{\"rules\": [], \"rules\": []}", "c.jsonc").expect_err("duplicate");
    assert!(error.message.contains("appears twice"), "{}", error.message);
}

#[test]
fn every_problem_is_reported_with_its_pointer() {
    let text = r#"{"rules": [
        {"kind": "concat", "id": "a", "output": "/abs", "parts": [], "extra": 1},
        {"kind": "nope", "id": "b"},
        {"kind": "mirror", "id": "a", "from": "x", "to": "x/../y"}
    ]}"#;
    let errors = decode_text(text).expect_err("invalid");
    let joined: String = errors.iter().map(|error| return error.message.clone()).collect::<Vec<String>>().join("\n");
    assert!(joined.contains("/rules/0/extra: unknown key \"extra\""), "{joined}");
    assert!(joined.contains("/rules/0/output"), "{joined}");
    assert!(joined.contains("/rules/1/kind: unknown kind \"nope\""), "{joined}");
    assert!(joined.contains("/rules/2/id: the id \"a\" is used by an earlier rule"), "{joined}");
    assert!(joined.contains("/rules/2/to"), "{joined}");
}

#[test]
fn a_user_only_kind_is_rejected_in_the_repository_config() {
    let text = r#"{"rules": [{"kind": "jetbrains-harper", "id": "h", "productPrefixes": [], "disabledLinters": [],
        "excludePatterns": [], "scopedServers": []}]}"#;
    let errors = decode_text(text).expect_err("scope");
    assert!(errors[0].message.contains("writes outside the repository"), "{}", errors[0].message);
}

#[test]
fn a_package_text_is_decoded_beside_the_repository_wide_texts() {
    let text = r#"{"rules": [{"kind": "package-licenses", "id": "p", "manifests": [], "directory": "LICENSES",
        "texts": {"GPL-3.0-or-later": "LICENSES/GPL-3.0-or-later.txt"},
        "packageTexts": {"MIT": "LICENSES/MIT.txt"},
        "requires": [{"operand": "MIT", "texts": ["MIT"]}]}]}"#;
    let config = decode_text(text).expect("config decodes");
    let RuleConfig::PackageLicenses { texts, package_texts, .. } = &config.rules[0] else {
        panic!("the rule is a package-licenses rule");
    };
    assert_eq!(texts.len(), 1);
    assert_eq!(package_texts.as_ref().expect("packageTexts decodes").get("MIT"), Some(&"LICENSES/MIT.txt".to_string()));
}

#[test]
fn a_requirement_names_a_text_from_one_of_the_two_tables() {
    let nowhere = r#"{"rules": [{"kind": "package-licenses", "id": "p", "manifests": [], "directory": "L",
        "texts": {}, "requires": [{"operand": "MIT", "texts": ["MIT"]}]}]}"#;
    let joined = decode_errors_joined(nowhere);
    assert!(joined.contains("is not a key of texts or packageTexts"), "{joined}");
    let both = r#"{"rules": [{"kind": "package-licenses", "id": "p", "manifests": [], "directory": "L",
        "texts": {"MIT": "LICENSES/MIT.txt"}, "packageTexts": {"MIT": "LICENSES/MIT.txt"},
        "requires": [{"operand": "MIT", "texts": ["MIT"]}]}]}"#;
    let joined_both = decode_errors_joined(both);
    assert!(joined_both.contains("is a key of both texts and packageTexts"), "{joined_both}");
}

#[test]
fn two_package_texts_never_name_one_path() {
    let text = r#"{"rules": [{"kind": "package-licenses", "id": "p", "manifests": [], "directory": "L",
        "texts": {}, "packageTexts": {"MIT": "LICENSES/MIT.txt", "MIT-0": "LICENSES/MIT.txt"},
        "requires": []}]}"#;
    let joined = decode_errors_joined(text);
    assert!(joined.contains("both name"), "{joined}");
}

/// What:
///  Every decode message of a config the decoder must refuse,
///  joined by newlines.
/// Why:
///  One mistake can be reported beside another,
///  so a test matches on the message it means and not on a position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeErrorsJoined(text: string): string
/// ```
fn decode_errors_joined(text: &str) -> String {
    let errors = decode_text(text).expect_err("the config is refused");
    return errors.into_iter().map(|error| return error.message).collect::<Vec<String>>().join("\n");
}
