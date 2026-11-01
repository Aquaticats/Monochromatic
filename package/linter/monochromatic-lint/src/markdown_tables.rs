//! What:
//!  Report pipe tables and convert only standalone,
//!  unindented tables to an HTML fallback.
//! Why:
//!  Nested container prefixes cannot be replaced safely by a whole-table edit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Preserve the incumbent's preferred headings/lists message and its source-preserving HTML fallback.
//! ```

/// Import shared diagnostic and edit contracts.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the edit and fix types that a table repair is expressed as.
use crate::edits::{Edit, Fix};
/// Import the finding constructor that wraps a table problem as a located diagnostic.
use crate::markdown_finding::finding;
/// Import the native source and final-context text encoder.
use crate::markdown_source::MarkdownSource;
/// Import the helper that reads the visible text of an HTML table cell.
use crate::markdown_table_text::html_table_cell_text;
/// Import installed table payload decoders rather than interpreting bytes manually.
use satteri_ast::mdast::{ColumnAlign, MdastNodeType, decode_table_alignments};

/// Preferred authoring forms and the explicitly secondary automatic fallback.
const DETAIL: &str = "Markdown pipe tables force each row onto one line; prefer headings or lists. `--fix` converts to an HTML table as a fallback.";

/// Strip cell delimiter pipes,
///  preserving an escaped final pipe when no outer delimiter was authored.
fn cell_content(written: &str) -> &str {
    // Borrow a narrower source view; no normalized parser text replaces inline Markdown spelling.
    let content: &str = written.strip_prefix('|').unwrap_or(written);
    // None means no final pipe was written; Some carries the text before the candidate closing delimiter.
    let Some(body): Option<&str> = content.strip_suffix('|') else {
        return content;
    };
    // Each backslash directly before the candidate flips whether that pipe is escaped content.
    // A flag, not a run length, holds the parity; walking backwards stops at the cell's first byte by itself.
    let mut escaped: bool = false;
    for byte in body.bytes().rev() {
        if byte != b'\\' {
            break;
        }
        escaped = !escaped;
    }
    if escaped {
        return content;
    }
    // The final pipe is syntax rather than escaped content.
    return body;
}

/// Append a row's cells using only fixed element/attribute names and escaped text.
fn append_row(
    context: &MarkdownSource,
    row: u32,
    header: bool,
    alignments: &[ColumnAlign],
    output: &mut String,
) {
    // A fixed boolean choice cannot interpolate an untrusted element name.
    let tag: &str = if header { "th" } else { "td" };
    output.push_str("<tr>\n");
    // Enumeration supplies the alignment slot without rescanning sibling cells.
    for (column, cell) in context.children(row).iter().enumerate() {
        // Missing alignment means no attribute, matching the incumbent's unaligned fallback.
        let alignment: ColumnAlign = alignments.get(column).copied().unwrap_or(ColumnAlign::None);
        let attribute: &str = if alignment == ColumnAlign::Left {
            " align=\"left\""
        } else if alignment == ColumnAlign::Right {
            " align=\"right\""
        } else if alignment == ColumnAlign::Center {
            " align=\"center\""
        } else {
            ""
        };
        // Borrow exact authored content and encode at the HTML/MDX interpolation boundary.
        let raw: &str = cell_content(context.slice(*cell));
        let text: String = html_table_cell_text(raw, context.mdx);
        output.push_str(format!("<{tag}{attribute}>{text}</{tag}>\n").as_str());
    }
    output.push_str("</tr>\n");
}

/// Render an HTML fallback with the first row in the header and remaining rows in the body.
fn table_html(context: &MarkdownSource, table: u32) -> String {
    // Decode the installed codec's typed alignment list; source nodes remain owned by context.
    let alignments: Vec<ColumnAlign> = decode_table_alignments(context.data(table));
    let rows: &[u32] = context.children(table);
    // String owns the accumulated output; &str could not grow with the table.
    let mut output: String = String::from("<table>\n");
    if let Some(header) = rows.first() {
        output.push_str("<thead>\n");
        append_row(context, *header, true, alignments.as_slice(), &mut output);
        output.push_str("</thead>\n");
    }
    if rows.len() > 1 {
        output.push_str("<tbody>\n");
        for row in &rows[1..] {
            append_row(context, *row, false, alignments.as_slice(), &mut output);
        }
        output.push_str("</tbody>\n");
    }
    output.push_str("</table>");
    return output;
}

/// Report all rule-visible pipe tables,
///  offering whole-table edits only at the standalone boundary.
pub fn no_pipe_tables(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    // Own findings so their lifetime does not depend on the parser's arena.
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Table {
            continue;
        }
        // Build the report independently of whether this table has a safe whole-node edit.
        let mut diagnostic: Diagnostic = finding(
            context,
            *id,
            "markdown/no-pipe-tables",
            severity,
            String::from(DETAIL),
            None,
        );
        if let Some(parent) = context.parent(*id)
            && context.kind(parent) == MdastNodeType::Root
            && context.node_span(*id).column == 1
        {
            // The shared editor applies this complete replacement atomically with other findings' fixes.
            let (start, end): (usize, usize) = context.offsets(*id);
            diagnostic.fix = Some(Fix {
                edits: vec![Edit {
                    start,
                    end,
                    replacement: table_html(context, *id),
                }],
            });
        }
        findings.push(diagnostic);
    }
    return findings;
}

/// Keep source-level conversion controls outside release artifacts.
#[cfg(test)]
#[path = "markdown_tables_tests.rs"]
mod tests;
