//! What:
//!  Structured policy oracles and arbitrary-source bounds for explicit Rust annotations.
//! Why:
//!  Valid counterexamples must be reached on every draw,
//!  not lost among malformed parser inputs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Verify known violation counts, try raw source, then restore a known passing overlay.
//! ```

/// Import the fixed in-memory workspace owner.
use crate::semantic_fixture::semantic_session;
/// Import the actual rule result model and production session.
use monochromatic_lint::diagnostic::{Diagnostic, Severity};
use monochromatic_lint::rust_semantic_session::RustSemanticSession;
/// Use a native absolute path matching the initialized virtual file.
use std::path::Path;

/// Fixed declarations give generated calls real generic and nongeneric identities.
const PREFIX: &str = "struct Generic; impl Generic { fn parse<T>(&self, value: T) -> T { return value; } } struct Plain; impl Plain { fn parse(&self, value: u16) -> u16 { return value; } } fn named(value: u16) -> u16 { return value; } fn main() {\n";

/// Fragments and independently counted policy violations,
///  not counts derived from the checker.
const CASES: &[(&str, usize)] = &[
    ("let value: u16 = Generic.parse::<u16>(1_u16);\n", 0),
    ("let value: u16 = Generic.parse(1_u16);\n", 1),
    ("let value: u16 = Plain.parse(1_u16);\n", 0),
    ("let value = 1_u16;\n", 1),
    ("let value: _ = 1_u16;\n", 1),
    ("let value: _ = named;\n", 0),
    ("let value: fn(u16) -> u16 = named;\n", 0),
    ("let value: _ = named as fn(u16) -> u16;\n", 1),
    ("let value: u16 = Generic.parse::<_>(1_u16);\n", 1),
    ("let value: &_ = &named;\n", 0),
    ("let value: _ = &named;\n", 1),
];

/// Emit a bounded authored program with a known expected count.
pub fn generated_explicit_source(data: &[u8]) -> (String, usize) {
    let mut source: String = String::from(PREFIX);
    let mut expected: usize = 0;
    for byte in data.iter().take(8) {
        let index: usize = usize::from(*byte) % CASES.len();
        let (fragment, count): (&str, usize) = CASES[index];
        source.push_str(fragment);
        expected += count;
    }
    source.push_str("}\n");
    return (source, expected);
}

/// Validate all emitted source ranges,
///  including reported failures on malformed or unresolved input.
fn check_bounds(source: &str, findings: &[Diagnostic]) {
    for finding in findings {
        assert!(finding.fix.is_none());
        for label in &finding.labels {
            assert!(label.span.offset <= source.len());
            assert!(label.span.length <= source.len() - label.span.offset);
            assert!(source.is_char_boundary(label.span.offset));
            assert!(source.is_char_boundary(label.span.offset + label.span.length));
            assert!(label.span.line >= 1);
            assert!(label.span.column >= 1);
        }
    }
}

/// Exercise the production checker and source-overlay invalidation for every input.
pub fn check_explicit_types(data: &[u8]) {
    let mut session: RustSemanticSession = semantic_session();
    let (source, expected): (String, usize) = generated_explicit_source(data);
    let findings: Vec<Diagnostic> = session
        .check_file(
            Path::new("/main.rs"),
            source.as_str(),
            "input.rs",
            Severity::Error,
        )
        .expect("generated source query completes");
    assert_eq!(findings.len(), expected, "{source}\n{findings:?}");
    for finding in &findings {
        assert!(
            !finding.processing_failure,
            "generated source must have semantic coverage: {finding:?}"
        );
    }
    check_bounds(source.as_str(), &findings);
    // Raw strings go only through the production overlay, never through the fixture's directive interpreter.
    if let Ok(raw) = std::str::from_utf8(data) {
        let raw_findings: Vec<Diagnostic> = session
            .check_file(Path::new("/main.rs"), raw, "input.rs", Severity::Warn)
            .expect("raw source must not panic the semantic backend");
        check_bounds(raw, &raw_findings);
    }
    let restored: Vec<Diagnostic> = session
        .check_file(
            Path::new("/main.rs"),
            "fn main() {}",
            "input.rs",
            Severity::Error,
        )
        .expect("restoring known source works");
    assert!(
        restored.is_empty(),
        "previous overlays must not leak findings"
    );
}

/// Reach every generator branch before coverage-guided execution is trusted.
#[test]
fn explicit_type_generator_reaches_all_controls() {
    let counts: [usize; 11] = [0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1];
    for (index, count) in counts.iter().enumerate() {
        let input: [u8; 1] = [index as u8];
        let (_, expected): (String, usize) = generated_explicit_source(&input);
        assert_eq!(expected, *count);
        check_explicit_types(&input);
    }
    check_explicit_types(&[]);
    check_explicit_types(&[255, 254]);
    // Fixture-looking tokens remain ordinary source bytes and must not create new files or project directives.
    check_explicit_types(b"//- /other.rs\nfn main() {}\n//- minicore: unknown_feature\n$0");
}
