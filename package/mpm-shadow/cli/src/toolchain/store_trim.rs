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
use super::archive_staging::has_complete_marker;
use super::cargo_installs::{CURRENT_LINK_NAME, recorded_version};
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
    RemovalMarker, VersionDirectory, downloads_of, inode_of, install_directory_name, leftover_downloads,
    removal_markers, tombstone_of, tree_size, version_directories,
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

/// What:
/// The longest one pass may spend removing:
/// one year.
/// Why:
/// `MPM_SHADOW_STORE_WORK_LIMIT` is read into a `Duration` and added to an `Instant`,
/// which panics rather than wrapping when the sum leaves what an `Instant` can hold
/// (`18446744073709551615` seconds did, and took the whole install down with it).
/// A year is longer than any pass, so a value above it already means "no limit".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_WORK_LIMIT = 365 * 24 * 3600;
/// ```
pub const MAX_WORK_LIMIT: Duration = Duration::from_secs(365 * 24 * 3600);

/// What:
/// The longest age floor the pass accepts:
/// one hundred years.
/// Why:
/// The floor is subtracted from a `SystemTime` for its line,
/// and an unbounded value panicked the install the same way the work limit did.
/// A hundred years is older than anything a store holds, so a value above it already means "keep everything".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_AGE_FLOOR = 100 * 365 * 24 * 3600;
/// ```
pub const MAX_AGE_FLOOR: Duration = Duration::from_secs(100 * 365 * 24 * 3600);

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
            // What: a span above its maximum is read as the maximum.
            // Why: both maxima already mean "effectively no limit", and neither can overflow an `Instant` or a
            // `SystemTime` when it is used, which an unbounded value did.
            age_floor: env
                .get(AGE_FLOOR_ENV)
                .and_then(parse_duration)
                .unwrap_or(Duration::from_secs(DEFAULT_AGE_FLOOR_SECONDS))
                .min(MAX_AGE_FLOOR),
            work_limit: env
                .get(WORK_LIMIT_ENV)
                .and_then(parse_duration)
                .unwrap_or(Duration::from_millis(DEFAULT_WORK_LIMIT_MILLIS))
                .min(MAX_WORK_LIMIT),
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
            return number.trim().parse::<u64>().ok().map(|value| return scaled(unit, value));
        }
    }
    return trimmed.parse::<u64>().ok().map(Duration::from_secs);
}

/// What:
/// What:
/// `count` of `unit` as a span,
/// and the longest span there is when the count does not fit the multiplier.
/// Why:
/// The count is a `u64` and the multiplier takes a `u32`,
/// and casting a count that does not fit truncated it instead of saturating:
/// `18446744073709551615ms` read as 49 days rather than as "no limit".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scaled(unit: Duration, count: bigint): Duration
/// ```
fn scaled(unit: Duration, count: u64) -> Duration {
    return match u32::try_from(count) {
        Ok(fits) => unit.saturating_mul(fits),
        Err(_) => Duration::MAX,
    };
}

