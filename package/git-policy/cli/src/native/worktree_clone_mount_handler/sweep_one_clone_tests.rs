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

use super::{CloneOutcome, IDENTITY_RECORD_STEP, LiveMountTableSource, MountTableSource, StandingMounts, SweepContext, UNMOUNT_AWAITS_STEP_NINE, is_this_clones_own_mount, sweep_one_clone};
use crate::start_probe::first_attach_record::{FIRST_ATTACH_NAME, FirstAttachClaim, FirstAttachOutcome, FirstAttachPlace};
use crate::start_probe::probe_fingerprint::ProbeFingerprint;
use crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::{CloneFolderIdentity, LookAgainAtSlotRequest, MountIdentity, SlotIdentityStore, publish_parent_identity_under_root_control};
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_handler::sweep_attach_step::{AttachRequest, AttachStepOutcome, CloneAttachStep};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    DEVICES_PROPERTY, DMU_OBJECT_SET_FILESYSTEM, ENCRYPTION_PROPERTY, ENCRYPTION_ROOT_PROPERTY, KEY_STATUS_PROPERTY, KEY_STATUS_UNAVAILABLE, MOUNTPOINT_PROPERTY, SETUID_PROPERTY,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, ReadFailure};
use crate::worktree_clone_mount_handler::sweep_first_attach::FirstAttachRecorder;
use crate::worktree_clone_mount_protocol::{CloneMountReceipt, DESIRED_STATE_PROPERTY, DatasetGuid, MOUNT_WORD, ReceiptOutcome, ReceiptsFolder, clone_mount_receipt::ReceiptMountIdentity};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, DmuObjsetStats, NvList};
use std::cell::{Cell, RefCell};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// The variable the inner half of each control sets.
const INSIDE: &str = "CLI_GIT_ZFS_SWEEP_ONE_CLONE_ROOT";

/// The pinned clone parent.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// The checkout dataset a clone's origin is a snapshot of.
const CHECKOUT_DATASET: &str = "zpcachyos/nosnap/checkout";

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
    let origin: String = format!("{CHECKOUT_DATASET}@git-worktree-clone-origin-0011223344556677");
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
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        self.reads.set(self.reads.get() + 1);
        let replies = self.replies.borrow();
        let Some((_name, stats, properties)) = replies.iter().find(|(name, _, _)| return name == dataset) else {
            return Err(ReadFailure::at(format!("this scripted pool holds no `{dataset}`")));
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
        let _asked = (request.checkout, request.clone, request.store);
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
/// a checkout with two registered slots,
/// a root-owned store with the parent record,
/// a claims folder,
/// a receipts folder and this boot's first attach.
struct Fixture {
    /// The checkout.
    checkout: PathBuf,
    /// The store folder.
    store: PathBuf,
    /// The claims folder.
    claims: PathBuf,
    /// The receipts folder.
    receipts: PathBuf,
    /// This boot's first attach,
    /// recorded in the fixture rather than under `/run`.
    first_attach: RecordingFirstAttach,
    /// What the canonical slot already holds.
    standing: ScriptedStanding,
}

impl Fixture {
    /// Build the fixture under this run's own temporary root.
    fn build(name: &str) -> Self {
        let root: PathBuf = crate::test_support::fixture(format!("sweep-one-clone-{name}").as_str());
        let checkout: PathBuf = root.join("checkout");
        let parent: PathBuf = checkout.join(".claude").join("worktrees");
        for slot in ["one", "two"] {
            std::fs::create_dir_all(parent.join(slot)).expect("the slot folder is made");
            // Each slot is a Git worktree registration of this checkout, which is what the first observation checks.
            let pointer: String = format!("gitdir: {}\n", checkout.join(".git").join("worktrees").join(slot).display());
            std::fs::write(parent.join(slot).join(".git"), pointer).expect("the slot's Git pointer is written");
        }
        let store: PathBuf = root.join("store");
        std::fs::create_dir(store.as_path()).expect("the store folder is made");
        std::fs::set_permissions(store.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the store is root's and readable");
        let identity: CloneFolderIdentity = identity_of(parent.as_path());
        publish_parent_identity_under_root_control(store.as_path(), &identity, &identity).expect("the setup run's parent record is published");
        let claims: PathBuf = root.join("claims");
        let receipts: PathBuf = root.join("receipts");
        let first_attach: RecordingFirstAttach = RecordingFirstAttach::at(root.as_path());
        return Fixture { checkout, store, claims, receipts, first_attach, standing: ScriptedStanding::nothing() };
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
        checkout_dataset: CHECKOUT_DATASET,
        store: fixture.store.as_path(),
        claims,
        receipts,
        boot: BOOT,
        time: &|| return String::from(TIME),
        reader,
        attach,
        first_attach: &fixture.first_attach,
        standing: &fixture.standing,
    };
}

/// What:
///  This boot's first attach as a fixture records it,
///  so a control reads what the sweep wrote without `/run`.
/// Why:
///  The claim and the rename are the record's own module's controls,
///  and what the sweep proves here is when it claims and what it writes.
#[derive(Debug)]
struct RecordingFirstAttach {
    /// The place the record is written.
    place: FirstAttachPlace,
    /// The fingerprint of a machine no control runs on.
    fingerprint: ProbeFingerprint,
    /// What the sweep recorded,
    /// where it won the name.
    outcome: RefCell<Option<FirstAttachOutcome>>,
}

impl RecordingFirstAttach {
    /// The fixture's own place.
    fn at(root: &Path) -> Self {
        return RecordingFirstAttach {
            place: FirstAttachPlace::at(root.join("first-attach")),
            fingerprint: ProbeFingerprint {
                kernel_release: String::from("6.8.0-fixture"),
                module_version: Some(String::from("2.4.4-1")),
                module_srcversion: Some(String::from("ABCDEF")),
                module_build_id: Some(String::from("00112233")),
                boot_id: String::from(BOOT),
                handler: Some(String::from("/opt/handler")),
            },
            outcome: RefCell::new(None),
        };
    }

    /// The record the sweep wrote,
    /// where it wrote one.
    fn recorded(&self) -> Option<FirstAttachOutcome> {
        return self.outcome.borrow().clone();
    }

    /// Whether the name was ever claimed,
    /// which the file's own presence answers.
    fn claimed(&self) -> bool {
        return self.place.path().exists();
    }
}

impl FirstAttachRecorder for RecordingFirstAttach {
    fn claim(&self) -> Option<FirstAttachClaim> {
        return self.place.claim(&self.fingerprint, TIME).ok();
    }

    fn record(&self, _claim: &FirstAttachClaim, outcome: &FirstAttachOutcome) {
        self.outcome.replace(Some(outcome.clone()));
    }
}

/// What the canonical slot holds before an attach,
/// as a control scripts it.
#[derive(Debug)]
struct ScriptedStanding {
    /// The mount it answers,
    /// or nothing for a slot holding no mount of this clone.
    mount: RefCell<Option<MountIdentity>>,
}

impl ScriptedStanding {
    /// A slot holding no mount.
    fn nothing() -> Self {
        return ScriptedStanding { mount: RefCell::new(None) };
    }
}

impl StandingMounts for ScriptedStanding {
    fn standing_mount_of(&self, _request: &LookAgainAtSlotRequest<'_>, _clone: &str) -> Option<MountIdentity> {
        return self.mount.borrow().clone();
    }
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
    return receipts.read(guid).unwrap_or_default();
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
///  An unmount request is answered by the step that builds the unmount,
///  no receipt is filed for it,
///  and the durable intent is left as it is.
/// Why:
///  A refusal files no receipt but an unavailable key's
///  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
///  "Receipts for refusals"),
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
    assert!(receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).unwrap_or_default().is_none(), "a refusal files no receipt");
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
///  An attach that refused before any mount is answered as a refusal,
///  the sweep's line names the call that refused,
///  and no receipt is filed.
/// Why:
///  A refusal files no receipt but an unavailable key's,
///  so the answer is the sweep's line and the handler's journal,
///  and the durable intent stands for the next trigger.
#[test]
fn an_attach_that_refused_files_no_receipt() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::an_attach_that_refused_files_no_receipt") {
        return;
    }
    let fixture: Fixture = Fixture::build("attach-refused");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach {
        answer: AttachStepOutcome::Refused { step: String::from("move-mount"), errno: Some(String::from("EXDEV")), said: String::from("the walk refused the slot") },
        asked: Cell::new(0),
    };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == "move-mount"), "{outcome:?}");
    assert!(outcome.line().contains("EXDEV"), "the line keeps the errno: {}", outcome.line());
    assert!(receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).unwrap_or_default().is_none(), "a refusal files no receipt");
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
    let attach: ScriptedAttach = ScriptedAttach {
        answer: AttachStepOutcome::Refused { step: String::from("identity-compare"), errno: None, said: String::from("the walk refused the slot's identity") },
        asked: Cell::new(0),
    };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { .. }), "{outcome:?}");
    let after: CloneFolderIdentity = fixture.opened_store().slot("one").expect("the record still stands");
    assert_eq!(after, recorded, "the record is the one that stood before the sweep");
    assert_ne!(after, identity_of(slot.as_path()), "the record is not the live folder, so nothing refreshed it");
    assert!(!outcome.line().contains(IDENTITY_RECORD_STEP), "the sweep recorded nothing: {}", outcome.line());
}

