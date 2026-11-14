//! What:
//!  The half of a removal that goes around a subvolume or a mount point it did not make:
//!  the record a pass leaves when a command is killed inside it,
//!  the continuation that record proves,
//!  the pass itself,
//!  and the words for a removal whose registration could not be removed after it.
//! Why:
//!  The removal's other steps decide whether it may run at all;
//! this half decides what it does with what it did not make,
//! and it is the half a
//!  command can be killed inside,
//!  so both the record that a kill leaves and the reading that finishes it belong here.
//!  It lives apart from `worktree_remove.rs` because the file of the whole removal outgrew the length a
//!  reader keeps in mind,
//! and the seam between the two is the proof of cleanliness: after it, this half runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { resuming, onlyDeletions } = continuationOf({ request, target, registration, foreign, begun });
//! const outcome = goAround({ request, held, registration }); // RemovedAround | Removed | Incomplete
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::remove_at;
use crate::subvolume_remove_around::{empty_directory_around, look_for_boundaries};
use crate::worktree_removal_goes_around_a_foreign_volume::around_incomplete_report;
use crate::worktree_removal_interrupted_destroy::forget_destroy;
use crate::subvolume_remove_walk::Boundary;
use crate::worktree_removal_around_begun::WhichRemoval;
use crate::worktree_removal_only_deletions::Reported;
use crate::worktree_removal_goes_around_a_foreign_volume::{stopped_around_report, volumes_below};
use crate::worktree_remove::{Held, RemoveOutcome, RemoveRequest, TAG, finish};
use crate::worktree_remove_registry::Registration;
use std::path::Path;

/// What:
///  Whether an unforced command may finish a removal an earlier one began,
///  and whether the worktree Git describes holds nothing but what that pass removed.
/// Why:
///  A pass that was cut short leaves the worktree's own entries deleted and the volumes standing,
/// and the ignore
///  rule that hid a volume is often one of the entries it deleted,
/// so Git reports the volume itself as
///  untracked.
///  The question is therefore asked of everything outside the volumes,
/// and it is asked at all only when a record
///  of this very worktree's own pass exists:
/// an ordinary removal pays one failed read of that record and no Git start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { resuming, reported } = continuationOf({ request, target, registration, foreign, begun });
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Continuation {
    /// Whether a record of this very worktree's own pass exists at all.
    pub(crate) begun: bool,
    /// Whether this command finishes the removal an earlier one began.
    pub(crate) resuming: bool,
    /// What Git reports about the tree,
    /// leaving out what stands inside the volumes this removal keeps,
    /// which the refusal's words need.
    pub(crate) reported: Reported,
}

/// What:
///  The continuation an earlier pass left,
///  when a record of this worktree's own pass says there is one.
/// Why:
///   Both answers come from the one Git start,
///  so the refusal cannot be worded from a different reading than the
///  decision that preceded it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = continuationOf(request, target, registration, foreign, begun);
/// ```
pub(crate) fn continuation_of(request: &RemoveRequest<'_>, target: &Path, registration: &Registration, held: &Held, foreign: &[Boundary]) -> Continuation {
    crate::worktree_removal_around_begun::prune_around_begun(request.common_dir);
    let which: WhichRemoval<'_> = WhichRemoval { id: registration.id.as_os_str(), admin: held.admin_identity, worktree: held.identity, target };
    if !crate::worktree_removal_around_begun::around_begun_by(request.common_dir, which) {
        return Continuation::default();
    }
    let reported: Reported = reported_beside(request, target, registration, foreign);
    // Only entries missing from the tree are the shape a pass leaves. A tree Git reports nothing in was not half emptied by
    // anything, and a tree holding a change of the person's is not this program's to finish.
    let resuming: bool = reported == Reported::OnlyDeletions;
    if resuming {
        debug(TAG, format!("a removal of {} began in an earlier command and was cut short; this one finishes it", target.display()).as_str());
    }
    return Continuation { begun: true, resuming, reported };
}

/// What:
///  The refusal of a removal that would have to delete more than Git does without `--force`,
///  while a volume someone else put there stands below it,
///  worded by what the tree in front of it is.
/// Why:
///   Git's own refusal cannot be the answer here,
/// because handing the tree to Git would let it delete what is mounted
///  at one of these,
/// so the report names each volume and gives the way on;
/// a tree a pass of this program left is finished
///  by the same command when the record proves it,
/// and one whose entries are only missing names `--force` and says
///  that moving a volume out cannot finish it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return refusalBesideAVolume({ request, target, registration, foreign, continuation, reason });
/// ```
pub(crate) fn refusal_beside_a_volume(request: &RemoveRequest<'_>, target: &Path, registration: &Registration, foreign: &[Boundary], continuation: Continuation, reason: &str) -> RemoveOutcome {
    let begun: bool = continuation.begun;
    let reported: Reported = match begun {
        true => continuation.reported,
        false => reported_beside(request, target, registration, foreign),
    };
    let report: String = crate::worktree_removal_goes_around_a_foreign_volume::refused_beside_a_volume_report(
        target,
        request.plan.operand.to_string_lossy().as_ref(),
        foreign,
        crate::worktree_removal_goes_around_a_foreign_volume::BesideAVolume { reason, begun, reported },
    );
    warn(TAG, report.as_str());
    return RemoveOutcome::Refused(report);
}

