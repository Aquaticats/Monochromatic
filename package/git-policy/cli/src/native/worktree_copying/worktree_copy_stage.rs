//! What:
//!  The private, validated copy of one source's ignored state,
//!  staged in a mode-`0700` directory beside a destination worktree.
//! Why:
//!  The stage sits on the destination's filesystem, so a file clone stays local and the install is a rename or a link.
//!  The source is copied entry by entry and compared again at the end,
//!  so a source that changed during the copy never reaches the destination.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const snapshot = await stageIgnoredSnapshot({ sourceRoot, destinationRoot, registeredRoots, gitPath });
//! ```

use crate::scoped_start::start_or_run;
use crate::private_storage::create_private_directory;
use crate::parallel_budget::{HostResources, sliced_workers};
use crate::random_id::random_uuid;
use crate::transaction_git::GitContext;
use crate::diagnostic_log::debug;
use crate::worktree_copy_compare::{FinalCheck, assert_final_source_equivalence_pruned};
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, wrap_raw};
use crate::worktree_copy_manifest::{
    EntryKind, FileStamp, WorktreeCopyEntry, apply_entry_modes, collect_pruned_manifest, stamp_of,
};
use crate::worktree_copy_prune::Prune;
use crate::worktree_copy_selection::{CopySelection, STAGE_PREFIX, filesystem_path, select_roots};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::json_record::quote;
use std::path::{Path, PathBuf};

/// What:
///  The name of the payload directory inside a stage container.
/// Why:
///   The payload mirrors the source's repository paths, so the container can hold other private state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(stageContainer, 'payload')
/// ```
const PAYLOAD_NAME: &str = "payload";

/// What:
///  The validated private snapshot of one source, ready to install.
/// Why:
///   Installation needs the entries it will create, the roots they came from and the stage to copy from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedWorktreeSnapshot = { entries; selectedRoots; sourceRoot; stageContainer; stageRoot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedSnapshot {
    /// Parent-first manifest that was staged and verified.
    pub entries: Vec<WorktreeCopyEntry>,
    /// Git-selected ignored roots that the manifest walked.
    pub selected_roots: Vec<String>,
    /// Canonical source worktree.
    pub source_root: PathBuf,
    /// Private container holding the payload; removed after installation.
    pub stage_container: PathBuf,
    /// Payload root mirroring the source's repository paths.
    pub stage_root: PathBuf,
    /// Repository paths the user could not read, left out of the stage;
    ///  empty for a stage rebuilt from a journal.
    pub unreadable: Vec<String>,
}

/// What:
///  Creates one private directory with mode `0700`, parents included.
/// Why:
///   Staged content is never readable by other users, even before installation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
#[cfg(unix)]
fn create_private_tree(path: &Path) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::DirBuilderExt;
    return std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(|error| return raw_failure(&error));
}

/// What:
///  Creates one directory tree on a platform without Unix modes.
/// Why:
///   Keeps the staging code identical on every platform; no mode is claimed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn create_private_tree(path: &Path) -> Result<(), WorktreeCopyError> {
    return std::fs::create_dir_all(path).map_err(|error| raw_failure(&error));
}

/// What:
///  Asks the filesystem to clone the source's content into a new destination file, when it can.
/// Why:
///   A clone shares storage on copy-on-write filesystems; any refusal means the caller must copy bytes instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, destination, constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE)
/// ```
#[cfg(target_os = "linux")]
fn try_clone(source: &std::fs::File, destination: &std::fs::File) -> bool {
    // The one `FICLONE` call lives with the other Linux filesystem primitives, which the cheap-worktree ladder shares.
    return crate::fs_clone_exchange::clone_file(source, destination, std::path::Path::new("")).is_ok();
}

/// What:
///  Declines cloning on platforms where this lane has no clone request, so every copy takes the byte path.
/// Why:
///   The incumbent's fallback is a full copy, which is always correct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, destination, constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE)
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn try_clone(_source: &std::fs::File, _destination: &std::fs::File) -> bool {
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
///   Filesystem timestamps have a granularity; a write that lands in the same tick as the stamp would leave it equal,
///  so a file changed this recently is compared byte for byte instead of trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RECENT_CHANGE_NANOS = 20_000_000n;
/// ```
const RECENT_CHANGE_NANOS: i128 = 20_000_000;

/// What:
///  Whether a stamp is old enough, against the clock now, to prove that no write follows it unseen.
/// Why:
///   A change time in the future or within the timestamp granularity of now proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// now - ctime >= RECENT_CHANGE_NANOS
/// ```
fn stamp_is_settled(stamp: &FileStamp) -> bool {
    let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return false;
    };
    let now_nanos: i128 = i128::from(now.as_secs()) * 1_000_000_000 + i128::from(now.subsec_nanos());
    let changed_nanos: i128 = i128::from(stamp.changed.0) * 1_000_000_000 + i128::from(stamp.changed.1);
    return now_nanos - changed_nanos >= RECENT_CHANGE_NANOS;
}

