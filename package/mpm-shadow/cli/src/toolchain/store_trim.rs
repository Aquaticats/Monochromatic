//! What:
//! The cleanup of old toolchain versions:
//! while the stores keep more than a size bound,
//! the versions written longest ago go first,
//! unless a lock names the version,
//! this run installed it,
//! the tool's `current` link names it,
//! or a running process uses it.
//! Why:
//! mpm-shadow installs one directory per version and removed nothing but this repository's own tools,
//! so on 2026-11-13 the two stores held 55.4 GiB,
//! 42 GiB of it versions no lock named
//! (a 3.1 GiB video-card build of llama.cpp, ten times over, and 4.5 GiB of downloads an earlier layout left).
//! The owner's answer that day:
//! "make our mpm-shadow auto cleanup old toolchains by size + last modified time".
//!
//! The pass runs at the end of every sync that installed a version,
//! and at the start of any later sync while a removal an earlier run started is unfinished,
//! so no command has to be remembered.
//! A run removes for at most [`DEFAULT_WORK_LIMIT_MILLIS`],
//! and a removal renames the version out of its name before deleting anything,
//! so the install never waits on a long deletion and a killed removal is finished by the next run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // store-trim.ts
//! export function boundsFromEnv(env: EnvMap): TrimBounds
//! export function readNamedVersions(locks: string[]): NamedVersions
//! export function candidatesOf(roots: string[], tools: ReadonlySet<string>): TrimCandidate[]
//! export function planTrim(candidates: TrimCandidate[], bounds: TrimBounds, now: number, protection: (c: TrimCandidate) => string | undefined): TrimPlan
//! export function applyTrim(plan: TrimPlan, run: TrimRun): TrimOutcome
//! export function trimAfterInstall(ctx: Context, installed: [string, string][]): void
//! ```

/// Imports the sorted map and set the rule works on.
use std::collections::{BTreeMap, BTreeSet};
/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Duration`,
/// `Instant` and `SystemTime` from `std::time`.
use std::time::{Duration, Instant, SystemTime};

/// Imports `Value` from `serde_json`,
/// which reads the pin rows of any lock.
use serde_json::Value;

/// Imports `format_size` from the post-run cleanup's wording,
/// so a size reads the same everywhere.
use crate::cleanup_report::format_size;

/// Imports `CURRENT_LINK_NAME` from `super::cargo_installs`.
use super::cargo_installs::CURRENT_LINK_NAME;
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `Route` from `super::model`.
use super::model::Route;
/// Imports `crate_of_entry` from `super::prune_cargo`,
/// which reads a lock row's crate name the way the prune of unnamed cargo versions does.
use super::prune_cargo::crate_of_entry;
/// Imports `ScopeKind` and `global_scope` from `super::scope`.
use super::scope::{EnvMap, ScopeKind, global_scope};
/// Imports what this module uses from `super::store_path_holders`.
use super::store_path_holders::{StoreHolders, held_reason, holders_of, opaque_reason};
/// Imports what this module uses from `super::store_versions`.
use super::store_versions::{
    VersionDirectory, install_directory_name, leftover_downloads, tombstone_of, tombstones, tree_size,
    version_directories,
};
/// Imports `worktree_lock_files` from `super::worktrees`.
use super::worktrees::worktree_lock_files;

/// Bytes in the next larger unit.
const BYTES_PER_KIB: u64 = 1024;

/// Bytes in a mebibyte.
const BYTES_PER_MIB: u64 = BYTES_PER_KIB * 1024;

/// Bytes in a gibibyte.
const BYTES_PER_GIB: u64 = BYTES_PER_MIB * 1024;

/// Bytes in a tebibyte.
const BYTES_PER_TIB: u64 = BYTES_PER_GIB * 1024;

/// Seconds in a minute.
const SECONDS_PER_MINUTE: u64 = 60;

/// Seconds in an hour.
const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;

/// Seconds in a day.
const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;

/// What:
/// The space both stores may keep before the versions written longest ago are removed:
/// 24 GiB.
/// Why:
/// Sized from the measurement this run made of this machine's stores
/// (`doc/toolchain-version-cleanup.md`, "The measurement it was sized from"):
/// the versions a visible lock names weighed 11.3 GiB across both roots,
/// so about twice that leaves room for the version an install brings and for several updates in flight,
/// and it would have removed 31.4 GiB of the 55.4 GiB the stores held on 2026-11-13.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DEFAULT_SIZE_BOUND_BYTES = 24 * 1024 ** 3;
/// ```
pub const DEFAULT_SIZE_BOUND_BYTES: u64 = 24 * BYTES_PER_GIB;

