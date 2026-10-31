//! What:     Unit tests of catalog parsing and validation, ported from the TypeScript `parse.unit.test.ts`.
//! Why:      The Rust parser must accept and reject exactly what the TypeScript parser did, with the same issue
//!           locations in the same order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parse.unit.test.ts
//! describe('parseRustCatalog', () => { it('reads default floors ...', () => { ... }); });
//! ```

use crate::catalog::parse_catalog;
use crate::error::Error;
use crate::types::{CatalogDocument, Entry, TrustPolicy};

/// Parses `lines` joined as a catalog file.
fn parse_lines(lines: &[&str]) -> Result<CatalogDocument, Error> {
    return parse_catalog(&lines.join("\n"), "rust-catalog.toml");
}

/// Parses `lines`, expecting a catalog error, and returns every issue location in order.
fn issue_locations(lines: &[&str]) -> Vec<String> {
    return match parse_lines(lines) {
        Err(Error::Catalog { issues, .. }) => issues.into_iter().map(|issue| return issue.location).collect(),
        other => panic!("expected a catalog error, got {other:?}"),
    };
}

/// A registry entry without a rename.
fn registry(version: &str) -> Entry {
    return Entry::Registry {
        version: version.to_string(),
        package: None,
    };
}

/// Default floors, renames and git pins are read in source order.
#[test]
fn reads_default_entries_in_source_order() {
    let document: CatalogDocument = parse_lines(&[
        "[catalog]",
        "anyhow = \">=1.0.104\"",
        "tracing-log = { version = \">=0.2\", package = \"tracing-log\" }",
        "helix-core = { git = \"https://github.com/helix-editor/helix\", rev = \"ba40e547426b0f9896c8bdc699a4ab11f2b37dbc\" }",
    ])
    .expect("valid catalog");
    let names: Vec<&str> = document.catalog.iter().map(|pair| return pair.0.as_str()).collect();
    assert_eq!(names, vec!["anyhow", "tracing-log", "helix-core"]);
    assert_eq!(document.catalog.get("anyhow"), Some(&registry(">=1.0.104")));
    assert_eq!(
        document.catalog.get("tracing-log"),
        Some(&Entry::Registry {
            version: ">=0.2".to_string(),
            package: Some("tracing-log".to_string()),
        })
    );
    assert_eq!(
        document.catalog.get("helix-core"),
        Some(&Entry::Git {
            git: "https://github.com/helix-editor/helix".to_string(),
            rev: "ba40e547426b0f9896c8bdc699a4ab11f2b37dbc".to_string(),
            package: None,
        })
    );
}

/// Named catalogs hold bounded requirements.
#[test]
fn reads_bounded_named_catalogs() {
    let document: CatalogDocument = parse_lines(&[
        "[catalogs.windows-legacy]",
        "windows = \"0.58\"",
        "a = \"^1.2\"",
        "b = \"~1.2.3\"",
        "c = \"=1.18.1\"",
    ])
    .expect("valid catalog");
    let legacy = document.named("windows-legacy").expect("named catalog");
    assert_eq!(legacy.get("windows"), Some(&registry("0.58")));
    assert_eq!(legacy.get("c"), Some(&registry("=1.18.1")));
    assert!(document.named("missing").is_none());
}

/// A file with no settings gets pnpm's defaults.
#[test]
fn applies_pnpm_defaults() {
    let document: CatalogDocument = parse_lines(&["[catalog]"]).expect("valid catalog");
    let settings = document.settings;
    assert!(!settings.catalog_prune);
    assert_eq!(settings.minimum_release_age, 1_440);
    assert!(settings.minimum_release_age_exclude.is_empty());
    assert!(!settings.minimum_release_age_exclude_prune);
    assert!(settings.minimum_release_age_ignore_missing_time);
    assert!(!settings.minimum_release_age_strict);
    assert_eq!(settings.trust_policy, TrustPolicy::Off);
    assert!(settings.trust_policy_exclude.is_empty());
    assert!(!settings.trust_policy_exclude_prune);
    assert_eq!(settings.trust_policy_ignore_after, None);
}

/// An explicit `minimumReleaseAge` turns strict mode on, and every setting is read.
#[test]
fn reads_every_setting() {
    let document: CatalogDocument = parse_lines(&[
        "catalogPrune = true",
        "minimumReleaseAge = 1440",
        "minimumReleaseAgeExclude = [\"ring\", \"zbus@5.19.0 || 5.19.1\"]",
        "minimumReleaseAgeExcludePrune = true",
        "minimumReleaseAgeIgnoreMissingTime = false",
        "trustPolicy = \"no-downgrade\"",
        "trustPolicyExclude = [\"clap@4.6.7\"]",
        "trustPolicyExcludePrune = true",
        "trustPolicyIgnoreAfter = 525600",
        "[catalog]",
    ])
    .expect("valid catalog");
    let settings = document.settings;
    assert!(settings.catalog_prune);
    assert_eq!(settings.minimum_release_age_exclude, vec!["ring".to_string(), "zbus@5.19.0 || 5.19.1".to_string()]);
    assert!(settings.minimum_release_age_exclude_prune);
    assert!(!settings.minimum_release_age_ignore_missing_time);
    assert!(settings.minimum_release_age_strict);
    assert_eq!(settings.trust_policy, TrustPolicy::NoDowngrade);
    assert_eq!(settings.trust_policy_exclude, vec!["clap@4.6.7".to_string()]);
    assert!(settings.trust_policy_exclude_prune);
    assert_eq!(settings.trust_policy_ignore_after, Some(525_600));
}

