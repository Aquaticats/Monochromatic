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
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_add_binding::Binding;
use crate::worktree_copy_notices::{DestinationOwnership, UNKNOWN_DESTINATION_NOTICE, nothing_but_a_reservation, not_this_commands_destination_notice, ownership_of, registered_by_another_notice, registration_changed, unconfirmed_destination_notice, uncopiable_notice, unreadable_notice, unsettled_notice};
use crate::worktree_subvolume::Reservation;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure_of};
use crate::worktree_copy_journal::create_journal;
use crate::worktree_copy_journal_encoding_and_validation::PendingJournal;
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_source_still_being_filled::{source_is_still_being_filled, source_still_being_filled_notice};
use crate::worktree_copy_stage::{StagedSnapshot, stage_ignored_snapshot_for};
use crate::worktree_copy_stage_binding::verify_stage_binding;
use crate::worktree_copy_stage_claim::{StageClaim, claim_stage_for_add};
use crate::worktree_copy::{Invocation, summary_line};
use crate::worktree_copy_registry::{CreatedWorktrees, WorktreeCopyObservation, command_token, find_created_worktree};
use crate::worktree_copy_transaction::{Outcome, SnapshotSource, complete_journal};
use std::io::Write;
use std::path::{Path, PathBuf};


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
fn claim_for_the_copy(recorded: Option<StageClaim>, common_dir: &Path, destination_root: &Path) -> Result<StageClaim, WorktreeCopyError> {
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
            std::fs::remove_dir_all(snapshot.stage_container.as_path())
                .map_err(raw_failure_of)?;
            // As above: the record of the unfilled destination outlives this command.
            return Err(error);
        }
    };
    debug(
        "worktreeCopyTransaction",
        format!("journal created in {} ms after the stage at {} ms", journaling.elapsed().as_millis(), staged_done.as_millis()).as_str(),
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
                let _written = notice.write_all(unreadable_notice(source_root, snapshot.unreadable.as_slice()).as_bytes());
            }
            if !snapshot.uncopiable.is_empty() {
                // The same writer and the same silence: a socket the copy cannot hold is named once, and the rest arrived.
                let _written = notice.write_all(uncopiable_notice(source_root, snapshot.uncopiable.as_slice()).as_bytes());
            }
            if !snapshot.unsettled.is_empty() {
                // The same writer and the same silence: an entry a build kept writing stays in its source, named.
                let _written = notice.write_all(unsettled_notice(source_root, snapshot.unsettled.as_slice()).as_bytes());
            }
            Ok(copied)
        }
        Outcome::Ended(error) => Err(error),
    };
}

