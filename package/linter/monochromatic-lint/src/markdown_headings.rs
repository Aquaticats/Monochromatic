//! What:
//!  Heading-level,
//!  single-title and emphasis-heading checks.
//! Why:
//!  Preserve the existing MD001,
//!  MD025 and MD036 behavior over the native tree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Port the existing report-only heading checks without a new rule configuration surface.
//! ```

/// Import the diagnostic and severity types that heading findings are reported with.
use crate::diagnostic::{Diagnostic, Severity};
/// Import node findings and the processing failure reported when the document's structure cannot be walked.
use crate::markdown_finding::{finding, structure_failure};
/// Import the parsed Markdown source whose heading structure is inspected.
use crate::markdown_source::MarkdownSource;
/// Import native heading data and shared finding construction.
use satteri_ast::mdast::{MdastNodeType, decode_heading_data};

/// Report heading-depth increases larger than one after the first heading.
pub fn heading_increment(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    let mut previous = 0_u8;
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Heading {
            continue;
        }
        let depth = decode_heading_data(context.data(*id)).depth;
        if previous != 0 && depth > previous + 1 {
            findings.push(finding(
                context,
                *id,
                "markdown/heading-increment",
                severity,
                format!("Heading level jumps from {previous} to {depth}; increment by one."),
                None,
            ));
        }
        previous = depth;
    }
    return findings;
}

/// Report every h1 after the first;
///  frontmatter titles are not headings.
pub fn single_h1(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    let mut count = 0;
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Heading {
            continue;
        }
        if decode_heading_data(context.data(*id)).depth != 1 {
            continue;
        }
        count += 1;
        if count > 1 {
            findings.push(finding(
                context,
                *id,
                "markdown/single-h1",
                severity,
                String::from("Multiple top-level headings; a document should have a single h1."),
                None,
            ));
        }
    }
    return findings;
}

/// What:
///  Sentence-ending punctuation accepted by the incumbent emphasis check.
/// Why:
///  Full-width punctuation and ASCII punctuation follow the same policy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const punctuation = new Set(['.', ',', ';', ':', '!', '?', '。', '，', '；', '：', '！', '？']);
/// ```
const SENTENCE_PUNCTUATION: &[char] = &[
    '.', ',', ';', ':', '!', '?', '。', '，', '；', '：', '！', '？',
];

/// What:
///  Whether one character is the sentence-ending punctuation the policy keeps.
/// Why:
///  `is_some_and` takes a plain function path,
///  so the ending test stays a named predicate instead of a capture-free closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSentencePunctuation(character: string): boolean;
/// ```
fn is_sentence_punctuation(character: char) -> bool {
    return SENTENCE_PUNCTUATION.contains(&character);
}

/// Report emphasis-only paragraphs that are not sentences or list labels.
pub fn no_emphasis_as_heading(context: &MarkdownSource, severity: Severity) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    for id in context.visible_nodes() {
        if context.kind(*id) != MdastNodeType::Paragraph {
            continue;
        }
        // `Ok(found)` answers the bounded ancestry walk; `Err` means the parent index has a cycle, so this rule stops.
        let in_list: bool = match context.has_ancestor(*id, MdastNodeType::ListItem) {
            Ok(found) => found,
            Err(error) => {
                findings.push(structure_failure(
                    context,
                    "markdown/no-emphasis-as-heading",
                    error,
                ));
                return findings;
            }
        };
        // List labels such as `- **Note**` are allowed to be emphasis-only.
        if in_list {
            continue;
        }
        let children = context.children(*id);
        if children.len() != 1 {
            continue;
        }
        let child = children[0];
        let kind = context.kind(child);
        if kind != MdastNodeType::Emphasis && kind != MdastNodeType::Strong {
            continue;
        }
        // `Ok(text)` is the emphasis text from the bounded descendant walk; `Err` means the child index has a cycle.
        let text: String = match context.text_content(child) {
            Ok(collected) => collected,
            Err(error) => {
                findings.push(structure_failure(
                    context,
                    "markdown/no-emphasis-as-heading",
                    error,
                ));
                return findings;
            }
        };
        let last = text.chars().last();
        if last.is_some_and(is_sentence_punctuation) {
            continue;
        }
        findings.push(finding(
            context,
            *id,
            "markdown/no-emphasis-as-heading",
            severity,
            String::from("Emphasis used as a heading; use a real heading instead."),
            None,
        ));
    }
    return findings;
}
