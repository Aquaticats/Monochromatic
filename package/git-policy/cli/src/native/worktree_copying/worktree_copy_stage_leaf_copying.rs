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
use crate::worktree_copy_error::failure_with_cause;
use crate::worktree_copy_selection::filesystem_path;
use crate::worktree_copy_vanished_entry::lstat_or_absent;
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
/// The source is opened by the plain-file reader,
/// which waits for no writer and follows no link,
///  so a name the walk listed as a file and a dev server replaced with a named pipe between the two passes
///  is refused at once instead of waiting for a writer that may never come,
/// with the checkout held and nothing
///  said ("nothing should ever wait forever or indefinitely").
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
        crate::plain_file::open_plain_file(source).map_err(raw_failure_of)?;
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
///  Copies the files and links of the manifest entries at `positions` into the stage,
/// recording each cloned file's stamp in a list of the slice's own.
/// Why:
///   Each entry is recreated from the source's no-follow view,
/// so no symbolic link is ever followed;
///  directories exist already,
/// and a parent is created only when the entry's parent changes.
///  The stamps come back in the order of the positions rather than into one shared list,
/// because a job of the blocking pool owns what it reads and writes;
/// the caller places them by the slice each position belongs to,
/// so a slice that stopped early cannot shift another slice's stamps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyLeaves({ sourceRoot, stageRoot, entries, positions, failed }): (FileStamp | undefined)[];
/// ```
fn copy_leaves(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
    positions: &[usize],
    failed: &AtomicBool,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<SliceCopy, WorktreeCopyError> {
    let mut last_parent: Option<PathBuf> = None;
    let mut stamps: Vec<Option<FileStamp>> = Vec::with_capacity(positions.len());
    let mut gone: Vec<Vec<u8>> = Vec::new();
    for at in positions {
        let Some(entry) = entries.get(*at) else { continue };
        if failed.load(Ordering::Relaxed) {
            return Ok(SliceCopy { stamps, gone });
        }
        if entry.kind == EntryKind::Directory {
            stamps.push(None);
            continue;
        }
        let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_slice())?;
        let parent: PathBuf = parent_of(stage_path.as_path());
        if last_parent.as_ref() != Some(&parent) {
            create_private_tree(parent.as_path())?;
            last_parent = Some(parent);
        }
        match copy_one_leaf(source_root, stage_path.as_path(), entry, clone)? {
            CopiedLeaf::Staged(stamp) => stamps.push(stamp),
            // The source no longer holds it, so there is nothing to copy and no reason to fail the add.
            CopiedLeaf::Gone => {
                stamps.push(None);
                gone.push(entry.relative_path.clone());
            }
        }
    }
    return Ok(SliceCopy { stamps, gone });
}

/// What:
///  The answer of copying one manifest entry into the stage.
/// Why:
///   A source entry that is no longer there has nothing to copy,
/// and a live checkout is constantly
///  removing and rewriting its ignored files,
/// so that answer is not a failure of the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopiedLeaf = { kind: 'staged'; stamp?: FileStamp } | { kind: 'gone' };
/// ```
pub enum CopiedLeaf {
    /// The entry is in the stage;
    /// the stamp proves the bytes when one was taken from a clone.
    Staged(Option<FileStamp>),
    /// The source no longer holds the entry.
    Gone,
}

/// What:
///  Copy one file or link of the manifest into `stage_path`,
/// and say whether the source still had it.
/// Why:
///   The walk and the copy are separate passes,
/// so a file a build removed between them is gone by now;
///  reading it is what discovers that,
/// and the answer is a skip rather than a failure.
/// An entry that changed kind in between is answered the same way:
/// it holds none of what the
///  listing promised,
/// so the entry the copy listed is no more part of this copy than one that went away.
/// A failure names the source path it
///  could not read,
/// never only the checkout it was copying from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyOneLeaf(sourceRoot, stagePath, entry): CopiedLeaf { try { return { kind: 'staged', stamp: copyFileExclusiveWith(...) }; } catch (e) { if (e.code === 'ENOENT' || stillHoldsTheListedKind(sourcePath, entry.kind) === false) return { kind: 'gone' }; throw `${sourcePath} cannot be copied (${e})`; } }
/// ```
pub fn copy_one_leaf(source_root: &Path, stage_path: &Path, entry: &WorktreeCopyEntry, clone: fn(&std::fs::File, &std::fs::File) -> bool) -> Result<CopiedLeaf, WorktreeCopyError> {
    let source_path: PathBuf = filesystem_path(source_root, entry.relative_path.as_slice())?;
    return match entry.kind {
        EntryKind::Directory => Ok(CopiedLeaf::Staged(None)),
        EntryKind::File => match copy_file_exclusive_with(source_path.as_path(), stage_path, clone) {
            Ok(stamp) => Ok(CopiedLeaf::Staged(stamp)),
            Err(_no_longer_the_listed_entry) if !still_holds_the_listed_kind(&source_path, EntryKind::File) => Ok(CopiedLeaf::Gone),
            Err(error) => Err(named_source_failure(&source_path, "cannot be copied", &error)),
        },
        EntryKind::Symlink => match std::fs::read_link(source_path.as_path()) {
            Ok(target) => {
                symlink_at(target.as_path(), stage_path)?;
                Ok(CopiedLeaf::Staged(None))
            }
            Err(_no_longer_the_listed_entry) if !still_holds_the_listed_kind(&source_path, EntryKind::Symlink) => Ok(CopiedLeaf::Gone),
            Err(error) => Err(named_source_failure(&source_path, "cannot be read", &failure_with_cause(format!("{error}").as_str(), None))),
        },
    };
}

