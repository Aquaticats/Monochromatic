//! What:
//!  The copying of a staged snapshot's files and links from the source into the stage,
//! one worker per slice of the manifest.
//! Why:
//!  The stage must hold every byte before the final comparison,
//! and a clone or a copy of one file is independent of the
//!  next,
//! so a large manifest is split in slices that run side by side,
//! each stopping when another has failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stamps = await copyManifestEntries(sourceRoot, stageRoot, entries); // one stamp per cloned file
//! ```

use crate::parallel_budget::{HostResources, sliced_workers};
use crate::side_by_side_on_the_runtime::run_all_side_by_side_on_the_runtime;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of};
use crate::worktree_copy_manifest::{EntryKind, FileStamp, WorktreeCopyEntry, stamp_of};
use crate::worktree_copy_selection::filesystem_path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::path::{Path, PathBuf};

/// What:
///  Creates one private directory with mode `0700`,
/// parents included.
/// Why:
///   Staged content is never readable by other users,
/// even before installation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
#[cfg(unix)]
pub fn create_private_tree(path: &Path) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::DirBuilderExt;
    return std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(raw_failure_of);
}

/// What:
///  Creates one directory tree on a platform without Unix modes.
/// Why:
///   Keeps the staging code identical on every platform;
/// no mode is claimed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn create_private_tree(path: &Path) -> Result<(), WorktreeCopyError> {
    return std::fs::create_dir_all(path).map_err(raw_failure_of);
}

/// What:
///  Asks the filesystem to clone the source's content into a new destination file,
/// when it can.
/// Why:
///   A clone shares storage on copy-on-write filesystems;
/// any refusal means the caller must copy bytes instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, destination, constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE)
/// ```
#[cfg(target_os = "linux")]
pub fn try_clone(source: &std::fs::File, destination: &std::fs::File) -> bool {
    // The one `FICLONE` call lives with the other Linux filesystem primitives, which the cheap-worktree ladder shares.
    return crate::fs_clone_exchange::clone_file(source, destination, std::path::Path::new("")).is_ok();
}

/// What:
///  Declines cloning on platforms where this lane has no clone request,
/// so every copy takes the byte path.
/// Why:
///   The incumbent's fallback is a full copy,
/// which is always correct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, destination, constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE)
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn try_clone(_source: &std::fs::File, _destination: &std::fs::File) -> bool {
    return false;
}

/// What:
///  Creates a destination file exclusively and fills it with the source's bytes,
///  cloning when the filesystem allows.
/// Why:
///   Exclusive creation never overwrites an existing staged file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyFile(sourcePath, stagePath, COPYFILE_EXCL | COPYFILE_FICLONE);
/// ```
pub fn copy_file_exclusive(source: &Path, destination: &Path) -> Result<(), WorktreeCopyError> {
    return copy_file_exclusive_with(source, destination, try_clone).map(drop);
}

/// What:
///  How recent a change time may be and still be too close to the clock to prove "unchanged".
/// Why:
///   Filesystem timestamps have a granularity;
/// a write that lands in the same tick as the stamp would leave it equal,
///  so a file changed this recently is compared byte for byte instead of trusted.
///  A control states it as well,
/// so it can ask for a settled stamp without waiting for the clock to move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RECENT_CHANGE_NANOS = 20_000_000n;
/// ```
pub(crate) const RECENT_CHANGE_NANOS: i128 = 20_000_000;

/// A time of the file system's stamping in nanoseconds since the epoch,
/// from its seconds and nanoseconds.
/// One conversion for the clock and the stamp,
/// so a control can compare the same two figures.
pub(crate) fn stamp_nanos(parts: (i64, i64)) -> i128 {
    return i128::from(parts.0) * 1_000_000_000 + i128::from(parts.1);
}

