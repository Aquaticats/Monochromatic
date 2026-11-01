//! What:
//!  Exact virtual Markdown text and host edits for Rustdoc margins,
//!  blank lines and block bodies.
//! Why:
//!  Margin arithmetic is visible only where authored lines indent differently or contain no ASCII margin.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Extract each comment form, compare the whole virtual string, then apply a mapped fix to the host.
//! ```

/// Import the consumer helpers and atomic fix models shared by every processor test module.
use super::{Edit, Fix, ProcessorLanguage, VirtualSource, fixed, inputs};

/// Extract the only Rustdoc input of a host and return its exact virtual Markdown.
fn doc_text(host: &str) -> String {
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Rust);
    assert_eq!(virtuals.len(), 1, "one authored comment and no fences");
    return String::from(virtuals[0].source());
}

/// A run without a common margin keeps each line's own indentation after the conventional space.
#[test]
fn processors_keep_relative_indentation_in_line_doc_runs_without_a_margin() {
    assert_eq!(
        doc_text("/// Alpha.\n///   indented.\n///\n/// Beta.\nfn item() {}\n"),
        "Alpha.\n  indented.\n\nBeta.\n"
    );
}

/// A common margin is removed once from every line;
///  deeper lines keep only their extra indentation.
#[test]
fn processors_strip_only_the_common_margin_from_line_doc_runs() {
    let host: &str = "///   Alpha.\n///     indented.\n///\n///   Beta.\nfn item() {}\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Rust);
    assert_eq!(virtuals[0].source(), "Alpha.\n  indented.\n\nBeta.\n");
    let split: usize = virtuals[0].source().find("ted.").expect("indented prose");
    // An inserted line repeats the comment marker, conventional space and common margin.
    assert_eq!(
        fixed(
            host,
            &virtuals[0],
            Fix {
                edits: vec![Edit {
                    start: split,
                    end: split,
                    replacement: String::from("\n"),
                }]
            }
        ),
        "///   Alpha.\n///     inden\n///   ted.\n///\n///   Beta.\nfn item() {}\n"
    );
    let word: usize = virtuals[0].source().find("Beta").expect("final prose");
    assert_eq!(
        fixed(
            host,
            &virtuals[0],
            Fix {
                edits: vec![Edit {
                    start: word,
                    end: word + "Beta".len(),
                    replacement: String::from("Gamma"),
                }]
            }
        ),
        "///   Alpha.\n///     indented.\n///\n///   Gamma.\nfn item() {}\n"
    );
    // Tabs count as margin bytes exactly like spaces.
    assert_eq!(
        doc_text("/// \tAlpha.\n/// \t\tindented.\nfn item() {}\n"),
        "Alpha.\n\tindented.\n"
    );
}

/// A line holding only non-ASCII whitespace is authored content:
///  margin removal never consumes or splits it.
#[test]
fn processors_keep_non_ascii_whitespace_lines_outside_stripped_margins() {
    assert_eq!(
        doc_text("///   Alpha.\n/// \u{a0}\n///   Beta.\nfn item() {}\n"),
        "Alpha.\n\u{a0}\nBeta.\n"
    );
    assert_eq!(
        doc_text("/**\n    Alpha.\n\u{3000}\u{3000}\n    Beta.\n*/\nfn item() {}\n"),
        "\nAlpha.\n\u{3000}\u{3000}\nBeta.\n"
    );
}

/// CRLF line docs keep each carriage return beside its line feed,
///  including on an empty comment line.
#[test]
fn processors_keep_crlf_pairs_whole_in_line_doc_runs() {
    assert_eq!(
        doc_text("/// Alpha.\r\n///\r\n///   Beta.\r\nfn item() {}\r\n"),
        "Alpha.\r\n\r\n  Beta.\r\n"
    );
    // The final comment of a file has no newline to copy.
    assert_eq!(doc_text("/// Alpha.\n/// Beta."), "Alpha.\nBeta.");
}

