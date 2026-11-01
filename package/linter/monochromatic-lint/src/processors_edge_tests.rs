//! What:
//!  Endpoint,
//!  nesting and unsupported-container controls at the public processor seam.
//! Why:
//!  Null diagnostics and refused fixes count only after positive authored controls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Vary every line endpoint and compare exact reparsed host/virtual source.
//! ```

/// Import the same consumer helpers and atomic fix models as ordinary projection tests.
use super::{Edit, Fix, ProcessorLanguage, finding, fixed, inputs};
/// Preserve the existing diagnostic severity model.
use crate::diagnostic::{Diagnostic, Severity};
/// Exercise the empty-snippet documentation policy through typed real rule selection.
use crate::rust_rule_settings::{LineBudget, RustRuleSettings};

/// First,
///  middle and last full-line deletion preserve surviving prefixes and bytes.
#[test]
fn processors_project_each_doc_line_endpoint_with_mixed_newline_spellings() {
    let source: &str = "/// Alpha.\r\n/// Beta.\n/// Gamma.\r\nfn item() {}\n";
    for (start, end, expected) in [
        (0, 8, "/// Beta.\n/// Gamma.\r\nfn item() {}\n"),
        (8, 14, "/// Alpha.\r\n/// Gamma.\r\nfn item() {}\n"),
        (14, 22, "/// Alpha.\r\n/// Beta.\nfn item() {}\n"),
    ] {
        let virtuals = inputs(source, ProcessorLanguage::Rust);
        assert_eq!(
            fixed(
                source,
                &virtuals[0],
                Fix {
                    edits: vec![Edit {
                        start,
                        end,
                        replacement: String::new(),
                    }]
                }
            ),
            expected
        );
    }
}

/// A full last-line replacement ending with newline creates no orphan prefix at EOF.
#[test]
fn processors_project_trailing_newline_replacements_without_orphan_prefixes() {
    let source: &str = "/// Alpha.\r\nfn item() {}\r\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let actual: String = fixed(
        source,
        &virtuals[0],
        Fix {
            edits: vec![Edit {
                start: 0,
                end: virtuals[0].source().len(),
                replacement: String::from("Beta.\nGamma.\n"),
            }],
        },
    );
    assert_eq!(actual, "/// Beta.\r\n/// Gamma.\r\nfn item() {}\r\n");
}

/// Project through both allowed nested Rust-fence levels,
///  not only one Markdown/Rustdoc pair.
#[test]
fn processors_project_depth_two_fix_to_original_host() {
    let source: String = super::fixture("nested");
    let virtuals = inputs(source.as_str(), ProcessorLanguage::Rust);
    let rust = virtuals
        .iter()
        .find(|input| return input.filename() == "host/2.md/0.rs/2.md/0.rs")
        .expect("second doctest");
    let at: usize = rust.source().find("= 1").expect("literal") + 2;
    assert_eq!(
        fixed(
            source.as_str(),
            rust,
            Fix {
                edits: vec![Edit {
                    start: at,
                    end: at + 1,
                    replacement: String::from("2"),
                }]
            }
        ),
        source.replace("= 1", "= 2")
    );
    let mapped = rust
        .project_diagnostic(finding(rust, at, 1))
        .expect("mapped")
        .expect("authored");
    assert_eq!(
        mapped.labels[0].span.offset,
        source.find("= 1").expect("original") + 2
    );
    let prose = virtuals
        .iter()
        .find(|input| return input.filename() == "host/2.md/0.rs/2.md/0.rs/1.md")
        .expect("Markdown at final Rust depth");
    let word: usize = prose.source().find("two").expect("authored prose");
    assert_eq!(
        fixed(
            source.as_str(),
            prose,
            Fix {
                edits: vec![Edit {
                    start: word,
                    end: word + 3,
                    replacement: String::from("second"),
                }]
            }
        ),
        source.replace("Level two", "Level second")
    );
}

/// Empty virtual snippets must not silently satisfy the opening Rustdoc requirement.
#[test]
fn processors_report_missing_opening_docs_for_empty_rust_fences() {
    let virtuals = inputs("```rust\n```\n", ProcessorLanguage::Markdown);
    let findings: Vec<Diagnostic> = virtuals[0]
        .check_rust(RustRuleSettings {
            rustdoc: Some(Severity::Error),
            ..RustRuleSettings::default()
        })
        .expect("syntax check");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].message, "Missing rustdoc on file.");
    assert_eq!(findings[0].filename, "host");
    let empty_doc = inputs("/*!*/\n", ProcessorLanguage::Rust);
    assert!(empty_doc[0].source().is_empty());
    let mapped = empty_doc[0]
        .project_diagnostic(finding(&empty_doc[0], 0, 0))
        .expect("empty authored mapping")
        .expect("authored finding must not be suppressed");
    assert_eq!(mapped.filename, "host");
    assert_eq!(mapped.labels[0].span.offset, 0);
}