/// What:
///  A clone whose own mount already stands at its slot is answered as
///  attached,
///  the attach step is not asked,
///  and the `done` receipt it filed keeps its nonce.
/// Why:
///  A clone whose intent is `mount` keeps that intent after it attached,
///  so the ordinary sweep is one over a mounted clone;
///  a sweep that walked its slot again would refuse it as a boundary the walk
///  may not cross and overwrite the `done` receipt with a `refused` one.
#[test]
fn a_standing_mount_of_this_clone_is_attached_and_keeps_its_done_receipt() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_standing_mount_of_this_clone_is_attached_and_keeps_its_done_receipt") {
        return;
    }
    let fixture: Fixture = Fixture::build("standing");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let first: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let attached: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &first, &receipts, &claims), CLONE);
    assert!(matches!(attached, CloneOutcome::Attached { mount_id: 41, .. }), "{attached:?}");
    let filed: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("a done receipt is filed");
    assert_eq!(filed.outcome, ReceiptOutcome::Done);

    // The second sweep finds this clone's own mount at the slot and must answer attached without walking the slot.
    fixture.standing.mount.replace(Some(mounted()));
    let second: ScriptedAttach = ScriptedAttach {
        answer: AttachStepOutcome::Refused { step: String::from("walk"), errno: None, said: String::from("the walk would refuse a mounted slot") },
        asked: Cell::new(0),
    };
    let again: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &second, &receipts, &claims), CLONE);
    assert!(matches!(again, CloneOutcome::Attached { mount_id: 41, ref slot, .. } if slot == "one"), "{again:?}");
    assert_eq!(second.asked.get(), 0, "a standing mount of this clone is never walked again");
    let kept: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("the done receipt stands");
    assert_eq!(kept.outcome, ReceiptOutcome::Done, "the done receipt is kept");
    assert_eq!(kept.nonce.as_str(), MOUNT_NONCE);
    assert_eq!(kept.mount.as_ref().map(|mount| return mount.mount_id), Some(41));
}

/// What:
///  A steady sweep of a standing clone whose `done` receipt is already the one
///  this request would file writes nothing.
/// Why:
///  The receipt's only changing field is its time,
///  and rewriting the same answer stages a file,
///  syncs it,
///  syncs its folder and renames it for every standing child,
///  which made a steady sweep cost grow faster than the child count.
#[test]
fn a_standing_mount_whose_done_receipt_is_unchanged_writes_nothing() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_standing_mount_whose_done_receipt_is_unchanged_writes_nothing") {
        return;
    }
    let fixture: Fixture = Fixture::build("standing-no-write");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let first: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(first, CloneOutcome::Attached { mount_id: 41, .. }), "{first:?}");
    let guid: DatasetGuid = DatasetGuid::parse(GUID.to_string().as_str()).expect("the GUID is canonical");
    let receipt_path: PathBuf = receipts.directory().join(receipts.entry_name(guid));
    let inode_before: u64 = std::fs::metadata(receipt_path.as_path()).expect("the receipt stands").ino();
    let text_before: String = std::fs::read_to_string(receipt_path.as_path()).expect("the receipt reads");

    fixture.standing.mount.replace(Some(mounted()));
    let second: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(second, CloneOutcome::Attached { mount_id: 41, .. }), "{second:?}");
    let inode_after: u64 = std::fs::metadata(receipt_path.as_path()).expect("the receipt still stands").ino();
    assert_eq!(inode_after, inode_before, "an unchanged done receipt is not staged and renamed again");
    assert_eq!(std::fs::read_to_string(receipt_path.as_path()).expect("the receipt still reads"), text_before, "the receipt's bytes are the ones already filed");
}

