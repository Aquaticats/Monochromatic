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
//! (a 3.1 GiB video-card build of llama.cpp,
//! ten times over,
//! and 4.5 GiB of downloads an earlier layout left).
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

/// Imports `CURRENT_LINK_NAME` and `shell_word` from `super::cargo_installs`.
use super::cargo_installs::{CURRENT_LINK_NAME, shell_word};
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `Route` from `super::model`.
use super::model::Route;
/// Imports what this module uses from `super::download_ownership`,
/// which says who owns a version's downloads while an install writes them and who may remove them.
use super::download_ownership::{RemovalClaim, RemovalRight, live_owner, removal_held, take_removal_right};
/// Imports `remove_orphan_downloads` from `super::published_archive`,
/// which owns where a version's archives are kept and what tells an install's use of them.
use super::published_archive::remove_orphan_downloads;
/// Imports `crate_of_entry` from `super::prune_cargo`,
/// which reads a lock row's crate name the way the prune of unnamed cargo versions does.
use super::prune_cargo::crate_of_entry;
/// Imports `ScopeKind` and `global_scope` from `super::scope`.
use super::scope::{EnvMap, ScopeKind, global_scope};
/// Imports what this module uses from `super::store_path_holders`.
use super::store_path_holders::{StoreHolders, held_reason, holders_of, opaque_reason};
/// Imports what this module uses from `super::store_versions`.
use super::store_versions::{
    DOWNLOADS_DIRECTORY_NAME, EntryNaming, TOMBSTONE_PREFIX, VersionDirectory, downloads_in_use, downloads_of,
    install_directory_name, leftover_downloads, tombstone_of, tombstones, tree_size, version_directories,
};
/// Imports `worktree_lock_files` from `super::worktrees`.
use super::worktrees::worktree_lock_files;
/// Imports what this module uses from `super::util`:
/// `file_name_text`,
/// which spells a path's last name for the suffix a claim adds to it,
/// and the process identity that tells a claim a live run holds from one a kill left.
use super::util::{ProcessIdentity, file_name_text, process_has_ended, this_process};

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
/// (`doc/toolchain-version-cleanup.md`,
/// "The measurement it was sized from"):
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
/// A proposal,
/// and the only bound here no measurement settles:
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
/// (4743 files in 145 ms,
/// 8820 in 260 ms,
/// and 1180 files of 3.2 GiB in 62 ms),
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
/// A byte count written as a number with an optional `KiB`,
/// `MiB`,
/// `GiB`,
/// `TiB` (or `K`,
/// `M`,
/// `G`,
/// `T`) suffix.
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
/// A span written as a number of seconds with an optional `s`,
/// `m`,
/// `h` or `d` suffix,
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
/// A span in the words a person reads (`41 h`,
/// `3 d`,
/// `12 min`).
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
    /// The path the lines name:
    /// the version directory or the leftover file.
    pub directory: PathBuf,
    /// The directory holding the archives downloaded for this version.
    pub downloads: Option<PathBuf>,
    /// The space removing it gives back,
    /// in bytes.
    pub bytes: u64,
    /// When it was last written.
    pub written: SystemTime,
    /// The numbers that name the entry,
    /// as the scan read them: what proves at the act that it is the entry that was judged.
    pub naming: EntryNaming,
    /// Whether this is a download file and not a version directory.
    pub leftover: bool,
}

/// What:
/// Everything one pass may remove,
/// in the order it is judged:
/// oldest write first.
/// Why:
/// The rule removes the versions written longest ago first,
/// so the order is the rule's own and is built here once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function candidatesOf(roots: string[], tools: ReadonlySet<string>): TrimCandidate[]
/// ```
pub fn candidates_of(roots: &[PathBuf], tools: &BTreeSet<String>, deadline: Instant) -> (Vec<TrimCandidate>, Vec<String>) {
    let mut candidates: Vec<TrimCandidate> = Vec::new();
    let mut incomplete: Vec<String> = Vec::new();
    for root in roots {
        let (versions, stopped) = version_directories(root, tools, deadline);
        if let Some(reason) = stopped {
            incomplete.push(format!("{}: {reason}", root.display()));
        }
        for version in versions {
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
                naming: leftover.naming,
                leftover: true,
            });
        }
    }
    candidates.sort_by(compare_oldest_first);
    return (candidates, incomplete);
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
        naming: version.naming,
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
    /// Tool directory name,
    /// then version,
    /// then the lock file that names it.
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
    /// Keep it,
    /// and say why.
    Keep(TrimCandidate, String),
}

