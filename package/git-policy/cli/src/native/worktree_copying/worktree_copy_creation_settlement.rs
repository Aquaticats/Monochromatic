//! What:
//!  What one creating command does around real Git:
//!  the record of the add it makes,
//!  the cheap-worktree ladder and the child Git it starts,
//!  the ignored-state copy into the worktree it created,
//!  and the settlement of a destination Git refused.
//! Why:
//!  No lock stands around the add:
//!  the destination,
//!  the stage claim and the journal belong to this command alone,
//!  so any number of adds run side by side in one repository.
//!  The record comes before real Git runs,
//!  so a command killed after Git returned still leaves a worktree whose ignored state some later command makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = isRegistered(destination); const status = await runLadder(); await synchronize(destination, before);
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::diagnostic_log::warn;
use crate::forwarding::ChildStarted;
use crate::worktree_add_binding::OwedCopy;
use crate::worktree_copy::{
    GitRunner,
    GitStatus,
    Invocation,
    Settlement,
};
use crate::worktree_copy_after_lock::AfterLock;
use crate::worktree_copy_after_refusal::settle_refused_destination;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_notices::nothing_but_a_reservation;
use crate::worktree_copy_owed_copy::owed_selection_of;
use crate::worktree_copy_registry::{
    WorktreeCopyObservation,
    destination_is_registered,
};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage_claim::{
    StageClaim,
    claim_stage_for_creation,
};
use crate::worktree_copy_synchronize::synchronize_after;
use crate::worktree_ladder::{
    LadderResult,
    begin_ladder_holding,
};
use crate::worktree_subvolume::{
    CreatedSubvolume,
    Reservation,
    reserve_destination,
};
#[cfg(target_os = "linux")]
use crate::worktree_trash::PendingTrash;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCopy";

/// What:
///  The record of the Git a creating command started:
///  the claim the command made before Git ran,
///  and the destination its Git fills.
/// Why:
///   The run reports the child's number while that child is still unreaped,
///  and the only thing to do with the number is name it in the claim;
///  so the claim and the destination are the report's own fields
///  instead of closures over them.
///  Both are absent when the command recorded nothing or named no destination,
///  and the report then does nothing.
struct ChildOfACreatingCommand<'a> {
    /// The claim made before Git ran,
    /// when the command made one.
    claim: Option<&'a StageClaim>,
    /// The destination the person's command line named.
    destination: Option<&'a Path>,
}

impl ChildStarted for ChildOfACreatingCommand<'_> {
    /// Name the Git in the claim,
    /// when the command has both a claim and a destination.
    fn started(
        &mut self,
        pid: u32,
    ) {
        if let (Some(claim), Some(destination)) = (self.claim, self.destination) {
            claim.remember_running_child(destination, pid);
        }
    }
}