/// What:
/// How recently a version must have been written to be kept whatever else is true:
/// one hour.
/// Why:
/// A proposal, and the only bound here no measurement settles:
/// an install writes its lock row before the version it installs,
/// so a version a lock names is protected by that lock,
/// and this floor exists only for a copy a person or another tool made by hand,
/// which no lock names yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DEFAULT_AGE_FLOOR_SECONDS = 3600;
/// ```
pub const DEFAULT_AGE_FLOOR_SECONDS: u64 = 60 * 60;

/// What:
/// How long one pass may spend removing versions:
/// two seconds.
/// Why:
/// Measured on this machine's store volume (2026-11-13,
/// `keep/deletion-cost-home-volume.txt` of the run that added this):
/// deleting a version costs about 30 microseconds per file
/// (4743 files in 145 ms, 8820 in 260 ms, and 1180 files of 3.2 GiB in 62 ms),
/// so two seconds removes about 66 000 files,
/// more than the largest version either store holds (19 521 files) and less than a fifth of the 43.2 GiB over the
/// bound that day.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DEFAULT_WORK_LIMIT_MILLIS = 2000;
/// ```
pub const DEFAULT_WORK_LIMIT_MILLIS: u64 = 2_000;

/// The environment variable holding the space the stores may keep,
/// in bytes or with a `KiB`/`MiB`/`GiB`/`TiB` suffix (`32GiB`).
pub const SIZE_BOUND_ENV: &str = "MPM_SHADOW_STORE_SIZE_BOUND";

/// The environment variable holding how recently a version may have been written and still be removed,
/// in seconds or with an `s`/`m`/`h`/`d` suffix (`2h`).
pub const AGE_FLOOR_ENV: &str = "MPM_SHADOW_STORE_AGE_FLOOR";

/// The environment variable holding how long one pass may spend removing versions,
/// in milliseconds or with an `s`/`m` suffix (`5s`).
pub const WORK_LIMIT_ENV: &str = "MPM_SHADOW_STORE_WORK_LIMIT";

/// The file an unfinished removal leaves in a store root,
/// so the next install finishes it even when that install installs nothing.
pub const UNFINISHED_MARKER_NAME: &str = ".mpm-shadow-removal-unfinished";

/// The three bounds one pass obeys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrimBounds {
    /// The space the stores may keep before the oldest versions go.
    pub size_bound_bytes: u64,
    /// How recently a version may have been written and still be removed.
    pub age_floor: Duration,
    /// How long one pass may spend removing versions.
    pub work_limit: Duration,
}

/// The methods of `TrimBounds`.
impl TrimBounds {
    /// The bounds the environment sets,
    /// with the shipped default for each it leaves unset.
    pub fn from_env(env: &EnvMap) -> TrimBounds {
        return TrimBounds {
            size_bound_bytes: env
                .get(SIZE_BOUND_ENV)
                .and_then(parse_size)
                .unwrap_or(DEFAULT_SIZE_BOUND_BYTES),
            age_floor: env
                .get(AGE_FLOOR_ENV)
                .and_then(parse_duration)
                .unwrap_or(Duration::from_secs(DEFAULT_AGE_FLOOR_SECONDS)),
            work_limit: env
                .get(WORK_LIMIT_ENV)
                .and_then(parse_duration)
                .unwrap_or(Duration::from_millis(DEFAULT_WORK_LIMIT_MILLIS)),
        };
    }
}

/// What:
/// A byte count written as a number with an optional `KiB`, `MiB`, `GiB`, `TiB` (or `K`, `M`, `G`, `T`) suffix.
/// Why:
/// The bound is set by a person typing an environment variable,
/// and `32GiB` is what a person reads back out of the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseSize(text: string): number | undefined
/// ```
pub fn parse_size(text: &str) -> Option<u64> {
    let trimmed: &str = text.trim();
    let lowered: String = trimmed.to_lowercase();
    for (suffix, unit) in [
        ("kib", BYTES_PER_KIB),
        ("mib", BYTES_PER_MIB),
        ("gib", BYTES_PER_GIB),
        ("tib", BYTES_PER_TIB),
        ("k", BYTES_PER_KIB),
        ("m", BYTES_PER_MIB),
        ("g", BYTES_PER_GIB),
        ("t", BYTES_PER_TIB),
    ] {
        if let Some(number) = lowered.strip_suffix(suffix) {
            return number.trim().parse::<u64>().ok().map(|value| return value.saturating_mul(unit));
        }
    }
    return lowered.parse::<u64>().ok();
}

