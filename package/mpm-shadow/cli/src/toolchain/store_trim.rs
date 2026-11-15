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
//! ## The names a version stands under, and what each may hold
//!
//! - the version's name (`<root>/<tool>/<version>`):
//!   a whole version, or a tree this module could not prove whole and put back unfinished (see the next section),
//!   which no install and no `current` choice reads as a finished copy.
//!   A deletion that has begun never comes back under it.
//! - a removal name (`<root>/<tool>/.mpm-shadow-removing.<version>`):
//!   a version a pass judged, moved aside, and recorded beside itself (`<removal name>.facts`: device, inode, file
//!   count, bytes, newest write inside, root write time).
//!   Every pass puts such a tree back under its version's name; what the record says decides only whether it goes back
//!   as the finished copy it was, or as a copy nothing may take for finished.
//! - a deletion name (`<root>/<tool>/.mpm-shadow-deleting.<version>`, and `.mpm-shadow-deleting..mpm-shadow-removing.
//!   <version>` for a tree whose deletion began from its removal name, which is how every removal this module deletes
//!   ends):
//!   a version whose deletion has begun, and which is finished, never restored.
//!   Only the deletion step writes this name, and it writes it before unlinking the first file, so whatever stands
//!   under it is condemned and may be partly gone.
//!   The record beside it names the version, so the pass finds it from either name.
//! - a downloads removal name and a downloads deletion name, for the archives, under `downloads/<tool>/`.
//! - a download in progress (`downloads/<tool>/<version>/<file>.download` and the file beside it):
//!   written by an install, never by this module.
//! - the store-root marker (`.mpm-shadow-removal-unfinished`):
//!   a hint that a pass left work; it never decides alone whether a pass runs, because two passes settling it at once
//!   can clear it after one has written it.
//!
//! ## What protects a version, what the pass reads to know it, and what a read that fails means
//!
//! A version stays when anything that can name, run or need it does.
//! Each of those readings can be impossible or partial, and the meaning is the same in every such case: the version
//! is protected.
//!
//! - a lock:
//!   the global scope's lock and the lock of every worktree of the repository that runs the pass, whatever scope asked.
//!   The global lock is found from `HOME` and `XDG_CONFIG_HOME`;
//!   the worktree locks from `git worktree list` of the repository the pass runs in, which a global pass names too
//!   (`MONOCHROMATIC_WORKTREE_ROOT`, the same variable the repository root is found from).
//!   A lock that cannot be read, a row whose version or whose route this release cannot read, a worktree list that
//!   fails, and a pass that can name no repository at all all mean "a lock that may name any version": every version
//!   stays, and the line says which read failed.
//!   A second clone of the repository, at another path, is named by no listing this program can make; this machine
//!   holds one repository and no clone (`/var/mnt/encrypted/repository` holds one, measured 2026-11-15).
//! - the tool's `current` link:
//!   read once before the plan, again at the instant of the removal, and again after the move.
//!   A link that cannot be read is a link that may name the version, so the version stays.
//! - a process:
//!   the process table, once per pass, through `exe`, `cwd`, `fd` and `maps` of every process of this user.
//!   A process the kernel refuses to describe may run any version, so any such process makes the scan incomplete and
//!   every version stays, with one line naming the processes.
//!   Measured on this machine at one moment: twelve processes of this user cannot be described (`ssh-agent`,
//!   `keepassxc`, `systemd`, `kwin_wayland` and the like), so a pass here keeps everything and removes nothing.
//! - the age:
//!   the newest write anywhere inside the version's tree, never the root's own write time alone, because a write into
//!   a folder that already exists leaves the root's time alone.
//!   The size record beside the store is a read of a moment, so it may prove a tree young and never old: a version
//!   whose recorded facts would put it outside the age floor is walked again before it is judged.
//!
//! ## The steps that move or delete a tree, and what each proves when it acts
//!
//! ## The one place that renames or unlinks anything
//!
//! Everything this module does to the drive goes through `settle`, and nothing else in it renames, moves back or
//! unlinks. It takes the exact path, the facts that path was judged by, the scan of the process table, and what the
//! caller wants done, and it reads the path again at the instant it acts:
//!
//! - a scan that could not be read in full answers kept, with the processes that could not be described;
//! - a process that holds that exact path, or anything below it, answers kept, with the process;
//! - a path it cannot read whole now answers kept: a failed read is a "cannot tell", never a "no";
//! - a path whose facts are not the facts it was judged by answers kept: a copy that replaced the judged one is a
//!   different object;
//! - only then does it rename (proving the entry under the new name is the same one and moving it back when it is not)
//!   or delete (renaming the tree to its deletion name before unlinking, so a kill inside leaves a name the next run
//!   finishes).
//!
//! A version and its archives are two objects, judged separately and settled separately; a flat leftover file is a third,
//! asked about by its own path. A recovery is a removal like any other: a tree a killed pass left is judged from
//! nothing, by what is on the drive now, never by what the dead pass had decided.
//!
//! ## Every read that can fail, and what its failure means
//!
//! - `judged_of_tree`: a path that cannot be read, or a tree the walk could not read in full, is nothing this pass
//!   judges, so nothing happens to it.
//! - `read_judged`: no record, or one this program did not write, proves nothing about the tree, so the tree is put
//!   back unfinished and never deleted.
//! - `provably_absent`: only `NotFound` says a path is absent; a permission or I/O failure answers kept.
//! - `current_names` / `current_links`: a failed read is "may name it", and the pass keeps every version.
//! - `judged_entry_difference`: a write time that cannot be read is not the judged entry, so the version is kept.
//! - `has_unfinished_removal` and the store scan: a marker that cannot be read is "no marker", which may make the next
//!   install skip the pass and never makes it delete anything; the store scan itself is what brings the pass back.
//! - `version_directories` / `removal_markers`: a tool directory or a marker that cannot be listed is not a version
//!   or not a marker, so the pass judges less and never removes more.
//! - `tree_facts`: a directory or an entry that cannot be read makes the walk partial, and a partial tree is kept.
//! - `holders_of`: a process the kernel will not describe makes the scan incomplete, and every version is kept.
//!
//! ## What bounds each call that can block
//!
//! - the inventory, the walks and the deletions stop at the work limit's deadline (a check every 128 entries);
//! - the process scan stops at the deadline (a check every 64 processes);
//! - `git worktree list` is started with the same deadline, and the real runner ends and reaps it there;
//! - a single file-system call (a rename, an unlink, a `stat`) cannot be bounded by this program, and a stalled drive
//!   holds the pass inside one;
//! - the lock files are read with plain reads and are small.
//!
//! ## What a kill leaves, and what the next run does
//!
//! - between the two renames of a removal: the archives removal name and a whole version, which the finish pass puts
//!   back as archives;
//! - after the version rename, before its record is written: a removal name holding a whole tree and no record, put
//!   back unfinished (the next section says why);
//! - after the record is written: a removal name holding a whole tree and the record of it, put back finished, and the
//!   plan of the same pass judges it again;
//! - after the rename to the deletion name, or inside the deletion: a deletion name holding a whole or partial tree
//!   and no other copy, which the next pass finishes deleting;
//! - a pass killed between writing the store-root marker and its first rename: nothing but the marker, which the next
//!   pass settles;
//! - the next run does this whenever a removal or deletion name stands, whether it installed anything or not, so
//!   recovery never waits on an unrelated install.
//!
//! ## A tree under a removal name with no record, and why it goes back unfinished
//!
//! The record is the only proof of a removed tree's wholeness this module has, so a tree that has none is not proven
//! whole. It is not really open whether such a tree exists:
//!
//! - a build of this line killed between its rename and its record write leaves one, and those are two adjacent system
//!   calls;
//! - a build of an earlier commit of this line deleted a removal name's tree in place, so a kill inside that deletion
//!   left a partly deleted tree under a removal name with no record at all;
//! - no published release can leave one: the removal names appear in no release's source, and this machine installs
//!   exactly one release of this tool (`~/.local/share/monochromatic-dev/toolchain/mpm-shadow/0.2.7-alpha.11`, whose
//!   source has no cleanup module), so no store on this machine holds such a tree today;
//! - a person or another program that renames a directory can make one.
//!
//! Leaving it where it is costs the version its name, makes the next install fetch it again, and leaves the tree under
//! the removal name as nobody's, with one line printed on every run: a leak, and a jam that never ends.
//! Putting it back as the finished copy it was costs the guarantee above: a tree a deletion has begun inside keeps its
//! executable and its completion marker, so `is_finished` would accept it and a later observer would take a broken copy
//! for a working one (the sixth review's first finding).
//! The evidence that could settle it otherwise: a lock names a version and its artifact with that artifact's SHA-256,
//! and says nothing about the tree's contents; the archive a version was unpacked from says what it holds, member by
//! member, but a `cargo binstall` version has no such archive, an install may have dropped it, and reading it needs a
//! decoder for every archive format, so it could prove some trees and never all of them.
//! What the record does prove is narrower and enough: which tree it was, and whether that tree changed.
//!
//! So the tree goes back under its version's name, and everything that would let it be read as a finished install is
//! taken away first (the completion marker, and a `cargo binstall` version's own record).
//! Then nothing hands it out as the right tool: `is_finished` is false in every release, so it is never the newest, never
//! why `current` moves, never why an older working version is deleted, and the next install of that version replaces it.
//! Nothing is deleted and nothing waits: the same pass that finds it resolves it, and the pass that follows judges the
//! tree as a version, by the plan, with the locks in hand.
//! A launcher follows the tool's `current` link without asking whether the version is finished, so when that link names
//! such a copy the pass takes the link away: while the copy was aside the link named nothing, and the launcher already
//! reports a missing program with the remedy that puts it back; leaving the link would turn that honest absence into a
//! run of a copy nothing proved whole. Nothing is lost by it, and the next install of that version points the link
//! again. Whether the launcher itself should refuse a version that did not finish, so the link could stay, is a
//! question for the owner.
//!
//! ## The work limit
//!
//! The limit bounds the work itself: the inventory, every deletion and every size a line prints stop at a deadline and
//! leave what they did not reach to the next run, which finds it under the same names. A tree the inventory could not
//! read whole is kept, never removed, and the pass says so. A single file-system call that never returns cannot be
//! bounded by this program, and the report says so.
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
/// Imports `Ceiling` from `super::runner`,
/// the bound a listing the pass starts is given.
use super::runner::Ceiling;
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `Route` from `super::model`.
use super::model::Route;
/// Imports `crate_of_entry` from `super::prune_cargo`,
/// which reads a lock row's crate name the way the prune of unnamed cargo versions does.
use super::prune_cargo::crate_of_entry;
/// Imports `ScopeKind` and `global_scope` from `super::scope`.
use super::scope::{EnvMap, ScopeKind, global_scope, named_repository_root};
/// Imports what this module uses from `super::store_path_holders`.
use super::store_path_holders::{StoreHolders, held_reason, holders_of};
/// Imports what this module uses from `super::store_versions`.
use super::store_versions::{
    JudgedTree, RemovalMarker, TreeFacts, VersionDirectory, deleting_of, downloads_of,
    IN_FLIGHT_DOWNLOAD_SUFFIX, install_directory_name, judged_of_tree, leftover_downloads, newest_write, read_judged,
    record_of, record_of_marker,
    removal_markers, tombstone_of,
    version_directories, write_judged,
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
    /// What the archives were judged to be, when they could be read whole.
    pub downloads_facts: Option<JudgedTree>,
    /// The space removing it gives back, in bytes.
    pub bytes: u64,
    /// The space the version's own tree weighs, in bytes, without the archives kept for it.
    pub tree_bytes: u64,
    /// When it was last written.
    pub written: SystemTime,
    /// The device and inode number of the version directory when it was judged,
    /// when the platform reports them.
    pub inode: Option<(u64, u64)>,
    /// How many files the tree held when it was judged.
    pub files: u64,
    /// The newest write anywhere inside it when it was judged.
    pub newest: SystemTime,
    /// Whether this is a download file and not a version directory.
    pub leftover: bool,
}