/// What:
///  An intent that changed under the claim attaches nothing on the first
///  reading,
///  and the holder reads it again after the claim is released.
/// Why:
///  The design says desired state and identities are rechecked under the claim,
///  and from plan step 14 on a standing `unmount` intent excludes new attaches;
///  a sweep acting on the reading it took before the claim could attach against
///  it.
///  The project also decided the holder reads the intent again after it releases
///  the claim,
///  so the changed reading is answered in the second turn rather than left to
///  the next trigger.
#[test]
fn a_changed_intent_is_read_again_after_the_claim_is_released() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_changed_intent_is_read_again_after_the_claim_is_released") {
        return;
    }
    /// A pool whose clone answers `first` on the first read and `then` after it.
    struct TwoReads {
        /// The first reply.
        first: NvList,
        /// Every later reply.
        then: NvList,
        /// How many reads were answered.
        reads: Cell<usize>,
    }

    impl DatasetReader for TwoReads {
        fn identity(&self, _dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
            let read: usize = self.reads.get();
            self.reads.set(read + 1);
            let properties: NvList = if read == 0 { self.first.clone() } else { self.then.clone() };
            return Ok(DatasetIdentity { stats: stats_of_a_clone(), properties });
        }

        fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
            return Ok(None);
        }
    }

    let fixture: Fixture = Fixture::build("intent-changed");
    let unmounting: String = format!("v1:{MOUNT_NONCE}:unmount");
    let reader: TwoReads = TwoReads {
        first: reply_of_a_clone(Some(request_nonce().as_str()), CLONE),
        then: reply_of_a_clone(Some(unmounting.as_str()), CLONE),
        reads: Cell::new(0),
    };
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(
        &SweepContext {
            parent_dataset: PARENT,
            checkout: fixture.checkout.as_path(),
            checkout_dataset: CHECKOUT_DATASET,
            store: fixture.store.as_path(),
            claims: &claims,
            receipts: &receipts,
            boot: BOOT,
            time: &|| return String::from(TIME),
            reader: &reader,
            attach: &attach,
            first_attach: &fixture.first_attach,
            standing: &fixture.standing,
        },
        CLONE,
    );
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == UNMOUNT_AWAITS_STEP_NINE), "the second turn answers the changed reading: {outcome:?}");
    assert_eq!(attach.asked.get(), 0, "an intent that changed under the claim attaches nothing");
    assert!(!fixture.first_attach.claimed(), "an intent that changed under the claim claims no first attach");
}

/// What:
///  A standing mount's turn releases its claim and still reads the intent
///  again,
///  so a request that arrives while the slot was claimed is acted on.
/// Why:
///  The holder reads the intent again after every release of a claim it took,
///  and the standing-mount answer is one of those releases;
///  a turn that returned there would leave the request to a trigger that may
///  not come.
#[test]
fn a_standing_mount_is_read_again_after_the_claim_is_released() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_standing_mount_is_read_again_after_the_claim_is_released") {
        return;
    }
    /// A pool that answers the mount request for the first two reads and an
    /// unmount request after them.
    struct Arriving {
        /// The mount request.
        mounting: NvList,
        /// The unmount request written during the turn.
        unmounting: NvList,
        /// How many reads were answered.
        reads: Cell<usize>,
    }

    impl DatasetReader for Arriving {
        fn identity(&self, _dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
            let read: usize = self.reads.get();
            self.reads.set(read + 1);
            let properties: NvList = if read < 2 { self.mounting.clone() } else { self.unmounting.clone() };
            return Ok(DatasetIdentity { stats: stats_of_a_clone(), properties });
        }

        fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
            return Ok(None);
        }
    }

    let fixture: Fixture = Fixture::build("standing-request-arrived");
    fixture.standing.mount.replace(Some(mounted()));
    let unmounting: String = format!("v1:{MOUNT_NONCE}:unmount");
    let reader: Arriving = Arriving {
        mounting: reply_of_a_clone(Some(request_nonce().as_str()), CLONE),
        unmounting: reply_of_a_clone(Some(unmounting.as_str()), CLONE),
        reads: Cell::new(0),
    };
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(
        &SweepContext {
            parent_dataset: PARENT,
            checkout: fixture.checkout.as_path(),
            checkout_dataset: CHECKOUT_DATASET,
            store: fixture.store.as_path(),
            claims: &claims,
            receipts: &receipts,
            boot: BOOT,
            time: &|| return String::from(TIME),
            reader: &reader,
            attach: &attach,
            first_attach: &fixture.first_attach,
            standing: &fixture.standing,
        },
        CLONE,
    );
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == UNMOUNT_AWAITS_STEP_NINE), "the second turn answers the request that arrived: {outcome:?}");
    assert_eq!(attach.asked.get(), 0, "the standing mount was recognised and the attach was never asked");
}

/// What:
///  A first turn that retained a mount keeps that failure when the intent
///  changes after the release.
/// Why:
///  A run whose only failure is replaced by the second turn's refusal counts no
///  failure,
///  and `boot-reconcile` would then publish `finished` over a mount that still
///  stands.
#[test]
fn a_retained_mount_is_kept_when_the_intent_changes_after_the_attach() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_retained_mount_is_kept_when_the_intent_changes_after_the_attach") {
        return;
    }
    /// A pool that answers the mount request for the first two reads and an
    /// unmount request after them.
    struct Arriving {
        /// The mount request.
        mounting: NvList,
        /// The unmount request written during the turn.
        unmounting: NvList,
        /// How many reads were answered.
        reads: Cell<usize>,
    }

    impl DatasetReader for Arriving {
        fn identity(&self, _dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
            let read: usize = self.reads.get();
            self.reads.set(read + 1);
            let properties: NvList = if read < 2 { self.mounting.clone() } else { self.unmounting.clone() };
            return Ok(DatasetIdentity { stats: stats_of_a_clone(), properties });
        }

        fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
            return Ok(None);
        }
    }

    let fixture: Fixture = Fixture::build("retained-request-arrived");
    let unmounting: String = format!("v1:{MOUNT_NONCE}:unmount");
    let reader: Arriving = Arriving {
        mounting: reply_of_a_clone(Some(request_nonce().as_str()), CLONE),
        unmounting: reply_of_a_clone(Some(unmounting.as_str()), CLONE),
        reads: Cell::new(0),
    };
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Retained { said: String::from("the mount 41 is still attached at /fixture/elsewhere") }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(
        &SweepContext {
            parent_dataset: PARENT,
            checkout: fixture.checkout.as_path(),
            checkout_dataset: CHECKOUT_DATASET,
            store: fixture.store.as_path(),
            claims: &claims,
            receipts: &receipts,
            boot: BOOT,
            time: &|| return String::from(TIME),
            reader: &reader,
            attach: &attach,
            first_attach: &fixture.first_attach,
            standing: &fixture.standing,
        },
        CLONE,
    );
    assert!(matches!(outcome, CloneOutcome::Failed { ref step, .. } if step == "rollback"), "the retained mount stays the turn's answer: {outcome:?}");
    assert_eq!(attach.asked.get(), 1, "the first turn attached once and retained the mount");
}

