//! What:
//!  Controls of the checks one child of the pinned parent passes.
//! Why:
//!  Every refusal here is a clone the sweep must not attach,
//!  and the key's refusal must be told apart from the identity ones,
//!  so each cause has its own control and its own step word.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(verify(facts, PARENT).slot).toBe('one');
//! ```

use super::{CloneRefusal, LEGACY_MOUNTPOINT, VerifiedClone, slot_of, verify};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts, DEVICES_PROPERTY, DMU_OBJECT_SET_FILESYSTEM, ENCRYPTION_PROPERTY, ENCRYPTION_ROOT_PROPERTY, KEY_STATUS_PROPERTY, PropertyValue, SETUID_PROPERTY,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipt::MAXIMUM_WORD_BYTES;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// The pinned clone parent.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// A clone one component below it.
const CLONE: &str = "zpcachyos/nosnap/clones/one";

/// The checkout dataset the add route snapshots.
const CHECKOUT: &str = "zpcachyos/nosnap/checkout";

/// The clone's own GUID.
const GUID: u64 = 12_345_678_901_234_567_890;

/// The encryption root the controls name.
const ENCRYPTION_ROOT: &str = "zpcachyos";

/// What:
///  The facts of a managed clone,
///  which a control then breaks one field at a time.
/// Why:
///   One baseline is what proves each refusal came from the field the control
///  changed rather than from another check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = managedClone(); facts.setuid = { kind: 'number', number: 1 };
/// ```
fn managed_clone() -> CloneObjectFacts {
    return CloneObjectFacts {
        name: String::from(CLONE),
        guid: GUID,
        object_type: DMU_OBJECT_SET_FILESYSTEM,
        is_snapshot: false,
        origin: format!("{CHECKOUT}@git-worktree-clone-origin-0011223344556677"),
        mountpoint: PropertyValue::Text(String::from(LEGACY_MOUNTPOINT)),
        setuid: PropertyValue::Number(0),
        devices: PropertyValue::Number(0),
        encryption: PropertyValue::Number(0),
        encryption_root: PropertyValue::Text(String::from(ENCRYPTION_ROOT)),
        key_status: PropertyValue::Number(0),
        properties: NvList::new(),
    };
}

/// The same facts with a key that is loaded.
fn encrypted_clone(key_status: u64) -> CloneObjectFacts {
    let mut facts: CloneObjectFacts = managed_clone();
    facts.encryption = PropertyValue::Number(7);
    facts.key_status = PropertyValue::Number(key_status);
    return facts;
}

/// A managed clone passes every check and gives the slot its name carries.
#[test]
fn a_managed_clone_passes_every_check() {
    let verified: VerifiedClone = verify(&managed_clone(), PARENT, CHECKOUT).expect("a managed clone passes");
    assert_eq!(verified.slot, "one");
    assert_eq!(verified.dataset_name, CLONE);
    assert_eq!(verified.dataset.file_name(), GUID.to_string());
}

/// A name that is not one component below the parent derives no slot.
#[test]
fn a_name_that_is_not_one_child_is_refused() {
    let mut nested: CloneObjectFacts = managed_clone();
    nested.name = format!("{CLONE}/below");
    assert!(matches!(verify(&nested, PARENT, CHECKOUT), Err(CloneRefusal::NotAChild { .. })), "{:?}", verify(&nested, PARENT, CHECKOUT));
    let mut elsewhere: CloneObjectFacts = managed_clone();
    elsewhere.name = String::from("zpcachyos/nosnap/other/one");
    assert!(matches!(verify(&elsewhere, PARENT, CHECKOUT), Err(CloneRefusal::NotAChild { .. })), "{:?}", verify(&elsewhere, PARENT, CHECKOUT));
}

