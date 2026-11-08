//! What:
//!  Reporting controls for pipe tables.
//! Why:
//!  A table is reported wherever it is written and is never given an edit,
//!  because every rewrite of a table changes its content.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Parse Markdown and MDX fixtures, count findings and require that none carries a fix.
//! ```

/// Import real rule execution.
use super::no_pipe_tables;
use crate::diagnostic::{Diagnostic, Severity};
use crate::markdown_source::MarkdownSource;

/// Parse a fixture in the caller's requested language mode.
fn check(source: &str, mdx: bool) -> Vec<Diagnostic> {
    let context: MarkdownSource =
        MarkdownSource::new(String::from("table.md"), String::from(source), mdx)
            .expect("table fixture");
    return no_pipe_tables(&context, Severity::Warn);
}

/// Standalone,
/// quoted,
/// indented and listed tables are each reported once and none gets an edit.
#[test]
fn every_table_is_reported_and_none_is_rewritten() {
    for mdx in [false, true] {
        for source in [
            "| Name | Age |\n| :--- | --: |\n| Bob | `a\\|b` |\n",
            "A | B\n--- | ---\n1 | 2\n",
            "> | A |\n> | - |\n> | 1 |\n",
            "  | A |\n  | - |\n  | 1 |\n",
            "- | A |\n  | - |\n  | 1 |\n",
            "| Payload |\n| - |\n| \\{danger()\\} |\n",
        ] {
            let findings: Vec<Diagnostic> = check(source, mdx);
            assert_eq!(findings.len(), 1, "{source}");
            assert!(findings[0].fix.is_none(), "{source}");
            assert_eq!(findings[0].severity, Severity::Warn);
            assert_eq!(findings[0].code, "markdown/no-pipe-tables");
        }
    }
}

/// Ordinary prose and existing HTML are not tables.
#[test]
fn prose_with_a_pipe_and_html_tables_are_not_reported() {
    for source in ["A | pipe\n", "<table><tr><td>kept</td></tr></table>\n"] {
        assert!(check(source, false).is_empty(), "{source}");
    }
}
