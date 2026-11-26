//! Controls for the route's gate.

use super::{
    CheckoutMount,
    RouteFacts,
    decide_the_routes_add,
    destination_is_the_routes_slot,
    the_routes_add,
};
use crate::worktree_identity::WorktreeIdentity;
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountHoldingAPath,
    MountedDatasetInstance,
    MountedDatasets,
};
use std::path::{
    Path,
    PathBuf,
};

/// A main checkout at `/home/user/Monochromatic`.
fn a_main_checkout() -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: PathBuf::from("/home/user/Monochromatic/.git"),
        git_dir: PathBuf::from("/home/user/Monochromatic/.git"),
        worktree_root: PathBuf::from("/home/user/Monochromatic"),
    };
}

/// The design's checkout dataset.
fn the_designs_checkout() -> CheckoutMount {
    return CheckoutMount::TheDesigns {
        dataset: String::from("tank/ROOT/home/monochromatic-git-checkout"),
    };
}

/// The destination an add at the named slot uses.
fn a_slot(name: &str) -> PathBuf {
    return PathBuf::from(format!("/home/user/Monochromatic/.claude/worktrees/{name}"));
}

#[test]
fn the_designs_dataset_and_slot_shape_is_the_routes() {
    let facts: RouteFacts = decide_the_routes_add(
        &a_main_checkout(),
        a_slot("add-cache-layer").as_path(),
        &the_designs_checkout(),
    )
    .expect("the design's own shape serves")
    .expect("the route serves");
    assert_eq!(facts.slot, "add-cache-layer");
    assert_eq!(
        facts.checkout_dataset,
        "tank/ROOT/home/monochromatic-git-checkout"
    );
    assert_eq!(
        facts.container_dataset,
        "tank/ROOT/home/monochromatic-git-worktree-clone-datasets"
    );
    assert_eq!(facts.pool, "tank");
    assert_eq!(
        facts.clone_dataset(),
        "tank/ROOT/home/monochromatic-git-worktree-clone-datasets/add-cache-layer"
    );
    assert_eq!(
        facts.worktrees_parent(),
        PathBuf::from("/home/user/Monochromatic/.claude/worktrees")
    );
}

#[test]
fn every_other_filesystem_or_dataset_keeps_the_ordinary_add() {
    let cases: [CheckoutMount; 2] = [CheckoutMount::NotZfs, CheckoutMount::OtherDataset];
    for checkout in cases {
        assert_eq!(
            decide_the_routes_add(&a_main_checkout(), a_slot("topic").as_path(), &checkout)
                .expect("nothing refuses"),
            None,
            "{checkout:?} is no route source"
        );
    }
}

#[test]
fn a_linked_source_or_a_destination_elsewhere_is_no_route_add() {
    let linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: PathBuf::from("/home/user/Monochromatic/.git"),
        git_dir: PathBuf::from("/home/user/Monochromatic/.git/worktrees/topic"),
        worktree_root: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/topic"),
    };
    assert_eq!(
        decide_the_routes_add(
            &linked,
            a_slot("another").as_path(),
            &the_designs_checkout()
        )
        .expect("nothing refuses"),
        None
    );
    let elsewhere: PathBuf = PathBuf::from("/home/user/Monochromatic/scratch/topic");
    assert_eq!(
        decide_the_routes_add(
            &a_main_checkout(),
            elsewhere.as_path(),
            &the_designs_checkout()
        )
        .expect("nothing refuses"),
        None
    );
    let one_deeper: PathBuf = PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one/two");
    assert_eq!(
        decide_the_routes_add(
            &a_main_checkout(),
            one_deeper.as_path(),
            &the_designs_checkout()
        )
        .expect("nothing refuses"),
        None
    );
}

#[test]
fn a_slot_name_the_route_cannot_key_refuses_with_a_way_forward() {
    let bad: PathBuf = PathBuf::from("/home/user/Monochromatic/.claude/worktrees/.hidden");
    let refusal: String =
        decide_the_routes_add(&a_main_checkout(), bad.as_path(), &the_designs_checkout())
            .expect_err("a leading dot is no slot name");
    assert!(refusal.contains("cannot key a record"), "{refusal}");
    assert!(refusal.contains("Ways forward:"), "{refusal}");
}

