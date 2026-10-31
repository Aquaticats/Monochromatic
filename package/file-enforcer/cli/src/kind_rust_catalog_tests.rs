//! What:
//!  Tests for the `rust-catalog` rule kind through the real run entry point:
//! allowed and rejected trees,
//! drift,
//! repair by `write`,
//! and composition with `toml-keys`.
//! Why:
//!  The kind exists to plan the catalog and every manifest table in one pass from the virtual view.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('rust-catalog rule', () => { it('detects drift and repairs it', () => {}); });
//! ```

/// Imports the kind under test.
use crate::kind_rust_catalog::RustCatalogBuilder;
/// Imports the tree and run helpers.
use crate::test_support::{count_code, read_text, run_config, scratch_dir, write_files};
/// Imports `ConfigScope` from `monochromatic_file_enforcer::config_decode`.
use monochromatic_file_enforcer::config_decode::ConfigScope;
/// Imports `RuleConfig` from `monochromatic_file_enforcer::config_types`.
use monochromatic_file_enforcer::config_types::RuleConfig;
/// Imports `RuleRegistry` from `monochromatic_file_enforcer::registry`.
use monochromatic_file_enforcer::registry::RuleRegistry;
/// Imports `RunMode` from `monochromatic_file_enforcer::run`.
use monochromatic_file_enforcer::run::RunMode;
/// Imports `RuleBuilder` from `monochromatic_file_enforcer::rule`.
use monochromatic_file_enforcer::rule::RuleBuilder;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// A config with only the catalog rule.
const CATALOG_ONLY: &str = r#"{"rules": [{"kind": "rust-catalog", "id": "catalog", "catalog": "rust-catalog.toml",
    "manifests": ["package/*/*/Cargo.toml"]}]}"#;

/// A catalog with no entries yet:
/// the sync adopts what the crates use.
const EMPTY_CATALOG: &str = "[catalog]\n";

/// A crate whose literal dependency the sync moves into the catalog.
const LITERAL_CRATE: &str = "[package]\nname = \"a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nanyhow = \"1\"\n\n[workspace]\n";

/// The registry under test.
fn registry() -> RuleRegistry {
    let mut registry = RuleRegistry::with_defaults();
    registry.register("rust-catalog", Box::new(RustCatalogBuilder));
    return registry;
}

/// Run a config in repository scope.
fn run_repository(root: &Path, config: &str, mode: RunMode) -> (i32, Vec<monochromatic_file_enforcer::event::Event>) {
    return run_config(root, config, ConfigScope::Repository, mode, &registry());
}

/// A drifting tree is reported by `check`,
/// repaired by `write`,
/// and then clean and stable.
#[test]
fn drift_is_detected_and_write_repairs_it() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("rust-catalog.toml", EMPTY_CATALOG), ("package/cli/a/Cargo.toml", LITERAL_CRATE)]);
    let (check_code, check) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Check { expected_dir: None });
    assert_eq!(check_code, 1, "{check:?}");
    assert_eq!(count_code(&check, "drift-content"), 2, "the catalog and the manifest both drift: {check:?}");
    assert_eq!(count_code(&check, "kind-not-wired"), 0);
    let (write_code, write) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Write);
    assert_eq!(write_code, 0, "{write:?}");
    assert!(read_text(dir.path(), "rust-catalog.toml").contains("anyhow = \">=1\""));
    let manifest = read_text(dir.path(), "package/cli/a/Cargo.toml");
    assert!(manifest.contains("anyhow = { workspace = true }"), "{manifest}");
    assert!(manifest.contains("[workspace.dependencies]\nanyhow = \">=1\"\n"), "{manifest}");
    let (clean_code, clean) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Check { expected_dir: None });
    assert_eq!(clean_code, 0, "{clean:?}");
    let (again_code, again) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Write);
    assert_eq!((again_code, count_code(&again, "file-written")), (0, 0), "a clean tree is not rewritten: {again:?}");
}