/// What:
/// A span in the words a person reads (`41 h`, `3 d`, `12 min`).
/// Why:
/// The rules' lines say how long a floor is,
/// and a span is read from the span itself rather than from two moments,
/// which cannot overflow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function formatSpan(span: Duration): string
/// ```
pub fn format_span(span: Duration) -> String {
    let seconds: u64 = span.as_secs();
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
    /// The device and inode number of the version directory when it was judged,
    /// when the platform reports them.
    pub inode: Option<(u64, u64)>,
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
                inode: None,
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
        inode: version.inode,
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
                format!(
                    "it was written {} ago, inside the {} floor",
                    format_age(now, candidate.written),
                    format_span(bounds.age_floor)
                ),
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

/// The bounds, the clock and the store roots one pass runs with.
#[derive(Clone, Debug)]
pub struct TrimRun {
    /// The bounds in force.
    pub bounds: TrimBounds,
    /// When the pass started,
    /// which the work limit is measured from.
    pub started: Instant,
    /// The store roots this pass trims,
    /// and the only ones it asks the process table about.
    pub roots: Vec<PathBuf>,
}

/// The methods of `TrimRun`.
impl TrimRun {
    /// The moment this pass's work limit runs out.
    pub fn deadline(&self) -> Instant {
        // What: a limit that does not fit after the start is read as the longest one the pass allows.
        // Why: `Instant + Duration` panics on overflow, and a pass's bound must never end an install.
        return self.started.checked_add(self.bounds.work_limit).unwrap_or(self.started + MAX_WORK_LIMIT);
    }

    /// What the process table says now about the roots this pass trims.
    pub fn look(&self) -> StoreHolders {
        return holders_of(&self.roots, self.deadline());
    }
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
    /// because the pass ran out of its time limit or could not finish one.
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

/// What a removal moved out of the way before anything was deleted.
pub(crate) struct MovedAside {
    /// The marker holding the version directory.
    pub(crate) version_marker: PathBuf,
    /// The marker holding the downloads directory,
    /// when the removal moved one aside.
    pub(crate) downloads_marker: Option<PathBuf>,
}

/// Where a version's own trees stand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Trees {
    /// Under their own names.
    InPlace,
    /// Moved aside by a removal,
    /// so each tree stands under a marker and its own name is free.
    MovedAside,
}

/// What:
/// Every path one version of a tool owns in a store:
/// its directory and the archives downloaded for it,
/// and, while a removal holds them aside, the markers that hold them with the names they take back.
/// Why:
/// A hold can name any of these,
/// and the checks that keep a version a live process uses are only as good as the list they consult.
/// One list answers for all of them, so no check can forget a tree.
/// The two trees are the whole of what an install writes for a version:
/// `install_archive` downloads into `<root>/downloads/<tool>/<version>` and unpacks into
/// `<root>/<tool>/<version>`, `install_cargo` and `install_rust_android` write only under the version directory,
/// and a `cargo binstall` crate keeps its record inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownedPaths(root: string, tool: string, version: string, trees: Trees): string[]
/// ```
fn owned_paths(root: &Path, tool: &str, version: &str, trees: Trees) -> Vec<PathBuf> {
    let version_directory: PathBuf = root.join(tool).join(version);
    let downloads_directory: PathBuf = downloads_of(root, tool, version);
    return match trees {
        Trees::InPlace => vec![version_directory, downloads_directory],
        // Both the markers and the names they take back: a hold recorded before the rename keeps its descriptor,
        // and the path the descriptor reports is the one the tree stands under now or the one it left.
        Trees::MovedAside => vec![
            tombstone_of(&version_directory),
            version_directory,
            tombstone_of(&downloads_directory),
            downloads_directory,
        ],
    };
}

/// What:
/// The reason a live process keeps `version` of `tool`, or `None` when nothing below the paths it owns is held.
/// Why:
/// Every check that decides whether a version may be removed asks this,
/// so both of its trees and every marker are asked about, once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownedHoldReason(root: string, tool: string, version: string, trees: Trees, holders: StoreHolders): string | undefined
/// ```
fn owned_hold_reason(holders: &StoreHolders, root: &Path, tool: &str, version: &str, trees: Trees) -> Option<String> {
    for path in owned_paths(root, tool, version, trees) {
        if let Some(reason) = held_reason(holders, &path) {
            return Some(reason);
        }
    }
    return None;
}

/// What:
/// Whether the tool's `current` link names `version` right now.
/// Why:
/// A pass reads every `current` once, before it plans,
/// and another command can repoint the link between that read and the removal:
/// a version the link names at the moment of removal is never removed,
/// so the link is read again right then, and again after the move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentNames(root: string, tool: string, version: string): boolean
/// ```
fn current_names(root: &Path, tool: &str, version: &str) -> bool {
    let link: PathBuf = root.join(tool).join(CURRENT_LINK_NAME);
    let Ok(target) = fs::read_link(&link) else {
        return false;
    };
    let Some(name) = target.file_name() else {
        return false;
    };
    return name.to_string_lossy() == version;
}

/// What:
/// How `metadata` differs from the entry `candidate` was judged as,
/// or `None` when it is that entry.
/// Why:
/// A pass looks at a version, decides, and acts later,
/// and an install can replace the directory in between;
/// the entry an install put there is not the one that was judged and must not be removed.
/// The inode number tells one directory from another at the same name,
/// and the write time tells an install that wrote in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgedEntryDifference(metadata: Stats, candidate: TrimCandidate): string | undefined
/// ```
fn judged_entry_difference(metadata: &fs::Metadata, candidate: &TrimCandidate) -> Option<&'static str> {
    // What: the inode number is compared first.
    // Why: a replaced directory answers both questions, and the inode says which of the two happened.
    match (candidate.inode, inode_of(metadata)) {
        (Some(judged), Some(found)) if judged != found => return Some(NOT_THE_JUDGED_ENTRY),
        // A platform that reports no inode number is answered by the write time alone.
        _ => {}
    }
    if metadata.modified().ok() != Some(candidate.written) {
        return Some("another run wrote it while the cleanup ran");
    }
    return None;
}