/// What:
///  What Git reports about this tree,
///  leaving out what stands inside the volumes the removal keeps.
/// Why:
///   The volumes a removal leaves are left out of the question,
/// because the pass deletes the ignore rule that often hid one
///  and Git then reports the volume itself as untracked;
/// a tree whose volumes were not looked at in this command is looked at
///  here, which happens only in a state a record of this program's own pass names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const onlyDeletions = onlyDeletionsBeside({ request, target, registration, foreign });
/// ```
pub(crate) fn reported_beside(request: &RemoveRequest<'_>, target: &Path, registration: &Registration, foreign: &[Boundary]) -> Reported {
    let seen: Vec<Boundary> = match foreign.is_empty() {
        true => volumes_below(target),
        false => foreign.to_vec(),
    };
    let volumes: Vec<&Path> = seen.iter().map(|one| return one.path.as_path()).collect();
    return crate::worktree_removal_only_deletions::reported_outside(request.context, target, registration.admin.as_path(), volumes.as_slice());
}

/// What:
///  Remove everything from the worktree except the volumes it did not make and the directories that lead to them,
///  then forget the worktree in Git.
/// Why:
///   The kernel cannot destroy a subvolume that contains another one,
/// and Git's own removal would delete what is mounted at
///  a mount point,
/// so a tree holding one is emptied here and given up to the kernel nowhere:
/// every other entry goes,
///  what was not the removal's stays with the directories that lead to it,
/// and the registration goes as it does
///  after a destroy.
/// A tree that is empty when the pass ends is removed as any other tree is:
/// the volume went away while the
///  pass ran,
///  and nothing is left to keep.
/// A pass that stops halfway leaves files gone and the registration standing,
/// which is neither of the two
///  answers a person can act on and is said in its own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = goAround({ request, held, registration }); // RemovedAround | Removed | Incomplete
/// ```
pub(crate) fn go_around(request: &RemoveRequest<'_>, held: &Held, registration: &Registration) -> RemoveOutcome {
    let target: &Path = request.plan.target.as_path();
    // The pass deletes the worktree's own entries before the registration goes, so a command killed in it leaves a tree Git
    // reports as changed and a state only `--force` clears. What is written here is the proof that those deletions are this
    // program's own, and it is written before the first entry goes. A record that cannot be written costs that proof and
    // nothing else: the removal goes on, and the words of a refusal name what then finishes the state.
    if let Err(reason) = crate::worktree_removal_around_begun::record_around_begun(
        request.common_dir,
        crate::worktree_removal_around_begun::WhichRemoval { id: registration.id.as_os_str(), admin: held.admin_identity, worktree: held.identity, target },
    ) {
        warn(TAG, format!("the pass that goes around the volume of {} is not recorded, so a command killed in it leaves a worktree only `--force` finishes: {reason}", target.display()).as_str());
    }
    if let Err(error) = empty_directory_around(&held.target, target) {
        let report: String = stopped_around_report(target, error.to_string().as_str());
        warn(TAG, report.as_str());
        return RemoveOutcome::Incomplete(report);
    }
    #[cfg(test)]
    crate::subvolume_remove_listing_test_support::after_the_around_pass(target);
    // What is left is looked at again through the descriptor this removal holds: a volume that went away while the pass ran
    // leaves a tree that is now empty, and the root then goes with it.
    let left: Vec<Boundary> = look_for_boundaries(&held.target, target).found;
    if left.is_empty() {
        if let Err(error) = remove_at(&held.parent, held.name.as_os_str(), true) {
            let report: String = stopped_around_report(target, format!("{} cannot be removed ({error})", target.display()).as_str());
            warn(TAG, report.as_str());
            return RemoveOutcome::Incomplete(report);
        }
        let outcome: RemoveOutcome = finish(held, registration, request);
        if outcome == RemoveOutcome::Removed {
            forget_destroy(request.common_dir, registration.id.as_os_str());
            crate::worktree_removal_around_begun::forget_around_begun(request.common_dir, registration.id.as_os_str());
            // The files went one by one in this command and the empty root went with one `rmdir`, so the ending is the around
            // route's and not the destroy's: nothing was destroyed in one step and nothing waits for the commit service.
            debug(TAG, format!("the volume of {} went away while the pass ran, so the emptied root went with one rmdir", target.display()).as_str());
            return RemoveOutcome::RemovedAroundNothingLeft;
        }
        return finished_around(outcome, target, registration, &[], request);
    }
    let outcome: RemoveOutcome = finish(held, registration, request);
    if outcome != RemoveOutcome::Removed {
        return finished_around(outcome, target, registration, left.as_slice(), request);
    }
    forget_destroy(request.common_dir, registration.id.as_os_str());
    crate::worktree_removal_around_begun::forget_around_begun(request.common_dir, registration.id.as_os_str());
    debug(TAG, format!("removed the worktree {} around {} volume(s) it did not make", target.display(), left.len()).as_str());
    return RemoveOutcome::RemovedAround(left);
}

/// What:
///  What a removal that went around its volumes ended as,
///  with a registration failure worded for the route that emptied the tree instead of destroyed it.
/// Why:
///  The report for a registration that could not be removed after a destroy says the subvolume was destroyed and the
///  directory is already gone,
/// and a tree that was emptied around what was left has neither:
/// its path still stands with the
///  volumes in it.
/// A person reading the wrong one of those would look for a path that is not there, or give up on a
///  removal that one more command finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = finishedAround(ended, target, registration, left, request);
/// ```
pub(crate) fn finished_around(outcome: RemoveOutcome, target: &Path, registration: &Registration, left: &[Boundary], request: &RemoveRequest<'_>) -> RemoveOutcome {
    return match outcome {
        RemoveOutcome::Incomplete(said) => {
            let reason: &str = said.lines().next().unwrap_or(said.as_str());
            let around: String = around_incomplete_report(target, registration.admin.as_path(), request.common_dir, left, reason);
            warn(TAG, around.as_str());
            RemoveOutcome::Incomplete(around)
        }
        other => other,
    };
}
