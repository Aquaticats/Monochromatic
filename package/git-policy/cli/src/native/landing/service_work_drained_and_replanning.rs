//! What:
//!  What one drain of a directory hands back (`Drained`),
//!  and the retry state it works with (`Replanning`):
//!  the wait before the directory is looked at again,
//!  the changes the drain asks of the plan,
//!  and the folder a plan is written to and taken out of.
//! Why:
//!  A drain runs on a worker and cannot change the service loop's plan,
//!  so it returns what it did and the loop applies it;
//!  the file of a plan a drain forgets goes on the thread that decided the
//!  branch needs no new try,
//!  which is the last place the file system can be asked before the file is
//!  gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Drained = { wait?: Duration, changes: RetryChange[] };
//! type Replanning = { first?: Duration, changes: RetryChange[], plans?: string };
//! await replanning.forget(gitDir, branchRef);
//! ```

use crate::push_retry::{
    Due,
    RetryChange,
};
use crate::work_item::WorkItem;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// What:
///  What one drain of a directory hands back:
/// how long until it must be looked at again for a claim another instance holds,
/// and
///  the changes it asks of the retry plan.
/// Why:
///   A drain runs on a worker and cannot change the plan,
/// which the service loop owns;
/// it returns what it did and the loop
///  applies it (`PushRetries::apply`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Drained = { wait?: number; changes: RetryChange[] };
/// ```
#[derive(Debug, Default)]
pub struct Drained {
    /// How long until the caller must look at this directory again:
    /// the pause a drain that left housekeeping for a later look asked for,
    /// or the end of the earliest claim it keeps,
    /// or nothing when neither is left.
    pub wait: Option<Duration>,
    /// The plans and the forgets the drain asks of the retry plan,
    /// in the order they were made.
    pub changes: Vec<RetryChange>,
}

/// What:
///  The retry state one drain works with:
/// the first pause (nothing when a retryable push is to be reported as failed) and the
///  changes it has asked of the plan so far.
/// Why:
///   One value is passed down to the places that plan or forget a retry,
/// so none of them reads the plan or shares it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Replanning = { first?: number; changes: RetryChange[] };
/// ```
pub(crate) struct Replanning {
    /// The pause before the first new try,
    /// or nothing for a drain that keeps no plan.
    pub(crate) first: Option<Duration>,
    /// What was asked of the plan.
    pub(crate) changes: Vec<RetryChange>,
    /// The directory the plan's files are kept in,
    /// which is where this drain writes the plan it promises and removes the one it forgets,
    /// or nothing for a plan kept in memory only.
    pub(crate) plans: Option<PathBuf>,
}

/// Methods over the state.
impl Replanning {
    /// What:
    ///  Ask the plan to drop the branch,
    /// and take its file out of the journal now.
    /// Why:
    ///   The file is what a restarted service rebuilds the plan from,
    /// so a forget that reached only the list of this process would leave the next service trying a push that has already landed
    ///  (finding 3 of the review of `bc3d54f9cd`).
    /// It goes here,
    /// on the thread that decided the branch needs no new try,
    /// as the plan a worker promises is already written here,
    ///  and not by a round that may reach the journal later,
    /// when a newer plan of the branch may have been written since
    ///  (finding 4 of the same review).
    /// A removal that fails is said,
    /// and the next start reads the file again.
    /// The removal is a job of the process's blocking pool,
    /// awaited here no longer than the journal's own state step:
    /// this runs inside the drain,
    /// which is a task of the runtime,
    /// and a task that read or removed a file itself would hold the worker the runtime needs
    /// (`blocking_jobs_within_the_budget.rs`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await forget(gitDir, branch): await removePlans(this.plans, gitDir, branch); this.changes.push({ forget: { gitDir, branch } });
    /// ```
    pub(crate) async fn forget(
        &mut self,
        git_dir: &Path,
        branch_ref: &str,
    ) {
        self.take_the_plan_file(git_dir, branch_ref).await;
        self.push_forget(git_dir, branch_ref, false, None);
    }

