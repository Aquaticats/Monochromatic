//! What:
//! The version directories an mpm-shadow store holds:
//! where each tool's versions are,
//! the space each version costs on disk,
//! when it was last written,
//! the downloads kept for it,
//! and the remove-in-progress names a removal cut short leaves behind.
//! Why:
//! The cleanup decides by size and by write time,
//! so both must be read from the store itself and from nothing else.
//! A removal renames its version out of the version's name first
//! (one atomic step),
//! so a reader never sees a half version under a version's name,
//! and the name it is moved to is what the next run finishes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // store-versions.ts
//! export function installDirectoryName(route: string, id: string): string | undefined
//! export function versionDirectories(root: string, tools: ReadonlySet<string>): VersionDirectory[]
//! export function leftoverDownloads(root: string, tools: ReadonlySet<string>): LeftoverDownload[]
//! export function tombstones(root: string, tools: ReadonlySet<string>): string[]
//! export function tombstoneOf(versionDirectory: string): string
//! export function treeSize(path: string): number
//! ```

/// Imports `BTreeMap` and `BTreeSet`,
/// the sorted map of the size records and the sorted set of tool directory names.
use std::collections::{BTreeMap, BTreeSet};
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `SystemTime` and `UNIX_EPOCH` from `std::time`.
use std::time::{SystemTime, UNIX_EPOCH};

/// Imports `Value` from `serde_json`,
/// which reads the size records.
use serde_json::Value;

/// Imports what this module uses from `super::cargo_installs` (the `cargo` namespace and the `current` link).
use super::cargo_installs::{CARGO_NAMESPACE, CURRENT_LINK_NAME};
/// Imports `install_dir_name` from `super::layout`.
use super::layout::install_dir_name;
/// Imports `Route` from `super::model`.
use super::model::Route;
/// Imports `file_name_text` and `write_atomically` from `super::util`.
use super::util::{file_name_text, write_atomically};

/// The directory below a toolchain root that holds the archives downloaded for each version.
pub const DOWNLOADS_DIRECTORY_NAME: &str = "downloads";

/// The prefix of the name a version is moved to while it is being removed:
/// `.<version>.mpm-shadow-removing` beside the version,
/// so every scan skips it (they all skip dot-names)
/// and the next run can finish it.
pub const TOMBSTONE_PREFIX: &str = ".mpm-shadow-removing.";

/// The suffix of a download that is still being written,
/// which is never a leftover to remove.
pub const IN_FLIGHT_DOWNLOAD_SUFFIX: &str = ".download";

/// What:
/// The file in a store root that records what each version directory weighed when it was last measured:
/// `.mpm-shadow-version-sizes.json`.
/// Why:
/// Measuring a store costs 9.3 s on this machine (103 version directories, 54.5 GiB, read on 2026-11-13),
/// and a pass that reads the records instead takes 1 ms,
/// and a version directory's own write time changes exactly when its content changes (`install` renames a finished
/// tree into place, so the whole version arrives as one rename),
/// so one `stat` per version tells whether the recorded size still holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SIZE_RECORDS_FILE_NAME = '.mpm-shadow-version-sizes.json';
/// ```
pub const SIZE_RECORDS_FILE_NAME: &str = ".mpm-shadow-version-sizes.json";

/// What one version directory weighed,
/// keyed by the write time it had.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SizeRecord {
    /// The version directory's own write time,
    /// in nanoseconds after the epoch.
    pub written_ns: u64,
    /// The bytes the version and the downloads kept for it weighed.
    pub bytes: u64,
}

/// The nanoseconds `written` lies after the epoch,
/// and zero for a time before it.
fn written_ns(written: SystemTime) -> u64 {
    return match written.duration_since(UNIX_EPOCH) {
        Ok(since) => u64::try_from(since.as_nanos()).unwrap_or(u64::MAX),
        Err(_) => 0,
    };
}

