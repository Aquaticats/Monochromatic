//! What:
//!  Controls of the exact source match:
//!  a dataset whose name begins with this one is not this one,
//!  a bind of the clone is a mount of it,
//!  another filesystem type naming the same source is not,
//!  and one namespace can hold the clone twice.
//! Why:
//!  A prefix match would report another dataset's holder as this dataset's,
//!  and a match that skipped a bind would miss a holder the design names
//!  ("including binds and staging").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(mountsOfSource(table, 'zpcachyos/…/clones/one').length).toBe(2);
//! ```

use super::{is_of_source, mounts_of_source};
use crate::fs_mount_options::{MountFacts, parse_mount_table};

/// The dataset every control here scans for.
const SOURCE: &str = "zpcachyos/nosnap/clones/one";

/// One table with the slot's own mount and a bind of the clone beside it.
fn held_twice() -> Vec<MountFacts> {
    let text: &str = concat!(
        "41 30 0:32 / /home/user/Monochromatic/.claude/worktrees/one ro,nosuid,nodev - zfs zpcachyos/nosnap/clones/one ro\n",
        "42 30 0:32 /sub /home/user/elsewhere ro,nosuid,nodev - zfs zpcachyos/nosnap/clones/one ro\n"
    );
    return parse_mount_table(text.as_bytes());
}

/// A dataset whose name has this one as a prefix is another dataset,
/// and another filesystem type naming it is no ZFS mount of it.
#[test]
fn a_prefix_and_another_type_are_not_matches() {
    let table: Vec<MountFacts> = parse_mount_table(
        concat!(
            "41 30 0:32 / /a rw - zfs zpcachyos/nosnap/clones/one-extra rw\n",
            "42 30 0:33 / /b rw - btrfs zpcachyos/nosnap/clones/one rw\n",
            "43 30 0:34 / /c rw - zfs zpcachyos/nosnap/clones/on rw\n"
        )
        .as_bytes(),
    );
    assert!(mounts_of_source(table.as_slice(), SOURCE).is_empty());
    assert!(!is_of_source(&table[0], SOURCE));
    assert!(!is_of_source(&table[1], SOURCE));
    assert!(!is_of_source(&table[2], SOURCE));
}

/// The slot's own mount and a bind of the clone elsewhere are both holders,
/// whatever root within the clone the bind shows.
#[test]
fn a_bind_of_the_clone_is_a_holder_beside_the_slot_mount() {
    let table: Vec<MountFacts> = held_twice();
    let held: Vec<&MountFacts> = mounts_of_source(table.as_slice(), SOURCE);
    assert_eq!(held.len(), 2);
    assert_eq!(held[0].mount_point, std::path::PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one"));
    assert_eq!(held[0].root, std::path::PathBuf::from("/"));
    assert_eq!(held[1].mount_point, std::path::PathBuf::from("/home/user/elsewhere"));
    assert_eq!(held[1].root, std::path::PathBuf::from("/sub"));
}
