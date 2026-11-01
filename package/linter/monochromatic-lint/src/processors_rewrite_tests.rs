//! What:
//!  Atomic-group validation,
//!  whole-line envelopes and container re-extraction refusals.
//! Why:
//!  A refused group must name its real reason,
//!  and an accepted one must produce exact host bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Project one group, then either compare the fixed host or compare the typed refusal message.
//! ```

/// Import the consumer helpers and atomic fix models shared by every processor test module.
use super::{Edit, Fix, ProcessorLanguage, VirtualSource, fixed, inputs};
/// Typed refusals carry the explanation under test.
use crate::processors::ProcessorError;

/// Refusal text for a range that is reversed,
///  out of bounds or inside a character.
const INVALID_RANGE: &str = "Processor fix has an invalid byte range or splits a UTF-8 character.";
/// Refusal text for a boundary between the two bytes of one authored newline.
const SPLIT_CRLF: &str = "Processor fix splits an authored CRLF newline.";
/// Refusal text for a rewrite whose container no longer re-extracts to the intended bytes.
const CHANGED_CONTAINER: &str = "Projected fix changes its Markdown/Rust container or hidden-line preparation. No edits in this atomic group can be applied safely.";

/// Build a one-edit group.
fn group(start: usize, end: usize, replacement: &str) -> Fix {
    return Fix {
        edits: vec![Edit {
            start,
            end,
            replacement: String::from(replacement),
        }],
    };
}

/// Project a group that must be refused and return its typed refusal.
fn refusal(input: &VirtualSource, fix: &Fix) -> ProcessorError {
    return input.project_fix(fix).expect_err("refused group");
}

/// A group supplied in descending order is sorted by byte position before validation.
#[test]
fn processors_project_groups_supplied_in_descending_order() {
    let host: &str = "/// First.\n/// Last.\nfn item() {}\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Rust);
    assert_eq!(virtuals[0].source(), "First.\nLast.\n");
    let descending: Fix = Fix {
        edits: vec![
            Edit {
                start: 7,
                end: 8,
                replacement: String::from("l"),
            },
            Edit {
                start: 5,
                end: 6,
                replacement: String::from("!"),
            },
            Edit {
                start: 0,
                end: 1,
                replacement: String::from("f"),
            },
        ],
    };
    assert_eq!(
        fixed(host, &virtuals[0], descending),
        "/// first!\n/// last.\nfn item() {}\n"
    );
}

/// Reversed ranges and ranges with exactly one endpoint inside a character are refused before slicing.
#[test]
fn processors_refuse_reversed_and_one_sided_split_edit_ranges() {
    let virtuals: Vec<VirtualSource> = inputs(
        "fn before() {}\n/// ab😀 text.\nfn item() {}\n",
        ProcessorLanguage::Rust,
    );
    let doc: &VirtualSource = &virtuals[0];
    assert_eq!(doc.source(), "ab😀 text.\n");
    for (start, end) in [(2, 1), (7, 6), (3, 6), (2, 3), (2, 99)] {
        let error: ProcessorError = refusal(doc, &group(start, end, "x"));
        assert_eq!(error.message, INVALID_RANGE);
        assert_eq!(error.offset, 15);
    }
    assert_eq!(
        fixed(
            "fn before() {}\n/// ab😀 text.\nfn item() {}\n",
            doc,
            group(2, 6, "x")
        ),
        "fn before() {}\n/// abx text.\nfn item() {}\n"
    );
}

/// An edit between the carriage return and line feed of one newline is refused for that reason.
#[test]
fn processors_refuse_edits_between_the_bytes_of_an_authored_crlf() {
    let host: &str = "/// Alpha.\r\n/// Beta.\r\nfn item() {}\r\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Rust);
    let doc: &VirtualSource = &virtuals[0];
    assert_eq!(doc.source(), "Alpha.\r\nBeta.\r\n");
    let between: usize = doc.source().find('\n').expect("first line feed");
    // Without this refusal the comment would re-extract to the intended bytes and corrupt the newline.
    assert_eq!(
        refusal(doc, &group(between, between, "X")).message,
        SPLIT_CRLF
    );
    assert_eq!(refusal(doc, &group(0, between, "")).message, SPLIT_CRLF);
    assert_eq!(
        refusal(doc, &group(between, between + 1, "")).message,
        SPLIT_CRLF
    );
    // Boundaries beside a whole newline pair stay editable.
    assert_eq!(
        fixed(host, doc, group(between - 1, between - 1, "!")),
        "/// Alpha.!\r\n/// Beta.\r\nfn item() {}\r\n"
    );
    assert_eq!(
        fixed(host, doc, group(between + 1, between + 1, "X")),
        "/// Alpha.\r\n/// XBeta.\r\nfn item() {}\r\n"
    );
}

