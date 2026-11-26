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
use super::{
    DEFAULT_MAX_IMAGE_BYTES,
    MarkdownRuleSettings,
    markdown_rule_settings,
};
use crate::configuration_rules::RULE_IDS;
use crate::diagnostic::Severity;
/// Import the largest bound the tracked-image reader reads one image under.
use crate::markdown_lfs_context::MAX_IMAGE_BYTES_BOUND;
use monochromatic_jsonc_edit::{
    JsoncValue,
    parse_jsonc,
};

/// Convert one fixture through the production path.
fn settings(source: &str) -> MarkdownRuleSettings {
    let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
    return markdown_rule_settings(&value).expect("complete valid rule settings");
}

/// Rule identifiers of the Markdown rules,
///  in registry order,
///  which is the order of the fields read by `severities`.
fn ids() -> Vec<&'static str> {
    return RULE_IDS
        .iter()
        .copied()
        .filter(|id| return id.starts_with("markdown/"))
        .collect();
}

/// The selected severity of every rule,
///  in registry order,
///  for whole-struct comparison.
fn severities(selected: &MarkdownRuleSettings) -> Vec<Option<Severity>> {
    let mut lfs: Option<Severity> = None;
    if let Some(setting) = &selected.lfs_image_url {
        lfs = Some(setting.severity);
    }
    return vec![
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

/// Absent and explicitly disabled rules select nothing;
///  rules of other languages are skipped.
#[test]
fn missing_and_disabled_rules_stay_disabled() {
    let rule_ids: Vec<&str> = ids();
    let none: Vec<Option<Severity>> = vec![None; rule_ids.len()];
    assert_eq!(severities(&settings("{}")), none);
    let mut off: String = String::from("{");
    for id in &rule_ids {
        off.push_str(format!("\"{id}\":{{\"severity\":\"off\"}},").as_str());
    }
    off.push_str("\"rust/max-lines\":{\"severity\":\"error\",\"max\":1}}");
    assert_eq!(severities(&settings(off.as_str())), none);
}

/// Each identifier sets exactly its own field,
///  with the configured severity.
#[test]
fn each_identifier_selects_only_its_own_rule() {
    let rule_ids: Vec<&str> = ids();
    for (index, id) in rule_ids.iter().enumerate() {
        let selected: MarkdownRuleSettings =
            settings(format!("{{\"{id}\":{{\"severity\":\"warn\"}}}}").as_str());
        let mut expected: Vec<Option<Severity>> = vec![None; rule_ids.len()];
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

/// The LFS rule's `max-image-bytes` option is the bound it reads one tracked image under,
///  and the default applies when configuration names none.
#[test]
fn the_lfs_image_bound_is_read_from_its_option() {
    let raised: u64 = DEFAULT_MAX_IMAGE_BYTES * 2;
    let selected: MarkdownRuleSettings = settings(
        format!(
            r#"{{"markdown/lfs-image-url":{{"severity":"error","max-image-bytes":{raised}}}}}"#
        )
        .as_str(),
    );
    assert_eq!(
        selected
            .lfs_image_url
            .expect("selected rule")
            .max_image_bytes,
        raised
    );
    let plain: MarkdownRuleSettings = settings(r#"{"markdown/lfs-image-url":{"severity":"warn"}}"#);
    assert_eq!(
        plain.lfs_image_url.expect("selected rule").max_image_bytes,
        DEFAULT_MAX_IMAGE_BYTES
    );
    // A spelling that is not an exact nonnegative integer is a configuration error naming the option.
    for source in [
        r#"{"markdown/lfs-image-url":{"severity":"error","max-image-bytes":"1048576"}}"#,
        r#"{"markdown/lfs-image-url":{"severity":"error","max-image-bytes":-1}}"#,
    ] {
        let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
        let error = markdown_rule_settings(&value).expect_err("not a count");
        assert!(
            error.message.contains("max-image-bytes"),
            "{}",
            error.message
        );
    }
    // The largest bound the reader can probe past is accepted as written.
    let largest: u64 = MAX_IMAGE_BYTES_BOUND;
    let widest: MarkdownRuleSettings = settings(
        format!(
            r#"{{"markdown/lfs-image-url":{{"severity":"error","max-image-bytes":{largest}}}}}"#
        )
        .as_str(),
    );
    assert_eq!(
        widest.lfs_image_url.expect("selected rule").max_image_bytes,
        largest
    );
    // One more than that is refused when the settings are read,
    // naming the option and the largest value it takes, in its own unit of bytes.
    let unprobeable: u64 = MAX_IMAGE_BYTES_BOUND + 1;
    let value: JsoncValue = parse_jsonc(
        format!(
            r#"{{"markdown/lfs-image-url":{{"severity":"error","max-image-bytes":{unprobeable}}}}}"#
        )
        .as_str(),
    )
    .expect("JSONC fixture");
    let refused =
        markdown_rule_settings(&value).expect_err("the probe past the bound has no answer");
    assert!(
        refused.message.contains("max-image-bytes"),
        "{}",
        refused.message
    );
    assert!(
        refused.message.contains(largest.to_string().as_str()),
        "{}",
        refused.message
    );
    // A byte count is never reported in the line-count unit the shared grammar's other options use.
    assert!(
        !refused.message.contains("line index"),
        "{}",
        refused.message
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
        r#"{"markdown/lfs-image-url":{"severity":"off","exclude":["a\\"]}}"#,
        r#"[]"#,
    ] {
        let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
        assert!(markdown_rule_settings(&value).is_err(), "{source}");
    }
    let value: JsoncValue =
        parse_jsonc(r#"{"markdown/lfs-image-url":{"severity":"error","exclude":["a\\"]}}"#)
            .expect("JSONC fixture");
    let error = markdown_rule_settings(&value).expect_err("a trailing escape");
    assert!(
        error.message.contains("markdown/lfs-image-url exclude"),
        "{}",
        error.message
    );
    assert!(error.message.contains(r#""a\\""#), "{}", error.message);
}