/// The slot's own grammar refuses a component a person could not have added
/// as a worktree this route names.
#[test]
fn a_slot_the_grammar_refuses_is_refused() {
    for slot in ["One", "one_two", "one.two", "one two", "-one", "one-"] {
        let mut facts: CloneObjectFacts = managed_clone();
        facts.name = format!("{PARENT}/{slot}");
        let refused = verify(&facts, PARENT, CHECKOUT).expect_err("the slot grammar refuses this name");
        assert!(matches!(refused, CloneRefusal::SlotName { .. }), "{slot}: {refused:?}");
        assert_eq!(refused.step(), "clone-name");
        assert!(refused.message().contains(slot), "{slot}: the refusal names the component: {}", refused.message());
    }
}

/// A slot longer than a record's name is refused before any folder is opened.
#[test]
fn a_slot_longer_than_a_record_name_is_refused() {
    let slot: String = "a".repeat(256);
    let refused = slot_of(format!("{PARENT}/{slot}").as_str(), PARENT).expect_err("a slot name is bounded");
    assert!(matches!(refused, CloneRefusal::SlotName { .. }), "{refused:?}");
}

/// A dataset name longer than the kernel takes is refused.
#[test]
fn a_dataset_name_over_the_kernel_bound_is_refused() {
    let slot: String = "a".repeat(240);
    let refused = slot_of(format!("{PARENT}/{slot}").as_str(), PARENT).expect_err("the whole dataset name is bounded");
    assert!(matches!(refused, CloneRefusal::SlotName { .. }), "{refused:?}");
}

/// A volume and a snapshot and a dataset with no origin are each refused.
#[test]
fn the_object_type_the_snapshot_flag_and_the_origin_are_each_checked() {
    let mut volume: CloneObjectFacts = managed_clone();
    volume.object_type = DMU_OBJECT_SET_FILESYSTEM + 1;
    assert!(matches!(verify(&volume, PARENT, CHECKOUT), Err(CloneRefusal::NotAFilesystem { .. })));

    let mut snapshot: CloneObjectFacts = managed_clone();
    snapshot.is_snapshot = true;
    assert!(matches!(verify(&snapshot, PARENT, CHECKOUT), Err(CloneRefusal::Snapshot { .. })));

    let mut plain: CloneObjectFacts = managed_clone();
    plain.origin = String::new();
    let refused = verify(&plain, PARENT, CHECKOUT).expect_err("a dataset with no origin is not a clone");
    assert!(matches!(refused, CloneRefusal::NoOrigin { .. }));
    assert_eq!(refused.step(), "clone-origin");

    let mut another: CloneObjectFacts = managed_clone();
    another.origin = String::from("zpcachyos/nosnap/other@origin");
    let outside = verify(&another, PARENT, CHECKOUT).expect_err("a clone of another dataset is not this sweep's");
    assert!(matches!(outside, CloneRefusal::OriginOutsideTheCheckout { .. }), "{outside:?}");
    assert!(outside.message().contains("zpcachyos/nosnap/other@origin"), "the refusal names the origin: {}", outside.message());
    assert!(outside.message().contains(CHECKOUT), "the refusal names the checkout dataset: {}", outside.message());
}

/// What:
///  A clone of the container's own snapshot is refused,
///  and a checkout snapshot that is not the add route's is refused too.
/// Why:
///  H1 of step 8's integration pass proved the sweep accepted only a snapshot of
///  the container,
///  which is the one shape the design's add route never makes:
///  it snapshots the checkout and clones that snapshot,
///  and the snapshot's own name is the prefix the design gives
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Dataset and path layout",
///  item 3 of "Requests,
/// results and ordering").
#[test]
fn an_origin_is_the_checkouts_own_clone_origin_snapshot_and_nothing_else() {
    let managed: CloneObjectFacts = managed_clone();
    verify(&managed, PARENT, CHECKOUT).expect("the design's add route origin is accepted");

    let mut container_snapshot: CloneObjectFacts = managed_clone();
    container_snapshot.origin = format!("{PARENT}@origin");
    let container_refusal = verify(&container_snapshot, PARENT, CHECKOUT).expect_err("a snapshot of the container is not the add route's origin");
    assert!(matches!(container_refusal, CloneRefusal::OriginOutsideTheCheckout { .. }), "{container_refusal:?}");

    let mut another_snapshot: CloneObjectFacts = managed_clone();
    another_snapshot.origin = format!("{CHECKOUT}@sanoid-2026-11-22");
    let snapshot_refusal = verify(&another_snapshot, PARENT, CHECKOUT).expect_err("a checkout snapshot the add route did not make is refused");
    assert!(matches!(snapshot_refusal, CloneRefusal::OriginIsNotACloneOrigin { prefix: "git-worktree-clone-origin-", .. }), "{snapshot_refusal:?}");
    assert_eq!(snapshot_refusal.step(), "clone-origin");
}

