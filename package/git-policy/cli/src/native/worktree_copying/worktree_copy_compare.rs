//! What:
//!  The final check that the source still matches the stage after staging:
//!  two walks taken beside each other,
//!  and the differences they name by relative path.
//! Why:
//!  A source that changed during the transfer must block the destination,
//!  and the settling loop above this one retries or leaves out exactly the entry a difference names.
//!  The comparison of one entry is in `worktree_copy_compare_entry_match.rs`,
//!  the two walks this check starts are in `worktree_copy_compare_walks.rs`,
//!  and what a caller read from here is re-exported so none of them changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await assertFinalSourceEquivalence({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries });
//! ```

use crate::json_record::quote;
use crate::side_by_side_on_the_runtime::run_beside_the_caller_on_the_runtime;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
    raw_failure,
};
use crate::worktree_copy_manifest::{
    FileStamp,
    WorktreeCopyEntry,
};
use crate::worktree_copy_prune::Prune;
use crate::worktree_copy_selection::shown_repository_path;
use std::collections::{
    BTreeMap,
    BTreeSet,
};
use std::path::{
    Path,
    PathBuf,
};

/// The buffer size one comparison reads with,
/// re-exported for the comparison beside the stage.
pub(crate) use crate::worktree_copy_compare_entry_match::COMPARE_BUFFER_BYTES;

/// The comparison of one entry against the filesystem,
/// re-exported so no caller changes.
pub use crate::worktree_copy_compare_entry_match::{
    entry_matches,
    entry_matches_stamped,
};

/// One walk of the source and one of the stage,
/// as `differences_from_the_source` starts them.
use crate::worktree_copy_compare_walks::{
    walk_source_pruned,
    walk_stage,
};

// The metadata question a comparison asks of both sides is the one every walk of a live checkout asks, so it has one
//  home (`worktree_copy_vanished_entry.rs`): a missing entry is an answer, and every other failure is not.
pub use crate::worktree_copy_vanished_entry::lstat_or_absent;

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
        &FinalCheck {
            prune: Prune::Never,
            stamps: &[],
        },
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
    let differences: Differences = differences_from_the_source(
        source_root,
        stage_root,
        selected_roots,
        excluded_source_roots,
        staged_entries,
        check,
    )
    .await?;
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
    let (source_joined, stage_walked): (
        Result<
            Result<Vec<WorktreeCopyEntry>, WorktreeCopyError>,
            crate::blocking_job::BlockingJobError,
        >,
        Result<Vec<WorktreeCopyEntry>, WorktreeCopyError>,
    ) = run_beside_the_caller_on_the_runtime(
        (
            (
                source_root.to_path_buf(),
                selected_roots.to_vec(),
                excluded_source_roots.to_vec(),
                check.prune,
            ),
            walk_source_pruned,
        ),
        (
            (stage_root.to_path_buf(), selected_roots.to_vec()),
            walk_stage,
        ),
    )
    .await;
    let source_walked: Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> = match source_joined {
        Ok(walked) => walked,
        Err(crate::blocking_job::BlockingJobError::Panicked) => Err(raw_failure(
            &std::io::Error::other("a manifest walk panicked"),
        )),
        Err(refused) => Err(raw_failure(&std::io::Error::other(format!(
            "a manifest walk did not run ({refused})"
        )))),
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