/// What:
///  An unavailable key keeps the `done` receipt of a clone whose own mount
///  stands at its slot.
/// Why:
///  The key question is not the answer to the request that mount already
///  answered,
///  and overwriting the `done` receipt with a `refused` one would make a waiting
///  caller read the later key state as its mount's answer.
#[test]
fn an_unavailable_key_keeps_the_done_receipt_of_a_standing_mount() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::an_unavailable_key_keeps_the_done_receipt_of_a_standing_mount") {
        return;
    }
    let fixture: Fixture = Fixture::build("key-kept-standing");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let attached: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(attached, CloneOutcome::Attached { mount_id: 41, .. }), "{attached:?}");

    fixture.standing.mount.replace(Some(mounted()));
    let mut reply: NvList = reply_of_a_clone(Some(request_nonce().as_str()), CLONE);
    reply.add_nvlist(ENCRYPTION_PROPERTY, pair_number(1, CLONE));
    reply.add_nvlist(KEY_STATUS_PROPERTY, pair_number(KEY_STATUS_UNAVAILABLE, CLONE));
    let refused_reader: ScriptedReader = ScriptedReader { children: vec![String::from(CLONE)], replies: RefCell::new(vec![(String::from(CLONE), stats_of_a_clone(), reply)]), reads: Cell::new(0) };
    let refused: CloneOutcome = sweep_one_clone(&context(&fixture, &refused_reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(refused, CloneOutcome::Refused { ref step, .. } if step == "key-unavailable"), "{refused:?}");
    let kept: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("the done receipt stands");
    assert_eq!(kept.outcome, ReceiptOutcome::Done, "the done receipt is kept");
    assert_eq!(kept.nonce.as_str(), MOUNT_NONCE);
    assert_eq!(kept.mount.as_ref().map(|mount| return mount.mount_id), Some(41));
}

/// What:
///  Only a mount of this clone's own dataset,
///  at root `/`,
///  as the ZFS type and with both enforced attributes,
///  is recognised as this clone's own mount.
/// Why:
///  A bind of the same dataset shows another root,
///  a mount another route made can lack `nosuid` or `nodev`,
///  and another type or source is another mount;
///  the standing-mount answer must not adopt any of them.
#[test]
fn only_this_clones_own_mount_carries_every_clause() {
    let own: MountFacts = MountFacts {
        mount_id: 41,
        parent_id: 40,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: PathBuf::from("/fixture/checkout/.claude/worktrees/one"),
        filesystem_type: String::from("zfs"),
        source: String::from(CLONE),
        mount_options: vec![String::from("rw"), String::from("nosuid"), String::from("nodev")],
        super_options: Vec::new(),
    };
    assert!(is_this_clones_own_mount(&own, CLONE), "the clone's own mount is recognised");
    let bind: MountFacts = MountFacts { root: PathBuf::from("/sub"), ..own.clone() };
    assert!(!is_this_clones_own_mount(&bind, CLONE), "a bind with another root is not this clone's own mount");
    let without_nosuid: MountFacts = MountFacts { mount_options: vec![String::from("rw"), String::from("nodev")], ..own.clone() };
    assert!(!is_this_clones_own_mount(&without_nosuid, CLONE), "a mount without nosuid is not this route's own");
    let without_nodev: MountFacts = MountFacts { mount_options: vec![String::from("rw"), String::from("nosuid")], ..own.clone() };
    assert!(!is_this_clones_own_mount(&without_nodev, CLONE), "a mount without nodev is not this route's own");
    let another_type: MountFacts = MountFacts { filesystem_type: String::from("tmpfs"), ..own.clone() };
    assert!(!is_this_clones_own_mount(&another_type, CLONE), "another filesystem type is not this clone's own mount");
    let another_source: MountFacts = MountFacts { source: String::from("zpcachyos/nosnap/clones/two"), ..own };
    assert!(!is_this_clones_own_mount(&another_source, CLONE), "another dataset's mount is not this clone's own mount");
}

/// What:
///  A request that arrives while the sweep holds the claim is acted on after
///  the claim's release,
///  in one second turn,
///  and the sweep does not loop.
/// Why:
///  The project decided the holder reads the intent again after it releases the
///  claim,
///  so a request made during a sweep is not left waiting for a trigger that may
///  never come
///  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
///  "A slot whose claim is held").
#[test]
fn a_request_that_arrives_while_the_claim_is_held_is_read_after_the_release() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_request_that_arrives_while_the_claim_is_held_is_read_after_the_release") {
        return;
    }
    /// A pool that answers a mount request for the first two reads and an
    /// unmount request after them,
    /// which is a request written while the sweep held the claim.
    struct Arriving {
        /// The mount request.
        mounting: NvList,
        /// The unmount request written during the turn.
        unmounting: NvList,
        /// How many reads were answered.
        reads: Cell<usize>,
    }

    impl DatasetReader for Arriving {
        fn identity(&self, _dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
            let read: usize = self.reads.get();
            self.reads.set(read + 1);
            let properties: NvList = if read < 2 { self.mounting.clone() } else { self.unmounting.clone() };
            return Ok(DatasetIdentity { stats: stats_of_a_clone(), properties });
        }

        fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
            return Ok(None);
        }
    }

    let fixture: Fixture = Fixture::build("request-arrived");
    let unmounting: String = format!("v1:{MOUNT_NONCE}:unmount");
    let reader: Arriving = Arriving {
        mounting: reply_of_a_clone(Some(request_nonce().as_str()), CLONE),
        unmounting: reply_of_a_clone(Some(unmounting.as_str()), CLONE),
        reads: Cell::new(0),
    };
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(
        &SweepContext {
            parent_dataset: PARENT,
            checkout: fixture.checkout.as_path(),
            checkout_dataset: CHECKOUT_DATASET,
            store: fixture.store.as_path(),
            claims: &claims,
            receipts: &receipts,
            boot: BOOT,
            time: &|| return String::from(TIME),
            reader: &reader,
            attach: &attach,
            first_attach: &fixture.first_attach,
            standing: &fixture.standing,
        },
        CLONE,
    );
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == UNMOUNT_AWAITS_STEP_NINE), "the second turn answers the request that arrived: {outcome:?}");
    assert_eq!(attach.asked.get(), 1, "the first turn attached once and the second turn attached nothing");
    assert_eq!(reader.reads.get(), 4, "the sweep reads the clone under the claim and once after the release, then turns once more");
    let filed: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("the turn that attached answered the nonce it mounted");
    assert_eq!(filed.outcome, ReceiptOutcome::Done, "the mount the first turn made is answered before the turn reads the intent again");
    assert_eq!(filed.nonce.as_str(), MOUNT_NONCE);
}

