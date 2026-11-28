//! What:
//!  The report of the shared mounts beneath the checkout:
//!  the exact count,
//!  the first four names,
//!  a private child that is not reported,
//!  a child found by its place rather than by its path,
//!  and a path written in the kernel's own spelling.
//! Why:
//!  The project decided the repair changes the checkout's own mount and no
//!  mount beneath it,
//!  and reports a shared mount it finds beneath,
//!  so the count must be exact and the paths must not break the record's lines
//!  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  "How far the setting reaches";
//!  the design's choice V7 bounds the names at four).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(sharedMountsBeneath([checkout, nested], 41)).toEqual({ count: 1, paths: ['/home/user/Monochromatic/nested'] });
//! ```

use super::*;
use crate::checkout_propagation::propagation_words::BENEATH_UNREAD;
use crate::subvolume::fs_mount_options::MountFacts;
use std::path::PathBuf;

/// The checkout's own row number,
/// which every child below names in its chain.
const CHECKOUT_ROW: u64 = 41;

/// What:
///  One mount table row,
///  with its own number,
///  the row it is mounted in,
///  and whether the kernel's tags say it is shared.
/// Why:
///  The scan selects by the parent chain and the tags and by nothing else,
///  so a control states exactly those three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const row = (id: number, parentId: number, shared: boolean, point: string): MountFacts => ({ ... });
/// ```
fn row(
    id: u64,
    parent_id: u64,
    shared: bool,
    point: &str,
) -> MountFacts {
    let tags: Vec<String> = match shared {
        true => vec![format!("shared:{}", id + 1_000)],
        false => Vec::new(),
    };
    return MountFacts {
        mount_id: id,
        parent_id,
        root: PathBuf::from("/"),
        optional_fields: tags,
        mount_point: PathBuf::from(point),
        filesystem_type: String::from("zfs"),
        source: String::from("pool/home/nested"),
        mount_options: vec![String::from("rw")],
        super_options: Vec::new(),
    };
}

#[test]
fn one_shared_child_is_counted_and_named() {
    let table: Vec<MountFacts> = vec![
        row(CHECKOUT_ROW, 1, true, "/home/user/Monochromatic"),
        row(
            267,
            CHECKOUT_ROW,
            true,
            "/home/user/Monochromatic/package/vm-builder/cli/output",
        ),
    ];
    let beneath: SharedMountsBeneath = shared_mounts_beneath(table.as_slice(), CHECKOUT_ROW);
    assert_eq!(
        beneath,
        SharedMountsBeneath::Counted {
            count: 1,
            paths: vec![String::from(
                "/home/user/Monochromatic/package/vm-builder/cli/output"
            )],
        }
    );
    assert_eq!(beneath.word(), "1");
}

#[test]
fn a_private_child_and_the_checkout_itself_are_not_reported() {
    let table: Vec<MountFacts> = vec![
        row(CHECKOUT_ROW, 1, true, "/home/user/Monochromatic"),
        row(
            268,
            CHECKOUT_ROW,
            false,
            "/home/user/Monochromatic/private-child",
        ),
    ];
    let beneath: SharedMountsBeneath = shared_mounts_beneath(table.as_slice(), CHECKOUT_ROW);
    assert_eq!(
        beneath,
        SharedMountsBeneath::Counted {
            count: 0,
            paths: Vec::new(),
        },
        "the checkout's own mount is not beneath itself, and a private child passes nothing on"
    );
}

#[test]
fn a_child_found_by_its_place_is_counted_even_when_another_mount_covers_it() {
    // The grandchild's parent is the child, which is itself hidden under a later mount at its own place: the chain is
    // what decides, not the path.
    let table: Vec<MountFacts> = vec![
        row(CHECKOUT_ROW, 1, true, "/home/user/Monochromatic"),
        row(267, CHECKOUT_ROW, true, "/home/user/Monochromatic/nested"),
        row(268, 267, true, "/home/user/Monochromatic/nested/deeper"),
    ];
    let beneath: SharedMountsBeneath = shared_mounts_beneath(table.as_slice(), CHECKOUT_ROW);
    assert_eq!(beneath.word(), "2");
    assert_eq!(beneath.paths().len(), 2);
}

#[test]
fn five_shared_children_are_counted_exactly_and_named_four_times() {
    let mut table: Vec<MountFacts> = vec![row(CHECKOUT_ROW, 1, true, "/home/user/Monochromatic")];
    for place in 1..=5_u64 {
        let point: String = format!("/home/user/Monochromatic/nested-{place}");
        table.push(row(300 + place, CHECKOUT_ROW, true, point.as_str()));
    }
    let beneath: SharedMountsBeneath = shared_mounts_beneath(table.as_slice(), CHECKOUT_ROW);
    assert_eq!(beneath.word(), "5", "the count is exact");
    assert_eq!(
        beneath.paths().len(),
        SHARED_MOUNTS_NAMED,
        "the record names four paths and the count carries the fifth"
    );
    assert_eq!(beneath.paths()[3], "/home/user/Monochromatic/nested-4");
}

#[test]
fn a_path_the_kernel_escapes_is_written_in_the_kernels_own_spelling() {
    let table: Vec<MountFacts> = vec![
        row(CHECKOUT_ROW, 1, true, "/home/user/Monochromatic"),
        row(
            267,
            CHECKOUT_ROW,
            true,
            "/home/user/Monochromatic/a b\tc\nd\\e",
        ),
    ];
    let beneath: SharedMountsBeneath = shared_mounts_beneath(table.as_slice(), CHECKOUT_ROW);
    assert_eq!(
        beneath.paths()[0],
        String::from("/home/user/Monochromatic/a\\040b\\011c\\012d\\134e"),
        "a space, a tab, a newline and a backslash are each three octal digits, as `/proc/self/mountinfo` writes them"
    );
}

#[test]
fn a_run_that_never_read_the_table_says_unread_rather_than_none() {
    let unread: SharedMountsBeneath = SharedMountsBeneath::Unread;
    assert_eq!(unread.word(), BENEATH_UNREAD);
    assert!(
        unread.paths().is_empty(),
        "a run that never looked names no path at all"
    );
}
