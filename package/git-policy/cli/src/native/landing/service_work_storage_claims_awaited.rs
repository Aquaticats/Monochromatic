//! What:
//!  The steps of one drain that touch the queue's own records:
//! claiming a directory's pending records,
//! deciding which of them are acceptable,
//! settling the ones that are done or refused,
//! confirming the claims this instance still holds,
//! finding when another instance's claim runs out,
//! taking back the claims whose holder stopped renewing them,
//! and reading the instance record of the directory to drain.
//! Why:
//!  The held claims have one owner at a time (`service_claim_renewal.rs`,
//! `HeldClaims`),
//! so a step that reads or renames a claim takes the whole set and hands it back with its answer,
//! and nothing locks it;
//! each step is run as a job of the pool by `service_work_storage_step_awaited.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claimed = await claimAllAwaiting(workDir, kinds); const [ok, refused] = await acceptableAwaiting(gitDir, claimed);
//! ```

use crate::service_claim_renewal::HeldClaims;
use crate::service_work_handlers_replanning_and_acceptance::{
    is_acceptable,
    settle_all,
    still_held,
};
use crate::service_work_storage_step_awaited::{
    a_storage_step_awaited_no_longer_than,
    refused,
    the_claim_steps_ceiling,
};
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
use crate::work_item::{
    Claimed,
    WorkKind,
    claim_all_of,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// What:
///  Claim every pending record of `work_dir` of the kinds `kinds` names.
/// Why:
///   The listing and the renames are storage,
/// and the answer is the drain's first step of a round.
/// The kinds are the drain's own:
/// the round that holds the service claims the pushes alone,
/// and the slow lane claims the rest
///  (`landing/service_work_slow_lane.rs`).
/// An error is the reason the directory could not be claimed,
///  which ends the round where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimed = await claimAllAwaiting(workDir, kinds)
/// ```
pub(crate) async fn claim_all_awaiting(
    work_dir: &Path,
    kinds: &'static [WorkKind],
) -> Result<Vec<Claimed>, String> {
    return claim_all_within_awaiting(the_claim_steps_ceiling(), work_dir, kinds).await;
}

/// `claim_all_awaiting` with the ceiling its place and its job are given,
/// so a control can reach the bound this step keeps in a moment (`service_round_tests.rs`).
pub(crate) async fn claim_all_within_awaiting(
    ceiling: Duration,
    work_dir: &Path,
    kinds: &'static [WorkKind],
) -> Result<Vec<Claimed>, String> {
    let directory: PathBuf = work_dir.to_path_buf();
    return match a_storage_step_awaited_no_longer_than(
        ceiling,
        "the look for this directory's pending records",
        "none was claimed",
        move || {
            return claim_all_of(directory.as_path(), kinds);
        },
    )
    .await
    {
        Ok(claimed) => claimed,
        Err(said) => Err(said),
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
pub(crate) async fn acceptable_awaiting(
    git_dir: &Path,
    claimed: Vec<Claimed>,
) -> (Vec<Claimed>, Vec<Claimed>) {
    let directory: PathBuf = git_dir.to_path_buf();
    return match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the look that decides which records are acceptable",
        "no record is decided either way",
        move || {
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
        },
    )
    .await
    {
        Ok((acceptable, refused_here)) => (acceptable, refused_here),
        // Nothing is decided on a job that did not run, so no record is failed for a reason the service did not read.
        Err(said) => {
            refused(said.as_str());
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
pub(crate) async fn settle_all_awaiting(
    held: HeldClaims,
    work_dir: &Path,
    claimed: Vec<Claimed>,
    state: &'static str,
) -> Option<HeldClaims> {
    if claimed.is_empty() {
        return Some(held);
    }
    let directory: PathBuf = work_dir.to_path_buf();
    let settling: Vec<Claimed> = claimed;
    return match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the settling of the records this round ran",
        "no record is settled",
        move || {
            settle_all(directory.as_path(), &held, settling.as_slice(), state);
            return held;
        },
    )
    .await
    {
        Ok(back) => Some(back),
        Err(said) => {
            refused(said.as_str());
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
pub(crate) async fn still_held_awaiting(
    held: HeldClaims,
    work_dir: &Path,
    claimed: Vec<Claimed>,
) -> Option<(HeldClaims, Vec<Claimed>)> {
    if claimed.is_empty() {
        return Some((held, Vec::new()));
    }
    let directory: PathBuf = work_dir.to_path_buf();
    return match a_storage_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the look that finds a claim taken over",
        "no claim is checked",
        move || {
            let mine: Vec<Claimed> = still_held(&held, directory.as_path(), claimed);
            return (held, mine);
        },
    )
    .await
    {
        Ok(answered) => Some(answered),
        Err(said) => {
            refused(said.as_str());
            None
        }
    };
}
