//! What:
//!  The two reads of a branch's gate records that a push makes before it sends anything,
//! each a job of the process's blocking pool awaited by the push's task,
//! and each awaited no longer than the after-commit work's other state steps.
//! Why:
//!  A push runs as a task of the runtime
//! (`service_work.rs`,
//! `push_awaiting`),
//! and these two reads list the worktree's `blocked` and `held` directories
//!  and read every record in them with `std::fs`.
//! A read there waits in the kernel on a slow disk,
//! a lock or a mount that stopped answering,
//! so a worker that made it would hold the thread the runtime needs to run the task that
//!  would end the wait,
//! which is what the package's own `storage-read-on-a-task` rule forbids
//!  (`source_guard/blocking_code_off_the_runtime_rules_tests.rs`):
//! on the current-thread runtime the build falls back to when the system refuses threads
//!  (`foundation/lazy_runtime.rs`) one such read holds the whole service.
//! The claim of the item being pushed keeps being renewed by a thread of its own,
//! so no other instance takes the item over while the read is held,
//! and the push is held for as long as the storage is.
//!  The other storage steps of one drain are jobs of the same pool awaited the same way
//! (`service_work_storage_awaited.rs`),
//! so these two are read by the same shape of step:
//! a place of the process's bound found within the ceiling,
//!  the read on the pool's thread,
//! and an answer that says which of the two failed when either did.
//! Limits:
//!  The ceiling bounds the push's wait and never the read:
//! a read that outlasts it is left behind on its thread holding its place,
//!  exactly as a storage step of the drain is,
//! and the push is answered by its own retry plan.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const blocked = await readOnThePoolWithin(AFTER_COMMIT_STATE_CEILING, () => blockedVerdicts(workDir));
//! const undecided = await readOnThePoolWithin(AFTER_COMMIT_STATE_CEILING, () => undecidedCommits(workDir, branchRef));
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_once_awaited_no_longer_than};
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
use crate::work_item::WorkItem;
use crate::work_item_holding::{BlockedVerdict, blocked_verdicts, undecided_commits};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What:
///  The read of the verdicts a gate kept,
///  with the ceiling its place and its job are given,
///  so a control reaches the bound this step keeps in a moment.
/// Why:
///   The shipped ceiling is sixty seconds,
/// and only a control wants to wait past a read that does not answer in a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blocked = await blockedVerdictsWithinAwaiting(200, workDir);
/// ```
pub(crate) async fn blocked_verdicts_within_awaiting(ceiling: Duration, work_dir: &Path) -> Result<Vec<BlockedVerdict>, String> {
    let directory: PathBuf = work_dir.to_path_buf();
    return match run_a_job_once_awaited_no_longer_than(ceiling, move || { return blocked_verdicts(directory.as_path()); }).await {
        TheAnswerOfAJobAwaited::Ran(verdicts) => Ok(verdicts),
        TheAnswerOfAJobAwaited::NotRun => Err(format!("the gate records of this branch could not be read, because no place of the process's blocking pool came within {ceiling:?}")),
        TheAnswerOfAJobAwaited::NoAnswer => Err(format!("the gate records of this branch gave no answer within {ceiling:?}, so what the gate decided is not known")),
    };
}

/// What:
///  The read of the commits whose gate still runs,
///  with the ceiling its place and its job are given,
///  so a control reaches the bound this step keeps in a moment.
/// Why:
///   The shipped ceiling is sixty seconds,
/// and only a control wants to wait past a read that does not answer in a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const undecided = await undecidedCommitsWithinAwaiting(200, workDir, branchRef);
/// ```
pub(crate) async fn undecided_commits_within_awaiting(ceiling: Duration, work_dir: &Path, branch_ref: &str) -> Result<Vec<String>, String> {
    let directory: PathBuf = work_dir.to_path_buf();
    let branch: String = String::from(branch_ref);
    return match run_a_job_once_awaited_no_longer_than(ceiling, move || { return undecided_commits(directory.as_path(), branch.as_str()); }).await {
        TheAnswerOfAJobAwaited::Ran(commits) => Ok(commits),
        TheAnswerOfAJobAwaited::NotRun => Err(format!("the held records of this branch could not be read, because no place of the process's blocking pool came within {ceiling:?}")),
        TheAnswerOfAJobAwaited::NoAnswer => Err(format!("the held records of this branch gave no answer within {ceiling:?}, so no push can tell which commits of it a gate still holds")),
    };
}

/// What:
///  What a push must know before it sends anything:
/// every verdict a post-commit gate kept,
/// and the commits of the branch whose own gate still runs.
/// Why:
///   The two are read together on the path of every push,
/// and a push that sends past either publishes a commit a gate may keep local.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GateRecords = { blocked: BlockedVerdict[]; undecided: string[] };
/// ```
pub(crate) struct GateRecords {
    /// Every verdict a post-commit gate of this worktree kept.
    pub(crate) blocked: Vec<BlockedVerdict>,
    /// The commits of the pushed branch whose held record says its gate has not decided.
    pub(crate) undecided: Vec<String>,
}

