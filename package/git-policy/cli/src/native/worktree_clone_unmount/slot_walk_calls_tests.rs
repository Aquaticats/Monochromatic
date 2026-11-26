//! What:
//!  Controls for the release walk:
//!  the checkout's own root,
//!  the two final hops,
//!  and the fresh lookup.
//! Why:
//!  No mount stands in a host fixture,
//!  so both final hops reach the same folder and the fresh lookup reads its
//!  mount;
//!  the difference between the hops is proved in the guest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = walk.openTheSlot(checkout, 'one'); expect(walk.lookAgain(checkout, 'one')).toBe(41);
//! ```

use super::{
    LiveSlotWalk,
    SlotWalk,
};

/// A checkout with `.claude/worktrees/one`.
fn checkout(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::testing::test_support::fixture_root().join(name);
    std::fs::create_dir_all(root.join(".claude/worktrees/one").as_path())
        .expect("the fixture slot is made");
    return root;
}

/// Both hops reach the slot,
/// and the fresh lookup names the folder's mount.
#[test]
fn both_hops_reach_the_slot() {
    let root = checkout("slot-walk");
    let crossed: std::fs::File = LiveSlotWalk
        .open_the_slot(root.as_path(), "one")
        .expect("the crossing hop reaches the slot");
    let uncrossed: std::fs::File = LiveSlotWalk
        .open_the_slot_uncrossed(root.as_path(), "one")
        .expect("the refusing hop reaches the slot when nothing covers it");
    let first: u64 = crate::fs_beneath_mount_facts::mount_id_of(&crossed)
        .expect("the crossing handle's mount is read");
    let second: u64 = crate::fs_beneath_mount_facts::mount_id_of(&uncrossed)
        .expect("the refusing handle's mount is read");
    assert_eq!(first, second);
    assert_eq!(LiveSlotWalk.look_again(root.as_path(), "one"), Ok(first));
}

/// A slot that is not there is a refusal that names the hop.
#[test]
fn a_missing_slot_is_refused() {
    let root = checkout("slot-walk-missing");
    assert!(LiveSlotWalk.open_the_slot(root.as_path(), "two").is_err());
}
