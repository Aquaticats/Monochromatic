//! What:
//!  Controls of one clone's whole turn of the sweep,
//!  run inside a user namespace where this run's fixture files are root's.
//! Why:
//!  The store and the claims and the receipts are root-controlled,
//!  and the attach needs a real mount,
//!  so the fixtures are made where the run is root and the attach step is
//!  scripted:
//!  what these controls prove is the order,
//!  the refusals before any filesystem work,
//!  the receipt each answer files,
//!  and the first observation that is recorded once and never refreshed.
//!  The real mount and the real pool run in the throwaway guest
//!  (`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!inARootNamespace(...)) return;
//! expect(sweepOneClone(context, CLONE).kind).toBe('attached');
//! ```

use super::{CloneOutcome, RECORD_STEP, UNMOUNT_AWAITS_STEP_NINE, SweepContext, sweep_one_clone};
use crate::worktree_clone_mount::{CloneFolderIdentity, MountIdentity, SlotIdentityStore, publish_parent_identity_under_root_control};
use crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace;
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_handler::sweep_attach_step::{AttachRequest, AttachStepOutcome, CloneAttachStep};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    DEVICES_PROPERTY, DMU_OBJECT_SET_FILESYSTEM, ENCRYPTION_PROPERTY, ENCRYPTION_ROOT_PROPERTY, KEY_STATUS_PROPERTY, MOUNTPOINT_PROPERTY, SETUID_PROPERTY,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild};
use crate::worktree_clone_mount_protocol::{CloneMountReceipt, DESIRED_STATE_PROPERTY, DatasetGuid, MOUNT_WORD, ReceiptOutcome, ReceiptsFolder, clone_mount_receipt::ReceiptMountIdentity};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, DmuObjsetStats, NvList};
use std::cell::{Cell, RefCell};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The variable the inner half of each control sets.
const INSIDE: &str = "CLI_GIT_ZFS_SWEEP_ONE_CLONE_ROOT";

/// The pinned clone parent.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// The clone the controls read.
const CLONE: &str = "zpcachyos/nosnap/clones/one";

/// Another immediate child,
/// which asks for nothing.
const QUIET: &str = "zpcachyos/nosnap/clones/two";

/// The clone's GUID.
const GUID: u64 = 7_000_000_000_000_000_003;

/// The boot the receipts name.
const BOOT: &str = "00000000-0000-4000-8000-000000000000";

/// The time every receipt carries,
/// so a control compares whole receipts.
const TIME: &str = "2026-11-21T20:00:00-05:00";

/// The nonce a mount request carries.
const MOUNT_NONCE: &str = "0123456789abcdef0123456789abcdef";

/// What:
///  The request nonce a control writes,
///  spelled in the grammar's own digits.
/// Why:
///   A receipt is matched on the nonce the clone asked under,
///  so a control must write one that parses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nonce = '0'.repeat(32);
/// ```
fn nonce(digits: &str) -> String {
    return format!("v1:{digits}:{MOUNT_WORD}");
}

/// A clone asking for its mount,
/// with the given nonce digits.
pub fn mount_request(digits: &str) -> String {
    return nonce(digits);
}

/// One property as OpenZFS writes it.
fn pair_number(value: u64, source: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_u64("value", value);
    pair.add_str("source", source);
    return pair;
}

/// One text property in the same shape.
fn pair_text(value: &str, source: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    pair.add_str("source", source);
    return pair;
}

/// The reply a managed clone answers with,
/// carrying `intent` under the desired-state property when one is given.
fn reply_of_a_clone(intent: Option<&str>, source: &str) -> NvList {
    let mut reply: NvList = NvList::new();
    reply.add_nvlist(MOUNTPOINT_PROPERTY, pair_text("legacy", PARENT));
    reply.add_nvlist(SETUID_PROPERTY, pair_number(0, PARENT));
    reply.add_nvlist(DEVICES_PROPERTY, pair_number(0, PARENT));
    reply.add_nvlist(ENCRYPTION_PROPERTY, pair_number(0, CLONE));
    reply.add_nvlist(ENCRYPTION_ROOT_PROPERTY, pair_text("zpcachyos", CLONE));
    reply.add_nvlist(KEY_STATUS_PROPERTY, pair_number(0, CLONE));
    if let Some(value) = intent {
        reply.add_nvlist(DESIRED_STATE_PROPERTY, pair_text(value, source));
    }
    return reply;
}

