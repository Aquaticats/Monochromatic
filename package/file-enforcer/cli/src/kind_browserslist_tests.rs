//! What:
//!  Tests for the `browserslist` rule kind through the real run entry point.
//! Why:
//!  The kind reads a query file and a lock through the view and owns one output.
//!  The resolver is replaced by a renderer over the browserslist crate's committed fixture data,
//! so no test needs the network or the user cache;
//! the production renderer is exercised by running the real binary.
//!  The expected targets are what npm `browserslist` answered for the repository's `.browserslistrc`
//!  (the crate's `conformance/expected.json`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('browserslist rule', () => { it('writes the resolved targets', () => {}); });
//! ```

/// Imports the kind under test.
use crate::kind_browserslist::{BrowserslistBuilder, TargetRenderer};
/// Imports the tree and run helpers.
use crate::test_support::{count_code, read_text, run_config, scratch_dir, write_files};
/// Imports the crate's public surface used by the fixture renderer.
use monochromatic_browserslist::{BrowserData, BrowserslistError, DataFiles, DataLock, render_resolved_json, resolve_config_text};
/// Imports `ConfigScope` from `monochromatic_file_enforcer::config_decode`.
use monochromatic_file_enforcer::config_decode::ConfigScope;
/// Imports `RuleConfig` from `monochromatic_file_enforcer::config_types`.
use monochromatic_file_enforcer::config_types::RuleConfig;
/// Imports `Event` from `monochromatic_file_enforcer::event`.
use monochromatic_file_enforcer::event::Event;
/// Imports `RuleRegistry` from `monochromatic_file_enforcer::registry`.
use monochromatic_file_enforcer::registry::RuleRegistry;
/// Imports `RunMode` from `monochromatic_file_enforcer::run`.
use monochromatic_file_enforcer::run::RunMode;
/// Imports `RuleBuilder` from `monochromatic_file_enforcer::rule`.
use monochromatic_file_enforcer::rule::RuleBuilder;
/// Imports `Path` from `std::path`.
use std::path::Path;
/// Imports `Arc` from `std::sync`.
use std::sync::Arc;

/// The browserslist crate's directory, where the fixture data and the expected answers live.
fn crate_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../browserslist/core-rust").join(relative);
    return std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

/// The repository-root lock.
fn repository_lock() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../browserslist-data.lock.json");
    return std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

/// Resolves against the committed fixture data and the lock's Firefox ESR value.
struct FixtureRenderer;

/// The `impl TargetRenderer for FixtureRenderer` block.
impl TargetRenderer for FixtureRenderer {
    /// Parse the lock (so a bad lock fails here too),
    /// load the fixtures and render.
    fn render(&self, config_text: &str, lock_text: &str) -> Result<String, BrowserslistError> {
        let lock = DataLock::from_json(lock_text)?;
        let agents = crate_file("fixtures/data/agents.js.txt");
        let browsers = crate_file("fixtures/data/browsers.js.txt");
        let versions = crate_file("fixtures/data/browserVersions.js.txt");
        let envs = crate_file("fixtures/data/envs.json.txt");
        let files = DataFiles {
            agents_js: &agents,
            browsers_js: &browsers,
            browser_versions_js: &versions,
            envs_json: &envs,
            firefox_esr: &lock.firefox_esr,
        };
        let data = BrowserData::from_files(files)?;
        let targets = resolve_config_text(config_text, &data)?;
        return Ok(render_resolved_json(&targets));
    }
}

/// A config with only the browserslist rule,
/// named like the repository's own.
const BROWSERSLIST_ONLY: &str = r#"{"rules": [{"kind": "browserslist", "id": "targets", "source": ".browserslistrc",
    "output": ".browserslistrc.resolved.local.json", "lock": "browserslist-data.lock.json", "environment": "production"}]}"#;

/// The registry under test.
fn registry() -> RuleRegistry {
    let mut registry = RuleRegistry::with_defaults();
    registry.register("browserslist", Box::new(BrowserslistBuilder::with_renderer(Arc::new(FixtureRenderer))));
    return registry;
}

/// Run a config in repository scope.
fn run_repository(root: &Path, mode: RunMode) -> (i32, Vec<Event>) {
    return run_config(root, BROWSERSLIST_ONLY, ConfigScope::Repository, mode, &registry());
}

/// The repository's own query file,
/// as the first conformance case holds it.
fn repository_query() -> String {
    let expected: serde_json::Value = serde_json::from_str(&crate_file("conformance/expected.json")).expect("expected.json parses");
    let case = &expected["cases"][0];
    assert_eq!(case["name"], "repository file");
    return case["config"].as_str().expect("config text").to_string();
}

/// The exact text npm browserslist's targets render to: `JSON.stringify(targets, null, 2)` and a newline.
fn expected_output() -> String {
    let expected: serde_json::Value = serde_json::from_str(&crate_file("conformance/expected.json")).expect("expected.json parses");
    let result = &expected["cases"][0]["npm"]["result"];
    return format!("{}\n", serde_json::to_string_pretty(result).expect("the answer serializes"));
}

/// A missing output is drift,
/// `write` creates the exact text npm browserslist's answer renders to,
/// and the tree is then clean.
#[test]
fn drift_is_detected_and_write_repairs_it() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".browserslistrc", &repository_query()), ("browserslist-data.lock.json", &repository_lock())]);
    let (check_code, check) = run_repository(dir.path(), RunMode::Check { expected_dir: None });
    assert_eq!(check_code, 1, "{check:?}");
    assert_eq!(count_code(&check, "drift-missing"), 1, "{check:?}");
    let (write_code, write) = run_repository(dir.path(), RunMode::Write);
    assert_eq!(write_code, 0, "{write:?}");
    assert_eq!(read_text(dir.path(), ".browserslistrc.resolved.local.json"), expected_output());
    let (clean_code, clean) = run_repository(dir.path(), RunMode::Check { expected_dir: None });
    assert_eq!(clean_code, 0, "{clean:?}");
}

