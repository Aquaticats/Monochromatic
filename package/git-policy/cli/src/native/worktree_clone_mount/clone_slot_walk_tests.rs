//! Controls for the walk from the pinned checkout to a clone slot:
//! both recorded identities are compared field by field,
//! a link or a missing component is refused naming it,
//! the two descriptors of the slot must name one folder,
//! and the second walk reads what the canonical slot holds now.

use super::*;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The store's header,
/// which a fixture store must start with.
const HEADER: &str = "# monochromatic-git-policy-cli worktree clone slot identities, version 1";

/// One fixture checkout with one slot below `.claude/worktrees`,
/// and a store recording what the two folders are now.
struct Fixture {
    /// The checkout.
    checkout: PathBuf,
    /// The store file.
    store: PathBuf,
    /// The `.claude/worktrees` folder.
    parent: PathBuf,
    /// The slot folder.
    slot: PathBuf,
}

/// What the fixture's checkout holds,
/// as the walk and the store read it.
impl Fixture {
    /// The parent folder's identity now.
    fn parent_identity(&self) -> CloneFolderIdentity {
        return CloneFolderIdentity::capture(&std::fs::File::open(&self.parent).expect("parent")).expect("identity");
    }

    /// The slot folder's identity now.
    fn slot_identity(&self) -> CloneFolderIdentity {
        return CloneFolderIdentity::capture(&std::fs::File::open(&self.slot).expect("slot")).expect("identity");
    }

    /// Write a store recording `parent` and `slot` for the slot named `name`.
    fn record(&self, name: &str, parent: &CloneFolderIdentity, slot: &CloneFolderIdentity) {
        let text: String = format!("{HEADER}\nparent - {}\nslot {name} {}\n", parent.encode(), slot.encode());
        std::fs::write(&self.store, text).expect("store");
    }

    /// Record the two folders as they are now.
    fn record_now(&self, name: &str) {
        self.record(name, &self.parent_identity(), &self.slot_identity());
    }
}

/// A fixture checkout named `name`,
/// with `.claude/worktrees/one` below it and a store recording both folders.
fn fixture_slot(name: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let checkout: PathBuf = root.join("checkout");
    let parent: PathBuf = checkout.join(".claude/worktrees");
    let slot: PathBuf = parent.join("one");
    std::fs::create_dir_all(&slot).expect("slot");
    let built: Fixture = Fixture { checkout, store: root.join("slot-identities"), parent, slot };
    built.record_now("one");
    return built;
}

/// The request holding the fixture's slot.
fn request<'a>(built: &'a Fixture, slot: &'a str) -> HoldCloneSlotRequest<'a> {
    return HoldCloneSlotRequest { checkout: built.checkout.as_path(), slot, store: built.store.as_path() };
}

/// Whether `refusal` is the field-level parent refusal naming `field` in its sentence.
fn names_parent_field(refusal: &CloneSlotRefusal, field: &str) -> bool {
    return matches!(refusal, CloneSlotRefusal::ParentIdentity { .. }) && refusal.message().contains(field);
}

/// Whether `refusal` is the field-level slot refusal naming `field` in its sentence.
fn names_slot_field(refusal: &CloneSlotRefusal, field: &str) -> bool {
    return matches!(refusal, CloneSlotRefusal::SlotIdentity { .. }) && refusal.message().contains(field);
}