/// What one pass decided.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrimPlan {
    /// The space every candidate holds.
    pub total_bytes: u64,
    /// Whether the stores keep more than the bounds allow.
    pub over_bound: bool,
    /// The candidates judged,
    /// oldest write first.
    pub steps: Vec<TrimStep>,
}

/// What:
/// The rule:
/// nothing while the stores keep no more than the size bound;
/// otherwise the versions written longest ago are removed until they do,
/// and each one that stays says why.
/// Why:
/// This is the whole decision,
/// kept free of the file system,
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
    /// The space they gave back,
    /// in bytes.
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
                match remove_one(candidate, run) {
                    Removal::Removed(bytes) => {
                        outcome.removed += 1;
                        outcome.removed_bytes = outcome.removed_bytes.saturating_add(bytes);
                        outcome.lines.push(removed_line(candidate));
                    }
                    // The version is gone already: another run removed it, and there is nothing to say.
                    Removal::Gone => {}
                    Removal::Kept(reason) => outcome.lines.push(kept_line(candidate, &reason)),
                    Removal::Deferred => {
                        deferred += 1;
                        outcome.unfinished.insert(candidate.root.clone());
                    }
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
/// Removes a flat download an earlier layout left,
/// by moving it to a hidden name of its own first and deleting it only when what moved is the file the scan measured.
/// Why:
/// Nothing writes these names any more, but an install can still publish an archive at one of them
/// (`downloads/<tool>-<version>-<file>`),
/// and a check and then a delete would take the new archive from an install that landed in between:
/// the move is one atomic step, so whatever stood at the name is in hand and can be compared with the one judged,
/// and anything else goes back.
fn remove_leftover(candidate: &TrimCandidate) -> Removal {
    let name: String = file_name_text(&candidate.directory);
    let set_aside: PathBuf = candidate.directory.with_file_name(format!(".{name}.{}", std::process::id()));
    match fs::rename(&candidate.directory, &set_aside) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Removal::Gone,
        Err(error) => return Removal::Kept(format!("it cannot be moved aside: {error}")),
    }
    let moved: Option<fs::Metadata> = fs::symlink_metadata(&set_aside).ok();
    let judged_still_there: bool = moved.as_ref().is_some_and(|metadata| {
        return metadata.modified().ok() == Some(candidate.written) && EntryNaming::of(metadata) == candidate.naming;
    });
    if !judged_still_there {
        // What: a file another run published after the scan goes back to the name it had.
        // Why: it is not the file that was judged, and the install that wrote it is about to unpack it.
        return match fs::rename(&set_aside, &candidate.directory) {
            Ok(()) => Removal::Kept("another run wrote it while the cleanup ran".to_string()),
            Err(error) => Removal::Kept(format!(
                "another run wrote it while the cleanup ran, and what it wrote cannot be put back: {error}; \
                 it is kept at {} instead of {}",
                set_aside.display(),
                candidate.directory.display()
            )),
        };
    }
    return match fs::remove_file(&set_aside) {
        Ok(()) => Removal::Removed(candidate.bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => Removal::Gone,
        Err(error) => Removal::Kept(format!("it was moved aside and cannot be removed: {error}; it is at {}", set_aside.display())),
    };
}

/// What one removal did.
/// Why:
/// A pass that ran out of its time is not a version that was kept for a reason about the version,
/// and the report has to tell them apart:
/// one is work for the next install,
/// the other says what about the version keeps it.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Removal {
    /// The version went,
    /// and gave back this many bytes.
    Removed(u64),
    /// Nothing stood at its name any more.
    Gone,
    /// The version stayed,
    /// for this reason.
    Kept(String),
    /// The pass's time ran out with the version still in place.
    Deferred,
}