/// The rule composes with a `toml-keys` rule before it:
/// one write applies the edition fix and the catalog table to the same manifest.
#[test]
fn composes_with_toml_keys_rules_that_run_before_it() {
    let config = r#"{"rules": [
      {"kind": "toml-keys", "id": "base", "manifests": ["package/*/*/Cargo.toml"],
       "edits": [{"guard": ["package"], "path": ["package", "edition"], "value": "2024"}], "blocks": []},
      {"kind": "rust-catalog", "id": "catalog", "catalog": "rust-catalog.toml", "manifests": ["package/*/*/Cargo.toml"]}
    ]}"#;
    let dir = scratch_dir();
    write_files(dir.path(), &[("rust-catalog.toml", EMPTY_CATALOG), ("package/cli/a/Cargo.toml", LITERAL_CRATE)]);
    let (code, events) = run_repository(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0, "{events:?}");
    let manifest = read_text(dir.path(), "package/cli/a/Cargo.toml");
    assert!(manifest.contains("edition = \"2024\""), "the key rule's change is kept: {manifest}");
    assert!(manifest.contains("anyhow = { workspace = true }"), "the catalog rule saw and kept it: {manifest}");
    let (clean_code, clean) = run_repository(dir.path(), config, RunMode::Check { expected_dir: None });
    assert_eq!(clean_code, 0, "{clean:?}");
}

/// Both rules compose,
/// so the engine accepts either order and the manifest ends the same;
/// the repository config still lists the catalog rule last,
/// where its input is final.
#[test]
fn either_order_with_a_toml_keys_rule_gives_the_same_manifest() {
    let keys = r#"{"kind": "toml-keys", "id": "base", "manifests": ["package/*/*/Cargo.toml"],
       "edits": [{"guard": ["package"], "path": ["package", "edition"], "value": "2024"}], "blocks": []}"#;
    let catalog = r#"{"kind": "rust-catalog", "id": "catalog", "catalog": "rust-catalog.toml", "manifests": ["package/*/*/Cargo.toml"]}"#;
    let mut results: Vec<String> = Vec::new();
    for rules in [format!("{keys},{catalog}"), format!("{catalog},{keys}")] {
        let dir = scratch_dir();
        write_files(dir.path(), &[("rust-catalog.toml", EMPTY_CATALOG), ("package/cli/a/Cargo.toml", LITERAL_CRATE)]);
        let (code, events) = run_repository(dir.path(), &format!("{{\"rules\": [{rules}]}}"), RunMode::Write);
        assert_eq!(code, 0, "{events:?}");
        results.push(read_text(dir.path(), "package/cli/a/Cargo.toml"));
    }
    assert_eq!(results[0], results[1]);
}

/// A `toml-keys` rule that reads a manifest and changes nothing does not block a later catalog change:
/// both compose,
/// so the catalog rule transforms what the key rule left.
#[test]
fn a_conforming_toml_keys_rule_does_not_block_a_later_catalog_change() {
    let config = r#"{"rules": [
      {"kind": "toml-keys", "id": "base", "manifests": ["package/*/*/Cargo.toml"],
       "edits": [{"guard": ["package"], "path": ["package", "edition"], "value": "2021"}], "blocks": []},
      {"kind": "rust-catalog", "id": "catalog", "catalog": "rust-catalog.toml", "manifests": ["package/*/*/Cargo.toml"]}
    ]}"#;
    let dir = scratch_dir();
    write_files(dir.path(), &[("rust-catalog.toml", EMPTY_CATALOG), ("package/cli/a/Cargo.toml", LITERAL_CRATE)]);
    let (code, events) = run_repository(dir.path(), config, RunMode::Write);
    assert_eq!(code, 0, "{events:?}");
    assert_eq!(count_code(&events, "rule-order"), 0);
    assert!(read_text(dir.path(), "package/cli/a/Cargo.toml").contains("anyhow = { workspace = true }"));
}

/// A rule that does not compose and reads a manifest must still come after the catalog rule that rewrites it.
#[test]
fn a_non_composing_reader_before_the_catalog_rule_is_rejected() {
    let config = r#"{"rules": [
      {"kind": "concat", "id": "copy", "output": "copy.txt", "parts": [{"file": "package/cli/a/Cargo.toml"}]},
      {"kind": "rust-catalog", "id": "catalog", "catalog": "rust-catalog.toml", "manifests": ["package/*/*/Cargo.toml"]}
    ]}"#;
    let dir = scratch_dir();
    write_files(dir.path(), &[("rust-catalog.toml", EMPTY_CATALOG), ("package/cli/a/Cargo.toml", LITERAL_CRATE)]);
    let (code, events) = run_repository(dir.path(), config, RunMode::Write);
    assert_eq!((code, count_code(&events, "rule-order")), (1, 1), "{events:?}");
    assert_eq!(read_text(dir.path(), "package/cli/a/Cargo.toml"), LITERAL_CRATE, "a failing run writes nothing");
}