/// What:
///  Creates a destination file exclusively and fills it with the source's bytes,
///  using the clone request the caller supplies, and returns the source's stamp when a clone proved the bytes.
/// Why:
///   Whether the filesystem clones is the machine's choice, so a test supplies a clone that accepts or refuses
///  and sees each path taken on every machine. A clone shares the source's blocks as of one instant; a source stamp that is
///  equal before and after it and old enough to be settled proves no write interleaved, and the final check compares it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stamp = await copyFileWith({ sourcePath, stagePath, clone }); // undefined when bytes were copied
/// ```
fn copy_file_exclusive_with(
    source: &Path,
    destination: &Path,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<Option<FileStamp>, WorktreeCopyError> {
    let mut input: std::fs::File =
        std::fs::File::open(source).map_err(|error| return raw_failure(&error))?;
    let mut output: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| return raw_failure(&error))?;
    let before: Option<FileStamp> = input.metadata().ok().as_ref().and_then(stamp_of);
    // A successful clone already holds every byte, so only a refused clone copies them.
    if !clone(&input, &output) {
        std::io::copy(&mut input, &mut output).map_err(|error| return raw_failure(&error))?;
        return Ok(None);
    }
    let after: Option<FileStamp> = input.metadata().ok().as_ref().and_then(stamp_of);
    return Ok(match (before, after) {
        (Some(first), Some(second)) if first == second && stamp_is_settled(&second) => Some(second),
        _ => None,
    });
}

/// What:
///  The entries below which a copy stays on one thread: `PARALLEL_BREAK_EVEN_ENTRIES`.
/// Why:
///   Starting threads costs more than a very small copy. Measured with `copy_manifest_entries` itself forced to 1, 2, 4 and 8
///  workers on reflinked trees of this repository (2026-11-03, five runs each in alternating orders, other sessions running, a
///  debug build): 8 workers took 0.7 ms against 1.1 ms for 30 entries, 1.9 ms against 3.6 ms for 90, 5.9 ms against 13.4 ms for
///  296, 25.4 ms against 91.4 ms for 2,119, 57.3 ms against 202.6 ms for 4,722, and 104.4 ms against 397.6 ms for 9,301, so the
///  old threshold of 4096 kept copies of up to 4,095 entries on one thread for up to 145 ms more. Picked by the coordinator under the owner's delegation, open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Picked for these follow-ups", option A).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_COPY_MIN_ENTRIES = PARALLEL_BREAK_EVEN_ENTRIES;
/// ```
const PARALLEL_COPY_MIN_ENTRIES: usize = crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES;

/// What:
///  The most threads one copy starts.
/// Why:
///   Cloning is a metadata operation bound by the filesystem's locks more than by CPU, and other sessions share this host.
///  Measured with `clone_tree` (`package_repair_tree_clone.rs`, the same `copy_file_exclusive` primitive as the copy stage) on trees of reflinked tracked files of this repository, medians of three runs, 28 threads, other sessions running, 2026-11-03: at 22,718 entries 1, 2, 4, 8 and 16 workers took 1,549, 880, 555, 397 and 415 ms (three runs each, the
///  spread of 16 workers 383 to 444 ms), at 9,301 entries 619, 363, 236, 183 and 194 ms, and at 4,723 entries 330, 190, 124, 99
///  and 106 ms, so more than eight workers were never faster. Kept at eight; the coordinator's pick under the owner's delegation, open to the owner's veto.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_COPY_MAX_WORKERS = 8;
/// ```
pub(crate) const PARALLEL_COPY_MAX_WORKERS: usize = 8;

