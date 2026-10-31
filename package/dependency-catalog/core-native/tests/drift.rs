//! What:
//! Drift reproduction:
//! manifests as they were committed before the catalog floors were tightened,
//! synced
//! against the catalog of that commit,
//! must become exactly what the TypeScript file-enforcer wrote.
//! Why:
//! The fixed-point test proves the library changes nothing when nothing should change;
//! this proves it
//! makes the same changes as the TypeScript tool when something must.
//! The fixtures are byte copies of
//! four real manifests (stale and TypeScript-regenerated) and the catalog they were synced against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = syncRustCatalog({ catalogPath, catalogSource: fixtureCatalog, manifests: staleManifests });
//! expect(result.manifests.get(path)).toBe(typescriptOutput);
//! ```

/// Imports `monochromatic_dependency_catalog::{Change, ManifestFile, SyncInput, SyncOutput, check, sync}`.
use monochromatic_dependency_catalog::{Change, ManifestFile, SyncInput, SyncOutput, check, sync};

/// One real crate:
/// its path and the stale and expected manifest texts.
struct Fixture {
    /// Repository-relative manifest path.
    path: &'static str,
    /// The manifest before the TypeScript tool ran.
    stale: &'static str,
    /// The manifest the TypeScript tool wrote.
    expected: &'static str,
}

/// The frozen catalog the fixtures were synced against.
const CATALOG: &str = include_str!("fixtures/drift/catalog.toml");

/// The four real crates;
/// `desktop-app` depends on `truepeak-core` by path and carries `default-features = false`
/// sites,
/// git-free but feature-rich entries and target-specific tables.
fn fixtures() -> [Fixture; 4] {
    return [
        Fixture {
            path: "package/mpm-shadow/cli/Cargo.toml",
            stale: include_str!("fixtures/drift/mpm-shadow.stale.toml"),
            expected: include_str!("fixtures/drift/mpm-shadow.expected.toml"),
        },
        Fixture {
            path: "package/music-player/desktop-app/Cargo.toml",
            stale: include_str!("fixtures/drift/desktop-app.stale.toml"),
            expected: include_str!("fixtures/drift/desktop-app.expected.toml"),
        },
        Fixture {
            path: "package/music-player/truepeak-core/Cargo.toml",
            stale: include_str!("fixtures/drift/truepeak-core.stale.toml"),
            expected: include_str!("fixtures/drift/truepeak-core.expected.toml"),
        },
        Fixture {
            path: "package/forbidden-strings/core/Cargo.toml",
            stale: include_str!("fixtures/drift/forbidden-core.stale.toml"),
            expected: include_str!("fixtures/drift/forbidden-core.expected.toml"),
        },
    ];
}

/// The frozen catalog narrowed to the four fixture crates.
///
/// The full catalog holds entries only other crates use and has `catalogPrune = true`,
/// so syncing four crates
/// against it prunes the rest;
/// that pruning is the only catalog change,
/// and the narrowed text is the catalog the
/// remaining tests use.
fn narrowed_catalog() -> String {
    let manifests: Vec<ManifestFile> = stale_manifests();
    let input = SyncInput {
        catalog_path: "rust-catalog.toml",
        catalog_source: CATALOG,
        manifests: &manifests,
    };
    let output: SyncOutput = sync(&input).expect("fixtures sync");
    assert!(!output.catalog.changes.is_empty());
    for change in &output.catalog.changes {
        assert!(matches!(change, Change::CatalogEntryPruned { .. }), "unexpected catalog change {change:?}");
    }
    return output.catalog.expected;
}

/// The stale manifests as library input.
fn stale_manifests() -> Vec<ManifestFile> {
    let mut manifests: Vec<ManifestFile> = Vec::new();
    for fixture in fixtures() {
        manifests.push(ManifestFile {
            path: fixture.path.to_string(),
            source: fixture.stale.to_string(),
        });
    }
    return manifests;
}

/// Every stale manifest becomes the TypeScript output,
/// byte for byte.
#[test]
fn stale_manifests_become_the_typescript_output() {
    let catalog: String = narrowed_catalog();
    let manifests: Vec<ManifestFile> = stale_manifests();
    let input = SyncInput {
        catalog_path: "rust-catalog.toml",
        catalog_source: &catalog,
        manifests: &manifests,
    };
    let output: SyncOutput = sync(&input).expect("fixtures sync");
    assert_eq!(output.catalog.expected, catalog);
    for fixture in fixtures() {
        assert_eq!(output.expected_manifest(fixture.path), Some(fixture.expected), "{}", fixture.path);
    }
}

/// Check mode names each drifting crate and why:
/// the generated table was rewritten.
#[test]
fn check_mode_names_the_drifting_crates() {
    let catalog: String = narrowed_catalog();
    let manifests: Vec<ManifestFile> = stale_manifests();
    let input = SyncInput {
        catalog_path: "rust-catalog.toml",
        catalog_source: &catalog,
        manifests: &manifests,
    };
    let report = check(&input).expect("fixtures check");
    assert!(!report.is_clean());
    assert_eq!(report.drifts.len(), 4);
    for drift in &report.drifts {
        assert_eq!(drift.changes, vec![Change::GeneratedTableWritten], "{}", drift.path);
        assert_ne!(drift.actual, drift.expected);
    }
    assert_eq!(report.affected_crates.len(), 4);
}

/// The expected manifests are a fixed point:
/// syncing them again drifts nothing.
#[test]
fn expected_manifests_are_a_fixed_point() {
    let mut manifests: Vec<ManifestFile> = Vec::new();
    for fixture in fixtures() {
        manifests.push(ManifestFile {
            path: fixture.path.to_string(),
            source: fixture.expected.to_string(),
        });
    }
    let catalog: String = narrowed_catalog();
    let input = SyncInput {
        catalog_path: "rust-catalog.toml",
        catalog_source: &catalog,
        manifests: &manifests,
    };
    let report = check(&input).expect("expected fixtures check");
    assert!(report.is_clean());
    assert!(report.affected_crates.is_empty());
}