/// What:
///  The clock that says whether a change time is old enough to prove that no write follows it unseen,
/// in
///  nanoseconds since the epoch.
/// Why:
///   A control has to ask the question about a stated instant,
/// not about how long the host took between
///  the write and the look:
/// a control of this repository failed once in a whole-suite run because a 7-byte write
///  and the look after its copy were more than the 20 ms granularity apart
/// (`doc/troubleshooting/git-policy-cli-stop-lock-controls-wait-on-order-not-time.md`).
/// Nothing of
///  the pinning is in the release executable.
fn clock_nanos() -> Option<i128> {
    #[cfg(test)]
    if let Some(pinned) = PINNED_CLOCK_NANOS.with(std::cell::Cell::get) {
        return Some(pinned);
    }
    let now: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?;
    return Some(i128::from(now.as_secs()) * 1_000_000_000 + i128::from(now.subsec_nanos()));
}

#[cfg(test)]
thread_local! {
    /// The reading a control on this thread gave the clock;
    /// nothing outside a control.
    static PINNED_CLOCK_NANOS: std::cell::Cell<Option<i128>> = const { std::cell::Cell::new(None) };
}

/// What:
///  Make the clock read `nanos` on this thread until the returned guard drops.
/// Why:
///   The control decides the question by the file's own change time against a stated instant,
/// so it never waits
///  for a clock to move and never fails because the host was slow.
#[cfg(test)]
pub(crate) fn clock_reads(nanos: i128) -> ClockPin {
    PINNED_CLOCK_NANOS.with(|slot| { slot.set(Some(nanos)); });
    return ClockPin;
}

/// Ends the pinned reading of this thread's clock when dropped.
#[cfg(test)]
pub(crate) struct ClockPin;

#[cfg(test)]
impl Drop for ClockPin {
    fn drop(&mut self) {
        PINNED_CLOCK_NANOS.with(|slot| { slot.set(None); });
    }
}

/// What:
///  Whether a stamp is old enough,
/// against the clock now,
/// to prove that no write follows it unseen.
/// Why:
///   A change time in the future or within the timestamp granularity of now proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// now - ctime >= RECENT_CHANGE_NANOS
/// ```
pub(crate) fn stamp_is_settled(changed: (i64, i64)) -> bool {
    let Some(now_nanos) = clock_nanos() else {
        return false;
    };
    return now_nanos - stamp_nanos(changed) >= RECENT_CHANGE_NANOS;
}

