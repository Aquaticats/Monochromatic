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
/// Imports `Duration`,
/// `Instant`,
/// `SystemTime` and `UNIX_EPOCH` from `std::time`.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Imports `Value` from `serde_json`,
/// which reads the size records.
use serde_json::Value;

/// Imports what this module uses from `super::cargo_installs` (the `cargo` namespace and the `current` link).
use super::cargo_installs::{CARGO_NAMESPACE, CURRENT_LINK_NAME};
/// Imports `install_dir_name` from `super::layout`.
use super::layout::install_dir_name;
/// Imports the name checks from `super::segment`,
/// which every name that becomes a path asks before it is joined.
use super::segment::{RESERVED_TOOL_DIRECTORIES, directory_name_problem};
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
/// Measuring a store costs 9.3 s on this machine (103 version directories,
/// 54.5 GiB,
/// read on 2026-11-13),
/// and a pass that reads the records instead takes 1 ms,
/// and a version directory's own write time changes exactly when its content changes (`install` renames a finished
/// tree into place,
/// so the whole version arrives as one rename),
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
/// The name is data from a file when a lock gives it, so it is one plain directory name before it is joined anywhere,
/// or the answer is `None`:
/// the review of this change showed a `cargo-binstall` row's `packageId` of `../../outside` becoming
/// `cargo/../../outside`, which made the cleanup scan and delete a directory outside the store.
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
            if directory_name_problem(crate_name).is_some() {
                return None;
            }
            Some(format!("{CARGO_NAMESPACE}/{crate_name}"))
        }
        // An archive, a Go module, the Android SDK and a crate installed from git each own a top-level directory
        // named after the tool's id, with one directory per version below it.
        Route::Archive | Route::GoInstall | Route::AndroidSdk | Route::CargoGit => {
            let directory: String = install_dir_name(id);
            if directory_name_problem(&directory).is_some() || RESERVED_TOOL_DIRECTORIES.contains(&directory.as_str()) {
                return None;
            }
            Some(directory)
        }
        // pnpm, uv, rustup and the `bin` downloader keep their payload in their own home, not in a version directory.
        Route::MpmPnpm | Route::MpmUvx | Route::RustupToolchain | Route::RustupComponents | Route::BinSourced => None,
    };
}

/// What:
/// One act of the cleanup on the store,
/// carrying the path that was just proved to lie under the store's root with no link on the way.
/// Why:
/// Every act that removes or overwrites something on the store goes through [`proved_under`]:
/// a name is checked at its reader, but a reader's check is not the proof at the act,
/// and a link at a tool's folder once sent a whole recovery outside the store
/// and a link at the recovery marker emptied a file a person owned.
/// Only this value reaches `remove_dir_all`, `remove_file`, `rename` and `write` in the cleanup,
/// so an act that did not come through the proof cannot be written without one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoreAct = { path: string };
/// export function provedUnder(root: string, path: string, forWhat: string): StoreAct  // throws
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreAct {
    /// The path the act uses; proved at the instant it was made.
    path: PathBuf,
}

/// The methods of `StoreAct`.
impl StoreAct {
    /// The path the act uses.
    pub fn path(&self) -> &Path {
        return &self.path;
    }
}

/// What:
/// Proves that `path` lies under the store `root` and that no component of it below the root is a link,
/// and returns the act that may now use it,
/// or refuses with the component and the link it found.
/// Why:
/// The path is read one component at a time from the root, with `symlink_metadata`, which never follows the last
/// component of what it reads,
/// so a link anywhere below the root is seen and refused before anything is removed or overwritten,
/// and it is never followed and never replaced.
/// A component that is not there ends the walk: nothing beyond it exists, and the caller's own not-found handling says
/// what that means.
/// A store root that does not resolve, or a path outside it, is refused.
/// What it holds at the act: the path, at the instant of the walk, and the root's own numbers, re-read by every act's
/// caller where it matters. A link that appears between the walk and the act is the window this leaves; closing it
/// needs the act itself to be made relative to a held descriptor of the root with `O_NOFOLLOW`, which this package's
/// dependency set does not carry yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function provedUnder(root: string, path: string, forWhat: string): StoreAct  // throws
/// ```
pub fn proved_under(root: &Path, path: &Path, for_what: &str) -> Result<StoreAct, String> {
    return proved_with(root, path, for_what, false);
}

