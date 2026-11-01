//! What:
//!  Run the selected Markdown rules over one shared parse.
//! Why:
//!  Rule order is the incumbent registry's order,
//!  which decides which fix wins when two
//! findings' edits conflict;
//!  absent and `off` rules are not executed at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // rules.flatMap(rule => rule.check({ tree, source, mdx, lfs }))
//! ```

/// Import owned findings and every shipped Markdown check.
/// Borrow the already parsed document instead of reparsing per rule.
use crate::{
    diagnostic::Diagnostic,
    markdown_code::fenced_code_language,
    markdown_commands::commands_show_output,
    markdown_definitions::reference_definitions,
    markdown_duplicate_headings::no_duplicate_heading,
    markdown_headings::{heading_increment, no_emphasis_as_heading, single_h1},
    markdown_lfs_context::LfsImageContext,
    markdown_lfs_image_url::lfs_image_url,
    markdown_links::{link_image_style, no_bare_urls},
    markdown_punctuation::no_trailing_punctuation,
    markdown_rule_settings::MarkdownRuleSettings,
    markdown_semantic_breaks::semantic_line_breaks,
    markdown_source::MarkdownSource,
    markdown_tables::no_pipe_tables,
};

/// What:
///  Collect findings from every selected rule,
///  in fixed registry order.
/// Why:
///  `rustdoc` switches the fence-language fix to `rust`,
///  because an unlabeled fence inside
/// rustdoc is a doc test.
///  `lfs` is `None` when the file has no LFS repository or is excluded,
/// which leaves `markdown/lfs-image-url` inert.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkMarkdownRules(document, settings, rustdoc, lfs): Diagnostic[];
/// ```
pub fn check_markdown_rules(
    document: &MarkdownSource,
    settings: &MarkdownRuleSettings,
    rustdoc: bool,
    lfs: Option<&LfsImageContext>,
) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    if let Some(severity) = settings.heading_increment {
        findings.extend(heading_increment(document, severity));
    }
    if let Some(severity) = settings.commands_show_output {
        findings.extend(commands_show_output(document, severity));
    }
    if let Some(severity) = settings.no_duplicate_heading {
        findings.extend(no_duplicate_heading(document, severity));
    }
    if let Some(severity) = settings.single_h1 {
        findings.extend(single_h1(document, severity));
    }
    if let Some(severity) = settings.no_trailing_punctuation {
        findings.extend(no_trailing_punctuation(document, severity));
    }
    if let Some(severity) = settings.no_bare_urls {
        findings.extend(no_bare_urls(document, severity));
    }
    if let Some(severity) = settings.no_emphasis_as_heading {
        findings.extend(no_emphasis_as_heading(document, severity));
    }
    if let Some(severity) = settings.fenced_code_language {
        findings.extend(fenced_code_language(document, severity, rustdoc));
    }
    if let Some(severity) = settings.reference_definitions {
        findings.extend(reference_definitions(document, severity));
    }
    if let Some(severity) = settings.link_image_style {
        findings.extend(link_image_style(document, severity));
    }
    if let Some(severity) = settings.no_pipe_tables {
        findings.extend(no_pipe_tables(document, severity));
    }
    if let Some(severity) = settings.semantic_line_breaks {
        findings.extend(semantic_line_breaks(document, severity));
    }
    if let Some(setting) = &settings.lfs_image_url
        && let Some(context) = lfs
    {
        findings.extend(lfs_image_url(document, setting.severity, context));
    }
    return findings;
}

/// Selection and ordering controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_dispatch_tests.rs"]
mod tests;
