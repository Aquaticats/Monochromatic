//! What:     Unit tests of adoption and manifest editing in the pure sync, ported from the TypeScript
//!           `sync.unit.test.ts`, plus the cases the Rust edit layer adds (dotted keys, creation and removal of the
//!           generated table).
//! Why:      Each adoption, resolution and edit behavior has a test that names it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync.unit.test.ts
//! describe('syncRustCatalog', () => { it('adopts literals across crates ...', () => { ... }); });
//! ```

use crate::change::Change;
use crate::crate_info::ManifestFile;
use crate::sync::{SyncInput, SyncOutput, sync};
use crate::sync_test_support::{crate_with_version, issues, krate, manifest_text, run_ok, text};

/// Literals across crates become one floor, keeping use-site features and comments.
#[test]
fn adopts_literals_across_crates() {
    let output: SyncOutput = run_ok(
        &["[catalog]"],
        &[
            krate(
                "package/cli/a",
                &["[dependencies]", "# Error type.", "anyhow = \"1\"", "clap = { version = \"4\", features = [\"derive\"] }"],
            ),
            krate("package/cli/b", &["[target.'cfg(unix)'.dependencies]", "anyhow = \"1.0.104\""]),
        ],
    );
    assert_eq!(output.catalog.expected, text(&["[catalog]", "anyhow = \">=1.0.104\"", "clap = \">=4\""]));
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("# Error type.\nanyhow = { workspace = true }\nclap = { workspace = true, features = [\"derive\"] }\n"), "{a}");
    assert!(a.contains("[workspace.dependencies]\nanyhow = \">=1.0.104\"\nclap = \">=4\"\n"), "{a}");
    let b: String = manifest_text(&output, "package/cli/b");
    assert!(b.contains("[target.'cfg(unix)'.dependencies]\nanyhow = { workspace = true }\n"), "{b}");
    assert_eq!(output.affected_crates, vec!["package/cli/a".to_string(), "package/cli/b".to_string()]);
    assert!(output.catalog.changes.contains(&Change::CatalogEntryAdded { name: "anyhow".to_string() }));
}

/// A second sync over the first sync's output changes nothing.
#[test]
fn is_idempotent() {
    let first: SyncOutput = run_ok(&["[catalog]"], &[krate("package/cli/a", &["[dependencies]", "anyhow = \"1\""])]);
    let manifests: Vec<ManifestFile> = first
        .manifests
        .iter()
        .map(|file| return ManifestFile { path: file.path.clone(), source: file.expected.clone() })
        .collect();
    let catalog: String = first.catalog.expected.clone();
    let input = SyncInput {
        catalog_path: "rust-catalog.toml",
        catalog_source: &catalog,
        manifests: &manifests,
    };
    let second: SyncOutput = sync(&input).expect("second sync");
    assert_eq!(second.catalog.expected, first.catalog.expected);
    assert_eq!(second.manifests[0].expected, first.manifests[0].expected);
    assert!(second.affected_crates.is_empty());
    assert!(second.drift().is_empty());
}

/// A crate whose catalog sites all disable default features gets `default-features = false` in its generated entry.
#[test]
fn disables_default_features_in_the_generated_entry() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "futures-util = \">=0.3\"", "git-dep = { git = \"https://example.com/g\", rev = \"abc\" }"],
        &[
            krate(
                "package/cli/a",
                &[
                    "[dependencies]",
                    "futures-util = { workspace = true, default-features = false, features = [\"std\"] }",
                    "git-dep = { workspace = true, default-features = false }",
                    "",
                    "[dev-dependencies]",
                    "futures-util = { version = \"0.3\", default-features = false }",
                ],
            ),
            krate("package/cli/b", &["[dependencies]", "futures-util = { workspace = true }"]),
        ],
    );
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(
        a.contains("[workspace.dependencies]\nfutures-util = { version = \">=0.3\", default-features = false }\ngit-dep = { git = \"https://example.com/g\", rev = \"abc\", default-features = false }\n"),
        "{a}"
    );
    assert!(a.contains("[dev-dependencies]\nfutures-util = { workspace = true, default-features = false }\n"), "{a}");
    let b: String = manifest_text(&output, "package/cli/b");
    assert!(b.contains("[workspace.dependencies]\nfutures-util = \">=0.3\"\n"), "{b}");
}

/// Sites that disagree on default features are reported.
#[test]
fn reports_sites_that_disagree_on_default_features() {
    let found: Vec<String> = issues(
        &["[catalog]", "futures-util = \">=0.3\""],
        &[krate(
            "package/cli/a",
            &[
                "[dependencies]",
                "futures-util = { workspace = true, default-features = false }",
                "",
                "[dev-dependencies]",
                "futures-util = { workspace = true }",
            ],
        )],
    );
    assert_eq!(found, vec!["package/cli/a/Cargo.toml futures-util".to_string()]);
}

