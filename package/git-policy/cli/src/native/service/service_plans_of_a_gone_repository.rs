//! What:
//!  The pushes a commit service owes a repository that no longer exists:
//!  the retry plan of each branch whose Git directory the filesystem proves gone ends,
//!  and every plan of a repository that is still there stays.
//! Why:
//!  A service that finds its own worktree or repository gone has nothing left to deliver
//! (`service_units_of_removed_worktrees.rs`),
//! and the plans it keeps are the only work that would outlive it:
//! a restarted service reads them (`service_journal_restored_at_start.rs`)
//! and spends a try on a directory nothing can reach,
//! ten minutes at a time,
//! which is the state the owner reported on 2026-11-11:
//! units of deleted throwaway repositories retrying at try 149.
//! The proof is the one the drain already uses
//! (`unreachable_git_dir_proven_gone.rs`),
//! so a plan this ends is a plan the drain would have ended too,
//! and a plan of a repository behind an unmounted mount point is kept for that drive.
//! Every other record in the journal is left alone:
//! the records of batches that landed,
//! the work items of other repositories the same instance was woken for,
//! and anything this release does not explain,
//! because a commit a person made is never discarded on a guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ended = endPlansOfGoneRepositories(journal); // ['refs/heads/main of 0123']
//! ```

use crate::push_retry::PLAN_DIRECTORY;
use crate::push_retry_store::{StoredPlan, load, remove};
use crate::unreachable_git_dir_proven_gone::{Proof, prove, the_mounts};
use std::path::{Path, PathBuf};

/// What:
///  Ends the retry plan of every branch whose Git directory is proven gone,
///  and gives back one line per push that will never be tried again.
/// Why:
///   The lines are what the service says once in its journal,
/// so a person can see which commits of the deleted repository lost their tries.
/// The shape is the one the drain's own line names them in:
/// the branch,
/// then `of`,
/// then the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endPlansOfGoneRepositories(journal) // ['refs/heads/main of 0123456...']
/// ```
pub fn end_plans_of_gone_repositories(journal: &Path) -> Vec<String> {
    let plans: PathBuf = journal.join(PLAN_DIRECTORY);
    let mut ended: Vec<String> = Vec::new();
    for plan in load(plans.as_path()) {
        if !repository_is_gone(&plan) {
            continue;
        }
        remove(plans.as_path(), plan.git_dir.as_path(), plan.item.branch_ref.as_str());
        ended.push(format!("{} of {}", plan.item.branch_ref, plan.item.oid));
    }
    return ended;
}

/// What:
///  Whether the repository a plan pushes to is proven gone:
///  its Git directory is missing,
///  and no worktree of that plan is there to say the directory moved instead.
/// Why:
///   A Git directory that is still there is a repository the push can still reach,
/// whatever a previous round said;
/// a missing one behind a mount point that holds nothing may return with its drive,
/// and the plan waits for it exactly as the drain's own proof says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryIsGone(plan) // false when plan.gitDir resolves, else prove(plan.gitDir, plan.item.repository)
/// ```
fn repository_is_gone(plan: &StoredPlan) -> bool {
    if std::fs::symlink_metadata(plan.git_dir.as_path()).is_ok() {
        return false;
    }
    return matches!(prove(plan.git_dir.as_path(), Some(plan.item.repository.as_path()), &the_mounts()), Proof::Gone(_));
}

/// Controls of the plans of a gone repository stay out of the release executable.
#[cfg(test)]
#[path = "service_plans_of_a_gone_repository_tests.rs"]
mod tests;
