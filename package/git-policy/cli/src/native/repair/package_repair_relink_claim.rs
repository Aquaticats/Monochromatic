//! What:
//!  The claim this command holds on the relinking of one new worktree,
//!  so that only one of the two who can relink it runs the tools.
//! Why:
//!   Two parties can relink one worktree:
//!  the add that wrote the record and runs the tools itself when no service read its wake,
//!  and the commit service, which claims `pending` records on every wake and on its own look every
//!  `LOOK_AGAIN_AFTER`.
//!  Both used to run the two installers at once over one `node_modules`
//!  (the review of 2026-11-13,
//!  finding 1: two runs of one worktree 1.5 s apart on this branch,
//!  and another way in on `main` when a service starts during the add's own run).
//!  The queue's own claim is the one step only one of them can win: a rename out of `pending`.
//!  So whoever is about to run the tools takes the record first,
//!  and the other, finding nothing to take, does not start.
//!   The claim is kept fresh while the tools run,
//!  by the same renewal the service uses,
//!  because a claim's lease is what recovers it after a kill
//!  and one tool may run longer than the lease.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = HeldRelinkClaim.take(workDir, id); // undefined when the service has the record
//! const running = claim.keptFresh(); await relinkHere(); await running.giveBack();
//! ```

use crate::claim_time::ClaimTime;
use crate::service_claim_renewal::{HeldClaims, Renewer, held_claims, renewal_interval};
use crate::service_claim_renewal_running::ClaimRenewal;
use crate::service_work::CLAIM_LEASE;
use crate::work_item::{Claimed, DONE, WorkItem, WorkKind, claim_of, find_claim, settle_named};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// What:
///  One relink record this command claimed,
///  and the record itself,
///  so the work it names is this command's alone.
/// Why:
///   The claim is the queue's own,
///  taken by the same rename the service takes,
///  so no service can claim the record while this value lives,
///  and a record the service already claimed is no claim to take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldRelinkClaim = { workDir: string; id: string; claimed: Claimed };
/// ```
#[derive(Debug)]
pub struct HeldRelinkClaim {
    /// The work directory the record was read from.
    work_dir: PathBuf,
    /// The record's id, which names it in every state directory but the claim.
    id: String,
    /// The claim as it was taken.
    claimed: Claimed,
}

impl HeldRelinkClaim {
    /// What:
    ///  Take the claim of the record `id` of one worktree's relink,
    ///  or answer nothing when another party holds it.
    /// Why:
    ///   The rename out of `pending` is what one consumer of the directory wins,
    ///  so answering nothing is how this command learns that the relinking is the service's work now
    ///  and not to start a second run of the tools.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const claim = (workDir, id) => { const claimed = claimOf(workDir, id, RELINK); return claimed && { workDir, id, claimed }; };
    /// ```
    pub fn take(work_dir: &Path, id: &str) -> Option<HeldRelinkClaim> {
        return match claim_of(work_dir, id, WorkKind::Relink, &ClaimTime::read()) {
            Ok(Some(claimed)) => Some(HeldRelinkClaim { work_dir: work_dir.to_path_buf(), id: String::from(id), claimed }),
            // A claim that could not be taken and a record another party holds are the same answer to this command: it does
            //  not run the tools. A record left in `pending` is still a record a service will claim.
            Ok(None) => None,
            Err(_unclaimed) => None,
        };
    }

    /// The record the claim holds.
    pub fn item(&self) -> &WorkItem {
        return &self.claimed.item;
    }

    /// What:
    ///  Keep this claim fresh while the work it names runs.
    /// Why:
    ///   A claim's lease is what recovers the record after a holder is killed,
    ///  and one tool runs for as long as its own quiet window and ceiling allow,
    ///  which is longer than one lease:
    ///  a claim that expired mid-run would be requeued and claimed by the service,
    ///  which is the second run of the tools this claim exists to prevent.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const running = claim.keptFresh(); // a renewal every third of the lease, until given back
    /// ```
    pub fn kept_fresh(self) -> RelinkClaimRunning {
        let (claims, renewer): (HeldClaims, Renewer) = held_claims(self.work_dir.as_path());
        claims.hold(&self.claimed);
        let renewal: ClaimRenewal = ClaimRenewal::start(renewer, renewal_interval(CLAIM_LEASE));
        return RelinkClaimRunning { work_dir: self.work_dir, id: self.id, claims: Mutex::new(claims), renewal: Mutex::new(Some(renewal)) };
    }
}

