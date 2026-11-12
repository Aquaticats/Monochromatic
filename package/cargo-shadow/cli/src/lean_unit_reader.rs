//! What:
//! The reader of a Cargo profile directory's units:
//! for each unit the compiler that built it (the `rustc` field of its fingerprint),
//! the time it was last invoked,
//! and every path that belongs to it,
//! in both layouts Cargo writes today.
//! Why:
//! The sweep and the size caps decide from these facts.
//! The owner allowed this one reader to parse Cargo's internal fingerprint files
//! (`doc/decision/cargo-build-dir-host-cache.md`,
//! amendment of 2026-11-07T22:19:43-05:00),
//! on the condition that a directory it does not recognise is left untouched and reported.
//! Nothing here may wait without end:
//! the walk obeys the run's deadline like every other walk,
//! and a read the deadline cut says so instead of answering a list it did not finish.
//! Old layout:
//! `.fingerprint/<package>-<hash>/`,
//! `deps/<name>-<hash>.*`,
//! `build/<package>-<hash>/`.
//! New layout:
//! `build/<package>/<hash>/` with its `fingerprint/` inside.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function readProfile(profile: string): { known: ProfileUnits } | { unknown: string }
//! ```

/// Named reads of paths and text.
use crate::lean_text::{all_hex, extension_is, read_small_bytes};
/// The instant the read must be over by.
use crate::lean_identity::Deadline;
/// Directory reading and metadata.
use std::{collections::HashMap, fs, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// How many bytes of a fingerprint file hold the `rustc` field (it is the first field Cargo writes).
pub const FINGERPRINT_HEAD_BYTES: usize = 512;

/// The length of the hexadecimal hash Cargo puts after a unit's name.
const HASH_LENGTH: usize = 16;

/// The file whose modification time is the time of the unit's last invocation.
pub const INVOKED_FILE: &str = "invoked.timestamp";

/// One unit of Cargo's work in a profile directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Unit {
    /// A name for reports:
    /// `<package>-<hash>`.
    pub label: String,
    /// The compiler the fingerprint records.
    pub compiler: u64,
    /// The last invocation in seconds from the epoch.
    pub invoked: i64,
    /// Every path of the unit,
    /// removed together or not at all.
    pub paths: Vec<PathBuf>,
}

/// An incremental-compilation directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncrementalDirectory {
    /// Where it is.
    pub path: PathBuf,
    /// Its last modification in seconds from the epoch.
    pub modified: i64,
}

/// What a profile directory holds.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProfileUnits {
    /// The units with a readable compiler.
    pub units: Vec<Unit>,
    /// Fingerprint directories whose compiler could not be read,
    /// reported and never removed.
    pub unreadable: Vec<String>,
    /// The incremental directories.
    pub incremental: Vec<IncrementalDirectory>,
}

/// What a read found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfileRead {
    /// A directory in a layout this reader knows.
    Known(
        /// The units of the directory.
        ProfileUnits,
    ),
    /// Not a layout this reader knows:
    /// left alone.
    Unknown(
        /// Why the layout is not known.
        String,
    ),
    /// The deadline passed before the walk finished,
    /// so no list of it is a list of what the directory holds and nothing in it may be judged.
    Cut,
}

/// What:
/// `pub fn compiler_of(head: &str) -> Option<u64>` reads the `rustc` number at the start of a fingerprint's JSON.
/// Why:
/// Cargo writes `{"rustc":<number>,...` first,
/// so the number is read without a JSON parser and without the rest of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const compilerOf = (head: string): bigint | undefined => /^\{"rustc":(\d+)/.exec(head)?.[1] ? BigInt(m[1]) : undefined;
/// ```
pub fn compiler_of(head: &str) -> Option<u64> {
    let rest: &str = head.trim_start().strip_prefix("{\"rustc\":")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    return digits.parse::<u64>().ok();
}

