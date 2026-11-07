//! What:
//!  Controls of the Rust-input router of a build without the `semantic` feature.
//! Why:
//!  That build is the one the Git policy wrapper links:
//!  syntax checks must still run,
//!  and a selected semantic rule must fail loudly instead of passing the file unchecked.
//!  The router's source file is compiled only without the feature,
//!  so these controls are the only code that reaches it,
//!  and `mise run test:syntax-only` runs them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a semantic rule fails and a syntax rule reports', () => { /* engine.check(...) */ });
//! ```
#![cfg(not(feature = "semantic"))]

/// Import the router, the settings it reads and the findings it returns.
use monochromatic_lint::{
    diagnostic::{Diagnostic, Severity},
    rust_file_engine::RustFileEngine,
    rust_manifest::WorkspacePreparation,
    rust_rule_settings::{LineBudget, RustRuleSettings},
};
/// Import the path type that names the checked file.
use std::path::Path;

/// A progress callback that discards its text,
///  because the router never starts a workspace to report on.
fn ignore_progress(_text: String) {}

/// A router as the callers build it.
fn engine() -> RustFileEngine {
    return RustFileEngine::new(WorkspacePreparation::SourceOnly, ignore_progress);
}

/// No workspace is ever loaded in this build.
#[test]
fn no_workspace_is_ever_loaded() {
    assert_eq!(engine().workspace_count(), 0);
}

/// A syntax rule that the settings select reports its finding through the router.
#[test]
fn a_selected_syntax_rule_reports_through_the_router() {
    let settings: RustRuleSettings = RustRuleSettings {
        max_lines: Some(LineBudget {
            severity: Severity::Error,
            max: 1,
        }),
        ..RustRuleSettings::default()
    };
    let found: Vec<Diagnostic> = engine()
        .check(
            Path::new("a.rs"),
            String::from("fn a() {}\nfn b() {}\nfn c() {}\n"),
            String::from("a.rs"),
            settings,
        )
        .expect("a syntax-only selection runs");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code, "rust/max-lines");
    assert_eq!(found[0].filename, "a.rs");
}

/// A source that breaks no selected rule has no finding.
#[test]
fn a_clean_source_has_no_finding() {
    let settings: RustRuleSettings = RustRuleSettings {
        max_lines: Some(LineBudget {
            severity: Severity::Error,
            max: 1,
        }),
        ..RustRuleSettings::default()
    };
    let found: Vec<Diagnostic> = engine()
        .check(
            Path::new("a.rs"),
            String::from("fn a() {}\n"),
            String::from("a.rs"),
            settings,
        )
        .expect("a syntax-only selection runs");
    assert_eq!(found, Vec::<Diagnostic>::new());
}

/// A selected semantic rule is an explicit failure that names the rule and the way forward,
///  and the file is not checked at all.
#[test]
fn a_selected_semantic_rule_fails_instead_of_passing_unchecked() {
    let settings: RustRuleSettings = RustRuleSettings {
        explicit_types: Some(Severity::Error),
        ..RustRuleSettings::default()
    };
    let error = engine()
        .check(
            Path::new("a.rs"),
            String::from("fn a() {}\n"),
            String::from("a.rs"),
            settings,
        )
        .expect_err("the semantic rule cannot run in this build");
    assert!(
        error.message.contains("rust/require-explicit-types"),
        "{}",
        error.message
    );
    assert!(
        error.message.contains("no semantic engine"),
        "{}",
        error.message
    );
}
