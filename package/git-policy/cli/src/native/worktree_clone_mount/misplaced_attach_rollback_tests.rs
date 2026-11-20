//! Controls for the rollback of a misplaced attach:
//! the plan names the mount and the two paths and carries the target descriptor,
//! an identity that proves no mount is refused,
//! and a retained mount is reported with both paths and the administrator's way forward.

use super::*;
use crate::attached_mount_identity::MountIdentity;
use crate::attach_post_check::{MisplacedAttach, MisplacedReason};
use crate::fs_beneath::{identity_of, open_root};
use crate::fs_beneath_mount_facts::{mount_id_of, observed_path};
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// The slot a misplaced attach should have reached,
/// below the fixture's checkout.
fn canonical_slot(root: &Path) -> PathBuf {
    return root.join("checkout/.claude/worktrees/one");
}

/// A misplaced attach with the mount `attached` and the canonical slot below `root`.
fn misplaced(root: &Path, attached: u64) -> MisplacedAttach {
    return MisplacedAttach { reason: MisplacedReason::MountElsewhere { fresh: attached + 1, attached }, attached_mount_id: attached, canonical_slot: canonical_slot(root), fresh: None };
}

/// The held target the attach acted through:
/// the folder the mount landed on,
/// opened as the walk opens it.
fn a_held_target(root: &Path, name: &str) -> std::fs::File {
    let folder: PathBuf = root.join(name);
    std::fs::create_dir_all(&folder).expect("the folder the mount landed on");
    return open_root(std::fs::canonicalize(folder).expect("canonical").as_path()).expect("the held target");
}

/// The mount ID of a folder of the fixture,
/// which is the mount the folders were already on.
fn fixture_mount_id(root: &Path) -> u64 {
    let folder: std::fs::File = open_root(std::fs::canonicalize(root).expect("canonical").as_path()).expect("the fixture root");
    return mount_id_of(&folder).expect("the fixture's mount");
}

/// The mount one report describes,
/// on the fixture's own mount.
fn mount(attached: u64, parent: u64, observed: &Path) -> MountIdentity {
    return MountIdentity {
        mount_id: attached,
        parent_mount_id: parent,
        mount_point: observed.to_path_buf(),
        source: String::from("zpcachyos/nosnap/worktrees/one"),
        file_system_type: String::from("zfs"),
        device: 66304,
        inode: 2,
        observed_path: observed.to_path_buf(),
    };
}

/// What:
///  A plan names the mount the attach made,
///  the path its target descriptor names now and the canonical slot,
///  and it carries the very target descriptor,
///  so an unmount acts on the folder the attach acted on.
/// Why:
///  A plan that held only a path would resolve it again,
///  and one that named no descriptor leaves the proof that the mount is its own to the caller.
#[test]
fn the_plan_names_the_mount_and_carries_the_target_descriptor() {
    let root: PathBuf = fixture("rollback-plan");
    let parent: u64 = fixture_mount_id(root.as_path());
    let attached: u64 = parent + 1;
    let misplaced: MisplacedAttach = misplaced(root.as_path(), attached);
    let target: std::fs::File = a_held_target(root.as_path(), "moved-one");
    let expected: PathBuf = observed_path(&target).expect("the target's path");
    let target_identity: crate::fs_beneath::ObjectId = identity_of(&target).expect("identity");
    let plan: RollbackPlan = plan_rollback(&misplaced, target, parent).expect("a mount with its own number is planned");
    assert_eq!(plan.attached_mount_id, attached);
    assert_eq!(plan.mounted_at, expected, "the report names the path the held descriptor names");
    assert_eq!(plan.canonical_slot, misplaced.canonical_slot);
    assert_eq!(identity_of(&plan.target).expect("identity"), target_identity, "the plan carries the target the attach acted through");
    remove(root.as_path());
}

/// What:
///  A mount number equal to the mount the folders were already on is refused,
///  and so is a target whose descriptor names no path a report can carry.
/// Why:
///  An unmount on a number that proves no attach could take down the folder's own mount,
///  and a report that named no path would send an administrator nowhere.
#[test]
fn the_plan_refuses_an_identity_that_proves_no_mount_and_a_place_it_cannot_name() {
    let root: PathBuf = fixture("rollback-refusal");
    let parent: u64 = fixture_mount_id(root.as_path());
    let misplaced: MisplacedAttach = misplaced(root.as_path(), parent);
    let refused: RollbackRefusal = plan_rollback(&misplaced, a_held_target(root.as_path(), "moved-one"), parent).expect_err("the mount of the folder itself is no attach");
    assert_eq!(refused, RollbackRefusal::NoVerifiedMount { attached_mount_id: parent, parent_mount_id: parent });
    assert!(refused.message().contains("no attach is proved"), "{}", refused.message());
    assert!(refused.message().contains(parent.to_string().as_str()), "{}", refused.message());
    let unnameable: RollbackRefusal = RollbackRefusal::NoPlaceRecorded { attached_mount_id: parent + 1 };
    assert!(unnameable.message().contains(&format!("mount {}", parent + 1)), "{}", unnameable.message());
    assert!(unnameable.message().contains("names no path"), "{}", unnameable.message());
    assert!(unnameable.message().contains(crate::fs_mount_options::MOUNTINFO_PATH), "the table is named: {}", unnameable.message());
    remove(root.as_path());
}

/// What:
///  A completed rollback reports the mount and both paths,
///  and a retained one reports the same facts plus the failure,
///  the refusal to retry and the administrator's way forward.
/// Why:
///  The stranded state is what the acceptance asks a report to show,
///  and an operator must find the mount from the report alone.
#[test]
fn a_rolled_back_and_a_retained_mount_are_both_reported_in_full() {
    let root: PathBuf = fixture("rollback-report");
    let parent: u64 = fixture_mount_id(root.as_path());
    let attached: u64 = parent + 1;
    let misplaced: MisplacedAttach = misplaced(root.as_path(), attached);
    let observed: PathBuf = observed_path(&a_held_target(root.as_path(), "moved-one")).expect("the target's path");
    let rolled: String = RollbackOutcome::RolledBack {
        plan: plan_rollback(&misplaced, a_held_target(root.as_path(), "moved-one"), parent).expect("plan"),
        mount: mount(attached, parent, observed.as_path()),
    }
    .report();
    assert!(rolled.contains(&format!("mount {attached}")), "{rolled}");
    assert!(rolled.contains("moved-one"), "{rolled}");
    assert!(rolled.contains("worktrees/one"), "the slot is named: {rolled}");
    assert!(rolled.contains("nothing of the clone is attached"), "{rolled}");
    let retained: String = RollbackOutcome::Retained {
        plan: plan_rollback(&misplaced, a_held_target(root.as_path(), "moved-one"), parent).expect("plan"),
        mount: mount(attached, parent, observed.as_path()),
        reason: String::from("the holder still holds the mount"),
    }
    .report();
    let mounted: String = format!("mount {attached}");
    for expected in [
        mounted.as_str(),
        "moved-one",
        "worktrees/one",
        "the holder still holds the mount",
        "Nothing else was tried",
        "administrator",
        "zpcachyos/nosnap/worktrees/one",
        "device 66304",
        "inode 2",
    ] {
        assert!(retained.contains(expected), "{expected} is in {retained}");
    }
    assert!(retained.contains("descriptor unmount route"), "the way forward names the descriptor route: {retained}");
    assert!(!retained.contains("--lazy"), "the way forward asks for no lazy detach: {retained}");
    remove(root.as_path());
}
