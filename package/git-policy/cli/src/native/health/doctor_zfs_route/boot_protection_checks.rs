//! What:
//!  The checkout's own protection:
//!  whether the covered entry under the mount the checkout is served from is
//!  the unmounted,
//!  immutable,
//!  account-owned underlay entry the boot needs before it attaches anything.
//! Why:
//!  the owner's answer V7 asks `doctor` to print the boot setup,
//!  and the protection is the one part of it that stands under the checkout
//!  rather than in a unit file
//!  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
//!  V7;
//!  plan step 19,
//!  "What it builds";
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Checkout boot mount protection").
//!  The three answers come from the underlay entry's own verdict,
//!  because the order of its checks is what keeps each sentence true:
//!  an entry that is not a directory says nothing about a mount,
//!  and a filesystem that reports no attributes says nothing about the flag.
//!  This check names the field a finding is about and leaves the repair
//!  sentence to the verdict,
//!  so the boot and the report say one thing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkBootProtection(facts); // pass | finding('entry'|'mount'|'immutable'|'owner') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use crate::checkout_underlay_entry::underlay_entry_read::{
    CoveredEntryFacts,
    EntryReadRefusal,
};
use crate::checkout_underlay_entry::underlay_entry_verdict::{
    UnderlayVerdict,
    UnprotectedReason,
    verify,
};

/// The field a finding about the entry's own kind names.
const FIELD_ENTRY: &str = "entry";

/// The field a finding about a mount standing at the path names.
const FIELD_MOUNT: &str = "mount";

/// The field a finding about the immutable attribute names.
const FIELD_IMMUTABLE: &str = "immutable";

/// The field a finding about the entry's owner names.
const FIELD_OWNER: &str = "owner";

/// What:
///  Everything the checkout's boot protection is judged by:
///  one read of the entry at the checkout path,
///  or the reason no read could be made,
///  and the account the route requires the entry to belong to.
/// Why:
///  the read is what the boot itself compares before it attaches anything,
///  so the report answers from the same value rather than from a second
///  reading of the path that could disagree with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = { entry: readOf('/home/user/Monochromatic'), wantedOwner: 1000 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootProtectionFacts {
    /// The kernel's answers about the covered entry,
    /// or why the read made none.
    pub entry: Result<CoveredEntryFacts, EntryReadRefusal>,
    /// The account the route requires the covered entry to be owned by.
    pub wanted_owner: u32,
}

/// What:
///  What the checkout's boot protection answers.
/// Why:
///  an entry nobody proved is unknown rather than a pass or a failure,
///  because a filesystem without the attribute proves neither,
///  and a person reading a pass on an unprotected checkout would attach a
///  clone onto a mount another process can rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkBootProtection(facts); // pass | finding('entry'|'mount'|'immutable'|'owner') | unknown
/// ```
pub fn check_boot_protection(facts: &BootProtectionFacts) -> CheckOutcome {
    let read: &CoveredEntryFacts = match &facts.entry {
        Ok(read) => read,
        Err(refusal) => {
            return unknown(format!(
                "the checkout's covered entry was not read, so whether its underlay is protected is unproved: {}",
                refusal.message()
            ));
        }
    };
    let verdict: UnderlayVerdict = verify(read, facts.wanted_owner);
    return match &verdict {
        UnderlayVerdict::Protected => pass(verdict.message(read)),
        UnderlayVerdict::Unprotected(reason) => finding(field_of(reason), verdict.message(read)),
        UnderlayVerdict::Unproven(_) => unknown(verdict.message(read)),
    };
}

/// What:
///  The field one unprotected answer is about.
/// Why:
///  a finding names the fact a person repairs,
///  and the verdict's own reason says why and how,
///  so the two together send a person to one call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const field = fieldOf(reason); // 'immutable'
/// ```
fn field_of(reason: &UnprotectedReason) -> String {
    let field: &'static str = match reason {
        UnprotectedReason::NotADirectory => FIELD_ENTRY,
        UnprotectedReason::MountedOver => FIELD_MOUNT,
        UnprotectedReason::ImmutableClear => FIELD_IMMUTABLE,
        UnprotectedReason::Owner {
            ..
        } => FIELD_OWNER,
    };
    return String::from(field);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "boot_protection_checks_tests.rs"]
mod tests;