/// A zero GUID is refused,
/// because a receipt is filed under the GUID and no file may be named zero.
#[test]
fn a_zero_guid_is_refused() {
    let mut facts: CloneObjectFacts = managed_clone();
    facts.guid = 0;
    let refused = verify(&facts, PARENT, CHECKOUT).expect_err("a zero GUID files no receipt");
    assert!(matches!(refused, CloneRefusal::NoGuid { .. }));
    assert_eq!(refused.step(), "clone-guid");
}

/// A mountpoint that is not `legacy`,
/// and a mountpoint the reply does not carry,
/// are each refused,
/// because only the first keeps every automatic mount route away from the clone.
#[test]
fn the_mountpoint_must_be_legacy_and_must_be_readable() {
    let mut plain: CloneObjectFacts = managed_clone();
    plain.mountpoint = PropertyValue::Text(String::from("/home/user/Monochromatic/.claude/worktrees/one"));
    let refused = verify(&plain, PARENT, CHECKOUT).expect_err("an ordinary mountpoint is refused");
    assert!(matches!(refused, CloneRefusal::MountpointIsNotLegacy { .. }));
    assert_eq!(refused.step(), "mountpoint");

    let mut absent: CloneObjectFacts = managed_clone();
    absent.mountpoint = PropertyValue::Absent;
    let unreadable = verify(&absent, PARENT, CHECKOUT).expect_err("a missing mountpoint is refused");
    assert!(matches!(unreadable, CloneRefusal::Property { .. }), "{unreadable:?}");
    assert!(unreadable.message().contains("not in the reply"), "a person learns the property is missing: {}", unreadable.message());
}

/// Setuid or devices switched on are each refused,
/// and each names its own property.
#[test]
fn the_two_safety_properties_must_be_off() {
    let mut setuid: CloneObjectFacts = managed_clone();
    setuid.setuid = PropertyValue::Number(1);
    let refused = verify(&setuid, PARENT, CHECKOUT).expect_err("setuid on is refused");
    assert!(matches!(refused, CloneRefusal::SetuidIsOn { .. }));
    assert_eq!(refused.step(), "setuid");
    assert!(refused.message().contains(SETUID_PROPERTY), "the refusal names the property: {}", refused.message());

    let mut devices: CloneObjectFacts = managed_clone();
    devices.devices = PropertyValue::Number(1);
    let devices_refusal = verify(&devices, PARENT, CHECKOUT).expect_err("devices on is refused");
    assert!(matches!(devices_refusal, CloneRefusal::DevicesIsOn { .. }));
    assert_eq!(devices_refusal.step(), "devices");
    assert!(devices_refusal.message().contains(DEVICES_PROPERTY), "the refusal names the property: {}", devices_refusal.message());
}

/// What:
///  An unloaded key is refused as itself,
///  naming the encryption root,
///  and says it is neither an identity change nor a permission failure.
/// Why:
///  The measurement this step repeats saw the mount setup refuse with
///  `EACCES` before attachment,
///  which a report must not confuse with a folder another account replaced.
#[test]
fn an_unloaded_key_is_reported_apart_from_the_identity_checks() {
    let refused = verify(&encrypted_clone(1), PARENT, CHECKOUT).expect_err("an unloaded key is refused");
    assert!(matches!(refused, CloneRefusal::KeyUnavailable { .. }), "{refused:?}");
    assert_eq!(refused.step(), "key-unavailable");
    let said: String = refused.message();
    assert!(said.contains(ENCRYPTION_ROOT), "the refusal names the encryption root: {said}");
    assert!(said.contains("neither a changed folder identity nor a permission failure"), "the refusal tells the key apart: {said}");
    assert!(said.contains(ENCRYPTION_PROPERTY), "the refusal names the encryption property: {said}");
    assert!(said.contains(KEY_STATUS_PROPERTY), "the refusal names the key-status property: {said}");
    assert!(said.contains(ENCRYPTION_ROOT_PROPERTY), "the refusal names the encryption-root property: {said}");
    assert!(said.contains("load-key"), "the refusal names the way forward: {said}");
}

