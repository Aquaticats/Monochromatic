//! What:
//!  Standalone consumer of the built internal library artifact.
//! Why:
//!  Module registration,
//!  exports and grouped edits must work outside the crate's unit tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Import the built library, extract a doctest, project a fix, apply and inspect host bytes.
//! ```

/// Explicit virtual semantic failures must be visible to ordinary library consumers.
use monochromatic_lint::diagnostic::Severity;
/// Apply projected fixes through the same owning edit engine.
use monochromatic_lint::edits::{Edit, Fix, apply_fixes};
/// Import the actual library artifact's processor and edit exports.
use monochromatic_lint::processors::{ProcessorLanguage, VirtualSource, extract};
/// Check real selected rule behavior from the consumer.
use monochromatic_lint::rust_rule_settings::RustRuleSettings;

/// Find a Rust virtual file through the public language interface.
fn rust_input(input: &&VirtualSource) -> bool {
    return input.language() == ProcessorLanguage::Rust;
}

/// Verification failures caused by a proposed rewrite must still address the original host snapshot.
fn verify_original_failure_anchor() {
    let host: &str = "```rust\n//! Example.\nlet value: u32 = 1;\n```\n";
    let inputs: Vec<VirtualSource> = extract(
        String::from("anchor.mdx"),
        String::from(host),
        ProcessorLanguage::Mdx,
    )
    .expect("original MDX");
    let rust: &VirtualSource = inputs.iter().find(rust_input).expect("Rust input");
    let at: usize = rust.source().find("let value").expect("authored insertion");
    let error = rust
        .project_fix(&Fix {
            edits: vec![Edit {
                start: at,
                end: at,
                replacement: String::from(
                    "```\n<A>\npadding padding padding padding padding padding padding\n</B>\n",
                ),
            }],
        })
        .expect_err("proposed MDX closing tag must be refused");
    assert_eq!(error.filename, "anchor.mdx");
    assert!(
        error.offset <= host.len(),
        "failure byte {} exceeds original host length {}",
        error.offset,
        host.len()
    );
    assert!(host.is_char_boundary(error.offset));
}

/// Extract and fix a hidden line in an authored CRLF Rustdoc doctest.
fn main() {
    let host: &str = "/// ```rust\r\n/// //! Example.\r\n/// # let value: u32 = 1;\r\n/// ```\r\nfn item() {}\r\n";
    let inputs: Vec<VirtualSource> = extract(
        String::from("consumer.rs"),
        String::from(host),
        ProcessorLanguage::Rust,
    )
    .expect("artifact extraction");
    let rust: &VirtualSource = inputs.iter().find(rust_input).expect("Rust doctest");
    assert_eq!(rust.filename(), "consumer.rs/1.md/0.rs");
    let at: usize = rust.source().find("= 1").expect("authored literal") + 2;
    let projected: Fix = rust
        .project_fix(&Fix {
            edits: vec![Edit {
                start: at,
                end: at + 1,
                replacement: String::from("2"),
            }],
        })
        .expect("artifact projection");
    let changed = apply_fixes(host, &[projected]).expect("artifact atomic edit");
    assert_eq!(changed.source, host.replace("= 1", "= 2"));
    let failures = rust
        .check_rust(RustRuleSettings {
            explicit_types: Some(Severity::Error),
            ..RustRuleSettings::default()
        })
        .expect("semantic refusal");
    assert_eq!(failures.len(), 1);
    assert!(failures[0].processing_failure);
    assert_eq!(failures[0].filename, "consumer.rs");
    verify_original_failure_anchor();
    println!("processor artifact consumer passed");
}
