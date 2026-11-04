//! What: Manifest, configuration and source path selection, with every case of
//!       `source-imports.unit.test.ts` for `isNonTestSourcePath`.
//! Why: The planner selects every file it reads with these predicates.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(isNonTestSourcePath({ directory: 'package/module/a', path: 'package/module/a/src/index.ts' })).toBe(true);
//! ```

/// The predicates under test.
use super::{is_non_test_source_path, is_workspace_manifest_path, package_directory};

/// Exactly `package/<category>/<name>/package.json` is a workspace manifest.
#[test]
fn selects_workspace_manifests() {
    assert!(is_workspace_manifest_path(b"package/module/a/package.json"));
    assert!(!is_workspace_manifest_path(b"package/a/package.json"));
    assert!(!is_workspace_manifest_path(
        b"package/module/a/b/package.json"
    ));
    assert!(!is_workspace_manifest_path(
        b"packages/module/a/package.json"
    ));
    assert!(!is_workspace_manifest_path(
        b"package/module/a/package.jsonc"
    ));
    assert!(!is_workspace_manifest_path(b"package.json"));
}

/// The directory is the manifest path without its file name.
#[test]
fn derives_the_package_directory() {
    assert_eq!(
        package_directory(b"package/module/a/package.json"),
        b"package/module/a"
    );
    assert_eq!(
        package_directory(b"package/module/a/README.md"),
        b"package/module/a/README.md"
    );
}

/// Ported: source files under the package `src` directory are accepted.
#[test]
fn accepts_source_files_under_src() {
    assert!(is_non_test_source_path(
        b"package/module/a",
        b"package/module/a/src/index.ts"
    ));
    assert!(is_non_test_source_path(
        b"package/module/a",
        b"package/module/a/src/deep/view.tsx"
    ));
    assert!(is_non_test_source_path(
        b"package/module/a",
        b"package/module/a/src/x.mjs"
    ));
    assert!(is_non_test_source_path(
        b"package/module/a",
        b"package/module/a/src/x.cjs"
    ));
}

/// Ported: tests, other extensions, other directories and sibling prefixes are rejected.
#[test]
fn rejects_tests_other_extensions_directories_and_siblings() {
    assert!(!is_non_test_source_path(
        b"package/module/a",
        b"package/module/a/src/index.unit.test.ts"
    ));
    assert!(!is_non_test_source_path(
        b"package/module/a",
        b"package/module/a/src/README.md"
    ));
    assert!(!is_non_test_source_path(
        b"package/module/a",
        b"package/module/a/test/index.ts"
    ));
    assert!(!is_non_test_source_path(
        b"package/module/a",
        b"package/module/ab/src/index.ts"
    ));
}

/// A `.test.` segment anywhere in the file name marks a test; one in a directory does not.
#[test]
fn reads_the_test_marker_in_the_file_name_only() {
    assert!(!is_non_test_source_path(
        b"p/m/a",
        b"p/m/a/src/x.test.helper.ts"
    ));
    assert!(is_non_test_source_path(
        b"p/m/a",
        b"p/m/a/src/x.test.d/view.ts"
    ));
    assert!(is_non_test_source_path(b"p/m/a", b"p/m/a/src/latest.ts"));
}
