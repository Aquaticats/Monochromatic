//! What:
//!  Whole-table conversion,
//!  reporting and source-boundary controls.
//! Why:
//!  The HTML fallback preserves alignment and literal cell content without rewriting enclosing containers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Apply actual grouped edits, compare exact HTML and reparse both Markdown and MDX.
//! ```

/// Import real rule execution and parser/fix boundaries.
use super::{cell_content, no_pipe_tables};
use crate::diagnostic::{Diagnostic, Severity};
use crate::edits::apply_fixes;
use crate::markdown_source::MarkdownSource;

/// Parse a fixture in the caller's requested language mode.
fn check(source: &str, mdx: bool) -> Vec<Diagnostic> {
    let context: MarkdownSource =
        MarkdownSource::new(String::from("table.md"), String::from(source), mdx)
            .expect("table fixture");
    return no_pipe_tables(&context, Severity::Warn);
}

/// Apply the sole expected table fix and prove the result no longer contains a visible pipe table.
fn fixed(source: &str, mdx: bool) -> String {
    let findings: Vec<Diagnostic> = check(source, mdx);
    assert_eq!(findings.len(), 1);
    let fix = findings[0].fix.clone().expect("standalone table fix");
    let output: String = apply_fixes(source, &[fix]).expect("table edit").source;
    assert!(check(output.as_str(), mdx).is_empty());
    return output;
}

/// Alignment and raw Markdown markers match the incumbent's exact HTML representation.
#[test]
fn table_alignment_and_cell_spelling_survive_conversion() {
    let source: &str = "| Name | Age | Note |\n| :--- | --: | :--: |\n| Bob | 30 | a \\| b |\n| Sue | 25 | **x** |\n";
    assert_eq!(
        fixed(source, false),
        concat!(
            "<table>\n<thead>\n<tr>\n",
            "<th align=\"left\">Name</th>\n<th align=\"right\">Age</th>\n<th align=\"center\">Note</th>\n",
            "</tr>\n</thead>\n<tbody>\n<tr>\n",
            "<td align=\"left\">Bob</td>\n<td align=\"right\">30</td>\n<td align=\"center\">a | b</td>\n",
            "</tr>\n<tr>\n",
            "<td align=\"left\">Sue</td>\n<td align=\"right\">25</td>\n<td align=\"center\">**x**</td>\n",
            "</tr>\n</tbody>\n</table>\n",
        )
    );
}

/// Ordinary prose and existing HTML pass;
///  containers and indentation remain report-only.
#[test]
fn conversion_respects_container_and_written_form_boundaries() {
    for source in ["A | pipe\n", "<table><tr><td>kept</td></tr></table>\n"] {
        assert!(check(source, false).is_empty());
    }
    for source in [
        "> | A |\n> | - |\n> | 1 |\n",
        "  | A |\n  | - |\n  | 1 |\n",
        "- | A |\n  | - |\n  | 1 |\n",
    ] {
        let findings: Vec<Diagnostic> = check(source, false);
        assert_eq!(findings.len(), 1, "{source}");
        assert!(findings[0].fix.is_none());
        assert_eq!(findings[0].severity, Severity::Warn);
    }
}

/// Header-only tables need no body wrapper and may omit the outer cell delimiters.
#[test]
fn header_only_tables_and_escaped_terminal_pipes_are_preserved() {
    assert_eq!(
        fixed("A | B\n--- | ---\n", false),
        "<table>\n<thead>\n<tr>\n<th>A</th>\n<th>B</th>\n</tr>\n</thead>\n</table>\n"
    );
    assert_eq!(cell_content("|a\\|"), "a\\|");
    assert_eq!(cell_content("|a\\\\|"), "a\\\\");
    assert_eq!(cell_content(""), "");
}

/// The backslash run may reach the cell's first byte,
///  and an empty delimited cell has no run at all.
#[test]
fn escape_runs_reaching_the_cell_start_decide_the_final_pipe() {
    // One or three backslashes escape the pipe; two leave it as the closing delimiter.
    assert_eq!(cell_content("\\|"), "\\|");
    assert_eq!(cell_content("|\\|"), "\\|");
    assert_eq!(cell_content("\\\\\\|"), "\\\\\\|");
    assert_eq!(cell_content("\\\\|"), "\\\\");
    assert_eq!(cell_content("||"), "");
    assert_eq!(cell_content("|"), "");
    assert_eq!(cell_content("a|"), "a");
}

/// Rows without outer delimiters keep a final escaped pipe as cell text,
///  also when it is the whole cell.
#[test]
fn parsed_cells_ending_in_escaped_pipes_keep_their_text() {
    assert_eq!(
        fixed("A | B\n--- | ---\n\\| | x \\|\n1 |\\|\n", false),
        concat!(
            "<table>\n<thead>\n<tr>\n<th>A</th>\n<th>B</th>\n</tr>\n</thead>\n<tbody>\n",
            "<tr>\n<td>|</td>\n<td>x |</td>\n</tr>\n",
            "<tr>\n<td>1</td>\n<td>|</td>\n</tr>\n",
            "</tbody>\n</table>\n",
        )
    );
}

/// The emitted HTML must not contain literal attacker-controlled tags,
///  quotes or MDX expressions.
#[test]
fn converted_text_cannot_introduce_markup_or_mdx_expressions() {
    let output: String = fixed(
        "| Payload |\n| - |\n| \\<img src=x onerror=alert(1)> & <b>\"x\"</b> ' |\n",
        false,
    );
    assert!(!output.contains("<img src=x"));
    assert!(output.contains("&lt;img src=x onerror=alert(1)&gt;"));
    assert!(output.contains("&amp;"));
    assert!(output.contains("&quot;x&quot;"));
    assert!(output.contains("&#39;"));
    let mdx: String = fixed("| Payload |\n| - |\n| \\{danger()\\} |\n", true);
    assert!(mdx.contains("&#123;danger()"));
    assert!(!mdx.contains("{danger()}"));
}