/// What:
/// Removes one candidate and returns the space it gave back,
/// or `None` when it was already gone.
/// Why:
/// The version directory's write time is read again before anything moves:
/// a version another run wrote after the scan is not the one that was judged,
/// and it stays.
/// The write time alone cannot carry that across the move,
/// because another run publishes a version by renaming a fresh directory onto the version's name,
/// which changes the entry at that name without changing anything this run can read before the move;
/// so the version is moved aside first and what was moved is compared with what was judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeOne(candidate: TrimCandidate): number | undefined  // throws on a refusal
/// ```
fn remove_one(candidate: &TrimCandidate, run: &TrimRun) -> Removal {
    // What: the deadline is looked at before each step that moves something, and never between the last rename and
    // the delete.
    // Why: the steps before that can be undone where they are, and a recursive delete cannot be cut in the middle
    // without leaving half a version: one that is cut leaves its tombstone behind, which carries the marker, and the
    // next pass finishes it (`finish_removals`).
    let out_of_time = || return run.started.elapsed() >= run.bounds.work_limit;
    if candidate.leftover {
        return remove_leftover(candidate);
    }
    if out_of_time() {
        return Removal::Deferred;
    }
    let metadata: fs::Metadata = match fs::symlink_metadata(&candidate.directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Removal::Gone,
        Err(error) => return Removal::Kept(format!("it cannot be read: {error}")),
    };
    let judged: JudgedVersion = JudgedVersion::of(&metadata);
    if judged.written != Some(candidate.written) {
        return Removal::Kept("another run wrote it while the cleanup ran".to_string());
    }
    // What: a version whose downloads an install is writing stays, whoever asked for its removal.
    // Why: the archives are that install's, and moving or deleting them fails it for a version this pass judged from
    // a snapshot of the locks that is older than the install.
    if let Some(downloads) = &candidate.downloads
        && let Some(reason) = downloads_in_use(downloads)
    {
        return Removal::Kept(format!("an install is using its downloads: {reason}"));
    }
    // What: the name the version will be moved to is chosen before anything moves.
    // Why: a name that cannot be had keeps the version where it is, and the pass has written nothing to undo.
    let Some(claimed) = claim_name_beside(&candidate.directory) else {
        return Removal::Kept("no free claim name is left beside it, so it is not moved aside".to_string());
    };
    // What: the marker is written and known to be written before the first move.
    // Why: the marker is what makes a later sync look at this root,
    // so destructive work must not begin without it:
    // a removal that cannot write it removes nothing, and its line says why.
    if let Err(error) = fs::write(unfinished_marker(&candidate.root), b"") {
        return Removal::Kept(format!(
            "its recovery marker {} cannot be written, so nothing was removed: {error}",
            unfinished_marker(&candidate.root).display()
        ));
    }
    if out_of_time() {
        return Removal::Deferred;
    }
    // What: the downloads are taken by a claim before they are moved, and an install that owns them keeps the
    // version whole.
    // Why: the claim is what an install looks for after it writes its owner file,
    // so no install starts writing here while this call holds it,
    // and an install that started before it is seen and left alone;
    // a check and then a move would leave the instant between them open.
    let mut downloads_right: Option<RemovalClaim> = None;
    // What: the downloads are moved out of their name first, then the version.
    // Why: a kill between the two leaves the version whole and a downloads marker the next run finishes, never a
    // version that is gone while its downloads still sit under a version's name.
    let moved_downloads: Option<PathBuf> = match &candidate.downloads {
        Some(downloads) => match take_removal_right(&candidate.root, &candidate.tool, &candidate.version) {
            Err(error) => return Removal::Kept(error.message),
            // The directory went between the scan and now: there is nothing to move aside.
            Ok(RemovalRight::Absent) => None,
            Ok(RemovalRight::Taken) => {
                return Removal::Kept("another command is removing its archives right now".to_string());
            }
            Ok(RemovalRight::Held { claim }) => {
                if live_owner(downloads).is_some() {
                    return Removal::Kept("an install owns its downloads and is unpacking them".to_string());
                }
                downloads_right = Some(claim);
                let marker: PathBuf = tombstone_of(downloads);
                match fs::rename(downloads, &marker) {
                    Ok(()) => Some(marker),
                    Err(error) if error.kind() == ErrorKind::NotFound => None,
                    Err(error) => return Removal::Kept(format!("its downloads cannot be moved aside: {error}")),
                }
            }
        },
        None => None,
    };
    // What: a deadline that passed after the downloads moved puts them back and leaves the version whole.
    // Why: the version's own name is where a reader expects it, and a downloads marker without a version beside it is
    // work the next pass would have to guess about.
    if out_of_time() {
        restore_downloads(&moved_downloads, &candidate.downloads);
        return Removal::Deferred;
    }
    // What: the version is taken out of its name before anything is compared or deleted,
    // to a hidden name no scan judges and no recovery deletes.
    // Why: the move is one atomic step, so whatever another run put at that name is in hand and can be told from the
    // version that was judged; a check and then a move would delete whatever the second run published in between.
    match fs::rename(&candidate.directory, &claimed) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {
            restore_downloads(&moved_downloads, &candidate.downloads);
            return Removal::Gone;
        }
        Err(error) => {
            restore_downloads(&moved_downloads, &candidate.downloads);
            return Removal::Kept(format!("it cannot be moved aside: {error}"));
        }
    }
    if !judged.still_at(&claimed) {
        return Removal::Kept(put_back(&candidate.directory, &claimed, &moved_downloads, &candidate.downloads));
    }
    // What: the deadline is looked at once more before the version takes its removal name.
    // Why: from there on the delete runs to its end, and a pass that would cut it short had better stop before it.
    if out_of_time() {
        return Removal::Kept(put_back(&candidate.directory, &claimed, &moved_downloads, &candidate.downloads));
    }
    let version_marker: PathBuf = tombstone_of(&candidate.directory);
    if let Err(error) = fs::rename(&claimed, &version_marker) {
        // The version stays, so it goes back to its name with the downloads that were moved aside for it.
        let reason: String = put_back(&candidate.directory, &claimed, &moved_downloads, &candidate.downloads);
        return Removal::Kept(format!("it cannot be moved aside: {error}; {reason}"));
    }
    for marker in [Some(version_marker), moved_downloads].into_iter().flatten() {
        match fs::remove_dir_all(&marker) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Removal::Kept(format!("{} is left half removed: {error}", marker.display())),
        }
    }
    // The claim goes before the tool's downloads directory is looked at, which is not empty while it is there.
    drop(downloads_right);
    if let Some(downloads) = &candidate.downloads
        && let Some(tool_downloads) = downloads.parent()
    {
        // Best effort: a tool's downloads directory goes once its last version's archives are gone.
        let _ = fs::remove_dir(tool_downloads);
    }
    return Removal::Removed(candidate.bytes);
}

