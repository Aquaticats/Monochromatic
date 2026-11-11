//! What:
//! Mechanism D,
//! the retention of verification run records under a `target/verification` directory.
//! Why:
//! Fuzz,
//! mutation and container-gate runs leave a record directory or log per run,
//! named by the program that wrote it (`RECORD_PREFIXES` names every such program and its prefix),
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
use crate::lean_removal::{Refusal, TRASH_DIRECTORY_NAME, claim_into_trash, delete_claimed, empty_trash, item_of, modes_named};
/// Imports the names this file uses from `crate::lean_sizes`.
use crate::lean_sizes::add_tree_bytes;

/// The name prefixes of run records,
/// one entry per program of this repository that writes a record of its own run into a verification directory,
/// each beside what writes it.
/// Why:
/// An entry whose name no prefix matches is never removed,
/// so a producer missing here leaves its records for good;
/// and an entry whose name matches one by accident would lose data,
/// so every entry is a name a program really writes.
/// The names are read from the programs and not invented:
/// a producer that takes a new name adds it here in the same change.
///
/// - `campaign-`:
///   `package/git-policy/cli.fuzz/bin/container.ts`
///   and `package/linter/monochromatic-lint.fuzz/bin/container.ts`.
/// - `clippy-` and `release-test-` and `test-`:
///   `package/forbidden-strings/cli/bin/test-container.ts` (`nameOf`).
/// - `container-`:
///   `package/forbidden-strings/cli/bin/container-snapshot.ts`
///   before it took a name per run;
///   the records of 2026-10-04 are still on disk.
/// - `coverage-`:
///   `package/mutation-test/cli-cargo-mutants/src/scope-tools.ts`.
/// - `embedding-fuzz-` and `fuzz-clippy-`:
///   `package/forbidden-strings/cli.fuzz/bin/smoke-container.ts`.
/// - `git-image-`:
///   `package/git-policy/cli/bin/build-git-test-image.ts`.
/// - `guard-observer-`:
///   `package/forbidden-strings/cli/bin/observe-guards-container.ts`.
/// - `guard-protected-` and `guard-without-`:
///   `package/forbidden-strings/cli/bin/guard-container.ts`.
///   The fixed names of `package/forbidden-strings/cli/bin/verify-evidence.ts`
///   (`guard-source-comparison` and `guard-input-comparisons.json`)
///   are rewritten in place and not kept per run,
///   and no prefix here matches them.
/// - `mutation-`:
///   `package/linter/monochromatic-lint/bin/mutate-container.ts`
///   and `package/forbidden-strings/core/bin/mutate-container.ts`
///   and `package/mutation-test/cli-cargo-mutants/src/campaign-attempt.ts`.
/// - `native-` (which also covers `native-mutation-`):
///   `package/git-policy/cli/bin/test-native-container.ts`
///   and the gate prefix of `package/git-policy/cli/bin/native-gate-proof.ts`.
/// - `pathname-mutation-` and `startup-mutation-`:
///   `package/forbidden-strings/cli/bin/mutate-container.ts`.
///   `pathname-mutation-` also covers the `pathname-mutation-list-` names of its `--list` runs.
/// - `planted-`:
///   the `planted-controls.ts` of both fuzz packages.
/// - `processors-mutation-`:
///   `package/linter/monochromatic-lint/bin/mutate-processors.ts`.
/// - `windows-`:
///   the scanner's Windows snapshots before the same producer took its names per run;
///   the records of 2026-10-05 are still on disk.
pub const RECORD_PREFIXES: [&str; 19] = [
    "campaign-",
    "clippy-",
    "container-",
    "coverage-",
    "embedding-fuzz-",
    "fuzz-clippy-",
    "git-image-",
    "guard-observer-",
    "guard-protected-",
    "guard-without-",
    "mutation-",
    "native-",
    "pathname-mutation-",
    "planted-",
    "processors-mutation-",
    "release-test-",
    "startup-mutation-",
    "test-",
    "windows-",
];

/// What:
/// `pub fn is_record_name(name: &str) -> bool` says whether an entry of a verification directory is a run record.
/// Why:
/// One predicate decides what the retention may remove,
/// so a test can ask it about a name without building a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const isRecordName = (name: string): boolean => RECORD_PREFIXES.some((prefix) => name.startsWith(prefix));
/// ```
pub fn is_record_name(name: &str) -> bool {
    for prefix in RECORD_PREFIXES {
        if name.starts_with(prefix) {
            return true;
        }
    }
    return false;
}

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
        if !is_record_name(&name) {
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
        let sweep = empty_trash(&trash);
        if !sweep.modes_added.is_empty() {
            emit(log, format!("finished an earlier run's deletion in {} ({} disk bytes){};", trash.display(), sweep.bytes, modes_named(&sweep.modes_added)));
        }
        outcome.refusals.extend(sweep.failures);
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
            Ok(claimed) => match delete_claimed(&claimed, item.identity) {
                Ok(deleted) => {
                    emit(log, format!(
                        "removed run record {} (device {} inode {}): {why}; proof: {}{}",
                        record.path.display(),
                        item.identity.device,
                        item.identity.inode,
                        scan.describe(),
                        modes_named(&deleted.modes_added)
                    ));
                    outcome.removed += 1;
                    outcome.bytes += deleted.bytes;
                    total = total.saturating_sub(record.bytes);
                }
                Err(Refusal::PartlyRemoved { deleted, .. }) => {
                    emit(log, format!(
                        "partly removed run record {} (device {} inode {}): {why}; {} disk bytes are gone and {} part(s) stay{}; each stayed because: {}",
                        record.path.display(),
                        item.identity.device,
                        item.identity.inode,
                        deleted.bytes,
                        deleted.remaining.len(),
                        modes_named(&deleted.modes_added),
                        deleted.remaining.join("; ")
                    ));
                    outcome.bytes += deleted.bytes;
                    outcome.refusals.push(format!("{} was only partly removed: {}", record.path.display(), deleted.remaining.join("; ")));
                }
                Err(refusal) => outcome.refusals.push(refusal.to_string()),
            },
            Err(refusal) => outcome.refusals.push(refusal.to_string()),
        }
    }
    let _ = fs::remove_dir(&trash);
    return outcome;
}
