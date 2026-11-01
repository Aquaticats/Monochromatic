//! What:
//!  Used and unique Markdown reference definitions.
//! Why:
//!  Reference identities come from the native parser,
//!  never a second case/whitespace normalization algorithm.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Collect resolved reference identifiers, then retain the first used definition of each identifier.
//! ```

/// Import the shared finding model so this rule uses the same output and fix contracts.
use crate::diagnostic::{Diagnostic, Severity};
/// Import a byte-addressed deletion rather than rebuilding surrounding Markdown.
use crate::edits::Edit;
/// Import the common diagnostic builder and original-source view.
use crate::markdown_finding::finding;
/// Import the parsed Markdown source in which link definitions are located.
use crate::markdown_source::MarkdownSource;
/// Import typed payload decoders;
///  identifiers are already normalized by the parser.
use satteri_ast::mdast::{
    DefinitionData, MdastNodeType, ReferenceData, decode_definition_data, decode_reference_data,
};
/// What:
///  BTreeSet keeps unique owned String values,
///  unlike Vec which would need repeated linear scans.
/// Why:
///  Owned String identifiers avoid carrying borrowed &str lifetimes between traversal passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Set<string> with deterministic ordering.
/// ```
use std::collections::BTreeSet;

/// What:
///  Delete a definition without joining adjacent container lines.
/// Why:
///  Whole-line removal is safe for whitespace-only prefixes;
///  quote/list prefixes must retain their line ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removalEdit(context, id): Edit;
/// ```
fn removal_edit(context: &MarkdownSource, id: u32) -> Edit {
    // Borrow the original bytes; usize addresses the complete buffer, unlike fixed-width u32/u64 or signed i32/i64.
    let bytes: &[u8] = context.source.as_bytes();
    // Copy the parser's original-source range instead of using normalized string lengths.
    let (start, mut end): (usize, usize) = context.offsets(id);
    // Find the line's beginning without interpreting container syntax as ordinary indentation.
    // Zero stands for a definition on the first line, where no earlier line ending exists.
    let mut line_start: usize = 0;
    // What: `rfind(['\n', '\r'])` is the standard reverse search for the nearest LF or CR before the
    // definition; `Some(ending)` carries that byte offset and `None` means the definition is on the first line.
    // Why: A library search has no hand-stepped index, so no mutation of a step can make this scan spin.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ending = Math.max(source.lastIndexOf('\n', start - 1), source.lastIndexOf('\r', start - 1));
    // if (ending !== -1) lineStart = ending + 1;
    // ```
    if let Some(ending) = context.source[..start].rfind(['\n', '\r']) {
        // Both line-ending characters are one byte, so the line begins directly after the match.
        line_start = ending + 1;
    }
    // Empty prefixes are safe; non-whitespace prefixes keep their physical line separate.
    let mut standalone: bool = true;
    for byte in &bytes[line_start..start] {
        if *byte != b' ' && *byte != b'\t' {
            standalone = false;
            break;
        }
    }
    if !standalone {
        // The empty owned String deletes only the definition, not its container prefix or newline.
        return Edit {
            start,
            end,
            replacement: String::new(),
        };
    }
    // Consume complete original line endings, including a CRLF pair rather than half of it.
    if end < bytes.len() && bytes[end] == b'\r' {
        end += 1;
    }
    if end < bytes.len() && bytes[end] == b'\n' {
        end += 1;
    }
    // Standalone indentation belongs to the removed line, not the following line.
    return Edit {
        start: line_start,
        end,
        replacement: String::new(),
    };
}

/// What:
///  Return findings for unused definitions and later definitions of a used identifier.
/// Why:
///  Two passes account for references preceding or following their definitions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function referenceDefinitions(context, severity): Diagnostic[];
/// ```
pub fn reference_definitions(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    // Owned growable Vec buffers collect unknown counts; &[T] borrows and [T; N] has a fixed length.
    let mut definitions: Vec<u32> = Vec::<u32>::new();
    // Separate sets distinguish resolved uses from definitions already retained.
    let mut used: BTreeSet<String> = BTreeSet::<String>::new();
    let mut kept: BTreeSet<String> = BTreeSet::<String>::new();
    // Borrow only the rule-visible traversal, excluding JSX/expression/ESM content.
    for id in context.visible_nodes() {
        // Copy the node identity for typed payload selection.
        let kind: MdastNodeType = context.kind(*id);
        if kind == MdastNodeType::LinkReference || kind == MdastNodeType::ImageReference {
            // Borrow the matching payload and copy its normalized identifier into set-owned storage.
            let data: ReferenceData = decode_reference_data(context.data(*id));
            used.insert(String::from(context.text(data.identifier)));
        } else if kind == MdastNodeType::Definition {
            // Retain source order for duplicate classification.
            definitions.push(*id);
        }
    }
    // Each result owns its message and edit independently of the parsed document.
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for id in definitions {
        // Borrow this definition's normalized identifier instead of guessing its reference spelling.
        let data: DefinitionData = decode_definition_data(context.data(id));
        let identifier: &str = context.text(data.identifier);
        if identifier == "//" {
            continue;
        }
        // Record the use decision once; a first used definition has no diagnostic.
        let is_used: bool = used.contains(identifier);
        if is_used && !kept.contains(identifier) {
            // Copy the first used identity into owned storage; later copies are duplicates.
            kept.insert(String::from(identifier));
            continue;
        }
        // A short conditional selects a borrowed static str, not a newly allocated String.
        let reason: &str = if is_used { "Duplicate" } else { "Unused" };
        // Some carries a verified edit; the shared builder groups it atomically with this finding.
        findings.push(finding(
            context,
            id,
            "markdown/link-image-reference-definitions",
            severity,
            format!("{reason} reference definition \"{identifier}\"."),
            Some(removal_edit(context, id)),
        ));
    }
    // Transfer the owned result list to the caller without borrowing the parser.
    return findings;
}

/// Compile source-preservation fixtures only for verification.
#[cfg(test)]
#[path = "markdown_definitions_tests.rs"]
mod tests;