/// What:
/// The refusal for an entry that is not the one a pass judged.
const NOT_THE_JUDGED_ENTRY: &str =
    "another run installed this version while the cleanup ran, so the entry found there is not the one judged";

/// What:
/// The first half of a removal:
/// reads the version's own entry and refuses unless it is the one the pass judged.
/// Why:
/// It is the cheap early answer, and it is a separate step from the move
/// so a control can hold the order between them without a race of threads:
/// whatever an install does between this step and the next is caught by the second look, inside the move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function judged_entry_still_there(candidate: TrimCandidate): boolean | undefined  // undefined: gone
/// ```
pub(crate) fn judged_entry_still_there(candidate: &TrimCandidate) -> Result<Option<()>, String> {
    let metadata: fs::Metadata = match fs::symlink_metadata(&candidate.directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("it cannot be read: {error}")),
    };
    if let Some(difference) = judged_entry_difference(&metadata, candidate) {
        return Err(difference.to_string());
    }
    return Ok(Some(()));
}

/// What:
/// The second half of a removal:
/// moves the version's downloads and then the version itself out of their names,
/// and puts both back unless the entry under the marker is the one the pass judged.
/// Why:
/// The marker is a name only this program writes,
/// so the entry standing under it after the rename is exactly what the removal holds,
/// and its inode and write time say whether it is the judged entry or one an install put there meanwhile.
/// A different entry goes back at once, so a fresh install is never what a removal deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function move_judged_entry(candidate: TrimCandidate): MovedAside | undefined  // throws on a refusal
/// ```
pub(crate) fn move_judged_entry(candidate: &TrimCandidate) -> Result<Option<MovedAside>, String> {
    // What: the marker is written before the first rename.
    // Why: a run killed inside `remove_dir_all` leaves the marker, and the next install finishes the removal.
    let _ = fs::write(unfinished_marker(&candidate.root), b"");
    // What: the downloads are moved out of their name first, then the version.
    // Why: a kill between the two leaves the version whole and a downloads marker the next run finishes, never a
    // version that is gone while its downloads still sit under a version's name.
    let downloads_marker: Option<PathBuf> = match &candidate.downloads {
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
        if let Some(marker) = &downloads_marker
            && let Some(downloads) = &candidate.downloads
        {
            let _ = fs::rename(marker, downloads);
        }
        return Err(format!("it cannot be moved aside: {error}"));
    }
    // What: the entry the removal now holds is compared with the one that was judged.
    // Why: an install that replaced the version between the look and this rename is holding a finished copy under
    // the marker now, and deleting it would discard the install.
    match fs::symlink_metadata(&version_marker) {
        Ok(held) if judged_entry_difference(&held, candidate).is_none() => {}
        Ok(held) => {
            let aside: MovedAside = MovedAside { version_marker: version_marker.clone(), downloads_marker };
            let difference: &str = judged_entry_difference(&held, candidate).unwrap_or(NOT_THE_JUDGED_ENTRY);
            let reason: String = match put_back(candidate, &aside) {
                PutBack::Back | PutBack::NameTaken | PutBack::NameUnfinished => difference.to_string(),
                PutBack::Failed(problem) => format!("{difference}; {problem}"),
            };
            return Err(reason);
        }
        // The marker is gone already: another run finished the removal this pass was making.
        Err(_) => return Ok(None),
    }
    return Ok(Some(MovedAside { version_marker, downloads_marker }));
}

/// What:
/// Moves one version out of its name,
/// in the two steps a control can hold apart:
/// the look that refuses an entry that is not the one judged,
/// then the move that refuses one an install put there between the look and itself.
/// Why:
/// A reader then sees the version or its removal marker and never a half version,
/// a kill at any later point leaves markers the next run finishes,
/// and the entry a removal deletes is always the one a pass judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveAside(candidate: TrimCandidate): MovedAside | undefined  // throws on a refusal
/// ```
fn move_aside(candidate: &TrimCandidate) -> Result<Option<MovedAside>, String> {
    if judged_entry_still_there(candidate)?.is_none() {
        return Ok(None);
    }
    return move_judged_entry(candidate);
}

/// What putting a version back under its name ended in.
enum PutBack {
    /// The version is back under its name.
    Back,
    /// The name holds another copy,
    /// so this copy is deleted instead of restored.
    NameTaken,
    /// The name holds an install that has not finished,
    /// so neither copy may be touched by this pass.
    NameUnfinished,
    /// Neither worked,
    /// and the marker stays.
    Failed(String),
}

