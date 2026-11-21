//! What:
//!  Controls of the arm's own tree and its walk:
//!  the slot,
//!  the move target and the refusal tree are made under the scratch tmpfs,
//!  the walk reaches the slot and records its identity,
//!  and the checkout root opens read-only.
//! Why:
//!  The refusal controls plant inside this tree,
//!  so a tree the walk cannot reach would make every control unproven,
//!  and the checkout open must prove the root itself reaches the kernel.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! makeScratchTree(scratch, slot, moveTarget); const id = walkToSlot(scratch, slot);
//! ```

use super::{make_scratch_tree, open_checkout, walk_to_slot};
use crate::testing::test_support::fixture;
use std::path::PathBuf;

/// The tree is made once,
/// a second make keeps it,
/// and the walk reaches the slot and records one identity for it.
#[test]
fn the_tree_is_made_and_the_walk_reaches_the_slot() {
    let root: PathBuf = fixture("start-probe-walk");
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir_all(scratch.as_path()).expect("the scratch root is made");
    let slot: PathBuf = scratch.join("slot");
    let move_target: PathBuf = scratch.join("move-target");
    make_scratch_tree(scratch.as_path(), slot.as_path(), move_target.as_path()).expect("the tree is made");
    make_scratch_tree(scratch.as_path(), slot.as_path(), move_target.as_path()).expect("a second make keeps it");
    assert!(scratch.join("refusal/bound/slot").is_dir(), "the refusal tree is made");
    let first = walk_to_slot(scratch.as_path(), slot.as_path()).expect("the walk reaches the slot");
    let again = walk_to_slot(scratch.as_path(), slot.as_path()).expect("the walk reaches it again");
    assert_eq!(first, again);
}

/// The checkout root opens read-only,
/// and a root that is not there is refused with the call that refused.
#[test]
fn the_checkout_root_opens_or_names_the_call() {
    let root: PathBuf = fixture("start-probe-walk-checkout");
    open_checkout(root.as_path()).expect("the fixture root opens");
    let missing: PathBuf = root.join("no-such-checkout");
    let refusal = open_checkout(missing.as_path()).expect_err("the missing root is refused");
    assert_eq!(refusal.errno_name(), Some("ENOENT"));
}
