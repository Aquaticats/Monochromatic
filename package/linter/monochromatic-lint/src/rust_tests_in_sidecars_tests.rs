//! What:
//!  Positive and negative controls for the sidecar test rule.
//! Why:
//!  Inline test modules and stray test functions must be reported,
//!  while declarations,
//! sidecar files and non-test configurations must not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Parse fixtures under chosen file names, run the real rule, and inspect findings and ranges.
//! ```

/// Import the actual rule and its helpers through the module under test.
use super::check_tests_in_sidecars;
/// Import the same owned findings returned to the executable.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the retained source context.
use crate::rust_source::RustSource;
/// Import the exact parser interface to validate that fixtures contain real Rust syntax.
use ra_ap_syntax::{Edition, Parse, SourceFile};

/// Parse a fixture under a chosen file name and return the rule's findings.
fn check_as(filename: &str, source: &str) -> Vec<Diagnostic> {
    let parsed: Parse<SourceFile> = SourceFile::parse(source, Edition::CURRENT);
    // The parser must accept the fixture before its lint results count as evidence.
    assert!(parsed.errors().is_empty(), "fixture must parse: {source}");
    let context: RustSource = RustSource::new(String::from(filename), String::from(source));
    return check_tests_in_sidecars(&context, Severity::Error);
}

/// Parse a fixture as an ordinary source file.
fn check(source: &str) -> Vec<Diagnostic> {
    return check_as("src/thing.rs", source);
}

/// An inline module under a test configuration is reported once,
/// at its name,
///  and its test functions are not reported again.
#[test]
fn an_inline_test_module_is_reported_once_at_its_name() {
    let source: &str = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn works() {}\n}\n";
    let findings: Vec<Diagnostic> = check(source);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "rust/tests-in-sidecars");
    assert_eq!(findings[0].severity, Severity::Error);
    assert_eq!(findings[0].labels[0].span.line, 2);
    assert_eq!(findings[0].labels[0].span.column, 5);
    assert_eq!(findings[0].labels[0].span.length, "tests".len());
    let help: String = findings[0].help.clone().expect("help");
    assert!(help.contains("thing_tests.rs"), "{help}");
    assert!(help.contains("#[path = \"thing_tests.rs\"]"), "{help}");
}

/// Every cfg shape that selects test builds counts,
///  and every shape that does not select them is left alone.
#[test]
fn cfg_predicates_are_read_structurally() {
    for predicate in [
        "test",
        "all(test, feature = \"x\")",
        "any(test, debug_assertions)",
        "all(not(not(test)), unix)",
    ] {
        let source: String =
            format!("#[cfg({predicate})]\nmod helpers {{\n    fn help() {{}}\n}}\n");
        assert_eq!(check(source.as_str()).len(), 1, "{predicate}");
    }
    for predicate in [
        "not(test)",
        "feature = \"test\"",
        "all(not(test), unix)",
        "debug_assertions",
        "target_os = \"test\"",
    ] {
        let source: String = format!("#[cfg({predicate})]\nmod real {{\n    fn help() {{}}\n}}\n");
        assert!(check(source.as_str()).is_empty(), "{predicate}");
    }
}

/// A module declaration that loads a file is the sidecar form and is never reported,
///  and an inline module without a test configuration is not a test module.
#[test]
fn declarations_and_plain_inline_modules_are_allowed() {
    for source in [
        "#[cfg(test)]\n#[path = \"thing_tests.rs\"]\nmod tests;\n",
        "#[cfg(test)]\nmod tests;\n",
        "mod plain {\n    fn helper() {}\n}\n",
        "pub mod nested {\n    pub mod deeper {\n        pub fn one() {}\n    }\n}\n",
    ] {
        assert!(check(source).is_empty(), "{source}");
    }
}

/// A test function in a source file is reported at its name,
///  whatever runtime attribute marks it.
#[test]
fn a_test_function_outside_a_sidecar_is_reported() {
    for attribute in [
        "#[test]",
        "#[tokio::test]",
        "#[tokio::test(flavor = \"multi_thread\")]",
    ] {
        let source: String = format!("{attribute}\nasync fn stray() {{}}\n");
        let findings: Vec<Diagnostic> = check(source.as_str());
        assert_eq!(findings.len(), 1, "{attribute}");
        assert_eq!(findings[0].labels[0].span.line, 2, "{attribute}");
        assert_eq!(
            findings[0].labels[0].span.length,
            "stray".len(),
            "{attribute}"
        );
    }
    // An unrelated attribute and a test-only helper function are not test functions.
    assert!(check("#[inline]\nfn quick() {}\n").is_empty());
    assert!(check("#[cfg(test)]\nfn helper() {}\n").is_empty());
    assert!(check("#[testing]\nfn nearly() {}\n").is_empty());
}

/// A test function inside a module that has no test configuration is still reported.
#[test]
fn a_test_function_in_a_plain_inline_module_is_reported() {
    let source: &str = "mod checks {\n    #[test]\n    fn works() {}\n}\n";
    assert_eq!(check(source).len(), 1);
}

/// Only the outermost inline test module is reported,
///  not each module nested in it.
#[test]
fn nested_inline_test_modules_report_only_the_outermost() {
    let source: &str = "#[cfg(test)]\nmod outer {\n    #[cfg(test)]\n    mod inner {\n        #[test]\n        fn works() {}\n    }\n}\n";
    let findings: Vec<Diagnostic> = check(source);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].labels[0].span.line, 2);
}

/// Sidecar file names and any path under a `tests` directory are exempt,
///  with either separator.
#[test]
fn sidecar_paths_are_exempt() {
    let source: &str =
        "#[cfg(test)]\nmod nested {\n    #[test]\n    fn works() {}\n}\n\n#[test]\nfn top() {}\n";
    for filename in [
        "src/thing_tests.rs",
        "src/tests.rs",
        "tests/api.rs",
        "package/tests/support/mod.rs",
        "src\\thing_tests.rs",
        "package\\tests\\api.rs",
        "thing_tests.rs",
    ] {
        assert!(check_as(filename, source).is_empty(), "{filename}");
    }
    // Names that only resemble a sidecar are checked like any source file.
    for filename in [
        "src/thing_test.rs",
        "src/contests.rs",
        "src/tests_helper.rs",
        "src/testsupport/mod.rs",
        "thing.rs",
    ] {
        assert_eq!(check_as(filename, source).len(), 2, "{filename}");
    }
}

/// The finding names the sidecar the file should have,
///  taken from the file's own name.
#[test]
fn the_requested_sidecar_follows_the_file_name() {
    let source: &str = "#[test]\nfn stray() {}\n";
    let findings: Vec<Diagnostic> = check_as("package/x/src/lib.rs", source);
    let help: String = findings[0].help.clone().expect("help");
    assert!(help.contains("`lib_tests.rs`"), "{help}");
    assert_eq!(findings[0].filename, "package/x/src/lib.rs");
}

/// Severity is carried through to the finding.
#[test]
fn severity_is_carried_through() {
    let parsed_source: &str = "#[test]\nfn stray() {}\n";
    let context: RustSource =
        RustSource::new(String::from("src/thing.rs"), String::from(parsed_source));
    let findings: Vec<Diagnostic> = check_tests_in_sidecars(&context, Severity::Warn);
    assert_eq!(findings[0].severity, Severity::Warn);
}
