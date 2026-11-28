//! What:
//!  Controls of the mounts a shutdown pass may look at:
//!  the container's child at a slot is one,
//!  and the container itself,
//!  a deeper child,
//!  another container's child,
//!  a bind of a folder inside the clone,
//!  a clone mounted elsewhere,
//!  and a name that is no slot are all left alone.
//! Why:
//!  The pass unmounts only the mounts the route could have made this boot,
//!  so a row this decision takes is a kernel call the pass may make and a row
//!  it leaves is one nothing may touch
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  The rows are values,
//!  so no control reads a machine's own table.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! mountsAtSlots([cloneRow(checkout, 'one')], CONTAINER, checkout).map((c) => c.slot); // ['one']
//! ```

use super::{
    CandidateMount,
    mounts_at_slots,
    slot_path_of,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_pass_test_fixture_values::CONTAINER;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use std::path::{
    Path,
    PathBuf,
};

/// One row of a table a control states.
fn row(
    dataset: &str,
    mount_point: &Path,
    root: &Path,
) -> MountedDatasetInstance {
    return MountedDatasetInstance {
        dataset: String::from(dataset),
        mount_point: mount_point.to_path_buf(),
        root: root.to_path_buf(),
    };
}

/// The clone dataset of one slot name.
fn clone_of(slot: &str) -> String {
    return format!("{CONTAINER}/{slot}");
}

/// The checkout one control reads,
/// which is a fixture folder.
fn checkout(name: &str) -> PathBuf {
    return fixture(name);
}

/// The container's child standing at its slot is a mount the pass may look at.
#[test]
fn a_clones_own_mount_at_its_slot_is_one() {
    let checkout: PathBuf = checkout("candidates-at-slot");
    let slot_path: PathBuf = slot_path_of(checkout.as_path(), "one");
    let rows: Vec<MountedDatasetInstance> = vec![
        row(
            clone_of("one").as_str(),
            slot_path.as_path(),
            Path::new("/"),
        ),
        // The checkout's own mount is the container's sibling, not a clone.
        row(
            "zpcachyos/nosnap/git-worktree-checkout",
            checkout.as_path(),
            Path::new("/"),
        ),
    ];
    assert_eq!(
        mounts_at_slots(rows.as_slice(), CONTAINER, checkout.as_path()),
        vec![CandidateMount {
            clone: clone_of("one"),
            slot: String::from("one"),
            slot_path,
        }]
    );
}

/// The container itself is no clone,
/// and neither is a dataset one level deeper.
#[test]
fn the_container_and_its_grandchildren_are_left_alone() {
    let checkout: PathBuf = checkout("candidates-container");
    let slot_path: PathBuf = slot_path_of(checkout.as_path(), "one");
    let rows: Vec<MountedDatasetInstance> = vec![
        row(CONTAINER, slot_path.as_path(), Path::new("/")),
        row(
            format!("{CONTAINER}/one/inner").as_str(),
            slot_path.as_path(),
            Path::new("/"),
        ),
    ];
    assert!(
        mounts_at_slots(rows.as_slice(), CONTAINER, checkout.as_path()).is_empty(),
        "the container and a grandchild are no slot's clone"
    );
}

/// Another container's child at the same slot path is left alone.
#[test]
fn another_containers_child_at_a_slot_path_is_left_alone() {
    let checkout: PathBuf = checkout("candidates-other-container");
    let slot_path: PathBuf = slot_path_of(checkout.as_path(), "one");
    let rows: Vec<MountedDatasetInstance> = vec![row(
        "zpcachyos/nosnap/another-container/one",
        slot_path.as_path(),
        Path::new("/"),
    )];
    assert!(mounts_at_slots(rows.as_slice(), CONTAINER, checkout.as_path()).is_empty());
}

/// A bind of a folder inside the clone shows a deeper root and is left alone.
#[test]
fn a_bind_of_a_folder_inside_the_clone_is_left_alone() {
    let checkout: PathBuf = checkout("candidates-bind");
    let slot_path: PathBuf = slot_path_of(checkout.as_path(), "one");
    let rows: Vec<MountedDatasetInstance> = vec![row(
        clone_of("one").as_str(),
        slot_path.as_path(),
        Path::new("/src"),
    )];
    assert!(
        mounts_at_slots(rows.as_slice(), CONTAINER, checkout.as_path()).is_empty(),
        "a bind of a folder inside the clone is not the clone's own mount"
    );
}

/// A clone mounted anywhere but at its slot is left alone.
#[test]
fn a_clone_mounted_elsewhere_is_left_alone() {
    let checkout: PathBuf = checkout("candidates-elsewhere");
    let elsewhere: PathBuf = checkout.join("somewhere-else");
    let rows: Vec<MountedDatasetInstance> = vec![row(
        clone_of("one").as_str(),
        elsewhere.as_path(),
        Path::new("/"),
    )];
    assert!(mounts_at_slots(rows.as_slice(), CONTAINER, checkout.as_path()).is_empty());
}

/// A name the route refuses as a slot is left alone,
/// even though its dataset and path agree.
#[test]
fn a_name_that_is_no_slot_is_left_alone() {
    let checkout: PathBuf = checkout("candidates-bad-name");
    let bad: &str = "one/../two";
    let slot_path: PathBuf = checkout.join(".claude").join("worktrees").join("two");
    let rows: Vec<MountedDatasetInstance> = vec![row(
        format!("{CONTAINER}/{bad}").as_str(),
        slot_path.as_path(),
        Path::new("/"),
    )];
    assert!(
        mounts_at_slots(rows.as_slice(), CONTAINER, checkout.as_path()).is_empty(),
        "a clone whose name is no slot is not a slot's clone"
    );
}

/// Every candidate keeps the kernel's order.
#[test]
fn the_candidates_keep_the_kernels_order() {
    let checkout: PathBuf = checkout("candidates-order");
    let rows: Vec<MountedDatasetInstance> = vec![
        row(
            clone_of("three").as_str(),
            slot_path_of(checkout.as_path(), "three").as_path(),
            Path::new("/"),
        ),
        row(
            clone_of("one").as_str(),
            slot_path_of(checkout.as_path(), "one").as_path(),
            Path::new("/"),
        ),
    ];
    let slots: Vec<String> = mounts_at_slots(rows.as_slice(), CONTAINER, checkout.as_path())
        .iter()
        .map(|candidate| return candidate.slot.clone())
        .collect();
    assert_eq!(slots, vec![String::from("three"), String::from("one")]);
}

/// The slot's path is the checkout's folder and the slot's name,
/// whatever the row says.
#[test]
fn the_slot_path_is_the_checkouts_folder_and_the_slots_name() {
    assert_eq!(
        slot_path_of(Path::new("/home/user/Mono"), "one"),
        PathBuf::from("/home/user/Mono/.claude/worktrees/one")
    );
}
