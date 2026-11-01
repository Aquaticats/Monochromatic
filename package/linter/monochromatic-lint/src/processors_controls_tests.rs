//! What:
//!  Unsupported mappings,
//!  malformed delimiters and deterministic bounded fuzz controls.
//! Why:
//!  A corrupting fix or clean result on unsupported input would invalidate processor verification.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Generate bounded Unicode/container cases and verify native round trips and refusal controls.
//! ```

/// Import the real consumer seam and immutable fixture helpers.
use super::{Edit, Fix, ProcessorLanguage, finding, fixed, inputs};
/// Application guards apply to original hosts,
///  not just extracted strings.
use crate::edits::apply_fixes;

/// A forged fence closer must not escape its enclosing container.
#[test]
fn processors_refuse_delimiter_injection_and_synthetic_fix_ranges() {
    let source: &str = "```rust\n//! Example.\nlet value: u32 = 1;\n```\n";
    let virtuals = inputs(source, ProcessorLanguage::Markdown);
    let rust = &virtuals[0];
    let at: usize = rust.source().find("let value").expect("authored");
    assert!(
        rust.project_fix(&Fix {
            edits: vec![Edit {
                start: at,
                end: at,
                replacement: String::from("```\n"),
            }]
        })
        .is_err()
    );
    let generated: usize = rust.source().find("fn main").expect("generated");
    assert!(
        rust.project_fix(&Fix {
            edits: vec![
                Edit {
                    start: generated,
                    end: generated + 2,
                    replacement: String::from("x")
                },
                Edit {
                    start: at,
                    end: at + 3,
                    replacement: String::from("let")
                },
            ]
        })
        .is_err()
    );
}

