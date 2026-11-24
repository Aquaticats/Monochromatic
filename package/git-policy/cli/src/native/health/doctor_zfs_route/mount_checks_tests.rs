//! What:
//!  Controls of the checkout's mount,
//!  its propagation and the nested mounts' path reachability.
//! Why:
//!  A row that stands at the checkout path but shows a subtree,
//!  a shared checkout and a nested dataset whose row hangs from the tree the
//!  checkout replaced are the three states the guest measured,
//!  so each control writes the table VM item 9's shapes make.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(check_nested_reachability(withRows([orphan]))).toMatchObject({ kind: 'finding', field: 'reachability' });
//! ```

use super::{MountCheckFacts, check_checkout_mount, check_checkout_propagation, check_nested_reachability, nested_mounts};
use crate::health::doctor_zfs_route::route_test_facts::{CHECKOUT, CHECKOUT_DATASET, healthy_mounts, mount_row};
use crate::fs_mount_options::parse_mount_table;
use std::path::PathBuf;

/// The facts of one table a control wrote.
fn facts(rows: &[String]) -> MountCheckFacts {
    return MountCheckFacts { checkout_path: PathBuf::from(CHECKOUT), checkout_dataset: String::from(CHECKOUT_DATASET), rows: parse_mount_table(rows.join("\n").as_bytes()) };
}

/// The healthy table with one extra row.
fn with_extra(row: String) -> MountCheckFacts {
    return with_extras(&[row]);
}

/// The healthy table with several extra rows.
fn with_extras(extra: &[String]) -> MountCheckFacts {
    let mut rows: Vec<String> = crate::health::doctor_zfs_route::route_test_facts::healthy_mount_text().lines().map(String::from).collect();
    for row in extra {
        rows.push(row.clone());
    }
    return facts(&rows);
}

//region The checkout's own mount

/// The design's table passes.
#[test]
fn the_designs_checkout_mount_passes() {
    let outcome: super::CheckOutcome = check_checkout_mount(&healthy_mounts());
    assert!(matches!(outcome, super::CheckOutcome::Pass { .. }), "{outcome:?}");
}

/// A table with no checkout row is a finding naming the mount.
#[test]
fn a_table_without_the_checkout_row_is_a_finding() {
    let table: MountCheckFacts = facts(&[mount_row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root", "")]);
    let super::CheckOutcome::Finding { field, detail } = check_checkout_mount(&table) else {
        panic!("a table without the checkout row is a finding");
    };
    assert_eq!(field, "checkout-mount");
    assert!(detail.contains(CHECKOUT), "{detail}");
}

/// A row of a subtree under the checkout's name is a finding naming the root.
#[test]
fn a_bind_of_a_subtree_under_the_checkout_name_is_a_finding() {
    let table: MountCheckFacts = facts(&[mount_row(40, 1, "/subtree", CHECKOUT, "zfs", CHECKOUT_DATASET, "")]);
    let super::CheckOutcome::Finding { field, detail } = check_checkout_mount(&table) else {
        panic!("a bind under the checkout's name is a finding");
    };
    assert_eq!(field, "root");
    assert!(detail.contains("bind"), "{detail}");
}

/// Two rows at the checkout path are a finding naming both.
#[test]
fn two_rows_at_the_checkout_path_are_a_finding() {
    let table: MountCheckFacts = facts(&[mount_row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET, ""), mount_row(44, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET, "")]);
    let super::CheckOutcome::Finding { field, detail } = check_checkout_mount(&table) else {
        panic!("two rows are a finding");
    };
    assert_eq!(field, "checkout-mount");
    assert!(detail.contains("40") && detail.contains("44"), "{detail}");
}

//endregion

//region Propagation

/// A private checkout passes.
#[test]
fn a_private_checkout_passes() {
    let outcome: super::CheckOutcome = check_checkout_propagation(&healthy_mounts());
    assert!(matches!(outcome, super::CheckOutcome::Pass { .. }), "{outcome:?}");
}

/// A shared checkout is a finding that prints the command and names the field.
#[test]
fn a_shared_checkout_is_a_finding_with_the_command() {
    let table: MountCheckFacts = facts(&[mount_row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET, "shared:7")]);
    let super::CheckOutcome::Finding { field, detail } = check_checkout_propagation(&table) else {
        panic!("a shared checkout is a finding");
    };
    assert_eq!(field, "propagation");
    assert!(detail.contains("shared:7"), "{detail}");
    assert!(detail.contains("mount --make-rprivate"), "{detail}");
}

/// A slave checkout is a finding as well,
/// because only a mount with neither field is private.
#[test]
fn a_slave_checkout_is_a_finding() {
    let table: MountCheckFacts = facts(&[mount_row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET, "master:3")]);
    let super::CheckOutcome::Finding { field, .. } = check_checkout_propagation(&table) else {
        panic!("a slave checkout is a finding");
    };
    assert_eq!(field, "propagation");
}

