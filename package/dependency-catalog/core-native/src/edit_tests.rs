//! What:
//! Unit tests of the format-preserving edits:
//! catalog insertion order,
//! floor raises,
//! pruning,
//! literal rewrites and the generated table.
//! Why:
//! The edit layer is where `toml_edit` decor handling can go wrong,
//! so each edit shape has a test with the exact text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // edit.unit.test.ts
//! describe('edits', () => { it('inserts new entries in name order above the following comment', () => { ... }); });
//! ```

/// Imports `crate::change::Change`.
use crate::change::Change;
/// Imports `crate::crate_info::ManifestFile`.
use crate::crate_info::ManifestFile;
/// Imports `crate::sync::SyncOutput`.
use crate::sync::SyncOutput;
/// Imports `crate::sync_test_support::{krate, manifest_text, run_ok, text}`.
use crate::sync_test_support::{krate, manifest_text, run_ok, text};

/// A manifest with an exact source text.
fn raw(dir: &str, source: &str) -> ManifestFile {
    return ManifestFile { path: format!("{dir}/Cargo.toml"), source: source.to_string() };
}

/// A new entry lands above the leading comment of the first entry that sorts after it.
#[test]
fn inserts_in_name_order_above_the_following_comment() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "alpha = \">=1\"", "# Why gamma.", "gamma = \">=1\""],
        &[krate("package/cli/a", &["[dependencies]", "alpha = { workspace = true }", "gamma = { workspace = true }", "beta = \"1\"", "zeta = \"2\""])],
    );
    assert_eq!(
        output.catalog.expected,
        text(&["[catalog]", "alpha = \">=1\"", "beta = \">=1\"", "# Why gamma.", "gamma = \">=1\"", "zeta = \">=2\""])
    );
}

/// Two new entries sharing an insertion point keep name order.
#[test]
fn merges_additions_at_one_insertion_point() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "omega = \">=1\""],
        &[krate("package/cli/a", &["[dependencies]", "omega = { workspace = true }", "beta = \"1\"", "alpha = \"1\""])],
    );
    assert_eq!(output.catalog.expected, text(&["[catalog]", "alpha = \">=1\"", "beta = \">=1\"", "omega = \">=1\""]));
}

/// A missing `[catalog]` table is created after one blank line.
#[test]
fn creates_the_catalog_table() {
    let with_newline: SyncOutput = run_ok(&["minimumReleaseAge = 1"], &[krate("package/cli/a", &["[dependencies]", "anyhow = \"1\""])]);
    assert_eq!(with_newline.catalog.expected, text(&["minimumReleaseAge = 1", "", "[catalog]", "anyhow = \">=1\""]));
}

/// A catalog written as an inline table still receives new entries.
#[test]
fn adds_to_an_inline_catalog_table() {
    let output: SyncOutput = run_ok(&["catalog = { a = \">=1\" }"], &[krate("package/cli/a", &["[dependencies]", "a = { workspace = true }", "b = \"2\""])]);
    assert!(output.catalog.expected.contains("b = \">=2\""), "{}", output.catalog.expected);
}

/// Floors raise inside inline tables and dotted tables,
/// keeping the rest of the line.
#[test]
fn raises_floors_in_table_forms() {
    let output: SyncOutput = run_ok(
        &["[catalog]", "serde = { version = \">=1\", package = \"serde\" } # c", "tokio.version = \">=1\""],
        &[krate(
            "package/cli/a",
            &["[dependencies]", "serde = { version = \"1.0.5\", package = \"serde\" }", "tokio = { version = \"1.2\" }"],
        )],
    );
    assert_eq!(
        output.catalog.expected,
        text(&["[catalog]", "serde = { version = \">=1.0.5\", package = \"serde\" } # c", "tokio.version = \">=1.2\""])
    );
}

/// Pruning removes an unused entry together with the comment lines above it.
#[test]
fn prunes_with_leading_comments() {
    let output: SyncOutput = run_ok(
        &["catalogPrune = true", "[catalog]", "a = \">=1\"", "# gone", "# really gone", "b = \">=1\"", "c = \">=1\""],
        &[krate("package/cli/a", &["[dependencies]", "a = { workspace = true }", "c = { workspace = true }"])],
    );
    assert_eq!(output.catalog.expected, text(&["catalogPrune = true", "[catalog]", "a = \">=1\"", "c = \">=1\""]));
    assert_eq!(output.catalog.changes, vec![Change::CatalogEntryPruned { path: "catalog.b".to_string() }]);
}

/// A string site becomes an inline table and keeps its trailing comment.
#[test]
fn rewrites_string_sites_keeping_comments() {
    let output: SyncOutput = run_ok(&["[catalog]"], &[krate("package/cli/a", &["[dependencies]", "anyhow = \"1\" # error type"])]);
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("anyhow = { workspace = true } # error type\n"), "{a}");
}