/// What:
///  Synchronizes the worktree this command registered at its destination and writes the summary line for it;
/// the root of
///  the created worktree,
/// a snapshot's included,
/// comes back for the package-manager repair.
/// Why:
///   The destination is the one the command line named,
/// so a worktree another add registered at the same time is never
///  copied into here;
/// a failure stops the copy with no summary written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = await findCreatedWorktree(...); const copied = await synchronizeCreatedWorktree(...); process.stderr.write(renderSummary(copied));
/// ```
pub(crate) fn synchronize_after(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    registered_before: bool,
    copies: bool,
    reserved: &Reservation,
    git_succeeded: bool,
    carried: Option<&Path>,
    recorded: &mut Option<StageClaim>,
    left_unfilled: &mut bool,
    another_commands_worktree: &mut bool,
    stderr: &mut dyn Write,
) -> Result<Vec<PathBuf>, WorktreeCopyError> {
    // A main checkout that names no path hands over nothing: no stage, no journal and no summary line.
    if matches!(&observation.selection, CopySelection::Paths(paths) if paths.is_empty()) {
        return Ok(carried.map(Path::to_path_buf).into_iter().collect());
    }
    let Some(destination) = invocation.destination.as_deref() else {
        stderr
            .write_all(UNKNOWN_DESTINATION_NOTICE.as_bytes())
            .map_err(raw_failure_of)?;
        return Ok(Vec::new());
    };
    // The destination is this command's only when it made the directory before real Git ran: another command's
    // registration stands at that path otherwise, and Git's own status says nothing about whose it is. A destination this
    // command reserved and no longer holds was moved away and made again, so its registration is not this command's
    // either. Both leave the record to the caller, which settles it, and the person is told.
    // A command that planned no copy writes nothing and says nothing: its destination is one Git refused, a snapshot that
    // carried the ignored state itself, or a selection with nothing in it.
    if !copies {
        return Ok(Vec::new());
    }
    // A snapshot rung that served the add reports the object it swapped into the destination, and `settle_creation`
    // reserves that object as this command's own, so the ownership question below answers `Mine` for a worktree the
    // snapshot filled and no missing-copy line is written. Nothing here needs to skip the question.
    let ownership: DestinationOwnership = ownership_of(reserved, git_succeeded, destination);
    // This command's own name for the registration: its stage claim's random name, which no other command shares, and the
    // process, so the person's line can name which process made a worktree this command did not.
    let mine: String = command_token(recorded.as_ref(), destination)?;
    if ownership != DestinationOwnership::Mine {
        // A Git that never made the destination leaves nothing to explain: its own error is the whole report, and the
        // empty directory this command reserved is removed again below.
        if ownership == DestinationOwnership::GitFailed && nothing_but_a_reservation(destination) {
            return Ok(Vec::new());
        }
        // A refused command whose destination carries another command's proof did not make that worktree, and its own
        // record must not describe the whole worktree as unfinished.
        if ownership == DestinationOwnership::GitFailed
            && crate::worktree_copy_registry::registered_by_another(observation.common_dir.as_path(), destination, mine.as_str())?
        {
            *another_commands_worktree = true;
        }
        // A destination this command made and no longer holds means the worktree it registered is elsewhere and was
        // never filled, so the record naming it stays for the next command to report.
        if ownership == DestinationOwnership::MadeAgain {
            *left_unfilled = true;
        }
        // A refused add whose destination carries another command's proof settles this command's record, so the line
        // must not promise one a later command would report.
        let record_stays: bool = !(ownership == DestinationOwnership::GitFailed && *another_commands_worktree);
        let line: String = not_this_commands_destination_notice(destination, ownership, record_stays);
        stderr
            .write_all(line.as_bytes())
            .map_err(raw_failure_of)?;
        return Ok(Vec::new());
    }
    // The object the destination was when this command decided it was his: the registration it takes must be that one.
    let expected: Option<crate::fs_beneath::ObjectId> = match reserved {
        Reservation::Mine(made) => made.identity,
        Reservation::Anothers(identity) => *identity,
        Reservation::NotAttempted => None,
    };
    let mut found: CreatedWorktrees =
        find_created_worktree(
            &invocation.context,
            observation,
            destination,
            registered_before,
            mine.as_str(),
            expected,
        )?;
    // The registration is bound once Git has returned; a reservation it does not describe is another command's
    // registration at this command's path, made after the ownership answer was read.
    if let Reservation::Mine(made) = reserved
        && let Some(identity) = made.identity
    {
        for worktree in &found.created {
            if worktree.binding.destination_id != identity {
                let line: String = not_this_commands_destination_notice(destination, DestinationOwnership::MadeAgain, true);
                stderr
                    .write_all(line.as_bytes())
                    .map_err(raw_failure_of)?;
                return Ok(Vec::new());
            }
        }
    }
    // The registration at the destination proves it was made by another command, so this command's own worktree is not
    // there and nothing is written into that one: its ignored state is that command's to copy.
    if let Some(other) = found.other_commands.as_deref() {
        let line: String = registered_by_another_notice(destination, other);
        stderr
            .write_all(line.as_bytes())
            .map_err(raw_failure_of)?;
        return Ok(Vec::new());
    }
    // A destination Git made but whose registration cannot be confirmed is told, not skipped without a word.
    if found.created.is_empty()
        && git_succeeded
        && !registered_before
        && std::fs::symlink_metadata(destination.join(".git")).is_ok()
    {
        let line: String = unconfirmed_destination_notice(destination);
        stderr
            .write_all(line.as_bytes())
            .map_err(raw_failure_of)?;
        return Ok(Vec::new());
    }
    let mut created_roots: Vec<PathBuf> = Vec::new();
    for worktree in &found.created {
        created_roots.push(worktree.root.clone());
    }
    // A destination that received the source's ignored state with a snapshot needs no copy.
    let mut kept = Vec::new();
    for worktree in found.created {
        if carried != Some(worktree.root.as_path()) {
            kept.push(worktree);
        }
    }
    found.created = kept;
    if found.created.is_empty() {
        return Ok(created_roots);
    }
    if let Some(source) = observation.source_root.as_ref() {
        // A source that is itself receiving a copy holds only part of its ignored files, so nothing is staged from it;
        // the new worktree is made whole by the same add once that copy has finished, and the person is told so.
        // This comes before the record below is taken, so the caller still holds the record and settles it.
        if let Some(unfinished) = source_is_still_being_filled(observation.common_dir.as_path(), source.as_path())? {
            let line: String = source_still_being_filled_notice(destination, source.as_path(), unfinished);
            stderr
                .write_all(line.as_bytes())
                .map_err(raw_failure_of)?;
            // The destination received no ignored state and this command's Git made it, so the record its owner wrote stays
            // for the next command to report and for the readiness check to hold others back with.
            *left_unfilled = true;
            return Ok(Vec::new());
        }
    }
    let mut copied: usize = 0;
    // The record made before real Git names the stage this copy fills, so the copy keeps the one record it has.
    let mut held: Option<StageClaim> = recorded.take();
    // `match` on the source: a bare repository has nothing to copy, so only a worktree source is copied.
    match observation.source_root.as_ref() {
        None => {}
        Some(source) => {
            for created in &found.created {
                copied += synchronize_created_worktree_noting(
                    &invocation.context,
                    observation.common_dir.as_path(),
                    source.as_path(),
                    created.root.as_path(),
                    found.registered_roots.as_slice(),
                    &observation.selection,
                    held.take(),
                    Some(&created.binding),
                    &CopyCeiling::unbounded(),
                    stderr,
                )?;
            }
        }
    }
    // A copy that did not happen leaves the record to the caller, which settles it.
    *recorded = held;
    let line: String = summary_line(
        copied,
        found.created.len(),
        observation.source_root.as_deref(),
    );
    stderr
        .write_all(line.as_bytes())
        .map_err(raw_failure_of)?;
    return Ok(created_roots);
}

/// Synchronization controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_synchronize_tests.rs"]
mod tests;
