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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notices = await finishOwedWorktreeCopies({ commonDir, verdicts, context, stderr });
//! ```

use crate::transaction_git::GitContext;
use crate::worktree_add_binding::{Binding, OwedCopy, OwedSelection, Recorded, decode_record, owed_copy_of, path_identity};
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_interrupted_checkout::CheckoutFinish;
use crate::worktree_copy_journal::{journal_root, read_pending_journals};
use crate::worktree_copy_registry::{WorktreeCopyObservation, read_registered_roots};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_source_still_being_filled::source_is_still_being_filled;
use crate::worktree_copy_stage_claim::{DESTINATION_SUFFIX, StageClaim, claim_names};
use crate::worktree_copy_stage_claim_reaping::{StageVerdicts, read_claim};
use crate::worktree_copy_synchronize::synchronize_created_worktree_noting;
use crate::worktree_copy_owed_copy_source::{source_could_be_a_worktree, source_is_no_worktree_reason};
use crate::worktree_copy_unfilled_record::remember_unfilled_with;
use crate::json_record::quote;
use std::io::Write;
use std::path::{Path, PathBuf};

//region The pieces this pass reads from their own modules

/// What:
///  `pub(crate) use` names each piece of this pass under the one module path its callers already write,
///  `crate::worktree_copy_owed_copy::name`.
/// Why:
///   The pass was split into sibling modules so each file stays a size a reader can hold,
/// and the glob-free re-export keeps every caller and the sidecar's `use super::*` unchanged.
/// Gotcha:
///   Only the names the pass itself uses are listed;
///  a private helper of a sibling stays there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { owedInAnotherWorktreeNotice } from './worktree_copy_owed_copy_notices';
/// export { takeTheCopy } from './worktree_copy_owed_taking';
/// ```
pub use super::worktree_copy_owed_copy_notices::owed_in_another_worktree_notice;
pub(crate) use super::worktree_copy_owed_copy_notices::{finished_notice, finishing_notice, not_finished_notice, still_owed_notice};
pub(crate) use super::worktree_copy_owed_copy_proofs::{clear_the_own_stage, source_is_registered, whole_registration};
pub(crate) use super::worktree_copy_owed_taking::{Taking, remove_dead_taking_marks, take_the_copy};

// The sidecar reads this name through `use super::*`, and the check that needs it now lives in
// `worktree_copy_owed_copy_proofs.rs`, so the name stays here for the test build alone.
#[cfg(test)]
pub(crate) use crate::worktree_copy_interrupted_add_settlement::registration_directory;

//endregion

/// What:
///  The selection an add's record names,
/// as the copy takes it.
/// Why:
///   The record writes the two listings a source can have,
/// and a reader takes only those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selectionOf = (owed) => owed.selection === 'main-checkout' ? CopySelection.MainCheckout : CopySelection.Ignored;
/// ```
fn selection_of(owed: &OwedCopy) -> CopySelection {
    return match owed.selection {
        OwedSelection::Ignored => CopySelection::Ignored,
        OwedSelection::MainCheckout => CopySelection::MainCheckout,
    };
}

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
///  One add's unfinished state,
/// as the claim and its two records say it.
/// Why:
///   Every check below reads the same fields,
/// so they are parsed once and named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwedAdd = { claim: string; container: string; record: string; destination: string; owed: OwedCopy };
/// ```
struct OwedAdd {
    /// The claim file,
    /// whose stage name the copy fills.
    claim: PathBuf,
    /// The stage container the claim names.
    container: PathBuf,
    /// The record of the destination written beside the claim.
    record: PathBuf,
    /// The worktree the add registered.
    destination: PathBuf,
    /// The identity the record holds for that worktree,
    /// when it was bound before the command died.
    destination_identity: Option<crate::fs_beneath::ObjectId>,
    /// The ignored-state copy the add owes.
    owed: OwedCopy,
}

/// What:
///  The add's unfinished state,
/// or nothing when the claim is not one an add made or its record names no copy.
/// Why:
///   A copy's own claim records no destination,
/// and a record written before the source was written names none:
/// both settle exactly as they did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owedAdd = owedAddOf(claimPath, container);
/// ```
fn owed_add_of(claim: &Path, container: &Path) -> Option<OwedAdd> {
    let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
    let text: String = std::fs::read_to_string(record.as_path()).ok()?;
    let (destination, destination_identity): (PathBuf, Option<crate::fs_beneath::ObjectId>) = match decode_record(text.as_str())? {
        Recorded::Planned(named) => (named, None),
        Recorded::Registered(binding) => (binding.destination, Some(binding.destination_id)),
    };
    let owed: OwedCopy = owed_copy_of(text.as_str())?;
    return Some(OwedAdd { claim: claim.to_path_buf(), container: container.to_path_buf(), record, destination, destination_identity, owed });
}

