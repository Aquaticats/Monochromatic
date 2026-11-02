//! Controls for the ladder's before and after steps: the destination subvolume is made when the plan names one on
//!  btrfs, settled away when Git leaves it empty, and nothing happens without a plan.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::test_support::remove;

/// Without a plan the ladder touches nothing.
#[test]
fn btrfs_no_plan_means_no_step() {
    let Some(root) = btrfs_fixture("worktree-ladder-none") else { return };
    let ladder: Ladder = begin_ladder(None);
    ladder.finish();
    assert_eq!(std::fs::read_dir(root.as_path()).expect("listing").count(), 0);
    remove(root.as_path());
}

/// A plan on btrfs makes the destination an empty subvolume before Git, and finishing removes it while it is empty.
#[test]
fn btrfs_a_plan_makes_the_destination_a_subvolume_that_finish_removes_when_empty() {
    let Some(root) = btrfs_fixture("worktree-ladder-plan") else { return };
    let plan: AddPlan = AddPlan { destination: root.join("topic"), insert_at: 2 };
    let ladder: Ladder = begin_ladder(Some(&plan));
    assert!(is_subvolume_root(plan.destination.as_path()).expect("probe"));
    ladder.finish();
    assert!(!plan.destination.exists());
    remove(root.as_path());
}

/// A destination that Git filled stays after finishing.
#[test]
fn btrfs_finish_keeps_a_filled_destination() {
    let Some(root) = btrfs_fixture("worktree-ladder-filled") else { return };
    let plan: AddPlan = AddPlan { destination: root.join("topic"), insert_at: 2 };
    let ladder: Ladder = begin_ladder(Some(&plan));
    std::fs::write(plan.destination.join("file"), b"x").expect("filled");
    ladder.finish();
    assert!(plan.destination.join("file").is_file());
    remove(root.as_path());
}
