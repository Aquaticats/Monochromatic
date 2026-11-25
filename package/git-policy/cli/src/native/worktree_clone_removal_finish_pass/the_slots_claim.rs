//! What:
//!  The per-slot claim the finish pass holds while it settles one plan's tail,
//!  and the answer it gives when a live owner holds the slot already.
//! Why:
//!  The pass settles a registration,
//!  removes a binding and removes a plan,
//!  which is the list's steps 11 to 14,
//!  and the list holds the claim from its first step to its last
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals";
//!  `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass that settles a registration").
//!  The claim is the slot's own generation file,
//!  keyed by the last path component of the plan's `targetHex`,
//!  and never by the dataset name in `cloneDataset`.
//!  A live owner means a removal is finishing this slot,
//!  so the pass touches nothing and waits for that command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = takeTheSlotsClaim(commonDir, slot); // held | busy | refused
//! ```

use crate::single_owner_claim::{BusyOwner, ClaimOutcome, OwnerClaim, take_claim};
use crate::worktree_clone_binding_record::registration_home::{claim_prefix, home_folder};
use std::path::Path;

/// What:
///  How one claim attempt ended for the pass.
/// Why:
///  A live owner is a normal answer and not a failure,
///  and a claim that could not be written proves nothing about the slot,
///  so the two travel apart from the held claim the pass releases last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheClaim = { kind: 'held'; claim } | { kind: 'busy'; owner } | { kind: 'refused'; said };
/// ```
pub enum TheClaim {
    /// This process holds the slot and may finish that plan's tail.
    Held(OwnerClaim),
    /// A live process holds the slot,
    /// so its own command finishes the removal.
    Busy(BusyOwner),
    /// The claim could not be written or read.
    Refused {
        /// What the claim call answered.
        said: String,
    },
}

/// What:
///  The claim for one slot,
///  under the name its generation files carry.
/// Why:
///  The slot is the key every reader of this family knows first,
///  and the plan names it as the last path component of its `targetHex` path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = takeTheSlotsClaim(commonDir, slot);
/// ```
pub fn take_the_slots_claim(common_dir: &Path, slot: &str) -> TheClaim {
    let folder = home_folder(common_dir);
    return match take_claim(folder.as_path(), claim_prefix(slot).as_os_str()) {
        Ok(ClaimOutcome::Held(claim)) => TheClaim::Held(claim),
        Ok(ClaimOutcome::Busy(owner)) => TheClaim::Busy(owner),
        Err(refusal) => TheClaim::Refused { said: refusal.to_string() },
    };
}

/// What:
///  The claim given back,
///  which is the list's step 14.
/// Why:
///  The released claim is the newest generation and is never removed,
///  so the next claimant takes the next one,
///  and a release that failed is said once rather than left to a drop that says nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// giveTheClaimBack(claim);
/// ```
pub fn give_the_claim_back(claim: &mut OwnerClaim) -> Result<(), String> {
    return claim.release().map_err(|refusal| return refusal.to_string());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_slots_claim_tests.rs"]
mod tests;