/// Inline sites drop git,
/// rev,
/// package and version and keep everything else as written.
#[test]
fn rewrites_inline_sites_keeping_other_members() {
    let output: SyncOutput = run_ok(
        &["[catalog]"],
        &[krate(
            "package/cli/a",
            &[
                "[dependencies]",
                "a = { version = \"1\", package = \"real-a\", optional = true, features = [\"x\",\n  \"y\"], default-features = false }",
                "b = {version=\"2\",features=[\"z\"]}",
            ],
        )],
    );
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(
        a.contains("a = { workspace = true, optional = true, features = [\"x\",\n  \"y\"], default-features = false }"),
        "{a}"
    );
    assert!(a.contains("b = { workspace = true, features = [\"z\"] }"), "{a}");
}

/// A section-form dependency replaces its first moved key and drops the rest.
#[test]
fn rewrites_section_sites_dropping_later_moved_keys() {
    let output: SyncOutput = run_ok(
        &["[catalog]"],
        &[krate(
            "package/cli/a",
            &["[dependencies.h]", "# where from", "git = \"https://x\"", "rev = \"r\"", "features = [\"f\"]", "package = \"real\""],
        )],
    );
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("[dependencies.h]\n# where from\nworkspace = true\nfeatures = [\"f\"]\n"), "{a}");
}

/// A crate with no workspace table gets one holding the generated entries,
/// below the generated-by comment.
#[test]
fn creates_the_generated_table_with_its_comment() {
    let source: String = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nanyhow = \"1\"\n".to_string();
    let output: SyncOutput = run_ok(&["[catalog]"], &[raw("package/cli/a", &source)]);
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(
        a.ends_with("anyhow = { workspace = true }\n\n# Generated by file-enforcer from rust-catalog.toml; edit the catalog, not this table (doc/decision/rust-dependency-catalog.md).\n[workspace.dependencies]\nanyhow = \">=1\"\n"),
        "{a}"
    );
    assert_eq!(output.manifests[0].changes, vec![Change::SiteInherited { location: "dependencies.anyhow".to_string() }, Change::GeneratedTableWritten]);
}

/// A source without a trailing newline is separated by a blank line.
#[test]
fn separates_with_a_blank_line_when_the_file_lacks_a_final_newline() {
    let source: String = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n[dependencies]\nanyhow = { workspace = true }".to_string();
    let output: SyncOutput = run_ok(&["[catalog]", "anyhow = \">=1\""], &[raw("package/cli/a", &source)]);
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("anyhow = { workspace = true }\n\n# Generated by"), "{a}");
}

/// A stale generated table is replaced in place and keeps the comment above it.
#[test]
fn replaces_a_stale_generated_table_in_place() {
    let source: String = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n[dependencies]\nanyhow = { workspace = true }\n\n# Hand note kept.\n[workspace.dependencies]\nold = \">=1\"\n\n[profile.release]\nlto = true\n".to_string();
    let output: SyncOutput = run_ok(&["[catalog]", "anyhow = \">=1.5\""], &[raw("package/cli/a", &source)]);
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.contains("# Hand note kept.\n[workspace.dependencies]\nanyhow = \">=1.5\"\n\n[profile.release]\nlto = true\n"), "{a}");
    assert!(!a.contains("old"), "{a}");
}

/// A generated table that already holds the right entries keeps a comment on its header line.
#[test]
fn leaves_a_correct_generated_table_untouched() {
    let source: String = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n[dependencies]\nanyhow = { workspace = true }\n\n[workspace.dependencies] # mine\nanyhow = \">=1\"\n".to_string();
    let output: SyncOutput = run_ok(&["[catalog]", "anyhow = \">=1\""], &[raw("package/cli/a", &source)]);
    assert_eq!(manifest_text(&output, "package/cli/a"), source);
    assert!(output.drift().is_empty());
}

/// A workspace written as an inline table is not edited.
#[test]
fn leaves_an_inline_workspace_alone() {
    let source: String = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n[dependencies]\nanyhow = { workspace = true }\n\n[workspace]\n".to_string();
    let output: SyncOutput = run_ok(&["[catalog]", "anyhow = \">=1\""], &[raw("package/cli/a", &source)]);
    assert!(manifest_text(&output, "package/cli/a").ends_with("[workspace.dependencies]\nanyhow = \">=1\"\n"));
    let inline: String = "workspace = {}\n[package]\nname = \"a\"\nversion = \"0.1.0\"\n[dependencies]\nanyhow = { workspace = true }\n".to_string();
    let untouched: SyncOutput = run_ok(&["[catalog]", "anyhow = \">=1\""], &[raw("package/cli/a", &inline)]);
    assert_eq!(manifest_text(&untouched, "package/cli/a"), inline);
}

/// Missing document positions still print the new table last.
#[test]
fn appends_after_every_existing_table() {
    let source: String = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n[dependencies]\nanyhow = \"1\"\n[profile.release]\nlto = true\n[[bin]]\nname = \"x\"\n".to_string();
    let output: SyncOutput = run_ok(&["[catalog]"], &[raw("package/cli/a", &source)]);
    let a: String = manifest_text(&output, "package/cli/a");
    assert!(a.ends_with("lto = true\n[[bin]]\nname = \"x\"\n\n# Generated by file-enforcer from rust-catalog.toml; edit the catalog, not this table (doc/decision/rust-dependency-catalog.md).\n[workspace.dependencies]\nanyhow = \">=1\"\n"), "{a}");
}
