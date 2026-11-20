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
    assert_eq!(decide_adoption(&with_nothing).expect("no binding and no slot is nothing to reconcile").identity, held);
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
    let refused: AdoptionRefusal = decide_adoption(&wrong_inode).expect_err("another inode");
    assert!(matches!(refused, AdoptionRefusal::Identity { field: CloneFolderField::Inode, .. }), "{refused:?}");
    assert!(refused.message().contains("Nothing was recorded"), "{}", refused.message());
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
        let refused: AdoptionRefusal = decide_adoption(&facts).expect_err(field);
        match &refused {
            AdoptionRefusal::Policy { fields } => assert!(fields.iter().any(|moved| moved.name() == *field), "{field}: {refused:?}"),
            other => panic!("{field}: {other:?}"),
        }
        assert!(refused.message().contains(field), "{field}: {}", refused.message());
        assert!(refused.message().contains("Ways forward"), "{}", refused.message());
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
        let refused: AdoptionRefusal = decide_adoption(&facts).expect_err("the walk refused");
        assert!(matches!(refused, AdoptionRefusal::Walk { .. }), "{refused:?}");
        assert!(refused.message().contains(".claude/worktrees"), "{}", refused.message());
        assert!(refused.message().contains(expected), "{}", refused.message());
        assert!(refused.message().contains("nothing was recorded"), "{}", refused.message());
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
    let refused: AdoptionRefusal = decide_adoption(&unresolved).expect_err("an unresolved binding");
    assert!(matches!(refused, AdoptionRefusal::BindingUnresolved { .. }), "{refused:?}");
    assert!(refused.message().contains("slot three") && refused.message().contains("its mount row is unreadable"), "{}", refused.message());
    assert!(refused.message().contains("recovery"), "{}", refused.message());
    let mut differing_slot: AdoptionFacts = adopting(&held);
    differing_slot.slots.push(SlotState::Differs { slot: String::from("four"), field: CloneFolderField::Mode });
    let refused: AdoptionRefusal = decide_adoption(&differing_slot).expect_err("a slot that differs");
    assert!(matches!(refused, AdoptionRefusal::SlotDiffers { .. }), "{refused:?}");
    assert!(refused.message().contains("`four`") && refused.message().contains("mode"), "{}", refused.message());
    assert!(refused.message().contains("never overwrites"), "{}", refused.message());
    let mut fresh_stopped: AdoptionFacts = adopting(&held);
    fresh_stopped.fresh = Err(walk_refusal(".claude/worktrees", WalkRefusalReason::Missing));
    let refused: AdoptionRefusal = decide_adoption(&fresh_stopped).expect_err("the fresh lookup stopped");
    assert!(matches!(refused, AdoptionRefusal::FreshLookup { .. }), "{refused:?}");
    assert!(refused.message().contains("not there"), "{}", refused.message());
    let mut fresh_other: AdoptionFacts = adopting(&held);
    fresh_other.fresh = Ok(CloneFolderIdentity { inode: held.inode + 1, ..held.clone() });
    let refused: AdoptionRefusal = decide_adoption(&fresh_other).expect_err("the fresh lookup reached another folder");
    assert!(matches!(refused, AdoptionRefusal::FreshLookupDiffers { ref fields } if fields.contains(&CloneFolderField::Inode)), "{refused:?}");
    assert!(refused.message().contains("inode"), "{}", refused.message());
    remove(root.as_path());
}