/// What:
///  How many threads a copy of this many entries uses: one below the threshold, else the host's budget (the CPUs the process may
///  use less a margin, and the free memory), capped.
/// Why:
///   The bound follows the CPUs the process is allowed (a container's quota included), never the whole host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries < MIN ? 1 : Math.min(availableParallelism(), MAX)
/// ```
fn copy_worker_count(entries: usize) -> usize {
    return copy_worker_count_on(&HostResources::read(), entries);
}

/// `copy_worker_count` with the host's resources chosen by the caller, so a control states a host of any size.
fn copy_worker_count_on(host: &HostResources, entries: usize) -> usize {
    if entries < PARALLEL_COPY_MIN_ENTRIES {
        return 1;
    }
    return sliced_workers(entries, PARALLEL_COPY_MAX_WORKERS, host);
}

/// What:
///  Copies the files and links of one slice of a manifest into the stage, recording each cloned file's stamp.
/// Why:
///   Each entry is recreated from the source's no-follow view, so no symbolic link is ever followed;
///  directories exist already, and a parent is created only when the entry's parent changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyLeaves({ sourceRoot, stageRoot, entries, stamps, failed }): Promise<void>;
/// ```
fn copy_leaves(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
    stamps: &mut [Option<FileStamp>],
    failed: &AtomicBool,
) -> Result<(), WorktreeCopyError> {
    let mut last_parent: Option<PathBuf> = None;
    for (entry, slot) in entries.iter().zip(stamps.iter_mut()) {
        if failed.load(Ordering::Relaxed) {
            return Ok(());
        }
        if entry.kind == EntryKind::Directory {
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
            EntryKind::Directory => {}
            EntryKind::File => {
                *slot = copy_file_exclusive_with(source_path.as_path(), stage_path.as_path(), try_clone)?;
            }
            EntryKind::Symlink => {
                let target: PathBuf = std::fs::read_link(source_path.as_path())
                    .map_err(|error| return raw_failure(&error))?;
                symlink_at(target.as_path(), stage_path.as_path())?;
            }
        }
    }
    return Ok(());
}

/// What:
///  Copies one manifest's entries into the stage: directories first, then files and links in parallel slices.
/// Why:
///   The manifest is parent first, so every directory exists before a worker needs it; the slices are contiguous so
///  neighbouring files share a directory, and the first failure stops the other workers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyManifestEntries({ sourceRoot, stageRoot, entries }): Promise<(FileStamp | undefined)[]>;
/// ```
fn copy_manifest_entries(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
) -> Result<Vec<Option<FileStamp>>, WorktreeCopyError> {
    for entry in entries.iter().filter(|entry| return entry.kind == EntryKind::Directory) {
        let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_str())?;
        create_private_tree(stage_path.as_path())?;
    }
    let mut stamps: Vec<Option<FileStamp>> = vec![None; entries.len()];
    let workers: usize = copy_worker_count(entries.len());
    let failed: AtomicBool = AtomicBool::new(false);
    if workers <= 1 {
        copy_leaves(source_root, stage_root, entries, stamps.as_mut_slice(), &failed)?;
        return Ok(stamps);
    }
    let slice_length: usize = entries.len().div_ceil(workers);
    let outcomes: Vec<Result<(), WorktreeCopyError>> = std::thread::scope(|scope| {
        let handles: Vec<_> = entries
            .chunks(slice_length)
            .zip(stamps.chunks_mut(slice_length))
            .map(|(slice, slots)| {
                let failed_ref: &AtomicBool = &failed;
                return start_or_run(scope, move || {
                    let outcome = copy_leaves(source_root, stage_root, slice, slots, failed_ref);
                    if outcome.is_err() {
                        failed_ref.store(true, Ordering::Relaxed);
                    }
                    return outcome;
                });
            })
            .collect();
        return handles
            .into_iter()
            .map(|handle| {
                return handle.join().unwrap_or_else(|_panic| {
                    return Err(raw_failure(&std::io::Error::other("a copy worker panicked")));
                });
            })
            .collect();
    });
    for outcome in outcomes {
        outcome?;
    }
    return Ok(stamps);
}

/// What:
///  The parent directory of a native path, which always exists for a validated path under a root.
/// Why:
///   Selected nested files can need parents the selection did not name, so each file creates its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dirname(path)
/// ```
fn parent_of(path: &Path) -> PathBuf {
    return path
        .parent()
        .map_or_else(|| return path.to_path_buf(), Path::to_path_buf);
}

