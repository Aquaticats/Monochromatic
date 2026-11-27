//! What:
//!  The removal a service makes around a volume that is not the removal's own,
//!  in the service's own process,
//!  and the words for what was left where a person finds it.
//! Why:
//!  The record a service acts on was written before the volume may have appeared,
//!  and Git must not see a tree holding one:
//!  its removal deletes what is mounted at a mount point and unlinks the files of a nested worktree.
//!  The steps are the person's own removal,
//!  made here with the checks that removal makes,
//!  so the service and the command cannot disagree about which tree may go.
//!  A removal that refuses keeps the worktree and says why,
//!  and one that removed what it could and stopped keeps it too,
//!  with the words for a tree that is partly gone.
//!  It is its own file because the service's own removal reached the repository's length limit
//!  with it in it,
//!  and because it is the one answer of that removal that reads the volume report a person sees.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! removeAroundWhatIsNotThisRemovals(item, target, registration): Verdict
//! ```

// What:     `use crate::service_work::Verdict;` imports the three answers a service call
//           gives: done, try again, or failed with words.
// Why:      The caller of this removal is the service's own record handler, which reads
//           exactly that answer.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Verdict } from '../service_work';
// ```
use crate::service_work::Verdict;
// What:     `use crate::transaction_git::GitContext;` imports how a removal starts real
//           Git, which the service reads from the repository it serves.
// Why:      The removal makes the person's own checks, and each of them starts Git.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { GitContext } from '../transaction_git';
// ```
use crate::transaction_git::GitContext;
// What:     `use crate::work_item::WorkItem;` imports the durable record of one
//           handed-over removal.
// Why:      The worktree this removal acts on is the one that record names.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { WorkItem } from '../work_item';
// ```
use crate::work_item::WorkItem;
// What:     `use crate::worktree_command_facts::RemovePlan;` imports the plan of one
//           `git worktree remove`.
// Why:      The person's removal is made here, so this file builds the plan the command
//           line would build, unforced.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { RemovePlan } from './worktree_command_facts';
// ```
use crate::worktree_command_facts::RemovePlan;
// What:     `use crate::worktree_landed_registrations::branch_of_registration;` imports the
//           branch a registration has checked out.
// Why:      The record of what was left names the branch, so a person can find it again.
//
// In TS you'd write (pseudocode):
// ```ts
// import { branchOfRegistration } from './worktree_landed_registrations';
// ```
use crate::worktree_landed_registrations::branch_of_registration;
// What:     `use crate::worktree_remove_hand_over_begun_record::forget_begun;` imports the
//           forget of the record that says a removal of this worktree began.
// Why:      The worktree is gone, so the record of a half-done removal of it goes too.
//
// In TS you'd write (pseudocode):
// ```ts
// import { forgetBegun } from './worktree_remove_hand_over_begun_record';
// ```
use crate::worktree_remove_hand_over_begun_record::forget_begun;
// What:     `use crate::worktree_remove_hand_over_kept_worktree_notice::keep_the_worktree;`
//           imports the notice that says a worktree was kept and why.
// Why:      A refusal keeps the worktree and must say so where the next command reads it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { keepTheWorktree } from './worktree_remove_hand_over_kept_worktree_notice';
// ```
use crate::worktree_remove_hand_over_kept_worktree_notice::keep_the_worktree;
// What:     `use crate::worktree_remove_hand_over_service_git_and_pool::the_service_git;`
//           imports the Git the service starts for the repository a record names.
// Why:      A service has no command line to read a repository from, so the record is
//           what names it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { theServiceGit } from './worktree_remove_hand_over_service_git_and_pool';
// ```
use crate::worktree_remove_hand_over_service_git_and_pool::the_service_git;
// What:     `use crate::worktree_remove_outcome::RemoveOutcome;` imports the endings a
//           removal answers with.
// Why:      The handler answers the service differently for each ending.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { RemoveOutcome } from './worktree_remove_outcome';
// ```
use crate::worktree_remove_outcome::RemoveOutcome;
// What:     `use crate::worktree_remove_registry::Registration;` imports which worktree
//           registration this is.
// Why:      The removal answers for that registration and for no other.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Registration } from './worktree_remove_registry';
// ```
use crate::worktree_remove_registry::Registration;
// What:     `use crate::worktree_remove_request_and_last_look::RemoveRequest;` imports what
//           one removal is asked to do.
// Why:      The request this handler builds is the person's own, with no last look.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { RemoveRequest } from './worktree_remove_request_and_last_look';
// ```
use crate::worktree_remove_request_and_last_look::RemoveRequest;
// What:     `use std::path::Path;` imports the borrowed path type.
// Why:      The target is lent to this removal by the caller that read the record.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent, a path is a plain string.
// ```
use std::path::Path;

