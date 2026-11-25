//! What:
//!  The per-slot claim a whole removal holds:
//!  taken before its first read,
//!  held through every destroy and every record it writes,
//!  and given back after its last step.
//! Why:
//!  Two removals of one slot,
//!  and an add of it,
//!  write the same binding record,
//!  the same plan folder and the same registration,
//!  so one of them must own the slot for as long as it works.
//!  A removal that took no claim was serialized by nothing
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals",
//!  steps 1 and 14;
//!  `SPEC.md`,
//!  "The per-slot claim").
//!  The claim is the add's own (`lock/single_owner_claim.rs`),
//!  keyed by the same `<slot>.claim.<generation>` name in the same folder,
//!  so a removal and an add of one slot meet each other rather than interleave.
//!  It is taken at the order's step 1,
//!  which is why this file is the completion's first call and not the entry's:
//!  the entry is one part of the order and the claim has to outlast every part.
//!  A held slot is refused rather than queued,
//!  and a removal that is killed before its last step leaves a claim whose owner no
//!  longer runs,
//!  which the next command takes over under the claim's own rule,
//!  so nothing is left to clean up by hand.
//!  A refusal or an error before the last step gives the claim back as the error path's
//!  own drop,
//!  best effort:
//!  a rewrite that fails there leaves the claim naming this process,
//!  and the claim's liveness rule frees the slot once the process ends,
//!  when the next command takes the next generation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = takeTheRemovalsClaim(commonDir, slot);
//! // ... the whole removal ...
//! const end = releaseAtTheLastStep(claim);
//! ```

use crate::single_owner_claim::{ClaimOutcome, OwnerClaim, take_claim};
use crate::worktree_clone_binding_record::registration_home::{claim_prefix, home_folder, open_or_create_home};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::{Path, PathBuf};

/// What:
///  The claim one removal holds on one slot for as long as its order runs.
/// Why:
///  Dropping it gives the slot back,
///  so a refusal or an error in the middle of the order releases it without the
///  order having to name every way it can end.
///  The claim files stay where the add's own claim leaves them:
///  the newest generation is never removed,
///  and a released one is rewritten in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalClaim = { claim: OwnerClaim; slot: string };
/// ```
#[derive(Debug)]
pub struct RemovalClaim {
    /// The claim this process took,
    /// which its drop releases when the order ends before its last step.
    claim: OwnerClaim,
    /// The slot the claim is about,
    /// which the release's sentence names.
    slot: String,
}

/// What:
///  Whether the removal gave the slot back at the order's last step.
/// Why:
///  The report says what a person still has to do,
///  and a claim that could not be rewritten is one of those things:
///  it names this process until the process ends,
///  and then stops counting as alive without any hand cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimEnd = { kind: 'held' } | { kind: 'released' } | { kind: 'refused'; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimEnd {
    /// The order still runs and has not reached its last step.
    /// A report a person reads after a removal returns never carries this.
    Held,
    /// The claim's file says `released`,
    /// so the next command on this slot starts at once.
    Released,
    /// The claim's file could not be rewritten,
    /// and it stops counting as alive when this process ends.
    RefusedToRelease {
        /// What the rewrite answered,
        /// in the claim module's own words.
        said: String,
    },
}

/// What:
///  Take the slot's claim for a whole removal,
///  or refuse with the live owner's process and claim file.
/// Why:
///  This is the order's step 1:
///  nothing of the removal reads a record or writes one before the claim stands,
///  and the add takes the same claim,
///  so an add and a removal of one slot cannot both work.
///
/// # Throws
/// A claim a live command holds,
/// one that cannot be taken,
/// or the record folder that cannot be opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = takeTheRemovalsClaim(commonDir, slot);
/// ```
pub fn take_the_removals_claim(common_dir: &Path, slot: &str) -> Result<RemovalClaim, RemovalStop> {
    let home: PathBuf = home_folder(common_dir);
    open_or_create_home(common_dir).map_err(|error| {
        return RemovalStop::TheClaimCannotBeTaken { slot: String::from(slot), said: format!("the wrapper's record folder {} cannot be opened ({error})", home.display()) };
    })?;
    return match take_claim(home.as_path(), claim_prefix(slot).as_os_str()) {
        Ok(ClaimOutcome::Held(claim)) => Ok(RemovalClaim { claim, slot: String::from(slot) }),
        Ok(ClaimOutcome::Busy(owner)) => Err(RemovalStop::TheSlotIsClaimed { slot: String::from(slot), pid: owner.pid, path: owner.path }),
        Err(error) => Err(RemovalStop::TheClaimCannotBeTaken { slot: String::from(slot), said: format!("{error}") }),
    };
}

/// What:
///  Give the slot back as the order's last step,
///  and say what the release did.
/// Why:
///  The release comes after the plan is proved gone,
///  so no command sees a free slot whose removal's last witness still stands,
///  and a release that failed is reported rather than swallowed:
///  the claim then names this process,
///  and the claim's own rule stops counting it as alive when the process ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const end = releaseAtTheLastStep(claim);
/// ```
pub fn release_at_the_last_step(claim: &mut RemovalClaim) -> ClaimEnd {
    return match claim.claim.release() {
        Ok(()) => ClaimEnd::Released,
        Err(error) => ClaimEnd::RefusedToRelease { said: format!("the claim on the slot {} could not be released ({error})", claim.slot) },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_slot_claim_tests.rs"]
mod tests;
