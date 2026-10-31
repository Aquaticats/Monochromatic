//! What:
//! Unit tests of use-site classification and manifest reading:
//! every shape a dependency entry can take and every problem the reader reports.
//! Why:
//! The sync trusts these results,
//! so each branch of the classifier and of the manifest reader is pinned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // scan.unit.test.ts
//! describe('classifySite', () => { it('reads a string as a registry literal', () => { ... }); });
//! ```

/// Imports `crate::crate_info::{CrateInfo, ManifestFile, read_crate_info}`.
use crate::crate_info::{CrateInfo, ManifestFile, read_crate_info};
/// Imports `crate::error::SyncIssue`.
use crate::error::SyncIssue;
/// Imports `crate::site::{SiteKind, UseSite}`.
use crate::site::{SiteKind, UseSite};
/// Imports `crate::sync_test_support::issues`.
use crate::sync_test_support::issues;
/// Imports `crate::types::Entry`.
use crate::types::Entry;

/// Reads a manifest made of `lines`,
/// returning the crate info and the issues found.
fn read(lines: &[&str]) -> (CrateInfo, Vec<SyncIssue>) {
    let manifest = ManifestFile {
        path: "package/cli/a/Cargo.toml".to_string(),
        source: format!("{}\n", lines.join("\n")),
    };
    let mut found: Vec<SyncIssue> = Vec::new();
    let info: CrateInfo = read_crate_info(&manifest, &mut found).expect("valid TOML");
    return (info, found);
}

/// The sites of a manifest.
fn sites(lines: &[&str]) -> Vec<UseSite> {
    return read(lines).0.sites;
}

/// Problems reported for a manifest as `location: problem` text.
fn problems(lines: &[&str]) -> Vec<String> {
    return read(lines).1.into_iter().map(|issue| return format!("{}: {}", issue.location, issue.problem)).collect();
}

/// Strings,
/// git pins,
/// versions with renames,
/// paths and inherited entries each get their kind.
#[test]
fn classifies_every_shape() {
    let found: Vec<UseSite> = sites(&[
        "[dependencies]",
        "plain = \"1\"",
        "versioned = { version = \"1\", package = \"real\", default-features = false }",
        "pinned = { git = \"https://x\", rev = \"r\", package = \"real\" }",
        "local = { path = \"../b\" }",
        "inherited = { workspace = true, features = [\"a\"], optional = true }",
    ]);
    let kinds: Vec<SiteKind> = found.iter().map(|site| return site.kind).collect();
    assert_eq!(kinds, vec![SiteKind::Literal, SiteKind::Literal, SiteKind::Literal, SiteKind::Path, SiteKind::Catalog]);
    assert_eq!(found[1].literal, Entry::Registry { version: "1".to_string(), package: Some("real".to_string()) });
    assert!(found[1].disables_default_features);
    assert_eq!(found[2].literal, Entry::Git { git: "https://x".to_string(), rev: "r".to_string(), package: Some("real".to_string()) });
    assert_eq!(found[3].path, "../b");
    assert!(!found[0].disables_default_features);
    assert_eq!(found[0].location(), "dependencies.plain");
}

/// Target-specific and build and dev tables are read after the top-level ones.
#[test]
fn reads_target_tables_in_cargo_order() {
    let found: Vec<UseSite> = sites(&[
        "[target.'cfg(unix)'.dependencies]",
        "u = \"1\"",
        "[dev-dependencies]",
        "d = \"1\"",
        "[build-dependencies]",
        "b = \"1\"",
        "[dependencies]",
        "t = \"1\"",
        "[target.'cfg(windows)'.build-dependencies]",
        "w = \"1\"",
    ]);
    let locations: Vec<String> = found.iter().map(UseSite::location).collect();
    assert_eq!(
        locations,
        vec![
            "dependencies.t",
            "dev-dependencies.d",
            "build-dependencies.b",
            "target.cfg(unix).dependencies.u",
            "target.cfg(windows).build-dependencies.w",
        ]
    );
}

