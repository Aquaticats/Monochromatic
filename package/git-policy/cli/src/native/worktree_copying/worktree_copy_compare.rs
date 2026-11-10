//! What:
//!  Exact comparison of one manifest entry against the filesystem:
//!  its kind,
//! its retained mode,
//! its symbolic-link target text or its regular-file bytes.
//!  Also the final check that the source still matches the stage after staging.
//! Why:
//!  A destination must receive exactly what was staged from the source,
//!  and a source that changed during the transfer must block the destination.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await assertFinalSourceEquivalence({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries });
//! ```

use crate::side_by_side_on_the_runtime::run_beside_the_caller_on_the_runtime;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure, raw_failure_of};
use crate::worktree_copy_manifest::{
    EntryKind, FileStamp, WorktreeCopyEntry, collect_entry_manifest, collect_pruned_manifest, retained_mode, stamp_of,
};
use crate::worktree_copy_prune::Prune;
use crate::worktree_copy_selection::{filesystem_path, shown_repository_path};
use crate::json_record::quote;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};

/// What:
///  The size of one comparison read.
/// Why:
///   Large files are compared in bounded chunks so memory does not grow with file size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMPARE_BUFFER_BYTES = 64 * 1_024;
/// ```
///  Exempt from a citation under `LQM` (class:
///  buffer size):
///  it sizes one read buffer,
///  and the result of the comparison
///  does not depend on it.
///  Classed once in `doc/planning/cli-git-shipped-limits-audit.md`,
///  "Exempt limits";
///  the
///  coordinator's pick under the owner's delegation,
///  open to the owner's veto.
pub(crate) const COMPARE_BUFFER_BYTES: usize = 64 * 1_024;

// The metadata question a comparison asks of both sides is the one every walk of a live checkout asks, so it has one
//  home (`worktree_copy_vanished_entry.rs`): a missing entry is an answer, and every other failure is not.
pub use crate::worktree_copy_vanished_entry::lstat_or_absent;