/// What:
/// A span written as a number of seconds with an optional `s`, `m`, `h` or `d` suffix,
/// or as a number of milliseconds with an `ms` suffix.
/// Why:
/// The age floor and the work limit are set the same way the size bound is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseDuration(text: string): number | undefined
/// ```
pub fn parse_duration(text: &str) -> Option<Duration> {
    let trimmed: String = text.trim().to_lowercase();
    for (suffix, unit) in [
        ("ms", Duration::from_millis(1)),
        ("s", Duration::from_secs(1)),
        ("m", Duration::from_secs(SECONDS_PER_MINUTE)),
        ("h", Duration::from_secs(SECONDS_PER_HOUR)),
        ("d", Duration::from_secs(SECONDS_PER_DAY)),
    ] {
        if let Some(number) = trimmed.strip_suffix(suffix) {
            return number.trim().parse::<u64>().ok().map(|value| return unit.saturating_mul(value as u32));
        }
    }
    return trimmed.parse::<u64>().ok().map(Duration::from_secs);
}

/// What:
/// A span in the words a person reads (`41 h`, `3 d`, `12 min`).
/// Why:
/// The lines say when a version was last written,
/// and hours or days are what a reader compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function formatAge(now: number, written: number): string
/// ```
pub fn format_age(now: SystemTime, written: SystemTime) -> String {
    let seconds: u64 = now.duration_since(written).map(|elapsed| return elapsed.as_secs()).unwrap_or(0);
    if seconds < SECONDS_PER_MINUTE {
        return format!("{seconds} s");
    }
    if seconds < SECONDS_PER_HOUR {
        return format!("{} min", seconds / SECONDS_PER_MINUTE);
    }
    if seconds < SECONDS_PER_DAY {
        return format!("{} h", seconds / SECONDS_PER_HOUR);
    }
    return format!("{} d", seconds / SECONDS_PER_DAY);
}

/// One thing a pass may remove:
/// a version directory with the downloads kept for it,
/// or a download file an earlier layout left.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrimCandidate {
    /// The store root the candidate lives in.
    pub root: PathBuf,
    /// The tool's directory name in the store.
    pub tool: String,
    /// The version,
    /// empty for a leftover download.
    pub version: String,
    /// The path the lines name: the version directory or the leftover file.
    pub directory: PathBuf,
    /// The directory holding the archives downloaded for this version.
    pub downloads: Option<PathBuf>,
    /// The space removing it gives back, in bytes.
    pub bytes: u64,
    /// When it was last written.
    pub written: SystemTime,
    /// Whether this is a download file and not a version directory.
    pub leftover: bool,
}

/// What:
/// Everything one pass may remove, in the order it is judged:
/// oldest write first.
/// Why:
/// The rule removes the versions written longest ago first,
/// so the order is the rule's own and is built here once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function candidatesOf(roots: string[], tools: ReadonlySet<string>): TrimCandidate[]
/// ```
pub fn candidates_of(roots: &[PathBuf], tools: &BTreeSet<String>) -> Vec<TrimCandidate> {
    let mut candidates: Vec<TrimCandidate> = Vec::new();
    for root in roots {
        for version in version_directories(root, tools) {
            candidates.push(candidate_of_version(root, version));
        }
        for leftover in leftover_downloads(root, tools) {
            candidates.push(TrimCandidate {
                root: root.clone(),
                tool: leftover.tool,
                version: String::new(),
                directory: leftover.file,
                downloads: None,
                bytes: leftover.bytes,
                written: leftover.written,
                leftover: true,
            });
        }
    }
    candidates.sort_by(compare_oldest_first);
    return candidates;
}

/// The candidate one version directory makes.
fn candidate_of_version(root: &Path, version: VersionDirectory) -> TrimCandidate {
    return TrimCandidate {
        root: root.to_path_buf(),
        tool: version.tool,
        version: version.version,
        directory: version.directory,
        downloads: version.downloads,
        bytes: version.bytes,
        written: version.written,
        leftover: false,
    };
}