/// A `target` that is not a table,
/// or holds non-tables,
/// contributes nothing.
#[test]
fn ignores_malformed_target_tables() {
    assert!(sites(&["target = 1"]).is_empty());
    assert!(sites(&["[target]", "'cfg(unix)' = 1"]).is_empty());
}

/// Every unusable entry shape is reported with its location.
#[test]
fn reports_unusable_entries() {
    let found: Vec<String> = problems(&[
        "[dependencies]",
        "number = 3",
        "extras = { workspace = true, version = \"1\", git = \"x\" }",
        "off = { workspace = false }",
        "registry = { version = \"1\", registry = \"corp\" }",
        "no-rev = { git = \"https://x\" }",
        "empty = { optional = true }",
    ]);
    assert_eq!(
        found,
        vec![
            "dependencies.number: is neither a version string nor a table",
            "dependencies.extras: declares `version`, `git` beside `workspace = true`",
            "dependencies.off: declares workspace, which the Rust catalog does not support",
            "dependencies.registry: declares registry, which the Rust catalog does not support",
            "dependencies.no-rev: git dependency has no `rev`",
            "dependencies.empty: has no version, git, path, or `workspace = true`",
        ]
    );
}

/// Package identity and selections are read;
/// a non-string selection is reported.
#[test]
fn reads_identity_and_selections() {
    let (info, found) = read(&[
        "[package]",
        "name = \"a\"",
        "version = \"0.2.0\"",
        "[package.metadata.rust-catalog.catalogs]",
        "windows = \"legacy\"",
        "broken = 3",
    ]);
    assert_eq!(info.package_name, "a");
    assert_eq!(info.package_version, "0.2.0");
    assert_eq!(info.crate_dir, "package/cli/a");
    assert_eq!(info.selection("windows"), Some("legacy"));
    assert_eq!(info.selection("broken"), None);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].location, "package.metadata.rust-catalog.catalogs.broken");
}

/// A manifest without a package table has empty identity.
#[test]
fn missing_package_table_has_empty_identity() {
    let (info, _) = read(&["[dependencies]"]);
    assert_eq!(info.package_name, "");
    assert_eq!(info.package_version, "");
}

/// A hand-written `workspace.dependencies` in any non-standard form is reported once.
#[test]
fn reports_hand_written_workspace_dependencies() {
    assert_eq!(problems(&["[workspace]", "dependencies = { a = \"1\" }"]).len(), 1);
    assert_eq!(problems(&["[workspace]", "dependencies.a = \"1\""]).len(), 1);
    assert_eq!(problems(&["[workspace.dependencies.a]", "version = \"1\""]).len(), 1);
    assert_eq!(problems(&["[workspace]", "dependencies = 3"]).len(), 1);
    assert!(problems(&["[workspace.dependencies]", "a = \"1\""]).is_empty());
    assert!(problems(&["[workspace]"]).is_empty());
    assert!(problems(&["workspace = 3"]).is_empty());
}

/// Invalid manifest TOML names the file.
#[test]
fn invalid_toml_is_a_hard_error() {
    let manifest = ManifestFile { path: "x/Cargo.toml".to_string(), source: "[".to_string() };
    let mut found: Vec<SyncIssue> = Vec::new();
    assert!(read_crate_info(&manifest, &mut found).is_err());
}

/// The sync reports the same classification problems before anything else.
#[test]
fn sync_reports_classification_problems() {
    let manifest = ManifestFile {
        path: "package/cli/a/Cargo.toml".to_string(),
        source: "[package]\nname = \"a\"\nversion = \"0.1.0\"\n[dependencies]\nnumber = 3\n".to_string(),
    };
    assert_eq!(issues(&["[catalog]"], &[manifest]), vec!["package/cli/a/Cargo.toml dependencies.number".to_string()]);
}
