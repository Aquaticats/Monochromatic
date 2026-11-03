//! What:
//! The fingerprint stamp that lets a cargo call skip the install check: after the authoritative check says a workspace's
//! `Cargo.lock` satisfies its manifests, the stamp records the size and modification time of the lockfile and of every
//! manifest the lockfile was resolved from.
//! Why:
//! The shadow sits on every cargo call, and the authoritative check (`cargo update --workspace --locked --offline`) costs a
//! cargo start plus index reads; measured on the largest lockfile in the repository it took 0.20 to 0.29 seconds. If every
//! recorded file still has the same size and modification time, nothing the check reads has changed, so the answer is still
//! "satisfied" and no process is started. This is the same test cargo's own fingerprints use for source files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stamp.ts
//! export function isFresh(stampFile: string): boolean
//! export function record(args: { stampFile: string; files: string[] }): void
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde_json::Value;

/// Where stamps live, relative to the repository root (gitignored with the rest of `.cache`).
pub const STAMP_DIR: &str = ".cache/cargo-shadow/stamps";

/// The stamp file of a crate root.
pub fn stamp_file(catalog_root: &Path, crate_dir: &str) -> PathBuf {
    return catalog_root.join(STAMP_DIR).join(format!("{}.json", crate_dir.replace('/', "__")));
}

/// A file's modification time in nanoseconds since 1970 and its length, as text, or `None` when it cannot be read.
fn fingerprint(path: &Path) -> Option<(String, String)> {
    let metadata = fs::metadata(path).ok()?;
    let nanos: u128 = metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    return Some((nanos.to_string(), metadata.len().to_string()));
}

/// What:
/// `pub fn is_fresh(stamp: &Path, lock: &Path) -> bool` says whether the stamp lists the lockfile and every listed file still
/// has the recorded size and modification time.
/// Why:
/// Any missing stamp, unreadable file, or difference answers `false`, which sends the caller to the authoritative check; a
/// wrong `false` costs time, a wrong `true` would let cargo rewrite the lockfile, so every doubt is `false`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isFresh(stamp: string, lock: string): boolean
/// ```
pub fn is_fresh(stamp: &Path, lock: &Path) -> bool {
    let text: String = match fs::read_to_string(stamp) {
        Ok(found) => found,
        Err(_) => return false,
    };
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(found) => found,
        Err(_) => return false,
    };
    let files: &Vec<Value> = match parsed.get("files").and_then(|value| { return value.as_array(); }) {
        Some(found) => found,
        None => return false,
    };
    let lock_text: String = lock.display().to_string();
    let mut lists_the_lock: bool = false;
    for entry in files {
        let path: &str = entry.get("path").and_then(|value| { return value.as_str(); }).unwrap_or("");
        let nanos: &str = entry.get("mtimeNanos").and_then(|value| { return value.as_str(); }).unwrap_or("");
        let length: &str = entry.get("length").and_then(|value| { return value.as_str(); }).unwrap_or("");
        if path == lock_text {
            lists_the_lock = true;
        }
        match fingerprint(Path::new(path)) {
            Some((current_nanos, current_length)) if current_nanos == nanos && current_length == length => {}
            _ => return false,
        }
    }
    return lists_the_lock;
}

/// Writes the stamp for the lockfile and the manifests; a failure to write only costs the next call a check.
pub fn record(stamp: &Path, lock: &Path, manifests: &[PathBuf]) {
    let mut entries: Vec<Value> = Vec::new();
    let mut all: Vec<&Path> = vec![lock];
    for manifest in manifests {
        all.push(manifest);
    }
    for path in all {
        let (nanos, length) = match fingerprint(path) {
            Some(found) => found,
            None => return,
        };
        let mut entry: serde_json::Map<String, Value> = serde_json::Map::new();
        entry.insert("path".to_string(), Value::String(path.display().to_string()));
        entry.insert("mtimeNanos".to_string(), Value::String(nanos));
        entry.insert("length".to_string(), Value::String(length));
        entries.push(Value::Object(entry));
    }
    let mut body: serde_json::Map<String, Value> = serde_json::Map::new();
    body.insert("files".to_string(), Value::Array(entries));
    if let Some(parent) = stamp.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    let _ = fs::write(stamp, format!("{}\n", Value::Object(body)));
}

/// The manifests a `cargo metadata` document says the lockfile was resolved from: every package without a registry or git
/// source (workspace members and path dependencies) plus the workspace root manifest.
pub fn manifests_from_metadata(metadata_json: &str) -> Vec<PathBuf> {
    let parsed: Value = match serde_json::from_str(metadata_json) {
        Ok(found) => found,
        Err(_) => return Vec::new(),
    };
    let mut manifests: Vec<PathBuf> = Vec::new();
    if let Some(root) = parsed.get("workspace_root").and_then(|value| { return value.as_str(); }) {
        manifests.push(Path::new(root).join("Cargo.toml"));
    }
    let empty: Vec<Value> = Vec::new();
    for package in parsed.get("packages").and_then(|value| { return value.as_array(); }).unwrap_or(&empty) {
        let has_source: bool = !package.get("source").map(|value| { return value.is_null(); }).unwrap_or(true);
        if has_source {
            continue;
        }
        if let Some(path) = package.get("manifest_path").and_then(|value| { return value.as_str(); }) {
            manifests.push(PathBuf::from(path));
        }
    }
    manifests.sort();
    manifests.dedup();
    return manifests;
}