/// Invalid Unicode boundaries,
///  internal group overlap and CRLF splitting fail explicitly.
#[test]
fn processors_refuse_invalid_byte_spans_and_atomic_group_overlap() {
    let virtuals = inputs("/// 😀 text.\r\nfn item() {}\r\n", ProcessorLanguage::Rust);
    let doc = &virtuals[0];
    for edits in [
        vec![Edit {
            start: 1,
            end: 2,
            replacement: String::new(),
        }],
        vec![
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
        vec![Edit {
            start: doc.source().len() - 1,
            end: doc.source().len(),
            replacement: String::new(),
        }],
        vec![Edit {
            start: 0,
            end: usize::MAX,
            replacement: String::new(),
        }],
    ] {
        assert!(doc.project_fix(&Fix { edits }).is_err());
    }
    let mut invalid = finding(doc, 0, 0);
    invalid.labels[0].span.offset = 1;
    assert!(doc.project_diagnostic(invalid).is_err());
}

/// Unclosed block docs cannot promise a safe mapping;
///  syntax errors inside compile_fail fences remain lintable.
#[test]
fn processors_handle_malformed_host_delimiters_without_guessing() {
    assert!(
        super::extract(
            String::from("host"),
            String::from("/** unclosed"),
            ProcessorLanguage::Rust
        )
        .is_err()
    );
    let virtuals = inputs(
        "```rust,compile_fail\n//! Example.\nlet x: = ;\n```\n",
        ProcessorLanguage::Markdown,
    );
    assert!(!virtuals[0].source().is_empty());
    let empty = inputs("```rust\n```\n", ProcessorLanguage::Markdown);
    assert!(
        empty[0]
            .project_fix(&Fix {
                edits: vec![Edit {
                    start: 0,
                    end: 0,
                    replacement: String::from("let x: u32 = 1;"),
                }]
            })
            .is_err()
    );
}

/// Native main detection excludes comment/string/nested-function spellings and honors an authored top-level main.
#[test]
fn processors_prepare_main_and_hidden_escape_controls() {
    for body in [
        "//! Example.\nfn main() {}\n",
        "//! Example.\npub fn main() {}\n",
        "//! Example.\nfn r#main() {}\n",
    ] {
        let virtuals = inputs(
            format!("```rust\n{body}```\n").as_str(),
            ProcessorLanguage::Markdown,
        );
        assert_eq!(virtuals[0].source(), body);
    }
    let virtuals = inputs(
        "```rust\n//! Example.\n#\n## visible\n# let x: u32 = 1;\nlet text = \"fn main()\";\n```\n",
        ProcessorLanguage::Markdown,
    );
    assert!(virtuals[0].source().contains("# visible\nlet x"));
    assert!(virtuals[0].source().contains("fn main() {\n"));
    let doc_only = inputs(
        "```rust\n//! Only docs.\n```\n",
        ProcessorLanguage::Markdown,
    );
    assert_eq!(doc_only[0].source(), "//! Only docs.\nfn main() {\n}\n");
    let empty = inputs("```rust\n```\n", ProcessorLanguage::Markdown);
    assert_eq!(empty[0].source(), "fn main() {\n}\n");
    let eof: &str = "```rust\n# let value: u32 = 1;";
    let unterminated = inputs(eof, ProcessorLanguage::Markdown);
    assert!(
        unterminated[0]
            .source()
            .ends_with("let value: u32 = 1;\n}\n")
    );
    let at: usize = unterminated[0].source().find("= 1").expect("EOF literal") + 2;
    assert_eq!(
        fixed(
            eof,
            &unterminated[0],
            Fix {
                edits: vec![Edit {
                    start: at,
                    end: at + 1,
                    replacement: String::from("2"),
                }]
            }
        ),
        eof.replace("= 1", "= 2")
    );
}

/// Generated adversarial cases vary payloads,
///  newline grammars,
///  fence markers and Markdown/Rustdoc containers.
#[test]
fn processors_bounded_fuzz_roundtrips_preserve_untouched_host_bytes() {
    let mut exercised: usize = 0;
    for ending in ["\n", "\r\n", "\r"] {
        for prefix in ["", "> ", "> > ", "  "] {
            for payload in [
                "α😀",
                "quote \" slash \\",
                "#[x]",
                "../../;|&",
                "~~~",
                "```",
                "plain",
            ] {
                let marker: &str = if payload == "```" { "~~~~" } else { "````" };
                let body: String = format!("//! {payload}.{ending}let value: u32 = 1;{ending}");
                let host: String = format!(
                    "{prefix}{marker}rust,compile_fail{ending}{prefix}//! {payload}.{ending}{prefix}let value: u32 = 1;{ending}{prefix}{marker}{ending}untouched 😀"
                );
                let virtuals = inputs(host.as_str(), ProcessorLanguage::Markdown);
                let rust = &virtuals[0];
                let at: usize = rust.source().find("= 1").expect("literal") + 2;
                let actual: String = fixed(
                    host.as_str(),
                    rust,
                    Fix {
                        edits: vec![Edit {
                            start: at,
                            end: at + 1,
                            replacement: String::from("2"),
                        }],
                    },
                );
                assert_eq!(actual, host.replace("= 1", "= 2"));
                assert!(actual.ends_with("untouched 😀"));
                assert!(
                    rust.source()
                        .starts_with(body.split("let value").next().expect("inner docs"))
                );
                let mapped = rust
                    .project_diagnostic(finding(rust, at, 1))
                    .expect("mapped")
                    .expect("authored");
                assert_eq!(
                    mapped.labels[0].span.offset,
                    host.find("= 1").expect("host") + 2
                );
                exercised += 1;
            }
        }
    }
    assert_eq!(exercised, 84);
}

/// Shared host empty-output refusal remains in force after processor projection.
#[test]
fn processors_keep_original_host_nonempty_guard() {
    let source: &str = "/// only\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let projected: Fix = virtuals[0]
        .project_fix(&Fix {
            edits: vec![Edit {
                start: 0,
                end: virtuals[0].source().len(),
                replacement: String::new(),
            }],
        })
        .expect("whole doc deletion");
    assert!(apply_fixes(source, &[projected]).is_err());
}
