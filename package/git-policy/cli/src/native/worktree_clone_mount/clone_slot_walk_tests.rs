//! Controls for the walk from the pinned checkout to a clone slot:
//! both recorded identities are compared field by field,
//! a link or a missing component is refused naming it,
//! the two descriptors of the slot must name one folder,
//! and the second walk reads what the canonical slot holds now.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{fixture, remove};
use std::os::fd::AsRawFd as _;
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
        let refused_2: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err(field);
        assert!(names_slot_field(&refused_2, field), "{field}: {}", refused_2.message());
        assert!(refused_2.message().contains("`one`"), "{}", refused_2.message());
        assert!(refused_2.message().contains("never refreshed"), "{}", refused_2.message());
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
    let refused_3: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("no slot record");
    assert!(matches!(refused_3, CloneSlotRefusal::Store(StoreRefusal::NoSlotRecord { .. })), "{refused_3:?}");
    assert!(refused_3.message().contains("sweep"), "{}", refused_3.message());
    std::fs::write(&built.store, format!("{HEADER}\nslot one {}\n", built.slot_identity().encode())).expect("store");
    let refused_4: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("no parent record");
    assert!(matches!(refused_4, CloneSlotRefusal::Store(StoreRefusal::NoParentRecord { .. })), "{refused_4:?}");
    assert!(refused_4.message().contains("setup"), "{}", refused_4.message());
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A checkout named through a link,
///  or through a `/proc/self/fd` descriptor,
///  is refused before the walk starts,
///  and the sentence names the canonical path that stands,
///  while the same checkout named by that path passes.
/// Why:
///  Resolving the path would mean walking from whichever folder the account's link names,
///  and the design's first rule is that the mount never resolves a path the account can change.
///  The descriptor path is item 2's measured `openat2` magic-link arm mapped onto this module:
///  its only path input is the checkout,
/// and a magic link there is no canonical path.
#[test]
fn a_checkout_named_through_a_link_is_refused() {
    let built: Fixture = fixture_slot("clone-walk-checkout-link");
    hold_clone_slot(&request(&built, "one")).expect("positive control: the canonical checkout passes");
    let root: PathBuf = built.checkout.parent().expect("root").to_path_buf();
    let linked: PathBuf = root.join("checkout-link");
    std::os::unix::fs::symlink(&built.checkout, &linked).expect("link");
    let refused: CloneSlotRefusal = hold_clone_slot(&HoldCloneSlotRequest { checkout: linked.as_path(), slot: "one", store: built.store.as_path() }).expect_err("a checkout reached through a link");
    assert!(matches!(refused, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::NotTheCanonicalPath, .. })), "{refused:?}");
    assert!(refused.message().contains("not the canonical path"), "{}", refused.message());
    assert!(refused.message().contains(built.checkout.to_string_lossy().as_ref()), "the canonical path that stands is named: {}", refused.message());
    assert!(refused.message().contains("Ways forward"), "{}", refused.message());
    let held: std::fs::File = std::fs::File::open(&built.checkout).expect("the checkout opens");
    let through_a_descriptor: PathBuf = Path::new("/proc/self/fd").join(held.as_raw_fd().to_string());
    let magic: CloneSlotRefusal = hold_clone_slot(&HoldCloneSlotRequest { checkout: through_a_descriptor.as_path(), slot: "one", store: built.store.as_path() }).expect_err("a checkout named through a /proc descriptor");
    assert!(matches!(magic, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::NotTheCanonicalPath, .. })), "{magic:?}");
    assert!(magic.message().contains("not the canonical path"), "{}", magic.message());
    std::fs::remove_file(&linked).expect("link away");
    remove(root.as_path());
}