/// Orders two candidates oldest write first,
/// by path when they were written in the same second.
fn compare_oldest_first(left: &TrimCandidate, right: &TrimCandidate) -> std::cmp::Ordering {
    let by_write: std::cmp::Ordering = left.written.cmp(&right.written);
    if by_write != std::cmp::Ordering::Equal {
        return by_write;
    }
    return left.directory.cmp(&right.directory);
}

/// What:
/// The versions the lock files name:
/// tool directory name to version to the lock that names it,
/// plus why a lock could not be read.
/// Why:
/// A version any lock names is never removed:
/// another tree's lock may pin it,
/// and about 150 worktrees of this repository each pin their own.
/// A lock that cannot be read is what makes the pass keep everything,
/// because an unreadable lock might name any version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamedVersions = { byTool: Map<string, Map<string, string>>; unreadable?: string };
/// export function readNamedVersions(locks: string[]): NamedVersions
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NamedVersions {
    /// Tool directory name, then version, then the lock file that names it.
    pub by_tool: BTreeMap<String, BTreeMap<String, String>>,
    /// Why a lock could not be read,
    /// so no version may be removed.
    pub unreadable: Option<String>,
}

/// What:
/// The versions every lock in `locks` pins,
/// and every tool directory those locks imply,
/// read without the full lock model so a lock of an older format still answers.
/// Why:
/// The scan of the store is driven by these directory names:
/// a tool no visible lock names is not scanned,
/// so nothing outside a lock's reach is ever removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readNamedVersions(locks: string[]): NamedVersions
/// ```
pub fn read_named_versions(locks: &[PathBuf]) -> NamedVersions {
    let mut named: NamedVersions = NamedVersions::default();
    for lock in locks {
        let text: String = match fs::read_to_string(lock) {
            Ok(text) => text,
            // Another worktree's lock may be gone by the time it is read (that tree removed its worktree).
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => {
                named.unreadable = Some(format!("{} cannot be read: {error}", lock.display()));
                return named;
            }
        };
        if let Err(reason) = named.add_lock_text(&text, lock) {
            named.unreadable = Some(reason);
            return named;
        }
    }
    return named;
}

/// The methods of `NamedVersions`.
impl NamedVersions {
    /// Reads one lock's pin rows into this set.
    fn add_lock_text(&mut self, text: &str, origin: &Path) -> Result<(), String> {
        let decoded: Value = match serde_json::from_str(text) {
            Ok(value) => value,
            Err(error) => return Err(format!("{} is not JSON: {error}", origin.display())),
        };
        let Some(tools) = decoded.get("tools").and_then(Value::as_object) else {
            return Err(format!("{} has no `tools` object", origin.display()));
        };
        for (id, entry) in tools {
            let Some(version) = entry.get("version").and_then(Value::as_str) else {
                // A row this pass cannot read might pin any version, so nothing is removed.
                return Err(format!(
                    "{} names a tool without a version (tools.{id}), so no version can be told from one a lock pins",
                    origin.display()
                ));
            };
            let Some(route) = entry.get("route").and_then(Value::as_str).and_then(Route::parse) else {
                // A route this release does not know keeps its payload somewhere this pass cannot see, and it pins
                // nothing in a version directory.
                continue;
            };
            // What: a crate `cargo binstall` installs is named by the crate, not by the tool id.
            // Why: its directory is `<root>/cargo/<crate>`, while every other route's directory is named after the id.
            let name_source: String = match route {
                Route::CargoBinstall => crate_of_entry(id, entry),
                _ => id.clone(),
            };
            let Some(directory_name) = install_directory_name(route, &name_source) else {
                continue;
            };
            self.by_tool
                .entry(directory_name)
                .or_default()
                .insert(version.to_string(), origin.display().to_string());
        }
        return Ok(());
    }

    /// The tool directory names this set knows.
    pub fn tool_directories(&self) -> BTreeSet<String> {
        return self.by_tool.keys().cloned().collect();
    }

    /// The lock naming `version` of the tool in `directory_name`,
    /// or `None` when no lock names it.
    pub fn naming_lock(&self, directory_name: &str, version: &str) -> Option<&str> {
        return self.by_tool.get(directory_name)?.get(version).map(String::as_str);
    }
}

