//! What:
//!  Textual heading-scope parity controls.
//! Why:
//!  Structural keys must preserve the incumbent's hierarchy and rendered-text identity without delimiter collisions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Parse actual Markdown/MDX and compare findings across sibling, parent and depth boundaries.
//! ```

/// Import the actual checker and shared native source model.
use super::no_duplicate_heading;
use crate::diagnostic::{Diagnostic, Severity};
use crate::markdown_source::MarkdownSource;

/// Check an authored source through the native parser.
fn check(source: &str, mdx: bool) -> Vec<Diagnostic> {
    let context: MarkdownSource =
        MarkdownSource::new(String::from("input.md"), String::from(source), mdx)
            .expect("fixture parses");
    return no_duplicate_heading(&context, Severity::Warn);
}

/// Repeated siblings report only later occurrences and preserve severity/position.
#[test]
fn repeated_siblings_are_reported() {
    let findings: Vec<Diagnostic> = check("# Parent\n\n## Child\n\n## Child\n\n## Child\n", false);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].labels[0].span.line, 5);
    assert_eq!(findings[1].labels[0].span.line, 7);
    assert_eq!(
        findings[0].message,
        "Duplicate heading \"Child\" among sibling headings."
    );
    assert_eq!(findings[0].severity, Severity::Warn);
    assert!(findings[0].fix.is_none());
}

/// Parent text,
///  depth and exact case distinguish scopes;
///  inline emphasis does not change rendered text.
#[test]
fn scope_and_rendered_text_match_the_existing_policy() {
    assert!(check("# One\n\n## Child\n\n# Two\n\n## Child\n", false).is_empty());
    assert!(check("# Name\n\n## Name\n\n## name\n", false).is_empty());
    assert_eq!(check("# Title\n\n# **Title**\n", false).len(), 1);
    assert_eq!(check("# ab\n\n# a**b**\n", false).len(), 1);
    // Textual parent identity, not physical parent-node identity, is the accepted incumbent scope.
    assert_eq!(
        check("# Same\n\n## Child\n\n# Same\n\n## Child\n", false).len(),
        2
    );
    assert!(check("plain\n", false).is_empty());
}

/// Quotes,
///  bracket characters and MDX-only subtrees cannot create false structural collisions.
#[test]
fn structural_keys_and_visible_traversal_are_not_string_heuristics() {
    assert!(check("# [a]\n\n## b\n\n# a\n\n## [b]\n", false).is_empty());
    assert!(
        check(
            "<Box>\n\n# Hidden\n\n# Hidden\n\n</Box>\n\n# Visible\n",
            true
        )
        .is_empty()
    );
}