/// What:
/// Everything one pass may remove, in the order it is judged:
/// oldest newest write first.
/// Why:
/// The rule removes the versions written longest ago first,
/// so the order is the rule's own and is built here once.
/// The age floor is handed in because a version whose recorded facts are older than the floor must be walked here:
/// a record cannot see a write inside the tree, so it may prove a version young and never old.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function candidatesOf(roots: string[], tools: ReadonlySet<string>, ageFloor: Duration): TrimCandidate[]
/// ```
pub fn candidates_of(
    roots: &[PathBuf],
    tools: &BTreeSet<String>,
    deadline: Instant,
    age_floor: Duration,
) -> Vec<TrimCandidate> {
    let mut candidates: Vec<TrimCandidate> = Vec::new();
    for root in roots {
        for version in version_directories(root, tools, deadline, age_floor) {
            candidates.push(candidate_of_version(root, version));
        }
        for leftover in leftover_downloads(root, tools) {
            candidates.push(TrimCandidate {
                root: root.clone(),
                tool: leftover.tool,
                version: String::new(),
                directory: leftover.file,
                downloads: None,
                downloads_facts: None,
                bytes: leftover.bytes,
                tree_bytes: leftover.bytes,
                written: leftover.written,
                inode: leftover.inode,
                files: 1,
                newest: leftover.written,
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
        downloads_facts: version.downloads_facts,
        bytes: version.bytes,
        tree_bytes: version.tree_bytes,
        written: version.written,
        inode: version.inode,
        files: version.files,
        newest: version.newest,
        leftover: false,
    };
}

/// Orders two candidates oldest write first,
/// by path when they were written in the same second.
fn compare_oldest_first(left: &TrimCandidate, right: &TrimCandidate) -> std::cmp::Ordering {
    let by_write: std::cmp::Ordering =
        newest_write(left.written, left.newest).cmp(&newest_write(right.written, right.newest));
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
                // What: a route this release cannot read makes the lock unreadable.
                // Why: such a row names a version this pass cannot place, so it may name any version, and treating it
                // as pinning nothing deleted a version a lock still named (a route a newer release writes).
                return Err(format!(
                    "{} names a tool with a route this release cannot read (tools.{id}), so no version can be told from \
                     one a lock pins",
                    origin.display()
                ));
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
        // What: the age is the age of the newest write anywhere inside the tree, read from the one place that says
        // when a tree was written.
        // Why: a write into a folder that already existed leaves the version root's own write time alone, so a tree
        // whose root looks old can hold a file written a moment ago.
        let newest: SystemTime = newest_write(candidate.written, candidate.newest);
        let age: Duration = now.duration_since(newest).unwrap_or(Duration::ZERO);
        if age < bounds.age_floor {
            steps.push(TrimStep::Keep(
                candidate.clone(),
                format!(
                    "it was written {} ago, inside the {} floor",
                    format_age(now, newest),
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
    /// What a scan of the process table found, when one was made already.
    /// A control hands in the holders it built,
    /// so it does not depend on which processes the machine happens to be running;
    /// a real pass leaves this `None` and reads the table itself.
    pub holders: Option<StoreHolders>,
}

/// The methods of `TrimRun`.
impl TrimRun {
    /// The moment this pass's work limit runs out.
    pub fn deadline(&self) -> Instant {
        // What: a limit that does not fit after the start is read as the longest one the pass allows.
        // Why: `Instant + Duration` panics on overflow, and a pass's bound must never end an install.
        return self.started.checked_add(self.bounds.work_limit).unwrap_or(self.started + MAX_WORK_LIMIT);
    }

    /// What the process table says about the roots this pass trims,
    /// or what the caller already read.
    pub fn look(&self) -> StoreHolders {
        return match &self.holders {
            Some(found) => found.clone(),
            None => holders_of(&self.roots, self.deadline()),
        };
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

/// What:
/// The two trees one version of a tool owns in a store: its directory and the archives downloaded for it.
/// Why:
/// A hold can name either, and the checks that keep a version a live process uses are only as good as the list they
/// consult. A tree a removal moved aside is asked about where it stands, by the one function that acts on it.
/// The two trees are the whole of what an install writes for a version:
/// `install_archive` downloads into `<root>/downloads/<tool>/<version>` and unpacks into
/// `<root>/<tool>/<version>`, `install_cargo` and `install_rust_android` write only under the version directory,
/// and a `cargo binstall` crate keeps its record inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownedPaths(root: string, tool: string, version: string): string[]
/// ```
fn owned_paths(root: &Path, tool: &str, version: &str) -> Vec<PathBuf> {
    return vec![root.join(tool).join(version), downloads_of(root, tool, version)];
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
fn owned_hold_reason(holders: &StoreHolders, root: &Path, tool: &str, version: &str) -> Option<String> {
    for path in owned_paths(root, tool, version) {
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
/// What the inventory judged one candidate to be, as the facts a removal proves before it acts.
/// Why:
/// The facts a decision was made on are the inventory's, not a read taken after the decision:
/// a copy that replaced the judged one between the two is a different object, and `settle` is asked to prove the
/// judged one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgedOfCandidate(candidate: TrimCandidate): TreeJudgement
/// ```
fn judged_of_candidate(candidate: &TrimCandidate) -> JudgedTree {
    return JudgedTree {
        inode: candidate.inode,
        written: candidate.written,
        facts: TreeFacts { bytes: candidate.tree_bytes, files: candidate.files, newest: candidate.newest, complete: true },
    };
}

/// How many entries the deletion walk reads between two reads of the clock.
const DEADLINE_CHECK_EVERY: u64 = 128;

/// The file an archive install writes last,
/// whose presence is what every release reads as "this copy finished".
const COMPLETE_MARKER: &str = super::archive_staging::COMPLETE_MARKER;

/// The record a `cargo binstall` crate writes last inside its version directory,
/// which is what makes that route's copy a finished one.
const CARGO_RECORD_NAME: &str = ".crates.toml";

/// The prefix of a cargo route's tool directory name (`cargo/below`).
const CARGO_TOOL_PREFIX: &str = "cargo/";

/// The refusal for an entry that is not the one a pass judged.
const NOT_THE_JUDGED_ENTRY: &str =
    "another run installed this version while the cleanup ran, so the entry found there is not the one judged";

/// A version a removal moved out of its name, with what was judged of it and of the archives kept for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MovedAside {
    /// The tree, under its removal name.
    pub version_marker: PathBuf,
    /// What the tree was judged to be, read at the moment it was moved.
    pub version_judged: JudgedTree,
    /// The archives, under their removal name, when the version had any.
    pub downloads_marker: Option<PathBuf>,
    /// What the archives were judged to be.
    pub downloads_judged: Option<JudgedTree>,
}

/// What a pass wants done with one tree or file it has judged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Settle {
    /// Move it out of its live name.
    Aside,
    /// Move it back under its live name.
    Back,
    /// Delete it for good.
    Delete,
}

/// What settling a tree or a file ended in.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Settled {
    /// It is where the action puts it.
    Done,
    /// Nothing happened to it, and why.
    Kept(String),
}

/// One tree or file to act on, with what the action has to prove before it acts.
struct ToSettle<'a> {
    /// The exact path the action is about, as it stands now.
    path: &'a Path,
    /// Where the action moves it: its removal name, or its live name.
    target: &'a Path,
    /// What it was judged to be.
    judged: &'a JudgedTree,
    /// What the process table said for this pass.
    holders: &'a StoreHolders,
    /// What to do.
    action: Settle,
    /// When the work limit runs out.
    deadline: Instant,
}

/// What:
/// The one place this module renames, moves back or unlinks a tree or a file.
/// Why:
/// A pass looks at a tree, decides, and acts later, and other commands and people act in between.
/// A deletion that trusted a name, a proof about a different object, and a read that failed each cost a review a copy,
/// so everything this module does to the drive goes through this function:
/// it reads the exact path again at the moment it acts, asks the process table about that exact path, and answers
/// "kept" for anything it cannot show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settle(tree: { path: string; target: string; judged: TreeJudgement; holders: StoreHolders;
///   action: "aside" | "back" | "delete" }): { done: true } | { kept: string }
/// ```
fn settle(tree: &ToSettle<'_>) -> Settled {
    // What: a scan that could not be completed, or a process that holds this exact path, keeps it.
    // Why: a process the kernel will not describe may run any version, and one that holds the path is using it.
    if let Some(reason) = &tree.holders.incomplete {
        return Settled::Kept(format!("the process table could not be read in full: {reason}"));
    }
    if let Some(reason) = held_reason(tree.holders, tree.path) {
        return Settled::Kept(format!("it is in use: {reason}"));
    }
    // What: the path is read again now, and must be the tree and the bytes that were judged.
    // Why: a rename or an unlink acts on whatever stands at the path at that instant.
    let Some(now) = judged_of_tree(tree.path, tree.deadline) else {
        return Settled::Kept("it could not be read in full at this moment".to_string());
    };
    if !tree.judged.matches(&now) {
        return Settled::Kept(if tree.judged.inode != now.inode {
            NOT_THE_JUDGED_ENTRY.to_string()
        } else {
            "something was written in it after it was judged".to_string()
        });
    }
    return match tree.action {
        Settle::Delete => match delete_tree_within(tree.path, tree.deadline) {
            None => Settled::Done,
            Some(failure) => Settled::Kept(failure),
        },
        Settle::Aside | Settle::Back => move_to(tree.path, tree.target, &now, tree.deadline),
    };
}

/// What:
/// Moves the tree at `path` to `target`, and proves the entry under the new name is the one that was judged.
/// Why:
/// A rename is one step on one filesystem, and saying so is what keeps a proof about one object from being read as a
/// proof about another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveTo(path: string, target: string, judged: TreeJudgement): { done: true } | { kept: string }
/// ```
fn move_to(path: &Path, target: &Path, judged: &JudgedTree, deadline: Instant) -> Settled {
    if let Err(error) = fs::rename(path, target) {
        return Settled::Kept(format!("{} could not be moved to {}: {error}", path.display(), target.display()));
    }
    return match judged_of_tree(target, deadline) {
        Some(moved) if judged.matches(&moved) => Settled::Done,
        Some(_) => {
            let _ = fs::rename(target, path);
            Settled::Kept(format!(
                "the entry moved to {} is not the tree that was judged, so it went back",
                target.display()
            ))
        }
        None => Settled::Kept(format!("{} could not be read after the move, so it is left there", target.display())),
    };
}

/// What:
/// Whether `path` is provably not there.
/// Why:
/// Only `NotFound` says a path is absent; a permission or I/O failure says nothing about it, and reading that as
/// "absent" deleted the archives of a version that was there all along.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provablyAbsent(path: string): boolean  // throws when it cannot tell
/// ```
fn provably_absent(path: &Path) -> Result<bool, String> {
    return match fs::symlink_metadata(path) {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!("{} cannot be read: {error}", path.display())),
    };
}

/// What:
/// Whether the copy of `tool`'s version at `directory` is a finished install.
/// Why:
/// A route that installs in place leaves a half-written directory under the version's name, and a finished copy at the
/// name is the only proof that a copy a removal holds may be deleted as a duplicate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishedCopyAt(tool: string, directory: string): boolean
/// ```
fn finished_copy_at(tool: &str, directory: &Path) -> bool {
    if has_complete_marker(directory) {
        return true;
    }
    if let Some(crate_name) = tool.strip_prefix(CARGO_TOOL_PREFIX) {
        return recorded_version(directory, crate_name).is_some();
    }
    return false;
}

/// What:
/// Takes away what would make the version at `tree` count as a finished install of `tool`.
/// Why:
/// A tree no record proved whole may be one a deletion has begun inside, and the completion marker it still carries
/// says otherwise to every later reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unfinishTree(tool: string, tree: string): string | undefined  // the failure, if any
/// ```
fn unfinish_tree(tool: &str, tree: &Path) -> Option<String> {
    let mut paths: Vec<PathBuf> = vec![tree.join(COMPLETE_MARKER)];
    if tool.starts_with(CARGO_TOOL_PREFIX) {
        paths.push(tree.join(CARGO_RECORD_NAME));
    }
    for path in paths {
        if let Err(error) = fs::remove_file(&path)
            && error.kind() != ErrorKind::NotFound
        {
            return Some(format!("{} still says it is finished: {error}", path.display()));
        }
    }
    return None;
}

/// What:
/// Deletes one tree,
/// renaming it to its deletion name first so a kill inside the walk leaves something the next run finishes rather
/// than something it could put back,
/// and stopping at `deadline` with whatever is left under that name for the next run.
/// Why:
/// A recursive deletion cannot be undone once a file inside it is gone, so the name says what happened to it:
/// a removal name may be put back, a deletion name never is.
/// It is called only from `settle`, which has proved the tree is the one judged and that nobody holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deleteTreeWithin(path: string, deadline: Instant): string | undefined
/// ```
pub(crate) fn delete_tree_within(path: &Path, deadline: Instant) -> Option<String> {
    let deleting: PathBuf = deleting_of(path);
    let move_to_deleting = || -> Option<String> {
        return match fs::rename(path, &deleting) {
            Ok(()) => None,
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => Some(format!("{} could not be moved to its deletion name: {error}", path.display())),
        };
    };
    // What: a walk whose limit is already gone does not begin.
    // Why: beginning it would hold the pass for work it has no time to finish.
    if Instant::now() >= deadline {
        if let Some(failure) = move_to_deleting() {
            return Some(failure);
        }
        return Some(format!("{} is left under its deletion name for the next run", deleting.display()));
    }
    if let Some(failure) = move_to_deleting() {
        return Some(failure);
    }
    // What: the walk stops at the deadline, and what it did not reach stays under the deletion name.
    // Why: the work limit has to bound the work itself; the next run finishes it without asking what it is.
    let mut pending: Vec<PathBuf> = vec![deleting.clone()];
    let mut directories: Vec<PathBuf> = Vec::new();
    let mut seen: u64 = 0;
    while let Some(current) = pending.pop() {
        seen = seen.saturating_add(1);
        if seen % DEADLINE_CHECK_EVERY == 0 && Instant::now() >= deadline {
            return Some(format!("{} is left under its deletion name for the next run", deleting.display()));
        }
        let Ok(metadata) = fs::symlink_metadata(&current) else {
            continue;
        };
        if metadata.is_dir() {
            directories.push(current.clone());
            if let Ok(children) = fs::read_dir(&current) {
                for child in children.flatten() {
                    pending.push(child.path());
                }
            }
            continue;
        }
        if let Err(error) = fs::remove_file(&current)
            && error.kind() != ErrorKind::NotFound
        {
            return Some(format!("{} is left half removed: {error}", current.display()));
        }
    }
    for directory in directories.into_iter().rev() {
        if let Err(error) = fs::remove_dir(&directory)
            && error.kind() != ErrorKind::NotFound
        {
            return Some(format!("{} is left half removed: {error}", directory.display()));
        }
    }
    return None;
}

/// What:
/// Moves one version out of its name, its archives first, each proved and asked about as the object it is.
/// Why:
/// A kill between the two renames leaves the version whole and its archives aside, which the next pass puts back;
/// a kill inside either rename leaves a name the next pass finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveAside(candidate: TrimCandidate, holders: StoreHolders): MovedAside | undefined  // throws the reason
/// ```
pub(crate) fn move_aside(candidate: &TrimCandidate, holders: &StoreHolders, deadline: Instant) -> Result<Option<MovedAside>, String> {
    let version_judged: JudgedTree = judged_of_candidate(candidate);
    // What: the archives are an object of their own, and are shown to be the ones judged before either moves.
    // Why: a replacement tree under the downloads path is not what the plan judged, and deleting it lost a download.
    let downloads_judged: Option<JudgedTree> = match (&candidate.downloads, &candidate.downloads_facts) {
        (Some(downloads), Some(judged)) => match judged_of_tree(downloads, deadline) {
            Some(now) if judged.matches(&now) => Some(now),
            Some(_) => {
                return Err("its archives are not the ones this pass judged, so neither they nor the version are removed"
                    .to_string());
            }
            None => {
                return Err("its archives could not be read in full at this moment, so neither they nor the version are removed"
                    .to_string());
            }
        },
        (Some(_), None) => {
            return Err("its archives could not be judged, so neither they nor the version are removed".to_string());
        }
        (None, _) => None,
    };
    // What: the marker is written before the first rename.
    // Why: a pass killed inside a removal leaves it, and the next install runs the pass to finish it.
    let _ = fs::write(unfinished_marker(&candidate.root), b"");
    let mut downloads_marker: Option<PathBuf> = None;
    if let (Some(downloads), Some(judged)) = (&candidate.downloads, &downloads_judged) {
        let target: PathBuf = tombstone_of(downloads);
        match settle(&ToSettle { path: downloads, target: &target, judged, holders, action: Settle::Aside, deadline }) {
            Settled::Done => downloads_marker = Some(target),
            Settled::Kept(reason) => return Err(format!("its archives stay where they are: {reason}")),
        }
    }
    let version_marker: PathBuf = tombstone_of(&candidate.directory);
    match settle(&ToSettle { path: &candidate.directory, target: &version_marker, judged: &version_judged, holders, action: Settle::Aside, deadline }) {
        Settled::Done => {}
        Settled::Kept(reason) => {
            if let (Some(marker), Some(downloads), Some(judged)) = (&downloads_marker, &candidate.downloads, &downloads_judged) {
                let _ = settle(&ToSettle { path: marker, target: downloads, judged, holders, action: Settle::Back, deadline });
            }
            return Err(reason);
        }
    }
    // What: the record of what this removal holds is written before anything may delete inside the tree.
    // Why: a later pass puts the tree back as the finished copy it was only when this record still describes it.
    let _ = write_judged(&candidate.directory, &version_judged);
    return Ok(Some(MovedAside { version_marker, version_judged, downloads_marker, downloads_judged }));
}

/// What:
/// Removes one download file an earlier layout left.
/// Why:
/// Such a file is not a version: it is judged, proved and asked about by its own path, and unlinked in one step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeLeftover(candidate: TrimCandidate, holders: StoreHolders): boolean  // false when it was gone
/// ```
fn remove_leftover(candidate: &TrimCandidate, holders: &StoreHolders, deadline: Instant) -> Result<bool, String> {
    if let Err(error) = fs::symlink_metadata(&candidate.directory)
        && error.kind() == ErrorKind::NotFound
    {
        return Ok(false);
    }
    let judged: JudgedTree = judged_of_candidate(candidate);
    return match settle(&ToSettle { path: &candidate.directory, target: &candidate.directory, judged: &judged, holders, action: Settle::Delete, deadline }) {
        Settled::Done => Ok(true),
        Settled::Kept(reason) => Err(reason),
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
    // What: the process table as it stands before anything is moved.
    // Why: a tree a process holds is never moved aside, and a scan that could not be completed keeps every version.
    let first_look: StoreHolders = run.look();
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
                    match remove_leftover(candidate, &first_look, run.deadline()) {
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
                match move_aside(candidate, &first_look, run.deadline()) {
                    Ok(Some(aside)) => moved.push((index, candidate.clone(), aside)),
                    Ok(None) => {}
                    Err(reason) => lines[index] = Some(kept_line(candidate, &reason)),
                }
            }
        }
    }
    if !moved.is_empty() {
        // What: the process table is asked again once the whole batch is out of its names.
        // Why: nothing can start using a version after its rename, so this second look is the moment the removals
        // become final, and a tree something holds goes back under its name.
        let second_look: StoreHolders = run.look();
        for (index, candidate, aside) in moved {
            // What: the tool's current link is read again, after the move.
            // Why: nothing can start using a version after its rename, but another command can repoint the link, and a
            // version the link names at that moment is never removed.
            let mut kept: Option<String> = if current_names(&candidate.root, &candidate.tool, &candidate.version) {
                Some("the tool's current link names it".to_string())
            } else {
                None
            };
            // What: nothing is deleted when a protection already answered kept.
            // Why: a version the link names at that moment is never removed, and a deletion once done cannot be undone.
            if kept.is_none()
                && let Settled::Kept(reason) = settle(&ToSettle {
                    path: &aside.version_marker,
                    target: &candidate.directory,
                    judged: &aside.version_judged,
                    holders: &second_look,
                    action: Settle::Delete,
                    deadline: run.deadline(),
                })
            {
                kept = Some(reason);
            }
            if kept.is_none()
                && let (Some(marker), Some(judged)) = (&aside.downloads_marker, &aside.downloads_judged)
                && let Settled::Kept(reason) = settle(&ToSettle {
                    path: marker,
                    target: marker,
                    judged,
                    holders: &second_look,
                    action: Settle::Delete,
                    deadline: run.deadline(),
                })
            {
                kept = Some(format!("its archives are still being removed: {reason}"));
            }
            match kept {
                None => {
                    let _ = fs::remove_file(record_of(&candidate.directory));
                    // Best effort: a tool's downloads directory goes once its last version's archives are gone.
                    if let Some(downloads) = &candidate.downloads
                        && let Some(tool_downloads) = downloads.parent()
                    {
                        let _ = fs::remove_dir(tool_downloads);
                    }
                    outcome.removed += 1;
                    outcome.removed_bytes = outcome.removed_bytes.saturating_add(candidate.bytes);
                    lines[index] = Some(removed_line(&candidate));
                }
                Some(reason) => {
                    // What: a tree the second look keeps goes back under its name, whole, as the record proved it.
                    // Why: it is the tree this pass judged a moment ago and nothing was written in it since; only its
                    // reader changed, and a copy a process uses is never deleted.
                    match settle(&ToSettle {
                        path: &aside.version_marker,
                        target: &candidate.directory,
                        judged: &aside.version_judged,
                        holders: &second_look,
                        action: Settle::Back,
                        deadline: run.deadline(),
                    }) {
                        Settled::Done => {
                            let _ = fs::remove_file(record_of(&candidate.directory));
                            if let (Some(marker), Some(downloads), Some(judged)) =
                                (&aside.downloads_marker, &candidate.downloads, &aside.downloads_judged)
                            {
                                let _ = settle(&ToSettle {
                                    path: marker,
                                    target: downloads,
                                    judged,
                                    holders: &second_look,
                                    action: Settle::Back,
                                    deadline: run.deadline(),
                                });
                            }
                            lines[index] = Some(kept_line(&candidate, &reason));
                        }
                        Settled::Kept(why) => {
                            outcome.unfinished.insert(candidate.root.clone());
                            let _ = fs::write(unfinished_marker(&candidate.root), b"");
                            lines[index] = Some(kept_line(
                                &candidate,
                                &format!("{reason}; it stays out of its name: {why}"),
                            ));
                        }
                    }
                }
            }
        }
    }
    // What: a root whose removals were all deferred gets the marker too.
    // Why: the marker is what makes the next install run the pass, and a pass that could not move anything left no
    // trace otherwise.
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
        format_age(SystemTime::now(), newest_write(candidate.written, candidate.newest))
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
    return finish_removals_with(run, &NamedVersions::default());
}

/// What:
/// `finish_removals` with the lock set the pass read.
/// Why:
/// A recovery is a removal like any other: a tree a killed pass left is judged again from nothing, by what is on the
/// drive now, and a lock that names its version now keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function finishRemovalsWith(run: TrimRun, named: NamedVersions): FinishOutcome
/// ```
pub fn finish_removals_with(run: &TrimRun, named: &NamedVersions) -> FinishOutcome {
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
        let locked: Option<String> = named.naming_lock(&marker.tool, &marker.version).map(str::to_string);
        // What: a deletion name is finished, never restored.
        // Why: only the deletion step writes that name, and it wrote it before unlinking the first file.
        if marker.version_deleting {
            finish_the_deletion(
                &marker.version_marker,
                &marker.root,
                &marker.tool,
                &marker.version,
                named,
                &holders,
                run.deadline(),
                &mut outcome,
                format!("{} and finished a deletion an earlier run started", marker.version_directory.display()),
            );
            continue;
        }
        if marker.downloads_deleting
            && let Some(downloads) = &marker.downloads_marker
        {
            finish_the_deletion(
                downloads,
                &marker.root,
                &marker.tool,
                &marker.version,
                named,
                &holders,
                run.deadline(),
                &mut outcome,
                format!("{} and finished a deletion an earlier run started", downloads.display()),
            );
            continue;
        }
        match fs::symlink_metadata(&marker.version_marker) {
            Ok(_) => restore_the_removal(&marker, locked.as_deref(), &holders, run.deadline(), &mut outcome),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                finish_without_a_version(&marker, &holders, run.deadline(), &mut outcome);
            }
            // What: a path that cannot be read is not a path that is absent.
            // Why: reading a failed read as "gone" deleted a version's archives while it was there.
            Err(error) => {
                outcome.unfinished.insert(marker.root.clone());
                outcome.lines.push(format!(
                    "mpm-shadow: kept {}: {} cannot be read: {error}",
                    marker.version_directory.display(),
                    marker.version_marker.display()
                ));
            }
        }
    }
    return outcome;
}

/// What:
/// Finishes a tree under a deletion name, judged again from nothing: its facts read now, the holders asked now, and a
/// lock that names its version keeping it.
/// Why:
/// The pass that began the deletion is dead, and what it decided is not evidence about what is on the drive now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishTheDeletion(tree: string, locked: string | undefined, holders: StoreHolders, line: string): void
/// ```
fn finish_the_deletion(
    tree: &Path,
    root: &Path,
    tool: &str,
    version: &str,
    named: &NamedVersions,
    holders: &StoreHolders,
    deadline: Instant,
    outcome: &mut FinishOutcome,
    line_when_done: String,
) {
    let Some(judged) = judged_of_tree(tree, deadline) else {
        outcome.unfinished.insert(root.to_path_buf());
        outcome.lines.push(format!("mpm-shadow: kept {}: it could not be read in full at this moment", tree.display()));
        return;
    };
    // What: a lock that cannot be read may name this version, so its deletion waits.
    // Why: what the pass could not read it takes for present.
    if let Some(reason) = &named.unreadable {
        outcome.unfinished.insert(root.to_path_buf());
        outcome.lines.push(format!("mpm-shadow: kept {}: {reason}, so its deletion waits", tree.display()));
        return;
    }
    if let Some(lock) = named.naming_lock(tool, version) {
        outcome.unfinished.insert(root.to_path_buf());
        outcome.lines.push(format!("mpm-shadow: kept {}: the lock {lock} names it now, so its deletion waits", tree.display()));
        return;
    }
    match settle(&ToSettle { path: tree, target: tree, judged: &judged, holders, action: Settle::Delete, deadline }) {
        Settled::Done => {
            if let Some(record) = record_of_marker(tree) {
                let _ = fs::remove_file(record);
            }
            outcome.lines.push(format!("mpm-shadow: removed {line_when_done}"));
        }
        Settled::Kept(reason) => {
            outcome.unfinished.insert(root.to_path_buf());
            outcome.lines.push(format!("mpm-shadow: kept {}: it is still being removed: {reason}", tree.display()));
        }
    }
}

/// What:
/// The archives alone are left aside: their version is still there and they go back, or it is provably gone at both
/// names and they are orphans that go.
/// Why:
/// A permission error is not proof of absence, and archives a version still uses belong to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishWithoutAVersion(marker: RemovalMarker, holders: StoreHolders): void
/// ```
fn finish_without_a_version(marker: &RemovalMarker, holders: &StoreHolders, deadline: Instant, outcome: &mut FinishOutcome) {
    let Some(downloads) = &marker.downloads_marker else {
        outcome.lines.push(format!("mpm-shadow: nothing to remove under {}", marker.version_marker.display()));
        return;
    };
    let version_there: bool = match provably_absent(&marker.version_directory) {
        Ok(absent) => !absent,
        Err(problem) => {
            outcome.unfinished.insert(marker.root.clone());
            outcome.lines.push(format!("mpm-shadow: kept {}: {problem}", downloads.display()));
            return;
        }
    };
    if version_there {
        match put_an_archive_back(marker, holders, deadline) {
            Ok(()) => outcome.lines.push(format!(
                "mpm-shadow: kept {}: the version it belongs to is still there, so what a removal moved aside is put back",
                downloads.display()
            )),
            Err(reason) => {
                outcome.unfinished.insert(marker.root.clone());
                outcome.lines.push(format!("mpm-shadow: kept {}: it is still being removed: {reason}", downloads.display()));
            }
        }
        return;
    }
    // The version is provably at neither name: what it was downloaded as belongs to nobody.
    let Some(judged) = judged_of_tree(downloads, deadline) else {
        outcome.unfinished.insert(marker.root.clone());
        outcome.lines.push(format!("mpm-shadow: kept {}: it could not be read in full at this moment", downloads.display()));
        return;
    };
    match settle(&ToSettle { path: downloads, target: downloads, judged: &judged, holders, action: Settle::Delete, deadline }) {
        Settled::Done => outcome.lines.push(format!(
            "mpm-shadow: removed {} and finished a removal an earlier run started",
            downloads.display()
        )),
        Settled::Kept(reason) => {
            outcome.unfinished.insert(marker.root.clone());
            outcome.lines.push(format!("mpm-shadow: kept {}: it is still being removed: {reason}", downloads.display()));
        }
    }
}

/// What:
/// Whether the archives under a live downloads name include a finished file.
/// Why:
/// An install writes its archive under a name ending in the in-flight suffix and renames it when it has finished, so a
/// folder that holds only such a file is a download under way, and a directory that cannot be read proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function liveArchivesAreFinished(directory: string): boolean
/// ```
fn live_archives_are_finished(directory: &Path) -> bool {
    let Ok(entries) = fs::read_dir(directory) else {
        return false;
    };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(IN_FLIGHT_DOWNLOAD_SUFFIX) {
            continue;
        }
        if entry.file_type().map(|kind| return kind.is_file()).unwrap_or(false) {
            return true;
        }
    }
    return false;
}

/// What:
/// Puts the archives a removal holds back under the version's name,
/// or deletes them when that name holds newer archives already.
/// Why:
/// They belong to a version that is still there, and a copy at the live name is the newer one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putAnArchiveBack(marker: RemovalMarker, holders: StoreHolders): string | undefined
/// ```
fn put_an_archive_back(marker: &RemovalMarker, holders: &StoreHolders, deadline: Instant) -> Result<(), String> {
    let (Some(archive), Some(live)) = (&marker.downloads_marker, &marker.downloads_directory) else {
        return Ok(());
    };
    let Some(judged) = judged_of_tree(archive, deadline) else {
        return Err("it could not be read in full at this moment".to_string());
    };
    if !provably_absent(live)? {
        // What: the live name holds something, and this copy is the older one only when that something finished.
        // Why: a live folder holding nothing but a download still in flight is not a replacement; deleting the complete
        // archives for it lost the only complete copy.
        if !live_archives_are_finished(live) {
            return Err(format!(
                "{} holds a download still in flight, so the complete archives beside it stay where they are",
                live.display()
            ));
        }
        return match settle(&ToSettle { path: archive, target: archive, judged: &judged, holders, action: Settle::Delete, deadline }) {
            Settled::Done => Ok(()),
            Settled::Kept(reason) => Err(reason),
        };
    }
    return match settle(&ToSettle { path: archive, target: live, judged: &judged, holders, action: Settle::Back, deadline }) {
        Settled::Done => Ok(()),
        Settled::Kept(reason) => Err(reason),
    };
}

/// What:
/// Puts a version a removal holds back under its name,
/// or deletes it when a finished copy of that version holds the name and the record proves this copy whole.
/// Why:
/// The copy is whole only when the record beside it still describes it; one that changed since, or that has no record,
/// may hold a person's later work and is never deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreTheRemoval(marker: RemovalMarker, locked: string | undefined, holders: StoreHolders): void
/// ```
fn restore_the_removal(marker: &RemovalMarker, locked: Option<&str>, holders: &StoreHolders, deadline: Instant, outcome: &mut FinishOutcome) {
    let Some(judged) = judged_of_tree(&marker.version_marker, deadline) else {
        outcome.unfinished.insert(marker.root.clone());
        outcome.lines.push(format!("mpm-shadow: kept {}: it could not be read in full at this moment", marker.version_marker.display()));
        return;
    };
    let whole: bool = read_judged(&marker.version_directory).is_some_and(|found| return found.matches(&judged));
    let reason: String = match locked {
        Some(lock) => format!("the lock {lock} names it now"),
        None => {
            if current_names(&marker.root, &marker.tool, &marker.version) {
                "the tool's current link names it".to_string()
            } else {
                "it is put back under its own name, and a later pass judges it again".to_string()
            }
        }
    };
    let name_free: bool = match provably_absent(&marker.version_directory) {
        Ok(absent) => absent,
        Err(problem) => {
            outcome.unfinished.insert(marker.root.clone());
            outcome.lines.push(format!("mpm-shadow: kept {}: {problem}", marker.version_marker.display()));
            return;
        }
    };
    if !name_free {
        // What: the name holds something else now.
        // Why: this copy is deleted only when its record proves it whole and the name holds a finished copy of it.
        if whole && finished_copy_at(&marker.tool, &marker.version_directory) {
            match settle(&ToSettle { path: &marker.version_marker, target: &marker.version_marker, judged: &judged, holders, action: Settle::Delete, deadline }) {
                Settled::Done => {
                    let _ = fs::remove_file(record_of(&marker.version_directory));
                    outcome.lines.push(format!(
                        "mpm-shadow: removed {} ({}) and finished a removal an earlier run started, and a newer copy holds its name",
                        marker.version_marker.display(),
                        format_size(judged.facts.bytes)
                    ));
                }
                Settled::Kept(why) => {
                    outcome.unfinished.insert(marker.root.clone());
                    outcome.lines.push(format!("mpm-shadow: kept {}: it is still being removed: {why}", marker.version_marker.display()));
                }
            }
            return;
        }
        outcome.unfinished.insert(marker.root.clone());
        outcome.lines.push(format!(
            "mpm-shadow: kept {}: its name now holds another copy and no record proves this one whole, so nothing is removed",
            marker.version_marker.display()
        ));
        return;
    }
    // What: the name is free, so the tree goes back, whole when its record proves it and unfinished otherwise.
    // Why: a copy nothing proved whole is never presented as a finished install.
    if !whole
        && let Some(failure) = unfinish_tree(&marker.tool, &marker.version_marker)
    {
        outcome.unfinished.insert(marker.root.clone());
        outcome.lines.push(format!("mpm-shadow: kept {}: {failure}", marker.version_marker.display()));
        return;
    }
    // What: a tree whose completion marker was just taken away is read once more, because that write changed it.
    // Why: \`settle\` proves the facts it is given against the facts on the drive, and the marker was one of them.
    let judged_now: JudgedTree = if whole {
        judged
    } else {
        let Some(after) = judged_of_tree(&marker.version_marker, deadline) else {
            outcome.unfinished.insert(marker.root.clone());
            outcome.lines.push(format!("mpm-shadow: kept {}: it could not be read in full at this moment", marker.version_marker.display()));
            return;
        };
        after
    };
    match settle(&ToSettle { path: &marker.version_marker, target: &marker.version_directory, judged: &judged_now, holders, action: Settle::Back, deadline }) {
        Settled::Done => {
            let _ = fs::remove_file(record_of(&marker.version_directory));
            if marker.downloads_marker.is_some()
                && let Err(archives_kept) = put_an_archive_back(marker, holders, deadline)
            {
                outcome.unfinished.insert(marker.root.clone());
                outcome.lines.push(format!("mpm-shadow: kept {}: it is still being removed: {archives_kept}", marker.version_marker.display()));
            }
            outcome.put_back.insert(marker.version_directory.clone(), reason.clone());
            if whole {
                outcome.lines.push(format!(
                    "mpm-shadow: kept {} ({}): {reason}; the removal an earlier run started is undone",
                    marker.version_directory.display(),
                    format_size(judged_now.facts.bytes)
                ));
            } else {
                let link_taken: bool = take_away_the_link(marker);
                outcome.lines.push(format!(
                    "mpm-shadow: kept {} ({}): {reason}; the removal an earlier run started is undone, and no record beside it \
                     proved the copy whole, so it is put back unfinished and the next install of that version replaces it{}",
                    marker.version_directory.display(),
                    format_size(judged_now.facts.bytes),
                    if link_taken { ", and the tool's current link is taken away until then" } else { "" }
                ));
            }
        }
        Settled::Kept(why) => {
            outcome.unfinished.insert(marker.root.clone());
            outcome.lines.push(format!(
                "mpm-shadow: kept {}: {reason}; it stays out of its name: {why}",
                marker.version_marker.display()
            ));
        }
    }
}

/// What:
/// Takes away the tool's `current` link when it names a copy that was put back unfinished.
/// Why:
/// A launcher follows that link without asking whether the version finished, and while the copy was aside the link named
/// nothing at all, so leaving it would turn an absent tool into a run of a copy nothing proved whole.
/// This is the one link this module removes, and it is read again first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeAwayTheLink(marker: RemovalMarker): void
/// ```
fn take_away_the_link(marker: &RemovalMarker) -> bool {
    let link: PathBuf = marker.root.join(&marker.tool).join(CURRENT_LINK_NAME);
    let named: Option<String> = match fs::read_link(&link) {
        Ok(target) => target.file_name().map(|name| return name.to_string_lossy().to_string()),
        Err(_) => None,
    };
    if named.as_deref() == Some(marker.version.as_str()) {
        return fs::remove_file(&link).is_ok();
    }
    return false;
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
/// the machine-global scope's,
/// the lock of every worktree of the repository the pass runs in,
/// whatever scope asked,
/// or why one of them cannot be found.
/// Why:
/// A version any of these pins is in use by some tree, and which locks can name a version in a store does not depend
/// on which scope asked: a global pass that read only its own lock removed a global-store version that a worktree's
/// lock still named.
/// A repository scope finds the repository in its own root; a global scope in `MONOCHROMATIC_WORKTREE_ROOT`, the same
/// variable the repository root is found from.
/// A global pass that can name no repository cannot see the worktrees' locks at all, so it removes nothing and says so,
/// rather than reading "no lock names it" into locks it never opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function visibleLocks(ctx: Context): { files: string[]; unreadable?: string }
/// ```
fn visible_locks(ctx: &Context, deadline: Instant) -> (Vec<PathBuf>, Option<String>) {
    let mut files: Vec<PathBuf> = vec![ctx.scope.lock_file.clone()];
    let mut unreadable: Option<String> = None;
    let repository: Option<PathBuf> = match ctx.scope.kind {
        ScopeKind::Repo => Some(ctx.scope.root.clone()),
        ScopeKind::Global => named_repository_root(ctx.env),
    };
    match repository {
        Some(root) => match worktree_lock_files(
            ctx.runner,
            root.as_path(),
            Ceiling::at(deadline, "the cleanup's own work limit"),
        ) {
            Ok(listed) => files.extend(listed),
            Err(error) => unreadable = Some(error.message),
        },
        None => {
            unreadable = Some(format!(
                "this pass is the machine-global scope and cannot name a repository, so the locks of the worktrees \
                 that install into this store cannot be read (set {} to the repository root, or run the sync inside \
                 the repository)",
                super::scope::WORKTREE_ROOT_ENV
            ));
        }
    }
    match global_scope(ctx.env) {
        Ok(global) if global.lock_file != ctx.scope.lock_file => files.push(global.lock_file),
        Ok(_) => {}
        // What: a global lock that cannot be found makes the locks unreadable.
        // Why: it may name any version, and reading "no lock names it" into a lock that was never found removed one.
        Err(error) => {
            if unreadable.is_none() {
                unreadable = Some(format!("the machine-global lock cannot be found: {}", error.message));
            }
        }
    }
    return (files, unreadable);
}

/// The roots one scope may remove versions from:
/// both of this machine's stores from a repository scope,
/// and only the machine-global store from the global scope,
/// which now reads the worktree locks of the repository it names.
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
    if let Some(reason) = owned_hold_reason(holders, &candidate.root, &candidate.tool, &candidate.version)
    {
        return Some(format!("it is in use: {reason}"));
    }
    return None;
}

/// What:
/// The `current` link targets of every tool of every root the pass trims,
/// or why one of them cannot be read.
/// Why:
/// A link left naming a removed version dangles,
/// and the tool's `current` is what every worktree's exposed links resolve through;
/// a link that cannot be read may name any version, so the pass keeps every version rather than judge one by a link it
/// never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CurrentLinks = { byTool: Map<[string, string], string>; unreadable?: string };
/// function currentLinks(roots: string[], tools: ReadonlySet<string>): CurrentLinks
/// ```
#[derive(Debug, Default)]
struct CurrentLinks {
    /// The version each tool's `current` link names, by root and tool directory name.
    by_tool: BTreeMap<(PathBuf, String), String>,
    /// Why one link could not be read.
    unreadable: Option<String>,
}

fn current_links(roots: &[PathBuf], tools: &BTreeSet<String>) -> CurrentLinks {
    let mut links: CurrentLinks = CurrentLinks::default();
    for root in roots {
        for tool in tools {
            let link: PathBuf = root.join(tool).join(CURRENT_LINK_NAME);
            match fs::read_link(&link) {
                Ok(target) => {
                    let name: String =
                        target.file_name().map(|name| return name.to_string_lossy().to_string()).unwrap_or_default();
                    if !name.is_empty() {
                        links.by_tool.insert((root.clone(), tool.clone()), name);
                    }
                }
                // What: a link that cannot be read may name any version of its tool.
                // Why: a failed read is not "the link names another version".
                Err(error) if error.kind() != ErrorKind::NotFound => {
                    links.unreadable = Some(format!("{} cannot be read: {error}", link.display()));
                }
                Err(_) => {}
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
    return trim_after_install_with(ctx, installed, None);
}

/// What:
/// `trim_after_install` with a scan of the process table already made.
/// Why:
/// A control hands in the scan it built, so it does not depend on which processes the machine happens to be running;
/// a desktop always holds processes the kernel refuses to describe, and the rule keeps every version while that is so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trimAfterInstallWith(ctx: Context, installed: [string, string][], holders?: StoreHolders): void
/// ```
pub fn trim_after_install_with(ctx: &Context, installed: &[(String, String)], holders: Option<StoreHolders>) {
    let roots: Vec<PathBuf> = roots_to_trim(ctx);
    if roots.is_empty() {
        return;
    }
    // What: the pass runs whenever a removal or deletion name stands, whatever this install did.
    // Why: the store-root marker is only a hint, and two passes settling it at once can clear it after one wrote it;
    // asking the store itself is what keeps a killed pass's work from waiting on an unrelated install.
    let pending: bool = roots.iter().any(|root| return has_unfinished_removal(root));
    let stood: bool = roots.iter().any(|root| return !removal_markers(root).is_empty());
    if installed.is_empty() && !pending && !stood {
        return;
    }
    let bounds: TrimBounds = TrimBounds::from_env(ctx.env);
    let run: TrimRun = TrimRun { bounds, started: Instant::now(), roots: roots.clone(), holders };
    let mut lines: Vec<String> = Vec::new();
    let mut unfinished: BTreeSet<PathBuf> = BTreeSet::new();
    // What: every lock this pass can read is read before anything is recovered.
    // Why: a recovery deletes a tree whose deletion has begun, and a lock that names that version keeps it; reading the
    // locks afterwards let recovery delete data a current lock still named, and a lock that could not be read
    // protected nothing at all.
    let (lock_files, lock_problem) = visible_locks(ctx, run.deadline());
    let named: NamedVersions = match lock_problem {
        Some(reason) => NamedVersions { by_tool: BTreeMap::new(), unreadable: Some(reason) },
        None => read_named_versions(&lock_files),
    };
    // What: the removals an earlier run started are finished or undone, judged by the locks just read.
    // Why: a tree a killed pass left is judged again from nothing, by what is on the drive now, and a lock that names
    // its version keeps it.
    let finished: FinishOutcome = finish_removals_with(&run, &named);
    lines.extend(finished.lines);
    unfinished.extend(finished.unfinished);
    let tools: BTreeSet<String> = named.tool_directories();
    if let Some(reason) = &named.unreadable {
        // What: a lock that cannot be read keeps every version.
        // Why: it might name any of them, so the pass cannot tell an old version from one in use.
        lines.push(format!(
            "mpm-shadow: removed no toolchain version: {reason}; every version is kept until the locks can be read"
        ));
    } else if !tools.is_empty() {
        let scan: StoreHolders = run.look();
        // What: a process table the scan could not complete keeps every version, with one line for the pass.
        // Why: a process the scan could not describe may run any version, so nothing can be told from nothing it read;
        // one line naming those processes is what a person needs, and one line per version would bury it.
        if let Some(reason) = &scan.incomplete {
            lines.push(format!(
                "mpm-shadow: removed no toolchain version: the process table could not be read in full: {reason}; \
                 every version is kept while that is so"
            ));
            settle_markers(&roots, &unfinished);
            for line in lines {
                eprintln!("{line}");
            }
            return;
        }
        let installed_set: BTreeSet<(String, String)> = installed.iter().cloned().collect();
        let links: CurrentLinks = current_links(&roots, &tools);
        // What: a `current` link this pass could not read may name any version of its tool.
        // Why: reading "the link does not name it" into a failed read removed a version the link names.
        if let Some(reason) = &links.unreadable {
            lines.push(format!(
                "mpm-shadow: removed no toolchain version: {reason}; every version is kept until the links can be read"
            ));
            settle_markers(&roots, &unfinished);
            for line in lines {
                eprintln!("{line}");
            }
            return;
        }
        let candidates: Vec<TrimCandidate> = candidates_of(&roots, &tools, run.deadline(), bounds.age_floor);
        let protection = |candidate: &TrimCandidate| -> Option<String> {
            return protection_of(candidate, &named, &scan, &installed_set, &links.by_tool, &finished.put_back);
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
