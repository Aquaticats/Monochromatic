//! What:
//!  Consumer-seam controls using real native Rust and Markdown parses.
//! Why:
//!  Fixtures prove extraction identities and synthetic-main policy,
//!  not copied regex expectations.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Read immutable fixture strings, extract virtual files, run current rule selections.
//! ```

/// Import the actual interface and rule settings used by executable integration.
use super::{ProcessorLanguage, VirtualSource, extract};
/// Inspect original-host wire findings and apply real atomic groups.
use crate::diagnostic::{Diagnostic, Severity};
/// Fix application is always the production grouped edit implementation.
use crate::edits::{Edit, Fix, apply_fixes};
/// Exercise selected rule behavior,
///  including the authored code-line budget.
use crate::rust_rule_settings::{LineBudget, RustRuleSettings};

/// Load one committed fixture field without touching shared files during verification.
pub(super) fn fixture(name: &str) -> String {
    // Embed the fixture in the built test artifact, including in mount-free containers.
    let value: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/processors.json"))
            .expect("processor fixture JSON");
    return String::from(value[name].as_str().expect("named processor fixture"));
}

/// Extract a fixture or fail loudly rather than treating processor errors as an empty result.
pub(super) fn inputs(source: &str, language: ProcessorLanguage) -> Vec<VirtualSource> {
    return extract(String::from("host"), String::from(source), language)
        .expect("native extraction");
}

/// Make one localized finding as a rule consumer would.
pub(super) fn finding(input: &VirtualSource, start: usize, length: usize) -> Diagnostic {
    let parsed = crate::rust_source::RustSource::new(
        String::from(input.filename()),
        String::from(input.source()),
    );
    return Diagnostic::new(
        "test/finding",
        Severity::Warn,
        String::from("authored"),
        String::from(input.filename()),
        parsed.span(start, length),
    );
}

/// Project a grouped fix and apply it through the shared host engine.
pub(super) fn fixed(host: &str, input: &VirtualSource, fix: Fix) -> String {
    let projected: Fix = input.project_fix(&fix).expect("safe projection");
    return apply_fixes(host, &[projected])
        .expect("atomic host edits")
        .source;
}

/// Rustdoc attributes,
///  nested containers,
///  Unicode and CRLF retain logical paths and original bytes.
#[test]
fn processors_extract_native_fences_with_attributes_and_container_prefixes() {
    let source: String = fixture("fences");
    let virtuals: Vec<VirtualSource> = inputs(source.as_str(), ProcessorLanguage::Markdown);
    let rust: Vec<&VirtualSource> = virtuals
        .iter()
        .filter(|input| return input.language() == ProcessorLanguage::Rust)
        .collect();
    assert_eq!(rust.len(), 2);
    assert_eq!(rust[0].filename(), "host/0.rs");
    assert_eq!(rust[1].filename(), "host/1.rs");
    assert!(
        rust[0]
            .source()
            .starts_with("//! Unicode 😀.\r\nfn main() {\n")
    );
    assert!(rust[0].source().contains("let value: u32 = 1;\r\n"));
    let at: usize = rust[0].source().find("value;").expect("authored token");
    let mapped: Diagnostic = rust[0]
        .project_diagnostic(finding(rust[0], at, 5))
        .expect("valid diagnostic")
        .expect("authored finding");
    assert_eq!(mapped.filename, "host");
    assert_eq!(
        mapped.labels[0].span.offset,
        source.find("value;").expect("original token")
    );
    assert_eq!(mapped.labels[0].span.line, 4);
    assert_eq!(mapped.labels[0].span.column, 5);
}

/// Native comment classification excludes doc attributes and lookalikes;
///  unlabeled doctests are Rust.
#[test]
fn processors_extract_authored_doc_runs_blocks_and_unlabeled_doctests() {
    let source: String = fixture("docs");
    let virtuals: Vec<VirtualSource> = inputs(source.as_str(), ProcessorLanguage::Rust);
    let names: Vec<&str> = virtuals.iter().map(VirtualSource::filename).collect();
    assert_eq!(
        names,
        [
            "host/1.md",
            "host/4.md",
            "host/14.md",
            "host/16.md",
            "host/4.md/0.rs",
            "host/4.md/0.rs/1.md"
        ]
    );
    assert_eq!(virtuals[0].source(), "First.\r\nSecond.\r\n");
    assert_eq!(
        virtuals[1].source(),
        "Item prose.\r\n```\r\n//! Example.\r\n# let value: u32 = 1;\r\nvalue;\r\n```\r\n"
    );
    assert!(
        virtuals[4]
            .source()
            .starts_with("//! Example.\r\nfn main() {\n")
    );
    assert!(virtuals[1].is_rustdoc());
}