/// What:
///  Runs one creating command:
/// whether its destination was registered already,
/// the ladder and real Git,
/// and the copy.
/// Why:
///   Nothing here waits for another command:
/// the destination,
/// the stage claim and the journal belong to this command alone,
///  so any number of adds run side by side in one repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = isRegistered(destination); const status = await runLadder(); await synchronize(destination, before);
/// ```
pub(super) fn settle_creation(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    run_git: &mut dyn GitRunner,
    stderr: &mut (dyn Write + Send),
) -> Result<(Settlement, AfterLock), WorktreeCopyError> {
    let registered_before: bool = match invocation.destination.as_deref() {
        Some(destination) => {
            destination_is_registered(observation.common_dir.as_path(), destination)
        }
        None => false,
    };
    let choose_rung_entered: bool = begin_step("chooseRung");
    // The record comes before real Git runs, so a command killed after Git returned still leaves a record of the worktree
    // it made; a command that will copy nothing records nothing. A destination that is registered already is no worktree
    // this command will make, so a record naming it would tell every other command that worktree is still being filled.
    let copies: bool = !registered_before
        && observation.source_root.is_some()
        && !matches!(&observation.selection, CopySelection::Paths(paths) if paths.is_empty());
    let record_destination_entered: bool = begin_step("recordDestination");
    // The record names the source and the selection as well as the destination, so a command killed before its copy ran
    // leaves everything the next command needs to make that copy itself. A selection of named paths is not one of the
    // two listings a record can name, so such a command records nothing and its worktree is named instead.
    let owed: Option<OwedCopy> = observation.source_root.as_ref().and_then(|source| {
        return owed_selection_of(&observation.selection).map(|selection| {
            return OwedCopy {
                source: source.clone(),
                selection,
            };
        });
    });
    let recorded = claim_stage_for_creation(
        observation.common_dir.as_path(),
        invocation.destination.as_deref(),
        copies,
        owed.as_ref(),
    );
    end_step(record_destination_entered);
    let mut held: Option<StageClaim> = match recorded {
        Ok(found) => found,
        Err(error) => {
            warn(TAG, format!("the worktree add could not be recorded durably ({}); an interrupted copy into it will not be reported", error.0).as_str());
            None
        }
    };
    let ladder = begin_ladder_holding(
        invocation.add_plan.as_ref(),
        &mut held,
        &invocation.context,
        &invocation.identity,
    );
    // Who owns the destination is decided here, before real Git runs, in one exclusive creation: the command whose
    // directory Git fills copies into it, whatever its own Git then returns, and every command that finds the directory
    // there copies nothing. A rung that already made its destination settles it itself, so its identity is not reserved
    // again here.
    let reserve_destination_entered: bool = begin_step("reserveDestination");
    // A rung may have made the destination already, as an empty subvolume Git fills: that object is this command's too,
    // and its rung settles it, so it is carried forward rather than reserved again. Otherwise one exclusive creation
    // decides the owner, and no other command's Git can be the one whose worktree this command fills.
    let mut made_here: bool = false;
    let reserved_before_git: Reservation = match (copies, ladder.destination_identity()) {
        (true, Some(identity)) => match invocation.destination.as_deref() {
            Some(destination) => Reservation::Mine(CreatedSubvolume {
                path: destination.to_path_buf(),
                identity: Some(identity),
            }),
            None => Reservation::NotAttempted,
        },
        (true, None) => match invocation.destination.as_deref() {
            Some(destination) => {
                let reservation: Reservation = reserve_destination(destination);
                made_here = matches!(reservation, Reservation::Mine(_));
                reservation
            }
            None => Reservation::NotAttempted,
        },
        _ => Reservation::NotAttempted,
    };
    end_step(reserve_destination_entered);
    end_step(choose_rung_entered);
    let git_add_entered: bool = begin_step("gitAdd");
    // The record names the Git the person's command started, so a command that runs after this one was killed can tell
    //  an add whose Git is still making the destination from one whose Git is gone.
    let naming_destination: Option<&Path> = invocation.destination.as_deref();
    let ran: Result<GitStatus, WorktreeCopyError> = run_git.run(
        &ladder.first_step(),
        &mut ChildOfACreatingCommand {
            claim: held.as_ref(),
            destination: naming_destination,
        },
    );
    end_step(git_add_entered);
    let settled: Result<LadderResult, WorktreeCopyError> = match ran {
        Ok(status) => {
            let finish_rung_entered: bool = begin_step("finishRung");
            // The line Git's checkout prints goes to the standard output, and every note about what the add did instead goes to the stream this command was given, so a caller that hands in its own report reads the whole of it.
            let finished = ladder.finish(
                status,
                run_git,
                &invocation.context,
                &mut std::io::stdout(),
                stderr,
            );
            end_step(finish_rung_entered);
            Ok(finished)
        }
        Err(error) => {
            // Git did not start: the ladder only has its empty subvolume to remove, whatever status it is given.
            ladder.finish(
                GitStatus {
                    code: None,
                },
                run_git,
                &invocation.context,
                &mut std::io::sink(),
                stderr,
            );
            Err(error)
        }
    };
    let finished: LadderResult = settled?;
    let git: GitStatus = finished.git;
    let carried: Option<PathBuf> = finished.carried;
    // A rung that served the add swapped its own snapshot into the destination, so the directory Git made is not there
    // any more and the object at the path is the one the rung made. The reservation names that object, or every later
    // question about whose destination it is compares against an object this command moved away itself and answers, wrongly,
    // that another command replaced it (a service of a snapshot of the main checkout would report no ignored-state copy and
    // would leave its copy record behind).
    let reserved: Reservation = match (finished.placed, invocation.destination.as_deref()) {
        (Some(placed), Some(destination)) => Reservation::Mine(CreatedSubvolume {
            path: destination.to_path_buf(),
            identity: Some(placed),
        }),
        _ => reserved_before_git,
    };
    #[cfg(target_os = "linux")]
    let trash: Option<PendingTrash> = finished.trash;
    // The claim the ladder took is handed back here, not released: the copy owns it until its journal names the stage,
    // so no command killed between the end of the checkout and that journal can leave a worktree whose ignored state was
    // never copied with no record naming it.
    let handed_over: Option<StageClaim> = finished.claim;
    // A snapshot that served the add swapped a subvolume into place, so its record is settled without a flush.
    let swapped: bool = carried.is_some();
    if handed_over.is_some() {
        held = handed_over;
    }
    // Set when the copy deliberately left the destination unfilled, as a source that is still being filled does.
    let mut left_unfilled: bool = false;
    // Set when the destination is another command's worktree, which carries its proof.
    let mut another_commands_worktree: bool = false;
    let (copy_failure, created): (Option<WorktreeCopyError>, Vec<PathBuf>) =
        if git.succeeded() || invocation.destination.is_some() {
            let copy_ignored_entered: bool = begin_step("copyIgnored");
            let copy_ignored_outcome = synchronize_after(
                invocation,
                observation,
                registered_before,
                copies,
                &reserved,
                git.succeeded(),
                carried.as_deref(),
                &mut held,
                &mut left_unfilled,
                &mut another_commands_worktree,
                stderr,
            );
            end_step(copy_ignored_entered);
            match copy_ignored_outcome {
                Ok(roots) => (None, roots),
                // A copy that failed leaves a worktree that is not whole; repairing it would hide that.
                Err(failure) => (Some(failure), Vec::new()),
            }
        } else {
            (None, Vec::new())
        };
    // A destination this command made and real Git refused is removed again while it is still the same object and still
    // empty, so a refused add leaves no empty folder inside the enclosing checkout for a later Git command to act on as if
    // the enclosing checkout were that worktree (the owner met four of them on 2026-11-10 with `git worktree add -b
    // <branch> <path> main`). `rmdir` cannot remove a directory that holds anything, so a destination another program
    // wrote into stays with everything in it, and the person is told which path this command left there. Nothing is
    // removed while the copy failed: what stands at the path is not known then.
    if made_here
        && copy_failure.is_none()
        && !git.succeeded()
        && let Reservation::Mine(made) = &reserved
    {
        // The person's stream carries this line, and a closed one keeps the status Git gave.
        if let Some(line) = settle_refused_destination(made) {
            let _written = stderr.write_all(line.as_bytes());
        }
    }
    // A record whose copy did not run (a destination Git did not register, a snapshot that carried the state itself) is
    // settled here: the add is whole, so the record says so before it goes. A copy that failed is not that: the worktree
    // was never filled, so the record stays and the next command reports it as an interrupted add.
    // A destination that still exists after a Git that failed is a worktree this command asked for and did not fill: the
    // record stays, so a later command reports it instead of finding a worktree that looks whole.
    /// Whether the path may hold something:
    /// absent is the only answer that says it does not,
    /// so a path this command
    /// may not look at keeps the record rather than losing it.
    fn may_hold_something(path: &Path) -> bool {
        return !matches!(
            std::fs::symlink_metadata(path),
            Err(ref error) if error.kind() == std::io::ErrorKind::NotFound
        );
    }
    /// A destination that holds anything other than this command's empty reservation:
    /// a worktree that was not filled.
    fn holds_a_worktree(path: &Path) -> bool {
        return may_hold_something(path) && !nothing_but_a_reservation(path);
    }
    let unfilled: bool = left_unfilled
        || (copies
            && !another_commands_worktree
            && !git.succeeded()
            && invocation
                .destination
                .as_deref()
                .is_some_and(holds_a_worktree));
    if copy_failure.is_none()
        && !unfilled
        && let Some(unused) = held.take()
    {
        let release_unused_entered: bool = begin_step("releaseClaim");
        let released: Result<(), WorktreeCopyError> = if swapped {
            unused.settle_unsynced()
        } else {
            unused.settle()
        };
        end_step(release_unused_entered);
        if let Err(error) = released {
            warn(
                TAG,
                format!(
                    "the record of the worktree add could not be removed ({})",
                    error.0
                )
                .as_str(),
            );
        }
    }
    #[cfg(target_os = "linux")]
    return Ok((
        Settlement {
            git,
            copy_failure,
        },
        AfterLock {
            source_root: observation.source_root.clone(),
            created,
            trash,
        },
    ));
    #[cfg(not(target_os = "linux"))]
    return Ok((
        Settlement {
            git,
            copy_failure,
        },
        AfterLock {
            source_root: observation.source_root.clone(),
            created,
        },
    ));
}
