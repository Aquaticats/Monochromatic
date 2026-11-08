//! What:
//! Mechanism D,
//! the retention of verification run records under a `target/verification` directory.
//! Why:
//! Fuzz,
//! mutation and container-gate runs leave a record directory or log per run
//! (`campaign-*`,
//! `native-*`,
//! `git-image-*`,
//! `mutation-*`,
//! `planted-*`),
//! and nothing ever removes them (`doc/planning/lean-build-dirs.md`,
//! option D;
//! 6.2 GiB in five packages on 2026-11-04).
//! A record is removed when it is older than the retention age,
//! and the oldest go first while the directory is over its size cap.
//! A record written within the protection window,
//! or one a process uses,
//! is never removed.
//! Anything else in the directory (a rendered handover,
//! a manifest) is not a run record and is never touched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function sweepRunRecords(directory: string, policy: RetentionPolicy, env: GuardEnvironment, now: number, dryRun: boolean): RecordsOutcome
//! ```

/// The streaming log line.
use crate::lean_run_report::emit;
/// Files and sets.
use std::{collections::HashSet, fs, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// The proof,
/// the removal and the sizes.
use crate::lean_guard::GuardEnvironment;
/// Imports the names this file uses from `crate::lean_identity`.
use crate::lean_identity::{Deadline, Identity};
/// Imports the names this file uses from `crate::lean_proc_scan`.
use crate::lean_proc_scan::{ScanReport, scan_processes};
/// Imports the names this file uses from `crate::lean_removal`.
use crate::lean_removal::{TRASH_DIRECTORY_NAME, claim_into_trash, delete_claimed, empty_trash, item_of};
/// Imports the names this file uses from `crate::lean_sizes`.
use crate::lean_sizes::add_tree_bytes;

/// The name prefixes of run records.
pub const RECORD_PREFIXES: [&str; 5] = ["campaign-", "native-", "git-image-", "mutation-", "planted-"];

/// The retention of one verification directory.
#[derive(Clone, Copy, Debug)]
pub struct RetentionPolicy {
    /// Records older than this many seconds are removed.
    pub max_age_seconds: i64,
    /// The oldest records are removed while the directory's records occupy more than this many disk bytes.
    pub max_bytes: u64,
    /// A record written within this many seconds is never removed.
    pub protect_seconds: i64,
}

/// What a sweep of one directory did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordsOutcome {
    /// Records removed,
    /// or in a dry run that would be.
    pub removed: usize,
    /// Disk bytes freed,
    /// or that would be.
    pub bytes: u64,
    /// Records a process was using,
    /// kept.
    pub kept_in_use: usize,
    /// Refusals,
    /// with reasons.
    pub refusals: Vec<String>,
}

/// One run record.
struct Record {
    /// Where it is.
    path: PathBuf,
    /// The newest write time of it and its direct children.
    modified: i64,
    /// Its disk bytes.
    bytes: u64,
}

/// What:
/// `fn newest_write(path: &Path) -> i64` is the newest modification time of an entry and its direct children.
/// Why:
/// A run keeps writing into its directory,
/// so the directory's own time is not enough.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const newestWrite = (path: string): number
/// ```
fn newest_write(path: &Path) -> i64 {
    let mut newest: i64 = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata.mtime(),
        Err(_) => 0,
    };
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                newest = newest.max(metadata.mtime());
            }
        }
    }
    return newest;
}

/// What:
/// `fn older_first(left: &Record, right: &Record) -> std::cmp::Ordering` orders records by write time,
/// oldest first.
/// Why:
/// Eviction goes oldest first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const olderFirst = (a: Record, b: Record): number => a.modified - b.modified;
/// ```
fn older_first(left: &Record, right: &Record) -> std::cmp::Ordering {
    return left.modified.cmp(&right.modified);
}

