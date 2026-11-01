//! What:
//!  Exact original-host positions for points,
//!  ranges,
//!  container anchors and refusals.
//! Why:
//!  Position arithmetic is pinned only where every layer strips a prefix and no expected offset is zero or one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Extract, map a zero-width or ranged label, then compare with host.indexOf(authoredText).
//! ```

/// Import the consumer helpers and atomic fix models shared by every processor test module.
use super::{Edit, Fix, ProcessorLanguage, VirtualSource, finding, inputs};
/// Inspect mapped labels and severities exactly as an output engine would.
use crate::diagnostic::{Diagnostic, Severity};
/// Typed refusals carry the original-host anchor under test.
use crate::processors::ProcessorError;
/// Reach the empty-snippet and semantic-refusal anchors through real rule selection.
use crate::rust_rule_settings::RustRuleSettings;

/// Two documented lines after ordinary code,
///  so no authored position is byte zero or one.
const DOC_HOST: &str = "fn before() {}\n/// Alpha.\n/// Beta.\nfn item() {}\n";

/// A doctest whose fence,
///  comment and hidden-line layers each remove a different prefix.
const NESTED_HOST: &str = "fn before() {}\n/// ```rust\n/// //! Example.\n/// # let value: u32 = 1;\n/// value;\n/// ```\nfn item() {}\n";

/// Find authored host text or fail the control loudly instead of comparing against a sentinel.
fn at(host: &str, needle: &str) -> usize {
    return host.find(needle).expect("authored host text");
}

/// Map one virtual label and return its original-host byte offset and length.
fn mapped(input: &VirtualSource, start: usize, length: usize) -> (usize, usize) {
    let mut label: Diagnostic = finding(input, start, 0);
    // The finding helper clamps to one virtual line; multi-line labels set their length afterwards.
    label.labels[0].span.length = length;
    let host: Diagnostic = input
        .project_diagnostic(label)
        .expect("valid label")
        .expect("authored label");
    return (host.labels[0].span.offset, host.labels[0].span.length);
}

/// Find the first prepared Rust input of a host.
fn rust_input(virtuals: &[VirtualSource]) -> &VirtualSource {
    for input in virtuals {
        if input.language() == ProcessorLanguage::Rust {
            return input;
        }
    }
    panic!("host has no Rust virtual input");
}

/// Zero-width labels map to the line that owns them,
///  preferring the following line at a shared boundary.
#[test]
fn processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end() {
    let virtuals: Vec<VirtualSource> = inputs(DOC_HOST, ProcessorLanguage::Rust);
    let doc: &VirtualSource = &virtuals[0];
    assert_eq!(doc.source(), "Alpha.\nBeta.\n");
    // The first authored byte follows the stripped comment prefix, not the container anchor.
    assert_eq!(mapped(doc, 0, 0), (at(DOC_HOST, "Alpha."), 0));
    assert_eq!(mapped(doc, 3, 0), (at(DOC_HOST, "ha."), 0));
    // A point between two lines belongs to the second line's payload, after its own prefix.
    assert_eq!(mapped(doc, 7, 0), (at(DOC_HOST, "Beta."), 0));
    assert_eq!(mapped(doc, 9, 0), (at(DOC_HOST, "ta."), 0));
    // The virtual end is the byte after the final authored newline.
    assert_eq!(mapped(doc, 13, 0), (at(DOC_HOST, "fn item"), 0));
    let located: Diagnostic = doc
        .project_diagnostic(finding(doc, 7, 0))
        .expect("valid label")
        .expect("authored label");
    assert_eq!(located.filename, "host");
    assert_eq!(located.labels[0].span.line, 3);
    assert_eq!(located.labels[0].span.column, 5);
}

/// A label ending exactly at a line boundary stops before the next line's stripped prefix.
#[test]
fn processors_map_ranges_ending_at_line_boundaries_without_the_next_prefix() {
    let virtuals: Vec<VirtualSource> = inputs(DOC_HOST, ProcessorLanguage::Rust);
    let doc: &VirtualSource = &virtuals[0];
    assert_eq!(
        mapped(doc, 0, 7),
        (at(DOC_HOST, "Alpha."), "Alpha.\n".len())
    );
    // Crossing into the second line includes that line's authored prefix bytes.
    assert_eq!(
        mapped(doc, 3, 6),
        (at(DOC_HOST, "ha."), "ha.\n/// Be".len())
    );
    assert_eq!(mapped(doc, 7, 6), (at(DOC_HOST, "Beta."), "Beta.\n".len()));
}

