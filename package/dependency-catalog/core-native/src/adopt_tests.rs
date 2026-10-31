//! What:
//! Unit tests of adoption planning and per-crate resolution:
//! requirement forms,
//! git agreement,
//! renames,
//! named-catalog selections and path dependencies.
//! Why:
//! Each conflict the planner can report,
//! and each way it can succeed,
//! has a test with the issue location.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // adopt.unit.test.ts
//! describe('planAdoption', () => { it('keeps the highest caret base as the floor', () => { ... }); });
//! ```

/// Imports `crate::crate_info::ManifestFile`.
use crate::crate_info::ManifestFile;
/// Imports `crate::error::Error`.
use crate::error::Error;
/// Imports `crate::sync::SyncOutput`.
use crate::sync::SyncOutput;
/// Imports `crate::sync_test_support::{crate_with_version, issues, krate, run, run_ok, text}`.
use crate::sync_test_support::{crate_with_version, issues, krate, run, run_ok, text};

/// The problems of a failed sync as `location: problem` text.
fn problems(catalog: &[&str], manifests: &[ManifestFile]) -> Vec<String> {
    return match run(catalog, manifests) {
        Err(Error::Sync { issues: found }) => found.into_iter().map(|issue| return format!("{}: {}", issue.location, issue.problem)).collect(),
        other => panic!("expected a sync error, got {other:?}"),
    };
}

/// Caret,
/// tilde and floor literals adopt the highest base version at its written precision.
#[test]
fn adopts_the_highest_base_across_requirement_forms() {
    let output: SyncOutput = run_ok(
        &["[catalog]"],
        &[
            krate("package/cli/a", &["[dependencies]", "x = \"^1.2\"", "y = \"~1.2.3\"", "z = \">=2\""]),
            krate("package/cli/b", &["[dependencies]", "x = \"1.2.5\"", "y = \"1.2.3\"", "z = \"2.1\""]),
        ],
    );
    assert_eq!(output.catalog.expected, text(&["[catalog]", "x = \">=1.2.5\"", "y = \">=1.2.3\"", "z = \">=2.1\""]));
}

/// A literal whose requirement the shared floor would violate is reported with a named-catalog fix.
#[test]
fn reports_literals_the_shared_floor_would_violate() {
    let found: Vec<String> = problems(
        &["[catalog]"],
        &[krate("package/cli/a", &["[dependencies]", "x = \"1.2\""]), krate("package/cli/b", &["[dependencies]", "x = \"2.0\""])],
    );
    assert_eq!(
        found,
        vec![
            "dependencies.x: requires \"1.2\", which the shared floor >=2.0 for x does not allow",
        ]
    );
}

/// Multi-comparator and wildcard literals cannot become floors.
#[test]
fn reports_unreadable_requirements() {
    let found: Vec<String> = problems(
        &["[catalog]"],
        &[krate("package/cli/a", &["[dependencies]", "x = \"*\"", "y = \">=1, <2\"", "z = \"=1.0.0\""])],
    );
    assert_eq!(found.len(), 3);
    assert!(found[0].starts_with("dependencies.x: requirement \"*\" is not one comparator the catalog can adopt"));
    assert!(found[1].starts_with("dependencies.y: requirement"));
    assert!(found[2].starts_with("dependencies.z: exact pin \"=1.0.0\" cannot become a `>=` floor in [catalog]"));
}

/// Renames must agree across crates and with the existing entry.
#[test]
fn reports_conflicting_renames() {
    assert_eq!(
        issues(
            &["[catalog]", "x = { version = \">=1\", package = \"real\" }"],
            &[krate("package/cli/a", &["[dependencies]", "x = \"1\""])]
        ),
        vec!["package/cli/a/Cargo.toml x".to_string()]
    );
}

/// A git pin that matches the existing entry is adopted as already present.
#[test]
fn accepts_git_pins_matching_the_catalog() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "h = { git = \"https://x\", rev = \"1\" }"],
        &[krate("package/cli/a", &["[dependencies]", "h = { git = \"https://x\", rev = \"1\" }"])],
    );
    assert!(output.catalog.changes.is_empty());
    assert!(manifest_has_inherited(&output));
}

/// Whether the first manifest now inherits `h`.
fn manifest_has_inherited(output: &SyncOutput) -> bool {
    return output.manifests[0].expected.contains("h = { workspace = true }");
}

/// A git pin that differs from the existing entry is reported.
#[test]
fn rejects_git_pins_differing_from_the_catalog() {
    assert_eq!(
        issues(
            &["[catalog]", "h = { git = \"https://x\", rev = \"1\" }"],
            &[krate("package/cli/a", &["[dependencies]", "h = { git = \"https://x\", rev = \"2\" }"])]
        ),
        vec!["package/cli/a/Cargo.toml h".to_string()]
    );
}