/// What:
/// Proves that `path` lies under the store `root` and that no link stands on the way to it,
/// where the path's own last entry may be a link,
/// and returns the act that may now use it.
/// Why:
/// Removing a link removes the link and never what it points at, and a rename replaces the entry it lands on:
/// both are acts on the store that a link at the last name does not send outside it.
/// A link at any name on the way still does, and is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function provedUnderItself(root: string, path: string, forWhat: string): StoreAct  // throws
/// ```
pub fn proved_under_itself(root: &Path, path: &Path, for_what: &str) -> Result<StoreAct, String> {
    return proved_with(root, path, for_what, true);
}

/// The walk both proofs share: `allow_last_link` decides whether the path's own last entry may be a link.
fn proved_with(root: &Path, path: &Path, for_what: &str, allow_last_link: bool) -> Result<StoreAct, String> {
    let resolved_root: PathBuf = match fs::canonicalize(root) {
        Ok(found) => found,
        Err(error) => return Err(format!("{for_what}: the store root {} cannot be resolved: {error}", root.display())),
    };
    // The path is spelled from the root it was built from; the root may itself be reached through a link ($HOME on this
    // machine), so both spellings are tried before the path is refused as outside.
    let relative: PathBuf = match path.strip_prefix(root) {
        Ok(rest) => rest.to_path_buf(),
        Err(_) => match path.strip_prefix(&resolved_root) {
            Ok(rest) => rest.to_path_buf(),
            Err(_) => {
                return Err(format!(
                    "{for_what}: {} does not lie under the store {}",
                    path.display(),
                    root.display()
                ));
            }
        },
    };
    let mut current: PathBuf = if path.starts_with(root) { root.to_path_buf() } else { resolved_root.clone() };
    let last: usize = relative.components().count().saturating_sub(1);
    for (index, component) in relative.components().enumerate() {
        current = current.join(component);
        if allow_last_link && index == last {
            break;
        }
        match fs::symlink_metadata(&current) {
            // What: a link below the root is refused, and the line names it.
            // Why: the act would otherwise leave the store, or empty a file the link points at.
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "{for_what}: {} is a link, and an act on the store never follows one and never replaces one",
                    current.display()
                ));
            }
            Ok(_) => {}
            // Nothing there: nothing beyond it exists either, and the act's own not-found answer says what that means.
            Err(_) => break,
        }
    }
    return Ok(StoreAct { path: path.to_path_buf() });
}

/// What:
/// Removes the directory tree at `path`, after proving that it lies under `root` with no link on the way.
/// Why:
/// This, [`remove_file_proved`] and [`rename_proved`] are the only places the cleanup's removals and moves are made,
/// and each one asks [`proved_under`] first;
/// a caller that skips them is a caller that can leave the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeTreeProved(root: string, path: string, forWhat: string): void  // throws
/// ```
pub fn remove_tree_proved(root: &Path, path: &Path, for_what: &str) -> Result<(), StoreActFailure> {
    let act: StoreAct = proved_under(root, path, for_what).map_err(StoreActFailure::Refused)?;
    return fs::remove_dir_all(act.path()).map_err(StoreActFailure::Failed);
}

/// What:
/// Removes the file at `path`, after proving that it lies under `root` with no link on the way.
/// Why:
/// See [`remove_tree_proved`]: one door for every act on the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeFileProved(root: string, path: string, forWhat: string): void  // throws
/// ```
pub fn remove_file_proved(root: &Path, path: &Path, for_what: &str) -> Result<(), StoreActFailure> {
    let act: StoreAct = proved_under_itself(root, path, for_what).map_err(StoreActFailure::Refused)?;
    return fs::remove_file(act.path()).map_err(StoreActFailure::Failed);
}

/// What:
/// Moves `from` to `to`, after proving that both lie under `root` with no link on the way.
/// Why:
/// See [`remove_tree_proved`]: one door for every act on the store, and a rename lands on both of its ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function renameProved(root: string, from: string, to: string, forWhat: string): void  // throws
/// ```
pub fn rename_proved(root: &Path, from: &Path, to: &Path, for_what: &str) -> Result<(), StoreActFailure> {
    let source: StoreAct = proved_under_itself(root, from, for_what).map_err(StoreActFailure::Refused)?;
    let target: StoreAct = proved_under_itself(root, to, for_what).map_err(StoreActFailure::Refused)?;
    return fs::rename(source.path(), target.path()).map_err(StoreActFailure::Failed);
}

