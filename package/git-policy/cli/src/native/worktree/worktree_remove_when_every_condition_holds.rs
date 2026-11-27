//! What:
//!  Remove a worktree if every condition holds,
//!  and report how the attempt ended.
//!  The conditions are the ones Git's own unforced removal makes,
//!  plus the ones this program can answer for:
//!  the registration the directory belongs to and its lock,
//!  the record a conversion leaves beside the worktree,
//!  the volumes below it that this program did not make,
//!  the proof that it is clean,
//!  and the last look its caller takes.
//! Why:
//!   `LeftToGit` means Git has to do the removal and nothing was changed,
//!  so every condition is answered before one file is deleted,
//!  and each answer that leaves the removal to Git is a line in the journal.
//!  It is its own file because `worktree_remove.rs` reached the crate's length limit with it in,
//!  and because the ordered steps of one removal are one subject.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function removeWith(operations: RemoveOperations, request: RemoveRequest): Promise<RemoveOutcome>;
//! ```

// What:     `use crate::diagnostic_log::{debug, warn};` imports the two log sinks
//           this file writes to.
// Why:      A check that leaves the removal to Git is a debug line, and a report the
//           person must read is a warning.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — the package's own logger, called by tag.
// ```
use crate::diagnostic_log::{
    debug,
    warn,
};
// What:     `use crate::fs_beneath::handle_path;` imports the path of a directory this
//           removal holds open, taken from the descriptor rather than from a name.
// Why:      The clear of the pools ahead of an add acts on the held administrative
//           directory, so it cannot be pointed at another one by a rename.
//
// In TS you'd write (pseudocode):
// ```ts
// import { handlePath } from '../fs_beneath';
// ```
use crate::fs_beneath::handle_path;
// What:     `use crate::subvolume_remove_walk::{Boundary, find_boundary_below_except};`
//           imports the record of one subvolume or mount point found below a
//           directory, and the walk that finds the first one and skips the folders it
//           is told to skip.
// Why:      A volume this program did not make below the worktree decides whether the
//           removal goes around it, and a volume inside the registration stops the
//           removal before anything is destroyed.
//
// In TS you'd write (pseudocode):
// ```ts
// import { findBoundaryBelowExcept, type Boundary } from '../subvolume_remove_walk';
// ```
use crate::subvolume_remove_walk::{
    Boundary,
    find_boundary_below_except,
};
// What:     `use crate::worktree_removal_clears_the_ahead_pool::{clear_the_pools_ahead_of_a_worktree, data_folders};`
//           imports the clear of the snapshots one add made ahead of another, and the
//           two folders of the registration that hold them.
// Why:      Those pools stand in the registration this removal empties after the
//           destroy, so they go first, and the proof of nested volumes leaves the two
//           folders of this program out.
//
// In TS you'd write (pseudocode):
// ```ts
// import { clearThePoolsAheadOfAWorktree, dataFolders } from './worktree_removal_clears_the_ahead_pool';
// ```
use crate::worktree_removal_clears_the_ahead_pool::{
    clear_the_pools_ahead_of_a_worktree,
    data_folders,
};
// What:     `use crate::worktree_removal_goes_around_a_foreign_volume::volumes_below;`
//           imports the walk for the volumes below a worktree that this program did
//           not make.
// Why:      A tree that holds one is never given to Git, whose removal would delete
//           what is mounted at a mount point.
//
// In TS you'd write (pseudocode):
// ```ts
// import { volumesBelow } from './worktree_removal_goes_around_a_foreign_volume';
// ```
use crate::worktree_removal_goes_around_a_foreign_volume::volumes_below;
// What:     `use crate::worktree_removal_holds_the_worktree::{Held, confirm, hold, lock_state};`
//           imports the directories one removal holds open, the hold that opens them,
//           the look that compares them with the registration again, and the question
//           whether the registration is locked.
// Why:      A locked worktree is left to Git, and the hold is what makes every later
//           destroy and proof answer for the directories that were proved.
//
// In TS you'd write (pseudocode):
// ```ts
// import { confirm, hold, lockState, type Held } from './worktree_removal_holds_the_worktree';
// ```
use crate::worktree_removal_holds_the_worktree::{
    Held,
    confirm,
    hold,
    lock_state,
};
// What:     `use crate::worktree_removal_interrupted_destroy::{forget_destroy, record_destroy, record_destroyed};`
//           imports the record a destroy leaves before the kernel call, the line that
//           said the files left their path, and the removal of that record.
// Why:      A command killed between the destroy and the removal of the registration
//           leaves a state the next command finishes instead of one only
//           `git worktree prune` could.
//
// In TS you'd write (pseudocode):
// ```ts
// import { forgetDestroy, recordDestroy, recordDestroyed } from './worktree_removal_interrupted_destroy';
// ```
use crate::worktree_removal_interrupted_destroy::{
    forget_destroy,
    record_destroy,
    record_destroyed,
};
// What:     `use crate::worktree_removal_takes_its_pending_deletions::{VolumesBelow, take_the_pending_deletions_below};`
//           imports the move of the subvolumes this program left inside a worktree, and
//           the three answers it gives: clear, foreign volumes found, or unreadable.
// Why:      What this program left goes out before the destroy, and what another
//           program put there is gone around or left to Git.
//
// In TS you'd write (pseudocode):
// ```ts
// import { takeThePendingDeletionsBelow, type VolumesBelow } from './worktree_removal_takes_its_pending_deletions';
// ```
use crate::worktree_removal_takes_its_pending_deletions::{
    VolumesBelow,
    take_the_pending_deletions_below,
};
// What:     `use crate::worktree_remove_operations::{RemoveOperations, REMOVAL_OPTION};`
//           imports the seam the checks are answered through, and the name of the
//           mount option that lets a regular user destroy a subvolume.
// Why:      Every check here is asked of the operations it was handed, so a control
//           runs the same order on a stand-in, and the mount option is what its answer
//           about the destroy is compared with.
//
// In TS you'd write (pseudocode):
// ```ts
// import { REMOVAL_OPTION, type RemoveOperations } from './worktree_remove_operations';
// ```
use crate::worktree_remove_operations::{
    REMOVAL_OPTION,
    RemoveOperations,
};
// What:     `use crate::worktree_remove_outcome::RemoveOutcome;` imports the endings a
//           removal answers with.
// Why:      This file produces every one of them, and its caller reads which happened.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { RemoveOutcome } from './worktree_remove_outcome';
// ```
use crate::worktree_remove_outcome::RemoveOutcome;
// What:     `use crate::worktree_remove_registry::{Registration, registration_of};`
//           imports the record of which worktree registration a directory belongs to,
//           and the lookup that answers it.
// Why:      Everything this file does is done for one registration and no other.
//
// In TS you'd write (pseudocode):
// ```ts
// import { registrationOf, type Registration } from './worktree_remove_registry';
// ```
use crate::worktree_remove_registry::{
    Registration,
    registration_of,
};
// What:     `use crate::worktree_remove_clears_the_registration::finish;` imports the step
//           that clears a registration once the subvolume is gone.
// Why:      This file destroys the subvolume and hands the rest to that step, which the
//           around pass finishes a removal with too.
//
// In TS you'd write (pseudocode):
// ```ts
// import { finish } from './worktree_remove_clears_the_registration';
// ```
use crate::worktree_remove_clears_the_registration::finish;
// What:     `use crate::worktree_remove_reports::{...};` imports the words a removal
//           ends with: one report for a leftover conversion stage and its state, one for
//           a volume the registration holds, one for the snapshots ahead of an add, one
//           for the pending deletions, and one for a registration that cannot be walked.
// Why:      A refusal is read by the person who asked for the removal, so its words are
//           one subject of their own and this file only chooses which to say.
//
// In TS you'd write (pseudocode):
// ```ts
// import { leftoverStageReport, pendingDeletionReport, registrationVolumeReport, snapshotsAheadReport, stageState, unwalkableRegistrationReport } from './worktree_remove_reports';
// ```
use crate::worktree_remove_reports::{
    leftover_stage_report,
    pending_deletion_report,
    registration_volume_report,
    snapshots_ahead_report,
    stage_state,
    unwalkable_registration_report,
};
// What:     `use crate::worktree_remove_request_and_last_look::{RemoveRequest, TAG};`
//           imports what the removal was asked and the tag its log lines carry.
// Why:      Every check is answered for the plan the person named, and every line of
//           this subject writes the same tag.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TAG, type RemoveRequest } from './worktree_remove_request_and_last_look';
// ```
use crate::worktree_remove_request_and_last_look::{
    RemoveRequest,
    TAG,
};
// What:     `use std::path::{Path, PathBuf};` imports the borrowed path type and its
//           owned sibling. Their text siblings are `&str` and `String`, which cannot
//           hold a path that is not UTF-8.
// Why:      The target and the registration's directory are borrowed, and the path of a
//           conversion's copy beside the worktree is built here, which makes an owned
//           path.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Path, PathBuf } from '../paths';
// ```
use std::path::{
    Path,
    PathBuf,
};

