//! Controls for the post-attach check and the mount facts:
//! the mount the attach recorded is the one the canonical slot shows,
//! and each way a mount can land elsewhere is refused naming its cause.

use super::*;
use crate::attached_mount_identity::{ATTACH_MOUNT_FLAGS, MountIdentity, mount_facts_of, mount_identity_of};
use crate::clone_folder_identity::CloneFolderIdentity;
use crate::clone_slot_walk::{HeldCloneSlot, HoldCloneSlotRequest, LookAgainAtSlotRequest, hold_clone_slot, look_again_at_the_slot};
use crate::fs_beneath::mount_id_of;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The store's header,
/// which the fixture store must start with.
const HEADER: &str = "# monochromatic-git-policy-cli worktree clone slot identities, version 1";

/// A fixture checkout with one slot recorded,
/// and the walk's held handles.
fn held_slot(name: &str) -> (PathBuf, HeldCloneSlot, PathBuf) {
    let root: PathBuf = fixture(name);
    let checkout: PathBuf = root.join("checkout");
    let slot: PathBuf = checkout.join(".claude/worktrees/one");
    std::fs::create_dir_all(&slot).expect("slot");
    let parent: CloneFolderIdentity = CloneFolderIdentity::capture(&std::fs::File::open(checkout.join(".claude/worktrees")).expect("parent")).expect("identity");
    let recorded: CloneFolderIdentity = CloneFolderIdentity::capture(&std::fs::File::open(&slot).expect("slot")).expect("identity");
    let store: PathBuf = root.join("slot-identities");
    std::fs::write(&store, format!("{HEADER}\nparent - {}\nslot one {}\n", parent.encode(), recorded.encode())).expect("store");
    let held: HeldCloneSlot = hold_clone_slot(&HoldCloneSlotRequest { checkout: checkout.as_path(), slot: "one", store: store.as_path() }).expect("held");
    return (root, held, checkout);
}

/// What:
///  With the mount the attach made at the canonical slot,
///  the check answers that the mount is the one the attach holds,
///  and records the mount's own facts.
/// Why:
///  This is the positive control:
///  every misplaced arm below is a refusal against a check that otherwise passes.
#[test]
fn the_mount_the_attach_made_is_the_one_the_slot_shows() {
    let (root, held, checkout): (PathBuf, HeldCloneSlot, PathBuf) = held_slot("post-check-attached");
    let fresh: FreshSlotLookup = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: checkout.as_path(), slot: "one" }).expect("the slot is there");
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id: fresh.parent_mount_id,
        attached_mount_id: fresh.mount_id,
        canonical_slot: held.canonical_slot.as_path(),
    };
    let outcome: AttachOutcome = post_attach_check(&check, Ok(fresh.clone())).expect("the mount table is readable");
    match outcome {
        AttachOutcome::Attached { mount, parent_identity, observed_path } => {
            assert_eq!(mount.mount_id, fresh.mount_id);
            assert_eq!(mount.parent_mount_id, fresh.parent_mount_id);
            assert_eq!((mount.device, mount.inode), (fresh.slot_identity.device, fresh.slot_identity.inode));
            assert_eq!(observed_path, held.canonical_slot);
            assert_eq!(parent_identity, held.parent_identity);
            assert!(!mount.file_system_type.is_empty(), "the recorded mount names its type: {}", mount.describe());
            assert!(!mount.mount_point.as_os_str().is_empty(), "{}", mount.describe());
            assert!(!mount.source.is_empty(), "{}", mount.describe());
        }
        other => panic!("the mount at the slot is the attach's own: {other:?}"),
    }
    remove(root.as_path());
}

/// What:
///  A mount at the canonical slot whose number is not the attach's is misplaced,
///  and the sentence names both numbers.
/// Why:
///  Another mount at the slot is not this attach,
///  and a person must see which mount is where.
#[test]
fn another_mount_at_the_slot_is_misplaced() {
    let (root, held, checkout): (PathBuf, HeldCloneSlot, PathBuf) = held_slot("post-check-elsewhere");
    let fresh: FreshSlotLookup = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: checkout.as_path(), slot: "one" }).expect("the slot is there");
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id: fresh.parent_mount_id,
        attached_mount_id: fresh.mount_id + 1,
        canonical_slot: held.canonical_slot.as_path(),
    };
    let outcome: AttachOutcome = post_attach_check(&check, Ok(fresh.clone())).expect("the mount table is readable");
    match outcome {
        AttachOutcome::Misplaced(misplaced) => {
            assert!(matches!(misplaced.reason, MisplacedReason::MountElsewhere { fresh: found, attached } if found == fresh.mount_id && attached == fresh.mount_id + 1), "{misplaced:?}");
            let message: String = misplaced.reason.message();
            assert!(message.contains(fresh.mount_id.to_string().as_str()) && message.contains((fresh.mount_id + 1).to_string().as_str()), "{message}");
            assert_eq!(misplaced.fresh, Some(fresh), "what the walk found is kept for the report");
        }
        other => panic!("a mount number the attach did not make is misplaced: {other:?}"),
    }
    remove(root.as_path());
}

