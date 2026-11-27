//! What:
//!  The storage steps of one drain of work,
//!  each a job of the process's blocking pool awaited by a task of its runtime:
//! the claiming of a directory's records,
//!  which of them are acceptable,
//! the settling of those that are not or are done,
//! the look that finds a claim taken over,
//!  and the records a push leaves for the person.
//! Why:
//!  The drain runs as a task of the runtime (`service_work_drain_pending_claims.rs`),
//! and every one of these steps reads or
//!  renames files,
//! which can wait on a disk,
//! a network file system or a FUSE mount:
//! a worker that waited there would hold a
//!  thread the runtime needs,
//! so each step goes to the pool and the worker awaits it.
//! The held claims go with it:
//!  they have one owner at a time
//! (`service_claim_renewal.rs`,
//! `HeldClaims`),
//! so a step that reads or renames a claim takes the
//!  whole set,
//! and hands it back with its answer;
//! nothing locks it.
//! A step the system refuses a thread for does nothing and says so;
//! the step that carried the claims cannot hand them
//!  back,
//! so it ends the round there,
//! and every record the round had claimed keeps its lease for the next one
//!  to find.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claimed = await claimAllAwaiting(held, workDir); const [ok, refused] = await acceptableAwaiting(gitDir, claimed); await settleAllAwaiting(held, workDir, refused, FAILED)
//! ```
//!
//! How one step is run is declared in `service_work_storage_step_awaited.rs`,
//! and the steps that touch the queue's own records in `service_work_storage_claims_awaited.rs`:
//! this file serves both.

use crate::claim_time::ClaimTime;
use crate::push_failure_notice::{
    PushOutcome,
    clear_push_failures_before,
    record_push_failure,
    record_push_report,
    record_push_skip,
};
use crate::service_work_handlers_replanning_and_acceptance::{
    MAX_TRIES,
    SkipNotice,
};
use crate::service_work_storage_step_awaited::{
    a_storage_step_awaited_no_longer_than,
    refused,
    the_claim_steps_ceiling,
};
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
use crate::work_item_recovery::{
    Recovered,
    next_claim_expiry,
    requeue_expired_reporting_telling,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// A control states the ceiling its own claim step takes.
#[cfg(test)]
pub(crate) use crate::service_work_storage_claims_awaited::claim_all_within_awaiting;
/// What:
///  The steps that touch the queue's own records,
/// declared beside this file and served by it,
/// so the paths its callers name stay the ones they named.
pub(crate) use crate::service_work_storage_claims_awaited::{
    acceptable_awaiting,
    claim_all_awaiting,
    settle_all_awaiting,
    still_held_awaiting,
};
/// A control states the ceiling the claim steps of this process take.
#[cfg(test)]
pub(crate) use crate::service_work_storage_step_awaited::hold_the_claim_steps_to;

/// What:
///  Forget this branch's push failures written before `since`,
/// because a push of it has just gone through.
/// Why:
///   The records are files in the repository's own directory,
/// so reading and removing them is storage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearPushFailuresBeforeAwaiting(gitDir, branch, since)
/// ```
pub(crate) async fn clear_push_failures_before_awaiting(
    git_dir: &Path,
    branch_ref: &str,
    since: std::time::SystemTime,
) {
    let directory: PathBuf = git_dir.to_path_buf();
    let branch: String = String::from(branch_ref);
    match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the removal of this branch's earlier push records",
        "none is removed",
        move || {
            clear_push_failures_before(directory.as_path(), branch.as_str(), since);
        },
    )
    .await
    {
        Ok(()) => {}
        Err(said) => {
            refused(said.as_str());
        }
    }
}