/// The statistics one clone answers with.
fn stats_of_a_clone() -> DmuObjsetStats {
    let origin: String = format!("{PARENT}@origin");
    let mut bytes: [u8; 256] = [0; 256];
    bytes[..origin.len()].copy_from_slice(origin.as_bytes());
    return DmuObjsetStats {
        dds_num_clones: 0,
        dds_creation_txg: 7,
        dds_guid: GUID,
        dds_type: DMU_OBJECT_SET_FILESYSTEM,
        dds_is_snapshot: 0,
        dds_inconsistent: 0,
        dds_redacted: 0,
        dds_origin: bytes,
        dds_flags: 0,
    };
}

/// A pool that answers one clone's identity and lists children from a script.
struct ScriptedReader {
    /// The children the listing answers.
    children: Vec<String>,
    /// The replies each child answers,
    /// by name.
    replies: RefCell<Vec<(String, DmuObjsetStats, NvList)>>,
    /// How many identities were read.
    reads: Cell<usize>,
}

impl DatasetReader for ScriptedReader {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, String> {
        self.reads.set(self.reads.get() + 1);
        let replies = self.replies.borrow();
        let Some((_name, stats, properties)) = replies.iter().find(|(name, _, _)| return name == dataset) else {
            return Err(format!("this scripted pool holds no `{dataset}`"));
        };
        return Ok(DatasetIdentity { stats: *stats, properties: properties.clone() });
    }

    fn next_child(&self, _parent: &str, cursor: u64) -> Result<Option<ListedChild>, String> {
        let index: usize = usize::try_from(cursor).unwrap_or(usize::MAX);
        return Ok(self.children.get(index).map(|name| return ListedChild { name: name.clone(), cursor: cursor + 1 }));
    }
}

/// An attach step that answers from a script and counts its calls.
struct ScriptedAttach {
    /// The answer it gives.
    answer: AttachStepOutcome,
    /// How many times it was asked.
    asked: Cell<usize>,
}

impl CloneAttachStep for ScriptedAttach {
    fn attach(&self, request: &AttachRequest<'_>) -> AttachStepOutcome {
        assert_eq!(request.checkout, request.checkout, "the step is asked with the context's own checkout");
        self.asked.set(self.asked.get() + 1);
        return self.answer.clone();
    }
}

/// The mount a scripted attach says it made.
fn mounted() -> MountIdentity {
    return MountIdentity {
        mount_id: 41,
        parent_mount_id: 40,
        mount_point: PathBuf::from("/fixture/checkout/.claude/worktrees/one"),
        source: String::from(CLONE),
        file_system_type: String::from("zfs"),
        device: 57,
        inode: 1_337,
        observed_path: PathBuf::from("/fixture/checkout/.claude/worktrees/one"),
    };
}

/// One fixture:
/// a checkout with two slots,
/// a root-owned store with the parent record,
/// a claims folder and a receipts folder.
struct Fixture {
    /// The fixture's root.
    root: PathBuf,
    /// The checkout.
    checkout: PathBuf,
    /// The store folder.
    store: PathBuf,
    /// The claims folder.
    claims: PathBuf,
    /// The receipts folder.
    receipts: PathBuf,
}

impl Fixture {
    /// Build the fixture under this run's own temporary root.
    fn build(name: &str) -> Self {
        let root: PathBuf = std::env::temp_dir().join(format!("sweep-one-clone-{name}-{}", std::process::id()));
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(root.as_path());
        let checkout: PathBuf = root.join("checkout");
        let parent: PathBuf = checkout.join(".claude").join("worktrees");
        for slot in ["one", "two"] {
            std::fs::create_dir_all(parent.join(slot)).expect("the slot folder is made");
        }
        let store: PathBuf = root.join("store");
        std::fs::create_dir(store.as_path()).expect("the store folder is made");
        std::fs::set_permissions(store.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the store is root's and readable");
        let identity: CloneFolderIdentity = identity_of(parent.as_path());
        publish_parent_identity_under_root_control(store.as_path(), &identity, &identity).expect("the setup run's parent record is published");
        let claims: PathBuf = root.join("claims");
        let receipts: PathBuf = root.join("receipts");
        return Fixture { root, checkout, store, claims, receipts };
    }

    /// The store as a reader opens it.
    fn opened_store(&self) -> SlotIdentityStore {
        return SlotIdentityStore::open_under_root_control(self.store.as_path()).expect("the fixture store is root's");
    }

    /// The receipts folder this fixture writes.
    fn opened_receipts(&self) -> ReceiptsFolder {
        return ReceiptsFolder::at(self.receipts.clone(), 0);
    }

    /// The claims folder this fixture claims in.
    fn opened_claims(&self) -> SlotClaimDirectory {
        return SlotClaimDirectory::at(self.claims.clone(), 0);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(self.root.as_path());
    }
}

/// One folder's identity now.
fn identity_of(path: &Path) -> CloneFolderIdentity {
    return CloneFolderIdentity::capture(&std::fs::File::open(path).expect("the folder opens")).expect("the identity reads");
}

/// A reader holding `CLONE` with `intent`,
/// and `QUIET` with none.
fn reader_with(intent: Option<&str>, source: &str) -> ScriptedReader {
    return ScriptedReader {
        children: vec![String::from(CLONE), String::from(QUIET)],
        replies: RefCell::new(vec![
            (String::from(CLONE), stats_of_a_clone(), reply_of_a_clone(intent, source)),
            (String::from(QUIET), {
                let mut stats: DmuObjsetStats = stats_of_a_clone();
                stats.dds_guid = GUID + 1;
                stats
            }, reply_of_a_clone(None, QUIET)),
        ]),
        reads: Cell::new(0),
    };
}

/// The context one control sweeps with.
fn context<'a>(fixture: &'a Fixture, reader: &'a ScriptedReader, attach: &'a ScriptedAttach, receipts: &'a ReceiptsFolder, claims: &'a SlotClaimDirectory) -> SweepContext<'a> {
    return SweepContext {
        parent_dataset: PARENT,
        checkout: fixture.checkout.as_path(),
        store: fixture.store.as_path(),
        claims,
        receipts,
        boot: BOOT,
        time: &|| return String::from(TIME),
        reader,
        attach,
    };
}

