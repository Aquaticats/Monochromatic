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

use super::{
    AdoptFailure,
    BETWEEN_THE_TWO_LISTINGS,
    adopt_at,
    recorded_slots,
};
use crate::test_support::fixture;
use crate::worktree_clone_mount::clone_folder_identity::{
    CloneFolderField,
    CloneFolderIdentity,
};
use crate::worktree_clone_mount::parent_adoption_decision::AdoptionRefusal;
use crate::worktree_clone_mount::parent_adoption_facts::{
    BindingState,
    SlotState,
};
use crate::worktree_clone_mount::slot_identity_store::SlotIdentityStore;
use crate::worktree_clone_mount::slot_identity_store_refusals::StoreRefusal;
use crate::worktree_clone_mount::{
    HeldWorktreesParent,
    PARENT_RECORD_NAME,
    SLOT_IDENTITY_STORE_PATH,
    STORE_HEADER,
    StoredIdentity,
    WHEN_THE_STORE_IS_LISTED,
    hold_the_worktrees_parent,
};
use crate::worktree_clone_mount_handler::handler_mode_arguments::AdoptRequest;
use std::path::Path;
use std::path::PathBuf;

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
    return Fixture {
        checkout,
        parent,
        store: root.join("slot-identities"),
    };
}

/// The identity of the folder at `path`,
/// read from its descriptor.
fn identity_of(path: &Path) -> CloneFolderIdentity {
    return CloneFolderIdentity::capture(&std::fs::File::open(path).expect("a folder to read"))
        .expect("an identity");
}

/// Write a store folder recording `parent` and the slot `name` as `slot`.
fn write_store(
    path: &Path,
    parent: &CloneFolderIdentity,
    name: &str,
    slot: &CloneFolderIdentity,
) {
    std::fs::create_dir_all(path).expect("the store folder fixture");
    let parent_text: String = format!(
        "{STORE_HEADER}\n{}\n",
        StoredIdentity::Parent(parent.clone()).encode()
    );
    std::fs::write(path.join(PARENT_RECORD_NAME), parent_text).expect("the parent record fixture");
    let slot_text: String = format!(
        "{STORE_HEADER}\n{}\n",
        StoredIdentity::Slot {
            name: name.to_string(),
            identity: slot.clone()
        }
        .encode()
    );
    std::fs::write(path.join(name), slot_text).expect("the slot record fixture");
}

/// The recorded slots of a fixture checkout,
/// read by the mode's own reader,
/// with the held parent folder.
fn slots_and_bindings(built: &Fixture) -> (Vec<SlotState>, Vec<BindingState>) {
    let held: HeldWorktreesParent = hold_the_worktrees_parent(built.checkout.as_path())
        .expect("the walk holds the recreated parent");
    let store: SlotIdentityStore =
        SlotIdentityStore::open(built.store.as_path()).expect("the fixture store opens");
    let names: Vec<String> = store.slot_names().expect("the fixture store lists");
    return recorded_slots(Some(&held), &store, names.as_slice()).expect("the recorded slots read");
}

/// An adoption of a fixture's checkout,
/// identified by the folder's own identity,
/// so no number here is one a control made up.
fn request(built: &Fixture) -> AdoptRequest {
    let identified: CloneFolderIdentity = identity_of(&built.parent);
    return AdoptRequest {
        checkout: built.checkout.clone(),
        device: identified.device,
        inode: identified.inode,
    };
}

/// A caller that is not root is refused before the store is opened,
/// and the sentence names the effective user.
#[test]
fn a_caller_that_is_not_root_is_refused_before_the_store_is_read() {
    let built: Fixture = fixture_checkout("adopt-not-root");
    let refusal: AdoptFailure = adopt_at(
        &request(&built),
        Path::new(SLOT_IDENTITY_STORE_PATH),
        false,
        1000,
    )
    .expect_err("a caller that is not root records nothing");
    assert_eq!(
        refusal,
        AdoptFailure::NotRoot {
            effective_user: 1000
        }
    );
    let message: String = refusal.message();
    assert!(
        message.starts_with(&format!(
            "{}: ",
            crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name()
        )),
        "the refusal opens with the words: {message}"
    );
    assert!(message.contains("effective user is 1000"), "{message}");
    assert!(
        message.contains("Nothing was read or recorded"),
        "{message}"
    );
}

