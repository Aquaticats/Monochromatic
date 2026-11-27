//! What:
//!  The clone's identity and its durable intent read again under the slot's
//!  claim,
//!  and whether the intent changed after the request a turn acted on.
//! Why:
//!  A sweep that waited on a claim,
//!  or ran slowly,
//!  can act on facts the clone no longer says,
//!  and the removal protocol relies on a standing `unmount` intent to exclude
//!  new attaches
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  which says "Recheck desired state and identities under a per-slot root
//!  claim").
//!  A request that arrived while the claim was held is acted on in a second turn
//!  rather than lost,
//!  which is why the same reading answers whether the intent moved.
//!  It is its own file so the turn stays short enough to read whole
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [verified, request] = rereadUnderTheClaim(context, 'zpcachyos/…/clones/one');
//! const changed = theIntentChanged(context, 'zpcachyos/…/clones/one', request);
//! ```

use super::sweep_clone_check::{
    VerifiedClone,
    verify,
};
use super::sweep_dataset_facts::{
    CloneObjectFacts,
    facts_of,
};
use super::sweep_turn_context_and_outcome::{
    CloneOutcome,
    INTENT_STEP,
    SweepContext,
};
use crate::diagnostic_log::warn;
use crate::worktree_clone_mount_protocol::{
    IntentReading,
    MountIntent,
    read_intent,
};

/// Module tag for diagnostics.
const TAG: &str = "sweepIntentReadAgain";

/// What:
///  Read the clone's identity and its intent again under the claim,
///  refusing where the first reading passed and the second does not.
/// Why:
///  A sweep that waited on a claim,
///  or ran slowly,
///  can act on facts the clone no longer says,
///  and the removal protocol relies on a standing `unmount` intent to exclude
///  new attaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [verified, request] = rereadUnderTheClaim(context, name);
/// ```
///
/// # Errors
/// The clone could not be read,
/// the checks refused it,
/// or its intent is absent,
/// inherited or malformed now.
pub(super) fn reread_under_the_claim(
    context: &SweepContext<'_>,
    name: &str,
) -> Result<(VerifiedClone, MountIntent), CloneOutcome> {
    let facts: CloneObjectFacts = match context.reader.identity(name) {
        Ok(identity) => facts_of(name, &identity),
        Err(reason) => {
            return Err(CloneOutcome::Refused {
                name: String::from(name),
                slot: None,
                step: String::from("read"),
                said: reason.message(),
            });
        }
    };
    let verified: VerifiedClone = verify(&facts, context.parent_dataset, context.checkout_dataset)
        .map_err(|refusal| {
            warn(TAG, refusal.message().as_str());
            return CloneOutcome::Refused {
                name: String::from(name),
                slot: None,
                step: String::from(refusal.step()),
                said: refusal.message(),
            };
        })?;
    return match read_intent(Some(&facts.properties), name) {
        IntentReading::Request(request) => Ok((verified, request)),
        IntentReading::Absent => Err(CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot),
            step: String::from(INTENT_STEP),
            said: String::from(
                "the clone's mount intent is gone under the claim, so nothing was attached on the reading that saw it",
            ),
        }),
        IntentReading::Inherited {
            from,
        } => Err(CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot),
            step: String::from(INTENT_STEP),
            said: format!(
                "the clone's mount intent is inherited from {from} under the claim, so nothing was attached"
            ),
        }),
        IntentReading::Refused(refusal) => Err(CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot),
            step: String::from(INTENT_STEP),
            said: refusal.message(),
        }),
    };
}

/// Whether the clone's intent changed after the request this turn acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = theIntentChanged(context, name, request); // true when a read answers another request
/// ```
pub(super) fn the_intent_changed(
    context: &SweepContext<'_>,
    name: &str,
    request: &MountIntent,
) -> bool {
    let Ok(identity) = context.reader.identity(name) else {
        return false;
    };
    let facts: CloneObjectFacts = facts_of(name, &identity);
    return match read_intent(Some(&facts.properties), name) {
        IntentReading::Request(now) => {
            now.nonce != request.nonce || now.operation != request.operation
        }
        IntentReading::Absent
        | IntentReading::Inherited {
            ..
        }
        | IntentReading::Refused(_) => true,
    };
}
