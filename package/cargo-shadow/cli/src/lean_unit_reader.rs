//! What:
//! The reader of a Cargo profile directory's units:
//! for each unit the compiler that built it (the `rustc` field of its fingerprint),
//! the time it was last invoked,
//! and every path that belongs to it,
//! in both layouts Cargo writes today.
//! Why:
//! The sweep and the size caps decide from these facts.
//! The owner allowed this one reader to parse Cargo's internal fingerprint files
//! (`doc/decision/cargo-build-dir-host-cache.md`, amendment of 2026-11-07T22:19:43-05:00),
//! on the condition that a directory it does not recognise is left untouched and reported.
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

/// Directory reading and metadata.
use std::{collections::HashMap, fs, io::Read, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// How many bytes of a fingerprint file hold the `rustc` field (it is the first field Cargo writes).
pub const FINGERPRINT_HEAD_BYTES: usize = 512;

/// The length of the hexadecimal hash Cargo puts after a unit's name.
const HASH_LENGTH: usize = 16;

/// The file whose modification time is the time of the unit's last invocation.
pub const INVOKED_FILE: &str = "invoked.timestamp";

/// One unit of Cargo's work in a profile directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Unit {
    /// A name for reports: `<package>-<hash>`.
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
    Known(ProfileUnits),
    /// Not a layout this reader knows:
    /// left alone.
    Unknown(String),
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readHead = (path: string): string | undefined => readFileSync(path).subarray(0, 512).toString();
/// ```
fn read_head(path: &Path) -> Option<String> {
    let mut file: fs::File = fs::File::open(path).ok()?;
    let mut buffer: Vec<u8> = vec![0; FINGERPRINT_HEAD_BYTES];
    let count: usize = file.read(&mut buffer).ok()?;
    buffer.truncate(count);
    return Some(String::from_utf8_lossy(&buffer).to_string());
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
        if entry.path().extension().is_some_and(|extension| { return extension == "json"; }) {
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
    if !front.ends_with('-') || !hash.chars().all(|character| { return character.is_ascii_hexdigit(); }) {
        return None;
    }
    return Some(hash);
}

/// What:
/// `fn deps_by_hash(deps: &Path) -> HashMap<String, Vec<PathBuf>>` groups the files of a `deps/` directory by unit hash.
/// Why:
/// One read of the directory serves every unit of the old layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function depsByHash(deps: string): Map<string, string[]>
/// ```
fn deps_by_hash(deps: &Path) -> HashMap<String, Vec<PathBuf>> {
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let Ok(entries) = fs::read_dir(deps) else { return groups };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if let Some(hash) = unit_hash_of_file_name(&name) {
            groups.entry(hash.to_string()).or_default().push(entry.path());
        }
    }
    return groups;
}

/// What:
/// `fn read_old_layout(profile: &Path, found: &mut ProfileUnits)` reads `.fingerprint/<package>-<hash>/`.
/// Why:
/// Each fingerprint directory is a unit whose artifacts are the `deps/` files and the `build/<package>-<hash>` directory with
/// the same hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readOldLayout(profile: string, found: ProfileUnits): void
/// ```
fn read_old_layout(profile: &Path, found: &mut ProfileUnits) {
    let deps: HashMap<String, Vec<PathBuf>> = deps_by_hash(&profile.join("deps"));
    let Ok(entries) = fs::read_dir(profile.join(".fingerprint")) else { return };
    for entry in entries.flatten() {
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
}

/// What:
/// `fn read_new_layout(profile: &Path, found: &mut ProfileUnits)` reads `build/<package>/<hash>/fingerprint/`.
/// Why:
/// In the new layout one directory is the whole unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readNewLayout(profile: string, found: ProfileUnits): void
/// ```
fn read_new_layout(profile: &Path, found: &mut ProfileUnits) {
    let Ok(packages) = fs::read_dir(profile.join("build")) else { return };
    for package in packages.flatten() {
        // An old-layout unit (`build/<package>-<hash>`) has a fingerprint directory of the same name: it is read there.
        if profile.join(".fingerprint").join(package.file_name()).exists() {
            continue;
        }
        let Ok(hashes) = fs::read_dir(package.path()) else { continue };
        for unit in hashes.flatten() {
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
}

/// What:
/// `pub fn read_profile(profile: &Path) -> ProfileRead` reads one profile directory.
/// Why:
/// A directory with neither layout's marker is `Unknown`,
/// and nothing in it is touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readProfile(profile: string): ProfileRead
/// ```
pub fn read_profile(profile: &Path) -> ProfileRead {
    let old: bool = profile.join(".fingerprint").is_dir();
    let new: bool = profile.join("build").is_dir();
    if !old && !new {
        return ProfileRead::Unknown(String::from("neither `.fingerprint` nor `build` exists, so the layout is not one this reader knows"));
    }
    let mut found: ProfileUnits = ProfileUnits::default();
    if old {
        read_old_layout(profile, &mut found);
    }
    if new {
        read_new_layout(profile, &mut found);
    }
    if let Ok(entries) = fs::read_dir(profile.join("incremental")) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                found.incremental.push(IncrementalDirectory { path: entry.path(), modified: metadata.mtime() });
            }
        }
    }
    return ProfileRead::Known(found);
}