/// An encrypted clone whose key is loaded passes,
/// and so does a clone whose data is not encrypted,
/// whether its reply says `off` or carries no encryption property at all.
#[test]
fn a_loaded_key_and_an_unencrypted_clone_both_pass() {
    assert_eq!(verify(&encrypted_clone(2), PARENT, CHECKOUT).expect("a loaded key passes").slot, "one");
    let mut unencrypted: CloneObjectFacts = managed_clone();
    unencrypted.encryption = PropertyValue::Number(0);
    unencrypted.key_status = PropertyValue::Absent;
    assert_eq!(verify(&unencrypted, PARENT, CHECKOUT).expect("an unencrypted clone needs no key").slot, "one");
    // The guest's own plain clone answered with no `encryption` property at all, which is the same instruction as
    // `off`.
    let mut silent: CloneObjectFacts = managed_clone();
    silent.encryption = PropertyValue::Absent;
    silent.key_status = PropertyValue::Absent;
    silent.encryption_root = PropertyValue::Absent;
    assert_eq!(verify(&silent, PARENT, CHECKOUT).expect("a reply with no encryption property is an unencrypted clone").slot, "one");
    let mut unreadable: CloneObjectFacts = managed_clone();
    unreadable.encryption = PropertyValue::Unreadable { why: String::from("the pair has no `value` this route reads") };
    assert!(matches!(verify(&unreadable, PARENT, CHECKOUT), Err(CloneRefusal::Property { .. })), "a value this route cannot read is still refused");
}

/// Every refusal names a step a receipt can carry and the dataset it is about.
#[test]
fn every_refusal_names_a_short_step_and_its_dataset() {
    let refusals: Vec<CloneRefusal> = vec![
        CloneRefusal::NotAChild { name: String::from(CLONE), parent: String::from(PARENT) },
        CloneRefusal::SlotName { name: String::from(CLONE), slot: String::from("one"), reason: String::from("it is empty") },
        CloneRefusal::NotAFilesystem { name: String::from(CLONE), found: 9 },
        CloneRefusal::Snapshot { name: String::from(CLONE) },
        CloneRefusal::NoGuid { name: String::from(CLONE) },
        CloneRefusal::NoOrigin { name: String::from(CLONE) },
        CloneRefusal::OriginOutsideTheCheckout { name: String::from(CLONE), origin: String::from("p@o"), checkout_dataset: String::from(CHECKOUT) },
        CloneRefusal::Property { name: String::from(CLONE), said: String::from("`setuid` is not in the reply the kernel gave for this dataset"), step: "setuid" },
        CloneRefusal::MountpointIsNotLegacy { name: String::from(CLONE), found: String::from("/tmp") },
        CloneRefusal::SetuidIsOn { name: String::from(CLONE), found: String::from("1") },
        CloneRefusal::DevicesIsOn { name: String::from(CLONE), found: String::from("1") },
        CloneRefusal::KeyUnavailable { name: String::from(CLONE), encryption_root: String::from(ENCRYPTION_ROOT), encryption: String::from("encryption=7") },
    ];
    for refusal in refusals {
        assert!(refusal.step().len() <= MAXIMUM_WORD_BYTES, "{} must fit a receipt's word", refusal.step());
        assert!(refusal.message().contains(CLONE), "{}: the message names the dataset: {}", refusal.step(), refusal.message());
        assert!(refusal.message().contains("Nothing was mounted"), "{}: the message says nothing acted: {}", refusal.step(), refusal.message());
        assert_eq!(refusal.name(), CLONE);
    }
}