/// What:
///  Creates a symbolic link with the exact target text.
/// Why:
///   The target is never resolved, so a link to a missing or outside path is copied as-is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(target, stagePath);
/// ```
#[cfg(unix)]
fn symlink_at(target: &Path, link: &Path) -> Result<(), WorktreeCopyError> {
    return std::os::unix::fs::symlink(target, link).map_err(|error| return raw_failure(&error));
}

/// What:
///  Creates a symbolic link on a platform where this lane does not stage links.
/// Why:
///   Links are a Unix feature in the copy contract; other platforms refuse them with the generic sentence.
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

/// What:
///  The registered roots strictly nested under the source, and the stage container, as exclusions.
/// Why:
///   A nested worktree or the stage itself must never be walked as ignored state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sourceExclusions({ sourceRoot, registeredRoots, stageContainer })
/// ```
fn source_exclusions(
    source_root: &Path,
    registered_roots: &[PathBuf],
    stage_container: &Path,
) -> Vec<PathBuf> {
    let mut exclusions: Vec<PathBuf> = registered_roots.to_vec();
    exclusions.push(stage_container.to_path_buf());
    return nested_under(source_root, exclusions.as_slice());
}

/// What:
///  The roots strictly inside the source root, never the source root itself.
/// Why:
///   A registered worktree nested in the source is another checkout's state, so the source's own walk must skip it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roots.filter((root) => root !== sourceRoot && isInside(sourceRoot, root))
/// ```
fn nested_under(source_root: &Path, roots: &[PathBuf]) -> Vec<PathBuf> {
    return roots
        .iter()
        .filter(|root| return root.as_path() != source_root && root.starts_with(source_root))
        .cloned()
        .collect();
}

/// What:
///  Whether a repository path names a private stage at any component.
/// Why:
///   A private stage is never selected, even when Git lists it as ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryPath.split('/').some((component) => component.startsWith(STAGE_PREFIX))
/// ```
fn names_private_stage(repository_path: &str) -> bool {
    return repository_path
        .split('/')
        .any(|component| return component.starts_with(STAGE_PREFIX));
}

/// What:
///  Names a fresh stage container beside a destination worktree, creating nothing.
/// Why:
///   The copy records the name durably before it creates the container, so an interrupted copy can be cleaned up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(dirname(destinationRoot), `${STAGE_PREFIX}${randomUUID()}`)
/// ```
pub fn new_stage_container(destination_root: &Path) -> Result<PathBuf, WorktreeCopyError> {
    let destination_parent: &Path = destination_root.parent().unwrap_or(destination_root);
    let unique: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    return Ok(destination_parent.join(format!("{STAGE_PREFIX}{unique}")));
}

/// What:
///  Stages every selected ignored entry of one source into a named stage container.
/// Why:
///   The stage is removed on any failure, so a failed copy leaves nothing behind for a later invocation to replay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stageIgnoredSnapshot({ sourceRoot, stageContainer, registeredRoots, gitPath }): Promise<StagedWorktreeSnapshot>;
/// ```
pub fn stage_ignored_snapshot(
    context: &GitContext,
    source_root: &Path,
    stage_container: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
) -> Result<StagedSnapshot, WorktreeCopyError> {
    let selecting: std::time::Instant = std::time::Instant::now();
    let nested: Vec<PathBuf> = nested_under(source_root, registered_roots);
    let git_selected: Vec<String> = select_roots(context, source_root, nested.as_slice(), selection)?;
    debug(
        "worktreeCopyStage",
        format!("selected {} ignored roots in {} ms", git_selected.len(), selecting.elapsed().as_millis()).as_str(),
    );
    create_private_directory(stage_container).map_err(|error| return raw_failure(&error))?;
    let stage_root: PathBuf = stage_container.join(PAYLOAD_NAME);
    create_private_tree(stage_root.as_path())?;
    let staged: Result<StagedSnapshot, WorktreeCopyError> = stage_into(
        source_root,
        registered_roots,
        stage_container,
        stage_root.as_path(),
        git_selected,
        selection.prune(),
    );
    // Any failure removes the whole container, so no partial stage survives the attempt.
    match staged {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => {
            // A failed removal replaces the copy failure, as the incumbent's cleanup throws its own error there.
            std::fs::remove_dir_all(stage_container)
                .map_err(|removal| return raw_failure(&removal))?;
            return Err(words_for_source(error, source_root));
        }
    }
}

