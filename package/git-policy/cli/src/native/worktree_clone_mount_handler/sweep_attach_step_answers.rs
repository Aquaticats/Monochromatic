//! What:
//!  What one attach step is asked,
//!  the answers it gives,
//!  and the step and errno each refusal is written as.
//! Why:
//!  A sweep reports each clone's own outcome,
//!  so the answer type and the words a refusal carries live beside each other
//!  and before the step that builds them,
//!  and a reader sees every answer a step gives without reading the walk
//!  that reaches one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type AttachRequest = { checkout: string; slot: string; clone: string; store: string };
//! type AttachStepOutcome = { kind: 'attached'; mount } | { kind: 'already-standing'; mount } | { kind: 'rolled-back'; said } | { kind: 'retained'; said } | { kind: 'refused'; step; errno; said };
//! ```

use crate::start_probe::probe_errno_names::ErrnoWord;
use crate::worktree_clone_mount::{
    CloneSlotRefusal,
    MountIdentity,
};
use std::path::Path;

/// Module tag for diagnostics.
pub(crate) const TAG: &str = "sweepAttachStep";

/// The step a comparison or an identity check is written as,
/// which the design's own step list names.
pub const IDENTITY_COMPARE_STEP: &str = "identity-compare";

/// The step the no-crossing walk is written as.
pub const WALK_STEP: &str = "walk";

/// The step the post-attach proofs are written as,
/// including the read-back of the mount the attach made.
pub const VERIFY_STEP: &str = "verify";

/// What:
///  What one clone's attach step is asked to do.
/// Why:
///  Every field is a name the step must not invent:
///  the checkout it walks,
///  the slot the clone's name gave,
///  the dataset it attaches and the store it compares,
///  so a caller cannot attach one clone at another clone's slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachRequest = { checkout: string; slot: string; clone: string; store: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AttachRequest<'a> {
    /// The checkout the walk starts at.
    pub checkout: &'a Path,
    /// The slot the clone's name gave.
    pub slot: &'a str,
    /// The clone's full dataset name.
    pub clone: &'a str,
    /// The store of recorded identities.
    pub store: &'a Path,
}

/// What:
///  What became of one attach step.
/// Why:
///  A sweep reports each clone's own outcome separately,
///  and a refusal that carries a mount the rollback could not take down must
///  not look like a refusal that left nothing behind.
///  A refusal also carries the step that failed and its errno,
///  because the receipt names the call rather than "the attach".
///  A clone already at its slot is its own answer:
///  no walk below the slot,
///  no mount this step made and no post-attach proof ran,
///  so it cannot read as an attach this step made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachStepOutcome = { kind: 'attached'; mount } | { kind: 'already-standing'; mount } | { kind: 'rolled-back'; said } | { kind: 'retained'; said } | { kind: 'refused'; step; errno; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachStepOutcome {
    /// The mount the attach made is the mount the canonical slot shows.
    Attached {
        /// The mount,
        /// as the kernel lists it.
        mount: MountIdentity,
    },
    /// The clone's own mount already stands at its slot and this step made none.
    AlreadyStanding {
        /// The mount the slot holds,
        /// proved to be this clone's own by the sweep's own standing answer.
        mount: MountIdentity,
    },
    /// The attach made a mount that landed elsewhere and the rollback took it
    /// down.
    RolledBack {
        /// The rollback's own report.
        said: String,
    },
    /// The attach made a mount that landed elsewhere and the rollback could not
    /// take it down.
    Retained {
        /// The rollback's own report,
        /// naming the mount and both paths.
        said: String,
    },
    /// Nothing was attached:
    /// the walk,
    /// an identity compare or a mount call refused.
    Refused {
        /// The handler's own step that refused,
        /// such as `walk`,
        /// `identity-compare` or `move-mount`.
        step: String,
        /// The errno's name,
        /// where the kernel answered one.
        errno: Option<String>,
        /// The refusal in words.
        said: String,
    },
}
/// What:
///  What one refused attach call answers a sweep.
/// Why:
///  A call after the move can have placed a mount that could not be taken down
///  again,
///  and that answer must read as a retained mount,
///  which files a failure receipt,
///  rather than as a refusal that left nothing behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = theOutcomeOfARefusedAttach(refusal); // refused | retained
/// ```
pub fn the_outcome_of_a_refused_attach(
    refusal: &crate::worktree_clone_mount::AttachRefusal
) -> AttachStepOutcome {
    if refusal.placement == crate::worktree_clone_mount::Placement::PlacedAndRetained {
        return AttachStepOutcome::Retained {
            said: refusal.message(),
        };
    }
    return AttachStepOutcome::Refused {
        step: String::from(refusal.call.word()),
        errno: the_errno_name(refusal.errno),
        said: refusal.message(),
    };
}

/// The step and errno one walk refusal is written as.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [step, errno] = stepOfWalkRefusal(refusal); // ['walk', undefined] or ['identity-compare', undefined]
/// ```
pub(crate) fn the_step_of_a_walk_refusal(
    refusal: &CloneSlotRefusal
) -> (&'static str, Option<String>) {
    return match refusal {
        CloneSlotRefusal::Walk(_walk) => (WALK_STEP, None),
        CloneSlotRefusal::Store(_)
        | CloneSlotRefusal::ParentIdentity {
            ..
        }
        | CloneSlotRefusal::SlotIdentity {
            ..
        }
        | CloneSlotRefusal::TargetAnotherObject {
            ..
        } => (IDENTITY_COMPARE_STEP, None),
    };
}

/// The errno's name,
/// where a call answered one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = refusal.errno ? errnoName(refusal.errno) : undefined;
/// ```
fn the_errno_name(errno: i32) -> Option<String> {
    return if errno == 0 {
        None
    } else {
        Some(ErrnoWord::of_number(Some(errno)).text())
    };
}