/// What:
/// One version directory as it was judged:
/// when it was written,
/// and on Unix the file system and inode that name the entry.
/// Why:
/// A version another run published over the judged one is a different directory under the same name,
/// which only those two numbers tell from the one the scan measured,
/// and a write into the judged one in place changes its write time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct JudgedVersion {
    /// When the version directory was last written.
    written: Option<SystemTime>,
    /// The file system and inode naming the entry,
    /// on Unix.
    #[cfg(unix)]
    naming: (u64, u64),
}

/// The methods of `JudgedVersion`.
impl JudgedVersion {
    /// The entry `metadata` describes.
    fn of(metadata: &fs::Metadata) -> JudgedVersion {
        /// Imports `MetadataExt` from `std::os::unix::fs`,
        /// for the two numbers that name an entry.
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        return JudgedVersion {
            written: metadata.modified().ok(),
            #[cfg(unix)]
            naming: (metadata.dev(), metadata.ino()),
        };
    }

    /// Whether `path` holds this entry now.
    fn still_at(self, path: &Path) -> bool {
        return match fs::symlink_metadata(path) {
            Ok(metadata) => JudgedVersion::of(&metadata) == self,
            Err(_) => false,
        };
    }
}

/// The first field of the hidden name a removal's one atomic move takes a version to.
const CLAIM_PREFIX: &str = ".mpm-shadow-claimed.";