/// What:
///  A link planted at `.claude`,
///  at `.claude/worktrees` or at the slot is refused,
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
    std::fs::rename(&built.slot, elsewhere.join("worktrees/one-target")).expect("slot away");
    std::os::unix::fs::symlink(elsewhere.join("worktrees"), &built.slot).expect("slot link");
    let refused_5: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("a link at the slot");
    assert!(matches!(refused_5, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::LinkInTheWay, .. })), "{refused_5:?}");
    assert!(refused_5.message().contains(".claude/worktrees/one"), "the slot is named: {}", refused_5.message());
    assert!(refused_5.message().contains("never follows a link"), "{}", refused_5.message());
    std::fs::remove_file(&built.slot).expect("link away");
    // A link at the worktrees folder itself, and one at `.claude`.
    std::fs::create_dir(elsewhere.join("claude-worktrees")).expect("elsewhere claude");
    let held_parent: PathBuf = elsewhere.join("claude-worktrees");
    std::fs::rename(&built.parent, &held_parent).expect("parent away");
    std::os::unix::fs::symlink(&held_parent, &built.parent).expect("parent link");
    let refused_6: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("a link at the parent");
    assert!(matches!(refused_6, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::LinkInTheWay, .. })), "{refused_6:?}");
    assert!(refused_6.message().contains(".claude/worktrees"), "{}", refused_6.message());
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
    let refused_7: CloneSlotRefusal = hold_clone_slot(&request(&built, "../escape")).expect_err("no plain name");
    assert!(matches!(refused_7, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::NotAPlainName, .. })), "{refused_7:?}");
    assert!(refused_7.message().contains("../escape"), "the input is named: {}", refused_7.message());
    // A name no worktree Git adds begins with a dot, so the walk refuses it as the store does.
    let hidden: CloneSlotRefusal = hold_clone_slot(&request(&built, ".hidden")).expect_err("a leading dot is no slot name");
    assert!(matches!(hidden, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::NotAPlainName, .. })), "{hidden:?}");
    assert!(hidden.message().contains(".hidden"), "the input is named: {}", hidden.message());
    assert!(hidden.message().contains("starts with a dot"), "the reason is given: {}", hidden.message());
    let file: PathBuf = built.parent.join("file-slot");
    std::fs::write(&file, b"x").expect("file");
    let refused_8: CloneSlotRefusal = hold_clone_slot(&request(&built, "file-slot")).expect_err("a file is no slot folder");
    assert!(matches!(refused_8, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::NotADirectory, .. })), "{refused_8:?}");
    assert!(refused_8.message().contains("file-slot"), "{}", refused_8.message());
    std::fs::remove_dir_all(&built.parent).expect("parent away");
    let refused_9: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("the parent is gone");
    assert!(matches!(refused_9, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::Missing, .. })), "{refused_9:?}");
    assert!(refused_9.message().contains(".claude/worktrees"), "{}", refused_9.message());
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
    let refused_10: Result<HeldCloneSlot, CloneSlotRefusal> = hold_clone_slot(&request(&built, "one"));
    AFTER_THE_SLOT_IS_READ.with(|hook| { return *hook.borrow_mut() = None });
    let refused_11: CloneSlotRefusal = refused_10.expect_err("the two descriptors name different folders");
    assert!(matches!(refused_11, CloneSlotRefusal::TargetAnotherObject { .. }), "{refused_11:?}");
    assert!(refused_11.message().contains("`one`"), "{}", refused_11.message());
    assert!(refused_11.message().contains("different folders") || refused_11.message().contains("one names"), "{}", refused_11.message());
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
    assert_eq!(again.mount_id, crate::fs_beneath_mount_facts::mount_id_of(&held.target).expect("the held target's mount"), "the slot is on the mount the held target is on");
    let elsewhere: PathBuf = built.checkout.parent().expect("root").join("elsewhere");
    std::fs::create_dir_all(elsewhere.join("one")).expect("elsewhere");
    std::fs::rename(&built.parent, elsewhere.join("worktrees")).expect("parent away");
    std::os::unix::fs::symlink(elsewhere.join("worktrees"), &built.parent).expect("parent link");
    let refused_12: CloneSlotRefusal = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: built.checkout.as_path(), slot: "one" }).expect_err("a link at the parent");
    assert!(matches!(refused_12, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::LinkInTheWay, .. })), "{refused_12:?}");
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
    let refused_13: CloneSlotRefusal = hold_clone_slot(&request).expect_err("no checkout");
    assert!(matches!(refused_13, CloneSlotRefusal::Walk(_)), "{refused_13:?}");
    assert!(refused_13.message().contains("checkout"), "{}", refused_13.message());
    remove(root.as_path());
}