/// What:
///  Leave the person a record that the push of `commit` did not land,
/// with what became of it.
/// Why:
///   The record is a file written into the repository's directory,
/// which can wait on storage like any other write.
/// A refusal is answered with the words it refused,
/// in the journal:
/// a caller leaves the work unsettled so that a later try says it again,
/// and a journal is what a
///  person reading the log has until then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushFailureAwaiting(gitDir, branch, commit, outcome, said)
/// ```
pub(crate) async fn record_push_failure_awaiting(
    git_dir: &Path,
    branch_ref: &str,
    commit: &str,
    outcome: PushOutcome,
    said: &str,
) -> Result<(), String> {
    let directory: PathBuf = git_dir.to_path_buf();
    let branch: String = String::from(branch_ref);
    let oid: String = String::from(commit);
    let words: String = String::from(said);
    let for_the_journal: String = crate::push_failure_notice::clean_words(said);
    let refusal: Result<(), String> = match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the record of a push that did not land",
        "none was written",
        move || {
            return record_push_failure(
                directory.as_path(),
                branch.as_str(),
                oid.as_str(),
                outcome,
                words.as_str(),
            );
        },
    )
    .await
    {
        // A record that was not written leaves the item to its lease, and the next round tells the person.
        Ok(answer) => answer,
        Err(refusal) => Err(refusal),
    };
    if let Err(reason) = &refusal {
        crate::service_error!(
            "the push of {commit} did not land, and what Git said could not be recorded for the person: {reason}. What it said:\n{for_the_journal}"
        );
    }
    return refusal;
}

/// What:
///  Leave the person a record of what a push that landed said,
/// with every line the push chose to show.
/// Why:
///   The record is a file written into the repository's directory,
/// which can wait on storage like any other write,
/// and it is the only copy of what the server and the reconcile said:
/// nothing makes an accepted push say it again.
///  A refusal is said in the journal with the words it refused,
/// and the caller keeps the item out of `done`,
/// because nothing makes an accepted push say them again
/// (`service_work_settling_a_verdict.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushReportAwaiting(gitDir, branch, commit, said)
/// ```
pub(crate) async fn record_push_report_awaiting(
    git_dir: &Path,
    branch_ref: &str,
    commit: &str,
    said: &str,
) -> Result<(), String> {
    let directory: PathBuf = git_dir.to_path_buf();
    let branch: String = String::from(branch_ref);
    let oid: String = String::from(commit);
    let words: String = String::from(said);
    // What the refusal says, for a person who can read the journal: the words are the only copy of what a server said about a push
    //  the remote accepted, and a notice this storage refused is the one place they would have been kept.
    let for_the_journal: String = crate::push_failure_notice::clean_words(said);
    let refusal: Result<(), String> = match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the record of what a push that landed said",
        "none was written",
        move || {
            return record_push_report(
                directory.as_path(),
                branch.as_str(),
                oid.as_str(),
                words.as_str(),
            );
        },
    )
    .await
    {
        Ok(answer) => answer,
        Err(refusal) => Err(refusal),
    };
    if let Err(reason) = &refusal {
        crate::service_error!(
            "the push of {commit} landed, but what it said could not be recorded for the person: {reason}. What it said:\n{for_the_journal}"
        );
    }
    return refusal;
}

/// What:
///  Leave the person a record that a push was skipped,
/// with the words the push gave.
/// Why:
///   The record is a file,
/// as for a failed push,
/// and a refusal is answered with the words it refused in the journal,
/// because the caller leaves the work unsettled and a later try says it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushSkipAwaiting(gitDir, branch, commit, said)
/// ```
pub(crate) async fn record_push_skip_awaiting(
    git_dir: &Path,
    branch_ref: &str,
    commit: &str,
    said: &str,
) -> Result<(), String> {
    let directory: PathBuf = git_dir.to_path_buf();
    let branch: String = String::from(branch_ref);
    let oid: String = String::from(commit);
    let words: String = String::from(said);
    let for_the_journal: String = crate::push_failure_notice::clean_words(said);
    let refusal: Result<(), String> = match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the record of a push that was skipped",
        "none was written",
        move || {
            return record_push_skip(
                directory.as_path(),
                branch.as_str(),
                oid.as_str(),
                words.as_str(),
            );
        },
    )
    .await
    {
        Ok(answer) => answer,
        Err(refusal) => Err(refusal),
    };
    if let Err(reason) = &refusal {
        crate::service_error!(
            "the push of {commit} was skipped, and why could not be recorded for the person: {reason}. What the service said:\n{for_the_journal}"
        );
    }
    return refusal;
}