/// What:
/// Why an act on the store did not happen: the proof that it lands inside the store refused it,
/// or the filesystem refused the act itself.
/// Why:
/// A caller prints its own sentence about the act,
/// and a refusal has words of its own that name the link or the place outside the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoreActFailure = { kind: "refused"; reason: string } | { kind: "failed"; error: Error };
/// ```
#[derive(Debug)]
pub enum StoreActFailure {
    /// The path was not proved to lie inside the store, or a link stood on the way to it.
    Refused(String),
    /// The act itself failed, and the error is the filesystem's.
    Failed(std::io::Error),
}

/// The methods of `StoreActFailure`.
impl std::fmt::Display for StoreActFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            StoreActFailure::Refused(reason) => formatter.write_str(reason),
            StoreActFailure::Failed(error) => write!(formatter, "{error}"),
        };
    }
}

/// What:
/// Whether what stands at `path` lies under `root` once both are resolved,
/// or whether nothing stands there at all.
/// Why:
/// Names come from files and are joined to the store root, and a lock's row, a plan's row or a link can name a path
/// that leaves the store.
/// Before anything is moved or deleted, the path that will be acted on is resolved and shown to lie under the store:
/// a name is data until that proof, whatever the checks at its reader allowed.
/// Nothing at the path is not a reason to refuse, because nothing there is nothing to lose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolvesUnder(root: string, path: string): boolean
/// ```
pub fn resolves_under(root: &Path, path: &Path) -> bool {
    let resolved_path: PathBuf = match fs::canonicalize(path) {
        Ok(found) => found,
        Err(_) => return true,
    };
    let resolved_root: PathBuf = match fs::canonicalize(root) {
        Ok(found) => found,
        Err(_) => return false,
    };
    return resolved_path.starts_with(&resolved_root);
}

/// What:
/// The two numbers that name one entry on its file system:
/// the device and the inode,
/// or zeros where the platform has none.
/// Why:
/// A write time tells two writes apart only as finely as the file system's clock,
/// so two files written in the same tick look alike;
/// the inode is what proves the entry standing at a name is the one that was judged,
/// which is what a removal needs before it deletes or moves anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryNaming = { device: number; inode: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EntryNaming {
    /// The file system the entry is on.
    pub device: u64,
    /// The entry's number on that file system.
    pub inode: u64,
}

/// The methods of `EntryNaming`.
impl EntryNaming {
    /// The numbers that name the entry `metadata` describes.
    #[cfg(unix)]
    pub fn of(metadata: &fs::Metadata) -> EntryNaming {
        /// Imports `MetadataExt` from `std::os::unix::fs`,
        /// for the two numbers that name an entry.
        use std::os::unix::fs::MetadataExt;
        return EntryNaming { device: metadata.dev(), inode: metadata.ino() };
    }

    /// The numbers that name the entry `metadata` describes;
    /// no platform but Unix has them.
    #[cfg(not(unix))]
    pub fn of(_metadata: &fs::Metadata) -> EntryNaming {
        return EntryNaming::default();
    }
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
    /// The size the version and its downloads weigh,
    /// in bytes.
    pub bytes: u64,
    /// When the version directory was last written.
    pub written: SystemTime,
    /// The numbers that name the version directory,
    /// as the scan read them.
    pub naming: EntryNaming,
}

/// A download an earlier layout left directly under the store's `downloads` directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeftoverDownload {
    /// The tool the file's name belongs to.
    pub tool: String,
    /// The file.
    pub file: PathBuf,
    /// The size it weighs,
    /// in bytes.
    pub bytes: u64,
    /// When it was last written.
    pub written: SystemTime,
    /// The numbers that name the file,
    /// as the scan read them.
    pub naming: EntryNaming,
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
/// the size is what a bound is compared with,
/// never a reason to remove something.
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

/// What:
/// The longest a downloaded file may go untouched and still belong to an install that is using it:
/// one hour.
/// Why:
/// An install downloads an archive and then unpacks it,
/// and until the unpack ends another command must not take the file away.
/// The working file shows that a download is running,
/// but a finished download waiting to be unpacked looks like a leftover,
/// so the age is what tells the two apart:
/// the client gives up on a download whose bytes stop for `http::DOWNLOAD_IDLE_TIMEOUT_SECONDS` (60 s),
/// a whole download is bounded by `http::DOWNLOAD_TOTAL_TIMEOUT_SECONDS` (30 min),
/// and unpacking reads the file for seconds to a minute,
/// so no install needs a file it has not touched for an hour.
/// An install whose process was stopped for longer than this can lose its download to it,
/// and it reports that download as failed and installs again.
/// The check and the removal are two steps,
/// so an install that writes its first byte in the instant between them can still meet a removal;
/// closing that needs the writer to declare itself before it writes,
/// which is a lock this package does not take.
pub const DOWNLOAD_IN_USE_FLOOR: Duration = Duration::from_secs(60 * 60);