/// A missing catalog fails the run and names the file.
#[test]
fn a_missing_catalog_is_a_failure() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("package/cli/a/Cargo.toml", LITERAL_CRATE)]);
    let (code, events) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Check { expected_dir: None });
    assert_eq!(code, 1, "{events:?}");
    assert_eq!(count_code(&events, "catalog-missing"), 1, "{events:?}");
}

/// A manifest that is not TOML fails the run and writes nothing,
/// even for the healthy manifests.
#[test]
fn an_invalid_manifest_fails_the_whole_run() {
    let dir = scratch_dir();
    write_files(
        dir.path(),
        &[("rust-catalog.toml", EMPTY_CATALOG), ("package/cli/a/Cargo.toml", LITERAL_CRATE), ("package/cli/b/Cargo.toml", "[package\n")],
    );
    let (code, events) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Write);
    assert_eq!(code, 1, "{events:?}");
    assert_eq!(count_code(&events, "manifest-invalid"), 1, "{events:?}");
    assert_eq!(read_text(dir.path(), "package/cli/a/Cargo.toml"), LITERAL_CRATE);
    assert_eq!(read_text(dir.path(), "rust-catalog.toml"), EMPTY_CATALOG);
}

/// An invalid catalog fails the run with the catalog's own diagnostic.
#[test]
fn an_invalid_catalog_fails_the_run() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("rust-catalog.toml", "[catalog\n"), ("package/cli/a/Cargo.toml", LITERAL_CRATE)]);
    let (code, events) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Check { expected_dir: None });
    assert_eq!(code, 1, "{events:?}");
    assert_eq!(count_code(&events, "catalog-invalid") + count_code(&events, "manifest-invalid"), 1, "{events:?}");
}

/// A dependency the sync refuses (an exact pin) fails the run with the sync's explanation.
#[test]
fn an_exact_pin_is_refused_with_the_sync_diagnostic() {
    let pinned = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nanyhow = \"=1.0.1\"\n\n[workspace]\n";
    let dir = scratch_dir();
    write_files(dir.path(), &[("rust-catalog.toml", EMPTY_CATALOG), ("package/cli/a/Cargo.toml", pinned)]);
    let (code, events) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Write);
    assert_eq!(code, 1, "{events:?}");
    assert_eq!(count_code(&events, "catalog-sync-failed"), 1, "{events:?}");
    assert_eq!(read_text(dir.path(), "package/cli/a/Cargo.toml"), pinned);
}

/// A catalog entry no crate uses is reported as a warning when pruning is off,
/// and a warning is not drift.
#[test]
fn an_unused_entry_is_a_warning_when_pruning_is_off() {
    let catalog = "catalogPrune = false\n\n[catalog]\nanyhow = \">=1\"\nunused = \">=2\"\n";
    let manifest = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nanyhow = { workspace = true }\n\n[workspace]\n\n\
        # Generated by file-enforcer from rust-catalog.toml; edit the catalog, not this table (doc/decision/rust-dependency-catalog.md).\n\
        [workspace.dependencies]\nanyhow = \">=1\"\n";
    let dir = scratch_dir();
    write_files(dir.path(), &[("rust-catalog.toml", catalog), ("package/cli/a/Cargo.toml", manifest)]);
    let (code, events) = run_repository(dir.path(), CATALOG_ONLY, RunMode::Check { expected_dir: None });
    assert_eq!(count_code(&events, "catalog-warning"), 1, "{events:?}");
    assert_eq!(code, 0, "{events:?}");
}

/// The builder refuses an entry of another kind.
#[test]
fn the_builder_refuses_another_kind() {
    let other = RuleConfig::Mirror { id: "m".to_string(), from: "a".to_string(), to: "b".to_string() };
    let error = RustCatalogBuilder.build(&other).err().expect("a mirror entry is refused");
    assert_eq!(error.code, "kind-unknown");
    assert_eq!(error.rule.as_deref(), Some("m"));
}
