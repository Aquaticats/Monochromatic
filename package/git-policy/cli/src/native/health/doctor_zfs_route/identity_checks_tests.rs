//! What:
//!  Controls of the recorded parent and slot identities.
//! Why:
//!  The route refuses a clone whose folder moved,
//!  and `doctor` must name the field rather than say "different",
//!  so each control changes one field of a recorded identity and reads the
//!  field the finding names,
//!  while a walk that could not read a folder stays unknown.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(check_parent_identity({ recorded, live })).toMatchObject({ kind: 'finding', field: 'mode' });
//! ```

use super::{LiveIdentity, ParentIdentityFacts, RecordedIdentity, SlotIdentityFacts, check_parent_identity, check_slot_identity, walk_refusal_is_unproved};
use crate::health::doctor_zfs_route::route_test_facts::{folder_identity, healthy_parent_identity};
use crate::worktree_clone_mount::{CloneFolderField, CloneSlotRefusal, WalkRefusal, WalkRefusalReason};

/// A walk that stopped on a component for one reason.
fn walk(component: &str, reason: WalkRefusalReason) -> WalkRefusal {
    return WalkRefusal { component: String::from(component), reason, detail: String::from("the system's own words") };
}

//region The parent

/// The recorded parent and the live folder agree.
#[test]
fn the_recorded_parent_and_the_live_folder_agree() {
    assert!(matches!(check_parent_identity(&healthy_parent_identity()), super::CheckOutcome::Pass { .. }));
}

/// A moved mode is a finding naming the mode.
#[test]
fn a_moved_mode_is_a_finding_naming_the_mode() {
    let mut live: crate::worktree_clone_mount::CloneFolderIdentity = folder_identity(7);
    live.mode = 0o755;
    let facts: ParentIdentityFacts = ParentIdentityFacts { recorded: RecordedIdentity::Read(Box::new(folder_identity(7))), live: LiveIdentity::Held(Box::new(live)) };
    let super::CheckOutcome::Finding { field, detail } = check_parent_identity(&facts) else {
        panic!("a moved mode is a finding");
    };
    assert_eq!(field, "mode");
    assert!(detail.contains("adopt-worktree-parent"), "{detail}");
}

/// Every moved field is named,
/// not only the first.
#[test]
fn every_moved_field_is_named() {
    let mut live: crate::worktree_clone_mount::CloneFolderIdentity = folder_identity(8);
    live.owner = 4242;
    let facts: ParentIdentityFacts = ParentIdentityFacts { recorded: RecordedIdentity::Read(Box::new(folder_identity(7))), live: LiveIdentity::Held(Box::new(live)) };
    let super::CheckOutcome::Finding { field, .. } = check_parent_identity(&facts) else {
        panic!("moved fields are a finding");
    };
    assert!(field.contains("inode"), "{field}");
    assert!(field.contains("owner"), "{field}");
}

/// A store with no parent record is a finding naming the adoption.
#[test]
fn a_store_without_a_parent_record_is_a_finding() {
    let facts: ParentIdentityFacts = ParentIdentityFacts { recorded: RecordedIdentity::Absent { why: String::from("no parent record is filed") }, live: LiveIdentity::Held(Box::new(folder_identity(7))) };
    let super::CheckOutcome::Finding { field, detail } = check_parent_identity(&facts) else {
        panic!("no parent record is a finding");
    };
    assert_eq!(field, "parent-record");
    assert!(detail.contains("adopt-worktree-parent"), "{detail}");
}

/// A parent record this account could not read leaves the answer unknown.
#[test]
fn an_unreadable_parent_record_is_unknown() {
    let facts: ParentIdentityFacts = ParentIdentityFacts { recorded: RecordedIdentity::Unreadable { why: String::from("permission denied") }, live: LiveIdentity::Held(Box::new(folder_identity(7))) };
    assert!(matches!(check_parent_identity(&facts), super::CheckOutcome::Unknown { .. }));
}

/// A parent folder that is not there is a finding naming the component.
#[test]
fn a_missing_parent_folder_is_a_finding_naming_the_component() {
    let facts: ParentIdentityFacts = ParentIdentityFacts { recorded: RecordedIdentity::Read(Box::new(folder_identity(7))), live: LiveIdentity::Refused(walk(".claude/worktrees", WalkRefusalReason::Missing)) };
    let super::CheckOutcome::Finding { field, .. } = check_parent_identity(&facts) else {
        panic!("a missing parent folder is a finding");
    };
    assert_eq!(field, ".claude/worktrees");
}

