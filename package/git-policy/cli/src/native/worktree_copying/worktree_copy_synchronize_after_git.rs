//! What:
//!  What one command does around the copy after real Git returned:
//!  whose destination the worktree is,
//!  which registration Git made for it,
//!  the line naming a destination this command did not make,
//!  and the summary line of a copy that ran.
//! Why:
//!  A destination is this command's only when it made the directory before real Git ran,
//! and a registration can be another command's even at this command's path,
//! so every question about ownership is answered before anything is written there;
//! the copy itself is the staging,
//! its journal and its installation in `worktree_copy_synchronize.rs`,
//!  which this file calls once per worktree it may fill.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = await findCreatedWorktree(...); const copied = await synchronizeCreatedWorktree(...); stderr.write(renderSummary(copied));
//! ```

use crate::worktree_copy::{
    Invocation,
    summary_line,
};
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    raw_failure_of,
};
use crate::worktree_copy_notices::{
    DestinationOwnership,
    UNKNOWN_DESTINATION_NOTICE,
    not_this_commands_destination_notice,
    nothing_but_a_reservation,
    ownership_of,
    registered_by_another_notice,
    unconfirmed_destination_notice,
};
use crate::worktree_copy_registry::{
    CreatedWorktrees,
    WorktreeCopyObservation,
    command_token,
    find_created_worktree,
};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_source_still_being_filled::{
    source_is_still_being_filled,
    source_still_being_filled_notice,
};
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_copy_synchronize::synchronize_created_worktree_noting;
use crate::worktree_subvolume::Reservation;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

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
            && crate::worktree_copy_registry::registered_by_another(
                observation.common_dir.as_path(),
                destination,
                mine.as_str(),
            )?
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
        let record_stays: bool =
            !(ownership == DestinationOwnership::GitFailed && *another_commands_worktree);
        let line: String =
            not_this_commands_destination_notice(destination, ownership, record_stays);
        stderr.write_all(line.as_bytes()).map_err(raw_failure_of)?;
        return Ok(Vec::new());
    }
    // The object the destination was when this command decided it was his: the registration it takes must be that one.
    let expected: Option<crate::fs_beneath::ObjectId> = match reserved {
        Reservation::Mine(made) => made.identity,
        Reservation::Anothers(identity) => *identity,
        Reservation::NotAttempted => None,
    };
    let mut found: CreatedWorktrees = find_created_worktree(
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
                let line: String = not_this_commands_destination_notice(
                    destination,
                    DestinationOwnership::MadeAgain,
                    true,
                );
                stderr.write_all(line.as_bytes()).map_err(raw_failure_of)?;
                return Ok(Vec::new());
            }
        }
    }
    // The registration at the destination proves it was made by another command, so this command's own worktree is not
    // there and nothing is written into that one: its ignored state is that command's to copy.
    if let Some(other) = found.other_commands.as_deref() {
        let line: String = registered_by_another_notice(destination, other);
        stderr.write_all(line.as_bytes()).map_err(raw_failure_of)?;
        return Ok(Vec::new());
    }
    // A destination Git made but whose registration cannot be confirmed is told, not skipped without a word.
    if found.created.is_empty()
        && git_succeeded
        && !registered_before
        && std::fs::symlink_metadata(destination.join(".git")).is_ok()
    {
        let line: String = unconfirmed_destination_notice(destination);
        stderr.write_all(line.as_bytes()).map_err(raw_failure_of)?;
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
        if let Some(unfinished) =
            source_is_still_being_filled(observation.common_dir.as_path(), source.as_path())?
        {
            let line: String =
                source_still_being_filled_notice(destination, source.as_path(), unfinished);
            stderr.write_all(line.as_bytes()).map_err(raw_failure_of)?;
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
    stderr.write_all(line.as_bytes()).map_err(raw_failure_of)?;
    return Ok(created_roots);
}