/// Environment variable that makes the control below run its inner half,
/// inside a user namespace of its own,
/// where this run's own files are root's.
#[cfg(target_os = "linux")]
const ROOT_SIDE_VARIABLE: &str = "CLONE_SLOT_WALK_ROOT_SIDE";

/// What:
///  The route's walk answers the store's root control first:
///  a store this account owns refuses before anything is compared,
///  while the walk's own entry holds the same fixture and records the same identities.
/// Why:
///  The store decides which folder a clone may be attached to,
///  so a store the account could rewrite must refuse the route rather than decide it,
///  and the walk's own entry is what a control on fixture files drives.
#[test]
fn the_route_refuses_a_store_this_account_could_rewrite_before_it_walks() {
    let built: Fixture = fixture_slot("clone-walk-root-control");
    hold_clone_slot(&request(&built, "one")).expect("positive control: the walk itself holds the slot");
    let refused: CloneSlotRefusal = hold_clone_slot_under_root_control(&request(&built, "one")).expect_err("this account owns the store");
    assert!(matches!(refused, CloneSlotRefusal::Store(StoreRefusal::NotUnderRootControl { .. })), "{refused:?}");
    assert!(refused.message().contains("not under root control"), "{}", refused.message());
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  With a store root wrote,
///  the route's walk holds the slot and records both identities.
/// Why:
///  The refusal above proves the control;
///  this is the arm the route runs as root,
///  and it needs a user namespace because this account cannot write a root-owned store otherwise.
#[cfg(target_os = "linux")]
#[test]
fn the_route_holds_a_slot_whose_store_root_wrote() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        ROOT_SIDE_VARIABLE,
        "worktree_clone_mount::clone_slot_walk::tests::the_route_holds_a_slot_whose_store_root_wrote",
    ) {
        return;
    }
    let built: Fixture = fixture_slot("clone-walk-root-owned");
    let held: HeldCloneSlot = hold_clone_slot_under_root_control(&request(&built, "one")).expect("a store root wrote is the route's own");
    assert_eq!(held.parent_identity, built.parent_identity());
    assert_eq!(held.slot_identity, built.slot_identity());
    remove(built.checkout.parent().expect("root"));
}

/// What:
///  A slot that is a btrfs subvolume is refused as a folder on another device,
///  naming the slot,
///  while the same tree with a plain folder passes.
/// Why:
///  `RESOLVE_NO_XDEV` refuses a mount point but not a subvolume,
///  and a nested subvolume at the slot is a folder on another device that no other check of the walk sees:
///  without the device compare it was accepted,
///  and the post-attach check would then report every correct
///  attach on it misplaced.
///  A subvolume needs a btrfs fixture root,
///  so the gate's tmpfs root acknowledges the control did not run and the host's btrfs task covers it.
#[test]
fn a_slot_on_another_device_is_refused() {
    let Some(root) = btrfs_fixture("clone-walk-slot-device") else { return };
    let checkout: PathBuf = root.join("checkout");
    let parent: PathBuf = checkout.join(".claude/worktrees");
    std::fs::create_dir_all(parent.as_path()).expect("the worktrees folder");
    let slot: PathBuf = parent.join("one");
    subvolume_create_at(slot.as_path()).expect("a nested subvolume at the slot");
    let built: Fixture = Fixture { checkout, store: root.join("slot-identities"), parent, slot };
    built.record_now("one");
    let refused: CloneSlotRefusal = hold_clone_slot(&request(&built, "one")).expect_err("a slot on another device is not crossed");
    assert!(matches!(refused, CloneSlotRefusal::Walk(WalkRefusal { reason: WalkRefusalReason::BoundaryCrossed, .. })), "{refused:?}");
    assert!(refused.message().contains("one"), "the slot is named: {}", refused.message());
    remove(root.as_path());
}