/// What:
/// Whether the copy of `tool`'s version at `directory` is a finished install.
/// Why:
/// A held, finished copy may be deleted only in favour of a copy that finished.
/// A route that installs in place leaves a half-written directory under the version's name,
/// and deleting the good copy for it would leave the only working one gone.
/// An archive install writes its completion marker last and a `cargo binstall` crate writes its record last;
/// a route that leaves neither is never read as finished here,
/// so its copy stays aside rather than being deleted for a name this pass cannot judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finished_copy_at(tool: string, directory: string): boolean
/// ```
fn finished_copy_at(tool: &str, directory: &Path) -> bool {
    if has_complete_marker(directory) {
        return true;
    }
    if let Some(crate_name) = tool.strip_prefix("cargo/") {
        return recorded_version(directory, crate_name).is_some();
    }
    return false;
}

/// What:
/// Puts a moved-aside version and its downloads back under their names,
/// or deletes them when the name is taken again.
/// Why:
/// A version that turns out to be in use is not removed,
/// and its name is the only thing it lost.
/// A name that holds a copy again means another run installed that version while this pass held the old copy out
/// of the way, and then the old copy is the one to delete:
/// leaving it a marker would retry the restore on every later pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putBack(candidate: TrimCandidate, moved: MovedAside): 'back' | 'nameTaken' | { failed: string }
/// ```
fn put_back(candidate: &TrimCandidate, moved: &MovedAside) -> PutBack {
    return restore_or_delete(
        &candidate.tool,
        &moved.version_marker,
        &candidate.directory,
        moved.downloads_marker.as_deref(),
        candidate.downloads.as_deref(),
    );
}

/// What:
/// Puts a version and its downloads back under their names,
/// or deletes them when the version's name was taken again,
/// and says which way it went.
/// Why:
/// The removal a pass made and the removal an earlier run left are the same shape,
/// so both put a version back through this one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreOrDelete(versionMarker: string, versionDirectory: string, downloadsMarker?: string, downloadsDirectory?: string): PutBack
/// ```
fn restore_or_delete(
    tool: &str,
    version_marker: &Path,
    version_directory: &Path,
    downloads_marker: Option<&Path>,
    downloads_directory: Option<&Path>,
) -> PutBack {
    if fs::symlink_metadata(version_directory).is_ok() {
        // What: a name taken by an install that has not finished keeps the held copy aside.
        // Why: deleting the held, finished copy for a half-written one loses the only working copy,
        // and putting the held copy back would write over the install in flight.
        if !finished_copy_at(tool, version_directory) {
            return PutBack::NameUnfinished;
        }
        return match remove_trees(version_marker, downloads_marker) {
            None => PutBack::NameTaken,
            Some(failure) => PutBack::Failed(failure),
        };
    }
    if let Err(error) = fs::rename(version_marker, version_directory) {
        return PutBack::Failed(format!("{} could not be put back: {error}", version_directory.display()));
    }
    if let (Some(marker), Some(directory)) = (downloads_marker, downloads_directory)
        && let Err(error) = fs::rename(marker, directory)
    {
        return PutBack::Failed(format!("{} could not be put back: {error}", directory.display()));
    }
    return PutBack::Back;
}

/// Deletes the trees a removal moved aside,
/// whichever of them exist.
fn remove_trees(version_marker: &Path, downloads_marker: Option<&Path>) -> Option<String> {
    if let Some(failure) = remove_dir_or_file(version_marker) {
        return Some(format!("{} is left half removed: {failure}", version_marker.display()));
    }
    if let Some(marker) = downloads_marker
        && let Some(failure) = remove_dir_or_file(marker)
    {
        return Some(format!("{} is left half removed: {failure}", marker.display()));
    }
    return None;
}

/// What:
/// Deletes what a removal moved aside.
/// Why:
/// This is the only step that gives the space back,
/// and it runs after the process table was asked about the moved version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deleteMoved(candidate: TrimCandidate, moved: MovedAside): string | undefined  // the failure, if any
/// ```
fn delete_moved(candidate: &TrimCandidate, moved: &MovedAside) -> Option<String> {
    if let Some(failure) = remove_trees(&moved.version_marker, moved.downloads_marker.as_deref()) {
        return Some(failure);
    }
    if let Some(downloads) = &candidate.downloads
        && let Some(tool_downloads) = downloads.parent()
    {
        // Best effort: a tool's downloads directory goes once its last version's archives are gone.
        let _ = fs::remove_dir(tool_downloads);
    }
    return None;
}