/// What:
/// `fn read_head(path: &Path) -> Option<String>` reads the first bytes of a file as text.
/// Why:
/// Only the `rustc` field is needed.
/// The name is one in a build tree,
/// which is a directory other commands write:
/// a pipe at it holds a plain read until a writer arrives,
/// and a device never reaches its end,
/// so the open never waits and the entry it opened must be a regular file.
/// A name that cannot be read that way is no compiler here,
/// and the unit it belongs to is reported as unreadable and never removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readHead = (path: string): string | undefined => { try { return readRegularSync(path).subarray(0, 512).toString(); } catch { return undefined; } };
/// ```
fn read_head(path: &Path) -> Option<String> {
    let head: Vec<u8> = read_small_bytes(path, FINGERPRINT_HEAD_BYTES).ok()?;
    return Some(String::from_utf8_lossy(&head).to_string());
}

/// What:
/// `fn compiler_in(fingerprint_directory: &Path) -> Option<(u64, i64)>` is the compiler and the last invocation time of one
/// fingerprint directory.
/// Why:
/// The compiler comes from the first `*.json` file (sorted by name),
/// and the time from `invoked.timestamp` (the newest JSON when that file is absent).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compilerIn(dir: string): { compiler: bigint; invoked: number } | undefined
/// ```
fn compiler_in(fingerprint_directory: &Path) -> Option<(u64, i64)> {
    let mut json_files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(fingerprint_directory).ok()?.flatten() {
        if extension_is(&entry.path(), "json") {
            json_files.push(entry.path());
        }
    }
    json_files.sort();
    let first: &PathBuf = json_files.first()?;
    let compiler: u64 = compiler_of(&read_head(first)?)?;
    let invoked: i64 = match fs::symlink_metadata(fingerprint_directory.join(INVOKED_FILE)) {
        Ok(metadata) => metadata.mtime(),
        Err(_) => fs::symlink_metadata(first).ok()?.mtime(),
    };
    return Some((compiler, invoked));
}

/// What:
/// `fn unit_hash_of_file_name(name: &str) -> Option<&str>` is the 16-digit hash a `deps/` file name ends its stem with.
/// Why:
/// `libfoo-0123456789abcdef.rlib`,
/// `foo-0123456789abcdef.d` and the extensionless executable `foo-0123456789abcdef` all belong to the unit with that hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const unitHashOfFileName = (name: string): string | undefined => /-([0-9a-f]{16})$/.exec(name.split('.')[0])?.[1];
/// ```
pub fn unit_hash_of_file_name(name: &str) -> Option<&str> {
    let stem: &str = name.split('.').next()?;
    if stem.len() <= HASH_LENGTH + 1 {
        return None;
    }
    let (front, hash) = stem.split_at(stem.len() - HASH_LENGTH);
    if !front.ends_with('-') || !all_hex(hash) {
        return None;
    }
    return Some(hash);
}

/// What:
/// `fn deps_by_hash(deps: &Path, deadline: &Deadline) -> Option<HashMap<String, Vec<PathBuf>>>` groups the files of a `deps/` directory by unit hash,
/// and answers nothing when the deadline passed before the listing finished.
/// Why:
/// One read of the directory serves every unit of the old layout,
/// and a `deps/` directory holds a file per unit,
/// so this is one of the walks a large profile directory spends its time in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function depsByHash(deps: string, deadline: Deadline): Map<string, string[]> | undefined
/// ```
fn deps_by_hash(deps: &Path, deadline: &Deadline) -> Option<HashMap<String, Vec<PathBuf>>> {
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let Ok(entries) = fs::read_dir(deps) else { return Some(groups) };
    for entry in entries.flatten() {
        if deadline.passed() {
            return None;
        }
        let name: String = entry.file_name().to_string_lossy().to_string();
        if let Some(hash) = unit_hash_of_file_name(&name) {
            groups.entry(hash.to_string()).or_default().push(entry.path());
        }
    }
    return Some(groups);
}