/// What:
///  Whether two regular files hold the same bytes,
/// compared in bounded chunks.
/// Why:
///   Sizes are compared first,
/// and a short read means the file changed under the comparison.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function regularFileBytesEqual({ leftPath, rightPath }): Promise<boolean>;
/// ```
fn regular_file_bytes_equal(
    left_path: &Path,
    right_path: &Path,
) -> Result<bool, WorktreeCopyError> {
    let mut left: std::fs::File =
        std::fs::File::open(left_path).map_err(raw_failure_of)?;
    let mut right: std::fs::File =
        std::fs::File::open(right_path).map_err(raw_failure_of)?;
    let left_size: u64 = left
        .metadata()
        .map_err(raw_failure_of)?
        .len();
    let right_size: u64 = right
        .metadata()
        .map_err(raw_failure_of)?
        .len();
    if left_size != right_size {
        return Ok(false);
    }
    let mut left_buffer: Vec<u8> = vec![0; COMPARE_BUFFER_BYTES];
    let mut right_buffer: Vec<u8> = vec![0; COMPARE_BUFFER_BYTES];
    let mut remaining: u64 = left_size;
    while remaining > 0 {
        let length: usize = match usize::try_from(remaining) {
            Ok(size) => size.min(COMPARE_BUFFER_BYTES),
            Err(_too_large) => COMPARE_BUFFER_BYTES,
        };
        let left_chunk: &mut [u8] = &mut left_buffer[..length];
        let right_chunk: &mut [u8] = &mut right_buffer[..length];
        // `read_exact` reports a file that ended early as `UnexpectedEof`, which is a difference, not a failure.
        match (left.read_exact(left_chunk), right.read_exact(right_chunk)) {
            (Ok(()), Ok(())) => {}
            (Err(error), _) | (_, Err(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                return Ok(false);
            }
            (Err(error), _) | (_, Err(error)) => {
                return Err(raw_failure(&error));
            }
        }
        if left_chunk != right_chunk {
            return Ok(false);
        }
        remaining -= u64::try_from(length).unwrap_or(0);
    }
    return Ok(true);
}

/// What:
///  The symbolic-link target text of one no-follow entry.
/// Why:
///   The target is compared as exact text,
/// never resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readlink(path)
/// ```
fn link_target(path: &Path) -> Result<PathBuf, WorktreeCopyError> {
    return std::fs::read_link(path).map_err(raw_failure_of);
}

/// What:
///  Whether one actual entry matches an expected entry exactly.
/// Why:
///   Kind,
/// mode and content must all match,
/// and an absent entry never matches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function entryMatches({ expectedRoot, actualRoot, entry }): Promise<boolean>;
/// ```
pub fn entry_matches(
    expected_root: &Path,
    actual_root: &Path,
    entry: &WorktreeCopyEntry,
) -> Result<bool, WorktreeCopyError> {
    return entry_matches_stamped(expected_root, actual_root, entry, None);
}

/// What:
///  Whether one actual entry matches an expected entry,
/// taking an unchanged stamp of the actual file as proof of its bytes.
/// Why:
///   A file the stage cloned holds the very blocks the source held at the clone;
/// when the source's stamp is still the one
///  recorded around the clone,
/// no write reached it since,
/// so reading both files again could only repeat the clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function entryMatchesStamped({ expectedRoot, actualRoot, entry, stamp }): Promise<boolean>;
/// ```
pub fn entry_matches_stamped(
    expected_root: &Path,
    actual_root: &Path,
    entry: &WorktreeCopyEntry,
    stamp: Option<&FileStamp>,
) -> Result<bool, WorktreeCopyError> {
    let expected_path: PathBuf = filesystem_path(expected_root, entry.relative_path.as_slice())?;
    let actual_path: PathBuf = filesystem_path(actual_root, entry.relative_path.as_slice())?;
    let Some(actual) = lstat_or_absent(actual_path.as_path())? else {
        return Ok(false);
    };
    if retained_mode(&actual) != entry.mode {
        return Ok(false);
    }
    // `match` on the entry kind selects the one comparison that applies to it.
    match entry.kind {
        EntryKind::Directory => return Ok(actual.is_dir()),
        EntryKind::File => {
            if !actual.is_file() {
                return Ok(false);
            }
            if let Some(recorded) = stamp {
                return Ok(stamp_of(&actual).as_ref() == Some(recorded));
            }
            return regular_file_bytes_equal(expected_path.as_path(), actual_path.as_path());
        }
        EntryKind::Symlink => {
            if !actual.file_type().is_symlink() {
                return Ok(false);
            }
            return Ok(link_target(expected_path.as_path())? == link_target(actual_path.as_path())?);
        }
    }
}

/// What:
///  The manifest of the source under the leave-out rule,
/// as the final check walks it on a job of the blocking pool beside the stage's walk.
/// Why:
///   Entries the walk cannot read are not reported here:
/// the final check compares the whole manifest and fails on any difference.
/// The walk is a wait on storage,
/// so it belongs on the blocking pool and never on a worker that polls tasks,
/// and a job of the pool owns what it reads,
/// so the root and the two lists travel as owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// walkSourcePruned([sourceRoot, selectedRoots, excludedSourceRoots, prune]) // collectPrunedManifest with a discarded unreadable list
/// ```
fn walk_source_pruned(
    (source_root, selected_roots, excluded_source_roots, prune): (PathBuf, Vec<Vec<u8>>, Vec<PathBuf>, Prune),
) -> Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> {
    let mut unreadable: Vec<Vec<u8>> = Vec::new();
    return collect_pruned_manifest(
        source_root.as_path(),
        selected_roots.as_slice(),
        excluded_source_roots.as_slice(),
        prune,
        &mut unreadable,
    );
}

/// What:
///  The manifest of the stage,
/// with no leave-out rule,
/// as the final check walks it beside the source's walk.
/// Why:
///   The stage holds exactly the selected roots,
/// so nothing is excluded or pruned,
///  and the walk is a wait on storage,
/// which a job of the blocking pool does with the root it owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// walkStage([stageRoot, selectedRoots]) // collectEntryManifest(stageRoot, selectedRoots, [])
/// ```
fn walk_stage(
    (stage_root, selected_roots): (PathBuf, Vec<Vec<u8>>),
) -> Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> {
    return collect_entry_manifest(stage_root.as_path(), selected_roots.as_slice(), &[]);
}

/// What:
///  Checks that the source still matches the initial manifest after staging,
///  and that the stage holds exactly that manifest.
/// Why:
///   Any difference means the source changed during the copy,
/// so the destination must not receive the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await assertFinalSourceEquivalence({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries });
/// ```
pub fn assert_final_source_equivalence(
    source_root: &Path,
    stage_root: &Path,
    selected_roots: &[Vec<u8>],
    excluded_source_roots: &[PathBuf],
    staged_entries: &[WorktreeCopyEntry],
) -> Result<(), WorktreeCopyError> {
    return assert_final_source_equivalence_pruned(
        source_root,
        stage_root,
        selected_roots,
        excluded_source_roots,
        staged_entries,
        &FinalCheck { prune: Prune::Never, stamps: &[] },
    );
}

/// What:
///  The leave-out rule the stage used and the stamps recorded around each clone.
/// Why:
///   The final check walks the source with the same rule,
/// and trusts a stamp only for the entry it was recorded for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FinalCheck = { prune: Prune; stamps: readonly (FileStamp | undefined)[] };
/// ```
pub struct FinalCheck<'a> {
    /// The rule the staged walk used.
    pub prune: Prune,
    /// One slot per staged entry;
    /// empty when no entry was cloned under a stamp.
    pub stamps: &'a [Option<FileStamp>],
}