/// What:
///  An unloaded key is the one refusal that files a receipt,
///  and the receipt names that step.
/// Why:
///  It is the one refusal a waiting command can act on,
///  which is what the project's receipt rule keeps
///  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
///  "Receipts for refusals").
#[test]
fn an_unloaded_key_files_a_refused_receipt_naming_it() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::an_unloaded_key_files_a_refused_receipt_naming_it") {
        return;
    }
    let fixture: Fixture = Fixture::build("key-refused");
    let mut reply: NvList = reply_of_a_clone(Some(request_nonce().as_str()), CLONE);
    reply.add_nvlist(ENCRYPTION_PROPERTY, pair_number(1, CLONE));
    reply.add_nvlist(KEY_STATUS_PROPERTY, pair_number(KEY_STATUS_UNAVAILABLE, CLONE));
    let reader: ScriptedReader = ScriptedReader { children: vec![String::from(CLONE)], replies: RefCell::new(vec![(String::from(CLONE), stats_of_a_clone(), reply)]), reads: Cell::new(0) };
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == "key-unavailable"), "{outcome:?}");
    assert_eq!(attach.asked.get(), 0);
    let receipt: CloneMountReceipt = receipts.read(DatasetGuid::parse(GUID.to_string().as_str()).expect("canonical")).expect("the folder reads").expect("an unloaded key is answered");
    assert_eq!(receipt.outcome, ReceiptOutcome::Refused);
    assert_eq!(receipt.failed_step.as_deref(), Some("key-unavailable"));
    assert_eq!(receipt.nonce.as_str(), MOUNT_NONCE);
}

/// What:
///  The first observation refuses a slot whose `.git` is not this checkout's
///  registration,
///  before any attach,
///  and records nothing.
/// Why:
///  The design requires the root-recorded underlay to hold only the expected
///  Git pointer,
///  so a slot an account emptied or replaced is no slot this route mounts.
#[test]
fn a_slot_without_this_checkouts_git_pointer_is_refused_before_the_attach() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_slot_without_this_checkouts_git_pointer_is_refused_before_the_attach") {
        return;
    }
    let fixture: Fixture = Fixture::build("no-pointer");
    let pointer: PathBuf = fixture.checkout.join(".claude").join("worktrees").join("one").join(".git");
    std::fs::remove_file(pointer.as_path()).expect("the fixture's pointer is removed");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == IDENTITY_RECORD_STEP), "{outcome:?}");
    assert_eq!(attach.asked.get(), 0, "a slot with no Git pointer is not attached");
    assert!(matches!(fixture.opened_store().slot("one"), Err(crate::worktree_clone_mount::StoreRefusal::NoSlotRecord { .. })), "nothing was recorded");
    assert!(matches!(fixture.first_attach.recorded(), Some(FirstAttachOutcome::Refused { ref step, .. }) if step == IDENTITY_RECORD_STEP), "the first attach records the refusing step");
}

/// What:
///  This boot's first attach is claimed by the turn that passes the key check
///  and writes the mount,
///  and a later turn claims nothing and records nothing.
/// Why:
///  The design writes the first real attach after boot and before the receipt,
///  and a second sweep of this boot must not replace that boot's first attempt
///  (`doc/planning/cli-git-zfs-start-probe-design.md`,
///  "The first real attach's record").
#[test]
fn the_first_attach_is_claimed_once_and_records_the_mount() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::the_first_attach_is_claimed_once_and_records_the_mount") {
        return;
    }
    let fixture: Fixture = Fixture::build("first-attach");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &attach, &receipts, &claims), CLONE);
    assert!(matches!(outcome, CloneOutcome::Attached { .. }), "{outcome:?}");
    let recorded: FirstAttachOutcome = fixture.first_attach.recorded().expect("the first attach is recorded");
    let FirstAttachOutcome::Attached { mount_id, flags, slot, clone, guid } = recorded.clone() else {
        panic!("the first attach records the mount: {recorded:?}");
    };
    assert_eq!(mount_id, 41);
    assert_eq!(slot, "one");
    assert_eq!(clone, CLONE);
    assert_eq!(guid, GUID.to_string());
    assert!(!flags.is_empty(), "the flags are read back from the table: {flags}");
    let path: PathBuf = fixture.checkout.parent().expect("the fixture has a root").join("first-attach").join(FIRST_ATTACH_NAME);
    assert!(path.exists(), "the record is a file in the fixture's place");

    fixture.standing.mount.replace(Some(mounted()));
    let second: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let _again: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &second, &receipts, &claims), CLONE);
    assert_eq!(fixture.first_attach.recorded(), Some(recorded), "a later turn records nothing over this boot's first attach");
}


//region The mount table's one read and its poll

/// What:
///  A table source a control owns:
///  the tables it answers in order,
///  whether the poll says the table changed,
///  and how many reads it was asked for.
/// Why:
///  The one-read property and the refresh after a change have to be provable
///  without a mount on the host,
///  so the source is the seam and the answer is a table the control wrote.
#[derive(Clone)]
struct ScriptedTable {
    /// The tables in order; the last one answers every later read.
    tables: std::rc::Rc<RefCell<Vec<Vec<MountFacts>>>>,
    /// Whether the poll says the table changed.
    changed: std::rc::Rc<Cell<bool>>,
    /// How many reads were asked for.
    reads: std::rc::Rc<Cell<usize>>,
}

impl ScriptedTable {
    /// A source holding `tables`, with the poll quiet.
    fn new(tables: Vec<Vec<MountFacts>>) -> Self {
        return Self { tables: std::rc::Rc::new(RefCell::new(tables)), changed: std::rc::Rc::new(Cell::new(false)), reads: std::rc::Rc::new(Cell::new(0)) };
    }

