//! What:
//!  The ignored-state copy an interrupted `git worktree add` still owes,
//! made by the first command run in the worktree it fills,
//! under the bound one command holds for work it
//! did not start.
//! Why:
//!  An add records the destination,
//! the source and the selection before real Git runs,
//! because the copy that fills the new worktree can be cut off at any instruction:
//! the stage is still being filled,
//! the stage is whole and no journal names it,
//! or the installation never ran.
//! Left alone,
//! the worktree stays unfilled and the person must remove it and add it again,
//! losing their own work in it;
//! so the first command run in that worktree stages from the recorded source and installs under the journal,
//! when the destination is still the registration the add made and its checkout is whole.
//! A command in another worktree never makes it:
//! the copy costs what the source's ignored tree costs,
//! and that
//! command gains nothing from it.
//! Nothing is overwritten:
//! the installation accepts an exact entry and refuses a differing one,
//! so a worktree that gained anything since the add is named rather than written into.
//! A copy that cannot be made becomes the durable record of an unfilled worktree,
//! which every later command names,
//! rather than being staged again by each one.
//! How one such copy is made is in `worktree_copy_owed_copy_making_it.rs`,
//! and the fields its claim and records give are in `worktree_copy_owed_copy_records.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notices = await finishOwedWorktreeCopies({ commonDir, verdicts, context, stderr });
//! ```

use crate::transaction_git::GitContext;
use crate::worktree_add_binding::OwedSelection;
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_journal::{
    journal_root,
    read_pending_journals,
};
use crate::worktree_copy_owed_copy_making_it::finish_one_owed_copy;
use crate::worktree_copy_owed_copy_records::owed_add_of;
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage_claim::claim_names;
use crate::worktree_copy_stage_claim_owner_verdicts::StageVerdicts;
use crate::worktree_copy_stage_claim_record::read_claim;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

//region The pieces this pass reads from their own modules

/// What:
///  `pub(crate) use` names each piece of this pass under the one module path its callers already write,
///  `crate::worktree_copy_owed_copy::name`.
/// Why:
///   The pass was split into sibling modules so each file stays a size a reader can hold,
/// and the glob-free re-export keeps every caller and the sidecar's `use super::*` unchanged.
/// Gotcha:
///   Only the names this file itself uses,
///  or the sidecar reads,
///  are listed;
///  a name the making of one copy uses is imported there instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { owedInAnotherWorktreeNotice } from './worktree_copy_owed_copy_notices';
/// export { removeDeadTakingMarks } from './worktree_copy_owed_taking';
/// ```
pub use super::worktree_copy_owed_copy_notices::owed_in_another_worktree_notice;
pub(crate) use super::worktree_copy_owed_taking::remove_dead_taking_marks;

// The sidecar reads this name through `use super::*`, and the making of one copy now lives in
// `worktree_copy_owed_copy_making_it.rs`, so the name stays here for the test build alone.
#[cfg(test)]
pub(super) use crate::worktree_copy_owed_copy_making_it::settle_unfilled;

// The sidecar reads this name through `use super::*`, and the making of one copy now lives in
// `worktree_copy_owed_copy_making_it.rs`, so the name stays here for the test build alone.
#[cfg(test)]
pub(super) use crate::worktree_copy_stage_claim::StageClaim;

// The sidecar reads this name through `use super::*`, and the check that needs it now lives in
// `worktree_copy_owed_copy_proofs.rs`, so the name stays here for the test build alone.
#[cfg(test)]
pub(crate) use crate::worktree_copy_interrupted_add_settlement::registration_directory;

//endregion

/// What:
///  Which of an add's ignored-state copies a selection is,
/// or nothing when the selection names paths a record cannot hold.
/// Why:
///   The record has room for the two listings a source can have,
/// so a command whose copy takes named paths records no
///  copy and its worktree is named instead of filled from the wrong set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owed = owedSelectionOf(selection); // 'ignored' | 'main-checkout' | undefined
/// ```
pub fn owed_selection_of(selection: &CopySelection) -> Option<OwedSelection> {
    return match selection {
        CopySelection::Ignored => Some(OwedSelection::Ignored),
        CopySelection::MainCheckout => Some(OwedSelection::MainCheckout),
        CopySelection::Paths(_) => None,
    };
}
/// What:
///  Makes every copy an interrupted add still owes into the worktree this command runs in,
/// on the dead owners the verdicts named.
/// Why:
///   A live owner's add is running and its copy with it,
/// so only a dead owner's records are finished here;
/// the judgement is the sweep's own,
/// taken before anything is changed.
/// A copy into another worktree stays owed and is named instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await finishOwedWorktreeCopies({ commonDir, verdicts, context, stderr });
/// ```
pub fn finish_owed_worktree_copies(
    common_dir: &Path,
    verdicts: &StageVerdicts,
    destinations: &crate::worktree_copy_recovery_destinations::RecoveryDestinations<'_>,
    ceiling: &CopyCeiling,
    context: &GitContext,
    stderr: &mut dyn Write,
) -> Result<Vec<String>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    remove_dead_taking_marks(root.as_path())?;
    let names: Vec<String> = claim_names(root.as_path())?;
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let mut journaled: Vec<PathBuf> = Vec::new();
    for pending in read_pending_journals(common_dir)? {
        journaled.push(pending.journal.stage_container);
    }
    let mut notices: Vec<String> = Vec::new();
    for name in names {
        if !verdicts.names_dead_claim(name.as_str()) {
            continue;
        }
        let claim: PathBuf = root.join(name.as_str());
        let Some((container, _owner)) = read_claim(claim.as_path())? else {
            continue;
        };
        let Some(add) = owed_add_of(claim.as_path(), container.as_path()) else {
            continue;
        };
        // An add whose own snapshot was swapped into place owes no copy: the snapshot carried the source's whole
        // ignored state. The sweep settles the stage and records that kill left, whoever runs, so this pass says
        // nothing about a copy that no command could make.
        if crate::worktree_copy_swapped_add_settlement::is_a_completed_exchange(
            claim.as_path(),
            add.container.as_path(),
        )? {
            continue;
        }
        // The same answer where a kill removed the record of the object the stage's creation made before this
        // command read the claim: the completion marker is the add's own record that it finished, so the file to
        // copy is already in the worktree and no command may name it missing.
        if crate::worktree_copy_interrupted_add_settlement::the_add_finished(claim.as_path()) {
            continue;
        }
        // The copy fills one worktree, and the command that runs in it is the one that waits: a command in another
        // worktree does not pay for the source's ignored state, and is told which worktree does.
        if !crate::worktree_copy_recovery_destinations::finishes_destination(
            destinations,
            add.destination.as_path(),
        ) {
            notices.push(owed_in_another_worktree_notice(add.destination.as_path()));
            continue;
        }
        let journaled_here: bool = journaled.contains(&add.container);
        if let Some(notice) =
            finish_one_owed_copy(common_dir, &add, journaled_here, ceiling, context, stderr)?
        {
            notices.push(notice);
        }
    }
    return Ok(notices);
}

/// Owed-copy controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_owed_copy_tests.rs"]
mod tests;
