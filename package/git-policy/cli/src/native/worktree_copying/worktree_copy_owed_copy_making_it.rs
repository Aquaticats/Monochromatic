//! What:
//!  How one owed copy is made:
//!  the checks that must still hold before anything is staged,
//!  the staging and journaling under the journal,
//!  and the record of an unfilled worktree when the copy cannot be made.
//! Why:
//!  The copy is the same staging and installation a creating command makes,
//!  so its never-overwrite rule and its resumption are the proven ones;
//!  this file adds the decision that the copy is still owed and that its destination is unchanged,
//!  and turns a copy that cannot be made into a record every later command names.
//!  The pass that finds the adds that owe one is in `worktree_copy_owed_copy.rs`,
//!  and the fields it reads are in `worktree_copy_owed_copy_records.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = finishOneOwedCopy({ commonDir, add, journaled, ceiling, context, stderr });
//! ```

use crate::json_record::quote;
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::{
    Binding,
    path_identity,
};
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_interrupted_checkout::CheckoutFinish;
use crate::worktree_copy_journal::journal_root;
use crate::worktree_copy_owed_copy_notices::{
    finished_notice,
    finishing_notice,
    not_finished_notice,
    still_owed_notice,
};
use crate::worktree_copy_owed_copy_proofs::{
    clear_the_own_stage,
    source_is_registered,
    whole_registration,
};
use crate::worktree_copy_owed_copy_records::{
    OwedAdd,
    selection_of,
};
use crate::worktree_copy_owed_copy_source::{
    source_could_be_a_worktree,
    source_is_no_worktree_reason,
};
use crate::worktree_copy_owed_taking::{
    Taking,
    take_the_copy,
};
use crate::worktree_copy_registry::{
    WorktreeCopyObservation,
    read_registered_roots,
};
use crate::worktree_copy_source_still_being_filled::source_is_still_being_filled;
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_copy_synchronize::synchronize_created_worktree_noting;
use crate::worktree_copy_unfilled_record::remember_unfilled_with;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};
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
pub(super) fn finish_one_owed_copy(
    common_dir: &Path,
    add: &OwedAdd,
    journaled: bool,
    ceiling: &CopyCeiling,
    context: &GitContext,
    stderr: &mut dyn Write,
) -> Result<Option<String>, WorktreeCopyError> {
    // A journal names the stage: the installation's own recovery resumes it, and this pass must not touch the stage.
    if journaled {
        crate::diagnostic_log::debug(
            "worktreeCopyOwed",
            format!("{} is journaled", add.destination.display()).as_str(),
        );
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
                stderr
                    .write_all(line.as_bytes())
                    .map_err(crate::worktree_copy_error::raw_failure_of)?;
                stderr
                    .write_all(b"\n")
                    .map_err(crate::worktree_copy_error::raw_failure_of)?;
                whole = whole_registration(common_dir, add.destination.as_path());
            }
            CheckoutFinish::Whole => {
                whole = whole_registration(common_dir, add.destination.as_path())
            }
            CheckoutFinish::NotProvable(reason) => {
                crate::diagnostic_log::debug(
                    "worktreeCopyOwed",
                    format!(
                        "the checkout of {} is not finished here: {reason}",
                        add.destination.display()
                    )
                    .as_str(),
                );
            }
        }
    }
    let Some(binding) = whole else {
        crate::diagnostic_log::debug(
            "worktreeCopyOwed",
            format!("{} is not a whole registration", add.destination.display()).as_str(),
        );
        return Ok(Some(still_owed_notice(
            add.destination.as_path(),
            "its checkout is not whole, and nothing proves the worktree holds only what the interrupted add made of it",
        )));
    };
    // A worktree removed and made again at the same path is another object than the add's, so nothing is copied into the
    // one that stands there now; a record written before the add was bound names no identity and is filled as before.
    if let Some(recorded) = add.destination_identity
        && path_identity(add.destination.as_path()) != Some(recorded)
    {
        return Ok(Some(still_owed_notice(
            add.destination.as_path(),
            "the directory there now is not the one the add registered, so nothing is written into it",
        )));
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
            crate::diagnostic_log::debug(
                "worktreeCopyOwed",
                "the registered roots could not be read",
            );
            // The listing runs in the source's own directory, so a source that is no directory any more
            // answers the question it could not be asked: nothing that is a worktree stands there, and the
            // copy can never be made from it. Every other failure proves nothing about the source, so the
            // records stay and the next command asks again.
            if !source_could_be_a_worktree(add.owed.source.as_path()) {
                crate::diagnostic_log::debug(
                    "worktreeCopyOwed",
                    format!(
                        "the source {} is no directory any more",
                        add.owed.source.display()
                    )
                    .as_str(),
                );
                return settle_unfilled(
                    add,
                    source_is_no_worktree_reason(add.owed.source.as_path()).as_str(),
                );
            }
            return Ok(None);
        }
    };
    if !source_is_registered(roots.as_slice(), add.owed.source.as_path()) {
        crate::diagnostic_log::debug(
            "worktreeCopyOwed",
            format!("the source {} is not registered", add.owed.source.display()).as_str(),
        );
        return settle_unfilled(
            add,
            source_is_no_worktree_reason(add.owed.source.as_path()).as_str(),
        );
    }
    // A source that is itself being filled holds part of its ignored state; the record stays and a later command asks again.
    if source_is_still_being_filled(common_dir, add.owed.source.as_path())?.is_some() {
        crate::diagnostic_log::debug(
            "worktreeCopyOwed",
            format!(
                "the source {} is still being filled",
                add.owed.source.display()
            )
            .as_str(),
        );
        return Ok(Some(still_owed_notice(
            add.destination.as_path(),
            "the worktree it takes the ignored state from is itself being filled, so it holds only part of its own",
        )));
    }
    // One command copies into one destination: a marker whose owner lives keeps this one out, and one whose owner died is
    // taken over.
    let taking: Option<Taking> = take_the_copy(&journal_root(common_dir), add.claim.as_path())?;
    if taking.is_none() {
        crate::diagnostic_log::debug(
            "worktreeCopyOwed",
            format!(
                "another command holds the copy of {}",
                add.destination.display()
            )
            .as_str(),
        );
        return Ok(None);
    }
    // A stage nothing proves is the copy's own is named by the settlement, which keeps it and its claim.
    if !clear_the_own_stage(add.container.as_path(), add.destination.as_path())? {
        crate::diagnostic_log::debug(
            "worktreeCopyOwed",
            format!(
                "the stage {} is not provably this copy's own",
                add.container.display()
            )
            .as_str(),
        );
        return Ok(Some(still_owed_notice(
            add.destination.as_path(),
            format!(
                "the stage {} holds something nothing proves this copy made, so it is not touched",
                quote(add.container.to_string_lossy().as_ref())
            )
            .as_str(),
        )));
    }
    let Some(claim) = StageClaim::adopted(
        add.claim.as_path(),
        add.container.as_path(),
        Some(add.record.as_path()),
    ) else {
        return Ok(None);
    };
    // Said before the first entry is read, so the command that waits for this copy is never silent while it lasts.
    stderr
        .write_all(
            finishing_notice(add.destination.as_path(), add.owed.source.as_path()).as_bytes(),
        )
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
        Ok(copied) => Ok(Some(finished_notice(
            add.destination.as_path(),
            add.owed.source.as_path(),
            copied,
        ))),
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
pub(super) fn settle_unfilled(
    add: &OwedAdd,
    reason: &str,
) -> Result<Option<String>, WorktreeCopyError> {
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
    if let Some(settle) = StageClaim::adopted(
        add.claim.as_path(),
        add.container.as_path(),
        Some(add.record.as_path()),
    ) {
        settle.settle()?;
    }
    // A destination whose identity cannot be read gets no record, so this pass's own line is the only one
    // that names it.
    return match identity {
        Some(_identified) => Ok(None),
        None => Ok(Some(not_finished_notice(add.destination.as_path(), reason))),
    };
}