/// What:
///  Remove the worktree around what someone else put inside it,
///  in this service's own process,
///  and say what was left where a person finds it.
/// Why:
///   The record this service acts on was written before the volume may have appeared,
/// and Git must not see a tree
///  holding one:
/// its removal deletes what is mounted at a mount point and unlinks the files of a nested worktree.
/// The steps are
///  the person's own removal,
/// made here with the checks that removal makes,
/// so the service and the command cannot disagree about
///  which tree may go.
/// A removal that refuses keeps the worktree and says why,
/// and a removal that removed what it could and stopped keeps
///  it too,
///  with the words for a tree that is partly gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeAroundWhatIsNotThisRemovals(item, target, registration): Verdict
/// ```
pub(crate) fn remove_around_what_is_not_this_removals(
    item: &WorkItem,
    target: &Path,
    registration: &Registration,
) -> Verdict {
    let context: GitContext = match the_service_git(item) {
        Ok(found) => found,
        Err(reason) => return Verdict::Retry(reason),
    };
    let plan: RemovePlan = RemovePlan {
        target: target.to_path_buf(),
        operand: target.as_os_str().to_os_string(),
        force: false,
    };
    let request: RemoveRequest<'_> = RemoveRequest {
        context: &context,
        plan: &plan,
        common_dir: item.git_dir.as_path(),
        last_look: None,
    };
    match crate::worktree_remove::remove_with(&crate::worktree_remove::LinuxOperations, &request) {
        RemoveOutcome::Removed => {
            // The record that says which worktree a half-done removal of this id was belongs to that removal, and the
            // worktree is gone: a later attempt for the name must not read it as its own.
            forget_begun(item.git_dir.as_path(), item.id().as_str());
            return Verdict::Done;
        }
        RemoveOutcome::RemovedAroundNothingLeft => {
            for line in
                crate::worktree_removal_goes_around_a_foreign_volume::emptied_around_report(target)
                    .lines()
            {
                crate::service_log!("{line}");
            }
            forget_begun(item.git_dir.as_path(), item.id().as_str());
            return Verdict::Done;
        }
        RemoveOutcome::DestroyedInPlace => {
            // The files are gone and nothing waits here; the record that names which worktree a half-done removal of this id
            // was belongs to that removal, so it goes with the worktree.
            forget_begun(item.git_dir.as_path(), item.id().as_str());
            return Verdict::Done;
        }
        RemoveOutcome::RemovedAround(left) => {
            let branch: String = branch_of_registration(registration.admin.as_path())
                .unwrap_or_else(|| return String::from("a head with no branch"));
            if let Err(reason) =
                crate::worktree_removal_goes_around_a_foreign_volume::record_left_in_place(
                    item.git_dir.as_path(),
                    target,
                    branch.as_str(),
                    left.as_slice(),
                )
            {
                crate::service_warn!(
                    "the worktree {} was removed around what was left in it, and the record of that could not be written: {reason}",
                    target.display()
                );
            }
            for line in crate::worktree_removal_goes_around_a_foreign_volume::around_removal_report(
                target,
                left.as_slice(),
            )
            .lines()
            {
                crate::service_log!("{line}");
            }
            forget_begun(item.git_dir.as_path(), item.id().as_str());
            return Verdict::Done;
        }
        RemoveOutcome::Refused(report) => {
            return keep_the_worktree(
                item,
                target,
                registration.admin.as_path(),
                report.as_str(),
                &[],
            );
        }
        RemoveOutcome::Incomplete(report) => {
            return keep_the_worktree(
                item,
                target,
                registration.admin.as_path(),
                report.as_str(),
                &[],
            );
        }
        // Nothing here was changed and Git's own removal is not this tree's to take, so the worktree stays and the person reads
        //  why. A record that names no removal this program can make must not be read as one that happened.
        RemoveOutcome::LeftToGit => {
            return keep_the_worktree(
                item,
                target,
                registration.admin.as_path(),
                "a subvolume or mount point that is not this removal's stands inside it, and the removal could not prove it may take the rest",
                &[],
            );
        }
    }
}
