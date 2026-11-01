//! What:
//!  Semantic line breaks at verified prose boundaries,
//! each replacing the single space it breaks at.
//! Why:
//!  Text-node tails must be interpreted through their inline delimiters and surrounding block syntax.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // For each eligible text node, replace the separating space with a source-aware break and keep every other byte.
//! ```

/// Import the established diagnostic/fix model and native source.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the edit and fix types that a semantic line-break repair is expressed as.
use crate::edits::{Edit, Fix};
/// Import byte-oriented lexical guards and AST boundary helpers.
use crate::markdown_block_start::starts_block_construct;
/// Import the planner that picks where a paragraph may break without changing its meaning.
use crate::markdown_break_points::break_offsets;
/// Import the processing-failure finding a rule reports when the document's structure cannot be walked.
use crate::markdown_finding::structure_failure;
/// Import the constructor that reports a document the break planner cannot read.
use crate::markdown_prose_context::{continuation_prefix, delimiter_tail, paragraph_for};
/// Import the prose-context helpers that locate each paragraph and its continuation prefix.
use crate::markdown_source::MarkdownSource;
/// Import the parsed Markdown source that paragraphs are read from.
use satteri_ast::mdast::MdastNodeType;

/// Whether a character is whitespace to a JavaScript `trim`,
///  which the rule this one replaces used:
///  the byte order mark counts as whitespace there and the next-line control does not.
fn is_script_whitespace(character: char) -> bool {
    return (character.is_whitespace() && character != '\u{85}') || character == '\u{feff}';
}

/// Whether a paragraph's whole source is one tag-like element,
/// such as a component written with JSX braces.
///  CommonMark does not read such a tag as HTML,
/// so it parses as prose that has nothing to break.
fn is_tag_like_paragraph(context: &MarkdownSource, paragraph: u32) -> bool {
    let text: &str = context.slice(paragraph).trim_matches(is_script_whitespace);
    return text.starts_with('<') && text.ends_with('>');
}

/// Report missing line breaks at their insertion points,
///  not at the paragraph's beginning.
pub fn semantic_line_breaks(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    // Match the incumbent's CRLF preference; existing breaks of either kind remain untouched.
    let newline: &str = if context.source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Text {
            continue;
        }
        // Walk the ancestors once, bounded; `Err` means the parent index has a cycle and nothing here can be trusted.
        let ancestors: Vec<u32> = match context.ancestors(*id) {
            Ok(chain) => chain,
            Err(error) => {
                findings.push(structure_failure(
                    context,
                    "markdown/semantic-line-breaks",
                    error,
                ));
                return findings;
            }
        };
        let Some(paragraph): Option<u32> = paragraph_for(context, ancestors.as_slice()) else {
            continue;
        };
        if is_tag_like_paragraph(context, paragraph) {
            continue;
        }
        let tail: u32 = delimiter_tail(context, *id, ancestors.as_slice());
        let (_, tail_end): (usize, usize) = context.offsets(tail);
        let (_, paragraph_end): (usize, usize) = context.offsets(paragraph);
        let (start, _): (usize, usize) = context.offsets(*id);
        let is_paragraph_tail: bool = context.children(paragraph).last() == Some(&tail);
        // Borrow the complete trailing source instead of limiting whitespace lookahead.
        let trailing: &str = &context.source[tail_end..paragraph_end];
        let written: &str = context.slice(*id);
        let prefix: String = continuation_prefix(context, paragraph);
        for relative in break_offsets(written, trailing, is_paragraph_tail) {
            // A text-tail break belongs after all actual closing delimiters, never inside them.
            let at: usize = if relative == written.len() {
                tail_end
            } else {
                start + relative
            };
            if starts_block_construct(context.source.as_str(), at) {
                continue;
            }
            // A space right after the break point is the separator the break replaces;
            //  keeping it would open every continuation line with a stray space.
            //  A break at the very end of a line has no space after it.
            let separator: usize = usize::from(context.source.as_bytes().get(at) == Some(&b' '));
            let mut diagnostic: Diagnostic = Diagnostic::new(
                "markdown/semantic-line-breaks",
                severity,
                String::from("A line break belongs here, after a prose break-point character."),
                context.filename.clone(),
                context.span(at, 0),
            );
            // The fix consumes at most the one separating space; the grouped editor resolves competing fixes.
            diagnostic.fix = Some(Fix {
                edits: vec![Edit {
                    start: at,
                    end: at + separator,
                    replacement: format!("{newline}{prefix}"),
                }],
            });
            findings.push(diagnostic);
        }
    }
    return findings;
}

/// Structural and lexical integration controls are test-only.
#[cfg(test)]
#[path = "markdown_semantic_breaks_tests.rs"]
mod tests;