/// Without the checkout row the propagation is unproved.
#[test]
fn a_table_without_the_checkout_row_leaves_propagation_unknown() {
    let table: MountCheckFacts = facts(&[mount_row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root", "")]);
    assert!(matches!(check_checkout_propagation(&table), super::CheckOutcome::Unknown { .. }));
}

//endregion

//region Nested reachability

/// A table with no nested dataset passes.
#[test]
fn a_table_with_no_nested_dataset_passes() {
    let outcome: super::CheckOutcome = check_nested_reachability(&healthy_mounts());
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("no nested dataset passes");
    };
    assert!(detail.contains('0'), "{detail}");
}

/// A nested dataset whose row hangs under the checkout is reachable.
#[test]
fn a_nested_row_under_the_checkout_is_reachable() {
    let table: MountCheckFacts = with_extra(mount_row(50, 40, "/", &format!("{CHECKOUT}/output"), "zfs", "zpcachyos/nosnap/vm-builder-output", ""));
    let outcome: super::CheckOutcome = check_nested_reachability(&table);
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("a nested row under the checkout is reachable");
    };
    assert!(detail.contains('1'), "{detail}");
}

/// A nested row hanging from the home underlay the checkout replaced is a
/// finding naming its path.
#[test]
fn a_nested_row_hanging_from_the_replaced_tree_is_a_finding() {
    let table: MountCheckFacts = with_extra(mount_row(50, 1, "/", &format!("{CHECKOUT}/output"), "zfs", "zpcachyos/nosnap/vm-builder-output", ""));
    let super::CheckOutcome::Finding { field, detail } = check_nested_reachability(&table) else {
        panic!("a row hanging from the underlay is a finding");
    };
    assert_eq!(field, "reachability");
    assert!(detail.contains("vm-builder-output"), "{detail}");
    assert!(detail.contains("output"), "{detail}");
}

/// Two rows at one path under the checkout are a finding naming the path.
#[test]
fn two_rows_at_one_nested_path_are_a_finding() {
    let table: MountCheckFacts = with_extras(&[
        mount_row(50, 40, "/", &format!("{CHECKOUT}/output"), "zfs", "zpcachyos/nosnap/vm-builder-output", ""),
        mount_row(51, 40, "/", &format!("{CHECKOUT}/output"), "zfs", "zpcachyos/nosnap/vm-builder-output", ""),
    ]);
    let super::CheckOutcome::Finding { field, detail } = check_nested_reachability(&table) else {
        panic!("two rows at one path are a finding");
    };
    assert_eq!(field, "path");
    assert!(detail.contains("50") && detail.contains("51"), "{detail}");
}

/// A row whose chain leaves the table reaches no checkout mount,
/// which is the shape a namespace cloned from another shows:
/// its root row names a parent the table does not hold.
#[test]
fn a_row_whose_chain_leaves_the_table_is_unreachable() {
    let table: MountCheckFacts = with_extra(mount_row(50, 99, "/", &format!("{CHECKOUT}/output"), "zfs", "zpcachyos/nosnap/vm-builder-output", ""));
    let super::CheckOutcome::Finding { field, .. } = check_nested_reachability(&table) else {
        panic!("a chain that leaves the table reaches no checkout mount");
    };
    assert_eq!(field, "reachability");
}

/// A row whose chain loops inside the table leaves reachability unknown.
#[test]
fn a_looping_chain_leaves_reachability_unknown() {
    let table: MountCheckFacts = with_extras(&[
        mount_row(50, 51, "/", &format!("{CHECKOUT}/output"), "zfs", "zpcachyos/nosnap/vm-builder-output", ""),
        mount_row(51, 50, "/", &format!("{CHECKOUT}/output-two"), "zfs", "zpcachyos/nosnap/vm-builder-output-two", ""),
    ]);
    assert!(matches!(check_nested_reachability(&table), super::CheckOutcome::Unknown { .. }));
}

/// The nested inventory lists a zfs row inside the checkout and no other.
#[test]
fn the_nested_inventory_lists_only_zfs_rows_inside_the_checkout() {
    let table: MountCheckFacts = with_extras(&[
        mount_row(50, 40, "/", &format!("{CHECKOUT}/output"), "zfs", "zpcachyos/nosnap/vm-builder-output", ""),
        mount_row(51, 40, "/", &format!("{CHECKOUT}/tmp"), "tmpfs", "tmpfs", ""),
    ]);
    let nested: Vec<crate::health::doctor_zfs_route::mount_checks::NestedMount> = nested_mounts(&table);
    assert_eq!(nested.len(), 1, "{nested:?}");
    assert_eq!(nested[0].dataset, "zpcachyos/nosnap/vm-builder-output");
}

//endregion
