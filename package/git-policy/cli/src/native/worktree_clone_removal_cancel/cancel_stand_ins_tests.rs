//! What:
//!  The stand-ins and fixtures the cancel command's controls share:
//!  a request writer that records what it was asked to file,
//!  a staging folder that records its settlements,
//!  a slot claim that records its takes,
//!  a reader no control needs,
//!  one recorded binding and one plan writer,
//!  and one throwaway repository with the slot's pointer.
//! Why:
//!  The design's order is the fact the controls must prove,
//!  and a helper two controls need lives here rather than in both
//!  (`doc/how-to-write-this-packages-code-and-tests.md`,
//!  "Write less code").
//!  Every control takes its own fixture folder,
//!  so the name a helper builds is the test's own name and no two tests share one folder.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const order: string[] = []; const request = new RecordingMountRequest(order, answer);
//! ```

use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, ReadFailure};
use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, MountOperation, intent_text};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{CLONE, CLONE_CREATETXG, CLONE_GUID, ORIGIN_SNAPSHOT, RemovalFixture, ScriptedPool, SLOT, guid, the_stats};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_removal_claim::{ClaimHeld, TheRemovalClaim};
use crate::worktree_clone_removal_cancel::the_reversing_request::{MountAnswer, TheMountRequest};
use crate::worktree_clone_removal_cancel::the_staged_state::{StagedSettlement, TheStagedFolder};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, NvList};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// What one seam was asked for,
/// in the order the callers asked.
pub type TheOrder = Rc<RefCell<Vec<String>>>;

/// The nonce the controls' clone carries.
pub const THE_NONCE: &str = "3f2a0000000000000000000003f2a000";

/// One fresh order.
pub fn an_order() -> TheOrder {
    return Rc::new(RefCell::new(Vec::new()));
}

/// One order's steps,
/// as the control reads them.
pub fn steps(order: &TheOrder) -> Vec<String> {
    return order.borrow().clone();
}

/// A nonce of the shape a request needs.
pub fn a_nonce() -> RequestNonce {
    return RequestNonce::parse(THE_NONCE).expect("a nonce");
}

/// One folder identity,
/// written from its own grammar so a control needs no folder to answer it.
pub fn an_identity() -> FolderIdentity {
    return FolderIdentity::decode("2:3:4").expect("the grammar reads one identity");
}

/// One add's recorded identities,
/// over the removal fixtures' own sample values.
pub fn a_recorded_binding() -> RecordedBinding {
    return RecordedBinding {
        slot: String::from(SLOT),
        slot_path: PathBuf::from("/fixture/repo/.claude/worktrees/fix-flaky-test"),
        pool: String::from("tank"),
        pool_guid: guid(1_234_567_890_123_456_789),
        clone_dataset: String::from(CLONE),
        clone_guid: guid(CLONE_GUID),
        clone_createtxg: CLONE_CREATETXG,
        origin_dataset: String::from("tank/mono-checkout"),
        origin_dataset_guid: guid(4_815_162_342_109_876_543),
        origin_snapshot: String::from(ORIGIN_SNAPSHOT),
        origin_snapshot_guid: guid(11_050_373_805_378_911_967),
        origin_snapshot_createtxg: 15,
        registration_id: Vec::from(SLOT.as_bytes()),
        registration_identity: an_identity(),
    };
}

/// What:
///  One plan written where the removal writes one.
/// Why:
///  The plan's folder is the common directory's own `0700` folder,
///  and a control that made it any other way would test the reader's folder check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = aPlanThatStands(common, plan);
/// ```
pub fn a_plan_that_stands(common: &Path, plan: &DestructionPlan) -> PathBuf {
    let folder: PathBuf = plans_folder(common);
    crate::private_storage::create_private_directory_all(folder.as_path()).expect("the plan folder");
    let path: PathBuf = folder.join(plan.facts.clone_guid.file_name());
    std::fs::write(path.as_path(), plan.encode().expect("the plan encodes")).expect("the plan file");
    return path;
}

/// What:
///  One throwaway repository with the removal's records and a `.git` pointer naming its own
///  registration.
/// Why:
///  The proof reads a real binding and a real pointer,
///  and the checkout the pointer is proved against is the fixture's own,
///  so a control changes one record and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [fixture, checkout] = aRepositoryWithItsPointer('a-control');
/// ```
pub fn a_repository_with_its_pointer(name: &str) -> (RemovalFixture, PathBuf) {
    let fixture: RemovalFixture = RemovalFixture::healthy(name);
    let checkout: PathBuf = fixture.common_dir.parent().expect("the common directory is inside the checkout").to_path_buf();
    let registration: PathBuf = checkout.join(".git").join("worktrees").join(SLOT);
    std::fs::write(fixture.slot_path.join(".git").as_path(), format!("gitdir: {}\n", registration.display())).expect("the slot's pointer");
    return (fixture, checkout);
}

/// What:
///  One clone's own request in the shape the kernel's reply carries it:
///  the property name mapped to a nested list with `value` and `source`.
/// Why:
///  The reader judges exactly this shape,
///  and a bare value would be a reply the kernel never sends;
///  the source is the dataset's own name, which is what a local value's source is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const properties = (intent) => new NvList([[DESIRED_STATE_PROPERTY, { value: intentText(...), source: 'tank/wt/fix-flaky-test' }]]);
/// ```
pub fn an_intent_property(intent: &MountIntent) -> NvList {
    let mut inner: NvList = NvList::new();
    inner.add_str("value", intent_text(intent.operation, &intent.nonce).as_str());
    inner.add_str("source", CLONE);
    let mut properties: NvList = NvList::new();
    properties.add_nvlist(DESIRED_STATE_PROPERTY, inner);
    return properties;
}

/// One clone's own request as the pool's reply.
pub fn a_pool_carrying(fixture: &RemovalFixture, operation: MountOperation) -> ScriptedPool {
    let intent: MountIntent = MountIntent { operation, nonce: a_nonce() };
    return fixture
        .healthy_pool()
        .with(CLONE, DatasetIdentity { stats: the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT), properties: an_intent_property(&intent) });
}

/// What:
///  A reader the records' proof never needs,
///  which answers every read with a refusal a control's failure names.
/// Why:
///  A control that reached it would otherwise read a machine's pool,
///  and the refusal says which read surprised the control.
pub struct NeverRead;

impl DatasetReader for NeverRead {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        return Err(ReadFailure { said: format!("the control does not read `{dataset}`"), errno: None });
    }

    fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
        return Err(String::from("the control does not walk the parent"));
    }
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
///  A slot claim that records its take and answers with a scripted one.
/// Why:
///  A caller's mistake must leave the claim untouched,
///  and a live command must refuse before anything is read.
pub struct ScriptedClaim {
    /// The order every seam appends to.
    pub order: TheOrder,
    /// The refusal the take returns,
    /// or nothing when the claim is held.
    pub refusal: Option<CancelRefusal>,
}

impl TheRemovalClaim for ScriptedClaim {
    fn take(&self, slot: &str) -> Result<ClaimHeld, CancelRefusal> {
        self.order.borrow_mut().push(format!("claim:{slot}"));
        return match &self.refusal {
            Some(refusal) => Err(refusal.clone()),
            None => Ok(ClaimHeld::for_a_control("4242")),
        };
    }
}