/// What:
/// `fn records_of(directory: &Path, deadline: &Deadline) -> Vec<Record>` lists the run records of a verification directory,
/// oldest first.
/// Why:
/// Only names with a run prefix are records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordsOf(directory: string, deadline: Deadline): Record[]
/// ```
fn records_of(directory: &Path, deadline: &Deadline) -> Vec<Record> {
    let mut records: Vec<Record> = Vec::new();
    let mut seen: HashSet<Identity> = HashSet::new();
    let Ok(entries) = fs::read_dir(directory) else { return records };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let mut is_record: bool = false;
        for prefix in RECORD_PREFIXES {
            if name.starts_with(prefix) {
                is_record = true;
            }
        }
        if !is_record {
            continue;
        }
        let path: PathBuf = entry.path();
        let bytes: u64 = add_tree_bytes(&path, &mut seen, deadline).bytes;
        records.push(Record { modified: newest_write(&path), path, bytes });
    }
    records.sort_by(older_first);
    return records;
}

/// What:
/// `pub fn sweep_run_records(directory: &Path, policy: &RetentionPolicy, environment: &GuardEnvironment, now: i64, dry_run: bool, deadline: &Deadline, log: &mut Vec<String>) -> RecordsOutcome`
/// applies the retention to one verification directory.
/// Why:
/// It is mechanism D.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function sweepRunRecords(directory: string, policy: RetentionPolicy, env: GuardEnvironment, now: number, dryRun: boolean, deadline: Deadline, log: string[]): RecordsOutcome
/// ```
pub fn sweep_run_records(
    directory: &Path,
    policy: &RetentionPolicy,
    environment: &GuardEnvironment,
    now: i64,
    dry_run: bool,
    deadline: &Deadline,
    log: &mut Vec<String>,
) -> RecordsOutcome {
    let mut outcome: RecordsOutcome = RecordsOutcome::default();
    let records: Vec<Record> = records_of(directory, deadline);
    let mut total: u64 = 0;
    for record in &records {
        total += record.bytes;
    }
    let scan: ScanReport = if dry_run { ScanReport::default() } else { scan_processes(&environment.proc_root, &[directory], &[], environment.own_pid) };
    let trash: PathBuf = directory.join(TRASH_DIRECTORY_NAME);
    if !dry_run {
        outcome.refusals.extend(empty_trash(&trash));
    }
    for record in &records {
        let age: i64 = now - record.modified;
        let over_age: bool = age > policy.max_age_seconds;
        let over_size: bool = total > policy.max_bytes;
        if age <= policy.protect_seconds || !(over_age || over_size) {
            continue;
        }
        let why: String = if over_age {
            format!("written {age} s ago, older than the retention of {} s", policy.max_age_seconds)
        } else {
            format!("the directory's records occupy {total} bytes, over the cap of {} bytes; oldest first", policy.max_bytes)
        };
        if dry_run {
            outcome.removed += 1;
            outcome.bytes += record.bytes;
            total = total.saturating_sub(record.bytes);
            continue;
        }
        if let Some(used) = scan.blocks(&record.path) {
            outcome.kept_in_use += 1;
            emit(log, format!("kept run record {}: pid {} uses {} as {}", record.path.display(), used.pid, used.path.display(), used.how));
            continue;
        }
        let item = match item_of(&record.path) {
            Ok(found) => found,
            Err(error) => {
                outcome.refusals.push(format!("{}: {error}", record.path.display()));
                continue;
            }
        };
        match claim_into_trash(&item, &trash, 0) {
            Ok(claimed) => {
                emit(log, format!("removing run record {} (device {} inode {}): {why}; proof: {}", record.path.display(), item.identity.device, item.identity.inode, scan.describe()));
                match delete_claimed(&claimed, item.identity) {
                    Ok(bytes) => {
                        outcome.removed += 1;
                        outcome.bytes += bytes;
                        total = total.saturating_sub(record.bytes);
                    }
                    Err(refusal) => outcome.refusals.push(refusal.to_string()),
                }
            }
            Err(refusal) => outcome.refusals.push(refusal.to_string()),
        }
    }
    let _ = fs::remove_dir(&trash);
    return outcome;
}
