//! What:
//! The equivalence test:
//! run the library over this repository's real `rust-catalog.toml` and every real
//! Cargo workspace root,
//! and require exactly what is committed.
//! Why:
//! The TypeScript tool's output is the committed state (file-enforcer writes it and CI checks it),
//! so a
//! sync that changes no byte proves the Rust library reproduces the TypeScript behavior on real input,
//! including every comment,
//! key order and spacing that `toml_edit` must print back unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = syncRustCatalog({ catalogPath, catalogSource, manifests: await readManifests(globs) });
//! expect(result.catalog).toBe(catalogSource);
//! expect(result.affectedCrates).toEqual([]);
//! ```

mod support;

/// Imports `monochromatic_dependency_catalog::{ManifestFile, SyncInput, SyncOutput, check, sync}`.
use monochromatic_dependency_catalog::{ManifestFile, SyncInput, SyncOutput, check, sync};

/// Imports `crate::support::{read_catalog, read_manifests, repository_root}`.
use crate::support::{read_catalog, read_manifests, repository_root};

/// The real catalog and manifests,
/// and the sync output over them.
fn sync_real_repository() -> (String, Vec<ManifestFile>, SyncOutput) {
    let root = repository_root();
    let catalog_source: String = read_catalog(&root);
    let manifests: Vec<ManifestFile> = read_manifests(&root);
    let input = SyncInput {
        catalog_path: "rust-catalog.toml",
        catalog_source: &catalog_source,
        manifests: &manifests,
    };
    let output: SyncOutput = sync(&input).expect("the committed catalog and manifests sync without problems");
    return (catalog_source, manifests, output);
}

/// The discovery finds the real workspace roots,
/// so a broken glob cannot pass the equivalence test vacuously.
#[test]
fn discovers_the_real_workspace_roots() {
    let (_, manifests, _) = sync_real_repository();
    let paths: Vec<&str> = manifests.iter().map(|manifest| return manifest.path.as_str()).collect();
    assert!(paths.contains(&"package/mpm-shadow/cli/Cargo.toml"));
    assert!(paths.contains(&"package/music-player/android-app/rust/Cargo.toml"));
    assert!(paths.contains(&"package/jsonc-edit/core-rust.fuzz/Cargo.toml"));
    assert!(paths.contains(&"package/dependency-catalog/core-native/Cargo.toml"));
    assert!(manifests.len() >= 25, "expected at least 25 workspace roots, found {}", manifests.len());
}

/// The sync over the committed files changes no byte of the catalog or of any manifest.
#[test]
fn the_committed_repository_is_a_fixed_point() {
    let (catalog_source, manifests, output) = sync_real_repository();
    assert_eq!(output.catalog.expected, catalog_source, "rust-catalog.toml drifts");
    assert_eq!(output.manifests.len(), manifests.len());
    for (manifest, result) in manifests.iter().zip(output.manifests.iter()) {
        assert_eq!(result.path, manifest.path);
        assert_eq!(result.expected, manifest.source, "{} drifts: {:?}", manifest.path, result.changes);
        assert!(result.changes.is_empty(), "{} reports changes: {:?}", manifest.path, result.changes);
    }
    assert!(output.affected_crates.is_empty(), "affected crates: {:?}", output.affected_crates);
    assert!(output.warnings.is_empty(), "warnings: {:?}", output.warnings);
}

/// Check mode agrees:
/// no file drifts.
#[test]
fn check_mode_reports_a_clean_repository() {
    let root = repository_root();
    let catalog_source: String = read_catalog(&root);
    let manifests: Vec<ManifestFile> = read_manifests(&root);
    let input = SyncInput {
        catalog_path: "rust-catalog.toml",
        catalog_source: &catalog_source,
        manifests: &manifests,
    };
    let report = check(&input).expect("the committed repository checks without problems");
    let drifting: Vec<String> = report.drifts.iter().map(|file| return file.path.clone()).collect();
    assert!(report.is_clean(), "drift: {drifting:?}");
}