/// What:
///  How long until the first claim still held in `work_dir` runs out its lease,
/// or nothing when none is held;
/// a look that could not be made is a refusal so its caller can look again rather than take it as proof that nothing is held.
/// Why:
///   The answer is a listing of the claim names,
/// and it is what wakes the service once another instance's lease is over,
/// so a failure to
///  read it is said and given back as a failure rather than read as "nothing is held".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await nextClaimExpiryAwaiting(workDir, now, lease)
/// ```
pub(crate) async fn next_claim_expiry_awaiting(
    work_dir: &Path,
    now: &ClaimTime,
    lease: Duration,
) -> Result<Option<Duration>, String> {
    let directory: PathBuf = work_dir.to_path_buf();
    let reading: ClaimTime = now.clone();
    return match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the look that finds when another instance's claim runs out",
        "no wait is answered",
        move || {
            return next_claim_expiry(directory.as_path(), &reading, lease);
        },
    )
    .await
    {
        Ok(wait) => Ok(wait),
        Err(said) => {
            refused(said.as_str());
            Err(said)
        }
    };
}

/// What:
///  Take back the claims of records whose holder stopped renewing them,
/// and leave a notice for the person about each record given up on,
///  answering whether the step ran.
/// Why:
///   It lists the claim names and the states and moves records,
/// which is storage;
/// a lookup of the person's notice is written from
///  inside the same step (`SkipNotice`),
/// so the words of a place that could not be reached are said where they are learned.
///  The answer is the same one the claim step gives the round:
/// a step that got no place claimed nothing,
///  and every pending record of this directory then waits for a wake nothing sends unless the round names its own pause.
///  It takes the ceiling the claim steps take,
/// which is `AFTER_COMMIT_STATE_CEILING` unless a control states another,
/// because taking a claim back is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = await requeueExpiredAwaiting(workDir, gitDir, now, lease)
/// ```
pub(crate) async fn requeue_expired_awaiting(
    work_dir: &Path,
    git_dir: &Path,
    now: &ClaimTime,
    lease: Duration,
) -> bool {
    let directory: PathBuf = work_dir.to_path_buf();
    let repository: PathBuf = git_dir.to_path_buf();
    let reading: ClaimTime = now.clone();
    // A record given up on is a notice about this repository, which the job says with its own path so nothing is lent to it.
    return match a_storage_step_awaited_no_longer_than(
        the_claim_steps_ceiling(),
        "the taking back of claims whose holder stopped renewing them",
        "none is taken back",
        move || {
            let _recovered: Recovered = requeue_expired_reporting_telling(
                directory.as_path(),
                &reading,
                lease,
                MAX_TRIES,
                &SkipNotice(repository.as_path()),
            );
        },
    )
    .await
    {
        Ok(()) => true,
        Err(said) => {
            refused(said.as_str());
            false
        }
    };
}

/// What:
///  Whether this directory is the instance's own Git directory to drain.
/// Why:
///   The look at the instance's record is storage,
/// and a directory that cannot be reached runs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reachable = await instanceOfAwaiting(gitDir)
/// ```
pub(crate) async fn instance_of_awaiting(
    git_dir: &Path
) -> Result<String, crate::service_instance::GitDirUnreachable> {
    let directory: PathBuf = git_dir.to_path_buf();
    return match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the look for this directory's instance record",
        "it is not read",
        move || {
            return crate::service_instance::try_instance_of(directory.as_path());
        },
    )
    .await
    {
        Ok(answer) => answer,
        Err(said) => {
            refused(said.as_str());
            Err(crate::service_instance::GitDirUnreachable {
                path: git_dir.to_path_buf(),
                reason: said,
            })
        }
    };
}

/// Storage-step controls stay out of the release executable.
#[cfg(test)]
#[path = "service_work_storage_awaited_tests.rs"]
mod tests;