/// An emptied block body keeps its authored payload position;
///  only a bodiless comment uses the anchor.
#[test]
fn processors_map_empty_block_doc_payloads_separately_from_container_anchors() {
    let spaced: &str = "fn before() {}\n/** */\nfn item() {}\n";
    let virtuals: Vec<VirtualSource> = inputs(spaced, ProcessorLanguage::Rust);
    assert_eq!(virtuals[0].source(), "");
    assert_eq!(mapped(&virtuals[0], 0, 0), (at(spaced, "*/"), 0));
    let bodiless: &str = "fn before() {}\n/*!*/\nfn item() {}\n";
    let empty: Vec<VirtualSource> = inputs(bodiless, ProcessorLanguage::Rust);
    assert_eq!(empty[0].source(), "");
    assert_eq!(mapped(&empty[0], 0, 0), (at(bodiless, "/*!"), 0));
}

/// Labels through fence,
///  comment,
///  hidden-line and wrapper layers add every stripped prefix once.
#[test]
fn processors_map_points_through_every_nested_layer() {
    let virtuals: Vec<VirtualSource> = inputs(NESTED_HOST, ProcessorLanguage::Rust);
    let rust: &VirtualSource = rust_input(&virtuals);
    assert_eq!(
        rust.source(),
        "//! Example.\nfn main() {\nlet value: u32 = 1;\nvalue;\n}\n"
    );
    let docs: usize = rust.source().find("//! Example").expect("inner docs");
    let hidden: usize = rust.source().find("let value").expect("hidden line");
    let visible: usize = rust.source().find("value;").expect("visible line");
    assert_eq!(mapped(rust, docs, 0), (at(NESTED_HOST, "//! Example"), 0));
    assert_eq!(mapped(rust, hidden, 0), (at(NESTED_HOST, "let value"), 0));
    assert_eq!(
        mapped(rust, hidden + 4, 5),
        (at(NESTED_HOST, "value"), "value".len())
    );
    assert_eq!(mapped(rust, visible, 0), (at(NESTED_HOST, "value;"), 0));
    // The generated closing brace has no host address.
    let generated: usize = rust.source().find("}\n").expect("generated brace");
    assert!(
        rust.project_diagnostic(finding(rust, generated, 1))
            .expect("valid label")
            .is_none()
    );
}

/// Refusals report the authored container,
///  and the rendered error names host,
///  byte and reason.
#[test]
fn processors_anchor_refusals_at_authored_containers_and_render_them() {
    let virtuals: Vec<VirtualSource> = inputs(DOC_HOST, ProcessorLanguage::Rust);
    let overlap: ProcessorError = virtuals[0]
        .project_fix(&Fix {
            edits: vec![
                Edit {
                    start: 0,
                    end: 0,
                    replacement: String::from("a"),
                },
                Edit {
                    start: 0,
                    end: 0,
                    replacement: String::from("b"),
                },
            ],
        })
        .expect_err("overlapping group");
    assert_eq!(overlap.filename, "host");
    assert_eq!(overlap.offset, at(DOC_HOST, "/// Alpha."));
    assert_eq!(
        overlap.to_string(),
        "host at byte 15: Processor fix contains overlapping edits in one atomic group."
    );
    let nested: Vec<VirtualSource> = inputs(NESTED_HOST, ProcessorLanguage::Rust);
    let rust: &VirtualSource = rust_input(&nested);
    let body: usize = rust.source().find("value;").expect("visible line");
    let closer: ProcessorError = rust
        .project_fix(&Fix {
            edits: vec![Edit {
                start: body,
                end: body,
                replacement: String::from("```\n"),
            }],
        })
        .expect_err("forged fence closer");
    // The fence layer refuses, so the anchor is the end of its authored opening marker.
    assert_eq!(closer.offset, at(NESTED_HOST, "```rust") + "```".len());
    assert_eq!(
        closer.message,
        "Projected fix changes its Markdown/Rust container or hidden-line preparation. No edits in this atomic group can be applied safely."
    );
}

