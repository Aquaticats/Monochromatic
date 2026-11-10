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

use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::claim_time::ClaimTime;
use crate::push_failure_notice::{PushOutcome, clear_push_failures_before, record_push_failure, record_push_skip};
use crate::service_claim_renewal::HeldClaims;
use crate::service_work_handlers_replanning_and_acceptance::{
    MAX_TRIES, SERVICE_KINDS, SkipNotice, is_acceptable, settle_all, still_held,
};
use crate::work_item::{Claimed, claim_all_of};
use crate::work_item_recovery::{Recovered, next_claim_expiry, requeue_expired_reporting_telling};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// What:
///  Say that a storage step did not run,
///  and carry on with what the drain can still do.
/// Why:
///   The step changed nothing,
/// so nothing is lost:
/// the records it would have moved keep their lease and the next
///  round finds them,
/// and a caller told nothing would read the silence as a drain that did all of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = (what, reason) => service_warn(`${what} did not run: ${reason}`)
/// ```
fn refused(step: &str, reason: &BlockingJobError) {
    crate::service_warn!("{step} did not run, so the records it would have moved keep their lease: {reason}");
}

/// What:
///  Claim every pending record of `work_dir`.
/// Why:
///   The listing and the renames are storage,
/// and the answer is the drain's first step of a round.
/// An error is the reason the directory could not be claimed,
///  which ends the round where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimed = await claimAllAwaiting(workDir)
/// ```
pub(crate) async fn claim_all_awaiting(work_dir: &Path) -> Result<Vec<Claimed>, String> {
    let directory: PathBuf = work_dir.to_path_buf();
    return match run_blocking_job_within_the_budget(move || { return claim_all_of(directory.as_path(), &SERVICE_KINDS); }).await {
        Ok(taken) => taken,
        Err(reason) => Err(format!("the look for this directory's pending records did not run, so none was claimed: {reason}")),
    };
}

/// What:
///  Split the claimed records into those this service may act on and those it may not.
/// Why:
///   Which of them is acceptable is a look at the repository's own directories,
/// which is storage like the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [acceptable, refused] = await acceptableAwaiting(gitDir, claimed)
/// ```
pub(crate) async fn acceptable_awaiting(git_dir: &Path, claimed: Vec<Claimed>) -> (Vec<Claimed>, Vec<Claimed>) {
    let directory: PathBuf = git_dir.to_path_buf();
    return match run_blocking_job_within_the_budget(move || {
        let mut acceptable: Vec<Claimed> = Vec::new();
        let mut refused_here: Vec<Claimed> = Vec::new();
        for item in claimed {
            if is_acceptable(&item.item, directory.as_path()) {
                acceptable.push(item);
            } else {
                refused_here.push(item);
            }
        }
        return (acceptable, refused_here);
    })
    .await
    {
        Ok((acceptable, refused_here)) => (acceptable, refused_here),
        // Nothing is decided on a job that did not run, so no record is failed for a reason the service did not read.
        Err(reason) => {
            refused("the look that decides which records are acceptable", &reason);
            (Vec::new(), Vec::new())
        }
    };
}

/// What:
///  Move the claims of `claimed` to `state` and stop renewing them,
/// and hand the claim set back to its owner.
/// Why:
///   A settlement renames a file and finds it by its generation,
/// so it is storage,
/// and the set it reads is the owner's,
///  which the job takes and gives back.
/// A claim another instance took over is said and left to it,
/// as the synchronous step does.
/// A job the system refuses a thread for does not run,
/// so the set goes with it and nothing is answered:
/// the caller ends the round rather than holding a set it cannot settle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const back = await settleAllAwaiting(held, workDir, claimed, state); // null when the job could not run
/// ```
pub(crate) async fn settle_all_awaiting(held: HeldClaims, work_dir: &Path, claimed: Vec<Claimed>, state: &'static str) -> Option<HeldClaims> {
    if claimed.is_empty() {
        return Some(held);
    }
    let directory: PathBuf = work_dir.to_path_buf();
    let settling: Vec<Claimed> = claimed;
    return match run_blocking_job_within_the_budget(move || {
        settle_all(directory.as_path(), &held, settling.as_slice(), state);
        return held;
    })
    .await
    {
        Ok(back) => Some(back),
        Err(reason) => {
            refused("the settling of the records this round ran", &reason);
            None
        }
    };
}

/// What:
///  The claims of `claimed` that this instance still holds,
/// before their work starts,
/// with the claim set handed back to its owner.
/// Why:
///   A claim whose file is gone under its generation was taken over,
/// so its work is another instance's now;
/// the look at the
///  directory is storage,
/// and the set is the owner's,
/// which the job takes and gives back.
/// A job that did not run answers nothing,
///  so no work starts on a claim the service could not confirm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [back, mine] = await stillHeldAwaiting(held, workDir, claimed) ?? [null, []]
/// ```
pub(crate) async fn still_held_awaiting(held: HeldClaims, work_dir: &Path, claimed: Vec<Claimed>) -> Option<(HeldClaims, Vec<Claimed>)> {
    if claimed.is_empty() {
        return Some((held, Vec::new()));
    }
    let directory: PathBuf = work_dir.to_path_buf();
    return match run_blocking_job_within_the_budget(move || {
        let mine: Vec<Claimed> = still_held(&held, directory.as_path(), claimed);
        return (held, mine);
    })
    .await
    {
        Ok(answered) => Some(answered),
        Err(reason) => {
            refused("the look that finds a claim taken over", &reason);
            None
        }
    };
}

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
pub(crate) async fn clear_push_failures_before_awaiting(git_dir: &Path, branch_ref: &str, since: std::time::SystemTime) {
    let directory: PathBuf = git_dir.to_path_buf();
    let branch: String = String::from(branch_ref);
    match run_blocking_job_within_the_budget(move || {
        clear_push_failures_before(directory.as_path(), branch.as_str(), since);
    })
    .await
    {
        Ok(()) => {}
        Err(reason) => {
            refused("the removal of this branch's earlier push records", &reason);
        }
    }
}