/// What:
/// The size records of the store at `toolchain_root`,
/// keyed by `<tool>/<version>`,
/// read from [`SIZE_RECORDS_FILE_NAME`].
/// Why:
/// A file that is missing,
/// unreadable or corrupt reads as no records,
/// so the store is measured again rather than trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readSizeRecords(root: string): Map<string, SizeRecord>
/// ```
pub fn read_size_records(toolchain_root: &Path) -> BTreeMap<String, SizeRecord> {
    let mut records: BTreeMap<String, SizeRecord> = BTreeMap::new();
    let Ok(text) = fs::read_to_string(toolchain_root.join(SIZE_RECORDS_FILE_NAME)) else {
        return records;
    };
    let Ok(decoded) = serde_json::from_str::<Value>(&text) else {
        return records;
    };
    let Some(versions) = decoded.get("versions").and_then(Value::as_object) else {
        return records;
    };
    for (key, record) in versions {
        let Some(written_ns) = record.get("writtenNs").and_then(Value::as_u64) else {
            continue;
        };
        let Some(bytes) = record.get("bytes").and_then(Value::as_u64) else {
            continue;
        };
        records.insert(key.clone(), SizeRecord { written_ns, bytes });
    }
    return records;
}

/// What:
/// Writes the size records of the store at `toolchain_root`.
/// Why:
/// A record that cannot be written costs the next pass a measurement and nothing else,
/// so a failure is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function writeSizeRecords(root: string, records: Map<string, SizeRecord>): void
/// ```
pub fn write_size_records(toolchain_root: &Path, records: &BTreeMap<String, SizeRecord>) {
    let mut versions: serde_json::Map<String, Value> = serde_json::Map::new();
    for (key, record) in records {
        let mut fields: serde_json::Map<String, Value> = serde_json::Map::new();
        fields.insert("writtenNs".to_string(), Value::from(record.written_ns));
        fields.insert("bytes".to_string(), Value::from(record.bytes));
        versions.insert(key.clone(), Value::Object(fields));
    }
    let mut document: serde_json::Map<String, Value> = serde_json::Map::new();
    document.insert("schemaVersion".to_string(), Value::from(1));
    document.insert("versions".to_string(), Value::Object(versions));
    let text: String = Value::Object(document).to_string();
    let _ = write_atomically(&toolchain_root.join(SIZE_RECORDS_FILE_NAME), text.as_bytes());
}

/// What:
/// The directory under a toolchain root whose children are the versions of one tool,
/// relative to the root (`node`,
/// `cargo/apple-codesign`),
/// or `None` for a route that keeps its payload outside the store's version directories.
/// Why:
/// The scan and the "this run installed it" check must agree on where a tool's versions live,
/// so both ask this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installDirectoryName(route: Route, id: string): string | undefined
/// ```
pub fn install_directory_name(route: Route, id: &str) -> Option<String> {
    return match route {
        // A crate `cargo binstall` installs gets one directory per crate under the store's `cargo` namespace.
        Route::CargoBinstall => {
            let crate_name: &str =
                if let Some(rest) = id.strip_prefix("cargo:") { rest } else { id };
            Some(format!("{CARGO_NAMESPACE}/{crate_name}"))
        }
        // An archive, a Go module, the Android SDK and a crate installed from git each own a top-level directory
        // named after the tool's id, with one directory per version below it.
        Route::Archive | Route::GoInstall | Route::AndroidSdk | Route::CargoGit => Some(install_dir_name(id)),
        // pnpm, uv, rustup and the `bin` downloader keep their payload in their own home, not in a version directory.
        Route::MpmPnpm | Route::MpmUvx | Route::RustupToolchain | Route::RustupComponents | Route::BinSourced => None,
    };
}

/// One version directory of a tool in a store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionDirectory {
    /// The tool's directory name in the store (`node`,
    /// `cargo/apple-codesign`).
    pub tool: String,
    /// The version (the directory's name).
    pub version: String,
    /// The version directory itself.
    pub directory: PathBuf,
    /// The directory holding the archives downloaded for this version,
    /// when the store keeps any.
    pub downloads: Option<PathBuf>,
    /// The size the version and its downloads weigh, in bytes.
    pub bytes: u64,
    /// When the version directory was last written.
    pub written: SystemTime,
}