/// Each Rust-fence nesting consumes depth;
///  no processor walks beyond the agreed second Rust embedding.
#[test]
fn processors_bound_nested_doctests_and_preserve_virtual_names() {
    let virtuals: Vec<VirtualSource> = inputs(fixture("nested").as_str(), ProcessorLanguage::Rust);
    let names: Vec<&str> = virtuals.iter().map(VirtualSource::filename).collect();
    assert!(names.contains(&"host/2.md/0.rs"));
    assert!(names.contains(&"host/2.md/0.rs/2.md/0.rs"));
    assert_eq!(
        virtuals
            .iter()
            .filter(|input| return input.language() == ProcessorLanguage::Rust)
            .count(),
        2
    );
}

/// The agreed second Rust embedding is an intentional stop,
///  not an unbounded processor recursion.
#[test]
fn processors_stop_before_a_third_nested_rust_fence() {
    let source: &str = "/// `````rust\n/// //! Level one.\n/// /// ````rust\n/// /// //! Level two.\n/// /// /// ```rust\n/// /// /// //! Level three.\n/// /// /// let deepest: u32 = 1;\n/// /// /// ```\n/// /// fn second() {}\n/// /// ````\n/// fn first() {}\n/// `````\nfn root() {}\n";
    let virtuals = inputs(source, ProcessorLanguage::Rust);
    let mut rust_count: usize = 0;
    let mut prose_at_cap: bool = false;
    for input in &virtuals {
        if input.language() == ProcessorLanguage::Rust {
            rust_count += 1;
            assert!(!input.filename().contains("/0.rs/2.md/0.rs/"));
        } else if input.filename().contains("/0.rs/2.md/0.rs/")
            && input.source().contains("Level three.")
        {
            prose_at_cap = true;
        }
    }
    assert_eq!(rust_count, 2);
    assert!(
        prose_at_cap,
        "authored Markdown still participates at the second Rust level"
    );
}

/// Rust fences under native MDX JSX remain discoverable without evaluating JSX.
#[test]
fn processors_discover_native_mdx_fences() {
    let virtuals: Vec<VirtualSource> = inputs(fixture("mdx").as_str(), ProcessorLanguage::Mdx);
    assert!(
        virtuals
            .iter()
            .any(|input| return input.filename() == "host/0.rs")
    );
}

/// Synthetic main is neither a finding nor a budget line,
///  and file documentation still applies.
#[test]
fn processors_check_authored_counts_and_missing_file_docs_without_synthetic_findings() {
    let virtuals: Vec<VirtualSource> = inputs(
        "```rust\nlet value: u32 = 1;\nvalue;\n```\n",
        ProcessorLanguage::Markdown,
    );
    let settings: RustRuleSettings = RustRuleSettings {
        rustdoc: Some(Severity::Error),
        max_lines: Some(LineBudget {
            max: 1,
            severity: Severity::Warn,
        }),
        ..RustRuleSettings::default()
    };
    let findings: Vec<Diagnostic> = virtuals[0].check_rust(settings).expect("syntax checks");
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].message, "Missing rustdoc on file.");
    assert!(findings[1].message.contains("file has 2 code lines"));
    assert!(findings.iter().all(|item| return item.filename == "host"));
    let generated: usize = virtuals[0]
        .source()
        .find("fn main")
        .expect("synthetic main");
    assert!(
        virtuals[0]
            .project_diagnostic(finding(&virtuals[0], generated, 2))
            .expect("synthetic span")
            .is_none()
    );
}

/// Full semantics without registered virtual context returns the required explicit processing failure.
#[test]
fn processors_refuse_semantic_guessing_even_for_a_valid_virtual_method_call() {
    let virtuals: Vec<VirtualSource> = inputs(
        "```rust\n//! Example.\nlet x = Vec::new();\nx.push(1);\n```\n",
        ProcessorLanguage::Markdown,
    );
    let findings: Vec<Diagnostic> = virtuals[0]
        .check_rust(RustRuleSettings {
            explicit_types: Some(Severity::Warn),
            ..RustRuleSettings::default()
        })
        .expect("explicit failure record");
    assert_eq!(findings.len(), 1);
    assert!(findings[0].processing_failure);
    assert_eq!(findings[0].code, "core/processing-failure");
    assert_eq!(findings[0].filename, "host");
}

/// Adversarial/native failure,
///  fuzz and synthetic-main controls.
#[path = "processors_controls_tests.rs"]
mod controls;
/// Exact Rustdoc margin,
///  blank-line and block-body extraction controls.
#[path = "processors_docs_tests.rs"]
mod docs;
/// Endpoint,
///  Unicode and allowed-depth positive controls.
#[path = "processors_edge_tests.rs"]
mod edges;
/// Bounded randomized native extraction/projection verification.
#[path = "processors_fuzz_tests.rs"]
mod fuzz;
/// Exact prepared doctest text for hidden markers,
///  helpers and Rustdoc identity.
#[path = "processors_prepare_tests.rs"]
mod prepare;
/// Fix projection controls are split to keep each source module inspectable.
#[path = "processors_projection_tests.rs"]
mod projection;
/// Atomic-group validation,
///  whole-line envelopes and container refusal reasons.
#[path = "processors_rewrite_tests.rs"]
mod rewrite;
/// Exact host positions for points,
///  ranges,
///  anchors and rendered refusals.
#[path = "processors_spans_tests.rs"]
mod spans;