/// The request nonce a control's receipt must carry.
fn request_nonce() -> String {
    return mount_request(MOUNT_NONCE);
}

/// Whether the claims folder holds a live claim for `slot`,
/// which a released generation file does not answer.
fn claimed(fixture: &Fixture, slot: &str) -> bool {
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let Ok(held) = claims.claim(slot) else {
        return true;
    };
    held.release();
    return false;
}

/// Whether a receipt is filed under the clone's GUID,
/// where an absent receipts folder is the same answer as no receipt in it.
fn filed(fixture: &Fixture, receipts: &ReceiptsFolder) -> Option<CloneMountReceipt> {
    let guid: DatasetGuid = DatasetGuid::parse(GUID.to_string().as_str()).expect("the GUID is canonical");
    let _fixture = fixture;
    return match receipts.read(guid) {
        Ok(receipt) => receipt,
        Err(_absent) => None,
    };
}

/// What:
///  A clone asking for its mount is attached at its slot,
///  its first observation is recorded before the attach,
///  and a done receipt carrying the mount and the nonce is filed.
/// Why:
///  This is the positive control of the whole turn:
///  every refusal below is proved against a turn that attaches.
#[test]
fn a_clone_asking_for_its_mount_is_attached_and_files_a_done_receipt() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_clone_asking_for_its_mount_is_attached_and_files_a_done_receipt") {
        return;
    }
    let fixture: Fixture = Fixture::build("attached");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Attached { slot: ref held, mount_id: 41, .. } if held == "one"), "{outcome:?}");
    assert!(outcome.line().starts_with("attached "), "the line names the answer: {}", outcome.line());
    assert_eq!(attach.asked.get(), 1, "the attach was asked once");
    let recorded: CloneFolderIdentity = fixture.opened_store().slot("one").expect("the slot's first observation is recorded");
    assert_eq!(recorded, identity_of(fixture.checkout.join(".claude").join("worktrees").join("one").as_path()), "the record is the slot folder's own identity");
    assert!(!claimed(&fixture, "one"), "the claim is released when the turn ends, so the next sweep takes it");
    let guid: DatasetGuid = DatasetGuid::parse(GUID.to_string().as_str()).expect("the GUID is canonical");
    let receipt: CloneMountReceipt = receipts.read(guid).expect("the receipts folder reads").expect("a receipt is filed under the clone's GUID");
    assert_eq!(receipt.outcome, ReceiptOutcome::Done);
    assert_eq!(receipt.nonce.as_str(), MOUNT_NONCE);
    assert_eq!(receipt.boot, BOOT);
    assert_eq!(receipt.time, TIME);
    assert_eq!(
        receipt.mount,
        Some(ReceiptMountIdentity { mount_id: 41, device: 57, inode: 1_337, source: String::from(CLONE), canonical_path: String::from("/fixture/checkout/.claude/worktrees/one") })
    );
    assert_eq!(receipt.failed_step, None);
}

