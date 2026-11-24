//! What:
//!  Controls of one mount as a report names it:
//!  a table row becomes the kernel's own three fields,
//!  and only the dataset's own mount shows the dataset's root.
//! Why:
//!  The exclusion a standing `unmount` intent makes reads the root,
//!  so a bind of a folder inside the clone must not look like the managed
//!  attach.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(HeldMount.of(facts).isTheDatasetRoot()).toBe(true);
//! ```

use super::HeldMount;
use crate::fs_mount_options::parse_mount_table;

/// The slot's own mount of the dataset and a bind of a folder inside it.
fn the_two_mounts() -> (HeldMount, HeldMount) {
    let table = parse_mount_table(
        concat!(
            "41 30 0:32 / /home/user/Monochromatic/.claude/worktrees/one ro - zfs zpcachyos/nosnap/clones/one ro\n",
            "42 30 0:32 /sub /srv/elsewhere ro - zfs zpcachyos/nosnap/clones/one ro\n"
        )
        .as_bytes(),
    );
    return (HeldMount::of(&table[0]), HeldMount::of(&table[1]));
}

/// The dataset's own mount shows `/` and a bind of a folder inside it does
/// not.
#[test]
fn only_the_datasets_own_mount_shows_its_root() {
    let (own, bind) = the_two_mounts();
    assert_eq!(own.mount_id, 41);
    assert_eq!(own.root, std::path::PathBuf::from("/"));
    assert_eq!(own.mount_point, std::path::PathBuf::from("/home/user/Monochromatic/.claude/worktrees/one"));
    assert!(own.is_the_dataset_root());
    assert_eq!(bind.root, std::path::PathBuf::from("/sub"));
    assert!(!bind.is_the_dataset_root());
}