/// A download an earlier layout left directly under the store's `downloads` directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeftoverDownload {
    /// The tool the file's name belongs to.
    pub tool: String,
    /// The file.
    pub file: PathBuf,
    /// The size it weighs, in bytes.
    pub bytes: u64,
    /// When it was last written.
    pub written: SystemTime,
}

/// What:
/// The size `path` weighs,
/// every file below it counted by its length,
/// links not followed.
/// Why:
/// A version is removed for the space it holds,
/// and the length of every file is the measure a person can reproduce from a listing
/// (the replay of a real store's inventory builds sparse files of exactly these lengths).
/// A work stack walks the tree:
/// a deep directory cannot overflow the call stack.
/// Anything that cannot be read counts as nothing:
/// the size is what a bound is compared with, never a reason to remove something.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function treeSize(path: string): number
/// ```
pub fn tree_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    let mut pending: Vec<PathBuf> = vec![path.to_path_buf()];
    while let Some(current) = pending.pop() {
        let Ok(metadata) = fs::symlink_metadata(&current) else {
            continue;
        };
        if metadata.is_dir() {
            let Ok(children) = fs::read_dir(&current) else {
                continue;
            };
            for child in children.flatten() {
                pending.push(child.path());
            }
            continue;
        }
        total = total.saturating_add(metadata.len());
    }
    return total;
}

/// The directory holding the archives downloaded for one version of one tool.
pub fn downloads_of(toolchain_root: &Path, tool: &str, version: &str) -> PathBuf {
    return toolchain_root
        .join(DOWNLOADS_DIRECTORY_NAME)
        .join(tool)
        .join(version);
}

/// Whether `entry` is a plain directory whose name can be a version's:
/// not hidden,
/// not the tool's `current` link.
fn is_version_entry(entry: &fs::DirEntry, name: &str) -> bool {
    if name.starts_with('.') || name == CURRENT_LINK_NAME {
        return false;
    }
    return match entry.file_type() {
        Ok(kind) => kind.is_dir(),
        Err(_) => false,
    };
}

/// What:
/// Every version directory of every tool in `tools`,
/// with the space it occupies and when it was last written,
/// newest write last.
/// Why:
/// This is the whole inventory the cleanup decides on.
/// A tool directory that is missing or unreadable holds no version,
/// and a version's downloads are counted with it:
/// they are kept for that version and go with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function versionDirectories(root: string, tools: ReadonlySet<string>): VersionDirectory[]
/// ```
pub fn version_directories(toolchain_root: &Path, tools: &BTreeSet<String>) -> Vec<VersionDirectory> {
    let recorded: BTreeMap<String, SizeRecord> = read_size_records(toolchain_root);
    let mut measured: BTreeMap<String, SizeRecord> = BTreeMap::new();
    let mut found: Vec<VersionDirectory> = Vec::new();
    for tool in tools {
        let tool_directory: PathBuf = toolchain_root.join(tool);
        let Ok(entries) = fs::read_dir(&tool_directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let version: String = entry.file_name().to_string_lossy().to_string();
            if !is_version_entry(&entry, &version) {
                continue;
            }
            let directory: PathBuf = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&directory) else {
                continue;
            };
            let Ok(written) = metadata.modified() else {
                continue;
            };
            let downloads_directory: PathBuf = downloads_of(toolchain_root, tool, &version);
            let downloads: Option<PathBuf> = match fs::symlink_metadata(&downloads_directory) {
                Ok(found_downloads) if found_downloads.is_dir() => Some(downloads_directory),
                _ => None,
            };
            // What: a version whose directory still has the write time it had when it was measured keeps the
            // recorded size; any other is measured now.
            // Why: measuring the whole store costs seconds, and a version directory's write time changes when an
            // install replaces the version, so one `stat` per version answers whether the record still holds.
            let key: String = format!("{tool}/{version}");
            let recorded_ns: u64 = written_ns(written);
            let bytes: u64 = match recorded.get(&key) {
                Some(record) if record.written_ns == recorded_ns => record.bytes,
                _ => {
                    let mut measured_bytes: u64 = tree_size(&directory);
                    if let Some(downloads_path) = &downloads {
                        measured_bytes = measured_bytes.saturating_add(tree_size(downloads_path));
                    }
                    measured_bytes
                }
            };
            measured.insert(key, SizeRecord { written_ns: recorded_ns, bytes });
            found.push(VersionDirectory { tool: tool.clone(), version, directory, downloads, bytes, written });
        }
    }
    write_size_records(toolchain_root, &measured);
    // Oldest write first: the cleanup removes in this order.
    found.sort_by(|left, right| {
        let by_write: std::cmp::Ordering = left.written.cmp(&right.written);
        if by_write != std::cmp::Ordering::Equal {
            return by_write;
        }
        return left.directory.cmp(&right.directory);
    });
    return found;
}

