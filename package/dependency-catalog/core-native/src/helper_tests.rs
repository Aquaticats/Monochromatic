//! What:
//! Unit tests of the small helpers:
//! quoting,
//! path joining,
//! directory names,
//! value rendering,
//! entry maps,
//! version edge cases and message text.
//! Why:
//! Each helper's every branch has a test that names it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // helpers.unit.test.ts
//! describe('helpers', () => { it('quote escapes control characters', () => { ... }); });
//! ```

/// Imports `std::cmp::Ordering`.
use std::cmp::Ordering;

/// Imports `crate::change::Change`.
use crate::change::Change;
/// Imports `crate::crate_info::posix_dirname`.
use crate::crate_info::posix_dirname;
/// Imports `crate::error::{CatalogIssue, Error, SyncIssue}`.
use crate::error::{CatalogIssue, Error, SyncIssue};
/// Imports `crate::posix_path::join_normalized`.
use crate::posix_path::join_normalized;
/// Imports `crate::quote::quote`.
use crate::quote::quote;
/// Imports `crate::render::{render_entry_value, render_generated_value}`.
use crate::render::{render_entry_value, render_generated_value};
/// Imports `crate::requirement::satisfies`.
use crate::requirement::satisfies;
/// Imports `crate::site::GeneratedEntry`.
use crate::site::GeneratedEntry;
/// Imports `crate::types::{Entry, EntryMap}`.
use crate::types::{Entry, EntryMap};
/// Imports `crate::version::{ParsedVersion, compare_versions, parse_version}`.
use crate::version::{ParsedVersion, compare_versions, parse_version};

/// A registry entry with an optional rename.
fn registry(version: &str, package: Option<&str>) -> Entry {
    return Entry::Registry {
        version: version.to_string(),
        package: package.map(str::to_string),
    };
}

/// A git entry with an optional rename.
fn git(package: Option<&str>) -> Entry {
    return Entry::Git {
        git: "https://example.com/g".to_string(),
        rev: "abc".to_string(),
        package: package.map(str::to_string),
    };
}

/// Parses a version for ordering assertions.
fn version(text: &str) -> ParsedVersion {
    return parse_version(text).expect("a valid version");
}

/// Quoting escapes quotes and backslashes and control characters and keeps other text.
#[test]
fn quote_escapes() {
    assert_eq!(quote("plain"), "\"plain\"");
    assert_eq!(quote("a\"b\\c"), "\"a\\\"b\\\\c\"");
    assert_eq!(quote("\n\r\t\u{8}\u{c}"), "\"\\n\\r\\t\\b\\f\"");
    assert_eq!(quote("\u{1}\u{7f}"), "\"\\u0001\\u007f\"");
    assert_eq!(quote("caf\u{e9}"), "\"caf\u{e9}\"");
}

/// Path joining resolves dot segments and keeps escaping segments.
#[test]
fn join_normalized_paths() {
    assert_eq!(join_normalized("package/cli/a", "../../rust-module/lib"), "package/rust-module/lib");
    assert_eq!(join_normalized("package/cli/a", "./b"), "package/cli/a/b");
    assert_eq!(join_normalized("package/a", "../../../elsewhere"), "../elsewhere");
    assert_eq!(join_normalized(".", ".."), "..");
    assert_eq!(join_normalized("a", ".."), ".");
    assert_eq!(join_normalized("/a/b", "../c"), "/a/c");
    assert_eq!(join_normalized("/", ".."), "/");
}

/// Directory names of manifest paths.
#[test]
fn dirname_of_paths() {
    assert_eq!(posix_dirname("package/a/Cargo.toml"), "package/a");
    assert_eq!(posix_dirname("Cargo.toml"), ".");
    assert_eq!(posix_dirname("/Cargo.toml"), "/");
}

/// Generated values render as plain strings or inline tables in a fixed field order.
#[test]
fn rendering_generated_values() {
    let plain = GeneratedEntry { entry: registry(">=1", None), default_features: true };
    assert_eq!(render_generated_value(&plain), "\">=1\"");
    let no_defaults = GeneratedEntry { entry: registry(">=1", None), default_features: false };
    assert_eq!(render_generated_value(&no_defaults), "{ version = \">=1\", default-features = false }");
    let renamed = GeneratedEntry { entry: registry(">=1", Some("real")), default_features: true };
    assert_eq!(render_generated_value(&renamed), "{ version = \">=1\", package = \"real\" }");
    let git_all = GeneratedEntry { entry: git(Some("real")), default_features: false };
    assert_eq!(
        render_generated_value(&git_all),
        "{ git = \"https://example.com/g\", rev = \"abc\", package = \"real\", default-features = false }"
    );
    assert_eq!(render_entry_value(&git(None)), "{ git = \"https://example.com/g\", rev = \"abc\" }");
}

