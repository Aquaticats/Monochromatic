//! What:
//!  The unmount arm of one clone's turn:
//!  the slot's claim,
//!  the slot's record,
//!  the account's cache and its pin,
//!  the staged unmount a stopped sweep left behind,
//!  the slot read again under the claim,
//!  and the unmount turn itself.
//! Why:
//!  An unmount this build cannot carry out is answered,
//!  never left to silence a caller reads as a timeout,
//!  and each refusal on the way is answered by the unmount's own file
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Receipts for refusals";
//!  `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "The sweep plan step 8").
//!  The answer to an unmount is its own file because it is not the answer to a
//!  mount,
//!  and each has to stay short enough to read whole
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = answerUnmount(context, verified, request);
//! ```

use super::sweep_clone_check::VerifiedClone;
use super::sweep_turn_context_and_outcome::{
    CloneOutcome,
    SweepContext,
};
use crate::worktree_clone_mount::{
    LookAgainAtSlotRequest,
    SlotIdentityStore,
    look_again_at_the_slot,
};
use crate::worktree_clone_mount_protocol::MountIntent;
use crate::worktree_clone_unmount::slot_walk_calls::LiveSlotWalk;
use crate::worktree_clone_unmount::staged_folder_identity::StagedFolderIdentity;
use crate::worktree_clone_unmount::staged_unmount::recover_a_staged_unmount;
use crate::worktree_clone_unmount::staging_calls::LiveStagingCalls;
use crate::worktree_clone_unmount::unmount_checks::ExpectedMount;
use crate::worktree_clone_unmount::unmount_turn::{
    UnmountOutcome,
    UnmountRequest,
    unmount_the_clone,
};
use crate::worktree_clone_unmount::user_cache_pin::{
    USER_CACHE_PIN_PATH,
    UserCachePin,
    open_cache,
    passwd_home_of,
    read_pin,
};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  Answer an unmount request with the step that builds it,
///  and leave the durable intent as it is.
/// Why:
///  The design answers every request with a receipt,
///  and an unmount that this build cannot carry out is answered rather than by
///  silence the caller reads as a timeout;
///  the project's receipt rule files nothing for a refusal but an unavailable
///  key,
///  so the answer is the sweep's line and the handler's journal,
///  and the durable intent stands for plan step 9.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = answerUnmount(verified, request);
/// ```
pub(super) fn answer_unmount(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    request: &MountIntent,
) -> CloneOutcome {
    let claim = match context.claims.claim(verified.slot.as_str()) {
        Ok(claim) => claim,
        Err(refusal) => {
            return CloneOutcome::Held {
                name: verified.dataset_name.clone(),
                slot: verified.slot.clone(),
                said: refusal.message(),
            };
        }
    };
    let outcome: CloneOutcome = answer_unmount_under_the_claim(context, verified, request);
    claim.release();
    return outcome;
}

/// The unmount turn once this clone's slot is claimed,
/// so two handlers cannot act on one clone.
fn answer_unmount_under_the_claim(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    request: &MountIntent,
) -> CloneOutcome {
    let recorded = match SlotIdentityStore::open_under_root_control(context.store)
        .and_then(|store| return store.slot(verified.slot.as_str()))
    {
        Ok(recorded) => recorded,
        Err(refusal) => {
            return super::sweep_unmount_answers::refuse_an_unmount(
                context,
                verified,
                request,
                "slot-identity",
                refusal.message(),
            );
        }
    };
    let owner_uid = match std::fs::metadata(context.checkout) {
        Ok(facts) => facts.uid(),
        Err(error) => {
            return super::sweep_unmount_answers::refuse_an_unmount(
                context,
                verified,
                request,
                "account-home",
                format!("the checkout's owner could not be read: {error}"),
            );
        }
    };
    let home = match passwd_home_of(owner_uid) {
        Ok(home) => home,
        Err(refusal) => {
            return super::sweep_unmount_answers::refuse_an_unmount(
                context,
                verified,
                request,
                "account-home",
                refusal.message(),
            );
        }
    };
    let cache = match open_cache(home.as_path()) {
        Ok(cache) => cache,
        Err(refusal) => {
            return super::sweep_unmount_answers::refuse_an_unmount(
                context,
                verified,
                request,
                "user-cache",
                refusal.message(),
            );
        }
    };
    let pin: UserCachePin = match read_pin(Path::new(USER_CACHE_PIN_PATH)) {
        Ok(pin) => pin,
        Err(refusal) => {
            return super::sweep_unmount_answers::refuse_an_unmount(
                context,
                verified,
                request,
                "user-cache-pin",
                refusal.message(),
            );
        }
    };
    let canonical_slot = context
        .checkout
        .join(".claude/worktrees")
        .join(verified.slot.as_str());
    let calls = LiveStagingCalls;
    let cache_path = home.join(".cache");
    let recovery = recover_a_staged_unmount(
        &crate::worktree_clone_unmount::staged_unmount::StagedSweep {
            cache: &cache,
            cache_path: cache_path.as_path(),
            guid: verified.dataset,
            boot: context.boot,
            pin: &pin,
            calls: &calls,
        },
        canonical_slot.as_path(),
        0,
    );
    match recovery {
        crate::worktree_clone_unmount::staged_unmount::RecoveryOutcome::Finished {
            mount_id,
        } => {
            return super::sweep_unmount_answers::answer_the_unmount(
                context,
                verified,
                request,
                UnmountOutcome::Unmounted {
                    mount_id,
                    slot: canonical_slot.clone(),
                    bind_gone: true,
                },
            );
        }
        crate::worktree_clone_unmount::staged_unmount::RecoveryOutcome::BindRemoved
        | crate::worktree_clone_unmount::staged_unmount::RecoveryOutcome::NotStaged => {}
        crate::worktree_clone_unmount::staged_unmount::RecoveryOutcome::Refused {
            step,
            said,
        } => {
            return super::sweep_unmount_answers::refuse_an_unmount(
                context,
                verified,
                request,
                step.as_str(),
                said,
            );
        }
    }
    let fresh = match look_again_at_the_slot(&LookAgainAtSlotRequest {
        checkout: context.checkout,
        slot: verified.slot.as_str(),
    }) {
        Ok(fresh) => fresh,
        Err(refusal) => {
            return super::sweep_unmount_answers::refuse_an_unmount(
                context,
                verified,
                request,
                "look-again",
                refusal.message(),
            );
        }
    };
    let expected = ExpectedMount {
        mount_id: fresh.mount_id,
        parent_mount_id: fresh.parent_mount_id,
        device: fresh.slot_identity.device,
        inode: fresh.slot_identity.inode,
        canonical_slot: canonical_slot.clone(),
        source: verified.dataset_name.clone(),
        guid: verified.dataset,
    };
    let slot_identity: StagedFolderIdentity = StagedFolderIdentity::from_clone_folder(&recorded);
    let guids = super::sweep_unmount_answers::HandlerDatasetGuids {
        reader: context.reader,
    };
    let walk = LiveSlotWalk;
    let turn = UnmountRequest {
        checkout: context.checkout,
        slot: verified.slot.as_str(),
        nonce: request.nonce.as_str(),
        expected: &expected,
        slot_identity: &slot_identity,
        pin: &pin,
        cache: &cache,
        cache_path: cache_path.as_path(),
        boot: context.boot,
        calls: &calls,
        walk: &walk,
        guids: &guids,
    };
    let outcome: UnmountOutcome = unmount_the_clone(&turn);
    return super::sweep_unmount_answers::answer_the_unmount(context, verified, request, outcome);
}