/// What:
///  Leave the person a record that the push of `commit` did not land,
/// with what became of it.
/// Why:
///   The record is a file written into the repository's directory,
/// which can wait on storage like any other write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushFailureAwaiting(gitDir, branch, commit, outcome, said)
/// ```
pub(crate) async fn record_push_failure_awaiting(git_dir: &Path, branch_ref: &str, commit: &str, outcome: PushOutcome, said: &str) -> Result<(), String> {
    let directory: PathBuf = git_dir.to_path_buf();
    let branch: String = String::from(branch_ref);
    let oid: String = String::from(commit);
    let words: String = String::from(said);
    return match run_blocking_job_within_the_budget(move || {
        return record_push_failure(directory.as_path(), branch.as_str(), oid.as_str(), outcome, words.as_str());
    })
    .await
    {
        Ok(answer) => answer,
        // A record that was not written leaves the item to its lease, and the next round tells the person.
        Err(reason) => Err(format!("{reason}")),
    };
}

/// What:
///  Leave the person a record that a push was skipped,
/// with the words the push gave.
/// Why:
///   The record is a file,
/// as for a failed push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushSkipAwaiting(gitDir, branch, commit, said)
/// ```
pub(crate) async fn record_push_skip_awaiting(git_dir: &Path, branch_ref: &str, commit: &str, said: &str) -> Result<(), String> {
    let directory: PathBuf = git_dir.to_path_buf();
    let branch: String = String::from(branch_ref);
    let oid: String = String::from(commit);
    let words: String = String::from(said);
    return match run_blocking_job_within_the_budget(move || {
        return record_push_skip(directory.as_path(), branch.as_str(), oid.as_str(), words.as_str());
    })
    .await
    {
        Ok(answer) => answer,
        Err(reason) => Err(format!("{reason}")),
    };
}

/// What:
///  How long until the first claim still held in `work_dir` runs out its lease,
/// or nothing when none is held.
/// Why:
///   The answer is a listing of the claim names,
/// and it is what wakes the service once another instance's lease is over,
/// so a failure to
///  read it is said rather than read as "nothing is held".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await nextClaimExpiryAwaiting(workDir, now, lease)
/// ```
pub(crate) async fn next_claim_expiry_awaiting(work_dir: &Path, now: &ClaimTime, lease: Duration) -> Option<Duration> {
    let directory: PathBuf = work_dir.to_path_buf();
    let reading: ClaimTime = now.clone();
    return match run_blocking_job_within_the_budget(move || {
        return next_claim_expiry(directory.as_path(), &reading, lease);
    })
    .await
    {
        Ok(wait) => wait,
        Err(reason) => {
            refused("the look that finds when another instance's claim runs out", &reason);
            None
        }
    };
}

/// What:
///  Take back the claims of records whose holder stopped renewing them,
/// and leave a notice for the person about each record given up on.
/// Why:
///   It lists the claim names and the states and moves records,
/// which is storage;
/// a lookup of the person's notice is written from
///  inside the same step (`SkipNotice`),
/// so the words of a place that could not be reached are said where they are learned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await requeueExpiredAwaiting(workDir, gitDir, now, lease)
/// ```
pub(crate) async fn requeue_expired_awaiting(work_dir: &Path, git_dir: &Path, now: &ClaimTime, lease: Duration) {
    let directory: PathBuf = work_dir.to_path_buf();
    let repository: PathBuf = git_dir.to_path_buf();
    let reading: ClaimTime = now.clone();
    // A record given up on is a notice about this repository, which the job says with its own path so nothing is lent to it.
    match run_blocking_job_within_the_budget(move || {
        let _recovered: Recovered = requeue_expired_reporting_telling(directory.as_path(), &reading, lease, MAX_TRIES, &SkipNotice(repository.as_path()));
    })
    .await
    {
        Ok(()) => {}
        Err(reason) => {
            refused("the taking back of claims whose holder stopped renewing them", &reason);
        }
    }
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
pub(crate) async fn instance_of_awaiting(git_dir: &Path) -> Result<String, crate::service_instance::GitDirUnreachable> {
    let directory: PathBuf = git_dir.to_path_buf();
    return match run_blocking_job_within_the_budget(move || {
        return crate::service_instance::try_instance_of(directory.as_path());
    })
    .await
    {
        Ok(answer) => answer,
        Err(reason) => {
            refused("the look for this directory's instance record", &reason);
            Err(crate::service_instance::GitDirUnreachable { path: git_dir.to_path_buf(), reason: format!("{reason}") })
        }
    };
}
