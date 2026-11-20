//! Controls for the rollback of a misplaced attach:
//! the plan names the mount and the two paths,
//! an identity that proves no mount is refused,
//! and a retained mount is reported with both paths and the administrator's way forward.

use super::*;
use crate::attached_mount_identity::MountIdentity;
use crate::attach_post_check::{MisplacedAttach, MisplacedReason};
use std::path::{Path, PathBuf};

/// A misplaced attach with the mount `attached` and the slot `canonical`.
fn misplaced(attached: u64, canonical: &str) -> MisplacedAttach {
    return MisplacedAttach { reason: MisplacedReason::MountElsewhere { fresh: 999, attached }, attached_mount_id: attached, canonical_slot: PathBuf::from(canonical), fresh: None };
}

/// The mount one report describes.
fn mount(attached: u64, observed: &str) -> MountIdentity {
    return MountIdentity {
        mount_id: attached,
        parent_mount_id: 40,
        mount_point: PathBuf::from("/gate-fixtures/checkout/.claude/worktrees/one"),
        source: String::from("zpcachyos/nosnap/worktrees/one"),
        file_system_type: String::from("zfs"),
        device: 66304,
        inode: 2,
        observed_path: PathBuf::from(observed),
    };
}

/// What:
///  A plan names the mount the attach made and both paths,
///  and a mount number equal to the folder's own mount is refused.
/// Why:
///  An unmount on a number that proves no attach could take down the folder's own mount,
///  so the plan must refuse it.
#[test]
fn the_plan_names_the_mount_and_refuses_an_identity_that_proves_no_mount() {
    let misplaced: MisplacedAttach = misplaced(41, "/gate-fixtures/checkout/.claude/worktrees/one");
    let plan: RollbackPlan = plan_rollback(&misplaced, Path::new("/gate-fixtures/checkout/.claude/worktrees/moved-one"), 40).expect("a mount with its own number is planned");
    assert_eq!(plan.attached_mount_id, 41);
    assert_eq!(plan.mounted_at, PathBuf::from("/gate-fixtures/checkout/.claude/worktrees/moved-one"));
    assert_eq!(plan.canonical_slot, misplaced.canonical_slot);
    let refused: RollbackRefusal = plan_rollback(&misplaced, Path::new("/gate-fixtures/checkout/.claude/worktrees/one"), 41).expect_err("the mount of the folder itself is no attach");
    assert_eq!(refused, RollbackRefusal::NoVerifiedMount { attached_mount_id: 41, parent_mount_id: 41 });
    assert!(refused.message().contains("no attach is proved"), "{}", refused.message());
    assert!(refused.message().contains("41"), "{}", refused.message());
    for no_path in [Path::new(""), Path::new("relative/path")] {
        let refused_2: RollbackRefusal = plan_rollback(&misplaced, no_path, 40).expect_err("a place a report cannot name");
        assert!(matches!(refused_2, RollbackRefusal::NoPlaceRecorded { attached_mount_id: 41 }), "{refused_2:?}");
        assert!(refused_2.message().contains("mount 41"), "{}", refused_2.message());
    }
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
    let misplaced: MisplacedAttach = misplaced(41, "/gate-fixtures/checkout/.claude/worktrees/one");
    let plan: RollbackPlan = plan_rollback(&misplaced, Path::new("/gate-fixtures/checkout/.claude/worktrees/moved-one"), 40).expect("plan");
    let mount: MountIdentity = mount(41, "/gate-fixtures/checkout/.claude/worktrees/moved-one");
    let rolled: String = RollbackOutcome::RolledBack { plan: plan.clone(), mount: mount.clone() }.report();
    assert!(rolled.contains("mount 41"), "{rolled}");
    assert!(rolled.contains("moved-one"), "{rolled}");
    assert!(rolled.contains("worktrees/one"), "the slot is named: {rolled}");
    assert!(rolled.contains("nothing of the clone is attached"), "{rolled}");
    let retained: String = RollbackOutcome::Retained { plan, mount, reason: String::from("the holder still holds the mount") }.report();
    for expected in ["mount 41", "moved-one", "worktrees/one", "the holder still holds the mount", "Nothing else was tried", "administrator", "zpcachyos/nosnap/worktrees/one", "device 66304", "inode 2"] {
        assert!(retained.contains(expected), "{expected} is in {retained}");
    }
    assert!(retained.contains("descriptor unmount route"), "the way forward names the descriptor route: {retained}");
    assert!(!retained.contains("--lazy"), "the way forward asks for no lazy detach: {retained}");
}