/// What:
///  Creates a destination file exclusively and fills it with the source's bytes,
///  using the clone request the caller supplies,
/// and returns the source's stamp when a clone proved the bytes.
/// Why:
///   Whether the filesystem clones is the machine's choice,
/// so a test supplies a clone that accepts or refuses
///  and sees each path taken on every machine.
/// A clone shares the source's blocks as of one instant;
/// a source stamp that is
///  equal before and after it and old enough to be settled proves no write interleaved,
/// and the final check compares it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stamp = await copyFileWith({ sourcePath, stagePath, clone }); // undefined when bytes were copied
/// ```
pub fn copy_file_exclusive_with(
    source: &Path,
    destination: &Path,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<Option<FileStamp>, WorktreeCopyError> {
    let mut input: std::fs::File =
        std::fs::File::open(source).map_err(raw_failure_of)?;
    let mut output: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(raw_failure_of)?;
    let before: Option<FileStamp> = input.metadata().ok().as_ref().and_then(stamp_of);
    // A successful clone already holds every byte, so only a refused clone copies them.
    if !clone(&input, &output) {
        std::io::copy(&mut input, &mut output).map_err(raw_failure_of)?;
        return Ok(None);
    }
    let after: Option<FileStamp> = input.metadata().ok().as_ref().and_then(stamp_of);
    return Ok(match (before, after) {
        (Some(first), Some(second)) if first == second && stamp_is_settled(second.changed) => Some(second),
        _ => None,
    });
}

/// What:
///  The entries below which a copy stays on one thread:
/// `PARALLEL_BREAK_EVEN_ENTRIES`.
/// Why:
///   Starting threads costs more than a very small copy.
/// Measured with `copy_manifest_entries` itself forced to 1,
/// 2,
/// 4 and 8
///  workers on reflinked trees of this repository (2026-11-03,
/// five runs each in alternating orders,
/// other sessions running,
/// a
///  debug build):
/// 8 workers took 0.7 ms against 1.1 ms for 30 entries,
/// 1.9 ms against 3.6 ms for 90,
/// 5.9 ms against 13.4 ms for
///  296,
/// 25.4 ms against 91.4 ms for 2,119,
/// 57.3 ms against 202.6 ms for 4,722,
/// and 104.4 ms against 397.6 ms for 9,301,
/// so the
///  old threshold of 4096 kept copies of up to 4,095 entries on one thread for up to 145 ms more.
/// Picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// option A).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_COPY_MIN_ENTRIES = PARALLEL_BREAK_EVEN_ENTRIES;
/// ```
pub const PARALLEL_COPY_MIN_ENTRIES: usize = crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES;

/// What:
///  The most threads one copy starts.
/// Why:
///   Cloning is a metadata operation bound by the filesystem's locks more than by CPU,
/// and other sessions share this host.
///  Measured with `clone_tree` (`package_repair_tree_clone.rs`,
/// the same `copy_file_exclusive` primitive as the copy stage) on trees of reflinked tracked files of this repository,
/// medians of three runs,
/// 28 threads,
/// other sessions running,
/// 2026-11-03:
/// at 22,718 entries 1,
/// 2,
/// 4,
/// 8 and 16 workers took 1,549,
/// 880,
/// 555,
/// 397 and 415 ms (three runs each,
/// the
///  spread of 16 workers 383 to 444 ms),
/// at 9,301 entries 619,
/// 363,
/// 236,
/// 183 and 194 ms,
/// and at 4,723 entries 330,
/// 190,
/// 124,
/// 99
///  and 106 ms,
/// so more than eight workers were never faster.
/// Kept at eight;
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_COPY_MAX_WORKERS = 8;
/// ```
pub(crate) const PARALLEL_COPY_MAX_WORKERS: usize = 8;

/// What:
///  How many threads a copy of this many entries uses:
/// one below the threshold,
/// else the host's budget (the CPUs the process may
///  use less a margin,
/// and the free memory),
/// capped.
/// Why:
///   The bound follows the CPUs the process is allowed (a container's quota included),
/// never the whole host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries < MIN ? 1 : Math.min(availableParallelism(), MAX)
/// ```
pub fn copy_worker_count(entries: usize) -> usize {
    return copy_worker_count_on(&HostResources::read(), entries);
}

/// `copy_worker_count` with the host's resources chosen by the caller,
/// so a control states a host of any size.
pub fn copy_worker_count_on(host: &HostResources, entries: usize) -> usize {
    if entries < PARALLEL_COPY_MIN_ENTRIES {
        return 1;
    }
    return sliced_workers(entries, PARALLEL_COPY_MAX_WORKERS, host);
}

/// What:
///  Copies the files and links of one slice of a manifest into the stage,
/// recording each cloned file's stamp in a list of the slice's own.
/// Why:
///   Each entry is recreated from the source's no-follow view,
/// so no symbolic link is ever followed;
///  directories exist already,
/// and a parent is created only when the entry's parent changes.
///  The stamps come back with the slice rather than into one shared list,
/// because a job of the blocking pool owns what it reads and writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyLeaves({ sourceRoot, stageRoot, entries, failed }): Promise<(FileStamp | undefined)[]>;
/// ```
fn copy_leaves(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
    failed: &AtomicBool,
) -> Result<Vec<Option<FileStamp>>, WorktreeCopyError> {
    let mut last_parent: Option<PathBuf> = None;
    let mut stamps: Vec<Option<FileStamp>> = Vec::with_capacity(entries.len());
    for entry in entries {
        if failed.load(Ordering::Relaxed) {
            return Ok(stamps);
        }
        if entry.kind == EntryKind::Directory {
            stamps.push(None);
            continue;
        }
        let source_path: PathBuf = filesystem_path(source_root, entry.relative_path.as_str())?;
        let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_str())?;
        let parent: PathBuf = parent_of(stage_path.as_path());
        if last_parent.as_ref() != Some(&parent) {
            create_private_tree(parent.as_path())?;
            last_parent = Some(parent);
        }
        // `match` on the kind: files are cloned or copied and links recreated with their exact target.
        match entry.kind {
            EntryKind::Directory => stamps.push(None),
            EntryKind::File => {
                stamps.push(copy_file_exclusive_with(source_path.as_path(), stage_path.as_path(), try_clone)?);
            }
            EntryKind::Symlink => {
                let target: PathBuf = std::fs::read_link(source_path.as_path())
                    .map_err(raw_failure_of)?;
                symlink_at(target.as_path(), stage_path.as_path())?;
                stamps.push(None);
            }
        }
    }
    return Ok(stamps);
}