/// What:
/// Removes one download file an earlier layout left.
/// Why:
/// Such a file is not a version:
/// unlinking it is one atomic step, so there is nothing to move aside and nothing to finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeLeftover(candidate: TrimCandidate): boolean  // false when it was gone already
/// ```
fn remove_leftover(candidate: &TrimCandidate) -> Result<bool, String> {
    return match fs::remove_file(&candidate.directory) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("it cannot be removed: {error}")),
    };
}

/// What:
/// Removes what the plan decided,
/// for at most the work limit,
/// and reports one line per version.
/// Why:
/// Every version is moved out of its name first,
/// and the process table is asked once the whole batch is moved,
/// because nothing can start using a version once it is renamed:
/// that one look is the moment the removals become final,
/// and a version something holds goes back under its name and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function applyTrim(plan: TrimPlan, run: TrimRun): TrimOutcome
/// ```
pub fn apply_trim(plan: &TrimPlan, run: &TrimRun) -> TrimOutcome {
    let mut outcome: TrimOutcome = TrimOutcome::default();
    let mut lines: Vec<Option<String>> = vec![None; plan.steps.len()];
    let mut moved: Vec<(usize, TrimCandidate, MovedAside)> = Vec::new();
    let mut deferred: usize = 0;
    for (index, step) in plan.steps.iter().enumerate() {
        match step {
            TrimStep::Keep(candidate, reason) => lines[index] = Some(kept_line(candidate, reason)),
            TrimStep::Remove(candidate) => {
                if run.started.elapsed() >= run.bounds.work_limit {
                    deferred += 1;
                    outcome.unfinished.insert(candidate.root.clone());
                    continue;
                }
                if candidate.leftover {
                    match remove_leftover(candidate) {
                        Ok(true) => {
                            outcome.removed += 1;
                            outcome.removed_bytes = outcome.removed_bytes.saturating_add(candidate.bytes);
                            lines[index] = Some(removed_line(candidate));
                        }
                        // The file is gone already: another run removed it, and there is nothing to say.
                        Ok(false) => {}
                        Err(reason) => lines[index] = Some(kept_line(candidate, &reason)),
                    }
                    continue;
                }
                match move_aside(candidate) {
                    Ok(Some(aside)) => moved.push((index, candidate.clone(), aside)),
                    Ok(None) => {}
                    Err(reason) => lines[index] = Some(kept_line(candidate, &reason)),
                }
            }
        }
    }
    if !moved.is_empty() {
        let holders: StoreHolders = run.look();
        for (index, candidate, aside) in moved {
            let reason: Option<String> = match &holders.incomplete {
                Some(failed) => Some(format!("the process table could not be read: {failed}")),
                None => {
                    // What: the link is read again after the move, with the holders.
                    // Why: a repoint that landed between the refusal in `move_aside` and the move itself is seen
                    // here, and the version goes back under its name.
                    if current_names(&candidate.root, &candidate.tool, &candidate.version) {
                        Some("the tool's current link names it".to_string())
                    } else {
                        match owned_hold_reason(
                            &holders,
                            &candidate.root,
                            &candidate.tool,
                            &candidate.version,
                            Trees::MovedAside,
                        ) {
                            Some(held) => Some(format!("it is in use: {held}")),
                            None => opaque_reason(&holders, &candidate.tool)
                                .map(|found| return format!("it may be in use: {found}")),
                        }
                    }
                }
            };
            match reason {
                Some(why) => match put_back(&candidate, &aside) {
                    PutBack::Back => lines[index] = Some(kept_line(&candidate, &why)),
                    PutBack::NameTaken => {
                        outcome.removed += 1;
                        outcome.removed_bytes = outcome.removed_bytes.saturating_add(candidate.bytes);
                        lines[index] = Some(format!(
                            "{}, and a newer copy took its name while it was held, so the held copy was deleted",
                            removed_line(&candidate)
                        ));
                    }
                    PutBack::NameUnfinished => {
                        outcome.unfinished.insert(candidate.root.clone());
                        lines[index] = Some(kept_line(
                            &candidate,
                            &format!("{why}; the version's name holds an install that has not finished, so the held copy stays aside"),
                        ));
                    }
                    PutBack::Failed(failure) => {
                        outcome.unfinished.insert(candidate.root.clone());
                        lines[index] = Some(kept_line(&candidate, &format!("{why}; {failure}")));
                    }
                },
                None => match delete_moved(&candidate, &aside) {
                    None => {
                        outcome.removed += 1;
                        outcome.removed_bytes = outcome.removed_bytes.saturating_add(candidate.bytes);
                        lines[index] = Some(removed_line(&candidate));
                    }
                    Some(failure) => {
                        outcome.unfinished.insert(candidate.root.clone());
                        lines[index] = Some(kept_line(&candidate, &failure));
                    }
                },
            }
        }
    }
    // What: a root whose removals were all deferred gets the marker too.
    // Why: the marker is what makes the next install run the pass, and only the move wrote it before, so a pass
    // that moved nothing left no trace and the cleanup waited for an install that happened to install something.
    for root in &outcome.unfinished {
        let _ = fs::write(unfinished_marker(root), b"");
    }
    for line in lines.into_iter().flatten() {
        outcome.lines.push(line);
    }
    if deferred > 0 {
        outcome.lines.push(format!(
            "mpm-shadow: the cleanup used its {} limit and left {deferred} version(s) for the next install",
            format_age(SystemTime::now(), SystemTime::now() - run.bounds.work_limit)
        ));
    }
    return outcome;
}