/// What:
///  Makes the copy one interrupted add owed,
/// or reports why it was not made.
/// Why:
///   The copy is the same staging,
/// journaling and installation a creating command makes,
/// so its never-overwrite rule,
/// its records and its resumption are the proven ones;
/// this pass adds the
///  decision that the copy is still owed and that its destination is unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = finishOneOwedCopy({ commonDir, add, journaled, context, stderr });
/// ```
fn finish_one_owed_copy(
    common_dir: &Path,
    add: &OwedAdd,
    journaled: bool,
    ceiling: &CopyCeiling,
    context: &GitContext,
    stderr: &mut dyn Write,
) -> Result<Option<String>, WorktreeCopyError> {
    // A journal names the stage: the installation's own recovery resumes it, and this pass must not touch the stage.
    if journaled {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("{} is journaled", add.destination.display()).as_str());
        return Ok(None);
    }
    // The add's own Git may never have finished its checkout: a kill inside it leaves a registered worktree with no
    // index, or with the lock of the step that publishes one. A command in that worktree finishes it here, and only
    // where everything standing there came from the source, so no file of the person's is ever replaced.
    let mut whole: Option<Binding> = whole_registration(common_dir, add.destination.as_path());
    if whole.is_none() {
        match crate::worktree_copy_interrupted_checkout::finish_interrupted_checkout(
            common_dir,
            add.destination.as_path(),
            add.owed.source.as_path(),
            context,
            // The caller reached this claim through the judgement that proved its owner dead, so the lock a kill left
            // here is a dead add's and no live writer's.
            true,
            ceiling,
        )? {
            CheckoutFinish::Finished(line) => {
                stderr.write_all(line.as_bytes()).map_err(crate::worktree_copy_error::raw_failure_of)?;
                stderr.write_all(b"\n").map_err(crate::worktree_copy_error::raw_failure_of)?;
                whole = whole_registration(common_dir, add.destination.as_path());
            }
            CheckoutFinish::Whole => whole = whole_registration(common_dir, add.destination.as_path()),
            CheckoutFinish::NotProvable(reason) => {
                crate::diagnostic_log::debug("worktreeCopyOwed", format!("the checkout of {} is not finished here: {reason}", add.destination.display()).as_str());
            }
        }
    }
    let Some(binding) = whole else {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("{} is not a whole registration", add.destination.display()).as_str());
        return Ok(Some(still_owed_notice(add.destination.as_path(), "its checkout is not whole, and nothing proves the worktree holds only what the interrupted add made of it")));
    };
    // A worktree removed and made again at the same path is another object than the add's, so nothing is copied into the
    // one that stands there now; a record written before the add was bound names no identity and is filled as before.
    if let Some(recorded) = add.destination_identity
        && path_identity(add.destination.as_path()) != Some(recorded)
    {
        return Ok(Some(still_owed_notice(add.destination.as_path(), "the directory there now is not the one the add registered, so nothing is written into it")));
    }
    // The listing runs where the source is, so a command's own working directory never decides what the repository
    // registers.
    let observation: WorktreeCopyObservation = WorktreeCopyObservation {
        common_dir: common_dir.to_path_buf(),
        effective_cwd: add.owed.source.clone(),
        source_root: Some(add.owed.source.clone()),
        selection: selection_of(&add.owed),
    };
    let roots: Vec<PathBuf> = match read_registered_roots(context, &observation) {
        Ok(found) => found,
        Err(_unreadable) => {
            crate::diagnostic_log::debug("worktreeCopyOwed", "the registered roots could not be read");
            // The listing runs in the source's own directory, so a source that is no directory any more
            // answers the question it could not be asked: nothing that is a worktree stands there, and the
            // copy can never be made from it. Every other failure proves nothing about the source, so the
            // records stay and the next command asks again.
            if !source_could_be_a_worktree(add.owed.source.as_path()) {
                crate::diagnostic_log::debug("worktreeCopyOwed", format!("the source {} is no directory any more", add.owed.source.display()).as_str());
                return settle_unfilled(add, source_is_no_worktree_reason(add.owed.source.as_path()).as_str());
            }
            return Ok(None);
        }
    };
    if !source_is_registered(roots.as_slice(), add.owed.source.as_path()) {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("the source {} is not registered", add.owed.source.display()).as_str());
        return settle_unfilled(add, source_is_no_worktree_reason(add.owed.source.as_path()).as_str());
    }
    // A source that is itself being filled holds part of its ignored state; the record stays and a later command asks again.
    if source_is_still_being_filled(common_dir, add.owed.source.as_path())?.is_some() {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("the source {} is still being filled", add.owed.source.display()).as_str());
        return Ok(Some(still_owed_notice(add.destination.as_path(), "the worktree it takes the ignored state from is itself being filled, so it holds only part of its own")));
    }
    // One command copies into one destination: a marker whose owner lives keeps this one out, and one whose owner died is
    // taken over.
    let taking: Option<Taking> = take_the_copy(&journal_root(common_dir), add.claim.as_path())?;
    if taking.is_none() {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("another command holds the copy of {}", add.destination.display()).as_str());
        return Ok(None);
    }
    // A stage nothing proves is the copy's own is named by the settlement, which keeps it and its claim.
    if !clear_the_own_stage(add.container.as_path(), add.destination.as_path())? {
        crate::diagnostic_log::debug("worktreeCopyOwed", format!("the stage {} is not provably this copy's own", add.container.display()).as_str());
        return Ok(Some(still_owed_notice(add.destination.as_path(), format!("the stage {} holds something nothing proves this copy made, so it is not touched", quote(add.container.to_string_lossy().as_ref())).as_str())));
    }
    let Some(claim) = StageClaim::adopted(add.claim.as_path(), add.container.as_path(), Some(add.record.as_path())) else {
        return Ok(None);
    };
    // Said before the first entry is read, so the command that waits for this copy is never silent while it lasts.
    stderr
        .write_all(finishing_notice(add.destination.as_path(), add.owed.source.as_path()).as_bytes())
        .map_err(crate::worktree_copy_error::raw_failure_of)?;
    return match synchronize_created_worktree_noting(
        context,
        common_dir,
        add.owed.source.as_path(),
        add.destination.as_path(),
        roots.as_slice(),
        &selection_of(&add.owed),
        Some(claim),
        Some(&binding),
        ceiling,
        stderr,
    ) {
        Ok(copied) => Ok(Some(finished_notice(add.destination.as_path(), add.owed.source.as_path(), copied))),
        Err(failure) => settle_unfilled(add, failure.0.as_str()),
    };
}