/// One step the rule decided.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrimStep {
    /// Remove this version.
    Remove(TrimCandidate),
    /// Keep it, and say why.
    Keep(TrimCandidate, String),
}

/// What one pass decided.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrimPlan {
    /// The space every candidate holds.
    pub total_bytes: u64,
    /// Whether the stores keep more than the bounds allow.
    pub over_bound: bool,
    /// The candidates judged, oldest write first.
    pub steps: Vec<TrimStep>,
}

/// What:
/// The rule:
/// nothing while the stores keep no more than the size bound;
/// otherwise the versions written longest ago are removed until they do,
/// and each one that stays says why.
/// Why:
/// This is the whole decision, kept free of the file system,
/// so the tests feed it plain lists and the reasons in force are the caller's.
/// A version written inside the age floor,
/// a version a lock names,
/// one this run installed,
/// one a `current` link names,
/// and one a running process uses are all kept,
/// and the walk stops as soon as the space is inside the bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planTrim(candidates, bounds, now, protection): TrimPlan
/// ```
pub fn plan_trim(
    candidates: &[TrimCandidate],
    bounds: &TrimBounds,
    now: SystemTime,
    protection: &dyn Fn(&TrimCandidate) -> Option<String>,
) -> TrimPlan {
    let total_bytes: u64 = candidates.iter().map(|candidate| return candidate.bytes).fold(0u64, u64::saturating_add);
    if total_bytes <= bounds.size_bound_bytes {
        return TrimPlan { total_bytes, over_bound: false, steps: Vec::new() };
    }
    let mut remaining: u64 = total_bytes;
    let mut steps: Vec<TrimStep> = Vec::new();
    for candidate in candidates {
        if remaining <= bounds.size_bound_bytes {
            break;
        }
        let age: Duration = now.duration_since(candidate.written).unwrap_or(Duration::ZERO);
        if age < bounds.age_floor {
            steps.push(TrimStep::Keep(
                candidate.clone(),
                format!("it was written {} ago, inside the {} floor", format_age(now, candidate.written), format_age(now, now - bounds.age_floor)),
            ));
            continue;
        }
        match protection(candidate) {
            Some(reason) => steps.push(TrimStep::Keep(candidate.clone(), reason)),
            None => {
                remaining = remaining.saturating_sub(candidate.bytes);
                steps.push(TrimStep::Remove(candidate.clone()));
            }
        }
    }
    return TrimPlan { total_bytes, over_bound: true, steps };
}

/// The bounds and the clock one pass runs with.
#[derive(Clone, Copy, Debug)]
pub struct TrimRun {
    /// The bounds in force.
    pub bounds: TrimBounds,
    /// When the pass started,
    /// which the work limit is measured from.
    pub started: Instant,
}

/// What one pass did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrimOutcome {
    /// One line per version removed and per version kept,
    /// oldest write first.
    pub lines: Vec<String>,
    /// How many versions were removed.
    pub removed: usize,
    /// The space they gave back, in bytes.
    pub removed_bytes: u64,
    /// The store roots where a removal is still pending,
    /// because the pass ran out of its time limit.
    pub unfinished: BTreeSet<PathBuf>,
}

/// What:
/// The space a removal started and not finished leaves,
/// in the store's own words (`<root>/.mpm-shadow-removal-unfinished`).
/// Why:
/// The next install must finish what this one started,
/// even when it installs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function unfinished_marker(root: string): string
/// ```
pub fn unfinished_marker(root: &Path) -> PathBuf {
    return root.join(UNFINISHED_MARKER_NAME);
}

/// What:
/// Whether a store root holds the marker an unfinished removal leaves.
/// Why:
/// One `stat` answers whether the next install has to run the pass at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function has_unfinished_removal(root: string): boolean
/// ```
pub fn has_unfinished_removal(root: &Path) -> bool {
    return fs::symlink_metadata(unfinished_marker(root)).is_ok();
}