    /// Mark that one change happened since the last poll.
    fn change(&self) {
        self.changed.set(true);
    }
}

impl MountTableSource for ScriptedTable {
    fn changed(&self) -> bool {
        if self.changed.get() {
            self.changed.set(false);
            return true;
        }
        return false;
    }

    fn read(&self) -> Result<Vec<MountFacts>, String> {
        self.reads.set(self.reads.get() + 1);
        let mut tables = self.tables.borrow_mut();
        if tables.len() > 1 {
            return Ok(tables.remove(0));
        }
        return Ok(tables.first().cloned().unwrap_or_default());
    }
}

/// One table mount the controls build.
fn table_mount(id: u64, point: &str, source: &str) -> MountFacts {
    return MountFacts {
        mount_id: id,
        parent_id: 40,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: PathBuf::from(point),
        filesystem_type: String::from("zfs"),
        source: String::from(source),
        mount_options: vec![String::from("rw"), String::from("nosuid"), String::from("nodev")],
        super_options: Vec::new(),
    };
}

/// What:
///  A table read once answers many standing children,
///  and one change makes it read again.
/// Why:
///  The cost win is the one read per sweep;
///  a later edit that reads the table per child would pass every other control
///  and give the cost back unseen.
#[test]
fn a_standing_sweep_reads_the_mount_table_once_and_again_after_one_change() {
    let source: ScriptedTable = ScriptedTable::new(vec![vec![table_mount(7_000_001, "/fixture/first", "pool/one")], vec![table_mount(7_000_001, "/fixture/second", "pool/two")]]);
    let control: ScriptedTable = source.clone();
    let standing: super::LiveStandingMounts = super::LiveStandingMounts::from_source(Box::new(source));
    assert_eq!(standing.table_reads(), 1, "the first read is the sweep's own");
    for _child in 0..64 {
        let facts: MountFacts = standing.facts_of(7_000_001).expect("the table holds this mount");
        assert_eq!(facts.mount_point, PathBuf::from("/fixture/first"), "the table answers each child");
    }
    assert_eq!(standing.table_reads(), 1, "64 standing children cost one table read");
    control.change();
    let after: MountFacts = standing.facts_of(7_000_001).expect("the changed table still holds the id");
    assert_eq!(after.mount_point, PathBuf::from("/fixture/second"), "one change makes the sweep read the table again");
    assert_eq!(standing.table_reads(), 2, "the change costs exactly one more read");
}

/// What:
///  A mount id the table does not hold is read from the live table.
/// Why:
///  A mount made after the sweep's read must not be refused because the old
///  table never saw it.
#[test]
fn a_mount_the_table_does_not_hold_is_read_live() {
    let source: ScriptedTable = ScriptedTable::new(vec![Vec::new()]);
    let standing: super::LiveStandingMounts = super::LiveStandingMounts::from_source(Box::new(source));
    let root: std::fs::File = std::fs::File::open("/").expect("the root opens");
    let live_id: u64 = crate::fs_beneath_mount_facts::mount_id_of(&root).expect("the root's mount number reads");
    let facts: MountFacts = standing.facts_of(live_id).expect("an id the table lacks is read live");
    assert_eq!(facts.mount_point, PathBuf::from("/"), "the live table answered for the root");
    assert_eq!(standing.table_reads(), 1, "a live fallback is not a table read");
}

/// What:
///  A mount replaced under the table's mount id is answered from the table only
///  until the poll fires,
///  and from the new mount after it.
/// Why:
///  The kernel gives a freed mount number to the next mount,
///  so a table keyed by that number can name the mount that held it before;
///  the poll is what makes the replacement visible.
#[test]
fn a_mount_replaced_under_a_reused_id_is_read_again_after_the_poll() {
    let source: ScriptedTable = ScriptedTable::new(vec![vec![table_mount(7_000_002, "/fixture/one", "pool/one")], vec![table_mount(7_000_002, "/fixture/one", "pool/bystander")]]);
    let control: ScriptedTable = source.clone();
    let standing: super::LiveStandingMounts = super::LiveStandingMounts::from_source(Box::new(source));
    let before: MountFacts = standing.facts_of(7_000_002).expect("the first table holds the id");
    assert!(is_this_clones_own_mount(&before, "pool/one"), "the first table names the clone's own mount");
    control.change();
    let after: MountFacts = standing.facts_of(7_000_002).expect("the second table holds the same id");
    assert_eq!(after.source, "pool/bystander", "the replacement under the same id is read after the poll");
    assert!(!is_this_clones_own_mount(&after, "pool/one"), "the replacement is not the clone's own mount");
    assert_eq!(standing.table_reads(), 2, "the replacement costs one re-read");
}

/// One tmpfs mount a control makes,
/// taken down however the control ends.
struct PolledMount {
    /// The mount point.
    path: PathBuf,
}

impl Drop for PolledMount {
    fn drop(&mut self) {
        let target: std::ffi::CString = std::ffi::CString::new(self.path.as_os_str().as_bytes()).expect("no NUL in the path");
        // SAFETY: `target` is a NUL-terminated path that outlives the call, and no flag is passed.
        let _taken: libc::c_int = unsafe { libc::umount2(target.as_ptr(), 0) };
    }
}

/// Mount one tmpfs at `path` with `flags`.
fn mount_tmpfs(source: &str, path: &Path, flags: libc::c_ulong) {
    let source_name: std::ffi::CString = std::ffi::CString::new(source).expect("no NUL in the source");
    let target: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in the path");
    let kind: std::ffi::CString = std::ffi::CString::new("tmpfs").expect("no NUL in the type");
    // SAFETY: both strings outlive the call, the type is a NUL-terminated name, and no data pointer is read.
    let mounted: libc::c_int = unsafe { libc::mount(source_name.as_ptr(), target.as_ptr(), kind.as_ptr(), flags, std::ptr::null()) };
    assert_eq!(mounted, 0, "the control's tmpfs mounts: {}", std::io::Error::last_os_error());
}

/// Remount one tmpfs in place with `flags`.
fn remount_tmpfs(path: &Path, flags: libc::c_ulong) {
    let target: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("no NUL in the path");
    // SAFETY: the target is a NUL-terminated path that outlives the call, and `MS_REMOUNT|MS_BIND` with no type or data
    // changes the per-mount flags in place.
    let changed: libc::c_int = unsafe { libc::mount(std::ptr::null(), target.as_ptr(), std::ptr::null(), libc::MS_REMOUNT | libc::MS_BIND | flags, std::ptr::null()) };
    assert_eq!(changed, 0, "the control's tmpfs remounts in place: {}", std::io::Error::last_os_error());
}