/// Every refusal names what an administrator fixes next,
/// and every one says the record is unchanged.
#[test]
fn every_refusal_names_its_cause_and_the_way_forward() {
    let built: Fixture = fixture_checkout("adopt-refusal-sentences");
    let identified: String = identity_of(&built.parent).describe();
    let held: String = identity_of(&built.checkout).describe();
    let store: PathBuf = PathBuf::from(SLOT_IDENTITY_STORE_PATH);
    let cases: Vec<(AdoptFailure, &str)> = vec![
        (
            AdoptFailure::Store(StoreRefusal::Unreadable {
                path: store.clone(),
                reason: String::from("the file is not there"),
            }),
            "could not be read",
        ),
        (
            AdoptFailure::NoParentRecord {
                path: store.clone(),
            },
            "holds no record",
        ),
        (
            AdoptFailure::Decision(AdoptionRefusal::Identity {
                field: CloneFolderField::Inode,
                identified,
                held,
            }),
            "Nothing was recorded",
        ),
        (
            AdoptFailure::Write(StoreRefusal::Unreadable {
                path: store,
                reason: String::from("the folder is gone"),
            }),
            "unchanged",
        ),
    ];
    for (refusal, expected) in cases {
        let message: String = refusal.message();
        assert!(
            message.starts_with(&format!(
                "{}: ",
                crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name()
            )),
            "{refusal:?} opens with the words every root-mode refusal opens with: {message}"
        );
        assert!(
            message.contains(expected),
            "{refusal:?} should say {expected}: {message}"
        );
        assert!(
            message.contains("Ways forward"),
            "{refusal:?} should say what to do next: {message}"
        );
    }
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
    let BindingState::Unresolved {
        what,
        reason,
    } = &bindings[0]
    else {
        panic!(
            "a recorded slot with no folder is an unresolved binding, and this is {:?}",
            bindings[0]
        );
    };
    assert!(
        what.contains("`one`"),
        "the sentence names the slot: {what}"
    );
    assert!(
        reason.contains("holds nothing at that slot"),
        "the sentence names the missing folder: {reason}"
    );
    assert!(
        reason.contains("recovery:"),
        "the sentence names the way forward: {reason}"
    );
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
    let BindingState::Unresolved {
        what,
        reason,
    } = &bindings[0]
    else {
        panic!(
            "a slot name that is no folder is an unresolved binding, and this is {:?}",
            bindings[0]
        );
    };
    assert!(
        what.contains("`one`"),
        "the sentence names the slot: {what}"
    );
    assert!(
        reason.contains("could not be read"),
        "the sentence names the folder: {reason}"
    );
    assert!(
        !reason.contains("holds nothing at that slot"),
        "a file at the name is no missing folder: {reason}"
    );
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
    assert_eq!(
        slots,
        vec![SlotState::Agrees {
            slot: String::from("one")
        }]
    );
    assert!(bindings.is_empty(), "{bindings:?}");
    write_store(&built.store, &parent, "one", &parent);
    let (changed_slots, changed_bindings) = slots_and_bindings(&built);
    assert!(changed_bindings.is_empty(), "{changed_bindings:?}");
    assert_eq!(
        changed_slots,
        vec![SlotState::Differs {
            slot: String::from("one"),
            field: CloneFolderField::Inode
        }]
    );
}

/// Environment variable that makes the control below run its inner half,
/// inside a user namespace of its own,
/// where this run's own files are root's.
#[cfg(target_os = "linux")]
const ROOT_SIDE_LISTINGS_VARIABLE: &str = "ADOPT_ROOT_SIDE_TWO_LISTINGS";

/// What:
///  A slot record published between the adoption's two listings refuses the adoption,
///  naming the slot,
///  and `.parent` keeps the identity it had.
/// Why:
///  The decision verified exactly the names its first listing produced,
///  so a slot that appeared afterwards was not verified by this adoption;
///  the second listing is what closes that window without a lock.
///  The count this control also reads is what holds the promise of two listings:
///  the first is also the pass that removes a killed writer's staging names,
///  so an adoption lists the folder twice and not once per thing it needs.
#[cfg(target_os = "linux")]
#[test]
fn a_slot_file_that_appears_between_the_two_listings_refuses_the_adoption() {
    if !crate::testing::root_namespace_test_support::in_a_root_namespace(
        ROOT_SIDE_LISTINGS_VARIABLE,
        "worktree_clone_mount_handler::adopt_worktree_parent_mode::tests::a_slot_file_that_appears_between_the_two_listings_refuses_the_adoption",
    ) {
        return;
    }
    let built: Fixture = fixture_checkout("adopt-two-listings");
    let surviving: PathBuf = built.parent.join("one");
    std::fs::create_dir(surviving.as_path()).expect("a surviving slot folder");
    let parent: CloneFolderIdentity = identity_of(&built.parent);
    write_store(
        &built.store,
        &parent,
        "one",
        &identity_of(surviving.as_path()),
    );
    let parent_before: Vec<u8> =
        std::fs::read(built.store.join(PARENT_RECORD_NAME)).expect("the parent record's bytes");
    let appeared: PathBuf = built.store.clone();
    let appeared_slot: PathBuf = built.parent.join("two");
    let listings: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::default();
    let counted: std::sync::Arc<std::sync::atomic::AtomicUsize> = std::sync::Arc::clone(&listings);
    WHEN_THE_STORE_IS_LISTED.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }));
    });
    BETWEEN_THE_TWO_LISTINGS.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            std::fs::create_dir(appeared_slot.as_path())
                .expect("a slot folder appears with the record");
            let slot_text: String = format!(
                "{STORE_HEADER}\n{}\n",
                StoredIdentity::Slot {
                    name: String::from("two"),
                    identity: identity_of(appeared_slot.as_path())
                }
                .encode()
            );
            std::fs::write(appeared.join("two"), slot_text)
                .expect("the slot's record is published in the window");
        }));
    });
    let refusal: AdoptFailure = adopt_at(&request(&built), built.store.as_path(), true, 0)
        .expect_err("a slot appeared between the two listings");
    BETWEEN_THE_TWO_LISTINGS.with(|hook| *hook.borrow_mut() = None);
    WHEN_THE_STORE_IS_LISTED.with(|hook| *hook.borrow_mut() = None);
    assert_eq!(
        listings.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "the adoption lists the folder exactly twice: once for the set it decides on and the staging names it removes, and once before it publishes"
    );
    assert!(
        matches!(refusal, AdoptFailure::Decision(AdoptionRefusal::SlotNotListed { ref slot }) if slot == "two"),
        "{refusal:?}"
    );
    assert!(
        refusal.message().contains("`two`"),
        "the slot is named: {}",
        refusal.message()
    );
    assert_eq!(
        std::fs::read(built.store.join(PARENT_RECORD_NAME)).expect("the parent record's bytes"),
        parent_before,
        "a refused adoption publishes nothing"
    );
}
