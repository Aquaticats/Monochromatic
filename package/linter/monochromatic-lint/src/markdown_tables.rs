//! What:
//!  Report pipe tables and never rewrite one.
//! Why:
//!  The repository forbids tables of either kind,
//!  and the HTML table the fixer once wrote broke backticks in cells and replaced the author's table with another;
//!  the author converts a table to headings or lists by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Report each table node with the incumbent's headings-or-lists message and offer no edit.
//! ```

/// Import shared diagnostic contracts.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the finding constructor that wraps a table problem as a located diagnostic.
use crate::markdown_finding::finding;
/// Import the native source the tables are read from.
use crate::markdown_source::MarkdownSource;
/// Import the node kind that marks a table.
use satteri_ast::mdast::MdastNodeType;

/// The preferred authoring forms;
///  no automatic form is offered because any rewrite of a table changes its content.
const DETAIL: &str = "Markdown pipe tables force each row onto one line; prefer headings or lists.";

/// What:
///  One finding per pipe table,
///  without a fix.
/// Why:
///  `--fix` changes only whitespace for this rule's findings, and a table has no whitespace-only repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noPipeTables(context, severity): Diagnostic[];
/// ```
pub fn no_pipe_tables(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    // Own findings so their lifetime does not depend on the parser's arena.
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Table {
            continue;
        }
        findings.push(finding(
            context,
            *id,
            "markdown/no-pipe-tables",
            severity,
            String::from(DETAIL),
            None,
        ));
    }
    return findings;
}

/// Table reporting controls are test-only.
#[cfg(test)]
#[path = "markdown_tables_tests.rs"]
mod tests;
