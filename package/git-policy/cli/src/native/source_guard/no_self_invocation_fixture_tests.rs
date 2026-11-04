//! Filesystem-backed controls of the scan in `no_self_invocation_tests.rs`: each builds a small crate layout with
//! subdirectories on disk and runs the scan over it, so the discovery, the release or test-only classification and the
//! identity of a finding are exercised on nested modules rather than assumed from the flat layout.

use super::rules::Allowed;
use super::{Finding, findings_in, stale, unexplained};
use crate::source_scan_tests::{release_root, write_tree};
use crate::test_support::fixture;

/// A line the `own-executable` rule matches.
const OWN_EXECUTABLE: &str = "let exe = std::env::current_exe();\n";

/// What:
///  `fn scan(name: &str, files: &[(&str, &str)]) -> Result<Vec<Finding>, String>` writes `files` into a fresh fixture
///  directory called `name` and scans it as a crate whose only root is `lib.rs`.
///  `&[(&str, &str)]` is a borrowed list of (relative name, text) pairs (TS: `[string, string][]`).
/// Why:
///  Every control states its layout as data and reads the scan's answer, with the error kept as a value so a control
///  can assert that a layout is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(name: string, files: [string, string][]): Finding[]  // throws Error
/// ```
fn scan(name: &str, files: &[(&str, &str)]) -> Result<Vec<Finding>, String> {
    let root = fixture(name);
    write_tree(&root, files);
    return findings_in(&root, &[release_root("lib.rs")]);
}

/// A forbidden text in a file two directories down is found under its relative path and is rejected.
#[test]
fn a_forbidden_text_in_a_nested_production_file_is_found_and_rejected() {
    let found: Vec<Finding> = scan(
        "self-scan-nested-production",
        &[("lib.rs", "mod engine;\n"), ("engine/mod.rs", "mod launch;\n"), ("engine/launch.rs", OWN_EXECUTABLE)],
    )
    .expect("a resolvable layout");
    assert_eq!(found, vec![(String::from("engine/launch.rs"), "own-executable", 1)]);
    let rejected: Vec<String> = unexplained(found.as_slice(), &[]);
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert!(rejected[0].starts_with("engine/launch.rs:1 [own-executable]"), "{rejected:?}");
}

/// A release file in a subdirectory whose name equals the name of a test-only module elsewhere is still scanned, and
/// the test-only module itself is not.
#[test]
fn a_nested_release_file_with_the_name_of_test_support_is_still_scanned() {
    let found: Vec<Finding> = scan(
        "self-scan-same-name-as-support",
        &[
            ("lib.rs", "#[cfg(test)]\nmod helper;\nmod real;\n"),
            ("helper.rs", OWN_EXECUTABLE),
            ("real/mod.rs", "mod helper;\n"),
            ("real/helper.rs", OWN_EXECUTABLE),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, vec![(String::from("real/helper.rs"), "own-executable", 1)]);
}

/// An exception that names one relative path excuses that file only, even when another directory holds a file of the
/// same name with the same finding.
#[test]
fn an_exception_for_one_path_does_not_excuse_another_file_with_the_same_name() {
    const ONLY_A: &[Allowed] = &[Allowed { file: "a/real_git.rs", rule: "own-executable", why: "identification only" }];
    let found: Vec<Finding> = scan(
        "self-scan-exception-by-path",
        &[
            ("lib.rs", "mod a;\nmod b;\n"),
            ("a/mod.rs", "mod real_git;\n"),
            ("b/mod.rs", "mod real_git;\n"),
            ("a/real_git.rs", OWN_EXECUTABLE),
            ("b/real_git.rs", OWN_EXECUTABLE),
        ],
    )
    .expect("a resolvable layout");
    let rejected: Vec<String> = unexplained(found.as_slice(), ONLY_A);
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    assert!(rejected[0].starts_with("b/real_git.rs:1"), "{rejected:?}");
    assert_eq!(stale(found.as_slice(), ONLY_A), Vec::<String>::new(), "the exception for a/real_git.rs still matches");
}

/// Test-only support in a subdirectory (a gated module and every module it declares) is classified as test-only, while
/// a genuine violation in a release file still fails and an exception that matches nothing is reported as stale.
#[test]
fn nested_test_support_is_skipped_while_genuine_violations_and_stale_exceptions_fail() {
    const GONE: &[Allowed] = &[Allowed { file: "engine/gone.rs", rule: "own-executable", why: "its code was removed" }];
    let found: Vec<Finding> = scan(
        "self-scan-nested-support",
        &[
            ("lib.rs", "mod engine;\n#[cfg(test)]\nmod support;\n"),
            ("engine/mod.rs", "mod launch;\n"),
            ("engine/launch.rs", OWN_EXECUTABLE),
            ("support/mod.rs", "mod inner;\n"),
            ("support/inner.rs", OWN_EXECUTABLE),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, vec![(String::from("engine/launch.rs"), "own-executable", 1)], "support/inner.rs is test-only");
    assert_eq!(unexplained(found.as_slice(), GONE).len(), 1, "the genuine violation is not excused");
    let stale_entries: Vec<String> = stale(found.as_slice(), GONE);
    assert_eq!(stale_entries.len(), 1, "{stale_entries:?}");
    assert!(stale_entries[0].starts_with("engine/gone.rs [own-executable]"), "{stale_entries:?}");
}

/// A module declared in the tree whose file is not there cannot be scanned, so the scan is an error and not a pass.
#[test]
fn a_declared_module_without_a_file_is_an_error() {
    let outcome = scan("self-scan-missing-module", &[("lib.rs", "mod engine;\n")]);
    assert!(outcome.is_err(), "{outcome:?}");
}

/// A source file that no declaration reaches cannot be classified, so the scan is an error and not a pass.
#[test]
fn a_source_file_no_declaration_reaches_is_an_error() {
    let outcome = scan("self-scan-orphan", &[("lib.rs", ""), ("engine/forgotten.rs", OWN_EXECUTABLE)]);
    let error: String = outcome.expect_err("an unreached file is an error");
    assert!(error.contains("engine/forgotten.rs"), "{error}");
}

/// A tree with no source file proves nothing, so the scan is an error and not a pass.
#[test]
fn a_tree_without_source_files_is_an_error() {
    let outcome = scan("self-scan-empty", &[("notes.md", "no source here")]);
    assert!(outcome.is_err(), "{outcome:?}");
}

/// A link in the tree may point outside it or nowhere, so the walk is an error and not a pass.
#[cfg(unix)]
#[test]
fn a_link_in_the_tree_is_an_error() {
    let root = fixture("self-scan-link");
    write_tree(&root, &[("lib.rs", "")]);
    std::os::unix::fs::symlink(root.join("nowhere.rs"), root.join("dangling.rs")).expect("a dangling link");
    let outcome = findings_in(&root, &[release_root("lib.rs")]);
    assert!(outcome.is_err(), "{outcome:?}");
}