/// What:
///  Every entry of the manifest,
/// shared with the jobs that copy the leaves among them,
/// and the two roots they go between.
/// Why:
///   A job of the blocking pool owns what it reads,
/// so the manifest the caller lent is shared once here
///  and each job takes its own share of it,
/// and its own slice of the stamps by position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheCopy = { entries: readonly WorktreeCopyEntry[]; source: string; stage: string; positions: number[]; failed: boolean };
/// ```
type ASliceToCopy = (Arc<[WorktreeCopyEntry]>, PathBuf, PathBuf, Vec<usize>, Arc<AtomicBool>);

/// What:
///  One slice of the manifest as the state of one job:
/// the entries,
/// the positions of this slice,
/// the two roots and the flag that stops the other workers.
/// Why:
///   The budget runner takes a function and the state it reads,
/// never a closure,
/// so the slice travels as values the job owns and
///  may be taken again where no thread can start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copyASliceOf = ([entries, source, stage, positions, failed]) => copySlice(source, stage, positions, failed);
/// ```
fn copy_a_slice_of((entries, source_root, stage_root, positions, failed): ASliceToCopy) -> Result<Vec<Option<FileStamp>>, WorktreeCopyError> {
    let mut slice: Vec<WorktreeCopyEntry> = Vec::with_capacity(positions.len());
    for at in positions {
        let Some(entry) = entries.get(at) else { continue };
        slice.push(entry.clone());
    }
    return copy_slice_and_flag_failure((source_root.as_path(), stage_root.as_path(), slice.as_slice(), failed.as_ref()));
}

/// What:
///  Copies one manifest's entries into the stage:
/// directories first,
/// then files and links in parallel slices.
/// Why:
///   The manifest is parent first,
/// so every directory exists before a worker needs it;
/// the slices are contiguous so
///  neighbouring files share a directory,
/// and the first failure stops the other workers.
/// Every slice is a job of the process's blocking pool,
/// so the copy never holds a worker of the runtime that polls tasks.
/// The callers of this face are synchronous (the staging of one created worktree's ignored state),
/// so it enters the process's runtime once and waits there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyManifestEntries({ sourceRoot, stageRoot, entries }): Promise<(FileStamp | undefined)[]>;
/// ```
pub fn copy_manifest_entries(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
) -> Result<Vec<Option<FileStamp>>, WorktreeCopyError> {
    return match crate::lazy_runtime::run_on_runtime(copy_manifest_entries_awaiting(source_root, stage_root, Arc::from(entries))) {
        Ok(stamps) => stamps,
        // Nothing was copied, and the reason the runtime could not run the workers is why.
        Err(refusal) => Err(raw_failure(&std::io::Error::other(format!("the staged copy could not be made ({refusal})")))),
    };
}