/// One table source that reads the live table and shapes the control's tmpfs as
/// the clone's own mount while it carries both enforced flags.
struct CloneShapedTable {
    /// The live source and its poll.
    live: LiveMountTableSource,
}

impl MountTableSource for CloneShapedTable {
    fn changed(&self) -> bool {
        return self.live.changed();
    }

    fn read(&self) -> Result<Vec<MountFacts>, String> {
        let mut facts: Vec<MountFacts> = self.live.read()?;
        for mount in &mut facts {
            if mount.source == "zfs-step-8-poll-clone" && mount.has_option("nosuid") && mount.has_option("nodev") {
                mount.filesystem_type = String::from("zfs");
                mount.source = String::from(CLONE);
                mount.root = PathBuf::from("/");
            }
        }
        return Ok(facts);
    }
}

/// What:
///  A mount whose enforced flags are removed in place while a sweep runs is
///  no longer recognised after the poll.
/// Why:
///  The pass showed `main` refusing a clone whose `nosuid` and `nodev` were
///  cleared mid-run while the snapshot build answered `attached`;
///  the in-place remount changes the mount's number not at all,
///  so only the poll can make the table read again.
#[test]
fn a_mount_whose_flags_change_in_place_is_read_again_after_the_poll() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_mount_whose_flags_change_in_place_is_read_again_after_the_poll") {
        return;
    }
    let root: PathBuf = crate::test_support::fixture("standing-poll-flags");
    let slot: PathBuf = root.join("checkout").join(".claude").join("worktrees").join("one");
    std::fs::create_dir_all(slot.as_path()).expect("the slot folder is made");
    mount_tmpfs("zfs-step-8-poll-clone", slot.as_path(), libc::MS_NOSUID | libc::MS_NODEV);
    let _guard: PolledMount = PolledMount { path: slot.clone() };
    let source: CloneShapedTable = CloneShapedTable { live: LiveMountTableSource::open() };
    let standing: super::LiveStandingMounts = super::LiveStandingMounts::from_source(Box::new(source));
    let checkout: PathBuf = root.join("checkout");
    let request: LookAgainAtSlotRequest<'_> = LookAgainAtSlotRequest { checkout: checkout.as_path(), slot: "one" };
    assert!(standing.standing_mount_of(&request, CLONE).is_some(), "the clone's own mount is recognised first");
    remount_tmpfs(slot.as_path(), 0);
    assert!(standing.standing_mount_of(&request, CLONE).is_none(), "the flags removed in place are seen after the poll");
    remount_tmpfs(slot.as_path(), libc::MS_NOSUID | libc::MS_NODEV);
    assert!(standing.standing_mount_of(&request, CLONE).is_some(), "the flags restored in place are seen after the poll");
}

//endregion


//region The receipt skip and the two remaining releases

/// A pool that answers one clone from a script of replies,
/// a `None` being a failed read,
/// then the last reply for ever.
struct Sequenced {
    /// The replies in order.
    script: Vec<Option<NvList>>,
    /// The reply after the script.
    last: NvList,
    /// How many reads were answered.
    reads: Cell<usize>,
}

impl DatasetReader for Sequenced {
    fn identity(&self, _dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        let at: usize = self.reads.get();
        self.reads.set(at + 1);
        return match self.script.get(at) {
            Some(Some(reply)) => Ok(DatasetIdentity { stats: stats_of_a_clone(), properties: reply.clone() }),
            Some(None) => Err(ReadFailure::at(String::from("the scripted pool fails this read"))),
            None => Ok(DatasetIdentity { stats: stats_of_a_clone(), properties: self.last.clone() }),
        };
    }

    fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// The clone's receipt GUID.
fn receipt_guid() -> DatasetGuid {
    return DatasetGuid::parse(GUID.to_string().as_str()).expect("the GUID is canonical");
}

/// Attach once,
/// leave the mount standing,
/// and answer a second turn with a scripted attach that must never be asked.
fn attached_once(fixture: &Fixture, receipts: &ReceiptsFolder, claims: &SlotClaimDirectory) -> ScriptedAttach {
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let first: CloneOutcome = sweep_one_clone(&context(fixture, &reader, &attach, receipts, claims), CLONE);
    assert!(matches!(first, CloneOutcome::Attached { mount_id: 41, .. }), "{first:?}");
    fixture.standing.mount.replace(Some(mounted()));
    return ScriptedAttach { answer: AttachStepOutcome::Refused { step: String::from("walk"), errno: None, said: String::from("a standing mount is never walked") }, asked: Cell::new(0) };
}

/// What:
///  A new request on a standing mount files its own receipt.
/// Why:
///  The all-equal control cannot tell a comparison that ignores the nonce,
///  and a waiting command whose request carries a new nonce would never get its
///  answer.
#[test]
fn a_standing_mount_answers_a_new_nonce_with_a_new_receipt() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_standing_mount_answers_a_new_nonce_with_a_new_receipt") {
        return;
    }
    let fixture: Fixture = Fixture::build("new-nonce-receipt");
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let second: ScriptedAttach = attached_once(&fixture, &receipts, &claims);
    let second_digits: &str = "ffeeddccbbaa99887766554433221100";
    let reader: ScriptedReader = reader_with(Some(mount_request(second_digits).as_str()), CLONE);
    let again: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &second, &receipts, &claims), CLONE);
    assert!(matches!(again, CloneOutcome::Attached { mount_id: 41, .. }), "{again:?}");
    let kept: CloneMountReceipt = receipts.read(receipt_guid()).expect("the folder reads").expect("a receipt stands");
    assert_eq!(kept.nonce.as_str(), second_digits, "a new request on a standing mount gets its own receipt");
}

/// What:
///  A receipt replaced by a refused one is filed again as the done answer.
/// Why:
///  A comparison that ignores the outcome would leave a caller reading
///  `refused` for a mount that stands.
#[test]
fn a_receipt_replaced_by_a_refused_one_is_filed_again() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_receipt_replaced_by_a_refused_one_is_filed_again") {
        return;
    }
    let fixture: Fixture = Fixture::build("refused-over-done");
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let second: ScriptedAttach = attached_once(&fixture, &receipts, &claims);
    let mut tampered: CloneMountReceipt = receipts.read(receipt_guid()).expect("the folder reads").expect("the receipt stands");
    tampered.outcome = ReceiptOutcome::Refused;
    tampered.mount = None;
    tampered.failed_step = Some(String::from("key-unavailable"));
    receipts.publish(&tampered).expect("the tampered receipt is filed");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let again: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &second, &receipts, &claims), CLONE);
    assert!(matches!(again, CloneOutcome::Attached { mount_id: 41, .. }), "{again:?}");
    let kept: CloneMountReceipt = receipts.read(receipt_guid()).expect("the folder reads").expect("the receipt stands");
    assert_eq!(kept.outcome, ReceiptOutcome::Done, "the done answer is filed again over a refused one");
}