/// What:
/// The download files an earlier layout left directly under the store's `downloads` directory,
/// each with the tool its name begins with.
/// Why:
/// Those files are named `<tool>-<version>-<file>` by a release that had no per-version directories,
/// nothing reads them any more,
/// and they are the largest thing an install leaves behind (4.6 GiB on this machine on 2026-11-13).
/// A file still being written ends with `.download` and is never one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function leftoverDownloads(root: string, tools: ReadonlySet<string>): LeftoverDownload[]
/// ```
pub fn leftover_downloads(toolchain_root: &Path, tools: &BTreeSet<String>) -> Vec<LeftoverDownload> {
    let downloads_directory: PathBuf = toolchain_root.join(DOWNLOADS_DIRECTORY_NAME);
    let Ok(entries) = fs::read_dir(&downloads_directory) else {
        return Vec::new();
    };
    let mut found: Vec<LeftoverDownload> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name.ends_with(IN_FLIGHT_DOWNLOAD_SUFFIX) {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            continue;
        }
        let file: PathBuf = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&file) else {
            continue;
        };
        let Ok(written) = metadata.modified() else {
            continue;
        };
        for tool in tools {
            // A tool whose versions live under the `cargo` namespace has no flat download name of its own, and a name
            // with a separator in it cannot be a file name.
            if tool.contains('/') {
                continue;
            }
            if !name.starts_with(&format!("{tool}-")) {
                continue;
            }
            found.push(LeftoverDownload { tool: tool.clone(), file: file.clone(), bytes: metadata.len(), written });
            break;
        }
    }
    return found;
}

/// The name a version directory is moved to while it is removed.
pub fn tombstone_of(directory: &Path) -> PathBuf {
    let name: String = file_name_text(directory);
    return directory.with_file_name(format!("{TOMBSTONE_PREFIX}{name}"));
}

/// What:
/// One removal an earlier run started and did not finish:
/// the version it moved out of its name,
/// where that version belongs,
/// and the downloads it moved aside with it.
/// Why:
/// The next run needs all of that to finish the removal or to undo it,
/// and it must be readable from the store alone:
/// a lock that cannot be read may leave the set of tool names empty,
/// which must not look like "no removal is pending".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalMarker = { root: string; tool: string; version: string; versionMarker: string;
///   versionDirectory: string; downloadsMarker?: string; downloadsDirectory?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovalMarker {
    /// The store root the marker stands in.
    pub root: PathBuf,
    /// The tool's directory name in the store (`node`,
    /// `cargo/apple-codesign`),
    /// or empty when the marker sits in a place that names no tool.
    pub tool: String,
    /// The version,
    /// taken from the marker's own name.
    pub version: String,
    /// The marker holding the version directory.
    pub version_marker: PathBuf,
    /// The name the version directory takes back when the removal is undone.
    pub version_directory: PathBuf,
    /// The marker holding the downloads of that version,
    /// when the removal moved some aside.
    pub downloads_marker: Option<PathBuf>,
    /// The name the downloads directory takes back.
    pub downloads_directory: Option<PathBuf>,
}