/// What:
/// Why an install is using this downloads directory now,
/// or `None` when nothing in it belongs to one.
/// Why:
/// Two removals ask this before they take a version's archives away,
/// the newest-only removal and the size- and write-time cleanup,
/// and both must answer it the same way:
/// a directory whose files are younger than `DOWNLOAD_IN_USE_FLOOR` is one an install is writing or unpacking,
/// and the version it belongs to is in use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function downloadsInUse(downloads: string): string | undefined
/// ```
pub fn downloads_in_use(downloads: &Path) -> Option<String> {
    let Ok(entries) = fs::read_dir(downloads) else {
        return None;
    };
    let now: SystemTime = SystemTime::now();
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(written) = metadata.modified() else {
            continue;
        };
        let age: Duration = now.duration_since(written).unwrap_or(Duration::ZERO);
        if age < DOWNLOAD_IN_USE_FLOOR {
            return Some(format!("{} was written {} s ago", entry.path().display(), age.as_secs()));
        }
    }
    return None;
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
/// The inventory stops when the deadline passes,
/// between one tool and the next and between one version and the next,
/// and says why it is not the whole store.
/// Two calls inside it cannot be cut in the middle and are the ones outside the limit:
/// one version's size walk (`tree_size`) and one directory read,
/// because a size half counted is a number a bound would be compared with wrongly;
/// neither leaves anything behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function versionDirectories(root: string, tools: ReadonlySet<string>, deadline: Date): [VersionDirectory[], string?]
/// ```
pub fn version_directories(
    toolchain_root: &Path,
    tools: &BTreeSet<String>,
    deadline: Instant,
) -> (Vec<VersionDirectory>, Option<String>) {
    let recorded: BTreeMap<String, SizeRecord> = read_size_records(toolchain_root);
    let mut measured: BTreeMap<String, SizeRecord> = BTreeMap::new();
    let mut found: Vec<VersionDirectory> = Vec::new();
    let mut stopped: Option<String> = None;
    for (index, tool) in tools.iter().enumerate() {
        // What: the inventory is cut between one tool and the next, never inside one version's measurement.
        // Why: the pass may spend only so long on a store, and a version whose size was half counted would be a
        // number the size bound is compared with wrongly.
        if Instant::now() >= deadline {
            stopped = Some(format!(
                "the inventory ran out of its time limit after {} of {} tools, so this pass judges only the versions it \
                 reached",
                index,
                tools.len()
            ));
            break;
        }
        let tool_directory: PathBuf = toolchain_root.join(tool);
        let Ok(entries) = fs::read_dir(&tool_directory) else {
            continue;
        };
        for entry in entries.flatten() {
            if Instant::now() >= deadline {
                stopped = Some(format!(
                    "the inventory ran out of its time limit inside {}, so this pass judges only the versions it reached",
                    tool
                ));
                break;
            }
            let version: String = entry.file_name().to_string_lossy().to_string();
            if !is_version_entry(&entry, &version) {
                continue;
            }
            let directory: PathBuf = entry.path();
            // What: a version whose path resolves outside the store is not listed, and the scan says it left one out.
            // Why: the names come from files and from directory entries, and a link at a tool's name leaves the store;
            // the scan is the first place that can prove it after the join.
            if !resolves_under(toolchain_root, &directory) {
                stopped = Some(format!(
                    "{} resolves outside {}, so no version of it is judged",
                    directory.display(),
                    toolchain_root.display()
                ));
                break;
            }
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
            found.push(VersionDirectory { tool: tool.clone(), version, directory, downloads, bytes, written, naming: EntryNaming::of(&metadata) });
        }
        if stopped.is_some() {
            break;
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
    return (found, stopped);
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
            found.push(LeftoverDownload {
                tool: tool.clone(),
                file: file.clone(),
                bytes: metadata.len(),
                written,
                naming: EntryNaming::of(&metadata),
            });
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
