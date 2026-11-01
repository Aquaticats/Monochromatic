//! What:
//!  Prose break offsets independent of parser-node placement.
//! Why:
//!  Existing newlines and whitespace past the text-node boundary must remain visible to the scanner.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Assert offsets on complete text and on deliberately split node/tail views.
//! ```

/// Import the real byte scanner and abbreviation classifier.
use super::{abbreviation_at, break_offsets};
/// Import the finding record and the severity every rule entry point takes.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the grouped fix record and the production applier,
///  so fixes are applied exactly as released.
use crate::edits::{Fix, apply_fixes};
/// Import the consuming rule,
///  which hands this scanner parser-placed text-node boundaries.
use crate::markdown_semantic_breaks::semantic_line_breaks;
/// Import the native parser adapter that owns exact source bytes and node offsets.
use crate::markdown_source::MarkdownSource;

/// Run the consuming rule so text-node boundaries come from the real parser,
///  not handcrafted slices.
fn check(source: &str) -> Vec<Diagnostic> {
    // Own the fixture text because the parsed source keeps its exact bytes for later offsets.
    let context: MarkdownSource =
        MarkdownSource::new(String::from("breaks.md"), String::from(source), false)
            .expect("fixture parses");
    return semantic_line_breaks(&context, Severity::Error);
}

/// All punctuation families share their word-boundary and paragraph-tail behavior.
#[test]
fn break_points_preserve_tokens_and_final_punctuation() {
    assert_eq!(break_offsets("left, right.", "", true), [5]);
    assert_eq!(break_offsets("first. next", "", true), [6]);
    assert_eq!(
        break_offsets("one; two: three? four! end.", "", true),
        [4, 9, 16, 22]
    );
    for source in [
        "3.14 here.",
        "1,000 here.",
        "Node.js here.",
        "9.0.0-rc.3 here.",
        "wait... here.",
        "e.g. here.",
        "first.\nnext.",
        "first.\r\nnext.",
    ] {
        assert!(break_offsets(source, "", true).is_empty(), "{source}");
    }
}

/// The offset passes Western closing delimiters but not a following punctuation token.
#[test]
fn closing_delimiters_are_completed_before_breaking() {
    assert_eq!(break_offsets("said \"done.\" Then", "", true), [12]);
    assert_eq!(break_offsets("asked \"why?\", then", "", true), [13]);
    assert_eq!(break_offsets("done.)] next", "", true), [7]);
    assert_eq!(break_offsets("done.” next", "", true), [8]);
}

/// Long inter-node whitespace and CRLF are not mistaken for missing newlines.
#[test]
fn source_after_the_text_node_is_not_truncated() {
    assert!(break_offsets("First.", "  \r\nNext", false).is_empty());
    assert_eq!(break_offsets("First.", " next", false), [6]);
    assert_eq!(break_offsets("First.", "", false), [6]);
    assert!(break_offsets("First.", "", true).is_empty());
    let trailing: String = format!("{}\nNext", " ".repeat(300));
    assert!(break_offsets("First.", trailing.as_str(), false).is_empty());
}

/// A break point that opens its text node has no earlier byte to inspect for an ellipsis.
#[test]
fn leading_break_points_are_not_mistaken_for_ellipses() {
    assert_eq!(break_offsets(". next word", "", true), [1]);
    assert_eq!(break_offsets("? next word", "", true), [1]);
    assert!(break_offsets(".. next word", "", true).is_empty());
}

/// Inline code places the following text node directly after itself;
///  its leading period still ends a sentence.
#[test]
fn parsed_text_nodes_may_begin_with_their_break_point() {
    let source: &str = "`code`. Next words here\n";
    let findings: Vec<Diagnostic> = check(source);
    assert_eq!(findings.len(), 1);
    // Byte 7 follows the six code bytes and the period, in original-source addressing.
    assert_eq!(findings[0].labels[0].span.offset, 7);
    let fix: Fix = findings[0].fix.clone().expect("fix");
    assert_eq!(
        apply_fixes(source, &[fix]).expect("apply").source,
        "`code`.\nNext words here\n"
    );
}

/// Source glued to a node's final break point is the next written character,
///  not a word boundary.
#[test]
fn glued_trailing_source_is_not_a_separator() {
    assert!(break_offsets("Sentence.", "`code` follows", false).is_empty());
    assert!(break_offsets("Sentence.", "x", true).is_empty());
    assert!(break_offsets("said \"done.\"", "`code`", false).is_empty());
    assert_eq!(break_offsets("Sentence.", " `code` follows", false), [9]);
    assert_eq!(break_offsets("said \"done.\"", " then", false), [12]);
}

/// The parser ends a text node where inline code begins,
///  with or without a written separator between them.
#[test]
fn parsed_inline_code_glued_to_a_sentence_gets_no_break() {
    assert!(check("Sentence.`code` follows here\n").is_empty());
    assert_eq!(check("Sentence. `code` follows here\n").len(), 1);
}

/// Case folding determines abbreviation identity without shifting byte coordinates.
#[test]
fn abbreviations_keep_original_unicode_offsets() {
    assert!(abbreviation_at("DR.", 3));
    assert!(abbreviation_at("e.g.", 4));
    assert!(!abbreviation_at("first.", 6));
    assert!(!abbreviation_at("Kst.", 6));
    assert!(break_offsets("🚀 DR. Example here.", "", true).is_empty());
    assert_eq!(break_offsets("🚀 first. next", "", true), [11]);
}