/// A walk this account was refused leaves the answer unknown.
#[test]
fn an_unreadable_parent_folder_is_unknown() {
    let facts: ParentIdentityFacts = ParentIdentityFacts { recorded: RecordedIdentity::Read(Box::new(folder_identity(7))), live: LiveIdentity::Refused(walk(".claude/worktrees", WalkRefusalReason::Unreadable)) };
    assert!(matches!(check_parent_identity(&facts), super::CheckOutcome::Unknown { .. }));
}

/// The unproved reasons are the reads that prove nothing about the folder.
#[test]
fn the_unproved_reasons_are_the_unreadable_ones() {
    assert!(walk_refusal_is_unproved(WalkRefusalReason::Unreadable));
    assert!(walk_refusal_is_unproved(WalkRefusalReason::UnreadableFields));
    assert!(walk_refusal_is_unproved(WalkRefusalReason::Checkout));
    assert!(!walk_refusal_is_unproved(WalkRefusalReason::Missing));
    assert!(!walk_refusal_is_unproved(WalkRefusalReason::LinkInTheWay));
}

//endregion

//region The slots

/// A slot that agreed passes.
#[test]
fn a_slot_that_agreed_passes() {
    assert!(matches!(check_slot_identity(&SlotIdentityFacts { slot: String::from("one"), refusal: None }), super::CheckOutcome::Pass { .. }));
}

/// A moved slot field is a finding naming the field.
#[test]
fn a_moved_slot_field_is_a_finding_naming_the_field() {
    let refusal: CloneSlotRefusal = CloneSlotRefusal::SlotIdentity {
        slot: String::from("one"),
        recorded: Box::new(folder_identity(7)),
        found: Box::new(folder_identity(9)),
        fields: vec![CloneFolderField::Inode],
    };
    let super::CheckOutcome::Finding { field, .. } = check_slot_identity(&SlotIdentityFacts { slot: String::from("one"), refusal: Some(refusal) }) else {
        panic!("a moved slot is a finding");
    };
    assert_eq!(field, "inode");
}

/// A slot whose two descriptors named different folders is a finding naming
/// the slot.
#[test]
fn a_slot_that_moved_between_two_looks_is_a_finding() {
    let refusal: CloneSlotRefusal = CloneSlotRefusal::TargetAnotherObject { slot: String::from("one"), first: String::from("inode 7"), second: String::from("inode 9") };
    let super::CheckOutcome::Finding { field, .. } = check_slot_identity(&SlotIdentityFacts { slot: String::from("one"), refusal: Some(refusal) }) else {
        panic!("a slot that moved is a finding");
    };
    assert_eq!(field, "slot");
}

/// A missing slot folder is a finding naming the component.
#[test]
fn a_missing_slot_folder_is_a_finding_naming_the_component() {
    let refusal: CloneSlotRefusal = CloneSlotRefusal::Walk(walk(".claude/worktrees/one", WalkRefusalReason::Missing));
    let super::CheckOutcome::Finding { field, .. } = check_slot_identity(&SlotIdentityFacts { slot: String::from("one"), refusal: Some(refusal) }) else {
        panic!("a missing slot is a finding");
    };
    assert_eq!(field, ".claude/worktrees/one");
}

/// A slot folder this account was refused leaves the answer unknown.
#[test]
fn an_unreadable_slot_folder_is_unknown() {
    let refusal: CloneSlotRefusal = CloneSlotRefusal::Walk(walk(".claude/worktrees/one", WalkRefusalReason::Unreadable));
    assert!(matches!(check_slot_identity(&SlotIdentityFacts { slot: String::from("one"), refusal: Some(refusal) }), super::CheckOutcome::Unknown { .. }));
}

/// A store refusal leaves the slot's answer unknown.
#[test]
fn a_store_refusal_leaves_the_slot_unknown() {
    let refusal: CloneSlotRefusal = CloneSlotRefusal::Store(crate::worktree_clone_mount::StoreRefusal::NoSlotRecord { path: std::path::PathBuf::from("/var/lib/store"), slot: String::from("one") });
    assert!(matches!(check_slot_identity(&SlotIdentityFacts { slot: String::from("one"), refusal: Some(refusal) }), super::CheckOutcome::Unknown { .. }));
}

//endregion