/// The opening line of a block loses one conventional space;
///  opening prose fixes the continuation margin at zero.
#[test]
fn processors_strip_one_conventional_space_from_block_doc_opening_lines() {
    assert_eq!(
        doc_text("/** Alpha beta. */\nfn item() {}\n"),
        "Alpha beta. "
    );
    assert_eq!(doc_text("/**Alpha beta.*/\nfn item() {}\n"), "Alpha beta.");
    assert_eq!(
        doc_text("/**  Alpha.\n    Beta.\n*/\nfn item() {}\n"),
        " Alpha.\n    Beta.\n"
    );
}

/// Undecorated block lines lose the common continuation margin and keep deeper indentation.
#[test]
fn processors_strip_only_the_common_margin_from_undecorated_block_docs() {
    let host: &str = "/**\n    Alpha.\n      indented.\n\n    Beta.\n*/\nfn item() {}\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Rust);
    assert_eq!(virtuals[0].source(), "\nAlpha.\n  indented.\n\nBeta.\n");
    let split: usize = virtuals[0].source().find("ted.").expect("indented prose");
    assert_eq!(
        fixed(
            host,
            &virtuals[0],
            Fix {
                edits: vec![Edit {
                    start: split,
                    end: split,
                    replacement: String::from("\n"),
                }]
            }
        ),
        "/**\n    Alpha.\n      inden\n    ted.\n\n    Beta.\n*/\nfn item() {}\n"
    );
}

/// Extract every Rustdoc input of a host as owned (virtual name,
///  exact virtual text) pairs,
///  in extraction order.
fn named_texts(host: &str) -> Vec<(String, String)> {
    // Owned pairs outlive the extracted inputs, so assertions compare plain strings.
    let mut result: Vec<(String, String)> = Vec::new();
    for input in inputs(host, ProcessorLanguage::Rust) {
        result.push((String::from(input.filename()), String::from(input.source())));
    }
    return result;
}

/// Adjacent doc comments are separate virtual files unless they form one line-comment run:
/// two adjacent block docs with the same prefix,
///  an inner line doc directly followed by an outer one,
/// and an outer line doc directly followed by a block doc each stay apart.
#[test]
fn processors_keep_adjacent_blocks_and_changed_prefixes_in_separate_virtual_files() {
    // Each tuple is (host, expected virtual files as (name, text)); the line number in a name is the comment's first line.
    let cases: [(&str, [(&str, &str); 2]); 3] = [
        (
            "/** Alpha. */\n/** Beta. */\nfn item() {}\n",
            [("host/1.md", "Alpha. "), ("host/2.md", "Beta. ")],
        ),
        (
            "//! Inner.\n/// Outer.\nfn item() {}\n",
            [("host/1.md", "Inner.\n"), ("host/2.md", "Outer.\n")],
        ),
        (
            "/// Outer.\n/** Block. */\nfn item() {}\n",
            [("host/1.md", "Outer.\n"), ("host/2.md", "Block. ")],
        ),
    ];
    for (host, expected) in cases {
        // Owned expectation pairs compare by value with the extracted owned pairs.
        let mut wanted: Vec<(String, String)> = Vec::new();
        for (name, text) in expected {
            wanted.push((String::from(name), String::from(text)));
        }
        assert_eq!(named_texts(host), wanted, "{host}");
    }
    // The positive control: two outer line docs on consecutive lines are one run and one virtual file.
    assert_eq!(
        named_texts("/// Alpha.\n/// Beta.\nfn item() {}\n"),
        [(String::from("host/1.md"), String::from("Alpha.\nBeta.\n"))]
    );
}

/// Decorated block lines lose their star and one space;
///  a bare star line becomes an empty virtual line.
#[test]
fn processors_strip_star_decoration_and_keep_relative_indentation_in_block_docs() {
    assert_eq!(
        doc_text("/**\n * Alpha.\n *   indented.\n *\n * Beta.\n */\nfn item() {}\n"),
        "\nAlpha.\n  indented.\n\nBeta.\n"
    );
}
