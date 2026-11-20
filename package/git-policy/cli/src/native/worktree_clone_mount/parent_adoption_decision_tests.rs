//! Controls for the administrator's adoption decision:
//! an identified replacement with the recorded shape and no unresolved binding is adopted,
//! and each field,
//! walk refusal,
//! unresolved binding,
//! differing slot and fresh-lookup mismatch refuses it naming what is wrong.

use super::*;
use crate::clone_folder_identity::CloneFolderIdentity;
use crate::clone_slot_walk_refusals::{WalkRefusal, WalkRefusalReason};
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// The identity of one real fixture folder,
/// so the policy is the folder's own shape and the ACL fields are real.
fn planted_identity(name: &str) -> (PathBuf, CloneFolderIdentity) {
    let root: PathBuf = fixture(name);
    std::fs::create_dir(root.join("worktrees")).expect("folder");
    let identity: CloneFolderIdentity = CloneFolderIdentity::capture(&std::fs::File::open(root.join("worktrees")).expect("open")).expect("identity");
    return (root, identity);
}

/// Facts that adopt `held` as it is.
fn adopting(held: &CloneFolderIdentity) -> AdoptionFacts {
    return AdoptionFacts {
        walk: Ok(held.clone()),
        identified_device: held.device,
        identified_inode: held.inode,
        policy: ParentPolicy::from_recorded(held),
        bindings: vec![
            BindingState::Live { what: String::from("slot one"), place: PathBuf::from("/checkout/.claude/worktrees/one") },
            BindingState::Staged { what: String::from("slot two"), place: PathBuf::from("/home/user/.cache/cli-git/staging/two") },
        ],
        slots: vec![SlotState::Agrees { slot: String::from("one") }, SlotState::Agrees { slot: String::from("two") }],
        recorded_slots: vec![String::from("one"), String::from("two")],
        fresh: Ok(held.clone()),
    };
}

/// The refusal of a walk that stopped on `component` for `reason`.
fn walk_refusal(component: &str, reason: WalkRefusalReason) -> WalkRefusal {
    return WalkRefusal { component: String::from(component), reason, detail: String::from("the system's own words") };
}

/// What:
///  An identified replacement with the recorded shape,
///  every binding live or staged and every slot agreeing is adopted,
///  and the identity adopted is the held folder's own.
/// Why:
///  This is the positive control of the table:
///  every refusal below is proved against facts that otherwise pass.
#[test]
fn an_identified_replacement_with_the_recorded_shape_is_adopted() {
    let (root, held): (PathBuf, CloneFolderIdentity) = planted_identity("adoption-positive");
    let decision: Adoption = decide_adoption(&adopting(&held)).expect("the replacement is the identified one");
    assert_eq!(decision.identity, held);
    let mut with_nothing: AdoptionFacts = adopting(&held);
    with_nothing.bindings = Vec::new();
    with_nothing.slots = Vec::new();
    with_nothing.recorded_slots = Vec::new();
    assert_eq!(decide_adoption(&with_nothing).expect("a store that records no slot has nothing to reconcile").identity, held);
    remove(root.as_path());
}

/// What:
///  A slot the store records with no state in the caller's list refuses the adoption,
///  naming the slot,
///  while a list that covers every recorded slot and an empty record both adopt.
/// Why:
///  An empty list adopted a replacement while the store still recorded surviving slots,
///  so an adoption could leave a slot's identity unexamined;
///  the caller's list is judged against the store's own record,
///  never trusted for being a list.
#[test]
fn a_recorded_slot_with_no_state_refuses_the_adoption() {
    let (root, held): (PathBuf, CloneFolderIdentity) = planted_identity("adoption-recorded-slots");
    assert_eq!(decide_adoption(&adopting(&held)).expect("positive control: every recorded slot is listed").identity, held);
    let mut unlisted: AdoptionFacts = adopting(&held);
    unlisted.recorded_slots = vec![String::from("one"), String::from("two"), String::from("three")];
    let refused: AdoptionRefusal = decide_adoption(&unlisted).expect_err("a recorded slot with no state");
    assert!(matches!(refused, AdoptionRefusal::SlotNotListed { ref slot } if slot == "three"), "{refused:?}");
    assert!(refused.message().contains("`three`"), "the slot is named: {}", refused.message());
    assert!(refused.message().contains("slot_names"), "the way forward names the store's own list: {}", refused.message());
    let mut nothing_recorded: AdoptionFacts = adopting(&held);
    nothing_recorded.slots = Vec::new();
    nothing_recorded.recorded_slots = Vec::new();
    nothing_recorded.bindings = Vec::new();
    assert_eq!(decide_adoption(&nothing_recorded).expect("a store that records no slot needs no state").identity, held);
    remove(root.as_path());
}

