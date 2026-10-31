//! What:
//! Unit tests of pruning and of every issue the pure sync can report,
//! ported from the TypeScript
//! `sync.unit.test.ts`.
//! Why:
//! Every issue is asserted by file,
//! location and order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync.unit.test.ts
//! describe('syncRustCatalog', () => { it('adopts literals across crates ...', () => { ... }); });
//! ```

/// Imports `crate::change::Change`.
use crate::change::Change;
/// Imports `crate::crate_info::ManifestFile`.
use crate::crate_info::ManifestFile;
/// Imports `crate::error::Error`.
use crate::error::Error;
/// Imports `crate::sync::SyncOutput`.
use crate::sync::SyncOutput;
/// Imports `crate::sync_test_support::{issues, krate, manifest_text, run, run_ok, text}`.
use crate::sync_test_support::{issues, krate, manifest_text, run, run_ok, text};

/// Unused entries are pruned with their comments when `catalogPrune` is on,
/// and warned about when it is off.
#[test]
fn prunes_or_warns_about_unused_entries() {
    let entries: [&str; 4] = ["[catalog]", "anyhow = \">=1\"", "# Rationale for gone.", "gone = \">=1\""];
    let manifests: [ManifestFile; 1] = [krate("package/cli/a", &["[dependencies]", "anyhow = { workspace = true }"])];
    let mut pruning: Vec<&str> = vec!["catalogPrune = true"];
    pruning.extend(entries);
    let pruned: SyncOutput = run_ok(&pruning, &manifests);
    assert_eq!(pruned.catalog.expected, text(&["catalogPrune = true", "[catalog]", "anyhow = \">=1\""]));
    assert!(pruned.catalog.changes.contains(&Change::CatalogEntryPruned { path: "catalog.gone".to_string() }));
    let kept: SyncOutput = run_ok(&entries, &manifests);
    assert_eq!(kept.catalog.expected, text(&entries));
    assert_eq!(
        kept.warnings,
        vec!["rust-catalog.toml: catalog.gone is used by no crate; set catalogPrune = true to remove unused entries on sync".to_string()]
    );
}

/// Unused entries of a named catalog are found too.
#[test]
fn prunes_unused_named_catalog_entries() {
    let output: SyncOutput = run_ok(
        &["catalogPrune = true", "[catalog]", "", "[catalogs.pinned]", "salsa = \"=0.1.0\""],
        &[krate("package/cli/a", &[])],
    );
    assert_eq!(output.catalog.expected, text(&["catalogPrune = true", "[catalog]", "", "[catalogs.pinned]"]));
}

/// Exact pins without a selection and incompatible majors are refused.
#[test]
fn refuses_exact_pins_and_incompatible_majors() {
    let found: Vec<String> = issues(
        &["[catalog]"],
        &[
            krate("package/cli/a", &["[dev-dependencies]", "i-slint-backend-testing = \"=1.18.1\"", "windows = \"0.58\""]),
            krate("package/cli/b", &["[dependencies]", "windows = \"0.62\""]),
        ],
    );
    assert_eq!(
        found,
        vec![
            "package/cli/a/Cargo.toml dev-dependencies.i-slint-backend-testing".to_string(),
            "package/cli/a/Cargo.toml dev-dependencies.windows".to_string()
        ]
    );
}

/// Missing entries,
/// bad paths,
/// stale selections and unsupported forms are reported together.
#[test]
fn reports_every_problem_together() {
    let found: Vec<String> = issues(
        &["[catalog]", "", "[catalogs.pinned]", "smithay = \"=0.7.0\""],
        &[krate(
            "package/cli/a",
            &[
                "[package.metadata.rust-catalog.catalogs]",
                "smithay = \"pinned\"",
                "tokio = \"missing\"",
                "",
                "[dependencies]",
                "anyhow = { workspace = true }",
                "outside = { path = \"../../../../elsewhere\" }",
                "pinned-dep = { workspace = true, version = \"1\" }",
                "other-registry = { version = \"1\", registry = \"corp\" }",
                "unpinned = { git = \"https://example.com/u\" }",
                "tokio = { workspace = true }",
            ],
        )],
    );
    assert_eq!(
        found,
        vec![
            "package/cli/a/Cargo.toml dependencies.pinned-dep".to_string(),
            "package/cli/a/Cargo.toml dependencies.other-registry".to_string(),
            "package/cli/a/Cargo.toml dependencies.unpinned".to_string(),
            "package/cli/a/Cargo.toml dependencies.anyhow".to_string(),
            "package/cli/a/Cargo.toml dependencies.outside".to_string(),
            "package/cli/a/Cargo.toml dependencies.tokio".to_string(),
            "package/cli/a/Cargo.toml package.metadata.rust-catalog.catalogs.smithay".to_string(),
        ]
    );
}