/// What:
/// Removes what the plan decided,
/// for at most the work limit,
/// and reports one line per version.
/// Why:
/// A rename out of the version's name comes first,
/// so a reader sees the version or its removal marker and never a half version,
/// and a kill at any later point leaves the marker for the next run to finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function applyTrim(plan: TrimPlan, run: TrimRun): TrimOutcome
/// ```
pub fn apply_trim(plan: &TrimPlan, run: &TrimRun) -> TrimOutcome {
    let mut outcome: TrimOutcome = TrimOutcome::default();
    let mut deferred: usize = 0;
    for step in &plan.steps {
        match step {
            TrimStep::Keep(candidate, reason) => outcome.lines.push(kept_line(candidate, reason)),
            TrimStep::Remove(candidate) => {
                if run.started.elapsed() >= run.bounds.work_limit {
                    deferred += 1;
                    outcome.unfinished.insert(candidate.root.clone());
                    continue;
                }
                match remove_one(candidate) {
                    Ok(Some(bytes)) => {
                        outcome.removed += 1;
                        outcome.removed_bytes = outcome.removed_bytes.saturating_add(bytes);
                        outcome.lines.push(removed_line(candidate));
                    }
                    // The version is gone already: another run removed it, and there is nothing to say.
                    Ok(None) => {}
                    Err(reason) => outcome.lines.push(kept_line(candidate, &reason)),
                }
            }
        }
    }
    if deferred > 0 {
        outcome.lines.push(format!(
            "mpm-shadow: the cleanup used its {} limit and left {deferred} version(s) for the next install",
            format_age(SystemTime::now(), SystemTime::now() - run.bounds.work_limit)
        ));
    }
    return outcome;
}

/// What:
/// Removes one candidate and returns the space it gave back,
/// or `None` when it was already gone.
/// Why:
/// The version directory's write time is read again before anything moves:
/// a version another run wrote after the scan is not the one that was judged,
/// and it stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeOne(candidate: TrimCandidate): number | undefined  // throws on a refusal
/// ```
fn remove_one(candidate: &TrimCandidate) -> Result<Option<u64>, String> {
    if candidate.leftover {
        return match fs::remove_file(&candidate.directory) {
            Ok(()) => Ok(Some(candidate.bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("it cannot be removed: {error}")),
        };
    }
    let metadata: fs::Metadata = match fs::symlink_metadata(&candidate.directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("it cannot be read: {error}")),
    };
    if metadata.modified().ok() != Some(candidate.written) {
        return Err("another run wrote it while the cleanup ran".to_string());
    }
    // What: the marker is written before the first rename.
    // Why: a run killed inside `remove_dir_all` leaves the marker, and the next install finishes the removal.
    let _ = fs::write(unfinished_marker(&candidate.root), b"");
    // What: the downloads are moved out of their name first, then the version.
    // Why: a kill between the two leaves the version whole and a downloads marker the next run finishes, never a
    // version that is gone while its downloads still sit under a version's name.
    let moved_downloads: Option<PathBuf> = match &candidate.downloads {
        Some(downloads) => {
            let marker: PathBuf = tombstone_of(downloads);
            match fs::rename(downloads, &marker) {
                Ok(()) => Some(marker),
                Err(error) if error.kind() == ErrorKind::NotFound => None,
                Err(error) => return Err(format!("its downloads cannot be moved aside: {error}")),
            }
        }
        None => None,
    };
    let version_marker: PathBuf = tombstone_of(&candidate.directory);
    if let Err(error) = fs::rename(&candidate.directory, &version_marker) {
        // The version stays, so its downloads go back to the name they had.
        if let Some(marker) = &moved_downloads
            && let Some(downloads) = &candidate.downloads
        {
            let _ = fs::rename(marker, downloads);
        }
        return Err(format!("it cannot be moved aside: {error}"));
    }
    for marker in [Some(version_marker), moved_downloads].into_iter().flatten() {
        match fs::remove_dir_all(&marker) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{} is left half removed: {error}", marker.display())),
        }
    }
    if let Some(downloads) = &candidate.downloads
        && let Some(tool_downloads) = downloads.parent()
    {
        // Best effort: a tool's downloads directory goes once its last version's archives are gone.
        let _ = fs::remove_dir(tool_downloads);
    }
    return Ok(Some(candidate.bytes));
}