/// What:
///  Checks that the source still matches the initial manifest after staging,
/// with the staged walk's leave-out rule,
///  and that the stage holds exactly that manifest.
/// Why:
///   Any difference means the source changed during the copy,
/// so the destination must not receive the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await assertFinalSourceEquivalencePruned({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries, check });
/// ```
pub fn assert_final_source_equivalence_pruned(
    source_root: &Path,
    stage_root: &Path,
    selected_roots: &[Vec<u8>],
    excluded_source_roots: &[PathBuf],
    staged_entries: &[WorktreeCopyEntry],
    check: &FinalCheck<'_>,
) -> Result<(), WorktreeCopyError> {
    return match crate::lazy_runtime::run_on_runtime(assert_final_source_equivalence_pruned_awaiting(
        source_root,
        stage_root,
        selected_roots,
        excluded_source_roots,
        staged_entries,
        check,
    )) {
        Ok(outcome) => outcome,
        // Neither walk ran, and the reason the runtime could not make them is why.
        Err(refusal) => Err(failure_with_cause(
            format!("monochromatic-git-policy-cli: the source could not be checked after staging ({refusal}).").as_str(),
            None,
        )),
    };
}

/// What:
///  `assert_final_source_equivalence_pruned` on the process's runtime:
/// the source's walk is one job of the blocking pool and the stage's walk runs beside it.
/// Why:
///   A caller that is itself a task of the runtime cannot enter it,
/// so it awaits this face instead;
/// the walks take as long as the slower of the two,
///  and both are storage waits rather than work on a worker that polls tasks.
/// A walk that panicked is the failure the scoped thread's panic was.
/// The verdict is this branch's:
/// every difference is named by relative path,
/// so a caller that may
///  tolerate one knows which one it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function assertFinalSourceEquivalencePruned({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries, check }): Promise<void>;
/// ```
async fn assert_final_source_equivalence_pruned_awaiting(
    source_root: &Path,
    stage_root: &Path,
    selected_roots: &[Vec<u8>],
    excluded_source_roots: &[PathBuf],
    staged_entries: &[WorktreeCopyEntry],
    check: &FinalCheck<'_>,
) -> Result<(), WorktreeCopyError> {
    let differences: Differences = differences_from_the_source(source_root, stage_root, selected_roots, excluded_source_roots, staged_entries, check).await?;
    if !differences.gone.is_empty() || !differences.appeared.is_empty() {
        return Err(failure_with_cause(
            "monochromatic-git-policy-cli: ignored source paths changed while snapshot was staged.",
            None,
        ));
    }
    if let Some(first) = differences.changed.first() {
        let message: String = format!(
            "monochromatic-git-policy-cli: ignored source entry changed while copying: {}.",
            quote(shown_repository_path(first.as_slice()).as_str())
        );
        return Err(failure_with_cause(message.as_str(), None));
    }
    return Ok(());
}

/// What:
///  What the source and the stage walks say about the manifest the copy made,
/// by relative path.
/// Why:
///   The caller has to know which entries went away,
/// which ones changed under the copy,
/// and which ones appeared after the listing:
/// a vanished entry has nothing left to copy,
/// an entry that
///  keeps changing is retried and then left in its source,
/// and an entry that appeared was never in the listing.
/// Paths,
///  not positions,
/// are what makes the three tellable apart once the walks disagree on how many entries there are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Differences = { gone: Uint8Array[]; changed: Uint8Array[]; appeared: Uint8Array[] };
/// ```
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Differences {
    /// Manifest entries the source no longer holds,
    /// as the bytes of each name.
    pub gone: Vec<Vec<u8>>,
    /// Manifest entries whose shape or bytes the source and the stage disagree about,
    /// as the bytes of each name.
    pub changed: Vec<Vec<u8>>,
    /// Source paths the listing did not name,
    /// which appeared while the copy ran,
    /// as the bytes of each name.
    pub appeared: Vec<Vec<u8>>,
}