/// The last field of that name,
/// which is what tells it from the names of every other hidden entry.
const CLAIM_SUFFIX: &str = ".claimed";

/// How many names of one process this removal tries beside a version before it keeps the version instead.
/// Why:
/// A name is taken only by a claim the same process instance left for the same version,
/// which a run that was killed and its own retry can each produce once;
/// a store holding more than this many of them is one a person has to look at.
const CLAIM_ATTEMPTS: u32 = 32;

/// What:
/// The name a removal gives the version directory it has taken out of the version's name:
/// the version it holds,
/// the process that took it,
/// and which of that process's claims it is.
/// Why:
/// The name is written by the removal and read back by the recovery pass,
/// so both ask this one definition and cannot disagree about which field holds what;
/// a version that holds dots,
/// dashes,
/// plus signs or the word `claimed` is written between the prefix and the
/// process's own three fields and read back as everything before them,
/// so the encoding is exact for every version name the store can hold.
/// The process's number alone would name a process only until the table reuses it,
/// so the start time the kernel gives it is written beside the number:
/// a reused number's process started later,
/// and the claim's maker is then known to be gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimedVersion = { version: string; maker: ProcessIdentity; attempt: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimedVersion {
    /// The version the claim holds,
    /// as the version directory was named.
    pub(crate) version: String,
    /// The process that claimed it.
    pub(crate) maker: ProcessIdentity,
    /// Which claim of that process it is,
    /// so two claims of one process can sit side by side.
    pub(crate) attempt: u32,
}

/// The methods of `ClaimedVersion`.
impl ClaimedVersion {
    /// The name this claim is written under.
    pub(crate) fn to_name(&self) -> String {
        return format!(
            "{CLAIM_PREFIX}{}.{}.{}.{}{CLAIM_SUFFIX}",
            self.version, self.maker.pid, self.maker.started, self.attempt
        );
    }

    /// The claim a name stands for,
    /// or `None` for a name this build did not write.
    pub(crate) fn parse(name: &str) -> Option<ClaimedVersion> {
        let rest: &str = name.strip_prefix(CLAIM_PREFIX)?.strip_suffix(CLAIM_SUFFIX)?;
        // The three fields this build writes are the last three, and the version is everything before them.
        let mut fields = rest.rsplitn(4, '.');
        let attempt: u32 = fields.next()?.parse().ok()?;
        let started: u64 = fields.next()?.parse().ok()?;
        let pid: u32 = fields.next()?.parse().ok()?;
        let version: String = fields.next()?.to_string();
        // A version names one directory: anything else would put the tree where it was never installed.
        if version.is_empty() || version.starts_with('.') || version.contains('/') {
            return None;
        }
        return Some(ClaimedVersion { version, maker: ProcessIdentity { pid, started }, attempt });
    }
}

/// The identity this process writes its claims under:
/// its number and the moment it started where the process table says,
/// and its number alone where that cannot be read.
fn claim_maker() -> ProcessIdentity {
    return this_process().unwrap_or(ProcessIdentity { pid: std::process::id(), started: 0 });
}