/// What:
/// Finishes every removal an earlier run started and returns one line per removal finished.
/// Why:
/// The version is named by its removal marker until it is gone,
/// so finishing it is the only way the space comes back,
/// and a run that is killed again leaves the marker for the next run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function finish_removals(roots: string[], tools: ReadonlySet<string>, run: TrimRun): [string[], Set<string>]
/// ```
pub fn finish_removals(
    roots: &[PathBuf],
    tools: &BTreeSet<String>,
    run: &TrimRun,
) -> (Vec<String>, BTreeSet<PathBuf>) {
    let mut lines: Vec<String> = Vec::new();
    let mut unfinished: BTreeSet<PathBuf> = BTreeSet::new();
    for root in roots {
        for marker in tombstones(root, tools) {
            if run.started.elapsed() >= run.bounds.work_limit {
                unfinished.insert(root.clone());
                break;
            }
            let bytes: u64 = tree_size(&marker);
            match fs::remove_dir_all(&marker) {
                Ok(()) => lines.push(format!(
                    "mpm-shadow: removed {} ({}) and finished a removal an earlier run started",
                    marker.display(),
                    format_size(bytes)
                )),
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => {
                    unfinished.insert(root.clone());
                    lines.push(format!("mpm-shadow: kept {}: it is still being removed: {error}", marker.display()));
                }
            }
        }
    }
    return (lines, unfinished);
}

/// The line for a version that stays,
/// with the reason it stays.
fn kept_line(candidate: &TrimCandidate, reason: &str) -> String {
    return format!(
        "mpm-shadow: kept {} ({}): {reason}",
        candidate.directory.display(),
        format_size(candidate.bytes)
    );
}

/// The line for a version this pass removed.
fn removed_line(candidate: &TrimCandidate) -> String {
    return format!(
        "mpm-shadow: removed {} ({}), last written {} ago",
        candidate.directory.display(),
        format_size(candidate.bytes),
        format_age(SystemTime::now(), candidate.written)
    );
}

/// What:
/// Removes the marker of every root with nothing left to remove.
/// Why:
/// The marker is what makes the next install run the pass,
/// so it must go as soon as the removals are complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function settle_markers(roots: string[], tools: ReadonlySet<string>, unfinished: Set<string>): void
/// ```
pub fn settle_markers(roots: &[PathBuf], tools: &BTreeSet<String>, unfinished: &BTreeSet<PathBuf>) {
    for root in roots {
        if unfinished.contains(root) || !tombstones(root, tools).is_empty() {
            continue;
        }
        let _ = fs::remove_file(unfinished_marker(root));
    }
}

/// What:
/// The lock files a pass reads:
/// this scope's own,
/// the lock of every worktree of this repository,
/// and the machine-global scope's,
/// or why one of them cannot be found.
/// Why:
/// A version any of these pins is in use by some tree,
/// and the global store holds versions a repository lock pins,
/// so both scopes' locks are read from either scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function visibleLocks(ctx: Context): { files: string[]; unreadable?: string }
/// ```
fn visible_locks(ctx: &Context) -> (Vec<PathBuf>, Option<String>) {
    let mut files: Vec<PathBuf> = vec![ctx.scope.lock_file.clone()];
    let mut unreadable: Option<String> = None;
    if ctx.scope.kind == ScopeKind::Repo {
        match worktree_lock_files(ctx.runner, &ctx.scope.root) {
            Ok(listed) => files.extend(listed),
            Err(error) => unreadable = Some(error.message),
        }
    }
    if let Ok(global) = global_scope(ctx.env)
        && global.lock_file != ctx.scope.lock_file
    {
        files.push(global.lock_file);
    }
    return (files, unreadable);
}

/// The roots one scope may remove versions from:
/// both of this machine's stores from a repository scope,
/// which reads every worktree's lock,
/// and only the machine-global store from the global scope,
/// which cannot see the worktrees' locks.
fn roots_to_trim(ctx: &Context) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = match ctx.scope.kind {
        ScopeKind::Repo => vec![ctx.scope.toolchain_root.clone(), ctx.scope.other_toolchain_root.clone()],
        ScopeKind::Global => vec![ctx.scope.toolchain_root.clone()],
    };
    roots.retain(|root| return root.is_dir());
    return roots;
}

/// How a candidate is protected,
/// given what the pass knows.
fn protection_of(
    candidate: &TrimCandidate,
    named: &NamedVersions,
    holders: &StoreHolders,
    installed: &BTreeSet<(String, String)>,
    current_links: &BTreeMap<(PathBuf, String), String>,
) -> Option<String> {
    if installed.contains(&(candidate.tool.clone(), candidate.version.clone())) {
        return Some("this install put it there".to_string());
    }
    if let Some(lock) = named.naming_lock(&candidate.tool, &candidate.version) {
        return Some(format!("the lock {lock} names it"));
    }
    if let Some(named_version) = current_links.get(&(candidate.root.clone(), candidate.tool.clone()))
        && *named_version == candidate.version
    {
        return Some("the tool's current link names it".to_string());
    }
    if let Some(reason) = held_reason(holders, &candidate.directory) {
        return Some(format!("it is in use: {reason}"));
    }
    if let Some(reason) = opaque_reason(holders, &candidate.tool) {
        return Some(format!("it may be in use: {reason}"));
    }
    return None;
}

