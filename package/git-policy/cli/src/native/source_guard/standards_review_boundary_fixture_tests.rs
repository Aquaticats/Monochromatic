//! Filesystem-backed controls of the scan in `standards_review_boundary_tests.rs`:
//! each builds a small crate layout with
//! subdirectories on disk and runs the scan over it,
//! so a forbidden import in a nested production file is shown to be
//! discovered and rejected instead of assumed,
//! and the identity of an exempt file is its relative path.

use super::{native_sources, native_sources_in, violations};
use crate::source_scan_tests::{classified_sources, real_roots, release_root, source_root, write_tree};
use crate::test_support::fixture;

/// What:
///  `fn scan(name: &str, files: &[(&str, &str)]) -> Result<Vec<String>, String>` writes `files` into a fresh fixture
///  directory called `name`,
/// reads it as a crate whose only root is `lib.rs`,
/// and returns the scan's violations.
/// Why:
///  Every control states its layout as data and reads the scan's answer,
/// with the error kept as a value so a control
///  can assert that a layout is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(name: string, files: [string, string][]): string[]  // throws Error
/// ```
fn scan(name: &str, files: &[(&str, &str)]) -> Result<Vec<String>, String> {
    let root = fixture(name);
    write_tree(&root, files);
    let sources = native_sources_in(&root, &[release_root("lib.rs")])?;
    return Ok(violations(sources.as_slice()));
}

/// A forbidden import and a forbidden library name in files below the top directory are discovered and rejected,
/// each
/// under its relative path.
#[test]
fn a_forbidden_import_in_a_nested_production_file_is_discovered_and_rejected() {
    let found: Vec<String> = scan(
        "boundary-scan-nested-production",
        &[
            ("lib.rs", "mod forwarding;\nmod entry;\n"),
            ("forwarding/mod.rs", "mod extra;\n"),
            ("forwarding/extra.rs", "use super::super::standards_review_client::post_decision;\n"),
            ("entry/mod.rs", "mod agent;\n"),
            ("entry/agent.rs", "let agent = ureq::Agent::new_with_defaults();\n"),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(
        found,
        vec![
            String::from("entry/agent.rs names the HTTP library"),
            String::from("forwarding/extra.rs names a service-only module"),
        ]
    );
}

/// A nested file with the name of an exempt file is checked like any other:
/// only the relative path of the exempt file
/// is exempt.
#[test]
fn a_nested_file_with_the_name_of_an_exempt_file_is_still_checked() {
    let found: Vec<String> = scan(
        "boundary-scan-exempt-name",
        &[
            ("lib.rs", "mod standards_review;\nmod nested;\n"),
            ("standards_review/mod.rs", "mod standards_review_client;\nmod standards_review_run;\n"),
            ("standards_review/standards_review_client.rs", "let agent = ureq::Agent::new_with_defaults();\n"),
            ("standards_review/standards_review_run.rs", "use super::standards_review_client::post_decision;\n"),
            ("nested/mod.rs", "mod standards_review_client;\nmod standards_review_run;\n"),
            ("nested/standards_review_client.rs", "let agent = ureq::Agent::new_with_defaults();\n"),
            ("nested/standards_review_run.rs", "use crate::service_review::run;\n"),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(
        found,
        vec![
            String::from("nested/standards_review_client.rs names the HTTP library"),
            String::from("nested/standards_review_run.rs names a service-only module"),
        ]
    );
}

/// A bare `mod` declaration of a service-only module in a module file is registration,
/// not a use of the module,
/// so a
/// directory of modules may declare its own members.
#[test]
fn a_module_declaration_in_a_nested_module_file_is_not_a_use() {
    let found: Vec<String> = scan(
        "boundary-scan-declaration",
        &[
            ("lib.rs", "mod bridge;\n"),
            ("bridge/mod.rs", "mod standards_review_run;\n"),
            ("bridge/standards_review_run.rs", "// the run itself\n"),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, Vec::<String>::new());
}

/// A use of a service-only module in a module file is still a use,
/// even beside the declaration.
#[test]
fn a_use_beside_a_declaration_in_a_nested_module_file_is_rejected() {
    let found: Vec<String> = scan(
        "boundary-scan-use-beside-declaration",
        &[
            ("lib.rs", "mod bridge;\n"),
            ("bridge/mod.rs", "mod standards_review_run;\nuse self::standards_review_run::run;\n"),
            ("bridge/standards_review_run.rs", "// the run itself\n"),
        ],
    )
    .expect("a resolvable layout");
    assert_eq!(found, vec![String::from("bridge/mod.rs names a service-only module")]);
}

/// A module declared in the tree whose file is not there cannot be scanned,
/// so the scan is an error and not a pass.
#[test]
fn a_declared_module_without_a_file_is_an_error() {
    let outcome = scan("boundary-scan-missing-module", &[("lib.rs", "mod forwarding;\n")]);
    assert!(outcome.is_err(), "{outcome:?}");
}

/// A source file that no declaration reaches cannot be classified,
/// so the scan is an error and not a pass.
#[test]
fn a_source_file_no_declaration_reaches_is_an_error() {
    let outcome = scan("boundary-scan-orphan", &[("lib.rs", ""), ("forwarding/forgotten.rs", "use ureq;\n")]);
    let error: String = outcome.expect_err("an unreached file is an error");
    assert!(error.contains("forwarding/forgotten.rs"), "{error}");
}

/// A tree with no source file proves nothing,
/// so the scan is an error and not a pass.
#[test]
fn a_tree_without_source_files_is_an_error() {
    let outcome = scan("boundary-scan-empty", &[("notes.md", "no source here")]);
    assert!(outcome.is_err(), "{outcome:?}");
}

/// A link in the tree may point outside it or nowhere,
/// so the walk is an error and not a pass.
#[cfg(unix)]
#[test]
fn a_link_in_the_tree_is_an_error() {
    let root = fixture("boundary-scan-link");
    write_tree(&root, &[("lib.rs", "")]);
    std::os::unix::fs::symlink(root.join("nowhere.rs"), root.join("dangling.rs")).expect("a dangling link");
    let outcome = native_sources_in(&root, &[release_root("lib.rs")]);
    assert!(outcome.is_err(), "{outcome:?}");
}

/// On the crate's real tree the scan reads exactly the files the module tree reaches,
/// so a source moved into a
/// subdirectory is read at its new path.
#[test]
fn the_real_scan_reads_exactly_the_files_the_module_tree_reaches() {
    let read: Vec<String> = native_sources().into_iter().map(|(path, _)| return path).collect();
    let reached: Vec<String> = classified_sources(&source_root(), &real_roots())
        .expect("the crate's own sources")
        .into_iter()
        .map(|file| return file.path)
        .collect();
    assert_eq!(read, reached);
}