/// What:
///  Whether the source still holds the entry the walk listed,
/// in the kind the walk recorded for it.
/// Why:
///   A file and the directory holding it can both go between the walk and the copy,
/// and only the
///  filesystem knows whether the failure was that,
/// so the question is asked again after the failure rather than guessed
///  from its text.
/// A kind that changed is the same answer as an entry that went away:
/// a named pipe where a
///  file was listed holds none of the bytes the listing promised,
/// and the copy stage's answer stays the same
///  whatever the entry became.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillHoldsTheListedKind = (path, kind) => { try { const found = lstatSync(path); return kind === 'directory' ? found.isDirectory() : kind === 'file' ? found.isFile() : found.isSymbolicLink(); } catch { return false; } };
/// ```
fn still_holds_the_listed_kind(source_path: &Path, kind: EntryKind) -> bool {
    let Ok(Some(metadata)) = lstat_or_absent(source_path) else { return false };
    return match kind {
        EntryKind::Directory => metadata.is_dir(),
        EntryKind::File => metadata.is_file(),
        EntryKind::Symlink => metadata.file_type().is_symlink(),
    };
}

/// What:
///  A failure of one source entry,
/// worded with the path the person can act on.
/// Why:
///   `No such file or directory` names no file,
/// and the checkout it came from holds thousands:
/// the line says which entry failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${path} ${what} (${error})`
/// ```
fn named_source_failure(source_path: &Path, what: &str, error: &WorktreeCopyError) -> WorktreeCopyError {
    let message: String = format!("{} {what} ({})", source_path.display(), error.0);
    return failure_with_cause(message.as_str(), None);
}

/// What:
///  What one slice of the manifest left in the stage,
/// and what it found gone.
/// Why:
///   A job of the blocking pool owns what it writes,
/// so its stamps come back in its own list,
/// which the
///  caller places by the slice each position belongs to;
/// an entry the source no longer holds has no stamp and no
///  stage file,
/// so the job names it instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SliceCopy = { stamps: (FileStamp | undefined)[]; gone: Uint8Array[] };
/// ```
struct SliceCopy {
    /// The stamp of every position of the slice,
    /// in position order.
    stamps: Vec<Option<FileStamp>>,
    /// The entries the source no longer held when their turn came,
    /// as the bytes of each name.
    gone: Vec<Vec<u8>>,
}

/// What:
///  What one copy of the manifest left in the stage and what it found gone.
/// Why:
///   A stamp belongs to the entry it was taken for,
/// and an entry the source no longer holds has no stamp
///  and no stage file;
/// the caller has to know both to compare the stage with the source and to install
///  only what exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopiedManifest = { stamps: (FileStamp | undefined)[]; gone: Uint8Array[] };
/// ```
pub struct CopiedManifest {
    /// The stamp of each entry,
    /// in manifest order;
    /// `None` when the bytes were copied rather than cloned,
    /// and for every entry the source no longer held.
    pub stamps: Vec<Option<FileStamp>>,
    /// The manifest entries the source no longer held when their turn came,
    /// in manifest order,
    /// as the bytes of each name.
    pub gone: Vec<Vec<u8>>,
}