/// The names of the plain directories directly under `directory`,
/// hidden ones left out.
fn subdirectory_names(directory: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if entry.file_type().map(|kind| return kind.is_dir()).unwrap_or(false) {
            names.push(name);
        }
    }
    names.sort();
    return names;
}

/// The removal markers directly inside `directory`,
/// each with the version its name carries.
fn markers_in(directory: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let Some(version) = name.strip_prefix(TOMBSTONE_PREFIX) else {
            continue;
        };
        found.push((version.to_string(), entry.path()));
    }
    found.sort();
    return found;
}

/// What:
/// Every tool directory of the store at `toolchain_root` whose children can be versions:
/// the top-level ones and the crates under `cargo`.
/// Why:
/// The removal markers are found by looking at the store,
/// so a lock that cannot be read never hides one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function toolDirectoriesOfStore(root: string): string[]
/// ```
pub fn tool_directories_of_store(toolchain_root: &Path) -> Vec<String> {
    let mut tools: Vec<String> = Vec::new();
    for name in subdirectory_names(toolchain_root) {
        if name == DOWNLOADS_DIRECTORY_NAME {
            continue;
        }
        if name == CARGO_NAMESPACE {
            for crate_name in subdirectory_names(&toolchain_root.join(CARGO_NAMESPACE)) {
                tools.push(format!("{CARGO_NAMESPACE}/{crate_name}"));
            }
            continue;
        }
        tools.push(name);
    }
    return tools;
}

/// What:
/// Every removal an earlier run started and did not finish,
/// found by walking the store's own directories.
/// Why:
/// This is what the next run finishes or undoes,
/// and it answers whether the store holds a half-removed version regardless of the locks:
/// a lock that cannot be read empties the set of tool names,
/// which must not read as "nothing is pending".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removalMarkers(root: string): RemovalMarker[]
/// ```
pub fn removal_markers(toolchain_root: &Path) -> Vec<RemovalMarker> {
    let downloads_root: PathBuf = toolchain_root.join(DOWNLOADS_DIRECTORY_NAME);
    let mut found: BTreeMap<(String, String), RemovalMarker> = BTreeMap::new();
    for tool in tool_directories_of_store(toolchain_root) {
        for (version, version_marker) in markers_in(&toolchain_root.join(&tool)) {
            let downloads_marker: PathBuf = downloads_root.join(&tool).join(format!("{TOMBSTONE_PREFIX}{version}"));
            let markers_tool: String = tool.clone();
            let has_downloads: bool = fs::symlink_metadata(&downloads_marker).is_ok();
            found.insert(
                (tool.clone(), version.clone()),
                RemovalMarker {
                    root: toolchain_root.to_path_buf(),
                    version_directory: toolchain_root.join(&tool).join(&version),
                    version_marker,
                    downloads_marker: if has_downloads { Some(downloads_marker) } else { None },
                    downloads_directory: if has_downloads {
                        Some(downloads_root.join(&tool).join(&version))
                    } else {
                        None
                    },
                    tool: markers_tool,
                    version,
                },
            );
        }
    }
    // What: a downloads marker with no version marker beside it is filled in here.
    // Why: a kill between the two renames leaves only the downloads moved aside, and the next run must take the
    // version's name back from the marker's own name.
    for tool in subdirectory_names(&downloads_root) {
        for (version, downloads_marker) in markers_in(&downloads_root.join(&tool)) {
            let entry: &mut RemovalMarker = found.entry((tool.clone(), version.clone())).or_insert_with(|| {
                return RemovalMarker {
                    root: toolchain_root.to_path_buf(),
                    version_directory: toolchain_root.join(&tool).join(&version),
                    version_marker: toolchain_root.join(&tool).join(format!("{TOMBSTONE_PREFIX}{version}")),
                    downloads_marker: None,
                    downloads_directory: Some(downloads_root.join(&tool).join(&version)),
                    tool: tool.clone(),
                    version: version.clone(),
                };
            });
            entry.downloads_marker = Some(downloads_marker);
            entry.downloads_directory = Some(downloads_root.join(&tool).join(&version));
        }
    }
    return found.into_values().collect();
}