/// What:
///  A device or an inode the administrator did not identify refuses the adoption,
///  naming the field and both values.
/// Why:
///  The replacement must be identified independently from the restored checkout,
///  never taken from whatever stands at the path.
#[test]
fn a_folder_the_administrator_did_not_identify_is_refused() {
    let (root, held): (PathBuf, CloneFolderIdentity) = planted_identity("adoption-identity");
    let mut wrong_device: AdoptionFacts = adopting(&held);
    wrong_device.identified_device = held.device + 1;
    let refused: AdoptionRefusal = decide_adoption(&wrong_device).expect_err("another device");
    assert!(matches!(refused, AdoptionRefusal::Identity { field: CloneFolderField::Device, .. }), "{refused:?}");
    assert!(refused.message().contains("device"), "{}", refused.message());
    let mut wrong_inode: AdoptionFacts = adopting(&held);
    wrong_inode.identified_inode = held.inode + 1;
    let refused_2: AdoptionRefusal = decide_adoption(&wrong_inode).expect_err("another inode");
    assert!(matches!(refused_2, AdoptionRefusal::Identity { field: CloneFolderField::Inode, .. }), "{refused_2:?}");
    assert!(refused_2.message().contains("Nothing was recorded"), "{}", refused_2.message());
    remove(root.as_path());
}

/// What:
///  Each policy field the replacement does not hold refuses the adoption,
///  naming the field,
///  with the owner and mode and ACL mismatches proved one at a time.
/// Why:
///  A replacement must not come back wider than the folder it replaces,
///  and the acceptance names the ACL mismatch as its own refusal.
#[test]
fn each_policy_field_is_a_refusal() {
    let (root, held): (PathBuf, CloneFolderIdentity) = planted_identity("adoption-policy");
    let changed: [(&str, CloneFolderIdentity); 4] = [
        ("owner", CloneFolderIdentity { owner: held.owner + 1, ..held.clone() }),
        ("mode", CloneFolderIdentity { mode: held.mode ^ 0o001, ..held.clone() }),
        ("access ACL", CloneFolderIdentity { access_acl: Some(vec![2, 0, 0, 0]), ..held.clone() }),
        ("default ACL", CloneFolderIdentity { default_acl: Some(vec![2, 0, 0, 0]), ..held.clone() }),
    ];
    for (field, replacement) in &changed {
        let mut facts: AdoptionFacts = adopting(&held);
        facts.walk = Ok(replacement.clone());
        facts.identified_device = replacement.device;
        facts.identified_inode = replacement.inode;
        let refused_3: AdoptionRefusal = decide_adoption(&facts).expect_err(field);
        match &refused_3 {
            AdoptionRefusal::Policy { fields } => assert!(fields.iter().any(|moved| { return moved.name() == *field; }), "{field}: {refused_3:?}"),
            other => panic!("{field}: {other:?}"),
        }
        assert!(refused_3.message().contains(field), "{field}: {}", refused_3.message());
        assert!(refused_3.message().contains("Ways forward"), "{}", refused_3.message());
    }
    remove(root.as_path());
}

