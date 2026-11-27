//! What:
//!  The two walks the final check compares:
//!  the source under the stage's leave-out rule,
//!  and the stage itself.
//! Why:
//!  Each walk is a wait on storage,
//!  so each is one job of the blocking pool that owns what it reads,
//!  and the final check in `worktree_copy_compare.rs` starts the two beside each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [source, stage] = await Promise.all([walkSourcePruned(sourceJob), walkStage(stageJob)]);
//! ```

use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_manifest::{
    WorktreeCopyEntry,
    collect_entry_manifest,
    collect_pruned_manifest,
};
use crate::worktree_copy_prune::Prune;
use std::path::PathBuf;

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
pub(crate) fn walk_source_pruned(
    (source_root, selected_roots, excluded_source_roots, prune): (
        PathBuf,
        Vec<Vec<u8>>,
        Vec<PathBuf>,
        Prune,
    )
) -> Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> {
    let mut left_out: crate::worktree_copy_manifest::LeftOut =
        crate::worktree_copy_manifest::LeftOut::default();
    return collect_pruned_manifest(
        source_root.as_path(),
        selected_roots.as_slice(),
        excluded_source_roots.as_slice(),
        prune,
        &mut left_out,
        &crate::worktree_copy_ceiling::CopyCeiling::unbounded(),
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
pub(crate) fn walk_stage(
    (stage_root, selected_roots): (PathBuf, Vec<Vec<u8>>)
) -> Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> {
    return collect_entry_manifest(stage_root.as_path(), selected_roots.as_slice(), &[]);
}