/// Failures that arise in generated or empty text still point at authored host bytes.
#[test]
fn processors_anchor_processing_failures_and_empty_snippets_at_authored_bytes() {
    let host: &str = "Intro.\n\n> ```rust\n> //! Example.\n> let value: u32 = 1;\n> ```\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Markdown);
    let rust: &VirtualSource = &virtuals[0];
    let generated: usize = rust.source().find("fn main").expect("generated main");
    let mut failure: Diagnostic = finding(rust, generated, 2);
    failure.processing_failure = true;
    let kept: Diagnostic = rust
        .project_diagnostic(failure)
        .expect("failure mapping")
        .expect("failure preserved");
    // A failure in scaffolding moves to the first authored byte of the snippet.
    assert_eq!(kept.labels[0].span.offset, at(host, "//! Example"));
    assert_eq!(kept.labels[0].span.length, 0);
    assert_eq!(kept.labels[0].span.line, 4);
    assert_eq!(kept.labels[0].span.column, 3);
    let semantic: Vec<Diagnostic> = rust
        .check_rust(RustRuleSettings {
            explicit_types: Some(Severity::Warn),
            ..RustRuleSettings::default()
        })
        .expect("explicit failure record");
    assert_eq!(semantic.len(), 1);
    assert_eq!(semantic[0].labels[0].span.offset, at(host, "//! Example"));
    let empty_host: &str = "Intro.\n\n> ```rust\n> ```\n";
    let empty: Vec<VirtualSource> = inputs(empty_host, ProcessorLanguage::Markdown);
    let missing: Vec<Diagnostic> = empty[0]
        .check_rust(RustRuleSettings {
            rustdoc: Some(Severity::Error),
            ..RustRuleSettings::default()
        })
        .expect("syntax check");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].message, "Missing rustdoc on file.");
    // An empty fence has no payload, so the finding sits at the end of its opening marker.
    assert_eq!(
        missing[0].labels[0].span.offset,
        at(empty_host, "```rust") + "```".len()
    );
    assert_eq!(missing[0].labels[0].span.line, 3);
    assert_eq!(missing[0].labels[0].span.column, 6);
    let filled_host: &str = "Intro.\n\n> ```rust\n> let value: u32 = 1;\n> ```\n";
    let filled: Vec<VirtualSource> = inputs(filled_host, ProcessorLanguage::Markdown);
    let undocumented: Vec<Diagnostic> = filled[0]
        .check_rust(RustRuleSettings {
            rustdoc: Some(Severity::Error),
            ..RustRuleSettings::default()
        })
        .expect("syntax check");
    assert_eq!(undocumented.len(), 1);
    assert_eq!(undocumented[0].message, "Missing rustdoc on file.");
    // A nonempty snippet reports at its first authored byte, after the container prefix.
    assert_eq!(
        undocumented[0].labels[0].span.offset,
        at(filled_host, "let value")
    );
}

/// A label with exactly one endpoint inside a UTF-8 character is an explicit failure,
///  not an absent finding.
#[test]
fn processors_refuse_labels_with_one_endpoint_inside_a_character() {
    let virtuals: Vec<VirtualSource> = inputs(
        "fn before() {}\n/// ab😀 text.\nfn item() {}\n",
        ProcessorLanguage::Rust,
    );
    let doc: &VirtualSource = &virtuals[0];
    assert_eq!(doc.source(), "ab😀 text.\n");
    for (start, length) in [(3, 3), (2, 1), (3, 0)] {
        let mut split: Diagnostic = finding(doc, 0, 0);
        split.labels[0].span.offset = start;
        split.labels[0].span.length = length;
        let error: ProcessorError = doc.project_diagnostic(split).expect_err("split character");
        assert_eq!(
            error.message,
            "Processor diagnostic has an invalid UTF-8 byte range."
        );
        assert_eq!(error.offset, 15);
    }
    assert_eq!(mapped(doc, 2, 4), (21, 4));
}

/// The mapping seam itself returns absence for reversed,
///  split or out-of-range spans.
/// No `VirtualSource` caller produces such a span,
///  so this control calls the crate seam directly.
#[test]
fn processors_host_range_seam_refuses_malformed_spans() {
    let virtuals: Vec<VirtualSource> = inputs(
        "fn before() {}\n/// ab😀 text.\nfn item() {}\n",
        ProcessorLanguage::Rust,
    );
    let mapping: &crate::processors_model::Mapping = &virtuals[0].mapping;
    assert_eq!(
        crate::processors_spans::host_range(mapping, 1, 2),
        Some((20, 21))
    );
    assert_eq!(
        crate::processors_spans::host_range(mapping, 2, 6),
        Some((21, 25))
    );
    // Reversed, with both endpoints on character boundaries inside the text.
    assert_eq!(crate::processors_spans::host_range(mapping, 2, 1), None);
    // Only the start splits the four-byte character.
    assert_eq!(crate::processors_spans::host_range(mapping, 3, 6), None);
    // Only the end splits it.
    assert_eq!(crate::processors_spans::host_range(mapping, 2, 3), None);
    // Past the end of the virtual text.
    assert_eq!(crate::processors_spans::host_range(mapping, 2, 99), None);
}