/// What:
/// A free hidden name beside `version_directory`,
/// where the one atomic move of a removal takes it,
/// or `None` when every name this process would write is taken.
/// Why:
/// No scan lists a hidden name as a version,
/// and the recovery pass finishes only the names it recognizes as a removal in progress,
/// so a copy whose identity has not been compared yet is never a name that pass deletes.
/// The maker's number and start time in the name are what a later pass reads to tell a claim a live run holds from
/// one a kill left,
/// and a made-up name is never a second shape a reader has to guess about.
fn claim_name_beside(version_directory: &Path) -> Option<PathBuf> {
    let version: String = file_name_text(version_directory);
    let maker: ProcessIdentity = claim_maker();
    for attempt in 0..CLAIM_ATTEMPTS {
        let candidate: PathBuf = version_directory
            .with_file_name(ClaimedVersion { version: version.clone(), maker, attempt }.to_name());
        if fs::symlink_metadata(&candidate).is_err() {
            return Some(candidate);
        }
    }
    return None;
}

/// Puts the downloads moved aside for a version back under their name,
/// as the removal does when the version stays.
fn restore_downloads(moved_downloads: &Option<PathBuf>, downloads: &Option<PathBuf>) {
    if let Some(marker) = moved_downloads
        && let Some(downloads_directory) = downloads
    {
        // Best effort: the version is staying, and a download this run cannot put back is a name the next pass reads.
        let _ = fs::rename(marker, downloads_directory);
    }
}

/// What:
/// Puts back the version that was taken out of its name,
/// with the downloads moved aside for it,
/// and says why it stayed.
/// Why:
/// What is in hand is not the version this run judged but the one another run published at its name,
/// so it goes back exactly where it was;
/// a put that cannot happen keeps it at the hidden name it was moved to,
/// and the line has to say that name and the command that puts it back,
/// because a kept version nobody can find is a lost version to the run that installed it.
fn put_back(version_directory: &Path, claimed: &Path, moved_downloads: &Option<PathBuf>, downloads: &Option<PathBuf>) -> String {
    let restored: std::io::Result<()> = fs::rename(claimed, version_directory);
    restore_downloads(moved_downloads, downloads);
    return match restored {
        Ok(()) => "another run wrote it while the cleanup ran".to_string(),
        Err(error) => format!(
            "another run wrote it while the cleanup ran, and what it wrote cannot be put back: {error}; it is kept at {} \
             instead of {}, and `mv -- {} {}` puts it back",
            claimed.display(),
            version_directory.display(),
            shell_word(&claimed.to_string_lossy()),
            shell_word(&version_directory.to_string_lossy())
        ),
    };
}

/// What:
/// Gives back every version a killed removal took out of its name and never judged,
/// and leaves the claims a live run holds.
/// Why:
/// The move out of the version's name is what lets the pass compare what it took with what it judged,
/// but a run killed between the two leaves the version's tree under a hidden claimed name:
/// no scan lists it as a version,
/// the tombstone pass does not know the name,
/// and without this the version is out of every tree until a person moves it back by hand.
/// A claim whose version's name is taken stays where it is:
/// the rename onto a non-empty directory fails rather than carrying away what another run published there,
/// and the line says where the tree is.
fn give_back_claimed(
    root: &Path,
    tool: &str,
    run: &TrimRun,
    lines: &mut Vec<String>,
    unfinished: &mut BTreeSet<PathBuf>,
) {
    let tool_directory: PathBuf = root.join(tool);
    let Ok(entries) = fs::read_dir(&tool_directory) else {
        return;
    };
    let mut abandoned: Vec<(ClaimedVersion, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let Some(claim) = ClaimedVersion::parse(&name) else {
            continue;
        };
        // What: only a claim whose maker has ended is given back.
        // Why: the claim of a run that is still comparing is that run's to put back or to delete,
        // and the maker is its number and start time together,
        // so a process that got a dead run's number does not hold a claim that run left.
        if process_has_ended(&claim.maker) {
            abandoned.push((claim, entry.path()));
        }
    }
    abandoned.sort_by(compare_by_version);
    for (claim, claimed) in abandoned {
        if run.started.elapsed() >= run.bounds.work_limit {
            unfinished.insert(root.to_path_buf());
            return;
        }
        match fs::rename(&claimed, tool_directory.join(&claim.version)) {
            Ok(()) => {
                lines.push(format!(
                    "mpm-shadow: gave back {} of {}, which a removal an earlier run started had taken out of its name",
                    claim.version, tool
                ));
                // What: the archives moved aside for this version come back with it.
                // Why: the removal moved the downloads before the version, so a kill between the two leaves the
                // version under its claim and the archives under their tombstone:
                // giving the version back and then letting the tombstone pass delete the archives would restore a
                // version without the files it was installed with.
                if let Err(error) = give_back_downloads(root, tool, &claim.version) {
                    lines.push(format!(
                        "mpm-shadow: gave back {} of {} without its archives: {error}",
                        claim.version, tool
                    ));
                }
            }
            // Another pass gave this one back a moment before this one reached it:
            // the version is where it belongs, and nothing here failed.
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                unfinished.insert(root.to_path_buf());
                lines.push(format!(
                    "mpm-shadow: kept {}: a removal an earlier run started took it out of its name and it cannot be given back: {error}",
                    claimed.display()
                ));
            }
        }
    }
}