/// What:
///  A receipt naming another mount is filed again with the mount that stands.
/// Why:
///  A comparison that ignores the mount would leave a caller confirming a
///  mount the slot no longer holds.
#[test]
fn a_receipt_naming_another_mount_is_filed_again() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_receipt_naming_another_mount_is_filed_again") {
        return;
    }
    let fixture: Fixture = Fixture::build("other-mount-receipt");
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let second: ScriptedAttach = attached_once(&fixture, &receipts, &claims);
    let mut tampered: CloneMountReceipt = receipts.read(receipt_guid()).expect("the folder reads").expect("the receipt stands");
    tampered.mount.as_mut().expect("a done receipt names its mount").mount_id = 99;
    receipts.publish(&tampered).expect("the tampered receipt is filed");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let again: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &second, &receipts, &claims), CLONE);
    assert!(matches!(again, CloneOutcome::Attached { mount_id: 41, .. }), "{again:?}");
    let kept: CloneMountReceipt = receipts.read(receipt_guid()).expect("the folder reads").expect("the receipt stands");
    assert_eq!(kept.mount.as_ref().map(|mount| return mount.mount_id), Some(41), "the receipt names the mount that stands now");
}

/// What:
///  A receipt truncated by hand is filed again.
/// Why:
///  A receipt the route cannot parse cannot be compared with the answer,
///  so the sweep must write a whole one.
#[test]
fn a_truncated_receipt_is_filed_again() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_truncated_receipt_is_filed_again") {
        return;
    }
    let fixture: Fixture = Fixture::build("truncated-receipt");
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let second: ScriptedAttach = attached_once(&fixture, &receipts, &claims);
    let path: PathBuf = receipts.directory().join(receipts.entry_name(receipt_guid()));
    let text: String = std::fs::read_to_string(path.as_path()).expect("the receipt reads");
    std::fs::write(path.as_path(), &text[..text.len() / 2]).expect("the receipt is truncated by hand");
    let reader: ScriptedReader = reader_with(Some(request_nonce().as_str()), CLONE);
    let again: CloneOutcome = sweep_one_clone(&context(&fixture, &reader, &second, &receipts, &claims), CLONE);
    assert!(matches!(again, CloneOutcome::Attached { mount_id: 41, .. }), "{again:?}");
    let kept: CloneMountReceipt = receipts.read(receipt_guid()).expect("the receipt parses again").expect("the receipt stands");
    assert_eq!(kept.outcome, ReceiptOutcome::Done);
}

/// What:
///  A re-read refused under the claim still gets the turn after the release.
/// Why:
///  The holder reads the intent again after every release of a claim it took,
///  and a refused re-read is one of the five releases.
#[test]
fn a_refused_reread_under_the_claim_is_read_again_after_the_release() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::a_refused_reread_under_the_claim_is_read_again_after_the_release") {
        return;
    }
    let fixture: Fixture = Fixture::build("refused-reread-release");
    let unmounting: String = format!("v1:{MOUNT_NONCE}:unmount");
    let reader: Sequenced = Sequenced {
        script: vec![Some(reply_of_a_clone(Some(request_nonce().as_str()), CLONE)), None],
        last: reply_of_a_clone(Some(unmounting.as_str()), CLONE),
        reads: Cell::new(0),
    };
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(
        &SweepContext {
            parent_dataset: PARENT,
            checkout: fixture.checkout.as_path(),
            checkout_dataset: CHECKOUT_DATASET,
            store: fixture.store.as_path(),
            claims: &claims,
            receipts: &receipts,
            boot: BOOT,
            time: &|| return String::from(TIME),
            reader: &reader,
            attach: &attach,
            first_attach: &fixture.first_attach,
            standing: &fixture.standing,
        },
        CLONE,
    );
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == UNMOUNT_AWAITS_STEP_NINE), "the second turn answers the request that arrived: {outcome:?}");
}

/// What:
///  A refusal of the first observation's record still gets the turn after the
///  release.
/// Why:
///  The holder reads the intent again after every release of a claim it took,
///  and the identity-record refusal is one of the five releases.
#[test]
fn an_identity_record_refusal_is_read_again_after_the_release() {
    if !in_a_root_namespace(INSIDE, "worktree_clone_mount_handler::sweep_one_clone::tests::an_identity_record_refusal_is_read_again_after_the_release") {
        return;
    }
    let fixture: Fixture = Fixture::build("identity-refusal-release");
    let pointer: PathBuf = fixture.checkout.join(".claude").join("worktrees").join("one").join(".git");
    std::fs::remove_file(pointer.as_path()).expect("the fixture's pointer is removed");
    let unmounting: String = format!("v1:{MOUNT_NONCE}:unmount");
    let reader: Sequenced = Sequenced {
        script: vec![Some(reply_of_a_clone(Some(request_nonce().as_str()), CLONE)), Some(reply_of_a_clone(Some(request_nonce().as_str()), CLONE))],
        last: reply_of_a_clone(Some(unmounting.as_str()), CLONE),
        reads: Cell::new(0),
    };
    let attach: ScriptedAttach = ScriptedAttach { answer: AttachStepOutcome::Attached { mount: mounted() }, asked: Cell::new(0) };
    let receipts: ReceiptsFolder = fixture.opened_receipts();
    let claims: SlotClaimDirectory = fixture.opened_claims();
    let outcome: CloneOutcome = sweep_one_clone(
        &SweepContext {
            parent_dataset: PARENT,
            checkout: fixture.checkout.as_path(),
            checkout_dataset: CHECKOUT_DATASET,
            store: fixture.store.as_path(),
            claims: &claims,
            receipts: &receipts,
            boot: BOOT,
            time: &|| return String::from(TIME),
            reader: &reader,
            attach: &attach,
            first_attach: &fixture.first_attach,
            standing: &fixture.standing,
        },
        CLONE,
    );
    assert!(matches!(outcome, CloneOutcome::Refused { ref step, .. } if step == UNMOUNT_AWAITS_STEP_NINE), "the second turn answers the request that arrived: {outcome:?}");
}

//endregion
