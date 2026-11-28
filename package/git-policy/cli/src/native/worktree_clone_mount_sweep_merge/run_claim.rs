//! What:
//!  The route-wide run claim of one event-driven pass:
//!  the repository's numbered claim under `run.<generation>`,
//!  taken without waiting,
//! and given back when the pass ends.
//! Why:
//!  One event-driven pass runs at a time,
//!  and a contender marks a request instead of running,
//!  because two triggers about one clone need one pass that reads the clone
//!  after both writes
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The principle" and "What a taker does";
//!  veto entry V4).
//!  The claim is the repository's own lock-free claim
//!  (`lock/single_owner_claim.rs`,
//!  `take_claim`):
//!  the kernel lets exactly one process create the next generation,
//!  the owner is named by process birth rather than by a reused process id,
//!  a loser answers at once with the owner's identity,
//!  and a killed pass leaves nothing to clean by hand.
//!  Its names are `run.<digits>` in one folder beside the mark and the budget,
//!  which the claim's own name filter requires and no subfolder is needed for
//!  (V6's first option).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = takeTheRunClaim(folder); // { held } | { heldByAnother } | { unreadable }
//! ```

use crate::single_owner_claim::{
    BusyOwner,
    ClaimOutcome,
    OwnerClaim,
    take_claim,
};
use crate::worktree_clone_mount_sweep_merge::sweeps_folder::{
    RUN_CLAIM_PREFIX,
    SweepsFolder,
};
use std::ffi::OsStr;

/// What:
///  What one attempt to take the run claim answered.
/// Why:
///   A held claim runs the pass,
///  a claim another process holds makes this trigger mark and try once more,
///  and an unreadable device lets the pass run without the claim at all,
///  which are the three arms the taker's steps 2 and 8 need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimAnswer = { kind: 'held'; claim } | { kind: 'held-by-another'; owner } | { kind: 'unreadable'; reason };
/// ```
#[derive(Debug)]
pub enum ClaimAnswer {
    /// This process holds the run claim.
    Held(HeldRunClaim),
    /// A live pass holds the run claim.
    HeldByAnother(BusyOwner),
    /// The device could not be read,
    /// so no claim was taken and nothing was marked.
    Unreadable {
        /// What was wrong,
        /// with the record it named.
        reason: String,
    },
}

/// What:
///  Take the run claim without waiting.
/// Why:
///   See the file's own description:
///  the folder is made and proved first,
///  and every failure is the unreadable arm rather than a refusal,
///  because the device is cost and the pass runs anyway (V7's first option).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = takeTheRunClaim(folder);
/// ```
pub fn take_the_run_claim(folder: &SweepsFolder) -> ClaimAnswer {
    if let Err(reason) = folder.prepare() {
        return ClaimAnswer::Unreadable {
            reason,
        };
    }
    let taken: Result<ClaimOutcome, String> =
        take_claim(folder.directory(), OsStr::new(RUN_CLAIM_PREFIX)).map_err(|error| {
            return format!(
                "the run claim in {} could not be read or written ({error})",
                folder.directory().display()
            );
        });
    return match taken {
        Ok(ClaimOutcome::Held(claim)) => ClaimAnswer::Held(HeldRunClaim {
            claim,
        }),
        Ok(ClaimOutcome::Busy(_owner)) => ClaimAnswer::Unreadable {
            reason: String::from("planted"),
        },
        Err(reason) => ClaimAnswer::Unreadable {
            reason,
        },
    };
}

/// What:
///  One event-driven pass's hold on the route-wide run claim.
/// Why:
///   The claim lives exactly as long as this value,
///  so a pass cannot act after giving its claim up by accident,
///  and the release is reported rather than logged,
///  because a record this device could not write is a line its answer carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HeldRunClaim { release(): void }
/// ```
#[derive(Debug)]
pub struct HeldRunClaim {
    /// The shared claim's own hold,
    /// which names this process and publishes the next generation.
    claim: OwnerClaim,
}

impl HeldRunClaim {
    /// What:
    ///  The claim file this hold created,
    ///  which a held line names when someone else's pass holds the claim.
    /// Why:
    ///   A person reading a held line needs the file the other pass holds,
    ///  and the claim names its own path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const path = claim.path();
    /// ```
    pub fn path(&self) -> &std::path::Path {
        return self.claim.path();
    }

    /// Give the claim up now rather than when the value is dropped.
    ///
    /// # Errors
    /// The claim file could not be rewritten as released,
    /// in the words the pass's answer carries.
    pub fn release(mut self) -> Result<(), String> {
        return self.claim.release().map_err(|error| {
            return format!(
                "the run claim {} could not be released ({error})",
                self.claim.path().display()
            );
        });
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "run_claim_tests.rs"]
mod tests;
