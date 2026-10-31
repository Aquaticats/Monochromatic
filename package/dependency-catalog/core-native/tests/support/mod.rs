//! What:
//! Test support:
//! reads a repository tree's catalog and manifests the way the file-enforcer globs do.
//! Why:
//! The library itself never touches the file system,
//! so only tests need this.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test helper
//! export async function readManifests(root: string): Promise<ManifestFile[]>
//! ```

/// Imports `std::fs`.
use std::fs;
/// Imports `std::path::{Path, PathBuf}`.
use std::path::{Path, PathBuf};

/// Imports `monochromatic_dependency_catalog::ManifestFile`.
use monochromatic_dependency_catalog::ManifestFile;

/// Repository root,
/// three directories above this crate (`package/dependency-catalog/core-native`).
pub fn repository_root() -> PathBuf {
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// Lists the sub-directories of `directory`,
/// sorted by name;
/// empty when it cannot be read.
fn sub_directories(directory: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            found.push(entry.path());
        }
    }
    found.sort();
    return found;
}

/// What:
/// `pub fn read_manifests(root: &Path) -> Vec<ManifestFile>` reads every first-party crate manifest the
/// file-enforcer globs `package/*/*/Cargo.toml` and `package/*/*/*/Cargo.toml` match,
/// sorted by path.
/// Why:
/// The same bounded-depth discovery the TypeScript tool used;
/// `node_modules` and `target` copies are
/// skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paths = [...await glob('package/*/*/Cargo.toml'), ...await glob('package/*/*/*/Cargo.toml')].toSorted();
/// ```
pub fn read_manifests(root: &Path) -> Vec<ManifestFile> {
    let mut level: Vec<PathBuf> = vec![root.join("package")];
    let mut manifests: Vec<ManifestFile> = Vec::new();
    for depth in 1..=3 {
        let mut next: Vec<PathBuf> = Vec::new();
        for directory in &level {
            for child in sub_directories(directory) {
                let name: String = child.file_name().map(|part| return part.to_string_lossy().to_string()).unwrap_or_default();
                if name == "node_modules" || name == "target" {
                    continue;
                }
                next.push(child);
            }
        }
        level = next;
        if depth < 2 {
            continue;
        }
        for directory in &level {
            let manifest: PathBuf = directory.join("Cargo.toml");
            if manifest.is_file() {
                let source: String = fs::read_to_string(&manifest).expect("a manifest the glob matched is readable");
                let relative: String = manifest
                    .strip_prefix(root)
                    .expect("a manifest lives under the root")
                    .to_string_lossy()
                    .replace('\\', "/");
                manifests.push(ManifestFile { path: relative, source });
            }
        }
    }
    manifests.sort_by(|left: &ManifestFile, right: &ManifestFile| return left.path.cmp(&right.path));
    return manifests;
}

/// Reads `rust-catalog.toml` under `root`.
pub fn read_catalog(root: &Path) -> String {
    return fs::read_to_string(root.join("rust-catalog.toml")).expect("rust-catalog.toml is readable");
}