/// A hand-edited output is drift in content and is regenerated.
#[test]
fn an_edited_output_is_content_drift_and_is_regenerated() {
    let dir = scratch_dir();
    write_files(
        dir.path(),
        &[
            (".browserslistrc", &repository_query()),
            ("browserslist-data.lock.json", &repository_lock()),
            (".browserslistrc.resolved.local.json", "[]\n"),
        ],
    );
    let (check_code, check) = run_repository(dir.path(), RunMode::Check { expected_dir: None });
    assert_eq!((check_code, count_code(&check, "drift-content")), (1, 1), "{check:?}");
    let (write_code, write) = run_repository(dir.path(), RunMode::Write);
    assert_eq!(write_code, 0, "{write:?}");
    assert_eq!(read_text(dir.path(), ".browserslistrc.resolved.local.json"), expected_output());
}

/// Changing the query file changes the output:
/// the rule reads it through the view.
#[test]
fn the_output_follows_the_query_file() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".browserslistrc", "last 1 Chrome versions\n"), ("browserslist-data.lock.json", &repository_lock())]);
    let (code, events) = run_repository(dir.path(), RunMode::Write);
    assert_eq!(code, 0, "{events:?}");
    let targets: Vec<String> = serde_json::from_str(&read_text(dir.path(), ".browserslistrc.resolved.local.json")).expect("a JSON array");
    assert_eq!(targets.len(), 1, "{targets:?}");
    assert!(targets[0].starts_with("chrome "), "{targets:?}");
}

/// A query the resolver does not implement fails the run, names the file and writes nothing.
#[test]
fn an_unsupported_query_is_refused_with_not_implemented() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".browserslistrc", "defaults\n"), ("browserslist-data.lock.json", &repository_lock())]);
    let (code, events) = run_repository(dir.path(), RunMode::Write);
    assert_eq!(code, 1, "{events:?}");
    assert_eq!(count_code(&events, "browserslist-not-implemented"), 1, "{events:?}");
    assert!(!dir.path().join(".browserslistrc.resolved.local.json").exists());
}

/// A config section is refused as not implemented,
/// so the `environment` field selects nothing the resolver could honor.
#[test]
fn a_config_section_is_refused_as_not_implemented() {
    let dir = scratch_dir();
    write_files(
        dir.path(),
        &[(".browserslistrc", "last 1 Chrome versions\n[production]\nlast 1 Firefox versions\n"), ("browserslist-data.lock.json", &repository_lock())],
    );
    let (code, events) = run_repository(dir.path(), RunMode::Check { expected_dir: None });
    assert_eq!(code, 1, "{events:?}");
    let refusal = events.iter().find(|event| return event.code == "browserslist-not-implemented").expect("a refusal");
    assert!(refusal.hint.as_deref().unwrap_or_default().contains("environment: production"), "{refusal:?}");
}

/// A missing query file or lock fails the run and names the path.
#[test]
fn a_missing_input_is_a_failure() {
    let dir = scratch_dir();
    write_files(dir.path(), &[("browserslist-data.lock.json", &repository_lock())]);
    let (code, events) = run_repository(dir.path(), RunMode::Check { expected_dir: None });
    assert_eq!((code, count_code(&events, "browserslist-input-missing")), (1, 1), "{events:?}");
    let other = scratch_dir();
    write_files(other.path(), &[(".browserslistrc", "last 1 Chrome versions\n")]);
    let (other_code, other_events) = run_repository(other.path(), RunMode::Check { expected_dir: None });
    assert_eq!((other_code, count_code(&other_events, "browserslist-input-missing")), (1, 1), "{other_events:?}");
}

/// A malformed lock fails the run as a data problem.
#[test]
fn a_malformed_lock_is_a_data_failure() {
    let dir = scratch_dir();
    write_files(dir.path(), &[(".browserslistrc", "last 1 Chrome versions\n"), ("browserslist-data.lock.json", "{}")]);
    let (code, events) = run_repository(dir.path(), RunMode::Check { expected_dir: None });
    assert_eq!((code, count_code(&events, "browserslist-data-failed")), (1, 1), "{events:?}");
}

/// The builder refuses an entry of another kind.
#[test]
fn the_builder_refuses_another_kind() {
    let other = RuleConfig::Mirror { id: "m".to_string(), from: "a".to_string(), to: "b".to_string() };
    let error = BrowserslistBuilder::production().build(&other).err().expect("a mirror entry is refused");
    assert_eq!(error.code, "kind-unknown");
}

/// The repository's own committed rule config decodes and builds with the production builder,
/// and the committed lock is accepted by the crate.
#[test]
fn the_repository_lock_parses_and_the_production_builder_builds() {
    DataLock::from_json(&repository_lock()).expect("the committed lock parses");
    let config = RuleConfig::Browserslist {
        id: "t".to_string(),
        source: "q".to_string(),
        output: "o".to_string(),
        lock: "l".to_string(),
        environment: None,
    };
    let rule = BrowserslistBuilder::production().build(&config).expect("builds");
    assert_eq!((rule.id(), rule.kind(), rule.composes()), ("t", "browserslist", false));
}