/// The line for a version that stays,
/// with the reason it stays.
fn kept_line(candidate: &TrimCandidate, reason: &str) -> String {
    return format!("mpm-shadow: kept {} ({}): {reason}", candidate.directory.display(), format_size(candidate.bytes));
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

/// The lines and the work finishing the removals of earlier runs leaves.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FinishOutcome {
    /// One line per removal finished, undone or left pending.
    pub lines: Vec<String>,
    /// The store roots where a removal is still pending.
    pub unfinished: BTreeSet<PathBuf>,
    /// The versions put back under their names,
    /// each with the reason it was in use,
    /// so this pass's own plan leaves them alone.
    pub put_back: BTreeMap<PathBuf, String>,
}

/// What:
/// Finishes every removal an earlier run started,
/// or puts a version back when something is using it,
/// with one look at the process table for the whole batch.
/// Why:
/// Such a version is already condemned,
/// so finishing it needs no lock and no plan,
/// and the marker that brings the next install here goes only once the store holds no marker at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function finishRemovals(run: TrimRun): FinishOutcome
/// ```
pub fn finish_removals(run: &TrimRun) -> FinishOutcome {
    let mut outcome: FinishOutcome = FinishOutcome::default();
    let mut markers: Vec<RemovalMarker> = Vec::new();
    for root in &run.roots {
        markers.extend(removal_markers(root));
    }
    if markers.is_empty() {
        return outcome;
    }
    let holders: StoreHolders = run.look();
    for marker in markers {
        if run.started.elapsed() >= run.bounds.work_limit {
            outcome.unfinished.insert(marker.root.clone());
            break;
        }
        let version_present: bool = fs::symlink_metadata(&marker.version_marker).is_ok();
        if !version_present {
            // What: a marker for the downloads alone, with the version still under its name, is a removal that
            // never got past its first rename: `move_aside` moves the downloads first, `put_back` restores the
            // version first. The version is live, and what it was downloaded as belongs to it.
            // Why: reading "no marker for the version" as "condemned" deleted the archives of a version every
            // protection keeps.
            if fs::symlink_metadata(&marker.version_directory).is_ok() {
                match restore_downloads(&marker) {
                    None => outcome.lines.push(format!(
                        "mpm-shadow: kept {} ({}): the version it belongs to is still there, so what a removal \
                         moved aside is put back",
                        marker.downloads_directory.as_ref().map(|path| return path.display().to_string()).unwrap_or_default(),
                        format_size(removal_marker_size(&marker))
                    )),
                    Some(failure) => {
                        outcome.unfinished.insert(marker.root.clone());
                        outcome.lines.push(format!(
                            "mpm-shadow: kept {}: it is still being removed: {failure}",
                            marker.version_directory.display()
                        ));
                    }
                }
                continue;
            }
            // The version is gone: only what it was downloaded as is left, and that goes.
            outcome.lines.push(finish_downloads_line(&marker));
            continue;
        }
        let bytes: u64 = removal_marker_size(&marker);
        let reason: Option<String> = match &holders.incomplete {
            Some(failed) => Some(format!("the process table could not be read: {failed}")),
            None => {
                if current_names(&marker.root, &marker.tool, &marker.version) {
                    Some("the tool's current link names it".to_string())
                } else {
                    match owned_hold_reason(&holders, &marker.root, &marker.tool, &marker.version, Trees::MovedAside) {
                        Some(held) => Some(format!("it is in use: {held}")),
                        None => opaque_reason(&holders, &marker.tool)
                            .map(|found| return format!("it may be in use: {found}")),
                    }
                }
            }
        };
        let candidate_reason: String = match reason {
            Some(why) => why,
            None => {
                match remove_marker(&marker) {
                    None => outcome.lines.push(format!(
                        "mpm-shadow: removed {} ({}) and finished a removal an earlier run started",
                        marker.version_directory.display(),
                        format_size(bytes)
                    )),
                    Some(failure) => {
                        outcome.unfinished.insert(marker.root.clone());
                        outcome.lines.push(format!(
                            "mpm-shadow: kept {}: it is still being removed: {failure}",
                            marker.version_directory.display()
                        ));
                    }
                }
                continue;
            }
        };
        match restore_marker(&marker) {
            Restored::Back => {
                outcome.put_back.insert(marker.version_directory.clone(), candidate_reason.clone());
                outcome.lines.push(format!(
                    "mpm-shadow: kept {} ({}): {candidate_reason}; the removal an earlier run started is undone",
                    marker.version_directory.display(),
                    format_size(bytes)
                ));
            }
            Restored::NameUnfinished => {
                outcome.unfinished.insert(marker.root.clone());
                outcome.lines.push(format!(
                    "mpm-shadow: kept {}: {candidate_reason}; the version's name holds an install that has not \
                     finished, so the held copy stays aside",
                    marker.version_marker.display()
                ));
            }
            Restored::NameTaken => outcome.lines.push(format!(
                "mpm-shadow: removed {} ({}) and finished a removal an earlier run started, and a newer copy holds \
                 its name",
                marker.version_marker.display(),
                format_size(bytes)
            )),
            Restored::Failed(failure) => {
                outcome.unfinished.insert(marker.root.clone());
                outcome.lines.push(format!(
                    "mpm-shadow: kept {}: {candidate_reason}; it is still being removed: {failure}",
                    marker.version_directory.display()
                ));
            }
        }
    }
    return outcome;
}

