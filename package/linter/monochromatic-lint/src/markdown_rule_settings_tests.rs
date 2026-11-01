//! What:
//!  Validated JSONC to typed Markdown selection controls.
//! Why:
//!  Omission,
//!  explicit `off` and a selected severity have different meanings,
//!  and each
//! identifier must reach its own field.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(markdownRuleSettings.name, () => { /* per-rule mapping, off, options, errors */ });
//! ```

/// Import the conversion under test.
use super::{MarkdownRuleSettings, markdown_rule_settings};
use crate::diagnostic::Severity;
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};

/// Convert one fixture through the production path.
fn settings(source: &str) -> MarkdownRuleSettings {
    let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
    return markdown_rule_settings(&value).expect("complete valid rule settings");
}

/// The selected severity of every rule,
///  in registry order,
///  for whole-struct comparison.
fn severities(selected: &MarkdownRuleSettings) -> [Option<Severity>; 13] {
    let mut lfs: Option<Severity> = None;
    if let Some(setting) = &selected.lfs_image_url {
        lfs = Some(setting.severity);
    }
    return [
        selected.heading_increment,
        selected.commands_show_output,
        selected.no_duplicate_heading,
        selected.single_h1,
        selected.no_trailing_punctuation,
        selected.no_bare_urls,
        selected.no_emphasis_as_heading,
        selected.fenced_code_language,
        selected.reference_definitions,
        selected.link_image_style,
        selected.no_pipe_tables,
        selected.semantic_line_breaks,
        lfs,
    ];
}

/// Rule identifiers in the same order as `severities`.
const IDS: [&str; 13] = [
    "markdown/heading-increment",
    "markdown/commands-show-output",
    "markdown/no-duplicate-heading",
    "markdown/single-h1",
    "markdown/no-trailing-punctuation",
    "markdown/no-bare-urls",
    "markdown/no-emphasis-as-heading",
    "markdown/fenced-code-language",
    "markdown/link-image-reference-definitions",
    "markdown/link-image-style",
    "markdown/no-pipe-tables",
    "markdown/semantic-line-breaks",
    "markdown/lfs-image-url",
];

/// Absent and explicitly disabled rules select nothing;
///  rules of other languages are skipped.
#[test]
fn missing_and_disabled_rules_stay_disabled() {
    assert_eq!(severities(&settings("{}")), [None; 13]);
    let mut off: String = String::from("{");
    for id in IDS {
        off.push_str(format!("\"{id}\":{{\"severity\":\"off\"}},").as_str());
    }
    off.push_str("\"rust/max-lines\":{\"severity\":\"error\",\"max\":1}}");
    assert_eq!(severities(&settings(off.as_str())), [None; 13]);
}

/// Each identifier sets exactly its own field,
///  with the configured severity.
#[test]
fn each_identifier_selects_only_its_own_rule() {
    for (index, id) in IDS.iter().enumerate() {
        let selected: MarkdownRuleSettings =
            settings(format!("{{\"{id}\":{{\"severity\":\"warn\"}}}}").as_str());
        let mut expected: [Option<Severity>; 13] = [None; 13];
        expected[index] = Some(Severity::Warn);
        assert_eq!(severities(&selected), expected, "{id}");
    }
    let error: MarkdownRuleSettings = settings(r#"{"markdown/single-h1":{"severity":"error"}}"#);
    assert_eq!(error.single_h1, Some(Severity::Error));
}

/// The LFS rule's `exclude` option compiles to a matcher;
///  without the option nothing is excluded.
#[test]
fn lfs_exclusions_are_compiled_from_the_option() {
    let selected: MarkdownRuleSettings = settings(
        r#"{"markdown/lfs-image-url":{"severity":"error","exclude":["package/ssg/","*.draft.md"]}}"#,
    );
    let setting = selected.lfs_image_url.expect("selected rule");
    assert_eq!(setting.severity, Severity::Error);
    assert!(setting.exclude.matches("package/ssg/README.md"));
    assert!(setting.exclude.matches("doc/a.draft.md"));
    assert!(!setting.exclude.matches("doc/a.md"));
    let plain: MarkdownRuleSettings = settings(r#"{"markdown/lfs-image-url":{"severity":"warn"}}"#);
    assert!(
        !plain
            .lfs_image_url
            .expect("selected rule")
            .exclude
            .matches("doc/a.md")
    );
}

/// Invalid settings are configuration errors,
///  including an invalid pattern on a disabled rule.
#[test]
fn invalid_settings_are_configuration_errors() {
    for source in [
        r#"{"markdown/single-h1":{}}"#,
        r#"{"markdown/single-h1":{"severity":"loud"}}"#,
        r#"{"markdown/single-h1":{"severity":"error","max":3}}"#,
        r#"{"markdown/unknown":{"severity":"error"}}"#,
        r#"{"markdown/lfs-image-url":{"severity":"error","exclude":"package/"}}"#,
        r#"{"markdown/lfs-image-url":{"severity":"off","exclude":["[z-a]"]}}"#,
        r#"[]"#,
    ] {
        let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
        assert!(markdown_rule_settings(&value).is_err(), "{source}");
    }
    let value: JsoncValue =
        parse_jsonc(r#"{"markdown/lfs-image-url":{"severity":"error","exclude":["[z-a]"]}}"#)
            .expect("JSONC fixture");
    let error = markdown_rule_settings(&value).expect_err("reversed range");
    assert!(
        error.message.contains("markdown/lfs-image-url exclude"),
        "{}",
        error.message
    );
    assert!(error.message.contains("[z-a]"), "{}", error.message);
}