/// A literal that selects a named catalog must match the named entry.
#[test]
fn checks_selected_literals_against_the_named_entry() {
    let selection: [&str; 2] = ["[package.metadata.rust-catalog.catalogs]", "w = \"legacy\""];
    let mismatch: Vec<String> = problems(
        &["[catalog]", "", "[catalogs.legacy]", "w = \"0.58\""],
        &[krate("package/cli/a", &[selection[0], selection[1], "[dependencies]", "w = \"0.62\""])],
    );
    assert_eq!(mismatch, vec!["dependencies.w: literal does not match the selected entry in [catalogs.legacy]"]);
    let missing: Vec<String> = problems(
        &["[catalog]"],
        &[krate("package/cli/a", &[selection[0], selection[1], "[dependencies]", "w = \"0.62\""])],
    );
    assert_eq!(missing, vec!["dependencies.w: selects catalog \"legacy\", which has no w entry"]);
    let git_named: Vec<String> = problems(
        &["[catalog]", "", "[catalogs.legacy]", "w = \"0.58\""],
        &[krate("package/cli/a", &[selection[0], selection[1], "[dependencies]", "w = { git = \"https://x\", rev = \"1\" }"])],
    );
    assert_eq!(git_named.len(), 1);
}

/// A caret literal inside the named range is accepted and rewritten.
#[test]
fn accepts_selected_literals_inside_the_named_range() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "", "[catalogs.legacy]", "w = \"0.58\""],
        &[krate("package/cli/a", &["[package.metadata.rust-catalog.catalogs]", "w = \"legacy\"", "[dependencies]", "w = \"0.58\""])],
    );
    assert!(output.manifests[0].expected.contains("w = { workspace = true }"));
    assert!(output.manifests[0].expected.contains("[workspace.dependencies]\nw = \"0.58\"\n"));
}

/// A `workspace = true` entry resolves through a selected catalog or reports what is missing.
#[test]
fn reports_missing_catalog_entries_for_inherited_sites() {
    let default_missing: Vec<String> = problems(&["[catalog]"], &[krate("package/cli/a", &["[dependencies]", "x = { workspace = true }"])]);
    assert_eq!(default_missing, vec!["dependencies.x: uses `workspace = true` but rust-catalog.toml [catalog] has no x entry"]);
    let named_missing: Vec<String> = problems(
        &["[catalog]", "", "[catalogs.pinned]", "y = \"=1.0.0\""],
        &[krate("package/cli/a", &["[package.metadata.rust-catalog.catalogs]", "x = \"pinned\"", "[dependencies]", "x = { workspace = true }"])],
    );
    assert_eq!(named_missing, vec!["dependencies.x: selects catalog \"pinned\", which has no x entry"]);
    let unknown_catalog: Vec<String> = problems(
        &["[catalog]"],
        &[krate("package/cli/a", &["[package.metadata.rust-catalog.catalogs]", "x = \"nope\"", "[dependencies]", "x = { workspace = true }"])],
    );
    assert_eq!(unknown_catalog, vec!["dependencies.x: selects catalog \"nope\", which has no x entry"]);
}

/// A path dependency must point at a first-party crate that declares a version.
#[test]
fn reports_path_dependencies_without_a_versioned_target() {
    let found: Vec<String> = problems(
        &["[catalog]"],
        &[
            crate_with_version("package/lib/unversioned", &[], ""),
            krate("package/cli/a", &["[dependencies]", "u = { path = \"../../lib/unversioned\" }", "m = { path = \"../../lib/missing\" }"]),
        ],
    );
    assert_eq!(found.len(), 2);
    assert!(found[0].starts_with("dependencies.u: path \"../../lib/unversioned\" does not point at a first-party crate with a package.version"));
    assert!(found[1].starts_with("dependencies.m: path"));
}

/// Default features of path sites do not count toward the generated entry's choice.
#[test]
fn ignores_path_sites_when_choosing_default_features() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "x = \">=1\""],
        &[
            crate_with_version("package/lib/x", &[], "0.3.0"),
            krate(
                "package/cli/a",
                &["[dependencies]", "x = { workspace = true, default-features = false }", "", "[target.'cfg(unix)'.dependencies]", "y = { path = \"../../lib/x\" }"],
            ),
        ],
    );
    assert!(output.manifests[1].expected.contains("x = { version = \">=1\", default-features = false }"));
    assert!(output.manifests[1].expected.contains("y = { path = \"../../lib/x\", version = \"=0.3.0\" }"));
}