/// What:
///  `copy_manifest_entries` on the process's runtime:
/// every slice of leaves is one job of the blocking pool.
/// Why:
///   A caller that is itself a task of the runtime cannot enter it,
/// so it awaits this face instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyManifestEntries({ sourceRoot, stageRoot, entries }): Promise<(FileStamp | undefined)[]>;
/// ```
async fn copy_manifest_entries_awaiting(
    source_root: &Path,
    stage_root: &Path,
    entries: Arc<[WorktreeCopyEntry]>,
) -> Result<Vec<Option<FileStamp>>, WorktreeCopyError> {
    for entry in entries.iter() {
        if entry.kind != EntryKind::Directory {
            continue;
        }
        let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_str())?;
        create_private_tree(stage_path.as_path())?;
    }
    let mut stamps: Vec<Option<FileStamp>> = vec![None; entries.len()];
    let workers: usize = copy_worker_count(entries.len());
    let failed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    if workers <= 1 {
        let copied: Vec<Option<FileStamp>> = copy_leaves(source_root, stage_root, entries.as_ref(), failed.as_ref())?;
        for (slot, stamp) in stamps.iter_mut().zip(copied) {
            *slot = stamp;
        }
        return Ok(stamps);
    }
    let slice_length: usize = entries.len().div_ceil(workers);
    let mut states: Vec<ASliceToCopy> = Vec::new();
    for chunk in entries.chunks(slice_length) {
        let first: usize = (chunk.as_ptr() as usize - entries.as_ptr() as usize) / std::mem::size_of::<WorktreeCopyEntry>();
        let positions: Vec<usize> = (first..first + chunk.len()).collect();
        states.push((Arc::clone(&entries), source_root.to_path_buf(), stage_root.to_path_buf(), positions, Arc::clone(&failed)));
    }
    let mut outcomes: Vec<Result<Vec<Option<FileStamp>>, WorktreeCopyError>> = Vec::new();
    for answer in run_all_side_by_side_on_the_runtime(states, copy_a_slice_of).await {
        outcomes.push(match answer {
            Ok(outcome) => outcome,
            Err(crate::blocking_job::BlockingJobError::Panicked) => Err(raw_failure(&std::io::Error::other("a copy worker panicked"))),
            Err(refused) => Err(raw_failure(&std::io::Error::other(format!("a copy worker did not run ({refused})")))),
        });
    }
    let mut at: usize = 0;
    for outcome in outcomes {
        for stamp in outcome? {
            let Some(slot) = stamps.get_mut(at) else { break };
            *slot = stamp;
            at += 1;
        }
    }
    return Ok(stamps);
}

/// What:
///  Copy one slice of leaf entries on a worker,
/// and tell the other workers to stop when it failed.
/// Why:
///   The workers share the flag only to end early;
/// the first failure is each worker's own result,
/// and the caller reports the
///  first in slice order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copySliceAndFlagFailure([sourceRoot, stageRoot, slice, failed]) { const outcome = copyLeaves(...); if (outcome.isErr()) failed.set(true); return outcome; }
/// ```
fn copy_slice_and_flag_failure(
    (source_root, stage_root, slice, failed): (&Path, &Path, &[WorktreeCopyEntry], &AtomicBool),
) -> Result<Vec<Option<FileStamp>>, WorktreeCopyError> {
    let outcome: Result<Vec<Option<FileStamp>>, WorktreeCopyError> = copy_leaves(source_root, stage_root, slice, failed);
    if outcome.is_err() {
        failed.store(true, Ordering::Relaxed);
    }
    return outcome;
}

/// What:
///  The parent directory of a native path,
/// which always exists for a validated path under a root.
/// Why:
///   Selected nested files can need parents the selection did not name,
/// so each file creates its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dirname(path)
/// ```
fn parent_of(path: &Path) -> PathBuf {
    return match path.parent() {
        Some(parent) => parent.to_path_buf(),
        None => path.to_path_buf(),
    };
}

/// What:
///  Creates a symbolic link with the exact target text.
/// Why:
///   The target is never resolved,
/// so a link to a missing or outside path is copied as-is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(target, stagePath);
/// ```
#[cfg(unix)]
fn symlink_at(target: &Path, link: &Path) -> Result<(), WorktreeCopyError> {
    return std::os::unix::fs::symlink(target, link).map_err(raw_failure_of);
}

/// What:
///  Creates a symbolic link on a platform where this lane does not stage links.
/// Why:
///   Links are a Unix feature in the copy contract;
/// other platforms refuse them with the generic sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(target, stagePath);
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn symlink_at(_target: &Path, _link: &Path) -> Result<(), WorktreeCopyError> {
    return Err(raw_failure(&std::io::Error::other(
        "symbolic links are not staged on this platform",
    )));
}
