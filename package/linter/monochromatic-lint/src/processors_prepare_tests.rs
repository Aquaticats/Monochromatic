//! What:
//!  Exact prepared doctest text for hidden markers,
//!  helper functions and Rustdoc identity.
//! Why:
//!  Substring checks cannot tell a stripped marker from a kept one,
//!  or a wrapped fragment from a bare one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Extract a fence, compare the whole prepared Rust string, then map a hidden-line fix to the host.
//! ```

/// Import the consumer helpers and atomic fix models shared by every processor test module.
use super::{Edit, Fix, ProcessorLanguage, VirtualSource, fixed, inputs};
/// Typed refusals carry the explanation and authored anchor under test.
use crate::processors::ProcessorError;

/// Only a function named main suppresses the generated wrapper;
///  any other top-level function is wrapped.
#[test]
fn processors_wrap_fragments_whose_only_functions_are_helpers() {
    let virtuals: Vec<VirtualSource> = inputs(
        "```rust\n//! Example.\nfn helper() {}\nlet value: u32 = 1;\n```\n",
        ProcessorLanguage::Markdown,
    );
    assert_eq!(
        virtuals[0].source(),
        "//! Example.\nfn main() {\nfn helper() {}\nlet value: u32 = 1;\n}\n"
    );
    // A main nested in a module is not the entry point of the fragment either.
    let nested: Vec<VirtualSource> = inputs(
        "```rust\n//! Example.\nmod inner { fn main() {} }\n```\n",
        ProcessorLanguage::Markdown,
    );
    assert_eq!(
        nested[0].source(),
        "//! Example.\nfn main() {\nmod inner { fn main() {} }\n}\n"
    );
}

/// A bare hidden marker becomes an empty line,
///  at any indentation,
///  and never opens the wrapper.
#[test]
fn processors_strip_bare_hidden_markers_to_empty_lines() {
    let host: &str = "```rust\n//! Example.\n#\n## visible\n  #\n# let x: u32 = 1;\nlet text = \"fn main()\";\n```\n";
    let virtuals: Vec<VirtualSource> = inputs(host, ProcessorLanguage::Markdown);
    assert_eq!(
        virtuals[0].source(),
        "//! Example.\n\nfn main() {\n# visible\n\nlet x: u32 = 1;\nlet text = \"fn main()\";\n}\n"
    );
    let literal: usize = virtuals[0].source().find("= 1").expect("hidden literal") + 2;
    assert_eq!(
        fixed(
            host,
            &virtuals[0],
            Fix {
                edits: vec![Edit {
                    start: literal,
                    end: literal + 1,
                    replacement: String::from("2"),
                }]
            }
        ),
        host.replace("= 1", "= 2")
    );
    // Text typed on an emptied line would follow its marker without a space and stop being hidden.
    let emptied: usize = virtuals[0].source().find("\n\nlet x").expect("bare marker") + 1;
    let unhidden: ProcessorError = virtuals[0]
        .project_fix(&Fix {
            edits: vec![Edit {
                start: emptied,
                end: emptied,
                replacement: String::from("let y: u32 = 2;"),
            }],
        })
        .expect_err("marker without its space");
    assert_eq!(
        unhidden.message,
        "Projected fix changes its Markdown/Rust container or hidden-line preparation. No edits in this atomic group can be applied safely."
    );
    // The hidden-line layer refuses, so the anchor is the first authored byte of the snippet.
    assert_eq!(
        unhidden.offset,
        host.find("//! Example").expect("first authored byte")
    );
}

/// Rustdoc identity belongs to comment-derived Markdown only,
///  never to the Rust it embeds.
#[test]
fn processors_mark_only_comment_derived_markdown_as_rustdoc() {
    let virtuals: Vec<VirtualSource> = inputs(
        "/// ```\n/// //! Example.\n/// let value: u32 = 1;\n/// ```\nfn item() {}\n",
        ProcessorLanguage::Rust,
    );
    let mut names: Vec<(&str, bool)> = Vec::new();
    for input in &virtuals {
        names.push((input.filename(), input.is_rustdoc()));
    }
    assert_eq!(
        names,
        [
            ("host/1.md", true),
            ("host/1.md/0.rs", false),
            ("host/1.md/0.rs/1.md", true)
        ]
    );
    let markdown: Vec<VirtualSource> = inputs(
        "```rust\nlet value: u32 = 1;\n```\n",
        ProcessorLanguage::Markdown,
    );
    assert_eq!(markdown.len(), 1);
    assert!(!markdown[0].is_rustdoc());
}