/// A virtual text that begins with a line feed accepts an insertion at byte zero.
#[test]
fn processors_project_insertions_before_a_leading_line_feed() {
    let host: &str = "///\n/// Alpha.\nfn item() {}\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Rust);
    assert_eq!(virtuals[0].source(), "\nAlpha.\n");
    assert_eq!(
        fixed(host, &virtuals[0], group(0, 0, "Intro.")),
        "///Intro.\n/// Alpha.\nfn item() {}\n"
    );
}

/// Whole-line deletion removes the deleted line's own prefix;
///  a partial deletion keeps it.
#[test]
fn processors_project_line_deletions_between_differently_indented_prefixes() {
    let host: &str = "/// First.\n    /// Removed.\n/// Last.\nfn item() {}\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Rust);
    let doc: &VirtualSource = &virtuals[0];
    assert_eq!(doc.source(), "First.\nRemoved.\nLast.\n");
    let line: usize = doc.source().find("Removed").expect("middle line");
    assert_eq!(
        fixed(host, doc, group(line, line + "Removed.\n".len(), "")),
        "/// First.\n/// Last.\nfn item() {}\n"
    );
    // Deleting only leading bytes of the same line leaves its indented comment marker in place.
    assert_eq!(
        fixed(host, doc, group(line, line + "Rem".len(), "")),
        "/// First.\n    /// oved.\n/// Last.\nfn item() {}\n"
    );
    // Deleting through the end of the line content keeps the prefix and the newline.
    assert_eq!(
        fixed(host, doc, group(line, line + "Removed.".len(), "")),
        "/// First.\n    /// \n/// Last.\nfn item() {}\n"
    );
}

/// Edits that reach generated main text are refused with the reason for each shape.
#[test]
fn processors_name_generated_text_in_refusals_that_reach_synthetic_main() {
    let host: &str = "```rust\n//! Example.\nlet value: u32 = 1;\n```\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Markdown);
    let rust: &VirtualSource = &virtuals[0];
    assert_eq!(
        rust.source(),
        "//! Example.\nfn main() {\nlet value: u32 = 1;\n}\n"
    );
    let docs: usize = rust.source().find("Example").expect("inner docs");
    let body: usize = rust.source().find("let value").expect("authored body");
    // Ends exactly where the first authored body line starts, so only the generated line is swallowed.
    assert_eq!(
        refusal(rust, &group(docs, body, "Docs.\n")).message,
        "Processor fix includes synthetic main text; the atomic group cannot be projected."
    );
    assert_eq!(
        refusal(rust, &group(docs, body + 3, "Docs.\n")).message,
        "Processor fix crosses synthetic main text; the atomic group cannot be projected."
    );
    assert_eq!(
        refusal(rust, &group(body - 3, body - 3, "x")).message,
        "Processor fix touches synthetic text or an empty container with no proven insertion mapping."
    );
}

/// A rewrite that removes its own container,
///  or leaves a different preparation,
///  is refused.
#[test]
fn processors_refuse_rewrites_whose_container_disappears_or_regenerates() {
    let line_host: &str = "///First.\nfn item() {}\n";
    let line_docs: Vec<VirtualSource> = inputs(line_host, ProcessorLanguage::Rust);
    assert_eq!(line_docs[0].source(), "First.\n");
    // A fourth slash turns the documentation comment into an ordinary comment.
    assert_eq!(
        refusal(&line_docs[0], &group(0, 0, "/")).message,
        CHANGED_CONTAINER
    );
    let block_host: &str = "/**First. */\nfn item() {}\n";
    let block_docs: Vec<VirtualSource> = inputs(block_host, ProcessorLanguage::Rust);
    assert_eq!(block_docs[0].source(), "First. ");
    // A slash closes the comment at once and would leave the prose as Rust code.
    assert_eq!(
        refusal(&block_docs[0], &group(0, 0, "/")).message,
        CHANGED_CONTAINER
    );
    let main_host: &str = "```rust\nfn main() {}\n```\n";
    let doctests: Vec<VirtualSource> = inputs(main_host, ProcessorLanguage::Markdown);
    assert_eq!(doctests[0].source(), "fn main() {}\n");
    // Emptying a snippet with an authored main would regenerate a synthetic one.
    assert_eq!(
        refusal(&doctests[0], &group(0, doctests[0].source().len(), "")).message,
        CHANGED_CONTAINER
    );
    // The same snippet still accepts an edit that keeps its authored main.
    assert_eq!(
        fixed(main_host, &doctests[0], group(11, 11, " ")),
        "```rust\nfn main() { }\n```\n"
    );
}
