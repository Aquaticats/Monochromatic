//! What:
//!  Run selected syntax-only Rust rules over one shared parse.
//! Why:
//!  Disabling semantic checking must avoid Cargo/toolchain initialization,
//!  not merely hide its diagnostics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Dispatch only selected checks; workspace-backed annotation checking is added by the semantic session.
//! ```

/// Import owned findings and typed execution settings.
use crate::diagnostic::Diagnostic;
/// Import the line budget and per-rule settings that decide which checks run.
use crate::rust_rule_settings::{LineBudget, RustRuleSettings};
/// Import the code-line and documentation checks that run over each Rust file's syntax tree.
use crate::rust_rules::{check_max_lines, check_rustdoc};
/// Borrow the already parsed source rather than reparsing per rule.
use crate::rust_source::RustSource;
/// Import the check that requires unit tests to live in sidecar files.
use crate::rust_tests_in_sidecars::check_tests_in_sidecars;

/// Preserve fixed shipped-rule ordering while leaving absent/off rules unexecuted.
pub fn check_syntax_rules(source: &RustSource, settings: &RustRuleSettings) -> Vec<Diagnostic> {
    let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
    if let Some(budget) = settings.max_lines {
        let selected: LineBudget = budget;
        findings.extend(check_max_lines(source, selected.max, selected.severity));
    }
    if let Some(severity) = settings.rustdoc {
        findings.extend(check_rustdoc(source, severity));
    }
    if let Some(severity) = settings.tests_in_sidecars {
        findings.extend(check_tests_in_sidecars(source, severity));
    }
    return findings;
}