/// What:
///  Both reads a push makes before it sends anything,
///  each a job of the process's blocking pool awaited within the after-commit work's state ceiling,
///  or the words the push answers with when one of them did not answer.
/// Why:
///   The push's round answers `Retry` with these words
/// (`service_work.rs`,
/// `push_awaiting`):
/// nothing of the branch was sent,
///  so the commit is neither lost nor reported as made,
/// the item keeps its place for the next round,
///  and the service's retry plan names the pause to the person
/// (`service_work_settling_a_verdict.rs`).
/// A record that cannot be read is a verdict nobody knows,
/// which is not the same as no verdict,
///  so the words say which of the two readings did not answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { blocked, undecided } = await gateRecordsAwaiting(workDir, item); // throws the words a push answers with
/// ```
pub(crate) async fn gate_records_awaiting(work_dir: &Path, item: &WorkItem) -> Result<GateRecords, String> {
    return gate_records_within_awaiting(the_gate_records_ceiling(), work_dir, item).await;
}

/// What:
///  `gate_records_awaiting` with the ceiling its two places and jobs are given,
///  so a control reaches the bound this step keeps in a moment.
/// Why:
///   The shipped ceiling is sixty seconds,
/// and only a control wants to wait past a record that does not answer in a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const records = await gateRecordsWithinAwaiting(200, workDir, item);
/// ```
pub(crate) async fn gate_records_within_awaiting(ceiling: Duration, work_dir: &Path, item: &WorkItem) -> Result<GateRecords, String> {
    let blocked: Vec<BlockedVerdict> = match blocked_verdicts_within_awaiting(ceiling, work_dir).await {
        Ok(verdicts) => verdicts,
        Err(said) => return Err(unread(item, said.as_str())),
    };
    let undecided: Vec<String> = match undecided_commits_within_awaiting(ceiling, work_dir, item.branch_ref.as_str()).await {
        Ok(commits) => commits,
        Err(said) => return Err(unread(item, said.as_str())),
    };
    return Ok(GateRecords { blocked, undecided });
}

/// What:
///  The words a push answers with when a gate record it must read before it sends anything did not answer.
/// Why:
///   They name the commit,
/// so the person's record says which commit is still only here,
///  what did not answer and within which ceiling,
/// and that the push is tried again,
///  which the retry plan the service keeps then dates
/// (`service_work_handlers_replanning_and_acceptance.rs`,
/// `planned_retry`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = unread(item, 'the gate records of this branch gave no answer within 60s, ...');
/// ```
fn unread(item: &WorkItem, said: &str) -> String {
    return format!("the commit {} was not pushed: {said}. Nothing of this branch was sent, and the push is tried again", item.oid);
}

/// The ceiling a control stated for these reads of this process,
/// in milliseconds,
/// or nothing when it stated none.
/// The release executable keeps nothing of it.
#[cfg(test)]
static A_CEILING_FOR_A_CONTROL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// What:
///  State the ceiling the gate-record reads of this process take,
///  until the guard this answers is dropped,
///  so a control reaches the bound a push keeps in a moment instead of the sixty seconds a shipped read waits.
/// Why:
///   A push whose records did not answer must ask to be tried again,
/// and only a control can reach that answer without waiting the shipped ceiling out;
/// the guard gives the previous ceiling back when it drops,
/// so the short ceiling covers one control's round and never every push of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { using short = holdTheGateRecordReadsTo(200); runOneRound(); }
/// ```
#[cfg(test)]
pub(crate) fn hold_the_gate_record_reads_to(ceiling: Duration) -> ACeilingForTheControl {
    let previous: u64 = A_CEILING_FOR_A_CONTROL.swap(u64::try_from(ceiling.as_millis()).unwrap_or(u64::MAX), std::sync::atomic::Ordering::SeqCst);
    return ACeilingForTheControl { previous };
}

/// The guard of a ceiling a control stated:
/// dropping it gives the ceiling this process had before back.
#[cfg(test)]
pub(crate) struct ACeilingForTheControl {
    /// The ceiling that was in force before the control stated one,
    /// in milliseconds.
    previous: u64,
}

#[cfg(test)]
impl Drop for ACeilingForTheControl {
    fn drop(&mut self) {
        A_CEILING_FOR_A_CONTROL.store(self.previous, std::sync::atomic::Ordering::SeqCst);
    }
}

/// The ceiling these reads take:
/// the one a control stated,
/// or `AFTER_COMMIT_STATE_CEILING`,
/// which is what every other state step of the after-commit work takes.
/// Why:
///   A push runs beside the drain and reads the same kind of storage,
/// so a read that may not end is given the wait the owner's "nothing should ever wait forever or
///  indefinitely" rule allows the drain's own steps
/// (`wait_ceilings.rs`,
/// `AFTER_COMMIT_STATE_CEILING`).
fn the_gate_records_ceiling() -> Duration {
    #[cfg(test)]
    {
        let stated: u64 = A_CEILING_FOR_A_CONTROL.load(std::sync::atomic::Ordering::SeqCst);
        if stated > 0 {
            return Duration::from_millis(stated);
        }
    }
    return AFTER_COMMIT_STATE_CEILING;
}

/// Controls and the measurement of these two steps stay out of the release executable.
#[cfg(test)]
#[path = "gate_records_read_off_the_runtime_tests.rs"]
mod tests;