/// Orders two abandoned claims by the version they hold,
/// so the pass gives them back in one order every time.
fn compare_by_version(left: &(ClaimedVersion, PathBuf), right: &(ClaimedVersion, PathBuf)) -> std::cmp::Ordering {
    return left.0.version.cmp(&right.0.version);
}

/// What:
/// The names of the versions a claim under any of `tools` holds in this root.
/// Why:
/// The tombstone pass has to know which archives a claim is still using or is being given back with,
/// and a claim's name is the only record of the version it holds.
fn claimed_versions(root: &Path, tools: &BTreeSet<String>) -> BTreeSet<String> {
    let mut versions: BTreeSet<String> = BTreeSet::new();
    for tool in tools {
        let Ok(entries) = fs::read_dir(root.join(tool)) else {
            continue;
        };
        for entry in entries.flatten() {
            let name: String = entry.file_name().to_string_lossy().to_string();
            if let Some(claim) = ClaimedVersion::parse(&name) {
                versions.insert(claim.version);
            }
        }
    }
    return versions;
}

/// What:
/// The version a downloads tombstone was made for,
/// or `None` for a tombstone of a version directory.
/// Why:
/// A downloads tombstone sits in `<root>/downloads/<tool>/` and a version's in `<root>/<tool>/`,
/// and only the first is tied to a claim under the tool directory: the second is a version the store has already
/// taken out of its name and nothing else may use.
fn downloads_tombstone(root: &Path, marker: &Path) -> Option<(String, String)> {
    // A downloads tombstone sits under `<root>/downloads/<tool>/`, and a version's under `<root>/<tool>/`.
    let downloads_directory: PathBuf = root.join(DOWNLOADS_DIRECTORY_NAME);
    let inside: &Path = marker.strip_prefix(&downloads_directory).ok()?;
    let tool: String = inside.parent()?.to_string_lossy().to_string();
    if tool.is_empty() {
        return None;
    }
    let name: String = file_name_text(marker);
    let version: &str = name.strip_prefix(TOMBSTONE_PREFIX)?;
    if version.is_empty() {
        return None;
    }
    return Some((tool, version.to_string()));
}

