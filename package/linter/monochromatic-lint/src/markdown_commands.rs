//! What:
//!  Remove shell prompts from prompt-only top-level fenced examples.
//! Why:
//!  Commands without shown output should be copyable,
//!  while output-bearing and indented examples remain unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Check decoded code content, then remove only actual authored '$ ' prefixes using byte edits.
//! ```

/// Import the diagnostic and severity types that command-example checks report through.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the edit and fix types that a command-example repair is expressed as.
use crate::edits::{Edit, Fix};
/// Import the fence lookup that finds where a command block's closing marker ends.
use crate::markdown_code::fence_marker_end;
/// Import the finding constructor that wraps a command-example problem as a diagnostic.
use crate::markdown_finding::finding;
/// Import the parsed Markdown source that command blocks are read from.
use crate::markdown_source::MarkdownSource;
/// Import native code payloads and shared diagnostics/fixes.
use satteri_ast::mdast::{MdastNodeType, decode_code_data};

/// Require at least one nonblank line and reject every non-prompt content line.
fn all_prompts(value: &str) -> bool {
    let mut seen: bool = false;
    // Both decoded LF and preserved bare CR delimit command/output lines; CRLF's empty segment is ignored.
    for line in value.split(['\n', '\r']) {
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with("$ ") {
            return false;
        }
        seen = true;
    }
    return seen;
}

/// What:
///  Find authored prompt prefixes without replacing normalized code-node content.
/// Why:
///  LF,
///  CRLF and bare CR remain byte-identical;
///  opening and closing fences cannot begin with a shell prompt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function promptEdits(source, nodeStart, nodeEnd): Edit[];
/// ```
fn prompt_edits(source: &str, start: usize, end: usize) -> Vec<Edit> {
    // Borrow the authored node, fence lines included, and let the standard iterator retain every newline byte.
    let written: &str = &source[start..end];
    let mut line_start: usize = start;
    let mut edits: Vec<Edit> = Vec::<Edit>::new();
    // CRLF yields a separate LF segment with no prompt; its byte still advances the next line's offset.
    for line in written.split_inclusive(['\n', '\r']) {
        if line.starts_with("$ ") {
            edits.push(Edit {
                start: line_start,
                end: line_start + 2,
                replacement: String::new(),
            });
        }
        line_start += line.len();
    }
    return edits;
}

/// Report each prompt-only column-one fenced block and group its localized prefix removals atomically.
pub fn commands_show_output(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Code || context.node_span(*id).column != 1 {
            continue;
        }
        if fence_marker_end(context, *id).is_none() {
            continue;
        }
        let data = decode_code_data(context.data(*id));
        if !all_prompts(context.text(data.value)) {
            continue;
        }
        let (start, end): (usize, usize) = context.offsets(*id);
        // The opening line begins with its fence marker, never a prompt, so the scan needs no body offset.
        let edits: Vec<Edit> = prompt_edits(context.source.as_str(), start, end);
        let mut diagnostic: Diagnostic = finding(
            context,
            *id,
            "markdown/commands-show-output",
            severity,
            String::from("Shell prompts with no shown output; remove the `$ ` prompts."),
            None,
        );
        if !edits.is_empty() {
            diagnostic.fix = Some(Fix { edits });
        }
        findings.push(diagnostic);
    }
    return findings;
}

/// Native source-preservation regressions are not release code.
#[cfg(test)]
#[path = "markdown_commands_tests.rs"]
mod tests;