/// What:
///  A relink's claim while the tools of that relink run,
///  and the renewal that keeps it from looking abandoned.
/// Why:
///   The renewal writes a file,
///  so it runs beside the tools and not on the task that waits for them;
///  the two pieces are kept in a `Mutex` so this value may be held across the tool waits,
///  which a task of the runtime is allowed to do only with values that may cross threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RelinkClaimRunning = { workDir: string; id: string; claims: HeldClaims; renewal?: ClaimRenewal };
/// ```
pub struct RelinkClaimRunning {
    /// The work directory the record is in.
    work_dir: PathBuf,
    /// The record's id.
    id: String,
    /// The claims the renewal keeps fresh,
    /// which must outlive it:
    /// a renewal with no held claim to be told of ends.
    claims: Mutex<HeldClaims>,
    /// The renewal itself,
    /// taken out when it is stopped,
    /// because stopping it consumes it.
    renewal: Mutex<Option<ClaimRenewal>>,
}

impl RelinkClaimRunning {
    /// What:
    ///  Settle the record as one this command finished,
    ///  and stop the renewal,
    ///  from a caller that is itself a task of the runtime.
    /// Why:
    ///   A record left claimed after its work is done would be requeued by its lease and run again,
    ///  and a claim whose renewal runs on would keep a file nothing reads.
    ///  The settle finds the claim's current name,
    ///  which the renewal has changed since the claim was taken.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await running.stopAwaiting(); running.claims.settle(workDir, id, DONE);
    /// ```
    pub async fn give_back(self) {
        let renewal: Option<ClaimRenewal> = match self.renewal.lock() {
            Ok(mut held) => held.take(),
            Err(_poisoned) => None,
        };
        // A lock a panic left poisoned holds no renewal this value owns any more, and the claim then expires by its lease.
        if let Some(running) = renewal {
            running.stop_awaiting().await;
        }
        self.settle();
    }

    /// `give_back` for a caller outside the runtime,
    /// which enters it once to wait for the renewal's task.
    pub fn give_back_on_the_calling_thread(self) {
        let renewal: Option<ClaimRenewal> = match self.renewal.lock() {
            Ok(mut held) => held.take(),
            Err(_poisoned) => None,
        };
        if let Some(running) = renewal {
            running.stop_on_the_calling_thread();
        }
        self.settle();
    }

    /// Move the record out of `claimed` now that its work is done,
    /// saying so where it could not be moved:
    /// a record that stays claimed is requeued by its lease and run again, which is the work repeated and never lost.
    fn settle(self) {
        let moved: Result<(), String> = match self.claims.lock() {
            Ok(claims) => claims.settle(self.work_dir.as_path(), self.id.as_str(), DONE),
            Err(_poisoned) => settle_by_id(self.work_dir.as_path(), self.id.as_str()),
        };
        if let Err(reason) = moved {
            crate::diagnostic_log::debug("packageRepairRelinkClaim", format!("the claim of {} stays: {reason}", self.id).as_str());
        }
    }
}

/// What:
///  Move the claimed record `id` to `done` without the held claims,
///  for the case their lock cannot be taken.
/// Why:
///   The claim is this command's by the rename that took it,
///  so its current name is found by its id,
///  and the settle is the queue's own step and not a second mechanism.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settleById = (workDir, id) => { const path = findClaim(workDir, id); const name = path && basename(path); return name ? settleNamed(workDir, name, DONE) : `not claimed`; };
/// ```
fn settle_by_id(work_dir: &Path, id: &str) -> Result<(), String> {
    let Some(path) = find_claim(work_dir, id) else { return Err(format!("{id} is not claimed")) };
    let name: String = path.file_name().map_or_else(String::new, |held| { return held.to_string_lossy().into_owned(); });
    return settle_named(work_dir, name.as_str(), DONE);
}

/// Controls of the relink claim stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_claim_tests.rs"]
mod tests;
