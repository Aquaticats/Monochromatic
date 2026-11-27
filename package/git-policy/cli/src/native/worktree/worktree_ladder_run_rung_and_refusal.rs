//! What:
//!  The rung the ladder chose before real Git ran,
//! with what it prepared for it,
//! and the two steps every refused rung takes:
//! one log line that carries both refusals,
//! and the size read an add abandons when its rung is refused.
//! Why:
//!  The snapshot rung holds the claim on its stage name,
//! which must be released on every path out,
//! and the rungs that leave Git or the wrapper to fill the destination prepare an empty subvolume there when they can.
//! Both fast rungs carry the conversion fence's answer in the shape their build runs it.
//! A read a refused rung started must not stay running beside Git's own checkout,
//! and its own step keeps the cost of ending the read apart from the decision that refused it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Rung = { kind: 'git' } | { kind: 'clone'; source; claim; fence } | { kind: 'snapshot'; source; claim; fence };
//! abandonTheReadBesideTheCheck(sizesBeside);
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::diagnostic_log::debug;
use crate::worktree_clone_conversion_beside_the_clone::CloneFence;
use crate::worktree_clone_conversion_fence::Fence;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_subvolume::CreatedSubvolume;

/// Import the tag every line of this folder's ladder work carries.
use crate::worktree::worktree_ladder_finish_binding::TAG;

/// What:
///  Which rung the ladder chose before Git ran,
/// with what it prepared for it.
/// Why:
///   The snapshot rung holds the claim on its stage name,
/// which must be released on every path out;
/// the rungs that leave
///  Git or the wrapper to fill the destination prepare an empty subvolume there when they can.
/// Both fast rungs carry the conversion fence's answer in the shape their build runs it:
/// the snapshot decides it before
///  the snapshot,
/// so its index is installed with the written paths' stat data already gone,
/// and the clone rung's shape is
///  `CloneFence`
/// (`worktree_clone_conversion_beside_the_clone.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rung = { kind: 'git'; subvolume?: CreatedSubvolume } | { kind: 'clone'; source: SnapshotSource; subvolume?: CreatedSubvolume; claim: StageClaim; fence: CloneFence } | { kind: 'snapshot'; source: SnapshotSource; claim: StageClaim; fence: Fence };
/// ```
/// Which rung the ladder chose,
/// re-exported by `worktree_ladder_run` so every caller names it there.
pub(crate) enum Rung {
    /// Git's own checkout,
    /// into an empty subvolume the wrapper made when it could.
    Git(Option<CreatedSubvolume>),
    /// Clones of the source's tracked files,
    /// Git running without a checkout first;
    /// the claim records the add durably,
    /// so
    ///  an interrupted one is removed by the next command,
    /// and the fence answers beside the copies of the files it reads.
    /// The index entries the limits already read are shared with the copies,
    /// so neither parses the index twice,
    /// and nothing there is an index the rung reads again.
    Clone(
        Box<SnapshotSource>,
        Option<CreatedSubvolume>,
        StageClaim,
        CloneFence,
        Option<Vec<crate::worktree_clean::StatEntry>>,
    ),
    /// A snapshot of the source,
    /// Git running without a checkout first,
    ///  with the conversion fence decided before the snapshot.
    Snapshot(Box<SnapshotSource>, StageClaim, Fence),
}

/// What:
///  Log why Git,
/// and not a snapshot or clones,
/// checks the worktree out.
/// Why:
///   One log line carries both refusals,
/// so a person reading the log sees why the cheap rungs were not used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// debug(`${plan.destination}: Git checks the worktree out (${reason}; snapshot: ${snapshotRefusal}; clone: ${cloneRefusal})`)
/// ```
pub(crate) fn log_git_checkout(
    plan: &AddPlan,
    reason: &str,
    snapshot_refusal: &Option<String>,
    clone_refusal: &Option<String>,
) {
    debug(TAG, format!("{}: Git checks the worktree out ({reason}; snapshot: {snapshot_refusal:?}; clone: {clone_refusal:?})", plan.destination.display()).as_str());
}

/// What:
///  Stop a read this rung will not use,
///  and state it in the add's record.
/// Why:
///   The check's refusal,
/// the fence's decline and the rung-2 fall-through all leave the read unused;
///  none of them may leave it running beside Git's own checkout.
/// Its own step keeps the cost of ending the read apart from the decision that refused it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const abandon = (started) => started?.abandon();
/// ```
pub(crate) fn abandon_the_read_beside_the_check(
    sizes_beside: &mut Option<crate::worktree_clone_conversion_size_read::SizeReadBesideTheCheck>
) {
    let abandon_entered: bool = begin_step("abandonRead");
    let abandoned = sizes_beside.take();
    if let Some(read) = abandoned {
        read.abandon();
    }
    end_step(abandon_entered);
}
