//! What:
//!  The stand-ins the command's and the cancellation's controls answer their seams with:
//!  a request writer that records what it was asked to file,
//!  a staging folder that records its settlements,
//!  and a claim that records its takes,
//!  over one shared order they append to.
//! Why:
//!  The design's order is the fact the controls must prove:
//!  the intent is written first,
//!  the staged state is settled on the answer,
//!  and the plan is the last thing touched,
//!  so each stand-in writes its own step into one list and the control reads the list.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const order: string[] = []; const request = new RecordingMountRequest(order, answer);
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, intent_text};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_removal_claim::{ClaimHeld, TheRemovalClaim};
use crate::worktree_clone_removal_cancel::the_reversing_request::{MountAnswer, TheMountRequest};
use crate::worktree_clone_removal_cancel::the_staged_state::{StagedSettlement, TheStagedFolder};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;
use std::cell::RefCell;
use std::rc::Rc;

/// What:
///  One clone's own request in the shape the kernel's reply carries it:
///  the property name mapped to a nested list with `value` and `source`.
/// Why:
///  The reader judges exactly this shape,
///  and a bare value would be a reply the kernel never sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const properties = (intent) => new NvList([[DESIRED_STATE_PROPERTY, { value: intentText(intent.operation, intent.nonce), source: the clone's own name }]]);
/// ```
pub fn an_intent_property(intent: &MountIntent) -> NvList {
    let mut inner: NvList = NvList::new();
    inner.add_str("value", intent_text(intent.operation, &intent.nonce).as_str());
    inner.add_str("source", "tank/wt/fix-flaky-test");
    let mut properties: NvList = NvList::new();
    properties.add_nvlist(DESIRED_STATE_PROPERTY, inner);
    return properties;
}

/// What one seam was asked for,
/// in the order the callers asked.
pub type TheOrder = Rc<RefCell<Vec<String>>>;

/// One fresh order.
pub fn an_order() -> TheOrder {
    return Rc::new(RefCell::new(Vec::new()));
}

/// One order's steps,
/// as the control reads them.
pub fn steps(order: &TheOrder) -> Vec<String> {
    return order.borrow().clone();
}

/// What:
///  A request writer that records the intent it was handed and answers with a scripted value.
/// Why:
///  The control must prove the writer sent a `mount` under a fresh nonce and nothing else,
///  and a refusal must leave the steps after it undone.
pub struct RecordingMountRequest {
    /// The order every seam appends to.
    pub order: TheOrder,
    /// The answer the handler gives,
    /// or the refusal it returns instead.
    pub answer: Result<MountAnswer, CancelRefusal>,
}

impl TheMountRequest for RecordingMountRequest {
    fn request_the_mount(&self, intent: &MountIntent) -> Result<MountAnswer, CancelRefusal> {
        self.order.borrow_mut().push(format!("request:{}:{}", intent.operation.word(), intent.nonce.as_str()));
        return match &self.answer {
            // A handler answers under the request's own nonce, so the stand-in does too.
            Ok(answer) => Ok(MountAnswer { nonce: intent.nonce.clone(), ..answer.clone() }),
            Err(refusal) => Err(refusal.clone()),
        };
    }
}

/// What:
///  A staging folder that records its settlement and answers with a scripted one.
/// Why:
///  The control proves the settlement happens after the answer,
///  and that a retained folder is the state the command refuses on.
pub struct ScriptedStaged {
    /// The order every seam appends to.
    pub order: TheOrder,
    /// What the settlement is.
    pub settlement: StagedSettlement,
}

impl TheStagedFolder for ScriptedStaged {
    fn settle(&self, _clone_guid: DatasetGuid) -> StagedSettlement {
        self.order.borrow_mut().push(String::from("staged"));
        return self.settlement.clone();
    }
}

/// What:
///  A claim that records its take and answers with a scripted one.
/// Why:
///  A caller's mistake must leave the claim untouched,
///  and a live removal must refuse before anything is read.
pub struct ScriptedClaim {
    /// The order every seam appends to.
    pub order: TheOrder,
    /// The refusal the take returns,
    /// or nothing when the claim is held.
    pub refusal: Option<CancelRefusal>,
}

impl TheRemovalClaim for ScriptedClaim {
    fn take(&self, clone_guid: DatasetGuid) -> Result<ClaimHeld, CancelRefusal> {
        self.order.borrow_mut().push(format!("claim:{}", clone_guid.file_name()));
        return match &self.refusal {
            Some(refusal) => Err(refusal.clone()),
            None => Ok(ClaimHeld::for_a_control("4242")),
        };
    }
}