/// An explicit `minimumReleaseAgeStrict = false` is honoured.
#[test]
fn honours_explicit_strict_false() {
    let document: CatalogDocument =
        parse_lines(&["minimumReleaseAge = 60", "minimumReleaseAgeStrict = false", "[catalog]"]).expect("valid catalog");
    assert!(!document.settings.minimum_release_age_strict);
}

/// A whole-number float counts as minutes, as it did in the TypeScript reader.
#[test]
fn accepts_whole_float_minutes() {
    let document: CatalogDocument = parse_lines(&["minimumReleaseAge = 60.0", "[catalog]"]).expect("valid catalog");
    assert_eq!(document.settings.minimum_release_age, 60);
}

/// Use-site keys in catalog entries are rejected, one issue per key.
#[test]
fn rejects_use_site_keys() {
    assert_eq!(
        issue_locations(&[
            "[catalog]",
            "tokio = { version = \">=1\", features = [\"rt\"] }",
            "turso = { version = \">=0.8\", default-features = false, optional = true }",
            "own = { path = \"../own\" }",
        ]),
        vec!["catalog.tokio", "catalog.turso", "catalog.turso", "catalog.own", "catalog.own"]
    );
}

/// Bounded versions in the default catalog and floors in named catalogs are rejected.
#[test]
fn rejects_wrong_requirement_kind() {
    assert_eq!(
        issue_locations(&[
            "[catalog]",
            "anyhow = \"1.0.104\"",
            "serde = { version = \"^1\" }",
            "[catalogs.legacy]",
            "windows = \">=0.58\"",
        ]),
        vec!["catalog.anyhow", "catalog.serde", "catalogs.legacy.windows"]
    );
}

/// Malformed git entries are rejected.
#[test]
fn rejects_malformed_git_entries() {
    assert_eq!(
        issue_locations(&[
            "[catalog]",
            "a = { git = \"https://example.com/a\" }",
            "b = { git = \"https://example.com/b\", rev = \"abc\", version = \">=1\" }",
            "c = { rev = \"abc\", version = \">=1\" }",
            "d = { git = \"https://example.com/d\", branch = \"main\", rev = \"abc\" }",
        ]),
        vec!["catalog.a", "catalog.b", "catalog.c", "catalog.d"]
    );
}

/// Adversarial names, unknown keys and non-table values are rejected.
#[test]
fn rejects_adversarial_entries() {
    assert_eq!(
        issue_locations(&[
            "[catalog]",
            "\"__proto__\" = \">=1\"",
            "\"../escape\" = \">=1\"",
            "\"a\\nb\" = \">=1\"",
            "\"9lives\" = \">=1\"",
            "x = { version = \">=1\", registry = \"other\" }",
            "y = 3",
            "z = { version = \">=1\", package = \"bad name\" }",
        ]),
        vec![
            "catalog.__proto__",
            "catalog.../escape",
            "catalog.a\nb",
            "catalog.9lives",
            "catalog.x",
            "catalog.y",
            "catalog.z"
        ]
    );
}

/// Unknown top-level keys, `catalogs.default` and malformed tables are rejected.
#[test]
fn rejects_unknown_top_level_and_malformed_tables() {
    assert_eq!(
        issue_locations(&["dependencies = {}", "catalog = 1", "[catalogs.default]", "a = \"1\""]),
        vec!["dependencies", "catalog", "catalogs.default"]
    );
    assert_eq!(issue_locations(&["catalogs = 1"]), vec!["catalogs"]);
}

/// Malformed settings are rejected in the TypeScript reporting order.
#[test]
fn rejects_malformed_settings() {
    assert_eq!(
        issue_locations(&[
            "minimumReleaseAge = -1",
            "minimumReleaseAgeExclude = \"ring\"",
            "minimumReleaseAgeStrict = \"yes\"",
            "trustPolicy = \"trust-everyone\"",
            "trustPolicyIgnoreAfter = 1.5",
            "[catalog]",
        ]),
        vec![
            "minimumReleaseAge",
            "trustPolicyIgnoreAfter",
            "minimumReleaseAgeExclude",
            "minimumReleaseAgeStrict",
            "trustPolicy"
        ]
    );
}

/// Mixed-type lists, non-boolean prune switches and quoted minute counts are rejected.
#[test]
fn rejects_mixed_lists_and_quoted_minutes() {
    assert_eq!(
        issue_locations(&[
            "minimumReleaseAge = \"1440\"",
            "catalogPrune = 1",
            "trustPolicyExclude = [\"clap@4.6.7\", 3]",
            "[catalog]",
        ]),
        vec!["minimumReleaseAge", "catalogPrune", "trustPolicyExclude"]
    );
}

/// Invalid TOML is reported as a TOML error, not a catalog error.
#[test]
fn reports_invalid_toml() {
    assert!(matches!(parse_lines(&["[catalog", "x = "]), Err(Error::Toml { .. })));
}

/// The message names each issue, its remedy and the decision record.
#[test]
fn message_names_issue_remedy_and_decision_doc() {
    let error: Error = parse_lines(&["[catalog]", "tokio = { version = \">=1\", features = [\"rt\"] }"]).expect_err("invalid");
    let message: String = error.to_string();
    assert!(message.contains("catalog.tokio: declares `features`."));
    assert!(message.contains("doc/decision/rust-dependency-catalog.md"));
    assert!(message.starts_with("rust-catalog.toml has 1 problem(s)"));
}
