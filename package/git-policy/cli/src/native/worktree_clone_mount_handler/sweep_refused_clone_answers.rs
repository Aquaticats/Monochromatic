//! What:
//!  The answer to a clone the checks refused,
//!  and the one receipt a refusal files:
//!  the `refused` receipt of an unavailable key.
//! Why:
//!  The project's receipt rule is that a refusal files no receipt but an
//!  unavailable key's,
//!  because that is the one a waiting command can act on
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Receipts for refusals").
//!  A clone whose own mount already stands at its slot keeps the `done`
//!  receipt it filed,
//!  because the key question is not the answer to the request that mount
//!  answered.
//!  It is its own file so a reader of the turn's refusals need not read the
//!  receipts of its mounts
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = refusedByTheChecks(context, 'zpcachyos/…/clones/one', facts, refusal);
//! ```

use super::sweep_clone_check::{
    CloneRefusal,
    slot_of,
};
use super::sweep_dataset_facts::CloneObjectFacts;
use super::sweep_turn_context_and_outcome::{
    CloneOutcome,
    RECEIPT_STEP,
    SweepContext,
};
use crate::diagnostic_log::warn;
use crate::worktree_clone_mount::LookAgainAtSlotRequest;
use crate::worktree_clone_mount_protocol::{
    CloneMountReceipt,
    IntentReading,
    ReceiptOutcome,
    read_intent,
};

/// Module tag for diagnostics.
const TAG: &str = "sweepRefusedCloneAnswers";

/// The answer to a clone the checks refused.
/// An unavailable key is the one refusal that files a receipt,
/// because it is the one a waiting command can act on.
pub(super) fn refused_by_the_checks(
    context: &SweepContext<'_>,
    name: &str,
    facts: &CloneObjectFacts,
    refusal: &CloneRefusal,
) -> CloneOutcome {
    warn(TAG, refusal.message().as_str());
    // A clone whose own mount already stands at its slot keeps the done receipt it filed: the key question that now
    // refuses its next attach is not the answer to the request that mount answered.
    if matches!(refusal, CloneRefusal::KeyUnavailable { .. })
        && !the_refused_clones_mount_stands(context, name, facts)
        && let Some(receipt) = the_key_refusals_receipt(context, facts, name, refusal.step())
        && let Err(filed) = context.receipts.publish(&receipt)
    {
        return CloneOutcome::Failed {
            name: String::from(name),
            slot: None,
            step: String::from(RECEIPT_STEP),
            said: format!(
                "{} The receipt could not be filed either: {}",
                refusal.message(),
                filed.message()
            ),
        };
    }
    return CloneOutcome::Refused {
        name: String::from(name),
        slot: None,
        step: String::from(refusal.step()),
        said: refusal.message(),
    };
}

/// Whether the clone a refusal is about already holds its own mount at its
/// slot,
/// which is when a `done` receipt must be kept rather than overwritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stands = theRefusedClonesMountStands(context, name, facts);
/// ```
fn the_refused_clones_mount_stands(
    context: &SweepContext<'_>,
    name: &str,
    facts: &CloneObjectFacts,
) -> bool {
    let Ok(slot) = slot_of(facts.name.as_str(), context.parent_dataset) else {
        return false;
    };
    let request: LookAgainAtSlotRequest<'_> = LookAgainAtSlotRequest {
        checkout: context.checkout,
        slot: slot.as_str(),
    };
    return context.standing.standing_mount_of(&request, name).is_some();
}

/// The receipt an unavailable key files,
/// where the clone's GUID and its request both parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receipt = keyRefusalsReceipt(context, facts, name, 'key-unavailable');
/// ```
fn the_key_refusals_receipt(
    context: &SweepContext<'_>,
    facts: &CloneObjectFacts,
    name: &str,
    step: &str,
) -> Option<CloneMountReceipt> {
    let dataset =
        crate::worktree_clone_mount_protocol::DatasetGuid::parse(facts.guid.to_string().as_str())
            .ok()?;
    let IntentReading::Request(request) = read_intent(Some(&facts.properties), name) else {
        return None;
    };
    return Some(CloneMountReceipt {
        dataset,
        operation: request.operation,
        nonce: request.nonce,
        outcome: ReceiptOutcome::Refused,
        mount: None,
        failed_step: Some(String::from(step)),
        errno: None,
        boot: String::from(context.boot),
        time: (context.time)(),
    });
}
