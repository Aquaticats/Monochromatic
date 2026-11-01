//! What:
//!  Structured and malformed-source invariants for the sidecar test rule.
//! Why:
//!  A raw-only fuzzer can spend its entire budget in invalid syntax without reaching real test modules and functions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Every draw exercises a known finding count plus arbitrary source and diagnostic bounds.
//! ```

/// Import the production rule and owned source/finding models,
///  not a second implementation.
use monochromatic_lint::diagnostic::{Diagnostic, Severity, Span};
/// Import the parser-owning input shared by all Rust rules.
use monochromatic_lint::rust_source::RustSource;
/// Import the actual sidecar test checker used by the linter.
use monochromatic_lint::rust_tests_in_sidecars::check_tests_in_sidecars;

/// What:
///  Grammar fragments paired with independently known finding counts.
/// Why:
///  Negative controls for test-looking text,
///  a module declaration,
///  a negated `cfg` and a key-value `cfg`
///  keep lexical approximations from passing.
/// &[...] borrows this fixed data;
///  a Vec would allocate,
///  and a fixed array type would repeat its length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cases: readonly [string, number][] = [...];
/// ```
const CASES: &[(&str, usize)] = &[
    ("#[test]\nfn plain() {}\n", 1),
    ("#[tokio::test]\nasync fn runtime() {}\n", 1),
    (
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn nested() {}\n}\n",
        1,
    ),
    (
        "#[cfg(all(test, unix))]\nmod platform {\n    mod inner {\n        #[test]\n        fn deep() {}\n    }\n}\n",
        1,
    ),
    ("#[cfg(test)]\nmod tests;\n", 0),
    (
        "#[cfg(not(test))]\nmod release {\n    fn helper() {}\n}\n",
        0,
    ),
    (
        "#[cfg(feature = \"test\")]\nmod gated {\n    fn helper() {}\n}\n",
        0,
    ),
    ("fn helper() {}\n", 0),
    (
        "const TEXT: &str = \"#[test] fn quoted() {}\"; // #[test]\n",
        0,
    ),
];

/// An inline test module appended to generated source so the sidecar rule has one known finding.
const INLINE_TEST_MODULE: &str =
    "\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn generated() {}\n}\n";

/// Build at most 64 grammar fragments while retaining the expected finding count.
pub fn generated_source(data: &[u8]) -> (String, usize) {
    // String owns growable text; &str would only borrow it. usize matches byte and collection indexes.
    let mut source: String = String::from("fn named(value: u16) -> u16 { return value; }\n");
    let mut expected: usize = 0;
    for byte in data.iter().take(64) {
        // Copy the borrowed byte's value and convert it to the platform-sized case index.
        let index: usize = usize::from(*byte) % CASES.len();
        let (fragment, added): (&str, usize) = CASES[index];
        source.push_str(fragment);
        expected += added;
    }
    return (source, expected);
}

/// Check the sidecar rule's findings and byte ranges for one source under one file name.
fn check_sidecar_source(source: &str, filename: &str) -> usize {
    let context: RustSource = RustSource::new(String::from(filename), String::from(source));
    let findings: Vec<Diagnostic> = check_tests_in_sidecars(&context, Severity::Error);
    for finding in &findings {
        assert_eq!(finding.code, "rust/tests-in-sidecars");
        assert_eq!(finding.filename, filename);
        assert_eq!(finding.severity, Severity::Error);
        assert!(finding.fix.is_none());
        assert_eq!(finding.labels.len(), 1);
        let span: &Span = &finding.labels[0].span;
        assert!(span.offset <= source.len());
        assert!(span.length <= source.len() - span.offset);
        assert!(source.is_char_boundary(span.offset));
        assert!(source.is_char_boundary(span.offset + span.length));
    }
    return findings.len();
}

/// Exercise generated syntax on every draw,
///  including non-UTF-8 byte inputs.
pub fn check_rust_style(data: &[u8]) {
    let (source, expected): (String, usize) = generated_source(data);
    // Borrow the generated String as a string slice for the real parser.
    assert_eq!(check_sidecar_source(source.as_str(), "fuzz.rs"), expected);
    // A sidecar file name or a `tests` directory exempts every generated fragment.
    assert_eq!(check_sidecar_source(source.as_str(), "fuzz_tests.rs"), 0);
    assert_eq!(check_sidecar_source(source.as_str(), "tests/fuzz.rs"), 0);
    // One appended inline test module adds exactly one finding to the generated count.
    let with_module: String = format!("{source}{INLINE_TEST_MODULE}");
    assert_eq!(
        check_sidecar_source(with_module.as_str(), "fuzz.rs"),
        expected + 1
    );
    assert_eq!(
        check_sidecar_source(with_module.as_str(), "fuzz_tests.rs"),
        0
    );
    // What: UTF-8 decoding returns Ok(text) or Err(details), rather than throwing.
    // Why: Only valid UTF-8 is a Rust source string; every rejected byte draw still ran generated syntax.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = decodeUtf8Strict(data); if (text !== undefined) checkSidecarSource(text, 'fuzz.rs');
    // ```
    if let Ok(raw) = std::str::from_utf8(data) {
        check_sidecar_source(raw, "fuzz.rs");
        assert_eq!(check_sidecar_source(raw, "fuzz_tests.rs"), 0);
    }
}

/// Verify each generator branch and both empty/invalid-byte inputs before fuzzing starts.
#[test]
fn every_rust_style_generator_branch_has_a_positive_or_negative_control() {
    // Each position independently states how many findings its grammar fragment holds.
    let expected: [usize; 9] = [1, 1, 1, 1, 0, 0, 0, 0, 0];
    assert_eq!(CASES.len(), expected.len());
    for (index, expected_count) in expected.iter().enumerate() {
        // The bounded case index fits in a byte by construction.
        let bytes: [u8; 1] = [index as u8];
        let (source, count): (String, usize) = generated_source(&bytes);
        // Read the borrowed count without taking ownership of the array slot.
        assert_eq!(count, *expected_count);
        assert_eq!(check_sidecar_source(source.as_str(), "fuzz.rs"), count);
    }
    check_rust_style(&[]);
    check_rust_style(&[255, 254, 253]);
    // Borrow a byte literal containing Unicode before a real test function.
    check_rust_style("/* 🚀 */\n#[test]\nfn after_unicode() {}\n".as_bytes());
    assert_eq!(
        check_sidecar_source("/* 🚀 */\n#[test]\nfn after_unicode() {}\n", "fuzz.rs"),
        1
    );
}

/// The sidecar rule has a positive control, a sidecar-name negative control and a no-test negative control.
#[test]
fn the_sidecar_rule_has_positive_and_negative_controls() {
    assert_eq!(check_sidecar_source(INLINE_TEST_MODULE, "fuzz.rs"), 1);
    assert_eq!(check_sidecar_source(INLINE_TEST_MODULE, "fuzz_tests.rs"), 0);
    assert_eq!(check_sidecar_source("fn main() {}\n", "fuzz.rs"), 0);
    assert_eq!(
        check_sidecar_source("#[test]\nfn stray() {}\n", "fuzz.rs"),
        1
    );
}