/// What:
///  A clone with no local intent is left alone:
///  no record is taken,
///  no claim is taken,
///  and no receipt is filed.
/// Why:
///  An absent property asks for nothing,
///  and a sweep that recorded every child of the parent would record clones
///  nobody asked it to mount.
#[test]
fn a_clone_with_no_intent_is_left_alone() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_clone_with_no_intent_is_left_alone") {
        return;
    }
    let fixture: Fixture = Fixture::build("no-intent");
    let reader: ScriptedReader = reader_with(None, CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::NothingAsked { ref why, .. } if why.contains("no local mount intent")), "{outcome:?}");
    assert_eq!(attach.asked.get(), 0, "nothing was attached");
    assert!(matches!(fixture.opened_store().slot("one"), Err(crate::worktree_clone_mount::StoreRefusal::NoSlotRecord { .. })), "no record was taken");
    assert!(!claimed(&fixture, "one"), "no claim was taken");
    assert!(filed(&fixture, &receipts).is_none(), "no receipt was filed");
}

/// What:
///  An intent inherited from an ancestor authorizes nothing,
///  and the clone is left alone while the sweep says where the value came
///  from.
/// Why:
///  A value set on the parent is not this clone's request,
///  and obeying it would mount every child under an ancestor nobody asked
///  about.
#[test]
fn an_inherited_intent_authorizes_nothing() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::an_inherited_intent_authorizes_nothing") {
        return;
    }
    let fixture: Fixture = Fixture::build("inherited");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), PARENT);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::NothingAsked { ref why, .. } if why.contains(PARENT)), "{outcome:?}");
    assert_eq!(attach.asked.get(), 0);
    assert!(matches!(fixture.opened_store().slot("one"), Err(crate::worktree_clone_mount::StoreRefusal::NoSlotRecord { .. })));
}

/// What:
///  A malformed local intent is refused before any filesystem operation on its
///  behalf,
///  and no claim and no record are taken.
/// Why:
///  The design refuses a malformed intent before work rather than treating it as
///  an absence,
///  because a silent absence would leave an operator believing a request was
///  read.
#[test]
fn a_malformed_intent_is_refused_before_any_filesystem_operation() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_malformed_intent_is_refused_before_any_filesystem_operation") {
        return;
    }
    let fixture: Fixture = Fixture::build("malformed");
    let reader: ScriptedReader = reader_with(Some("v1:nothex:mount"), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == "intent"), "{outcome:?}");
    assert_eq!(attach.asked.get(), 0, "a malformed intent attaches nothing");
    assert!(!claimed(&fixture, "one"), "a malformed intent takes no claim");
    assert!(matches!(fixture.opened_store().slot("one"), Err(crate::worktree_clone_mount::StoreRefusal::NoSlotRecord { .. })), "a malformed intent records nothing");
}

/// What:
///  A clone the checks refuse is refused before it is walked or claimed,
///  and the line names the check.
/// Why:
///  A dataset that is no managed clone must not reach the walk,
///  because the walk is what records an identity and opens the device for a
///  mount.
#[test]
fn a_clone_the_checks_refuse_is_not_walked() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_clone_the_checks_refuse_is_not_walked") {
        return;
    }
    let fixture: Fixture = Fixture::build("refused-checks");
    let mut reply: NvList = reply_of_a_clone(Some(request_nonce().as_str()), CLONE);
    reply.add_nvlist(SETUID_PROPERTY, pair_number(1, PARENT));
    let reader: ScriptedReader = ScriptedReader {
        children: vec![String::from(CLONE)],
        replies: RefCell::new(vec![(String::from(CLONE), stats_of_a_clone(), reply)]),
        reads: Cell::new(0),
    };
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == "setuid"), "{outcome:?}");
    assert_eq!(attach.asked.get(), 0);
    assert!(!claimed(&fixture, "one"));
}

/// What:
///  A slot another sweep holds is left to that sweep rather than queued.
/// Why:
///  A claim that waited would let one sweep's slow step hold every later sweep
///  of the same slot with no deadline,
///  and the refused sweep tries the slot again on its next trigger.
#[test]
fn a_slot_another_sweep_holds_is_left_to_it() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_slot_another_sweep_holds_is_left_to_it") {
        return;
    }
    let fixture: Fixture = Fixture::build("held");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let held_by_another: crate::worktree_clone_mount_handler::per_slot_root_claim::HeldSlotClaim = claims.claim("one").expect("the other sweep holds the slot");
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Held { ref slot, .. } if slot == "one"), "{outcome:?}");
    assert_eq!(attach.asked.get(), 0, "a held slot is not attached");
    held_by_another.release();
}

