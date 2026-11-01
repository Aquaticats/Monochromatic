//! What:
//!  Surface-annotation controls,
//!  independent of the later semantic checks.
//! Why:
//!  A present underscore is not proof of a complete type,
//!  and syntax-only checks must not claim otherwise.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Verify only missing declaration syntax; inferred holes and generic calls have separate semantic controls.
//! ```

/// Import the production helper and its real source/finding models.
use super::check_declaration_annotations;
/// Import owned diagnostics and the caller-selected severity.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the parser-owning source context.
use crate::rust_source::RustSource;

/// Parse and check one source without replacing the implementation with a test double.
fn check(source: &str) -> Vec<Diagnostic> {
    // String owns the fixture bytes; &str only borrows them. The source context needs ownership.
    let context: RustSource = RustSource::new(String::from("input.rs"), String::from(source));
    // Lend the context without consuming it; the diagnostics returned by the rule own their data.
    return check_declaration_annotations(&context, Severity::Warn);
}

/// Every let-statement shape uses the annotation after its whole pattern.
#[test]
fn flags_missing_types_on_simple_and_destructuring_bindings() {
    let source: &str = "fn main() { let x = 1; let mut y = 2; let (a, b) = pair; let Some(z) = value else { return; }; let later; }";
    let findings: Vec<Diagnostic> = check(source);
    assert_eq!(findings.len(), 5);
    for finding in &findings {
        assert_eq!(finding.code, "rust/require-explicit-types");
        assert_eq!(finding.severity, Severity::Warn);
        assert_eq!(finding.filename, "input.rs");
        assert_eq!(
            finding.message,
            "Missing explicit type annotation on variable binding."
        );
        assert!(finding.fix.is_none());
        assert!(finding.help.is_some());
    }
}

/// Written types satisfy only the presence check;
///  inference holes still require semantic validation.
#[test]
fn leaves_present_annotations_for_semantic_validation() {
    let source: &str = "fn main() { let port: u16 = input.parse::<u16>()?; let (a, b): (u8, u16) = pair; let Some(value): Option<u16> = item else { return; }; let inferred: _ = item; let partial: Vec<_> = values; }";
    assert!(check(source).is_empty());
}

/// Closure parameters and results are independent annotation requirements,
///  whether or not a closure is otherwise acceptable.
#[test]
fn checks_each_anonymous_parameter_and_return_annotation() {
    let findings: Vec<Diagnostic> = check("fn main() { call(|first, second: u16| first); }");
    assert_eq!(findings.len(), 2);
    assert_eq!(
        findings[0].message,
        "Missing explicit type annotation on anonymous-function parameter."
    );
    assert_eq!(
        findings[1].message,
        "Missing explicit return-type annotation on anonymous function."
    );
    assert!(
        check("fn main() { call(|value: u16| -> u16 { return value; }); call(|| -> () {}); }")
            .is_empty()
    );
    assert_eq!(check("fn main() { call(|| {}); }").len(), 1);
}

/// Visiting a let statement must not skip the anonymous function nested in its initializer.
#[test]
fn continues_into_nested_declarations() {
    let findings: Vec<Diagnostic> = check("fn main() { let callback = |value| value; }");
    assert_eq!(findings.len(), 3);
    let typed_binding: Vec<Diagnostic> = check("fn main() { let callback: _ = |value| value; }");
    assert_eq!(typed_binding.len(), 2);
    assert_eq!(
        check("fn main() { call(move || { let inner = 1; }); }").len(),
        2
    );
}

/// Rust has no annotation slot on for,
///  if-let,
///  while-let or match binding patterns.
#[test]
fn does_not_demand_unavailable_pattern_annotation_syntax() {
    let source: &str = "fn main() { for value in values {} if let Some(value) = optional {} while let Some(value) = optional {} match optional { Some(value) => {}, None => {} } }";
    assert!(check(source).is_empty());
    // Ordinary named functions declare unit by default, not by result-type inference.
    assert!(check("fn named() {} struct Item; impl Item { fn method(&self) {} }").is_empty());
}

/// Comment/string contents are not declarations,
///  and recovery nodes without patterns are not bindings.
#[test]
fn ignores_text_and_patternless_recovery_nodes() {
    let source: &str =
        "fn main() { /* let fake = || {}; */ use_text(\"let fake = |x| x;\"); let; }";
    assert!(check(source).is_empty());
    assert!(check("").is_empty());
}

/// Register the complete rule ID without inventing an exemption or enabling it by default.
#[test]
fn accepts_only_declared_rule_settings() {
    assert!(crate::configuration::parse_configuration(
        r#"[{"files":["**/*.rs"],"rules":{"rust/require-explicit-types":{"severity":"error"}}}]"#
    ).is_ok());
    assert!(crate::configuration::parse_configuration(
        r#"[{"files":["**/*.rs"],"rules":{"rust/require-explicit-types":{"severity":"off","allowInferred":true}}}]"#
    ).is_err());
}

/// Underlines refer to the binding's UTF-8 bytes,
///  not character or UTF-16 indexes.
#[test]
fn anchors_findings_to_the_whole_authored_pattern() {
    let source: &str = "fn main() {\n    /* 🚀 */ let (left, right) = value;\n}\n";
    let findings: Vec<Diagnostic> = check(source);
    assert_eq!(findings.len(), 1);
    // find returns an optional byte offset; the fixture marker must exist for this control to count.
    let offset: usize = source.find("(left, right)").expect("pattern marker");
    assert_eq!(findings[0].labels[0].span.offset, offset);
    assert_eq!(findings[0].labels[0].span.length, "(left, right)".len());
    assert_eq!(findings[0].labels[0].span.line, 2);
    assert_eq!(
        findings[0].labels[0].span.column,
        "    /* 🚀 */ let ".len() + 1
    );
}