#[test]
fn the_slot_shape_is_the_worktrees_folder() {
    assert!(destination_is_the_routes_slot(
        Path::new("/home/user/Monochromatic"),
        Path::new("/home/user/Monochromatic/.claude/worktrees/topic")
    ));
    assert!(!destination_is_the_routes_slot(
        Path::new("/home/user/Monochromatic"),
        Path::new("/home/user/Monochromatic/.claude/worktrees")
    ));
    assert!(!destination_is_the_routes_slot(
        Path::new("/home/user/Monochromatic"),
        Path::new("/home/user/other/topic")
    ));
}

/// A table that answers one mount for the checkout and nothing else.
struct OneMount {
    /// The dataset the checkout is on.
    dataset: String,
    /// The checkout's path.
    root: PathBuf,
}

impl MountedDatasets for OneMount {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        return Ok(vec![MountedDatasetInstance {
            dataset: self.dataset.clone(),
            mount_point: self.root.clone(),
            root: PathBuf::from("/"),
        }]);
    }

    fn mount_holding(
        &self,
        path: &Path,
    ) -> Result<Option<MountHoldingAPath>, String> {
        return Ok(Some(MountHoldingAPath {
            dataset: if path.starts_with(self.root.as_path()) {
                Some(self.dataset.clone())
            } else {
                None
            },
            filesystem_type: String::from("zfs"),
            mount_point: self.root.clone(),
        }));
    }
}

#[test]
fn the_host_reads_reach_the_same_decision() {
    // The gate's own host reads need a real ZFS mount, so this control makes
    // the one answer a host test can make: a checkout on this test machine's
    // filesystem is not ZFS, and the route leaves the add to Git.
    let mounts: OneMount = OneMount {
        dataset: String::from("tank/ROOT/home/monochromatic-git-checkout"),
        root: PathBuf::from("/nonexistent-checkout-of-the-control"),
    };
    assert_eq!(
        the_routes_add(&a_main_checkout(), a_slot("topic").as_path(), &mounts)
            .expect("a tmpfs or ext4 checkout answers nothing"),
        None,
        "the control's own checkout is not on ZFS, so the route does not serve it"
    );
}

/// A mount table no control may read: asking it for one fails the test.
/// Why:
///  A destination outside the route's own slot is no add this route serves,
///  and a table that fails when read proves the gate decided that before it
///  read the machine.
struct ATableNoControlReads;

impl MountedDatasets for ATableNoControlReads {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        panic!("the gate read the mount table before it compared the destination's shape");
    }

    fn mount_holding(
        &self,
        _path: &Path,
    ) -> Result<Option<MountHoldingAPath>, String> {
        panic!(
            "the gate read the mount at the destination before it compared the destination's shape"
        );
    }
}

/// What:
///  An add whose destination is outside the route's slot answers nothing
///  without a `statfs` and without reading the mount table.
/// Why:
///  Every add at any other path is this case,
///  and the checkout's filesystem kind costs a syscall that no destination
///  outside the route can need.
///  On a fixture root that is no ZFS dataset this control passes without
///  exercising the order,
///  because the filesystem kind answers first in that arm;
///  on this host's ZFS root it fails before the fix.
#[test]
fn an_add_outside_the_routes_slot_reads_no_mount_table() {
    let root: PathBuf = crate::test_support::fixture("route-gate-outside-the-slot");
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: root.join(".git"),
        git_dir: root.join(".git"),
        worktree_root: root.clone(),
    };
    let destination: PathBuf = root.join("somewhere").join("topic");
    assert_eq!(
        the_routes_add(&identity, destination.as_path(), &ATableNoControlReads)
            .expect("a destination outside the route answers nothing"),
        None,
        "an add outside the route's slot is no add of this route"
    );
    crate::test_support::remove(root.as_path());
}