/// The last look before the destroy,
/// asked of a request its caller handed in.
#[path = "worktree_remove_last_look.rs"]
mod last_look;

/// What:
///  Remove the worktree if every condition holds;
/// report how it ended.
/// Why:
///   `LeftToGit` means Git has to do the removal,
/// and nothing was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeWith(operations: RemoveOperations, request: RemoveRequest): Promise<RemoveOutcome>;
/// ```
pub fn remove_with(
    operations: &dyn RemoveOperations,
    request: &RemoveRequest<'_>,
) -> RemoveOutcome {
    let target: &Path = request.plan.target.as_path();
    let registration: Registration =
        match registration_of(request.common_dir, target, request.plan.operand.as_os_str()) {
            Ok(found) => found,
            Err(reason) => {
                debug(
                    TAG,
                    format!("{} is not served ({reason}); Git decides", target.display()).as_str(),
                );
                return RemoveOutcome::LeftToGit;
            }
        };
    match lock_state(registration.admin.as_path()) {
        Ok(false) => {}
        Ok(true) => {
            debug(
                TAG,
                format!("{} is locked; Git decides", target.display()).as_str(),
            );
            return RemoveOutcome::LeftToGit;
        }
        Err(reason) => {
            debug(TAG, format!("{reason}; Git decides").as_str());
            return RemoveOutcome::LeftToGit;
        }
    }
    if !operations.is_subvolume_root(target) {
        debug(TAG, format!("{} is not a subvolume; Git removes it file by file unless a volume it did not make stands below it", target.display()).as_str());
    }
    // A conversion's copy beside the worktree is named by the journal in the registration this removal deletes, so a removal
    // now would leave it behind with nothing to find it by. The person finishes or discards it first. What proves the copy is
    // a conversion's is the record the conversion writes beside the worktree: a directory at that name without one is the
    // person's own, and refusing on the name alone would block the removal with nothing able to clear it.
    let converting: PathBuf = crate::worktree_convert::converting_path(target);
    if crate::worktree_convert_leftover_stage_record::recorded_stage_of(target).is_some() {
        let report: String = leftover_stage_report(
            target,
            converting.as_path(),
            stage_state(registration.admin.as_path(), target, converting.as_path()),
        );
        debug(TAG, report.as_str());
        return RemoveOutcome::Refused(report);
    }
    let subvolume: bool = operations.is_subvolume_root(target);
    let destroy: bool = subvolume && operations.removal_allowed(target);
    if subvolume && !destroy {
        debug(
            TAG,
            format!(
                "the mount of {} lacks {REMOVAL_OPTION}; the worktree is removed file by file",
                target.display()
            )
            .as_str(),
        );
    }
    // A tree that holds a subvolume or a mount point this program did not make is never given to Git: Git's own removal
    // deletes what is mounted at a mount point, and it unlinks the files of a nested worktree before the kernel can refuse
    // anything. The look is the one every route makes, and it changes nothing.
    let foreign: Vec<Boundary> = volumes_below(target);
    if !destroy && foreign.is_empty() {
        return RemoveOutcome::LeftToGit;
    }
    let held: Held = match hold(target, &registration) {
        Ok(held) => held,
        Err(reason) => {
            debug(TAG, format!("{reason}; Git decides").as_str());
            return RemoveOutcome::LeftToGit;
        }
    };
    // A removal of this very worktree that was cut short inside its pass is finished by the same unforced command, and a tree
    // a pass left is refused in words that match it: both answers come from the record an earlier pass wrote and from what Git
    // reports about the tree.
    let continuation: crate::worktree_remove_around::Continuation =
        crate::worktree_remove_around::continuation_of(
            request,
            target,
            &registration,
            &held,
            foreign.as_slice(),
        );
    // Git's own removal makes the cleanliness proof only when the person did not force it (`remove_worktree` in
    // `builtin/worktree.c`: `if (!force) check_clean_worktree(wt, av[0])`), and a forced removal deletes the worktree
    // it was given whatever it holds, so the destroy answers for the same set of files.
    if !request.plan.force
        && !continuation.resuming
        && let Err(reason) =
            operations.check_clean(request, registration.admin.as_path(), request.common_dir)
    {
        if foreign.is_empty() {
            debug(
                TAG,
                format!(
                    "{} is not proven clean ({reason}); Git decides",
                    target.display()
                )
                .as_str(),
            );
            return RemoveOutcome::LeftToGit;
        }
        // The state a worktree is in the moment it was added makes the stat fields of its entries prove nothing, by Git's own
        // racy rule (a whole-second modification time that is not older than the index's), and the proof then refuses a tree
        // Git itself finds unchanged. Git settles such an entry by comparing it with the commit, and so does this program's own
        // proof that reads the files: the state is proven the same way this repository proves a conversion's tree, and no
        // worktree is removed on a cleanliness that was never proven.
        match operations.check_clean_by_content(request, registration.admin.as_path(), request.common_dir) {
            Ok(()) => debug(TAG, format!("the stat fields of {} proved nothing and its files are the ones the index names; the proof holds by content", target.display()).as_str()),
            Err(by_content) => {
                return crate::worktree_remove_around::refusal_beside_a_volume(request, target, &registration, foreign.as_slice(), continuation, by_content.as_str());
            }
        }
    }
    if let Err(reason) = confirm(&held, &registration) {
        debug(TAG, format!("{reason}; Git decides").as_str());
        return RemoveOutcome::LeftToGit;
    }
    // The volumes below the tree are the removal's own decision, and this is where it is taken:
    // what this program left
    //  for its commit service is moved out,
    //  and anything else is left where it stands.
    let goes_around: bool =
        match take_the_pending_deletions_below(request.common_dir, &held, target) {
            Ok(VolumesBelow::Clear) => {
                if !destroy {
                    return RemoveOutcome::LeftToGit;
                }
                false
            }
            Ok(VolumesBelow::Foreign(_found)) => true,
            Ok(VolumesBelow::Unreadable(reason)) => {
                // A tree that cannot be read proves nothing either way, and the destroy is refused by the kernel for a subvolume
                // that contains another one; Git is not given a tree this program could not look at.
                debug(
                    TAG,
                    format!("{reason}; the destroy is the kernel's own decision").as_str(),
                );
                if !destroy {
                    return RemoveOutcome::LeftToGit;
                }
                false
            }
            Err(reason) => {
                let report: String = pending_deletion_report(target, reason.as_str());
                warn(TAG, report.as_str());
                return RemoveOutcome::Refused(report);
            }
        };
    // The snapshots ahead of an add are this program's own and stand in the registration the removal empties after the
    // destroy, so they go first: a pool that cannot be taken stops the removal while nothing has happened.
    if let Err(reason) =
        clear_the_pools_ahead_of_a_worktree(request.common_dir, handle_path(&held.admin).as_path())
    {
        let report: String =
            snapshots_ahead_report(target, registration.admin.as_path(), reason.as_str());
        warn(TAG, report.as_str());
        return RemoveOutcome::Refused(report);
    }
    // The registration is emptied after the destroy, and that emptying refuses a nested subvolume, so a volume this
    // program does not own inside it would leave the files gone and the registration stuck, with a notice on every later
    // command. It is proved now, while the worktree and every file of it are still there, so the person reads what stands in
    // the way instead of a removal that cannot finish.
    // This program's own folders there are left out of the proof: the clear above emptied their pools, a fill that made a
    // pool of its own in the moments since is this program's as well, and it is taken before the destroy rather than
    // refused.
    let own: [PathBuf; 2] = data_folders(registration.admin.as_path());
    let except: [&Path; 2] = [own[0].as_path(), own[1].as_path()];
    match find_boundary_below_except(&held.admin, registration.admin.as_path(), &except) {
        Ok(None) => {}
        Ok(Some(found)) => {
            let report: String =
                registration_volume_report(target, registration.admin.as_path(), &found);
            warn(TAG, report.as_str());
            return RemoveOutcome::Refused(report);
        }
        Err(error) => {
            let report: String = unwalkable_registration_report(
                target,
                registration.admin.as_path(),
                error.to_string().as_str(),
            );
            warn(TAG, report.as_str());
            return RemoveOutcome::Refused(report);
        }
    }
    // The walk and the clear above took time, and the destroy names the held subvolume, so the names and the lock are looked
    // at once more: what the first look proved is what this destroy answers for, and a lock or a swap since it is Git's.
    if let Err(reason) = confirm(&held, &registration) {
        debug(TAG, format!("{reason}; Git decides").as_str());
        return RemoveOutcome::LeftToGit;
    }
    // The caller's own last look comes after the proof of cleanliness and after every check above it: a worktree a command
    // added inside this one, or a record written into its Git directory, can arrive while that proof runs, and a caller
    // that read them before it proved this one clean has read them too early. Nothing is changed when it refuses.
    if let Err(reason) = last_look::before_the_destroy(request, target) {
        return RemoveOutcome::Refused(reason);
    }
    if goes_around {
        return crate::worktree_remove_around::go_around(request, &held, &registration);
    }
    // What is about to be destroyed is recorded before the kernel call, so a command killed between the destroy and the
    // removal of the registration leaves a state the next command can finish instead of one only `git worktree prune`
    // could. A record that cannot be written costs that finish and nothing else, so the destruction goes on.
    if let Err(reason) = record_destroy(
        request.common_dir,
        registration.id.as_os_str(),
        &held.admin_identity,
        target,
    ) {
        warn(TAG, format!("the destroy of {} is not recorded, so a command that dies before its registration is removed leaves it to `git worktree prune`: {reason}", target.display()).as_str());
    }
    // The subvolume is handed to the service when it can be: it is renamed inside the parent this removal holds, a durable record
    // names it, and the service's own call deletes it later, so the person waits for one rename and no kernel delete.
    let mut marked: bool = false;
    match operations.hand_over(
        request.context,
        request.common_dir,
        &held,
        target,
        registration.id.as_os_str(),
    ) {
        Ok(()) => debug(
            TAG,
            format!(
                "the subvolume of {} is recorded for the service's delete",
                target.display()
            )
            .as_str(),
        ),
        Err(why) => {
            debug(
                TAG,
                format!(
                    "{why}; the subvolume of {} is destroyed here",
                    target.display()
                )
                .as_str(),
            );
            // The kernel call is next and nothing marks it afterwards, so the record says now that the files leave this path:
            // a command killed inside the call leaves a marked record whose path is gone, which the next command finishes,
            // while a path that is still there is dropped with nothing changed (a review finding, F4).
            record_destroyed(request.common_dir, registration.id.as_os_str());
            marked = true;
            if let Err(error) = operations.destroy(&held) {
                debug(TAG, format!("the subvolume of {} was not destroyed ({error}); the tree is looked at again", target.display()).as_str());
                forget_destroy(request.common_dir, registration.id.as_os_str());
                // The destroy ran after the walk, and the two are not one step: a worktree added below this one since the
                // walk is what makes the kernel refuse the call (`ENOTEMPTY`), and Git's own removal would unlink that
                // worktree's files. A volume the tree holds now is gone around, and anything else is Git's, which is what
                // it was before this proof existed.
                match take_the_pending_deletions_below(request.common_dir, &held, target) {
                    Ok(VolumesBelow::Foreign(_found)) => {
                        return crate::worktree_remove_around::go_around(
                            request,
                            &held,
                            &registration,
                        );
                    }
                    Err(reason) => {
                        let report: String = pending_deletion_report(target, reason.as_str());
                        warn(TAG, report.as_str());
                        return RemoveOutcome::Refused(report);
                    }
                    Ok(VolumesBelow::Clear | VolumesBelow::Unreadable(_)) => {}
                }
                return RemoveOutcome::LeftToGit;
            }
        }
    }
    // Either the kernel call returned or the directory was renamed away, so the record now proves the files are gone from the
    // path the person named; only that proof lets a later command remove the registration.
    if !marked {
        record_destroyed(request.common_dir, registration.id.as_os_str());
    }
    let outcome: RemoveOutcome = finish(&held, &registration, request);
    // A removal that finished has nothing left for a later command; one that could not remove the registration keeps its
    // record, which is what the next command finishes.
    if outcome == RemoveOutcome::Removed {
        forget_destroy(request.common_dir, registration.id.as_os_str());
        // The files went in this command, not in a later round of the commit service, so the ending says which of the two
        // happened: the hand-over's line promises a delete on this path that nobody will make.
        if marked {
            return RemoveOutcome::DestroyedInPlace;
        }
    }
    return outcome;
}