/// What:
/// Renames the downloads tombstone of `version` back to the directory it was made from,
/// when one is there.
/// Why:
/// The archives were moved aside before the version, so a removal that is given back needs them back too,
/// and the name it was moved to is the only record of where they are.
fn give_back_downloads(root: &Path, tool: &str, version: &str) -> Result<(), String> {
    let downloads: PathBuf = downloads_of(root, tool, version);
    let marker: PathBuf = tombstone_of(&downloads);
    if fs::symlink_metadata(&marker).is_err() {
        // Nothing was moved aside for this version, or another pass took it: either way there is nothing to give back.
        return Ok(());
    }
    return match fs::rename(&marker, &downloads) {
        Ok(()) => Ok(()),
        // The archives are back under their name already, which is where this call wanted them.
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{} cannot be renamed back: {error}", marker.display())),
    };
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
        for tool in tools {
            give_back_claimed(root, tool, run, &mut lines, &mut unfinished);
        }
        let claimed_here: BTreeSet<String> = claimed_versions(root, tools);
        for marker in tombstones(root, tools) {
            if run.started.elapsed() >= run.bounds.work_limit {
                unfinished.insert(root.clone());
                break;
            }
            // What: a downloads tombstone a removal still holds is left where it is.
            // Why: that removal moved the archives aside and has not judged the version yet,
            // or this pass gave the version back above and restored the archives with it;
            // deleting the tombstone would take the archives from that removal or from the version just restored,
            // and a claim whose maker has ended does not hold it.
            if let Some((tool, version)) = downloads_tombstone(root, &marker)
                && (claimed_here.contains(&version) || removal_held(root, &tool, &version))
            {
                continue;
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
        // What: the marker goes only when nothing under the root is a removal in progress, alive or dead.
        // Why: a claim names work that is not done, even when this pass cannot finish it (its owner is still running),
        // and a marker cleared then is never looked at again by a sync that installs nothing,
        // so the version would stay hidden under its claimed name for good.
        if unfinished.contains(root) || !tombstones(root, tools).is_empty() || has_claimed_versions(root, tools) {
            continue;
        }
        let _ = fs::remove_file(unfinished_marker(root));
    }
}

/// What:
/// Whether any claim of a removal sits under this root,
/// whatever its maker is doing.
/// Why:
/// A claim is a version taken out of its name,
/// and until it is given back or deleted the work it belongs to is not
/// done;
/// the marker is what makes a later sync look.
fn has_claimed_versions(root: &Path, tools: &BTreeSet<String>) -> bool {
    for tool in tools {
        let Ok(entries) = fs::read_dir(root.join(tool)) else {
            continue;
        };
        for entry in entries.flatten() {
            if ClaimedVersion::parse(&entry.file_name().to_string_lossy()).is_some() {
                return true;
            }
        }
    }
    return false;
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
pub(super) fn visible_locks(ctx: &Context) -> (Vec<PathBuf>, Option<String>) {
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rootsToTrim(ctx: Context): string[]
/// ```
pub(super) fn roots_to_trim(ctx: &Context) -> Vec<PathBuf> {
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
        // What: the archives of versions this root no longer has installed go, once nothing has touched them for the
        // in-use floor.
        // Why: a removal keeps an archive an install is still unpacking, and a run killed between removing a version
        // and removing its archives leaves them; nothing else reads a downloads directory whose version is gone.
        for root in &roots {
            lines.extend(remove_orphan_downloads(root, &tools, run.started + bounds.work_limit));
        }
        let holders: StoreHolders = holders_of(&roots, run.started + bounds.work_limit);
        let installed_set: BTreeSet<(String, String)> = installed.iter().cloned().collect();
        let links: BTreeMap<(PathBuf, String), String> = current_links(&roots, &tools);
        let (candidates, inventory_incomplete): (Vec<TrimCandidate>, Vec<String>) =
            candidates_of(&roots, &tools, run.started + bounds.work_limit);
        // What: an inventory that ran out of its time is said, and this pass judges only what it reached.
        // Why: the versions it did not reach are not in the plan, so nothing is removed on a guess about them,
        // and a person reading the run knows why the pass was short.
        for reason in &inventory_incomplete {
            lines.push(format!("mpm-shadow: the cleanup's inventory is not the whole store: {reason}"));
        }
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
    // What: a pass that could not read the locks keeps every root's marker, because it could look at nothing.
    // Why: with no tool directory named, the claim and tombstone scans cover nothing,
    // so "nothing is left" would be an answer about nothing;
    // a marker deleted then is never looked at again by a sync that installs nothing,
    // and the claimed version stays hidden for good.
    if named.unreadable.is_some() {
        unfinished.extend(roots.iter().cloned());
    }
    settle_markers(&roots, &tools, &unfinished);
    for line in lines {
        eprintln!("{line}");
    }
}