/// What:
/// The `current` link target of every tool of every root the pass trims.
/// Why:
/// A link left naming a removed version dangles,
/// and the tool's `current` is what every worktree's exposed links resolve through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentLinks(roots: string[], tools: ReadonlySet<string>): Map<[string, string], string>
/// ```
fn current_links(roots: &[PathBuf], tools: &BTreeSet<String>) -> BTreeMap<(PathBuf, String), String> {
    let mut links: BTreeMap<(PathBuf, String), String> = BTreeMap::new();
    for root in roots {
        for tool in tools {
            let link: PathBuf = root.join(tool).join(CURRENT_LINK_NAME);
            let Ok(target) = fs::read_link(&link) else {
                continue;
            };
            let name: String = target
                .file_name()
                .map(|name| return name.to_string_lossy().to_string())
                .unwrap_or_default();
            if !name.is_empty() {
                links.insert((root.clone(), tool.clone()), name);
            }
        }
    }
    return links;
}

/// What:
/// Runs the cleanup after a sync that installed `installed` (tool directory name and version),
/// and prints its lines on standard error.
/// Why:
/// This is the one place the cleanup happens,
/// so no command has to be remembered,
/// and a sync that installed nothing runs it only when an earlier removal is unfinished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trimAfterInstall(ctx: Context, installed: [string, string][]): void
/// ```
pub fn trim_after_install(ctx: &Context, installed: &[(String, String)]) {
    let roots: Vec<PathBuf> = roots_to_trim(ctx);
    if roots.is_empty() {
        return;
    }
    let pending: bool = roots.iter().any(|root| return has_unfinished_removal(root));
    if installed.is_empty() && !pending {
        return;
    }
    let bounds: TrimBounds = TrimBounds::from_env(ctx.env);
    let run: TrimRun = TrimRun { bounds, started: Instant::now() };
    let (lock_files, lock_problem) = visible_locks(ctx);
    let named: NamedVersions = match lock_problem {
        Some(reason) => NamedVersions { by_tool: BTreeMap::new(), unreadable: Some(reason) },
        None => read_named_versions(&lock_files),
    };
    let tools: BTreeSet<String> = named.tool_directories();
    let mut lines: Vec<String> = Vec::new();
    let mut unfinished: BTreeSet<PathBuf> = BTreeSet::new();
    if !tools.is_empty() {
        let (finished, still_pending) = finish_removals(&roots, &tools, &run);
        lines.extend(finished);
        unfinished.extend(still_pending);
    }
    if let Some(reason) = &named.unreadable {
        // What: a lock that cannot be read keeps every version.
        // Why: it might name any of them, so the pass cannot tell an old version from one in use.
        lines.push(format!(
            "mpm-shadow: removed no toolchain version: {reason}; every version is kept until the locks can be read"
        ));
    } else if !tools.is_empty() {
        let holders: StoreHolders = holders_of(&roots, run.started + bounds.work_limit);
        let installed_set: BTreeSet<(String, String)> = installed.iter().cloned().collect();
        let links: BTreeMap<(PathBuf, String), String> = current_links(&roots, &tools);
        let candidates: Vec<TrimCandidate> = candidates_of(&roots, &tools);
        // What: a scan that could not be completed keeps every version.
        // Why: a version a process the scan did not reach uses must not be removed.
        let unread: Option<String> = holders.incomplete.clone();
        let protection = |candidate: &TrimCandidate| -> Option<String> {
            if let Some(reason) = &unread {
                return Some(format!("the process table could not be read: {reason}"));
            }
            return protection_of(candidate, &named, &holders, &installed_set, &links);
        };
        let plan: TrimPlan = plan_trim(&candidates, &bounds, SystemTime::now(), &protection);
        let outcome: TrimOutcome = apply_trim(&plan, &run);
        lines.extend(outcome.lines);
        unfinished.extend(outcome.unfinished);
    }
    settle_markers(&roots, &tools, &unfinished);
    for line in lines {
        eprintln!("{line}");
    }
}