/// Entry maps keep first positions and report membership.
#[test]
fn entry_map_behaviour() {
    let mut map: EntryMap = EntryMap::new();
    assert!(map.is_empty());
    map.insert("b", registry(">=1", None));
    map.insert("a", registry(">=2", None));
    map.insert("b", registry(">=3", None));
    assert_eq!(map.len(), 2);
    assert!(map.contains("a"));
    assert!(!map.contains("c"));
    assert_eq!(map.get("b"), Some(&registry(">=3", None)));
    let names: Vec<&str> = map.iter().map(|pair| return pair.0.as_str()).collect();
    assert_eq!(names, vec!["b", "a"]);
    assert_eq!(registry("1", Some("x")).package(), Some("x"));
    assert_eq!(git(None).package(), None);
}

/// Prerelease identifiers order numerically before alphabetically and shorter lists first.
#[test]
fn prerelease_ordering_edges() {
    assert_eq!(compare_versions(&version("1.0.0-1"), &version("1.0.0-alpha")), Ordering::Less);
    assert_eq!(compare_versions(&version("1.0.0-alpha"), &version("1.0.0-1")), Ordering::Greater);
    assert_eq!(compare_versions(&version("1.0.0-alpha"), &version("1.0.0-alpha.1")), Ordering::Less);
    assert_eq!(compare_versions(&version("1.0.0-beta"), &version("1.0.0-alpha")), Ordering::Greater);
    assert_eq!(compare_versions(&version("1.0.0-rc.1"), &version("1.0.0-rc.1")), Ordering::Equal);
    assert_eq!(compare_versions(&version("1.0.0"), &version("1.0.0-rc.1")), Ordering::Greater);
    assert_eq!(compare_versions(&version("2.0.0"), &version("1.9.9")), Ordering::Greater);
}

/// Malformed input to the requirement check is an error with a message.
#[test]
fn malformed_requirements_error() {
    let error = satisfies("1.0.0", "*").expect_err("not a requirement");
    assert!(error.to_string().contains("is not a Cargo version"));
    assert!(satisfies("x", "1").is_err());
    assert_eq!(satisfies("1.2.3", "=1.2.3"), Ok(true));
}

/// Every change prints one readable line.
#[test]
fn change_display() {
    assert_eq!(Change::CatalogEntryAdded { name: "a".to_string() }.to_string(), "adopt a into [catalog]");
    assert_eq!(
        Change::CatalogFloorRaised { name: "a".to_string(), requirement: ">=2".to_string() }.to_string(),
        "raise the [catalog] floor of a to >=2"
    );
    assert_eq!(Change::CatalogEntryPruned { path: "catalog.a".to_string() }.to_string(), "prune the unused entry catalog.a");
    assert_eq!(Change::SiteInherited { location: "dependencies.a".to_string() }.to_string(), "rewrite dependencies.a to `workspace = true`");
    assert_eq!(
        Change::PathVersionWritten { location: "dependencies.a".to_string(), version: "=1".to_string() }.to_string(),
        "write version \"=1\" on dependencies.a"
    );
    assert_eq!(Change::GeneratedTableWritten.to_string(), "write the generated [workspace.dependencies] table");
    assert_eq!(Change::GeneratedTableRemoved.to_string(), "remove the generated [workspace.dependencies] table");
}

/// Error messages name the file,
/// the location,
/// the problem and the fix.
#[test]
fn error_display() {
    let toml = Error::Toml { path: "a/Cargo.toml".to_string(), message: "bad".to_string() };
    assert_eq!(toml.to_string(), "a/Cargo.toml is not valid TOML: bad");
    let catalog = Error::Catalog {
        source_path: "rust-catalog.toml".to_string(),
        issues: vec![CatalogIssue { location: "catalog.a".to_string(), problem: "p".to_string(), remedy: "r".to_string() }],
    };
    assert_eq!(
        catalog.to_string(),
        "rust-catalog.toml has 1 problem(s) the Rust dependency catalog cannot accept:\n- catalog.a: p.\n  Fix: r.\nSee doc/decision/rust-dependency-catalog.md for the catalog format."
    );
    let sync = Error::Sync {
        issues: vec![SyncIssue { file: "f".to_string(), location: "l".to_string(), problem: "p".to_string(), remedy: "r".to_string() }],
    };
    assert_eq!(
        sync.to_string(),
        "The Rust dependency catalog sync found 1 problem(s); nothing was written:\n- f l: p.\n  Fix: r.\nSee doc/decision/rust-dependency-catalog.md."
    );
}