/// The space a removal marker holds,
/// the version and the downloads counted together.
fn removal_marker_size(marker: &RemovalMarker) -> u64 {
    let mut bytes: u64 = tree_size(&marker.version_marker);
    if let Some(downloads) = &marker.downloads_marker {
        bytes = bytes.saturating_add(tree_size(downloads));
    }
    return bytes;
}

/// What:
/// Puts the downloads a marker holds back under the name the version keeps them at.
/// Why:
/// They belong to a version that is still there,
/// and a newer install may have written its own archives under that name already,
/// in which case this copy is the older one and goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreDownloads(marker: RemovalMarker): string | undefined  // the failure, if any
/// ```
fn restore_downloads(marker: &RemovalMarker) -> Option<String> {
    let (Some(downloads_marker), Some(downloads_directory)) = (&marker.downloads_marker, &marker.downloads_directory)
    else {
        return None;
    };
    if fs::symlink_metadata(downloads_directory).is_ok() {
        return remove_dir_or_file(downloads_marker);
    }
    return match fs::rename(downloads_marker, downloads_directory) {
        Ok(()) => None,
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => Some(format!("{} could not be put back: {error}", downloads_directory.display())),
    };
}

/// The line for a marker whose version is gone and whose downloads were left aside.
fn finish_downloads_line(marker: &RemovalMarker) -> String {
    let Some(downloads) = &marker.downloads_marker else {
        return format!("mpm-shadow: nothing to remove under {}", marker.version_marker.display());
    };
    let bytes: u64 = tree_size(downloads);
    return match remove_dir_or_file(downloads) {
        None => format!(
            "mpm-shadow: removed {} ({}) and finished a removal an earlier run started",
            downloads.display(),
            format_size(bytes)
        ),
        Some(failure) => format!("mpm-shadow: kept {}: it is still being removed: {failure}", downloads.display()),
    };
}

/// Deletes a directory or a file,
/// whichever the path is.
fn remove_dir_or_file(path: &Path) -> Option<String> {
    let is_directory: bool = fs::symlink_metadata(path).map(|metadata| return metadata.is_dir()).unwrap_or(false);
    let result: std::io::Result<()> = if is_directory { fs::remove_dir_all(path) } else { fs::remove_file(path) };
    return match result {
        Ok(()) => None,
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => Some(error.to_string()),
    };
}

/// Deletes what one removal marker holds.
fn remove_marker(marker: &RemovalMarker) -> Option<String> {
    return remove_trees(&marker.version_marker, marker.downloads_marker.as_deref());
}

