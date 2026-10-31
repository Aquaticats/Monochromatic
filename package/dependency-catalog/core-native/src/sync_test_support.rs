//! What:
//! Fixtures and runners shared by the sync test files.
//! Why:
//! Both the edit tests and the issue tests build the same manifests and run the same sync.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync.unit.test.ts (fixtures section)
//! function text(lines: string[]): string
//! function crate(args: { dir: string; body: string[]; version?: string }): ManifestFile
//! ```

/// Imports `crate::crate_info::ManifestFile`.
use crate::crate_info::ManifestFile;
/// Imports `crate::error::Error`.
use crate::error::Error;
/// Imports `crate::sync::{SyncInput, SyncOutput, sync}`.
use crate::sync::{SyncInput, SyncOutput, sync};

/// Joins lines into file text with a trailing newline.
pub(crate) fn text(lines: &[&str]) -> String {
    return format!("{}\n", lines.join("\n"));
}

/// Builds a crate manifest fixture:
/// `[package]`,
/// the body,
/// then an empty `[workspace]`.
pub(crate) fn crate_with_version(dir: &str, body: &[&str], version: &str) -> ManifestFile {
    let name: &str = dir.rsplit('/').next().unwrap_or("x");
    let mut lines: Vec<String> = vec![
        "[package]".to_string(),
        format!("name = \"{name}\""),
        format!("version = \"{version}\""),
        String::new(),
    ];
    for line in body {
        lines.push((*line).to_string());
    }
    lines.push(String::new());
    lines.push("[workspace]".to_string());
    return ManifestFile {
        path: format!("{dir}/Cargo.toml"),
        source: format!("{}\n", lines.join("\n")),
    };
}

/// A crate at version 0.1.0.
pub(crate) fn krate(dir: &str, body: &[&str]) -> ManifestFile {
    return crate_with_version(dir, body, "0.1.0");
}

/// Runs the sync on catalog lines and manifests.
pub(crate) fn run(catalog: &[&str], manifests: &[ManifestFile]) -> Result<SyncOutput, Error> {
    let catalog_text: String = text(catalog);
    let input = SyncInput {
        catalog_path: "rust-catalog.toml",
        catalog_source: &catalog_text,
        manifests,
    };
    return sync(&input);
}

/// Runs the sync expecting success.
pub(crate) fn run_ok(catalog: &[&str], manifests: &[ManifestFile]) -> SyncOutput {
    return run(catalog, manifests).expect("sync succeeds");
}

/// Runs the sync expecting a sync error and returns each issue as `file location`.
pub(crate) fn issues(catalog: &[&str], manifests: &[ManifestFile]) -> Vec<String> {
    return match run(catalog, manifests) {
        Err(Error::Sync { issues: found }) => found.into_iter().map(|issue| return format!("{} {}", issue.file, issue.location)).collect(),
        other => panic!("expected a sync error, got {other:?}"),
    };
}

/// The expected text of one manifest by crate directory.
pub(crate) fn manifest_text(output: &SyncOutput, dir: &str) -> String {
    return output
        .expected_manifest(&format!("{dir}/Cargo.toml"))
        .expect("manifest present")
        .to_string();
}
