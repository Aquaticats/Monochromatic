//! What:
//!  Controls of the adoption's refusals on a machine without a root-owned store:
//!  the not-root answer,
//!  the sentence each cause gives,
//!  and that a refusal records nothing;
//!  and controls of the surviving-slot read over a plain fixture folder:
//!  a slot whose folder is gone,
//!  a slot name that is no folder at all,
//!  the slot that agrees with its record,
//!  and the one that differs.
//! Why:
//!  The mode writes one store under `/var/lib`,
//!  which a control must never touch,
//!  so what runs here is the part before any write:
//!  the two facts a control cannot set,
//!  the store's path and the process's effective user,
//!  are arguments of the driver,
//!  and the store and the slots of the read controls are fixtures of the test's own.
//!  What still needs the guest is the mount arm:
//!  a mount standing at a slot reads as unresolved only where a mount can be made
//!  (`slot_folder_reason`'s `EXDEV` sentence),
//!  and the end-to-end arm runs in the throwaway guest
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 5,
//!  "Acceptance tests").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(adoptAt({ runningAsRoot: false, effectiveUser: 1000 })).toEqual({ refused: 'not-root' });
//! ```

use super::{AdoptFailure, adopt_at, recorded_slots};
use crate::test_support::fixture;
use crate::worktree_clone_mount::clone_folder_identity::{CloneFolderField, CloneFolderIdentity};
use crate::worktree_clone_mount::parent_adoption_decision::AdoptionRefusal;
use crate::worktree_clone_mount::parent_adoption_facts::{BindingState, SlotState};
use crate::worktree_clone_mount::slot_identity_store::SlotIdentityStore;
use crate::worktree_clone_mount::slot_identity_store_refusals::StoreRefusal;
use crate::worktree_clone_mount::{HeldWorktreesParent, STORE_HEADER, StoredIdentity, hold_the_worktrees_parent};
use crate::worktree_clone_mount_handler::handler_mode_arguments::AdoptRequest;
use std::path::Path;
use std::path::PathBuf;

/// An adoption of the checkout this repository is,
/// identified as the test reads it.
fn request() -> AdoptRequest {
    return AdoptRequest { checkout: PathBuf::from("/home/user/Monochromatic"), device: 66306, inode: 1_234_567 };
}

/// A caller that is not root is refused before the store is opened,
/// and the sentence names the effective user.
#[test]
fn a_caller_that_is_not_root_is_refused_before_the_store_is_read() {
    let refusal: AdoptFailure = adopt_at(&request(), Path::new("/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities"), false, 1000)
        .expect_err("a caller that is not root records nothing");
    assert_eq!(refusal, AdoptFailure::NotRoot { effective_user: 1000 });
    let message: String = refusal.message();
    assert!(message.contains("effective user is 1000"), "{message}");
    assert!(message.contains("Nothing was read or recorded"), "{message}");
}

/// Every refusal names what an administrator fixes next,
/// and every one says the record is unchanged.
#[test]
fn every_refusal_names_its_cause_and_the_way_forward() {
    let cases: Vec<(AdoptFailure, &str)> = vec![
        (
            AdoptFailure::Store(StoreRefusal::Unreadable { path: PathBuf::from("/var/lib/x"), reason: String::from("the file is not there") }),
            "could not be read",
        ),
        (AdoptFailure::NoParentRecord { path: PathBuf::from("/var/lib/x") }, "holds no record"),
        (
            AdoptFailure::Decision(AdoptionRefusal::Identity { field: crate::worktree_clone_mount::CloneFolderField::Inode, identified: String::from("1"), held: String::from("2") }),
            "Nothing was recorded",
        ),
        (
            AdoptFailure::Write(StoreRefusal::Unreadable { path: PathBuf::from("/var/lib/x"), reason: String::from("the folder is gone") }),
            "unchanged",
        ),
    ];
    for (refusal, expected) in cases {
        let message: String = refusal.message();
        assert!(message.contains(expected), "{refusal:?} should say {expected}: {message}");
        assert!(message.contains("Ways forward"), "{refusal:?} should say what to do next: {message}");
    }
}

/// One fixture checkout with a recreated `.claude/worktrees` folder,
/// and the path of a store under the fixture root a control writes.
struct Fixture {
    /// The checkout,
    /// canonical,
    /// as the walk reaches it.
    checkout: PathBuf,
    /// The `.claude/worktrees` folder the walk holds.
    parent: PathBuf,
    /// The store a control writes for this fixture.
    store: PathBuf,
}

/// A fixture checkout named `name`,
/// with `.claude/worktrees` below it and no store yet.
fn fixture_checkout(name: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let checkout: PathBuf = root.join("checkout");
    let parent: PathBuf = checkout.join(".claude/worktrees");
    std::fs::create_dir_all(&parent).expect("the worktrees parent");
    return Fixture { checkout, parent, store: root.join("slot-identities") };
}

/// The identity of the folder at `path`,
/// read from its descriptor.
fn identity_of(path: &Path) -> CloneFolderIdentity {
    return CloneFolderIdentity::capture(&std::fs::File::open(path).expect("a folder to read")).expect("an identity");
}

