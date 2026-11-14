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

/// Imports `BTreeSet`,
/// the sorted set of tool directory names.
use std::collections::BTreeSet;
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `SystemTime` from `std::time`.
use std::time::SystemTime;

/// Imports what this module uses from `super::cargo_installs` (the `cargo` namespace and the `current` link).
use super::cargo_installs::{CARGO_NAMESPACE, CURRENT_LINK_NAME};
/// Imports `install_dir_name` from `super::layout`.
use super::layout::install_dir_name;
/// Imports `Route` from `super::model`.
use super::model::Route;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

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
            let mut bytes: u64 = tree_size(&directory);
            if let Some(downloads_path) = &downloads {
                bytes = bytes.saturating_add(tree_size(downloads_path));
            }
            found.push(VersionDirectory { tool: tool.clone(), version, directory, downloads, bytes, written });
        }
    }
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
/// Every remove-in-progress directory a cut-short removal left,
/// in the tools' directories and in their downloads directories.
/// Why:
/// The next run finishes them,
/// and until it does they are the store's own record that a version is partly gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function tombstones(root: string, tools: ReadonlySet<string>): string[]
/// ```
pub fn tombstones(toolchain_root: &Path, tools: &BTreeSet<String>) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for tool in tools {
        for parent in [toolchain_root.join(tool), toolchain_root.join(DOWNLOADS_DIRECTORY_NAME).join(tool)] {
            let Ok(entries) = fs::read_dir(&parent) else {
                continue;
            };
            for entry in entries.flatten() {
                let name: String = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(TOMBSTONE_PREFIX) {
                    found.push(entry.path());
                }
            }
        }
    }
    found.sort();
    return found;
}