/// Authored labels spanning multiple lines include original container bytes,
///  with correct Unicode host columns.
#[test]
fn processors_map_multi_line_unicode_labels_and_reject_wrong_language_dispatch() {
    let source: &str = "/// 😀 Alpha.\n/// Beta.\nfn item() {}\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let mut multi_line: Diagnostic = finding(&virtuals[0], 5, 0);
    multi_line.labels[0].span.length = 10;
    let mapped = virtuals[0]
        .project_diagnostic(multi_line)
        .expect("map")
        .expect("authored");
    assert_eq!(mapped.labels[0].span.offset, 9);
    assert_eq!(mapped.labels[0].span.column, 10);
    assert_eq!(mapped.labels[0].span.length, 14);
    assert!(virtuals[0].check_rust(RustRuleSettings::default()).is_err());
}

/// Inline line docs have no reusable safe prefix and are refused instead of copying preceding Rust code.
#[test]
fn processors_refuse_inline_line_docs_but_support_inline_block_docs() {
    assert!(
        super::extract(
            String::from("host"),
            String::from("fn item() { /// docs\n}\n"),
            ProcessorLanguage::Rust
        )
        .is_err()
    );
    let source: &str = "fn item() { /** Alpha beta. */ let x: u32 = 1; }\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let at: usize = virtuals[0].source().find(" beta").expect("body");
    assert_eq!(
        fixed(
            source,
            &virtuals[0],
            Fix {
                edits: vec![Edit {
                    start: at,
                    end: at + 1,
                    replacement: String::from("\n"),
                }]
            }
        ),
        "fn item() { /** Alpha\nbeta. */ let x: u32 = 1; }\n"
    );
}

/// Ordinary comments and blank physical lines divide runs even when the next token is authored Rustdoc.
#[test]
fn processors_split_authored_doc_runs_at_blank_lines_and_ordinary_comments() {
    let virtuals = inputs(
        "/// First.\n\n/// Second.\n// plain\n/// Third.\nfn item() {}\n",
        ProcessorLanguage::Rust,
    );
    assert_eq!(virtuals.len(), 3);
    assert_eq!(virtuals[0].filename(), "host/1.md");
    assert_eq!(virtuals[1].filename(), "host/3.md");
    assert_eq!(virtuals[2].filename(), "host/5.md");
}

/// Common authored margins expose fences without treating their indentation as an indented-code block.
#[test]
fn processors_extract_and_project_indented_block_and_line_doc_fences() {
    for source in [
        "/**\n    ```rust\n    //! Example.\n    let value: u32 = 1;\n    ```\n*/\nfn item() {}\n",
        "///     ```rust\n///     //! Example.\n///     let value: u32 = 1;\n///     ```\nfn item() {}\n",
    ] {
        let virtuals = inputs(source, ProcessorLanguage::Rust);
        let rust = virtuals
            .iter()
            .find(|input| return input.language() == ProcessorLanguage::Rust)
            .expect("dedented fence");
        let at: usize = rust.source().find("= 1").expect("literal") + 2;
        assert_eq!(
            fixed(
                source,
                rust,
                Fix {
                    edits: vec![Edit {
                        start: at,
                        end: at + 1,
                        replacement: String::from("2"),
                    }]
                }
            ),
            source.replace("= 1", "= 2")
        );
    }
    let indented: &str = "    /**\n     * Alpha.\n     * Beta.\n     */\n    fn item() {}\n";
    let block = inputs(indented, ProcessorLanguage::Rust);
    let start: usize = block[0].source().find("Beta").expect("final prose");
    let length: usize = "Beta.\n".len();
    assert_eq!(
        fixed(
            indented,
            &block[0],
            Fix {
                edits: vec![Edit {
                    start,
                    end: start + length,
                    replacement: String::new(),
                }]
            }
        ),
        "    /**\n     * Alpha.\n     */\n    fn item() {}\n"
    );
    let split: usize = block[0].source().find("Alpha").expect("prose") + 3;
    assert_eq!(
        fixed(
            indented,
            &block[0],
            Fix {
                edits: vec![Edit {
                    start: split,
                    end: split,
                    replacement: String::from("\n"),
                }]
            }
        ),
        "    /**\n     * Alp\n     * ha.\n     * Beta.\n     */\n    fn item() {}\n"
    );
    let ordinary_stars = inputs(
        "/**\n * list\n prose\n*/\nfn item() {}\n",
        ProcessorLanguage::Rust,
    );
    assert!(ordinary_stars[0].source().contains("* list"));
}

