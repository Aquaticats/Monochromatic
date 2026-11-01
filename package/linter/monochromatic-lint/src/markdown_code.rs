//! What:
//!  Fence-language normalization over exact authored source.
//! Why:
//!  Indented code stays unchanged,
//!  and unlabeled rustdoc fences must retain Rust semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // MD040 inserts text normally, rust inside rustdoc.
//! ```

/// Import the diagnostic and severity types that code-fence checks report through.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the edit type that a code-fence repair is written as.
use crate::edits::Edit;
/// Import the finding constructor that turns a located problem into a diagnostic.
use crate::markdown_finding::finding;
/// Import the parsed Markdown source that fence positions are read from.
use crate::markdown_source::MarkdownSource;
/// Import native code data and shared finding construction.
use satteri_ast::mdast::{MdastNodeType, decode_code_data};

/// What:
///  Return a fenced opener's absolute marker end,
///  independent of its language label.
/// Why:
///  Native fenced-code spans already begin at the marker,
///  excluding indentation/container prefixes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fenceMarkerEnd(context, id): number | undefined;
/// ```
pub(crate) fn fence_marker_end(context: &MarkdownSource, id: u32) -> Option<usize> {
    // Copy the native start offset and borrow exact authored bytes, not normalized code content.
    let (start, _): (usize, usize) = context.offsets(id);
    let written: &str = context.slice(id);
    if !written.starts_with("```") && !written.starts_with("~~~") {
        // None means indented code, not a fenced opener lacking a language label.
        return None;
    }
    // Marker presence proves this byte exists; its consecutive run determines the insertion boundary.
    let marker: u8 = written.as_bytes()[0];
    let mut width: usize = 0;
    for byte in written.bytes() {
        if byte != marker {
            break;
        }
        width += 1;
    }
    // Some carries the verified absolute boundary after the complete marker run.
    return Some(start + width);
}

/// Add a language label to an unlabeled fence without changing its body or trailing opener whitespace.
pub fn fenced_code_language(
    context: &MarkdownSource,
    severity: Severity,
    rustdoc: bool,
) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Code {
            continue;
        }
        let data = decode_code_data(context.data(*id));
        if !data.lang.is_empty() {
            continue;
        }
        let Some(offset) = fence_marker_end(context, *id) else {
            continue;
        };
        let label = if rustdoc { "rust" } else { "text" };
        findings.push(finding(
            context,
            *id,
            "markdown/fenced-code-language",
            severity,
            String::from("Fenced code block has no language label."),
            Some(Edit {
                start: offset,
                end: offset,
                replacement: String::from(label),
            }),
        ));
    }
    return findings;
}
