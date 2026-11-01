//! What:
//!  Real grouped host edits across Markdown,
//!  Rustdoc and preparation layers.
//! Why:
//!  Newline insertion,
//!  full-line deletion and overlapping fixes must not damage prefixes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Extract, project, apply the production atomic group, then reparse the original host.
//! ```

/// Import consumer helpers and the actual edit interface.
use super::{Edit, Fix, ProcessorLanguage, fixed, inputs};
/// Conflict selection belongs to the existing host edit engine.
use crate::edits::apply_fixes;

/// Newlines inserted into /// prose gain the original indentation,
///  prefix and CRLF spelling.
#[test]
fn processors_project_rustdoc_newlines_with_prefixes_and_untouched_bytes() {
    let source: &str =
        "//! Root.\r\n  /// Alpha beta.\r\n  /// Untouched 😀.\r\n  fn item() {}\r\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let doc = &virtuals[1];
    let at: usize = doc.source().find(" beta").expect("break");
    let actual: String = fixed(
        source,
        doc,
        Fix {
            edits: vec![Edit {
                start: at,
                end: at + 1,
                replacement: String::from("\n"),
            }],
        },
    );
    assert_eq!(
        actual,
        "//! Root.\r\n  /// Alpha\r\n  /// beta.\r\n  /// Untouched 😀.\r\n  fn item() {}\r\n"
    );
    assert_eq!(
        inputs(actual.as_str(), ProcessorLanguage::Rust)[1].source(),
        "Alpha\r\nbeta.\r\nUntouched 😀.\r\n"
    );
}

/// Removing one entire virtual line removes its own prefix,
///  never joining adjacent comment markers.
#[test]
fn processors_project_whole_doc_line_deletion_without_prefix_concatenation() {
    let source: &str = "/// First.\n/// Removed.\n/// Last.\nfn item() {}\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let doc = &virtuals[0];
    let start: usize = doc.source().find("Removed").expect("line");
    let actual: String = fixed(
        source,
        doc,
        Fix {
            edits: vec![Edit {
                start,
                end: start + "Removed.\n".len(),
                replacement: String::new(),
            }],
        },
    );
    assert_eq!(actual, "/// First.\n/// Last.\nfn item() {}\n");
}

/// Partial line joining removes only the consumed following prefix.
#[test]
fn processors_project_partial_cross_line_deletion_and_atomic_multi_edits() {
    let source: &str = "/// First.\n/// Last.\nfn item() {}\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let actual: String = fixed(
        source,
        &virtuals[0],
        Fix {
            edits: vec![
                Edit {
                    start: 0,
                    end: 1,
                    replacement: String::from("f"),
                },
                Edit {
                    start: 6,
                    end: 7,
                    replacement: String::from(" "),
                },
            ],
        },
    );
    assert_eq!(actual, "/// first. Last.\nfn item() {}\n");
}

/// Preparation and nested Rustdoc prefixes compose back through the same host group.
#[test]
fn processors_project_hidden_nested_doctest_fixes_to_original_host() {
    let source: &str = "/// ```rust\r\n/// //! Example.\r\n/// # let value: u32 = 1;\r\n/// value;\r\n/// ```\r\nfn item() {}\r\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let rust = virtuals
        .iter()
        .find(|input| return input.filename().ends_with("/0.rs"))
        .expect("doctest");
    let at: usize = rust.source().find("= 1").expect("literal") + 2;
    let actual: String = fixed(
        source,
        rust,
        Fix {
            edits: vec![Edit {
                start: at,
                end: at + 1,
                replacement: String::from("2"),
            }],
        },
    );
    assert_eq!(actual, source.replace("= 1", "= 2"));
}

/// A newly inserted hidden line retains its # marker as well as its enclosing comment prefix.
#[test]
fn processors_project_inserted_hidden_lines_with_all_container_prefixes() {
    let source: &str =
        "/// ```rust\n/// //! Example.\n/// # let x: u32 = 1;\n/// x;\n/// ```\nfn item() {}\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let rust = virtuals
        .iter()
        .find(|input| return input.filename().ends_with("/0.rs"))
        .expect("doctest");
    let at: usize = rust.source().find("let x").expect("hidden") + "let x: u32 = 1;".len();
    let actual: String = fixed(
        source,
        rust,
        Fix {
            edits: vec![Edit {
                start: at,
                end: at,
                replacement: String::from("\nlet y: u32 = 2;"),
            }],
        },
    );
    assert!(actual.contains("/// # let x: u32 = 1;\n/// # let y: u32 = 2;\n"));
}

/// A whole virtual Rust body may become empty while the real file's container bytes remain nonempty.
#[test]
fn processors_project_nonempty_virtual_doc_to_empty_without_emptying_host() {
    let source: &str = "/// Prose.\nfn item() {}\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let actual: String = fixed(
        source,
        &virtuals[0],
        Fix {
            edits: vec![Edit {
                start: 0,
                end: virtuals[0].source().len(),
                replacement: String::new(),
            }],
        },
    );
    assert_eq!(actual, "fn item() {}\n");
}

/// Competing projected groups remain whole when one edit of the later group conflicts.
#[test]
fn processors_preserve_atomic_groups_when_host_fixes_overlap() {
    let source: &str = "/// Alpha beta.\nfn item() {}\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let first: Fix = virtuals[0]
        .project_fix(&Fix {
            edits: vec![Edit {
                start: 0,
                end: 5,
                replacement: String::from("First"),
            }],
        })
        .expect("first group");
    let later: Fix = virtuals[0]
        .project_fix(&Fix {
            edits: vec![
                Edit {
                    start: 1,
                    end: 2,
                    replacement: String::from("X"),
                },
                Edit {
                    start: 6,
                    end: 10,
                    replacement: String::from("second"),
                },
            ],
        })
        .expect("second group");
    let applied = apply_fixes(source, &[first, later]).expect("selection");
    assert_eq!(applied.applied, [0]);
    assert_eq!(applied.rejected, [1]);
    assert_eq!(applied.source, "/// First beta.\nfn item() {}\n");
}

/// Decorated block docs retain both delimiters and their decorative prefixes.
#[test]
fn processors_project_block_doc_insertions_and_refuse_closing_delimiters() {
    let source: &str = "/**\n * Alpha beta.\n * Last.\n */\nfn item() {}\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let at: usize = virtuals[0].source().find(" beta").expect("prose");
    let actual: String = fixed(
        source,
        &virtuals[0],
        Fix {
            edits: vec![Edit {
                start: at,
                end: at + 1,
                replacement: String::from("\n"),
            }],
        },
    );
    assert_eq!(
        actual,
        "/**\n * Alpha\n * beta.\n * Last.\n */\nfn item() {}\n"
    );
    assert!(
        virtuals[0]
            .project_fix(&Fix {
                edits: vec![Edit {
                    start: at,
                    end: at,
                    replacement: String::from("*/"),
                }]
            })
            .is_err()
    );
}
