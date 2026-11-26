//! What:
//!  The slot's claim as the cancel command sees it:
//!  the whole removal's own claim,
//!  taken through the merged `take_the_removals_claim`,
//!  which is the add's own claim in the add's folder,
//!  and given back when the claim is dropped.
//! Why:
//!  The project's merged removal path takes one claim per slot before its first read,
//!  in the add's folder and under the add's own name,
//!  so a removal and an add of one slot meet each other rather than interleave
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals",
//!  steps 1 and 14).
//!  The command is a removal-side operation,
//!  so it takes the same claim and is serialized against both.
//!  A live owner refuses the command with its process and claim file,
//!  and the claim's own rule replaces a dead owner's claim with the next generation,
//!  so nothing is left to clean up by hand.
//!  The design's separate `<guid>.removal` folder is not what merged:
//!  the replayed claim is the slot-keyed one,
//!  and this module takes that and writes no second claim of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = claim.take('one'); // { claim } or a refusal naming the live owner
//! // dropping it gives the slot back
//! ```

use crate::worktree_clone_removal_proof_and_plan::removal_slot_claim::{RemovalClaim, take_the_removals_claim};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use std::path::Path;

/// What:
///  A slot claim this run holds.
/// Why:
///  The live arm is the removal order's own claim,
///  whose drop gives the slot back on every path out;
///  the control arm creates nothing,
///  so the command's decisions are provable without a repository.
///  One enum keeps the live claim's only reader from being a field no code names,
///  which is its own drop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldClaim = { kind: 'live'; claim: RemovalClaim } | { kind: 'a-control' };
/// ```
#[derive(Debug)]
pub enum HeldClaim {
    /// The claim this process took,
    /// released when this value goes.
    Live(RemovalClaim),
    /// A control's answer,
    /// which holds nothing.
    AControl,
}

/// What:
///  The slot's claim,
///  as the command takes it.
/// Why:
///  A control must be able to refuse with a live owner and to hold a claim without a file,
///  so the take is one seam and the drop is the release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheRemovalClaim { take(slot: string): HeldClaim }
/// ```
pub trait TheRemovalClaim {
    /// Take this slot's removal claim.
    ///
    /// # Errors
    /// A live command holding the slot,
    /// or a claim whose folder or file could not be used.
    fn take(&self, slot: &str) -> Result<HeldClaim, CancelRefusal>;
}

/// What:
///  The release answer:
///  the removal order's own claim in the add's folder.
/// Why:
///  The mechanism and the names are the merged removal path's,
///  so this module adds no second claim and no second folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = new LiveTheRemovalClaim(commonDir);
/// ```
pub struct LiveTheRemovalClaim<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
}

impl TheRemovalClaim for LiveTheRemovalClaim<'_> {
    fn take(&self, slot: &str) -> Result<HeldClaim, CancelRefusal> {
        let claim: RemovalClaim = take_the_removals_claim(self.common_dir, slot).map_err(|stop| return of_a_claim_stop(&stop))?;
        return Ok(HeldClaim::Live(claim));
    }
}

/// One claim refusal as this command's,
/// naming the owner and the file a person has to look at.
fn of_a_claim_stop(stop: &RemovalStop) -> CancelRefusal {
    return match stop {
        RemovalStop::TheSlotIsClaimed { pid, path, .. } => CancelRefusal::ALiveRemoval { owner: format!("pid {pid} (its claim is {})", path.display()) },
        other => CancelRefusal::TheClaimCouldNotBeTaken { said: other.message() },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_removal_claim_tests.rs"]
mod tests;