/// What:
///  Walk the source and the stage beside each other and compare them with the manifest,
/// by relative
///  path.
/// Why:
///   Two walks taken at one instant are the only evidence the stage matches the source,
/// and a
///  difference has to be named by its path so the caller can retry or leave out exactly that entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function differencesFromTheSource(...): Differences;
/// ```
pub async fn differences_from_the_source(
    source_root: &Path,
    stage_root: &Path,
    selected_roots: &[Vec<u8>],
    excluded_source_roots: &[PathBuf],
    staged_entries: &[WorktreeCopyEntry],
    check: &FinalCheck<'_>,
) -> Result<Differences, WorktreeCopyError> {
    // The source and the stage are independent trees, so their two walks run side by side.
    let (source_joined, stage_walked): (Result<Result<Vec<WorktreeCopyEntry>, WorktreeCopyError>, crate::blocking_job::BlockingJobError>, Result<Vec<WorktreeCopyEntry>, WorktreeCopyError>) =
        run_beside_the_caller_on_the_runtime(
            ((source_root.to_path_buf(), selected_roots.to_vec(), excluded_source_roots.to_vec(), check.prune), walk_source_pruned),
            ((stage_root.to_path_buf(), selected_roots.to_vec()), walk_stage),
        )
        .await;
    let source_walked: Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> = match source_joined {
        Ok(walked) => walked,
        Err(crate::blocking_job::BlockingJobError::Panicked) => Err(raw_failure(&std::io::Error::other("a manifest walk panicked"))),
        Err(refused) => Err(raw_failure(&std::io::Error::other(format!("a manifest walk did not run ({refused})")))),
    };
    let final_source: Vec<WorktreeCopyEntry> = source_walked?;
    let final_stage: Vec<WorktreeCopyEntry> = stage_walked?;
    let mut by_source: BTreeMap<&[u8], &WorktreeCopyEntry> = BTreeMap::new();
    for entry in &final_source {
        by_source.insert(entry.relative_path.as_slice(), entry);
    }
    let mut by_stage: BTreeMap<&[u8], &WorktreeCopyEntry> = BTreeMap::new();
    for entry in &final_stage {
        by_stage.insert(entry.relative_path.as_slice(), entry);
    }
    let mut differences: Differences = Differences::default();
    for (index, staged) in staged_entries.iter().enumerate() {
        let path: &[u8] = staged.relative_path.as_slice();
        let Some(source_entry) = by_source.get(path) else {
            differences.gone.push(path.to_vec());
            continue;
        };
        let same_shape: bool = match by_stage.get(path) {
            Some(stage_entry) => {
                source_entry.kind == staged.kind
                    && source_entry.mode == staged.mode
                    && stage_entry.kind == staged.kind
                    && stage_entry.mode == staged.mode
                    && stage_entry.relative_path == staged.relative_path
            }
            None => false,
        };
        let stamp: Option<&FileStamp> = check.stamps.get(index).and_then(Option::as_ref);
        // The content check runs last so the cheap shape checks fail fast without reading large files.
        if !same_shape || !entry_matches_stamped(stage_root, source_root, staged, stamp)? {
            differences.changed.push(path.to_vec());
        }
    }
    let mut staged_paths: BTreeSet<&[u8]> = BTreeSet::new();
    for entry in staged_entries {
        staged_paths.insert(entry.relative_path.as_slice());
    }
    for path in by_source.keys() {
        if !staged_paths.contains(path) {
            differences.appeared.push((*path).to_vec());
        }
    }
    return Ok(differences);
}

/// What:
///  `differences_from_the_source` for a caller that is not a task of the runtime:
/// the walks run on the process's runtime and this face waits for them.
/// Why:
///   The staging of one created worktree's ignored state is synchronous,
/// and its settling loop asks
///  this question once per pass;
/// a caller that is itself a task of the runtime awaits
///  `differences_from_the_source` instead,
/// because a job of the pool is already inside the runtime and
///  the synchronous face refuses there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function differencesFromTheSource({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries, check }): Differences;
/// ```
pub fn differences_from_the_source_sync(
    source_root: &Path,
    stage_root: &Path,
    selected_roots: &[Vec<u8>],
    excluded_source_roots: &[PathBuf],
    staged_entries: &[WorktreeCopyEntry],
    check: &FinalCheck<'_>,
) -> Result<Differences, WorktreeCopyError> {
    return match crate::lazy_runtime::run_on_runtime(differences_from_the_source(
        source_root,
        stage_root,
        selected_roots,
        excluded_source_roots,
        staged_entries,
        check,
    )) {
        Ok(differences) => differences,
        // Neither walk ran, and the reason the runtime could not make them is why.
        Err(refusal) => Err(failure_with_cause(
            format!("monochromatic-git-policy-cli: the source could not be checked after staging ({refusal}).").as_str(),
            None,
        )),
    };
}

/// Compare controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_compare_tests.rs"]
mod tests;
