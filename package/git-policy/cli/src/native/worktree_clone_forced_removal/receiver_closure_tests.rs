//! Controls for the mounts an unmount reaches:
//!  a nested-in-nested receiver is in the closure,
//!  a parent that shares with a group of its own keeps the host's own copy out,
//!  a peer shape names the host and leaves way 8 unusable,
//!  a member with no mount at the slot is not a receiver,
//!  a covered member is named, and a parent that shares with nothing takes way 7.

use crate::worktree_clone_forced_removal::receiver_closure::*;

/// The dataset the clone's own mount carries.
const DATASET: &str = "tank/one";

/// One mount row,
/// with the tags the closure reads.
fn row(mount_id: u32, parent_id: u32, mount_point: &str, shared: &[u32], master: Option<u32>) -> MountRow {
    return MountRow {
        mount_id,
        parent_id,
        mount_point: String::from(mount_point),
        root: String::from("/"),
        filesystem: String::from("ext4"),
        source: String::from("/dev/vda1"),
        shared: shared.to_vec(),
        master,
    };
}

/// The clone's own mount at a slot,
/// which is the copy the act is about.
fn the_copy(mount_id: u32, parent_id: u32, mount_point: &str, shared: &[u32], master: Option<u32>) -> MountRow {
    return MountRow {
        mount_id,
        parent_id,
        mount_point: String::from(mount_point),
        root: String::from("/"),
        filesystem: String::from("zfs"),
        source: String::from(DATASET),
        shared: shared.to_vec(),
        master,
    };
}

/// One namespace: its name, its rows and the topmost mount at the slot.
fn table(namespace: &str, rows: Vec<MountRow>, at_the_slot: Option<MountRow>) -> NamespaceTable {
    return NamespaceTable { namespace: String::from(namespace), rows, at_the_slot };
}

/// A nested-in-nested receiver is in the closure,
/// because the walk descends into each receiver's slaves.
#[test]
fn a_nested_in_nested_receiver_is_in_the_closure() {
    let acting: NamespaceTable = table(
        "mnt:u",
        vec![row(10, 1, "/checkout", &[7], None), the_copy(11, 10, "/checkout/one", &[], None)],
        Some(the_copy(11, 10, "/checkout/one", &[], None)),
    );
    let inner: NamespaceTable = table(
        "mnt:v",
        vec![row(20, 1, "/checkout", &[], Some(7)), the_copy(21, 20, "/checkout/one", &[], None)],
        Some(the_copy(21, 20, "/checkout/one", &[], None)),
    );
    let copy: MountRow = the_copy(11, 10, "/checkout/one", &[], None);
    let handled: Vec<String> = vec![String::from("mnt:u"), String::from("mnt:v")];
    let tables: [NamespaceTable; 2] = [acting.clone(), inner];
    let plan: ReceiverSet = plan_the_receivers(&acting, &copy, &tables, DATASET, handled.as_slice());
    assert_eq!(plan.seed, vec![7]);
    let named: Vec<&str> = plan.receivers.iter().map(|receiver| return receiver.namespace.as_str()).collect();
    assert_eq!(named, vec!["mnt:u", "mnt:v"]);
    assert!(plan.outside.is_empty());
    assert!(plan.way_eight);
}

/// A peer shape names the host: the host's copy is a receiver outside the act,
/// so way 8 is not used there.
#[test]
fn a_peer_shape_leaves_way_eight_unusable() {
    let acting: NamespaceTable = table(
        "mnt:u",
        vec![row(10, 1, "/checkout", &[7], None), the_copy(11, 10, "/checkout/one", &[9], None)],
        Some(the_copy(11, 10, "/checkout/one", &[9], None)),
    );
    let host: NamespaceTable = table(
        "mnt:[4026531837]",
        vec![row(1, 0, "/", &[], None), row(2, 1, "/checkout", &[7], None), the_copy(3, 2, "/checkout/one", &[9], None)],
        Some(the_copy(3, 2, "/checkout/one", &[9], None)),
    );
    let copy: MountRow = the_copy(11, 10, "/checkout/one", &[9], None);
    let handled: Vec<String> = vec![String::from("mnt:u")];
    let tables: [NamespaceTable; 2] = [acting.clone(), host];
    let plan: ReceiverSet = plan_the_receivers(&acting, &copy, &tables, DATASET, handled.as_slice());
    assert_eq!(plan.outside, vec![String::from("mnt:[4026531837]")]);
    assert!(!plan.way_eight);
}

