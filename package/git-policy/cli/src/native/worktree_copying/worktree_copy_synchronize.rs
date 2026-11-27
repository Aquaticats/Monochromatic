//! What:
//!  The synchronization of one created worktree with its source's ignored state:
//! the stage of that state,
//! its journal,
//!  the installation into the worktree the command registered,
//! and the notice naming what the copy left out.
//! Why:
//!  The destination of a copy is a path,
//! and the registration behind a path can be removed and replaced by another command
//!  between the moment real Git returned and the moment the first entry is written.
//! The binding the creating command proved is
//!  carried beside the staged payload and checked before staging and again before the first entry,
//! so a copy only ever writes into
//!  the worktree it was made for.
//! A command that ends its copy before it is whole leaves the destination to the person,
//!  who is told what to do there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copied = await synchronizeCreatedWorktreeNoting({ commonDir, sourceRoot, destinationRoot, binding, recorded, selection, notice });
//! ```

use crate::diagnostic_log::debug;
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::Binding;
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    raw_failure_of,
};
use crate::worktree_copy_journal::create_journal;
use crate::worktree_copy_journal_encoding_and_validation::PendingJournal;
use crate::worktree_copy_notices::{
    registration_changed,
    uncopiable_notice,
    unreadable_notice,
    unsettled_notice,
};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage::{
    StagedSnapshot,
    stage_ignored_snapshot_for,
};
use crate::worktree_copy_stage_binding::verify_stage_binding;
use crate::worktree_copy_stage_claim::{
    StageClaim,
    claim_stage_for_add,
};
use crate::worktree_copy_transaction::{
    Outcome,
    SnapshotSource,
    complete_journal,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The claim this copy runs under:
/// the one the creating command recorded before real Git,
/// or,
/// when that could not be written,
/// one made here that still records the destination.
/// Why:
///   The record is what tells every other command that this worktree is being filled.
/// A claim that recorded no destination
///  would hide a copy that is staging behind nothing,
/// so another command could stage its source's half-filled
///  ignored state and never be told.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = recorded ?? await claimStageForAdd({ commonDir, destinationRoot });
/// ```
fn claim_for_the_copy(
    recorded: Option<StageClaim>,
    common_dir: &Path,
    destination_root: &Path,
) -> Result<StageClaim, WorktreeCopyError> {
    return match recorded {
        Some(written) => Ok(written),
        None => claim_stage_for_add(common_dir, destination_root),
    };
}

/// What:
///  Stages,
/// journals and installs the ignored state of one newly created worktree.
/// Why:
///   The stage is removed if the journal cannot be created,
/// and a process that dies while staging leaves a claim,
///  so no stage survives without a record that recovery can act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function synchronizeCreatedWorktree({ commonDir, sourceRoot, destinationRoot, registeredRoots, gitPath }): Promise<number>;
/// ```
pub fn synchronize_created_worktree(
    context: &GitContext,
    common_dir: &Path,
    source_root: &Path,
    destination_root: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
) -> Result<usize, WorktreeCopyError> {
    return synchronize_created_worktree_noting(
        context,
        common_dir,
        source_root,
        destination_root,
        registered_roots,
        selection,
        None,
        None,
        &CopyCeiling::unbounded(),
        &mut std::io::sink(),
    );
}

/// What:
///  Stages one source's ignored state,
/// installs it into one created worktree,
/// and writes a notice naming the
///  entries the user could not read.
/// Why:
///   The notice goes to the writer the lifecycle holds for the person;
/// the count of copied entries is the result.
/// A `recorded` claim is the record the creating command wrote before real Git ran,
/// which names the stage
///  this copy fills;
/// without one the claim is made here,
/// after Git,
/// so an interruption before it is unrecorded.
/// The `binding` is the registration real Git made for the destination;
/// every write is checked against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function synchronizeCreatedWorktreeNoting({ commonDir, sourceRoot, destinationRoot, registeredRoots, selection, recorded, binding, notice }): Promise<number>;
/// ```
pub fn synchronize_created_worktree_noting(
    context: &GitContext,
    common_dir: &Path,
    source_root: &Path,
    destination_root: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
    recorded: Option<StageClaim>,
    binding: Option<&Binding>,
    ceiling: &CopyCeiling,
    notice: &mut dyn std::io::Write,
) -> Result<usize, WorktreeCopyError> {
    let started: std::time::Instant = std::time::Instant::now();
    let claim: StageClaim = claim_for_the_copy(recorded, common_dir, destination_root)?;
    // The registration is checked before the stage costs anything, so a worktree that was removed or replaced meanwhile
    // stops the copy before a byte is staged for it.
    if let Some(bound) = binding
        && let Err(reason) = verify_stage_binding(common_dir, bound)
    {
        // The record stays, so the next command reports the destination this copy did not fill.
        return Err(registration_changed(destination_root, reason.as_str()));
    }
    // `match` on staging: a failure has already removed its container, so only the claim is left to settle.
    let snapshot: StagedSnapshot = match stage_ignored_snapshot_for(
        context,
        source_root,
        claim.container(),
        registered_roots,
        selection,
        binding,
        ceiling,
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            // The claim and its record stay: the destination is not filled, and the next command must see that.
            return Err(error);
        }
    };
    // The registration went into the container before the payload did (`stage_ignored_snapshot_for`), so a recovery
    // checks the pair the copy was made for and not the path alone, and a container killed while staging is provably this
    // copy's own.
    let staged_done: std::time::Duration = started.elapsed();
    let journaling: std::time::Instant = std::time::Instant::now();
    let pending: PendingJournal = match create_journal(common_dir, destination_root, &snapshot) {
        Ok(pending) => pending,
        Err(error) => {
            // A failed removal replaces the journal failure, as the incumbent's cleanup throws its own error there.
            std::fs::remove_dir_all(snapshot.stage_container.as_path()).map_err(raw_failure_of)?;
            // As above: the record of the unfilled destination outlives this command.
            return Err(error);
        }
    };
    debug(
        "worktreeCopyTransaction",
        format!(
            "journal created in {} ms after the stage at {} ms",
            journaling.elapsed().as_millis(),
            staged_done.as_millis()
        )
        .as_str(),
    );
    // The journal names the stage from here on, so the claim has done its work.
    claim.release()?;
    // The last check before the first entry is written: staging takes minutes, and the registration it was staged for
    // can be gone by now. The journal and stage stay pending, so the next command's recovery reports them.
    if let Some(bound) = binding
        && let Err(reason) = verify_stage_binding(common_dir, bound)
    {
        return Err(registration_changed(destination_root, reason.as_str()));
    }
    let staged_after: std::time::Duration = started.elapsed();
    // The last moment before installation: a command that finishes a copy it did not start stops here too, rather than
    //  installing a tree that cost more than its ceiling.
    if ceiling.reached() {
        return Err(ceiling.stopped_error());
    }
    let staged: StagedSnapshot = snapshot.clone();
    let outcome: Outcome = complete_journal(pending, &SnapshotSource::Staged(&staged), ceiling)?;
    debug(
        "worktreeCopyTransaction",
        format!(
            "settled {} entries in {} ms: stage and journal {} ms, install and completion {} ms",
            snapshot.entries.len(),
            started.elapsed().as_millis(),
            staged_after.as_millis(),
            (started.elapsed() - staged_after).as_millis()
        )
        .as_str(),
    );
    return match outcome {
        Outcome::Installed(copied) => {
            if !snapshot.unreadable.is_empty() {
                // A closed standard error leaves nowhere to print; the log line of the stage still names the entries.
                let _written = notice.write_all(
                    unreadable_notice(source_root, snapshot.unreadable.as_slice()).as_bytes(),
                );
            }
            if !snapshot.uncopiable.is_empty() {
                // The same writer and the same silence: a socket the copy cannot hold is named once, and the rest arrived.
                let _written = notice.write_all(
                    uncopiable_notice(source_root, snapshot.uncopiable.as_slice()).as_bytes(),
                );
            }
            if !snapshot.unsettled.is_empty() {
                // The same writer and the same silence: an entry a build kept writing stays in its source, named.
                let _written = notice.write_all(
                    unsettled_notice(source_root, snapshot.unsettled.as_slice()).as_bytes(),
                );
            }
            Ok(copied)
        }
        Outcome::Ended(error) => Err(error),
    };
}

/// Synchronization controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_synchronize_tests.rs"]
mod tests;