/// What:
///  A walk whose fixtures match the recorded identities holds every folder,
///  and the slot's two descriptors name one folder.
/// Why:
///  This is the positive control of the whole walk:
///  every later refusal must be shown against a walk that otherwise passes.
#[test]
fn the_walk_holds_each_folder_and_compares_both_records() {
    let built: Fixture = fixture_slot("clone-walk-holds");
    let held: HeldCloneSlot = hold_clone_slot(&request(&built, "one")).expect("the walk holds the slot");
    assert_eq!(held.parent_identity, built.parent_identity());
    assert_eq!(held.slot_identity, built.slot_identity());
    assert_eq!(held.canonical_slot, std::fs::canonicalize(&built.slot).expect("canonical"));
    let checkout: std::fs::Metadata = held.checkout.metadata().expect("checkout");
    let named: std::fs::Metadata = std::fs::metadata(&built.checkout).expect("named");
    assert_eq!((checkout.dev(), checkout.ino()), (named.dev(), named.ino()));
    let target: crate::fs_beneath::ObjectId = identity_of(&held.target).expect("target");
    assert_eq!(target, identity_of(&held.slot).expect("slot"), "the mount target is the folder that was judged");
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A store whose parent record differs in one field is refused,
///  and the sentence names that field.
/// Why:
///  A replaced `.claude/worktrees` is exactly what the record exists for,
///  and each field is proved on its own so no field is compared by accident.
#[test]
fn each_recorded_field_of_the_parent_is_a_refusal() {
    let built: Fixture = fixture_slot("clone-walk-parent-fields");
    let recorded: CloneFolderIdentity = built.parent_identity();
    hold_clone_slot(&request(&built, "one")).expect("positive control");
    let changed: [(&str, CloneFolderIdentity); 6] = [
        ("device", CloneFolderIdentity { device: recorded.device + 1, ..recorded.clone() }),
        ("inode", CloneFolderIdentity { inode: recorded.inode + 1, ..recorded.clone() }),
        ("owner", CloneFolderIdentity { owner: recorded.owner + 1, ..recorded.clone() }),
        ("mode", CloneFolderIdentity { mode: recorded.mode ^ 0o001, ..recorded.clone() }),
        ("access ACL", CloneFolderIdentity { access_acl: Some(vec![2, 0, 0, 0]), ..recorded.clone() }),
        ("default ACL", CloneFolderIdentity { default_acl: Some(vec![2, 0, 0, 0]), ..recorded.clone() }),
    ];
    for (field, identity) in &changed {
        built.record("one", identity, &built.slot_identity());
        let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err(field);
        assert!(names_parent_field(&refused, field), "{field}: {}", refused.message());
        assert!(refused.message().contains("`.claude/worktrees`"), "{}", refused.message());
        assert!(refused.message().contains("Ways forward"), "{}", refused.message());
    }
    built.record_now("one");
    hold_clone_slot(&request(&built, "one")).expect("the record as it was passes again");
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A store whose slot record differs in one field is refused,
///  and the sentence names that field and the slot.
/// Why:
///  A changed identity is not permission to adopt the replacement,
///  and the person must know which field moved.
#[test]
fn each_recorded_field_of_the_slot_is_a_refusal() {
    let built: Fixture = fixture_slot("clone-walk-slot-fields");
    let recorded: CloneFolderIdentity = built.slot_identity();
    let changed: [(&str, CloneFolderIdentity); 6] = [
        ("device", CloneFolderIdentity { device: recorded.device + 1, ..recorded.clone() }),
        ("inode", CloneFolderIdentity { inode: recorded.inode + 1, ..recorded.clone() }),
        ("owner", CloneFolderIdentity { owner: recorded.owner + 1, ..recorded.clone() }),
        ("mode", CloneFolderIdentity { mode: recorded.mode ^ 0o001, ..recorded.clone() }),
        ("access ACL", CloneFolderIdentity { access_acl: Some(vec![2, 0, 0, 0]), ..recorded.clone() }),
        ("default ACL", CloneFolderIdentity { default_acl: Some(vec![2, 0, 0, 0]), ..recorded.clone() }),
    ];
    for (field, identity) in &changed {
        built.record("one", &built.parent_identity(), identity);
        let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err(field);
        assert!(names_slot_field(&refused, field), "{field}: {}", refused.message());
        assert!(refused.message().contains("`one`"), "{}", refused.message());
        assert!(refused.message().contains("never refreshed"), "{}", refused.message());
    }
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A slot with no record asks for the sweep that first verifies a clone,
///  and a store with no parent record asks for the setup run.
/// Why:
///  Neither record is invented by a walk,
///  and each sentence names the missing record.
#[test]
fn an_unrecorded_slot_or_parent_asks_for_the_run_that_writes_it() {
    let built: Fixture = fixture_slot("clone-walk-unrecorded");
    let parent: CloneFolderIdentity = built.parent_identity();
    std::fs::write(&built.store, format!("{HEADER}\nparent - {}\n", parent.encode())).expect("store");
    let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("no slot record");
    assert!(matches!(refused, CloneSlotRefusal::Store(StoreRefusal::NoSlotRecord { .. })), "{refused:?}");
    assert!(refused.message().contains("sweep"), "{}", refused.message());
    std::fs::write(&built.store, format!("{HEADER}\nslot one {}\n", built.slot_identity().encode())).expect("store");
    let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("no parent record");
    assert!(matches!(refused, CloneSlotRefusal::Store(StoreRefusal::NoParentRecord { .. })), "{refused:?}");
    assert!(refused.message().contains("setup"), "{}", refused.message());
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A link planted at `.claude`, at `.claude/worktrees` or at the slot is refused,
///  and the sentence names the component,
///  while the same tree without a link passes.
/// Why:
///  The walk must never resolve a path the account can change,
///  and the refusal must say which folder to look at.
#[test]
fn a_link_planted_in_the_way_is_refused_naming_the_component() {
    let built: Fixture = fixture_slot("clone-walk-link");
    hold_clone_slot(&request(&built, "one")).expect("positive control");
    let elsewhere: PathBuf = built.checkout.parent().expect("root").join("elsewhere");
    std::fs::create_dir_all(elsewhere.join("worktrees/one")).expect("elsewhere tree");
    std::fs::rename(&built.slot, &elsewhere.join("worktrees/one-target")).expect("slot away");
    std::os::unix::fs::symlink(elsewhere.join("worktrees"), &built.slot).expect("slot link");
    let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("a link at the slot");
    assert!(matches!(refused, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::LinkInTheWay, .. })), "{refused:?}");
    assert!(refused.message().contains(".claude/worktrees/one"), "the slot is named: {}", refused.message());
    assert!(refused.message().contains("never follows a link"), "{}", refused.message());
    std::fs::remove_file(&built.slot).expect("link away");
    // A link at the worktrees folder itself, and one at `.claude`.
    std::fs::create_dir(elsewhere.join("claude-worktrees")).expect("elsewhere claude");
    let held_parent: PathBuf = elsewhere.join("claude-worktrees");
    std::fs::rename(&built.parent, &held_parent).expect("parent away");
    std::os::unix::fs::symlink(&held_parent, &built.parent).expect("parent link");
    let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("a link at the parent");
    assert!(matches!(refused, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::LinkInTheWay, .. })), "{refused:?}");
    assert!(refused.message().contains(".claude/worktrees"), "{}", refused.message());
    std::fs::remove_file(&built.parent).expect("link away");
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A component that is not there,
///  a slot name that is no plain name,
///  and a slot that is a file are each refused naming the input,
///  and the same fixture without the break passes.
/// Why:
///  The sentence must name what the person typed or what is missing,
///  not just say that something failed.
#[test]
fn a_missing_component_and_a_name_that_is_no_component_are_refused() {
    let built: Fixture = fixture_slot("clone-walk-missing");
    let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "../escape")).expect_err("no plain name");
    assert!(matches!(refused, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::NotAPlainName, .. })), "{refused:?}");
    assert!(refused.message().contains("../escape"), "the input is named: {}", refused.message());
    let file: PathBuf = built.parent.join("file-slot");
    std::fs::write(&file, b"x").expect("file");
    let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "file-slot")).expect_err("a file is no slot folder");
    assert!(matches!(refused, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::NotADirectory, .. })), "{refused:?}");
    assert!(refused.message().contains("file-slot"), "{}", refused.message());
    std::fs::remove_dir_all(&built.parent).expect("parent away");
    let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("the parent is gone");
    assert!(matches!(refused, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::Missing, .. })), "{refused:?}");
    assert!(refused.message().contains(".claude/worktrees"), "{}", refused.message());
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A folder that takes the slot's name between the reading open and the mount-target open is refused,
///  naming both identities.
/// Why:
///  The attach acts on the target descriptor,
///  so two descriptors that name different folders must stop the walk.
#[test]
fn a_slot_that_changed_between_the_two_opens_is_refused() {
    let built: Fixture = fixture_slot("clone-walk-changed");
    let slot: PathBuf = built.slot.clone();
    let moved: PathBuf = built.parent.join("one-moved");
    AFTER_THE_SLOT_IS_READ.with(|hook| {
        return *hook.borrow_mut() = Some(Box::new(move || {
            std::fs::rename(&slot, &moved).expect("slot away");
            std::fs::create_dir(&slot).expect("a replacement at the name");
        }));
    });
    let refused: Result<HeldCloneSlot, CloneSlotRefusal> = hold_clone_slot(&request(&built, "one"));
    AFTER_THE_SLOT_IS_READ.with(|hook| { return *hook.borrow_mut() = None });
    let refused: CloneSlotRefusal = refused.expect_err("the two descriptors name different folders");
    assert!(matches!(refused, CloneSlotRefusal::TargetAnotherObject { .. }), "{refused:?}");
    assert!(refused.message().contains("`one`"), "{}", refused.message());
    assert!(refused.message().contains("different folders") || refused.message().contains("one names"), "{}", refused.message());
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  The second walk reads the parent's identity,
///  the slot's identity and both mount IDs at the canonical path,
///  and refuses a link in the way as the first walk does.
/// Why:
///  The post-attach check compares this answer with the mount the attach recorded,
///  so the walk must read the canonical slot rather than the held descriptors.
#[test]
fn the_second_walk_reads_the_canonical_slot() {
    let built: Fixture = fixture_slot("clone-walk-again");
    let held: HeldCloneSlot = hold_clone_slot(&request(&built, "one")).expect("held");
    let again: FreshSlotLookup = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: built.checkout.as_path(), slot: "one" }).expect("the slot is there");
    assert_eq!(again.parent_identity, held.parent_identity);
    assert_eq!(again.slot_identity, held.slot_identity);
    assert_eq!(again.observed_path, held.canonical_slot, "the fresh walk reached the canonical slot");
    assert_eq!(again.parent_mount_id, again.mount_id, "both folders are on one mount here");
    assert_eq!(again.mount_id, crate::fs_beneath::mount_id_of(&held.target).expect("the held target's mount"), "the slot is on the mount the held target is on");
    let elsewhere: PathBuf = built.checkout.parent().expect("root").join("elsewhere");
    std::fs::create_dir_all(elsewhere.join("one")).expect("elsewhere");
    std::fs::rename(&built.parent, &elsewhere.join("worktrees")).expect("parent away");
    std::os::unix::fs::symlink(elsewhere.join("worktrees"), &built.parent).expect("parent link");
    let refused: CloneSlotRefusal = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: built.checkout.as_path(), slot: "one" }).expect_err("a link at the parent");
    assert!(matches!(refused, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::LinkInTheWay, .. })), "{refused:?}");
    std::fs::remove_file(&built.parent).expect("link away");
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A checkout path that is not there is refused,
///  naming the checkout,
///  before any store is read.
/// Why:
///  A person who runs the command in a folder that was moved away
///  must be told about the checkout rather than about the store it never reached.
#[test]
fn a_checkout_that_is_not_there_is_refused() {
    let root: PathBuf = fixture("clone-walk-no-checkout");
    let absent: PathBuf = root.join("absent");
    let request: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest { checkout: absent.as_path(), slot: "one", store: Path::new("/nonexistent/store") };
    let refused: CloneSlotRefusal = hold_clone_slot(&request).expect_err("no checkout");
    assert!(matches!(refused, CloneSlotRefusal::Walk(_)), "{refused:?}");
    assert!(refused.message().contains("checkout"), "{}", refused.message());
    remove(root.as_path());
}