/// What:
///  Words a raw failure of the staging steps with the source it was copying.
/// Why:
///   The user learns which worktree's ignored state could not be staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`cli-git: could not stage ignored state from ${JSON.stringify(sourceRoot)}.`, error)
/// ```
fn words_for_source(error: WorktreeCopyError, source_root: &Path) -> WorktreeCopyError {
    let message: String = format!(
        "cli-git: could not stage ignored state from {}.",
        quote(source_root.to_string_lossy().as_ref())
    );
    return wrap_raw(error, message.as_str());
}

/// What:
///  The copy steps that run once the private stage exists.
/// Why:
///   Separating them lets one cleanup cover every failure after the container is created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { entries = await collectEntryManifest(...); await copyManifestEntries(...); await applyEntryModes(...); await assertFinalSourceEquivalence(...); } catch { await rm(stageContainer, ...); }
/// ```
fn stage_into(
    source_root: &Path,
    registered_roots: &[PathBuf],
    stage_container: &Path,
    stage_root: &Path,
    git_selected: Vec<String>,
    prune: Prune,
) -> Result<StagedSnapshot, WorktreeCopyError> {
    let excluded_source_roots: Vec<PathBuf> =
        source_exclusions(source_root, registered_roots, stage_container);
    let candidate_roots: Vec<String> = git_selected
        .into_iter()
        .filter(|repository_path| return !names_private_stage(repository_path.as_str()))
        .filter(|repository_path| return !prune.skips_path(repository_path.as_str()))
        .filter(|repository_path| {
            let source_path: PathBuf = match filesystem_path(source_root, repository_path.as_str())
            {
                Ok(path) => path,
                Err(_) => return false,
            };
            return !excluded_source_roots
                .iter()
                .any(|excluded| return source_path.starts_with(excluded.as_path()));
        })
        .collect();
    let started: std::time::Instant = std::time::Instant::now();
    let mut unreadable: Vec<String> = Vec::new();
    let entries: Vec<WorktreeCopyEntry> = collect_pruned_manifest(
        source_root,
        candidate_roots.as_slice(),
        excluded_source_roots.as_slice(),
        prune,
        &mut unreadable,
    )?;
    // A root the walk left out whole (another checkout, an unreadable directory) has no entry and is no root of the stage.
    let staged_paths: std::collections::HashSet<&str> =
        entries.iter().map(|entry| return entry.relative_path.as_str()).collect();
    let selected_roots: Vec<String> = candidate_roots
        .into_iter()
        .filter(|repository_path| return staged_paths.contains(repository_path.as_str()))
        .collect();
    drop(staged_paths);
    if !unreadable.is_empty() {
        // The person gets the count and the first paths in one notice line; the verbose log keeps the whole list.
        debug(
            "worktreeCopyStage",
            format!(
                "{} unreadable ignored entries are left out of the copy from {}: {}",
                unreadable.len(),
                source_root.display(),
                unreadable.join(", ")
            )
            .as_str(),
        );
    }
    let walked: std::time::Duration = started.elapsed();
    let stamps: Vec<Option<FileStamp>> =
        copy_manifest_entries(source_root, stage_root, entries.as_slice())?;
    let copied: std::time::Duration = started.elapsed();
    apply_entry_modes(stage_root, entries.as_slice())?;
    let moded: std::time::Duration = started.elapsed();
    assert_final_source_equivalence_pruned(
        source_root,
        stage_root,
        selected_roots.as_slice(),
        excluded_source_roots.as_slice(),
        entries.as_slice(),
        &FinalCheck { prune, stamps: stamps.as_slice() },
    )?;
    debug(
        "worktreeCopyStage",
        format!(
            "staged {} entries ({} cloned under a stamp) in {} ms: walk {} ms, copy by {} workers {} ms, modes {} ms, final check {} ms",
            entries.len(),
            stamps.iter().flatten().count(),
            started.elapsed().as_millis(),
            walked.as_millis(),
            copy_worker_count(entries.len()),
            (copied - walked).as_millis(),
            (moded - copied).as_millis(),
            (started.elapsed() - moded).as_millis()
        )
        .as_str(),
    );
    return Ok(StagedSnapshot {
        entries,
        selected_roots,
        source_root: source_root.to_path_buf(),
        stage_container: stage_container.to_path_buf(),
        stage_root: stage_root.to_path_buf(),
        unreadable,
    });
}

/// Stage controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_tests.rs"]
mod tests;