/// What:
/// `fn read_old_layout(profile: &Path, found: &mut ProfileUnits, deadline: &Deadline) -> bool` reads `.fingerprint/<package>-<hash>/`,
/// and answers whether it finished before the deadline.
/// Why:
/// Each fingerprint directory is a unit whose artifacts are the `deps/` files and the `build/<package>-<hash>` directory with
/// the same hash.
/// One unit costs a handful of system calls and a profile directory holds thousands of them,
/// so the loop that reads them is where the deadline has to reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readOldLayout(profile: string, found: ProfileUnits, deadline: Deadline): boolean
/// ```
fn read_old_layout(profile: &Path, found: &mut ProfileUnits, deadline: &Deadline) -> bool {
    let Some(deps) = deps_by_hash(&profile.join("deps"), deadline) else { return false };
    let Ok(entries) = fs::read_dir(profile.join(".fingerprint")) else { return true };
    for entry in entries.flatten() {
        if deadline.passed() {
            return false;
        }
        let label: String = entry.file_name().to_string_lossy().to_string();
        let Some((_, hash)) = label.rsplit_once('-') else { continue };
        if hash.len() != HASH_LENGTH {
            found.unreadable.push(label);
            continue;
        }
        let Some((compiler, invoked)) = compiler_in(&entry.path()) else {
            found.unreadable.push(label);
            continue;
        };
        let mut paths: Vec<PathBuf> = vec![entry.path()];
        let build: PathBuf = profile.join("build").join(&label);
        if build.is_dir() {
            paths.push(build);
        }
        if let Some(artifacts) = deps.get(hash) {
            paths.extend(artifacts.iter().cloned());
        }
        found.units.push(Unit { label, compiler, invoked, paths });
    }
    return true;
}

/// What:
/// `fn read_new_layout(profile: &Path, found: &mut ProfileUnits, deadline: &Deadline) -> bool` reads `build/<package>/<hash>/fingerprint/`,
/// and answers whether it finished before the deadline.
/// Why:
/// In the new layout one directory is the whole unit,
/// so the walk of two levels is the loop the deadline has to reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readNewLayout(profile: string, found: ProfileUnits, deadline: Deadline): boolean
/// ```
fn read_new_layout(profile: &Path, found: &mut ProfileUnits, deadline: &Deadline) -> bool {
    let Ok(packages) = fs::read_dir(profile.join("build")) else { return true };
    for package in packages.flatten() {
        if deadline.passed() {
            return false;
        }
        // An old-layout unit (`build/<package>-<hash>`) has a fingerprint directory of the same name: it is read there.
        if profile.join(".fingerprint").join(package.file_name()).exists() {
            continue;
        }
        let Ok(hashes) = fs::read_dir(package.path()) else { continue };
        for unit in hashes.flatten() {
            if deadline.passed() {
                return false;
            }
            let fingerprint: PathBuf = unit.path().join("fingerprint");
            if !fingerprint.is_dir() {
                continue;
            }
            let label: String = format!("{}-{}", package.file_name().to_string_lossy(), unit.file_name().to_string_lossy());
            match compiler_in(&fingerprint) {
                Some((compiler, invoked)) => found.units.push(Unit { label, compiler, invoked, paths: vec![unit.path()] }),
                None => found.unreadable.push(label),
            }
        }
    }
    return true;
}

/// What:
/// `pub fn read_profile(profile: &Path, deadline: &Deadline) -> ProfileRead` reads one profile directory.
/// Why:
/// A directory with neither layout's marker is `Unknown`,
/// and nothing in it is touched.
/// A walk the deadline stopped is `Cut`:
/// a list it did not finish is not the list of what the directory holds,
/// and the sweep must not judge a removal from part of a unit's paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readProfile(profile: string, deadline: Deadline): ProfileRead
/// ```
pub fn read_profile(profile: &Path, deadline: &Deadline) -> ProfileRead {
    let old: bool = profile.join(".fingerprint").is_dir();
    let new: bool = profile.join("build").is_dir();
    if !old && !new {
        return ProfileRead::Unknown(String::from("neither `.fingerprint` nor `build` exists, so the layout is not one this reader knows"));
    }
    if deadline.passed() {
        return ProfileRead::Cut;
    }
    let mut found: ProfileUnits = ProfileUnits::default();
    if old && !read_old_layout(profile, &mut found, deadline) {
        return ProfileRead::Cut;
    }
    if new && !read_new_layout(profile, &mut found, deadline) {
        return ProfileRead::Cut;
    }
    if let Ok(entries) = fs::read_dir(profile.join("incremental")) {
        for entry in entries.flatten() {
            if deadline.passed() {
                return ProfileRead::Cut;
            }
            if let Ok(metadata) = entry.metadata() {
                found.incremental.push(IncrementalDirectory { path: entry.path(), modified: metadata.mtime() });
            }
        }
    }
    return ProfileRead::Known(found);
}