/// Write a store recording `parent` and the slot `name` as `slot`.
fn write_store(path: &Path, parent: &CloneFolderIdentity, name: &str, slot: &CloneFolderIdentity) {
    let text: String = format!(
        "{STORE_HEADER}\n{}\n{}\n",
        StoredIdentity::Parent(parent.clone()).encode(),
        StoredIdentity::Slot { name: name.to_string(), identity: slot.clone() }.encode()
    );
    std::fs::write(path, text).expect("a store fixture");
}

/// The recorded slots of a fixture checkout,
/// read by the mode's own reader,
/// with the held parent folder.
fn slots_and_bindings(built: &Fixture) -> (Vec<SlotState>, Vec<BindingState>) {
    let held: HeldWorktreesParent = hold_the_worktrees_parent(built.checkout.as_path()).expect("the walk holds the recreated parent");
    let store: SlotIdentityStore = SlotIdentityStore::read(built.store.as_path()).expect("the fixture store reads");
    return recorded_slots(Some(&held), &store);
}

/// What:
///  A recorded slot whose folder is gone is an unresolved binding,
///  and the sentence names the slot and the recovery it needs.
/// Why:
///  The removal workflow settles a slot whose folder went with the parent,
/// so the adoption refuses it
///  instead of keeping it as a surviving slot
///  or adopting over a binding nothing proves survives.
#[test]
fn a_recorded_slot_with_no_folder_is_an_unresolved_binding() {
    let built: Fixture = fixture_checkout("adopt-slot-gone");
    let parent: CloneFolderIdentity = identity_of(&built.parent);
    write_store(&built.store, &parent, "one", &parent);
    let (slots, bindings) = slots_and_bindings(&built);
    assert!(slots.is_empty(), "no slot survives here: {slots:?}");
    assert_eq!(bindings.len(), 1, "{bindings:?}");
    let BindingState::Unresolved { what, reason } = &bindings[0] else {
        panic!("a recorded slot with no folder is an unresolved binding, and this is {:?}", bindings[0]);
    };
    assert!(what.contains("`one`"), "the sentence names the slot: {what}");
    assert!(reason.contains("holds nothing at that slot"), "the sentence names the missing folder: {reason}");
    assert!(reason.contains("recovery:"), "the sentence names the way forward: {reason}");
}

/// What:
///  A slot name that holds no readable folder is an unresolved binding too,
///  and its sentence is the third cause rather than the missing folder's.
/// Why:
///  A file or a link at the slot name is the folder's own defect,
///  whose recovery differs from a registration the removal workflow settles,
///  so the reading must not report it as a missing folder.
#[test]
fn a_slot_name_that_is_no_folder_is_an_unresolved_binding() {
    let built: Fixture = fixture_checkout("adopt-slot-not-a-folder");
    let parent: CloneFolderIdentity = identity_of(&built.parent);
    std::fs::write(built.parent.join("one"), "not a folder").expect("a file at the slot name");
    write_store(&built.store, &parent, "one", &parent);
    let (slots, bindings) = slots_and_bindings(&built);
    assert!(slots.is_empty(), "no slot survives here: {slots:?}");
    assert_eq!(bindings.len(), 1, "{bindings:?}");
    let BindingState::Unresolved { what, reason } = &bindings[0] else {
        panic!("a slot name that is no folder is an unresolved binding, and this is {:?}", bindings[0]);
    };
    assert!(what.contains("`one`"), "the sentence names the slot: {what}");
    assert!(reason.contains("could not be read"), "the sentence names the folder: {reason}");
    assert!(!reason.contains("holds nothing at that slot"), "a file at the name is no missing folder: {reason}");
}

/// What:
///  A slot folder that is its record agrees,
///  and one whose folder differs from its record is refused naming the field.
/// Why:
///  These are the other two readings of the same walk,
///  and they are what tells a surviving slot from a replaced one;
///  a reading that answered one for the other would keep a slot nothing proves survives.
#[test]
fn a_slot_folder_that_is_its_record_agrees_and_a_changed_one_differs() {
    let built: Fixture = fixture_checkout("adopt-slot-states");
    let slot: PathBuf = built.parent.join("one");
    std::fs::create_dir(&slot).expect("a slot folder");
    let parent: CloneFolderIdentity = identity_of(&built.parent);
    write_store(&built.store, &parent, "one", &identity_of(&slot));
    let (slots, bindings) = slots_and_bindings(&built);
    assert_eq!(slots, vec![SlotState::Agrees { slot: String::from("one") }]);
    assert!(bindings.is_empty(), "{bindings:?}");
    write_store(&built.store, &parent, "one", &parent);
    let (changed_slots, changed_bindings) = slots_and_bindings(&built);
    assert!(changed_bindings.is_empty(), "{changed_bindings:?}");
    assert_eq!(
        changed_slots,
        vec![
            SlotState::Differs {
                slot: String::from("one"),
                field: CloneFolderField::Inode
            }
        ]
    );
}