/// A member with no mount at the slot is not a receiver,
/// and a member whose own mount covers the copy is named as covered.
#[test]
fn only_a_mount_at_the_slot_counts_and_a_cover_is_named() {
    let acting: NamespaceTable = table(
        "mnt:u",
        vec![row(10, 1, "/checkout", &[7], None), the_copy(11, 10, "/checkout/one", &[], None)],
        Some(the_copy(11, 10, "/checkout/one", &[], None)),
    );
    let without_a_copy: NamespaceTable =
        table("mnt:w", vec![row(30, 1, "/checkout", &[], Some(7))], None);
    let covered: NamespaceTable = table(
        "mnt:x",
        vec![row(40, 1, "/checkout", &[], Some(7)), row(41, 40, "/checkout/one", &[], None)],
        Some(row(41, 40, "/checkout/one", &[], None)),
    );
    let copy: MountRow = the_copy(11, 10, "/checkout/one", &[], None);
    let handled: Vec<String> = vec![String::from("mnt:u"), String::from("mnt:w"), String::from("mnt:x")];
    let tables: [NamespaceTable; 3] = [acting.clone(), without_a_copy, covered];
    let plan: ReceiverSet = plan_the_receivers(&acting, &copy, &tables, DATASET, handled.as_slice());
    let named: Vec<&str> = plan.receivers.iter().map(|receiver| return receiver.namespace.as_str()).collect();
    assert_eq!(named, vec!["mnt:u", "mnt:x"]);
    let cover: &Receiver = plan.receivers.iter().find(|receiver| return receiver.namespace == "mnt:x").expect("the covered one");
    assert!(cover.covered);
    assert!(plan.way_eight);
}

/// A parent that shares with no group of its own takes way 7 and seeds nothing,
/// which is the reading the design fixes after the arm that over-read a `master:`.
#[test]
fn a_parent_with_no_group_of_its_own_takes_way_seven() {
    let acting: NamespaceTable = table(
        "mnt:u",
        vec![row(10, 1, "/checkout", &[], Some(7)), the_copy(11, 10, "/checkout/one", &[], None)],
        Some(the_copy(11, 10, "/checkout/one", &[], None)),
    );
    let copy: MountRow = the_copy(11, 10, "/checkout/one", &[], None);
    let handled: Vec<String> = vec![String::from("mnt:u")];
    let plan: ReceiverSet = plan_the_receivers(&acting, &copy, std::slice::from_ref(&acting), DATASET, handled.as_slice());
    assert!(plan.seed.is_empty());
    assert!(plan.receivers.is_empty());
    assert!(!plan.way_eight);
}

/// The parent is the mount the copy sits on,
/// and a copy whose parent is not in the table answers nothing.
#[test]
fn the_parent_is_the_mount_the_copy_sits_on() {
    let acting: NamespaceTable = table(
        "mnt:u",
        vec![row(10, 1, "/checkout", &[7], None), the_copy(11, 10, "/checkout/one", &[], None)],
        Some(the_copy(11, 10, "/checkout/one", &[], None)),
    );
    let copy: MountRow = the_copy(11, 10, "/checkout/one", &[], None);
    assert_eq!(parent_of(&acting, &copy).map(|parent| return parent.mount_id), Some(10));
    let orphan: MountRow = the_copy(11, 99, "/checkout/one", &[], None);
    assert_eq!(parent_of(&acting, &orphan).map(|parent| return parent.mount_id), None);
    let handled: Vec<String> = vec![String::from("mnt:u")];
    assert!(!plan_the_receivers(&acting, &orphan, std::slice::from_ref(&acting), DATASET, handled.as_slice()).way_eight);
}