/// What putting a removal marker's version back ended in.
enum Restored {
    /// The version is back under its name.
    Back,
    /// The name holds another copy,
    /// so the marker is deleted instead.
    NameTaken,
    /// The name holds an install that has not finished,
    /// so the marker stays.
    NameUnfinished,
    /// Neither worked.
    Failed(String),
}

/// Puts a removal marker's version and downloads back under their names,
/// or deletes them when the name is taken again.
fn restore_marker(marker: &RemovalMarker) -> Restored {
    return match restore_or_delete(
        &marker.tool,
        &marker.version_marker,
        &marker.version_directory,
        marker.downloads_marker.as_deref(),
        marker.downloads_directory.as_deref(),
    ) {
        PutBack::Back => Restored::Back,
        PutBack::NameTaken => Restored::NameTaken,
        PutBack::NameUnfinished => Restored::NameUnfinished,
        PutBack::Failed(failure) => Restored::Failed(failure),
    };
}

/// What:
/// Removes the marker of every root whose store holds no removal in progress.
/// Why:
/// The marker is what makes the next install run the pass,
/// so it must go as soon as the removals are complete,
/// and it must stay while any removal marker stands,
/// which is read from the store itself and not from a list of locks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function settle_markers(roots: string[], unfinished: Set<string>): void
/// ```
pub fn settle_markers(roots: &[PathBuf], unfinished: &BTreeSet<PathBuf>) {
    for root in roots {
        if unfinished.contains(root) || !removal_markers(root).is_empty() {
            continue;
        }
        let _ = fs::remove_file(unfinished_marker(root));
    }
}

/// What:
/// The lock files a pass reads:
/// this scope's own,
/// the lock of every worktree of this repository when this is a repository scope,
/// and the machine-global scope's,
/// or why one of them cannot be found.
/// Why:
/// A version any of these pins is in use by some tree.
/// A repository scope reads the worktrees' locks and the machine scope's,
/// so it protects versions in either store;
/// the machine scope reads only its own lock,
/// because it knows no repository to list worktrees from and is given no path to one,
/// so a global-store version that only a worktree's lock names can be removed by a global pass.
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
    put_back: &BTreeMap<PathBuf, String>,
) -> Option<String> {
    if let Some(reason) = put_back.get(&candidate.directory) {
        return Some(reason.clone());
    }
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
    if let Some(reason) = owned_hold_reason(holders, &candidate.root, &candidate.tool, &candidate.version, Trees::InPlace)
    {
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
            let name: String = target.file_name().map(|name| return name.to_string_lossy().to_string()).unwrap_or_default();
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
/// The removals an earlier run left half done are settled first and without the locks,
/// because such a version is condemned already.
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
    let run: TrimRun = TrimRun { bounds, started: Instant::now(), roots: roots.clone() };
    let mut lines: Vec<String> = Vec::new();
    let mut unfinished: BTreeSet<PathBuf> = BTreeSet::new();
    // What: the removals an earlier run started are finished or undone before anything else is judged.
    // Why: their marker is the only record that a version's space is not back, and a lock that cannot be read must
    // not make the store look empty of pending removals.
    let finished: FinishOutcome = finish_removals(&run);
    lines.extend(finished.lines);
    unfinished.extend(finished.unfinished);
    let (lock_files, lock_problem) = visible_locks(ctx);
    let named: NamedVersions = match lock_problem {
        Some(reason) => NamedVersions { by_tool: BTreeMap::new(), unreadable: Some(reason) },
        None => read_named_versions(&lock_files),
    };
    let tools: BTreeSet<String> = named.tool_directories();
    if let Some(reason) = &named.unreadable {
        // What: a lock that cannot be read keeps every version.
        // Why: it might name any of them, so the pass cannot tell an old version from one in use.
        lines.push(format!(
            "mpm-shadow: removed no toolchain version: {reason}; every version is kept until the locks can be read"
        ));
    } else if !tools.is_empty() {
        let holders: StoreHolders = run.look();
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
            return protection_of(candidate, &named, &holders, &installed_set, &links, &finished.put_back);
        };
        let plan: TrimPlan = plan_trim(&candidates, &bounds, SystemTime::now(), &protection);
        let outcome: TrimOutcome = apply_trim(&plan, &run);
        lines.extend(outcome.lines);
        unfinished.extend(outcome.unfinished);
    }
    settle_markers(&roots, &unfinished);
    for line in lines {
        eprintln!("{line}");
    }
}
