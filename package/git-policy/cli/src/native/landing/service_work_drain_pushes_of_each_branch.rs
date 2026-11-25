//! What:
//!  The push step of one round of the drain:
//!  the claimed pushes grouped by repository and branch,
//!  the branches run beside each other in waves,
//!  each branch settled as its own answer comes,
//! and the one line a push writes.
//! Why:
//!  A silent server holds its sibling for the push deadline or the connected push's quiet window,
//! so a finished push is settled and its notice written before its siblings answer.
//! A branch another instance took over is left to it,
//! and the same branch's records are one group,
//! so one branch is never pushed twice at once.
//! The step sits in a file of its own
//!  because the round that calls it is a file's worth of steps beside it
//! (`service_work_drain_pending_claims.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pushed = await pushTheBranchesAwaiting(held, workDir, gitDir, handlers, pushes, host, replanning) // HeldClaims | null
//! ```

use crate::journald_native::Priority;
use crate::parallel_budget::HostResources;
use crate::service_claim_renewal::HeldClaims;
use crate::service_timing::log_entry;
use crate::service_work_handlers_replanning_and_acceptance::{Replanning, Verdict, WorkHandlers};
use crate::service_work_items_side_by_side::{pushes_side_by_side, waves_of};
use crate::service_work_settling_a_verdict::settle_after_the_push;
use crate::service_work_storage_awaited::still_held_awaiting;
use crate::side_by_side_on_the_current_task::{AnAnswer, AnswersAsTheyCome};
use crate::work_item::{Claimed, WorkItem};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Instant, SystemTime};

/// The key two pushes may stand in each other's way over:
/// the repository and the branch the group's newest record names.
/// A group is never empty,
/// so a group with no newest record is a state this module cannot make.
fn branch_of(members: &[Claimed]) -> (PathBuf, String) {
    return match members.last() {
        Some(newest) => (newest.item.repository.clone(), newest.item.branch_ref.clone()),
        None => (PathBuf::new(), String::new()),
    };
}

/// What:
///  Group the claimed pushes of one round by repository and branch,
///  check that every group is still this round's,
///  run the branches beside each other in waves,
/// and settle each branch as its own answer comes.
/// Why:
///   A silent server holds its sibling for the push deadline or the connected push's quiet window,
/// so a finished push is settled and written before its siblings answer.
/// `None` means the set went with a job the system refused a thread for,
/// which ends the round rather than the set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pushTheBranchesAwaiting(held, workDir, gitDir, handlers, pushes, host, replanning): Promise<HeldClaims | null>
/// ```
pub(super) async fn push_the_branches_awaiting<H: WorkHandlers + Sync + ?Sized>(
    mut held: HeldClaims,
    work_dir: &Path,
    git_dir: &Path,
    handlers: &H,
    pushes: Vec<Claimed>,
    host: &HostResources,
    replanning: &mut Replanning,
) -> Option<HeldClaims> {
    let mut groups: Vec<((PathBuf, String), Vec<Claimed>)> = Vec::new();
    for item in pushes {
        let key = (item.item.repository.clone(), item.item.branch_ref.clone());
        let mut joined_at: Option<usize> = None;
        for (at, existing) in groups.iter().enumerate() {
            if existing.0 == key {
                joined_at = Some(at);
                break;
            }
        }
        match joined_at {
            Some(at) => groups[at].1.push(item),
            None => groups.push((key, vec![item])),
        }
    }
    // The claims of every branch are checked before any push starts, so one listing of the claim directory serves
    //  them all, and a branch another instance took over is left to it.
    let mut branches: Vec<Vec<Claimed>> = Vec::new();
    for (_, claimed_members) in groups {
        let members: Vec<Claimed> = match still_held_awaiting(held, work_dir, claimed_members).await {
            Some((back, mine)) => {
                held = back;
                mine
            }
            None => return None,
        };
        if !members.is_empty() {
            branches.push(members);
        }
    }
    // The pushes of different branches run side by side: each names its own branch, its own objects are already in the
    //  repository's store, and only the branch it updates and the reference it tracks are its own. The same branch's
    //  records are one group, so one branch is never pushed twice at once.
    let branches_wide: usize = pushes_side_by_side(branches.len(), host);
    let branch_keys: Vec<(PathBuf, String)> = branches.iter().map(Vec::as_slice).map(branch_of).collect();
    for wave in waves_of(branch_keys.as_slice(), branches_wide) {
        let mut running: Vec<Pin<Box<dyn Future<Output = (Verdict, SystemTime, u128)> + Send + '_>>> = Vec::with_capacity(wave.len());
        for index in &wave {
            let mut items: Vec<WorkItem> = Vec::new();
            for member in &branches[*index] {
                items.push(member.item.clone());
            }
            running.push(Box::pin(async move {
                // The push carries what existed when it began; a notice written after this moment is about something newer.
                let pushing_since: SystemTime = SystemTime::now();
                let pushing: Instant = Instant::now();
                let verdict: Verdict = handlers.push_awaiting(items.as_slice()).await;
                return (verdict, pushing_since, pushing.elapsed().as_millis());
            }));
        }
        // Each branch is settled, and its notice written, as its own push ends:
        //  a silent server holds its sibling for the push deadline or the connected push's quiet window, and the
        //  refusal Git already gave must not wait for that.
        let mut answers: AnswersAsTheyCome<'_, (Verdict, SystemTime, u128)> = AnswersAsTheyCome::starting(running);
        while let Some((at, answer)) = answers.the_next_answer().await {
            let (verdict, pushing_since, push_ms): (Verdict, SystemTime, u128) = match answer {
                AnAnswer::Answered(answered) => answered,
                // A push whose handler stopped unexpectedly fails with what happened, and the branches beside it go on.
                AnAnswer::Stopped(said) => (Verdict::Failed(said), SystemTime::now(), 0),
            };
            let members: &Vec<Claimed> = &branches[wave[at]];
            if let Some(newest) = members.last() {
                // One line for every push the service ran, so a branch that was not backed up (or was) is found by its name.
                let (priority, outcome, said): (Priority, &str, &str) = match &verdict {
                    Verdict::Done | Verdict::Delivered { .. } => (Priority::Info, "done", ""),
                    Verdict::Failed(text) => (Priority::Warning, "failed", text.as_str()),
                    Verdict::Retry(text) => (Priority::Warning, "retrying", text.as_str()),
                    Verdict::Skipped(text) => (Priority::Info, "skipped", text.as_str()),
                };
                let fields: [(&str, String); 3] = [
                    ("CLI_GIT_BRANCH", newest.item.branch_ref.clone()),
                    ("CLI_GIT_OID", newest.item.oid.clone()),
                    ("CLI_GIT_OUTCOME", outcome.to_string()),
                ];
                log_entry(
                    priority,
                    format!(
                        "pushed branch={:?} oid={} items={} outcome={outcome} push_ms={push_ms}{}{said}",
                        newest.item.branch_ref,
                        newest.item.oid,
                        members.len(),
                        if said.is_empty() { "" } else { ": " }
                    )
                    .as_str(),
                    &fields,
                );
            }
            match settle_after_the_push(held, work_dir, git_dir, members, verdict, pushing_since, replanning).await {
                Some(back) => held = back,
                None => {
                    // The set went with a job the system refused a thread for, so the branches still pushing are
                    //  waited for and their records keep their lease.
                    answers.the_rest().await;
                    return None;
                }
            }
        }
    }
    return Some(held);
}
