//! What:
//!  Selection and ordering controls for Markdown dispatch.
//! Why:
//!  A rule must run exactly when selected,
//!  in registry order,
//!  with rustdoc and LFS context
//! reaching the rules that use them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(checkMarkdownRules.name, () => { /* nothing selected, each selected, order, rustdoc, lfs */ });
//! ```

/// Import the dispatcher and its typed inputs.
use super::check_markdown_rules;
use crate::diagnostic::{Diagnostic, Severity};
use crate::markdown_lfs_context::{LfsImageContext, LfsImageTarget};
use crate::markdown_rule_settings::{MarkdownRuleSettings, markdown_rule_settings};
use crate::markdown_source::MarkdownSource;
use monochromatic_jsonc_edit::parse_jsonc;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// One document that violates every Markdown rule except the LFS rule at least once.
const DOCUMENT: &str = "# One\n\n### Skipped level.\n\n# One\n\n**Emphasis as heading**\n\n```sh\n$ ls\n```\n\n```\nunlabeled\n```\n\nSee https://example.com and [shortcut] here. Second sentence follows on one line.\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n\n![shot](asset/shot.png)\n\n[shortcut]: https://example.com/a\n[unused]: https://example.com/b\n";

/// Rule identifiers in registry order,
///  excluding the LFS rule.
const IDS: [&str; 12] = [
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
];

/// Parse the shared fixture.
fn document() -> MarkdownSource {
    return MarkdownSource::new(String::from("a.md"), String::from(DOCUMENT), false)
        .expect("fixture parses");
}

/// Build typed settings from a JSONC rules record.
fn settings(source: &str) -> MarkdownRuleSettings {
    return markdown_rule_settings(&parse_jsonc(source).expect("JSONC fixture"))
        .expect("valid settings");
}

/// The distinct codes of findings,
///  in first-appearance order.
fn distinct_codes(findings: &[Diagnostic]) -> Vec<String> {
    let mut codes: Vec<String> = Vec::<String>::new();
    for finding in findings {
        if !codes.contains(&finding.code) {
            codes.push(finding.code.clone());
        }
    }
    return codes;
}

/// An LFS context in which the fixture's image is tracked.
fn lfs() -> LfsImageContext {
    let mut targets: BTreeMap<String, LfsImageTarget> = BTreeMap::<String, LfsImageTarget>::new();
    targets.insert(
        String::from("asset/shot.png"),
        LfsImageTarget::Lfs {
            oid: "a".repeat(64),
        },
    );
    return LfsImageContext {
        file_path: PathBuf::from("/repo/a.md"),
        repo_root: PathBuf::from("/repo"),
        object_base: String::from("https://lfs.example"),
        targets,
    };
}

/// With nothing selected no rule runs,
///  even with an LFS context available.
#[test]
fn nothing_selected_reports_nothing() {
    let findings: Vec<Diagnostic> = check_markdown_rules(
        &document(),
        &MarkdownRuleSettings::default(),
        false,
        Some(&lfs()),
    );
    assert!(findings.is_empty());
}

/// Each rule alone reports only its own code,
///  with its configured severity.
#[test]
fn each_selected_rule_runs_alone() {
    let parsed: MarkdownSource = document();
    for id in IDS {
        let selected: MarkdownRuleSettings =
            settings(format!("{{\"{id}\":{{\"severity\":\"warn\"}}}}").as_str());
        let findings: Vec<Diagnostic> = check_markdown_rules(&parsed, &selected, false, None);
        assert_eq!(distinct_codes(findings.as_slice()), [id], "{id}");
        for finding in &findings {
            assert_eq!(finding.severity, Severity::Warn, "{id}");
        }
    }
}

/// All rules together report in registry order,
///  which decides fix precedence.
#[test]
fn all_selected_rules_report_in_registry_order() {
    let mut source: String = String::from("{");
    for id in IDS {
        source.push_str(format!("\"{id}\":{{\"severity\":\"error\"}},").as_str());
    }
    source.push_str("\"markdown/lfs-image-url\":{\"severity\":\"error\"}}");
    let selected: MarkdownRuleSettings = settings(source.as_str());
    let context: LfsImageContext = lfs();
    let findings: Vec<Diagnostic> =
        check_markdown_rules(&document(), &selected, false, Some(&context));
    let mut expected: Vec<&str> = Vec::<&str>::from(IDS);
    expected.push("markdown/lfs-image-url");
    assert_eq!(distinct_codes(findings.as_slice()), expected);
}

/// The LFS rule needs both its selection and a context;
///  either one alone leaves it inert.
#[test]
fn the_lfs_rule_needs_selection_and_context() {
    let selected: MarkdownRuleSettings =
        settings(r#"{"markdown/lfs-image-url":{"severity":"warn"}}"#);
    let context: LfsImageContext = lfs();
    assert!(check_markdown_rules(&document(), &selected, false, None).is_empty());
    let findings: Vec<Diagnostic> =
        check_markdown_rules(&document(), &selected, false, Some(&context));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "markdown/lfs-image-url");
    assert_eq!(findings[0].severity, Severity::Warn);
}

/// Inside rustdoc an unlabeled fence is a doc test,
///  so the fix inserts `rust` instead of `text`.
#[test]
fn rustdoc_changes_the_fence_language_fix() {
    let selected: MarkdownRuleSettings =
        settings(r#"{"markdown/fenced-code-language":{"severity":"error"}}"#);
    let parsed: MarkdownSource = document();
    let ordinary: Vec<Diagnostic> = check_markdown_rules(&parsed, &selected, false, None);
    let rustdoc: Vec<Diagnostic> = check_markdown_rules(&parsed, &selected, true, None);
    assert_eq!(ordinary.len(), 1);
    assert_eq!(rustdoc.len(), 1);
    assert_eq!(
        ordinary[0].fix.as_ref().expect("fix").edits[0].replacement,
        "text"
    );
    assert_eq!(
        rustdoc[0].fix.as_ref().expect("fix").edits[0].replacement,
        "rust"
    );
}
