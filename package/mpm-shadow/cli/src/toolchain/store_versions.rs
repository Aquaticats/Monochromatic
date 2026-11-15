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
/// Imports `Instant`,
/// `SystemTime` and `UNIX_EPOCH` from `std::time`.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

/// The prefix of the name a version is moved to when its deletion begins:
/// `.<version>.mpm-shadow-deleting` beside it.
/// Only the deletion step writes this name, so whatever stands under it is condemned and is finished, never restored.
pub const DELETING_PREFIX: &str = ".mpm-shadow-deleting.";

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
    /// The bytes the version weighed.
    pub bytes: u64,
    /// How many files stood in it.
    pub files: u64,
    /// The newest write inside it,
    /// in nanoseconds after the epoch.
    pub newest_ns: u64,
}

/// The nanoseconds `written` lies after the epoch,
/// and zero for a time before it.
pub fn written_ns(written: SystemTime) -> u64 {
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
        let Some(files) = record.get("files").and_then(Value::as_u64) else {
            continue;
        };
        let Some(newest_ns) = record.get("newestNs").and_then(Value::as_u64) else {
            continue;
        };
        records.insert(key.clone(), SizeRecord { written_ns, bytes, files, newest_ns });
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
        fields.insert("files".to_string(), Value::from(record.files));
        fields.insert("newestNs".to_string(), Value::from(record.newest_ns));
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
    /// The size the version's own tree weighs, in bytes, without the archives kept for it.
    pub tree_bytes: u64,
    /// When the version directory was last written.
    pub written: SystemTime,
    /// The device and inode number of the version directory,
    /// when the platform reports them:
    /// what tells this entry from one an install puts at the same name later.
    pub inode: Option<(u64, u64)>,
    /// How many files the tree held when it was read.
    pub files: u64,
    /// The newest write anywhere inside it when it was read.
    pub newest: SystemTime,
}

/// What:
/// The device and inode number `metadata` reports,
/// or `None` on a platform that reports none.
/// Why:
/// An inode number names one directory for as long as it exists,
/// and an install that replaces a version by an atomic rename puts a directory with another inode at that name,
/// so the pair is how a removal knows it holds the entry it judged and not a newer one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function inodeOf(metadata: Stats): [number, number] | undefined
/// ```
#[cfg(unix)]
pub fn inode_of(metadata: &fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    return Some((metadata.dev(), metadata.ino()));
}