/// An existing floor rises when a literal needs a newer release.
#[test]
fn raises_an_existing_floor() {
    let output: SyncOutput = run_ok(&["[catalog]", "zbus = \">=5\""], &[krate("package/cli/a", &["[dependencies]", "zbus = \"5.19.0\""])]);
    assert_eq!(output.catalog.expected, text(&["[catalog]", "zbus = \">=5.19.0\""]));
    assert!(output.catalog.changes.contains(&Change::CatalogFloorRaised {
        name: "zbus".to_string(),
        requirement: ">=5.19.0".to_string()
    }));
}

/// A floor raise keeps the comment that follows it on the same line.
#[test]
fn raising_a_floor_keeps_the_trailing_comment() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "zbus = \">=5\" # why", "tokio = { version = \">=1\", package = \"tokio\" } # pinned name"],
        &[krate("package/cli/a", &["[dependencies]", "zbus = \"5.19.0\"", "tokio = { version = \"1.50\", package = \"tokio\" }"])],
    );
    assert_eq!(
        output.catalog.expected,
        text(&["[catalog]", "zbus = \">=5.19.0\" # why", "tokio = { version = \">=1.50\", package = \"tokio\" } # pinned name"])
    );
}

/// Git pins are adopted and section-form entries are rewritten in place.
#[test]
fn adopts_git_pins_and_rewrites_section_form() {
    let output: SyncOutput = run_ok(
        &["[catalog]"],
        &[krate(
            "package/cli/a",
            &[
                "[dependencies]",
                "helix-core = { git = \"https://github.com/helix-editor/helix\", rev = \"abc\" }",
                "",
                "[dependencies.serde]",
                "version = \"1\"",
                "features = [\"derive\"]",
            ],
        )],
    );
    assert!(output.catalog.expected.contains("helix-core = { git = \"https://github.com/helix-editor/helix\", rev = \"abc\" }\n"));
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("[dependencies.serde]\nworkspace = true\nfeatures = [\"derive\"]\n"), "{a}");
}

/// Dotted-key entries are rewritten in place and later moved keys are dropped.
#[test]
fn rewrites_dotted_key_entries() {
    let output: SyncOutput = run_ok(
        &["[catalog]"],
        &[krate(
            "package/cli/a",
            &["[dependencies]", "serde.version = \"1\"", "serde.package = \"serde\"", "serde.features = [\"derive\"]"],
        )],
    );
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("[dependencies]\nserde.workspace = true\nserde.features = [\"derive\"]\n"), "{a}");
}

/// A crate that selects a named catalog resolves exact pins from it.
#[test]
fn resolves_named_catalog_selections() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "", "[catalogs.pinned]", "smithay = \"=0.7.0\""],
        &[krate(
            "package/cli/a",
            &[
                "[package.metadata.rust-catalog.catalogs]",
                "smithay = \"pinned\"",
                "",
                "[dependencies]",
                "smithay = { version = \"=0.7.0\", default-features = false }",
            ],
        )],
    );
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("smithay = { workspace = true, default-features = false }"), "{a}");
    assert!(a.contains("[workspace.dependencies]\nsmithay = { version = \"=0.7.0\", default-features = false }\n"), "{a}");
}

/// Internal path dependencies get the exact target version.
#[test]
fn writes_the_exact_target_version_on_path_dependencies() {
    let output: SyncOutput = run_ok(
        &["[catalog]"],
        &[
            crate_with_version("package/rust-module/lib", &[], "0.1.1"),
            krate("package/cli/a", &["[dependencies]", "lib = { path = \"../../rust-module/lib\", version = \"0.1.0\" }"]),
            krate("package/cli/b", &["[dependencies.lib]", "path = \"../../rust-module/lib\"", "features = [\"x\"]"]),
            krate("package/cli/c", &["[dependencies]", "lib = { path = \"../../rust-module/lib\", features = [\"y\"] }"]),
        ],
    );
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("lib = { path = \"../../rust-module/lib\", version = \"=0.1.1\" }"), "{a}");
    let b: String = manifest_text(&output, "package/cli/b");
    assert!(b.contains("features = [\"x\"]\nversion = \"=0.1.1\"\n"), "{b}");
    let c: String = manifest_text(&output, "package/cli/c");
    assert!(c.contains("lib = { path = \"../../rust-module/lib\", features = [\"y\"], version = \"=0.1.1\" }"), "{c}");
    assert!(output.affected_crates.is_empty());
}

/// An internal path version that already matches changes nothing.
#[test]
fn leaves_a_matching_path_version_alone() {
    let output: SyncOutput = run_ok(
        &["[catalog]"],
        &[
            crate_with_version("package/rust-module/lib", &[], "0.1.1"),
            krate("package/cli/a", &["[dependencies]", "lib = { path = \"../../rust-module/lib\", version = \"=0.1.1\" }"]),
        ],
    );
    assert!(output.drift().is_empty());
}
