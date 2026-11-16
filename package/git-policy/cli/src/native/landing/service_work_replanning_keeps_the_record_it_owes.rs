//! What:
//!  The plan of a push whose end owes the person a record,
//! and the plan of a try that is due again:
//! how either is kept on disk,
//! what the list of the loop is asked of it,
//! and what becomes of a plan whose repository is away or gone.
//! Why:
//!   A try and a record the person is owed are one file,
//! which differs only in the words it carries
//! (`StoredPlan` and `OwedRecord` in `src/native/push/push_retry_store.rs`).
//! The file is written by the worker that decided the branch,
//! before any words that promise it reach the person,
//! so a kill between the two leaves a plan rather than a promise nothing keeps
//! (`Replanning`, `src/native/landing/service_work_handlers_replanning_and_acceptance.rs`).
//! A plan that cannot be written answers that it is not kept,
//! so its caller promises nothing the next service cannot keep.
//! A plan whose repository is not reachable keeps what it owes for the look that follows,
//! and one whose repository is proven gone says its words in the journal,
//! because the directory a record goes in went with the repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await replanning.keepThePlan(...); await replanning.keepTheOwedRecord(...); await keepWhatThePlanOwes(...);
//! ```

use crate::push_retry::{Due, FIRST_PAUSE, RetryChange, pause_before};
use crate::push_retry_store::{OwedRecord, StoredPlan};
use crate::service_work_handlers_replanning_and_acceptance::{PlannedRetry, Replanning, planned_retry};
use crate::work_item::WorkItem;
use std::path::Path;
use std::time::{Duration, Instant};

/// Methods over the plan the workers keep.
impl Replanning {
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
    pub(crate) async fn keep_the_plan(&self, git_dir: &Path, item: &WorkItem, attempt: u32) -> bool {
        return self.keep_a_plan(git_dir, item, attempt, None).await;
    }

    /// What:
    ///  Keep the plan of a push whose end owes the person `owed`,
    /// for the look that writes that record,
    /// and ask the list to look at the branch again when the look is due.
    /// Why:
    ///   The words of the record and the plan that promises to write them are one statement:
    /// a plan removed before the record is written leaves neither the notice nor anything that would write it.
    /// The file goes before the list changes,
    /// as a try's does,
    /// so a kill between the two leaves the words in the journal.
    /// A plan the storage refused answers that it is not kept,
    /// and the list is asked for the look anyway:
    /// the caller keeps the push as a try in that case, whose own end says the same words again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kept = await savePlans(this.plans, { gitDir, item, attempt, owes });
    /// this.changes.push({ plan: { gitDir, item, attempt, due, owes } });
    /// return kept;
    /// ```
    pub(crate) async fn keep_the_owed_record(&mut self, git_dir: &Path, item: &WorkItem, attempt: u32, owed: OwedRecord) -> bool {
        let kept: bool = self.keep_a_plan(git_dir, item, attempt, Some(owed.clone())).await;
        let pause: Duration = pause_before(attempt, self.first.unwrap_or(FIRST_PAUSE));
        self.changes.push(RetryChange::Plan { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt, due: Instant::now() + pause, owes: Some(owed) });
        return kept;
    }

    /// What:
    ///  Keep one plan on disk,
    /// answer whether it is kept,
    /// and say so when the storage refused it.
    /// Why:
    ///   A try and a record the person is owed are the same file,
    /// so both are written here and only the words they carry differ.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// return savePlans(this.plans, { gitDir, item, attempt, owes });
    /// ```
    async fn keep_a_plan(&self, git_dir: &Path, item: &WorkItem, attempt: u32, owes: Option<OwedRecord>) -> bool {
        let Some(directory) = self.plans.as_deref() else {
            // A plan kept in memory only is kept as long as the service that keeps it lives.
            return true;
        };
        let plan: StoredPlan = StoredPlan { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt, owes };
        return match crate::push_retry_store::save_awaiting(directory, &plan).await {
            Ok(()) => true,
            Err(reason) => {
                crate::service_warn!(
                    "the plan of {:?} {} could not be kept for a restart: {reason}",
                    item.branch_ref,
                    crate::service_timing::in_the_repository_at(item.repository.as_path())
                );
                false
            }
        };
    }
}

/// What:
///  Keep what a plan owes the person for the look that follows,
/// for a Git directory that cannot be reached now:
/// a try keeps the pause that comes next,
/// and a record the plan owes keeps its words and its looks.
/// Why:
///   The record's own path is inside the unreachable directory,
/// so a look can write nothing there and is postponed rather than turned into a try that would drop the words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// due.owes ? await replanning.keepTheOwedRecord(gitDir, due.item, due.attempt + 1, due.owes) : await plannedRetry(gitDir, replanning, first, due.item, due.attempt + 1, cause);
/// ```
pub(crate) async fn keep_what_the_plan_owes(replanning: &mut Replanning, git_dir: &Path, due: &Due, first: Duration, cause: &str) {
    match due.owes.clone() {
        Some(owed) => {
            let _kept: bool = replanning.keep_the_owed_record(git_dir, &due.item, due.attempt + 1, owed).await;
        }
        None => {
            let _said: PlannedRetry = planned_retry(git_dir, replanning, first, &due.item, due.attempt + 1, cause).await;
        }
    }
}

/// What:
///  Say what a record a plan owes holds,
/// for a plan whose repository is proven gone.
/// Why:
///   The directory a record goes in went with the repository,
/// so the journal is the only place left that can say what became of the backup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (due.owes) logError(`the record of ${due.item.oid} cannot be written: ${due.owes.words}`);
/// ```
pub(crate) fn say_what_a_gone_plan_owes(due: &Due) {
    let Some(owed) = due.owes.as_ref() else { return };
    crate::service_error!(
        "the record of the push of {} cannot be written for the person, because the repository is gone, so the journal holds what it says:\n{}",
        due.item.oid,
        owed.words
    );
}