/// The device and inode number `metadata` reports (none, on a platform that reports none).
#[cfg(not(unix))]
pub fn inode_of(_metadata: &fs::Metadata) -> Option<(u64, u64)> {
    return None;
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
    /// The device and inode number of the file,
    /// when the platform reports them.
    pub inode: Option<(u64, u64)>,
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
/// export function versionDirectories(root: string, tools: ReadonlySet<string>, ageFloor: Duration): VersionDirectory[]
/// ```
pub fn version_directories(
    toolchain_root: &Path,
    tools: &BTreeSet<String>,
    deadline: Instant,
    age_floor: Duration,
) -> Vec<VersionDirectory> {
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
            // What: a record is trusted only while what it has to prove is that the tree is young.
            // Why: a record was read at one moment and a write inside the tree leaves the root's own write time alone,
            // so the recorded newest write is a lower bound, never the truth. A recorded newest write inside the age
            // floor proves the version is inside the floor (a write only makes it newer); anything else is walked again.
            let floor_ns: u64 = written_ns(SystemTime::now()).saturating_sub(u64::try_from(age_floor.as_nanos()).unwrap_or(u64::MAX));
            let facts: TreeFacts = match recorded.get(&key) {
                Some(record) if record.written_ns == recorded_ns && record.newest_ns >= floor_ns => TreeFacts {
                    bytes: record.bytes,
                    files: record.files,
                    newest: UNIX_EPOCH + std::time::Duration::from_nanos(record.newest_ns),
                    complete: true,
                },
                _ => tree_facts(&directory, deadline),
            };
            // A tree this pass could not read whole is kept: its size is unknown, so it is not judged.
            if !facts.complete {
                continue;
            }
            let mut bytes: u64 = facts.bytes;
            if let Some(downloads_path) = &downloads {
                // What: the archives are weighed by the same deadline-bounded walk as the version.
                // Why: the work limit bounds the work itself, and a walk that ignores the deadline holds the install
                // for however long the filesystem takes.
                bytes = bytes.saturating_add(tree_facts(downloads_path, deadline).bytes);
            }
            measured.insert(
                key,
                SizeRecord { written_ns: recorded_ns, bytes: facts.bytes, files: facts.files, newest_ns: written_ns(facts.newest) },
            );
            let inode: Option<(u64, u64)> = inode_of(&metadata);
            found.push(VersionDirectory {
                tool: tool.clone(),
                version,
                directory,
                downloads,
                bytes,
                tree_bytes: facts.bytes,
                written,
                inode,
                files: facts.files,
                newest: facts.newest,
            });
        }
    }
    write_size_records(toolchain_root, &measured);
    // Oldest newest write first: the cleanup removes in this order, and a tree a write landed in is not old.
    found.sort_by(|left, right| {
        let left_newest: SystemTime = if left.newest > left.written { left.newest } else { left.written };
        let right_newest: SystemTime = if right.newest > right.written { right.newest } else { right.written };
        let by_write: std::cmp::Ordering = left_newest.cmp(&right_newest);
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
            let inode: Option<(u64, u64)> = inode_of(&metadata);
            found.push(LeftoverDownload { tool: tool.clone(), file: file.clone(), bytes: metadata.len(), written, inode });
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

/// The suffix of the sidecar that records what a removal judged of the tree it moved aside.
/// It sits beside the removal name, holds the tree's identity and the facts a walk of it read,
/// and is what lets a later pass tell a whole tombstone from one a deletion has begun inside.
pub const FACTS_SUFFIX: &str = ".facts";

/// The sidecar that records what a removal judged of the tree it moved aside,
/// beside the removal name.
pub fn facts_of(marker: &Path) -> PathBuf {
    let name: String = file_name_text(marker);
    return marker.with_file_name(format!("{name}{FACTS_SUFFIX}"));
}

/// The sidecar that describes the tree of the version at `version_directory`,
/// whether that tree stands under its own name,
/// its removal name or its deletion name.
pub fn record_of(version_directory: &Path) -> PathBuf {
    return facts_of(&tombstone_of(version_directory));
}

/// The version name a tree under a removal name or a deletion name carries,
/// or `None` when the name carries none of the two prefixes.
/// Why:
/// A deletion name is built from the name the tree stood under, so a tree whose deletion began from its removal name
/// carries both prefixes, and the record beside it names the version, not the name the tree has now.
pub fn version_name_of_marker(marker: &Path) -> Option<String> {
    let mut name: String = file_name_text(marker);
    let mut carried: Option<String> = None;
    loop {
        if let Some(version) = name.strip_prefix(TOMBSTONE_PREFIX) {
            carried = Some(version.to_string());
            name = version.to_string();
            continue;
        }
        if let Some(version) = name.strip_prefix(DELETING_PREFIX) {
            carried = Some(version.to_string());
            name = version.to_string();
            continue;
        }
        return carried;
    }
}

/// The record beside the removal name of the version a tree under either removal name belongs to.
pub fn record_of_marker(marker: &Path) -> Option<PathBuf> {
    let version: String = version_name_of_marker(marker)?;
    return Some(marker.with_file_name(format!("{TOMBSTONE_PREFIX}{version}{FACTS_SUFFIX}")));
}

/// The name a version directory is moved to when its deletion begins.
pub fn deleting_of(directory: &Path) -> PathBuf {
    let name: String = file_name_text(directory);
    return directory.with_file_name(format!("{DELETING_PREFIX}{name}"));
}

/// What:
/// What one walk of a tree read:
/// the space it weighs,
/// how many files it holds,
/// the newest write anywhere inside it,
/// and whether the walk reached the end.
/// Why:
/// The plan judges a version by what the whole tree was, not by its root alone:
/// a write into a folder that already existed changes the newest write and the file count,
/// and changes neither the root's inode nor its own write time.
/// `complete` is false when the deadline ran out,
/// and a tree the pass could not read whole is kept rather than removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function treeFacts(path: string, deadline: Instant): TreeFacts
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeFacts {
    /// The space the tree weighs,
    /// in bytes.
    pub bytes: u64,
    /// How many files stand in it.
    pub files: u64,
    /// The newest write time anywhere inside it,
    /// or the epoch for an empty tree.
    pub newest: SystemTime,
    /// Whether the walk reached the end before its deadline.
    pub complete: bool,
}

/// How many entries a walk reads between two reads of the clock.
const DEADLINE_CHECK_EVERY: u64 = 128;

/// What:
/// Walks `path` and reads its facts,
/// stopping at `deadline` and saying that it stopped.
/// Why:
/// The work limit has to bound the work itself, and the inventory is the largest piece of it;
/// a store that is slow to read leaves the rest of its trees for the next run,
/// which finds them under the same names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function treeFacts(path: string, deadline: Instant): TreeFacts
/// ```
pub fn tree_facts(path: &Path, deadline: Instant) -> TreeFacts {
    let mut facts: TreeFacts = TreeFacts { bytes: 0, files: 0, newest: UNIX_EPOCH, complete: true };
    if Instant::now() >= deadline {
        facts.complete = false;
        return facts;
    }
    let mut pending: Vec<PathBuf> = vec![path.to_path_buf()];
    let mut seen: u64 = 0;
    while let Some(current) = pending.pop() {
        seen = seen.saturating_add(1);
        if seen % DEADLINE_CHECK_EVERY == 0 && Instant::now() >= deadline {
            facts.complete = false;
            return facts;
        }
        let Ok(metadata) = fs::symlink_metadata(&current) else {
            // What: an entry of the tree that could not be read makes the walk partial.
            // Why: a tree read only in part proves nothing about what it holds, so the version is kept rather than
            // judged by a file count and a newest write that left the unread part out.
            facts.complete = false;
            return facts;
        };
        if metadata.is_dir() {
            let Ok(children) = fs::read_dir(&current) else {
                facts.complete = false;
                return facts;
            };
            for child in children.flatten() {
                pending.push(child.path());
            }
            continue;
        }
        facts.files = facts.files.saturating_add(1);
        facts.bytes = facts.bytes.saturating_add(metadata.len());
        if let Ok(written) = metadata.modified()
            && written > facts.newest
        {
            facts.newest = written;
        }
    }
    return facts;
}

/// What:
/// What a pass judged of one version's whole tree:
/// the identity that tells that tree from any other at the same name,
/// and the facts that change whenever anything inside it is written.
/// Why:
/// A record beside a removal name is the only thing a later pass has to tell a tree a removal moved aside whole
/// from one a deletion has begun inside;
/// the identity in it is what keeps a record that outlived its tree from describing a newer one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JudgedTree = { inode: [number, number] | undefined; written: number; facts: TreeFacts };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JudgedTree {
    /// The device and inode number of the tree's root when it was judged,
    /// when the platform reports them.
    pub inode: Option<(u64, u64)>,
    /// The root's own write time when it was judged.
    pub written: SystemTime,
    /// What a walk of the whole tree read when it was judged.
    pub facts: TreeFacts,
}

/// The methods of `JudgedTree`.
impl JudgedTree {
    /// What:
    /// The record's own text:
    /// `device inode files bytes newest_ns written_ns`,
    /// or `-` for the inode pair a platform does not report.
    /// Why:
    /// The record is read by a later run of this program only,
    /// and numbers keep it independent of any locale or format change.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function textOf(judged: JudgedTree): string
    /// ```
    pub fn text(&self) -> String {
        let inode: String = match self.inode {
            Some((device, number)) => format!("{device} {number}"),
            None => "- -".to_string(),
        };
        return format!(
            "{inode} {} {} {} {}\n",
            self.facts.files,
            self.facts.bytes,
            written_ns(self.facts.newest),
            written_ns(self.written)
        );
    }

    /// What:
    /// The judgement `text` records,
    /// or `None` when it is not a record this program wrote.
    /// Why:
    /// A record a person or another program wrote says nothing about the tree,
    /// and a half-written one must not be read as a whole one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function parseJudged(text: string): JudgedTree | undefined
    /// ```
    pub fn parse(text: &str) -> Option<JudgedTree> {
        let mut parts = text.split_ascii_whitespace();
        let device: &str = parts.next()?;
        let number: &str = parts.next()?;
        let inode: Option<(u64, u64)> = if device == "-" && number == "-" {
            None
        } else {
            Some((device.parse().ok()?, number.parse().ok()?))
        };
        let files: u64 = parts.next()?.parse().ok()?;
        let bytes: u64 = parts.next()?.parse().ok()?;
        let newest_ns: u64 = parts.next()?.parse().ok()?;
        let written_ns: u64 = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        return Some(JudgedTree {
            inode,
            written: UNIX_EPOCH + std::time::Duration::from_nanos(written_ns),
            facts: TreeFacts {
                bytes,
                files,
                newest: UNIX_EPOCH + std::time::Duration::from_nanos(newest_ns),
                complete: true,
            },
        });
    }

    /// What:
    /// Whether `standing` is the tree this judgement was made of,
    /// as a whole tree.
    /// Why:
    /// The identity alone says which entry it is;
    /// the file count, the bytes and the newest write are what change when anything inside it is written,
    /// and a walk that did not reach the end proves nothing either way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function matches(judged: JudgedTree, standing: JudgedTree): boolean
    /// ```
    pub fn matches(&self, standing: &JudgedTree) -> bool {
        if self.inode != standing.inode || self.written != standing.written {
            return false;
        }
        if !self.facts.complete || !standing.facts.complete {
            return false;
        }
        return self.facts.files == standing.facts.files
            && self.facts.bytes == standing.facts.bytes
            && self.facts.newest == standing.facts.newest;
    }
}

/// What:
/// What a pass judges of the tree at `path` right now,
/// or `None` when it cannot be read whole or its root's write time cannot be read.
/// Why:
/// A removal records what it holds before anything may delete inside it,
/// and a control that builds such a state by hand writes the same record through this function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function judgedOfTree(path: string, deadline: Instant): JudgedTree | undefined
/// ```
pub fn judged_of_tree(path: &Path, deadline: Instant) -> Option<JudgedTree> {
    let metadata: fs::Metadata = fs::symlink_metadata(path).ok()?;
    let written: SystemTime = metadata.modified().ok()?;
    let facts: TreeFacts = tree_facts(path, deadline);
    if !facts.complete {
        return None;
    }
    return Some(JudgedTree { inode: inode_of(&metadata), written, facts });
}

/// What:
/// The judgement recorded for the version whose own directory is `version_directory`,
/// or `None` when no record stands beside it or it cannot be read.
/// Why:
/// A record that is missing, unreadable or written by something else proves nothing about the tree.
/// The path named here is the version's own name, never its removal name:
/// the record sits beside the removal name, and a path that is already a removal name would name a second one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readJudged(versionDirectory: string): JudgedTree | undefined
/// ```
pub fn read_judged(version_directory: &Path) -> Option<JudgedTree> {
    let text: String = fs::read_to_string(record_of(version_directory)).ok()?;
    return JudgedTree::parse(&text);
}

/// What:
/// Records `judged` as what the removal of the version at `version_directory` holds,
/// writing it beside that version's removal name.
/// Why:
/// A later pass that finds the tree under a removal name needs this record to put it back as the finished copy it
/// was instead of as a copy nothing may take for finished;
/// a record that cannot be written leaves that later pass with the safe answer, so the failure is reported and not
/// fatal.
/// The path named here is the version's own name, never its removal name, for the reason `read_judged` gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function writeJudged(versionDirectory: string, judged: JudgedTree): void  // throws
/// ```
pub fn write_judged(version_directory: &Path, judged: &JudgedTree) -> std::io::Result<()> {
    return fs::write(record_of(version_directory), judged.text());
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
    /// Whether the version's marker is a deletion name,
    /// whose tree a pass has begun to delete and which is finished, never restored.
    pub version_deleting: bool,
    /// The name the version directory takes back when the removal is undone.
    pub version_directory: PathBuf,
    /// The marker holding the downloads of that version,
    /// when the removal moved some aside.
    pub downloads_marker: Option<PathBuf>,
    /// Whether the downloads marker is a deletion name.
    pub downloads_deleting: bool,
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
fn markers_in(directory: &Path) -> Vec<(String, PathBuf, bool)> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf, bool)> = Vec::new();
    for entry in entries.flatten() {
        let is_plain_directory: bool = entry.file_type().map(|kind| return kind.is_dir()).unwrap_or(false);
        if !is_plain_directory {
            continue;
        }
        let name: String = entry.file_name().to_string_lossy().to_string();
        if let Some(version) = name.strip_prefix(TOMBSTONE_PREFIX) {
            found.push((version.to_string(), entry.path(), false));
            continue;
        }
        if let Some(version) = name.strip_prefix(DELETING_PREFIX) {
            found.push((version.to_string(), entry.path(), true));
        }
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
        for (version, version_marker, version_deleting) in markers_in(&toolchain_root.join(&tool)) {
            let downloads_marker: PathBuf = downloads_root.join(&tool).join(format!("{TOMBSTONE_PREFIX}{version}"));
            let markers_tool: String = tool.clone();
            let has_downloads: bool = fs::symlink_metadata(&downloads_marker).is_ok();
            found.insert(
                (tool.clone(), version.clone()),
                RemovalMarker {
                    root: toolchain_root.to_path_buf(),
                    version_directory: toolchain_root.join(&tool).join(&version),
                    version_marker,
                    version_deleting,
                    downloads_marker: if has_downloads { Some(downloads_marker) } else { None },
                    downloads_deleting: false,
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
        for (version, downloads_marker, downloads_deleting) in markers_in(&downloads_root.join(&tool)) {
            let entry: &mut RemovalMarker = found.entry((tool.clone(), version.clone())).or_insert_with(|| {
                return RemovalMarker {
                    root: toolchain_root.to_path_buf(),
                    version_directory: toolchain_root.join(&tool).join(&version),
                    version_marker: toolchain_root.join(&tool).join(format!("{TOMBSTONE_PREFIX}{version}")),
                    version_deleting: false,
                    downloads_marker: None,
                    downloads_deleting: false,
                    downloads_directory: Some(downloads_root.join(&tool).join(&version)),
                    tool: tool.clone(),
                    version: version.clone(),
                };
            });
            entry.downloads_marker = Some(downloads_marker);
            entry.downloads_deleting = downloads_deleting;
            entry.downloads_directory = Some(downloads_root.join(&tool).join(&version));
        }
    }
    return found.into_values().collect();
}