/// What:
///  Turns an add whose copy cannot be made into the durable record of an unfilled worktree,
/// and settles its claim.
/// Why:
///   Every later command names a worktree whose record says files are missing,
/// and none of them stages the source again trying what just failed;
/// the person settles it by removing the worktree or replacing it.
/// The record's own line is written by this same pass,
/// so the failure is said once there;
/// it is written before the claim is settled,
/// so a command killed in between still leaves both the record and the claim that names it;
/// only a destination whose identity cannot be read,
/// and which therefore gets no record,
/// is named by this function's own line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = await settleUnfilled(add, reason); // undefined when the record says it
/// ```
fn settle_unfilled(add: &OwedAdd, reason: &str) -> Result<Option<String>, WorktreeCopyError> {
    let identity: Option<crate::fs_beneath::ObjectId> = path_identity(add.destination.as_path());
    // The record comes before the claim is settled, so a command killed between the two leaves the record
    // and the claim that names it: every later command still names this worktree and the reason it is
    // unfilled. The other order left a kill between the two with nothing at all on disk, and no later
    // command could tell that worktree from a whole one.
    if let Some(found) = identity {
        remember_unfilled_with(
            add.claim.as_path(),
            add.destination.as_path(),
            found,
            add.owed.source.as_path(),
            None,
            reason,
        )?;
    }
    if let Some(settle) = StageClaim::adopted(add.claim.as_path(), add.container.as_path(), Some(add.record.as_path())) {
        settle.settle()?;
    }
    // A destination whose identity cannot be read gets no record, so this pass's own line is the only one
    // that names it.
    return match identity {
        Some(_identified) => Ok(None),
        None => Ok(Some(not_finished_notice(add.destination.as_path(), reason))),
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
    destinations: &crate::worktree_copy_recovery::RecoveryDestinations<'_>,
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
        if crate::worktree_copy_swapped_add_settlement::is_a_completed_exchange(claim.as_path(), add.container.as_path())? {
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
        if !crate::worktree_copy_recovery::finishes_destination(destinations, add.destination.as_path()) {
            notices.push(owed_in_another_worktree_notice(add.destination.as_path()));
            continue;
        }
        let journaled_here: bool = journaled.contains(&add.container);
        if let Some(notice) = finish_one_owed_copy(common_dir, &add, journaled_here, ceiling, context, stderr)? {
            notices.push(notice);
        }
    }
    return Ok(notices);
}

/// Owed-copy controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_owed_copy_tests.rs"]
mod tests;