/// What:
///  A link or a boundary the walk refused stops the adoption,
///  and its sentence names the component.
/// Why:
///  The acceptance names a link and a boundary as refusals,
///  and the walk's own refusal is what reports them.
#[test]
fn a_link_or_a_boundary_in_the_walk_is_refused() {
    let (root, held): (PathBuf, CloneFolderIdentity) = planted_identity("adoption-walk");
    for (reason, expected) in [(WalkRefusalReason::LinkInTheWay, "never follows a link"), (WalkRefusalReason::BoundaryCrossed, "never crosses such a boundary")] {
        let mut facts: AdoptionFacts = adopting(&held);
        facts.walk = Err(walk_refusal(".claude/worktrees", reason));
        let refused_4: AdoptionRefusal = decide_adoption(&facts).expect_err("the walk refused_4");
        assert!(matches!(refused_4, AdoptionRefusal::Walk { .. }), "{refused_4:?}");
        assert!(refused_4.message().contains(".claude/worktrees"), "{}", refused_4.message());
        assert!(refused_4.message().contains(expected), "{}", refused_4.message());
        assert!(refused_4.message().contains("nothing was recorded"), "{}", refused_4.message());
    }
    remove(root.as_path());
}

/// What:
///  An unresolved live or staged binding refuses the adoption,
///  naming the binding and the recovery it needs,
///  and the fresh lookup's refusal or mismatch refuses it too.
/// Why:
///  A replacement is recorded only once every affected binding is resolved,
///  and only when the fresh canonical lookup reached the held folder.
#[test]
fn an_unresolved_binding_a_differing_slot_and_a_fresh_lookup_mismatch_are_refused() {
    let (root, held): (PathBuf, CloneFolderIdentity) = planted_identity("adoption-bindings");
    let mut unresolved: AdoptionFacts = adopting(&held);
    unresolved.bindings.push(BindingState::Unresolved { what: String::from("slot three"), reason: String::from("its mount row is unreadable") });
    let refused_5: AdoptionRefusal = decide_adoption(&unresolved).expect_err("an unresolved binding");
    assert!(matches!(refused_5, AdoptionRefusal::BindingUnresolved { .. }), "{refused_5:?}");
    assert!(refused_5.message().contains("slot three") && refused_5.message().contains("its mount row is unreadable"), "{}", refused_5.message());
    assert!(refused_5.message().contains("recovery"), "{}", refused_5.message());
    let mut differing_slot: AdoptionFacts = adopting(&held);
    differing_slot.slots.push(SlotState::Differs { slot: String::from("four"), field: CloneFolderField::Mode });
    let refused_6: AdoptionRefusal = decide_adoption(&differing_slot).expect_err("a slot that differs");
    assert!(matches!(refused_6, AdoptionRefusal::SlotDiffers { .. }), "{refused_6:?}");
    assert!(refused_6.message().contains("`four`") && refused_6.message().contains("mode"), "{}", refused_6.message());
    assert!(refused_6.message().contains("never overwrites"), "{}", refused_6.message());
    let mut fresh_stopped: AdoptionFacts = adopting(&held);
    fresh_stopped.fresh = Err(walk_refusal(".claude/worktrees", WalkRefusalReason::Missing));
    let refused_7: AdoptionRefusal = decide_adoption(&fresh_stopped).expect_err("the fresh lookup stopped");
    assert!(matches!(refused_7, AdoptionRefusal::FreshLookup { .. }), "{refused_7:?}");
    assert!(refused_7.message().contains("not there"), "{}", refused_7.message());
    let mut fresh_other: AdoptionFacts = adopting(&held);
    fresh_other.fresh = Ok(CloneFolderIdentity { inode: held.inode + 1, ..held.clone() });
    let refused_8: AdoptionRefusal = decide_adoption(&fresh_other).expect_err("the fresh lookup reached another folder");
    assert!(matches!(refused_8, AdoptionRefusal::FreshLookupDiffers { ref fields } if fields.contains(&CloneFolderField::Inode)), "{refused_8:?}");
    assert!(refused_8.message().contains("inode"), "{}", refused_8.message());
    remove(root.as_path());
}