    /// What:
    ///  Ask the plan to drop the branch of a push whose repository a reading of the filesystem proved gone,
    /// and take its file out of the journal unless the repository is back.
    /// Why:
    ///   That reading is a point in time,
    /// and this runs after it:
    /// a repository renamed back by then is there again,
    ///  so its plan must not be dropped on the stale proof.
    /// The file goes where every forget's file goes,
    /// on the thread that decided the branch needs no new try,
    /// so this is the last place the filesystem can be asked before the file is gone;
    /// the applier asks it once more before it drops the branch from the list
    /// (`push_retry.rs`,
    /// `PushRetries::apply`),
    ///  which is what keeps a repository that came back during the round trying,
    /// and the plan this reading ended travels with the ask,
    /// so the repository that is back keeps the try the round took out of the plan
    ///  rather than only its on-disk file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await forgetWhenUnreachable(gitDir, due):
    ///   if (!isDirectory(gitDir)) await removePlans(this.plans, gitDir, due.item.branchRef);
    ///   this.changes.push({ forget: { gitDir, branchRef: due.item.branchRef, onlyWhileUnreachable: true, taken: due } });
    /// ```
    pub(crate) async fn forget_when_unreachable(
        &mut self,
        git_dir: &Path,
        due: &Due,
    ) {
        if !git_dir.is_dir() {
            self.take_the_plan_file(git_dir, due.item.branch_ref.as_str())
                .await;
        }
        self.push_forget(
            git_dir,
            due.item.branch_ref.as_str(),
            true,
            Some(due.clone()),
        );
    }

    /// Removes the plan's file of a branch from the journal,
    /// where a directory keeps one at all,
    /// as a job of the process's blocking pool.
    /// Why:
    ///   The removal lists the journal,
    /// reads each file of the branch to prove it is the branch's,
    /// and unlinks it,
    /// and any of those can wait in the kernel on a disk,
    /// a lock or a mount that stopped answering.
    /// The caller is a task of the runtime,
    /// which may not make that wait itself.
    async fn take_the_plan_file(
        &mut self,
        git_dir: &Path,
        branch_ref: &str,
    ) {
        let Some(plans) = self.plans.clone() else {
            return;
        };
        let (repository, branch): (PathBuf, String) =
            (git_dir.to_path_buf(), String::from(branch_ref));
        if let Err(reason) = crate::push_retry_store::remove_awaiting(
            plans.as_path(),
            repository.as_path(),
            branch.as_str(),
        )
        .await
        {
            crate::service_warn!(
                "the retry plan file of {branch} {} could not be removed, so the next start reads it again: {reason}",
                crate::service_timing::in_the_repository_at(repository.as_path())
            );
        }
    }

    /// The one place a forget is added,
    /// so both kinds name the directory and the branch the same way.
    fn push_forget(
        &mut self,
        git_dir: &Path,
        branch_ref: &str,
        only_while_unreachable: bool,
        taken: Option<Due>,
    ) {
        self.changes.push(RetryChange::Forget {
            git_dir: git_dir.to_path_buf(),
            branch_ref: branch_ref.to_string(),
            only_while_unreachable,
            taken,
        });
    }

    /// What:
    ///  Keep the plan of the try `attempt` of the branch of `item` on disk now,
    /// before any words that promise it reach the person,
    /// and answer whether it
    ///  is kept.
    /// Why:
    ///   The loop applies the changes a worker asks only once every repository of the round is drained,
    /// so a kill before it applies them would take the plan,
    /// and a notice that says the service keeps trying would be the only statement left and it would be false.
    /// A worker writes only its own branch's plan,
    /// whose file name the branch key and the try number make,
    /// and the loop writes no file:
    /// what it applies is the list it reads.
    ///  A plan that cannot be written answers that it is not kept,
    /// so the caller promises nothing the next service cannot keep.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// return savePlans(this.plans, { gitDir, item, attempt });
    /// ```
    pub(crate) async fn keep_the_plan(
        &self,
        git_dir: &Path,
        item: &WorkItem,
        attempt: u32,
    ) -> bool {
        let Some(directory) = self.plans.as_deref() else {
            // A plan kept in memory only is kept as long as the service that keeps it lives.
            return true;
        };
        let plan: crate::push_retry_store::StoredPlan = crate::push_retry_store::StoredPlan {
            git_dir: git_dir.to_path_buf(),
            item: item.clone(),
            attempt,
        };
        return match crate::push_retry_store::save_awaiting(directory, &plan).await {
            Ok(()) => true,
            Err(reason) => {
                crate::service_warn!(
                    "the plan to push {:?} again {} could not be kept for a restart: {reason}",
                    item.branch_ref,
                    crate::service_timing::in_the_repository_at(item.repository.as_path())
                );
                false
            }
        };
    }
}