/// Both the authored line budget and real documentable items report authored positions.
#[test]
fn processors_check_the_authored_line_budget_and_documentable_items() {
    let source: &str = "```rust\n//! Example.\nlet callback = 1;\nfn helper() {}\n```\n";
    let virtuals = inputs(source, ProcessorLanguage::Markdown);
    let findings = virtuals[0]
        .check_rust(RustRuleSettings {
            rustdoc: Some(Severity::Warn),
            max_lines: Some(LineBudget {
                severity: Severity::Error,
                max: 0,
            }),
            ..RustRuleSettings::default()
        })
        .expect("prepared checks");
    assert_eq!(findings.len(), 2);
    // The budget counts the authored lines only and anchors on the first one beyond it, `let callback`.
    let budget = findings
        .iter()
        .find(|item| return item.code == "rust/max-lines")
        .expect("authored line budget finding");
    assert_eq!(budget.severity, Severity::Error);
    assert_eq!(budget.labels[0].span.line, 3);
    assert!(
        findings
            .iter()
            .any(|item| return item.message == "Missing rustdoc on function \"helper\".")
    );
    assert!(findings.iter().all(|item| return item.filename == "host"));
    let explicit = inputs(
        "```rust\n//! Example.\nfn main() {}\n```\n",
        ProcessorLanguage::Markdown,
    );
    let main_findings = explicit[0]
        .check_rust(RustRuleSettings {
            rustdoc: Some(Severity::Error),
            ..RustRuleSettings::default()
        })
        .expect("authored main check");
    assert_eq!(main_findings.len(), 1);
    assert_eq!(
        main_findings[0].message,
        "Missing rustdoc on function \"main\"."
    );
}

/// Multiple labels retain authored ranges;
///  synthetic failures are never silently treated as clean.
#[test]
fn processors_map_multiple_labels_and_preserve_processing_failures() {
    let virtuals = inputs(
        "```rust\n//! Example.\nlet x: u32 = 1;\n```\n",
        ProcessorLanguage::Markdown,
    );
    let rust = &virtuals[0];
    let generated: usize = rust.source().find("fn main").expect("synthetic");
    let authored: usize = rust.source().find("let x").expect("authored");
    let mut multi: Diagnostic = finding(rust, generated, 2);
    multi
        .labels
        .push(finding(rust, authored, 3).labels.remove(0));
    let literal: usize = rust.source().find("= 1").expect("literal") + 2;
    multi.fix = Some(Fix {
        edits: vec![Edit {
            start: literal,
            end: literal + 1,
            replacement: String::from("2"),
        }],
    });
    let mapped = rust
        .project_diagnostic(multi)
        .expect("labels")
        .expect("authored");
    assert_eq!(mapped.labels.len(), 1);
    assert_eq!(mapped.fix.expect("mapped atomic fix").edits.len(), 1);
    let mut failure: Diagnostic = finding(rust, generated, 2);
    failure.processing_failure = true;
    assert!(
        rust.project_diagnostic(failure)
            .expect("failure mapping")
            .expect("failure preserved")
            .processing_failure
    );
    let mut unlabeled: Diagnostic = finding(rust, authored, 0);
    unlabeled.labels.clear();
    assert!(rust.project_diagnostic(unlabeled).is_err());
    let mut overflow: Diagnostic = finding(rust, authored, 0);
    overflow.labels[0].span.length = usize::MAX;
    assert!(rust.project_diagnostic(overflow).is_err());
    let broken: &str = "\u{feff}<A>\n</B>\n";
    let error = super::extract(
        String::from("broken.mdx"),
        String::from(broken),
        ProcessorLanguage::Mdx,
    )
    .expect_err("native closing-tag failure");
    assert_eq!(error.filename, "broken.mdx");
    assert_eq!(
        error.offset,
        broken.find("</B>").expect("affected original marker")
    );
}

/// No-op fixes preserve mixed newline spelling and never invent output edits.
#[test]
fn processors_keep_no_op_groups_byte_identical() {
    let source: &str = "/// Alpha.\r\n/// Beta.\nfn item() {}\r\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let fix: Fix = virtuals[0]
        .project_fix(&Fix {
            edits: vec![Edit {
                start: 0,
                end: virtuals[0].source().len(),
                replacement: String::from(virtuals[0].source()),
            }],
        })
        .expect("no op");
    assert!(fix.edits.is_empty());
}