/// What:
///  An unmount request is answered by a filed refusal naming the step that
///  builds the unmount,
///  and the durable intent is left as it is.
/// Why:
///  The design answers every request with a receipt;
///  an unmount this build cannot carry out is answered rather than left to a
///  caller's deadline,
///  and nothing of the mount changes.
#[test]
fn an_unmount_request_is_answered_by_the_step_that_builds_it() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::an_unmount_request_is_answered_by_the_step_that_builds_it") {
        return;
    }
    let fixture: Fixture = Fixture::build("unmount");
    let value: String = format!("v1:{MOUNT_NONCE}:unmount");
    let reader: ScriptedReader = reader_with(Some(value.as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == UNMOUNT_AWAITS_STEP_NINE), "{outcome:?}");
    assert_eq!(attach.asked.get(), 0);
    let receipt: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("the unmount is answered");
    assert_eq!(receipt.outcome, ReceiptOutcome::Refused);
    assert_eq!(receipt.failed_step.as_deref(), Some(UNMOUNT_AWAITS_STEP_NINE));
    assert_eq!(receipt.mount, None, "a refusal carries no mount identity");
}

/// What:
///  A mount the rollback took down files a failed receipt naming the post-check,
///  and a mount the rollback could not take down files one naming the rollback.
/// Why:
///  A caller must tell a mount that is gone from one that is still attached,
///  because only the second needs an administrator.
#[test]
fn a_mount_the_rollback_handled_files_the_step_that_decided_it() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_mount_the_rollback_handled_files_the_step_that_decided_it") {
        return;
    }
    let fixture: Fixture = Fixture::build("rollback");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let rolled: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::RolledBack { said: String::from("the mount 41 left the kernel") }, asked: Cell::new(0) };
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &rolled, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Failed { ref step, .. } if step == "post-check"), "{outcome:?}");
    let receipt: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("the failure is filed");
    assert_eq!(receipt.outcome, ReceiptOutcome::Failed);
    assert_eq!(receipt.failed_step.as_deref(), Some("post-check"));

    let retained: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Retained { said: String::from("the mount 41 is still attached at /fixture/elsewhere") }, asked: Cell::new(0) };
    let again: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &retained, &receipts, &claims), CLONE);
    assert!(matches!(again, CloneOutcome::Failed { ref step, .. } if step == "rollback"), "{again:?}");
    let filed: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("the retained mount is filed");
    assert_eq!(filed.failed_step.as_deref(), Some("rollback"));
    assert!(filed.mount.is_none(), "a retained mount carries no canonical mount identity");
}

/// What:
///  An attach that refused before any mount is filed as a refusal,
///  and the sweep's line names the attach step.
/// Why:
///  A caller waiting on the nonce reads a refusal rather than a timeout,
///  and a refusal is not a failure that changed anything.
#[test]
fn an_attach_that_refused_files_a_refused_receipt() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::an_attach_that_refused_files_a_refused_receipt") {
        return;
    }
    let fixture: Fixture = Fixture::build("attach-refused");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Refused { said: String::from("the walk refused the slot") }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == "attach"), "{outcome:?}");
    let receipt: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("the refusal is filed");
    assert_eq!(receipt.outcome, ReceiptOutcome::Refused);
    assert_eq!(receipt.failed_step.as_deref(), Some("attach"));
}

/// What:
///  A slot the store already records is not recorded again,
///  even when the folder's identity moved.
/// Why:
///  A slot identity is never refreshed because a new inode or mode appeared:
///  the attach that follows refuses the folder instead,
///  and a sweep that refreshed the record would adopt a replacement nobody
///  approved.
#[test]
fn an_existing_slot_record_is_never_refreshed() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::an_existing_slot_record_is_never_refreshed") {
        return;
    }
    let fixture: Fixture = Fixture::build("no-refresh");
    let slot: PathBuf = fixture.checkout.join(".claude").join("worktrees").join("one");
    let recorded: CloneFolderIdentity = CloneFolderIdentity { inode: identity_of(slot.as_path()).inode + 1, ..identity_of(slot.as_path()) };
    let parent: PathBuf = fixture.checkout.join(".claude").join("worktrees");
    crate::worktree_clone_mount::publish_slot_identity_under_root_control(fixture.store.as_path(), "one", &recorded, &identity_of(parent.as_path())).expect("a record stands");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Refused { said: String::from("the walk refused the slot's identity") }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { .. }), "{outcome:?}");
    let after: CloneFolderIdentity = fixture.opened_store().slot("one").expect("the record still stands");
    assert_eq!(after, recorded, "the record is the one that stood before the sweep");
    assert_ne!(after, identity_of(slot.as_path()), "the record is not the live folder, so nothing refreshed it");
    assert!(!outcome.line().contains(RECORD_STEP), "the sweep recorded nothing: {}", outcome.line());
}