/// What:
///  A slot whose mount hangs on another parent mount is misplaced,
///  and a parent folder replaced after the walk is misplaced naming the field.
/// Why:
///  The mount's parent is what tells the handler's attach from an unrelated bind,
///  and a replaced parent folder is what a fresh walk catches.
#[test]
fn another_parent_mount_and_a_replaced_parent_are_misplaced() {
    let (root, held, checkout): (PathBuf, HeldCloneSlot, PathBuf) = held_slot("post-check-parent");
    let fresh: FreshSlotLookup = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: checkout.as_path(), slot: "one" }).expect("the slot is there");
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id: fresh.parent_mount_id + 1,
        attached_mount_id: fresh.mount_id,
        canonical_slot: held.canonical_slot.as_path(),
    };
    let outcome: AttachOutcome = post_attach_check(&check, Ok(fresh.clone())).expect("the mount table is readable");
    assert!(matches!(outcome, AttachOutcome::Misplaced(MisplacedAttach { reason: MisplacedReason::OtherParentMount { .. }, .. })), "{outcome:?}");
    // The account renames `.claude` away and puts another tree at the canonical path.
    let old_parent: PathBuf = checkout.join(".claude-old");
    std::fs::rename(checkout.join(".claude"), &old_parent).expect("claude away");
    std::fs::create_dir_all(checkout.join(".claude/worktrees/one")).expect("replacement tree");
    let fresh_after: FreshSlotLookup = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: checkout.as_path(), slot: "one" }).expect("the replacement is walkable");
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id: fresh.parent_mount_id,
        attached_mount_id: fresh.mount_id,
        canonical_slot: held.canonical_slot.as_path(),
    };
    let outcome: AttachOutcome = post_attach_check(&check, Ok(fresh_after)).expect("the mount table is readable");
    match outcome {
        AttachOutcome::Misplaced(misplaced) => {
            assert!(matches!(misplaced.reason, MisplacedReason::ParentChanged { ref fields } if fields.contains(&CloneFolderField::Inode)), "{misplaced:?}");
            assert!(misplaced.reason.message().contains("inode"), "{}", misplaced.reason.message());
        }
        other => panic!("a replaced parent folder is misplaced: {other:?}"),
    }
    remove(root.as_path());
}

/// What:
///  A fresh walk that stops,
///  a slot descriptor that names another path,
///  and a mount table read that fails are each reported as their own cause.
/// Why:
///  A report that only said "misplaced" would leave a person without the path or the number to inspect.
#[test]
fn a_stopped_walk_a_moved_path_and_an_unreadable_table_are_reported() {
    let (root, held, checkout): (PathBuf, HeldCloneSlot, PathBuf) = held_slot("post-check-stopped");
    let fresh: FreshSlotLookup = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: checkout.as_path(), slot: "one" }).expect("the slot is there");
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id: fresh.parent_mount_id,
        attached_mount_id: fresh.mount_id,
        canonical_slot: held.canonical_slot.as_path(),
    };
    let walked_away: CloneSlotRefusal = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: root.as_path(), slot: "one" }).expect_err("the checkout has no slot");
    let outcome: AttachOutcome = post_attach_check(&check, Err(walked_away)).expect("no table is read when the walk stopped");
    match outcome {
        AttachOutcome::Misplaced(misplaced) => {
            assert!(matches!(misplaced.reason, MisplacedReason::WalkRefused { .. }), "{misplaced:?}");
            assert_eq!(misplaced.fresh, None);
            assert!(misplaced.reason.message().contains("could not be walked"), "{}", misplaced.reason.message());
        }
        other => panic!("a stopped walk is misplaced: {other:?}"),
    }
    let mut moved: FreshSlotLookup = fresh.clone();
    moved.observed_path = root.join("elsewhere");
    let outcome: AttachOutcome = post_attach_check(&check, Ok(moved)).expect("the table is readable");
    assert!(matches!(outcome, AttachOutcome::Misplaced(MisplacedAttach { reason: MisplacedReason::ObservedPathMoved { .. }, .. })), "{outcome:?}");
    let unknown: u64 = u64::MAX;
    let refused: MountIdentityRefusal = mount_facts_of(unknown).expect_err("no mount has that number");
    assert!(matches!(refused, MountIdentityRefusal::NoMountEntry { mount_id } if mount_id == unknown), "{refused:?}");
    assert!(refused.message().contains(crate::fs_mount_options::MOUNTINFO_PATH), "{}", refused.message());
    remove(root.as_path());
}

/// What:
///  The flags the attach enforces are exactly `nosuid` and `nodev`,
///  and a folder's mount identity reads its mount,
///  its parent mount,
///  its device and inode and the path its descriptor names.
/// Why:
///  The measured setuid control is what these flags exist for,
///  and the recorded mount is what a later unmount compares against.
#[test]
fn the_attach_flags_and_a_folder_mount_identity_are_recorded() {
    assert_eq!(ATTACH_MOUNT_FLAGS, libc::MS_NOSUID | libc::MS_NODEV, "the attach enforces both flags");
    let (root, held, _checkout): (PathBuf, HeldCloneSlot, PathBuf) = held_slot("post-check-mount");
    let mount: MountIdentity = mount_identity_of(&held.slot).expect("the slot is on a mount");
    let named: std::fs::Metadata = held.slot.metadata().expect("metadata");
    assert_eq!((mount.device, mount.inode), (named.dev(), named.ino()));
    assert_eq!(mount.mount_id, mount_id_of(&held.slot).expect("the descriptor's mount"));
    assert_eq!(mount.observed_path, std::fs::canonicalize(&mount.observed_path).expect("canonical"));
    let facts = mount_facts_of(mount.mount_id).expect("the table lists it");
    assert_eq!(facts.parent_id, mount.parent_mount_id);
    assert!(format!("{}", mount.describe()).contains(mount.mount_id.to_string().as_str()));
    remove(root.as_path());
}