/// What:
///  One slice of the manifest as the state of one job:
/// the entries,
/// the positions of this slice,
/// the two roots,
/// the flag that stops the other workers,
/// and the clone request.
/// Why:
///   A job of the blocking pool owns what it reads,
/// so the manifest the caller lent is shared once here
///  and each job takes its own share of it with the positions it copies,
/// which no job has to copy an entry for.
/// The budget runner takes a function and the state it reads,
/// never a closure,
/// so the slice travels as values the job owns and
///  may be taken again where no thread can start;
/// the clone request travels the same way,
/// so a control can place a write between one entry's copy and the next one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ASliceToCopy = [entries: readonly WorktreeCopyEntry[], source: string, stage: string, positions: number[], failed: boolean, clone: CloneRequest];
/// ```
type ASliceToCopy = (Arc<[WorktreeCopyEntry]>, PathBuf, PathBuf, Vec<usize>, Arc<AtomicBool>, fn(&std::fs::File, &std::fs::File) -> bool);

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
/// function copySliceAndFlagFailure([entries, sourceRoot, stageRoot, positions, failed]) { const outcome = copyLeaves(...); if (outcome.isErr()) failed.set(true); return outcome; }
/// ```
fn copy_slice_and_flag_failure(
    (entries, source_root, stage_root, positions, failed, clone): ASliceToCopy,
) -> Result<SliceCopy, WorktreeCopyError> {
    let outcome: Result<SliceCopy, WorktreeCopyError> = copy_leaves(source_root.as_path(), stage_root.as_path(), entries.as_ref(), positions.as_slice(), failed.as_ref(), clone);
    if outcome.is_err() {
        failed.store(true, Ordering::Relaxed);
    }
    return outcome;
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
/// The answers are this branch's:
/// a slice names the entries the source no longer held,
/// and a failure of one entry
///  carries its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyManifestEntries({ sourceRoot, stageRoot, entries }): CopiedManifest;
/// ```
pub fn copy_manifest_entries(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
) -> Result<CopiedManifest, WorktreeCopyError> {
    return copy_manifest_entries_with(source_root, stage_root, entries, try_clone);
}

/// What:
///  `copy_manifest_entries` with the clone request the caller supplies,
/// so a control can make a source entry change or
///  vanish at a stated moment.
/// Why:
///   The landing and the worktree copy both reach the same primitive,
/// and only a supplied clone lets a
///  control place a write between one entry's copy and the next one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyManifestEntriesWith({ sourceRoot, stageRoot, entries, clone }): CopiedManifest;
/// ```
pub fn copy_manifest_entries_with(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<CopiedManifest, WorktreeCopyError> {
    return match crate::lazy_runtime::run_on_runtime(copy_manifest_entries_awaiting(source_root, stage_root, Arc::from(entries), clone)) {
        Ok(copied) => copied,
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
/// async function copyManifestEntries({ sourceRoot, stageRoot, entries, clone }): Promise<CopiedManifest>;
/// ```
async fn copy_manifest_entries_awaiting(
    source_root: &Path,
    stage_root: &Path,
    entries: Arc<[WorktreeCopyEntry]>,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<CopiedManifest, WorktreeCopyError> {
    for entry in entries.iter() {
        if entry.kind != EntryKind::Directory {
            continue;
        }
        let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_slice())?;
        create_private_tree(stage_path.as_path())?;
    }
    let mut stamps: Vec<Option<FileStamp>> = vec![None; entries.len()];
    let workers: usize = copy_worker_count(entries.len());
    let failed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let slice_length: usize = entries.len().div_ceil(workers.max(1)).max(1);
    let mut states: Vec<ASliceToCopy> = Vec::new();
    let mut slices: Vec<std::ops::Range<usize>> = Vec::new();
    let mut at: usize = 0;
    for chunk in entries.chunks(slice_length) {
        let range: std::ops::Range<usize> = at..at + chunk.len();
        states.push((Arc::clone(&entries), source_root.to_path_buf(), stage_root.to_path_buf(), range.clone().collect(), Arc::clone(&failed), clone));
        slices.push(range);
        at += chunk.len();
    }
    let mut outcomes: Vec<Result<SliceCopy, WorktreeCopyError>> = Vec::new();
    for answer in run_all_side_by_side_on_the_runtime(states, copy_slice_and_flag_failure).await {
        outcomes.push(match answer {
            Ok(outcome) => outcome,
            Err(crate::blocking_job::BlockingJobError::Panicked) => Err(raw_failure(&std::io::Error::other("a copy worker panicked"))),
            Err(refused) => Err(raw_failure(&std::io::Error::other(format!("a copy worker did not run ({refused})")))),
        });
    }
    let mut gone: Vec<Vec<u8>> = Vec::new();
    for (range, outcome) in slices.into_iter().zip(outcomes) {
        let copied: SliceCopy = outcome?;
        // A slice writes the stamps of its own positions, so a slice that stopped early cannot shift another's.
        for (slot, stamp) in stamps[range].iter_mut().zip(copied.stamps) {
            *slot = stamp;
        }
        gone.extend(copied.gone);
    }
    return Ok(CopiedManifest { stamps, gone });
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