/// Git pins that disagree across crates are rejected.
#[test]
fn rejects_git_pins_that_disagree() {
    let found: Vec<String> = issues(
        &["[catalog]"],
        &[
            krate("package/cli/a", &["[dependencies]", "h = { git = \"https://x\", rev = \"1\" }"]),
            krate("package/cli/b", &["[dependencies]", "h = { git = \"https://x\", rev = \"2\" }"]),
        ],
    );
    assert_eq!(found, vec!["package/cli/a/Cargo.toml h".to_string()]);
}

/// A name that is a git dependency in one crate and a registry one in another is rejected.
#[test]
fn rejects_mixed_sources() {
    let found: Vec<String> = issues(
        &["[catalog]"],
        &[
            krate("package/cli/a", &["[dependencies]", "h = { git = \"https://x\", rev = \"1\" }"]),
            krate("package/cli/b", &["[dependencies]", "h = \"1\""]),
        ],
    );
    assert_eq!(found, vec!["package/cli/a/Cargo.toml h".to_string()]);
}

/// Renames that disagree across crates are rejected.
#[test]
fn rejects_conflicting_renames() {
    let found: Vec<String> = issues(
        &["[catalog]"],
        &[
            krate("package/cli/a", &["[dependencies]", "h = { version = \"1\", package = \"real-a\" }"]),
            krate("package/cli/b", &["[dependencies]", "h = { version = \"1\", package = \"real-b\" }"]),
        ],
    );
    assert_eq!(found, vec!["package/cli/a/Cargo.toml h".to_string()]);
}

/// A hand-written inline `workspace.dependencies` is rejected.
#[test]
fn rejects_a_hand_written_inline_workspace_dependencies() {
    let manifest = ManifestFile {
        path: "package/cli/a/Cargo.toml".to_string(),
        source: text(&["[package]", "name = \"a\"", "version = \"0.1.0\"", "", "[workspace]", "dependencies = { anyhow = \"1\" }"]),
    };
    assert_eq!(issues(&["[catalog]"], &[manifest]), vec!["package/cli/a/Cargo.toml workspace.dependencies".to_string()]);
}

/// A generated table that no crate dependency needs any more is removed with its comment.
#[test]
fn removes_the_generated_table_when_nothing_uses_the_catalog() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "anyhow = \">=1\""],
        &[krate("package/cli/a", &["[dependencies]", "anyhow = { workspace = true }"])],
    );
    let with_table: String = manifest_text(&output, "package/cli/a");
    let without: String = with_table.replace("anyhow = { workspace = true }\n", "");
    let second: SyncOutput = run_ok(
        &["[catalog]", "anyhow = \">=1\""],
        &[ManifestFile { path: "package/cli/a/Cargo.toml".to_string(), source: without }],
    );
    let after: String = manifest_text(&second, "package/cli/a");
    assert!(!after.contains("[workspace.dependencies]"), "{after}");
    assert!(!after.contains("Generated by file-enforcer"), "{after}");
    assert!(second.manifests[0].changes.contains(&Change::GeneratedTableRemoved));
}

/// Invalid TOML in a manifest is a TOML error naming the file.
#[test]
fn reports_invalid_manifest_toml() {
    let manifest = ManifestFile { path: "package/cli/a/Cargo.toml".to_string(), source: "[package".to_string() };
    match run(&["[catalog]"], &[manifest]) {
        Err(Error::Toml { path, .. }) => assert_eq!(path, "package/cli/a/Cargo.toml"),
        other => panic!("expected a TOML error, got {other:?}"),
    }
}

/// An invalid catalog is a catalog error before any manifest is looked at.
#[test]
fn reports_an_invalid_catalog() {
    assert!(matches!(run(&["[catalog]", "anyhow = \"1\""], &[]), Err(Error::Catalog { .. })));
}
