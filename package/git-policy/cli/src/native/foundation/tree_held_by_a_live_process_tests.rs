//! Controls for the claim a live process keeps on a tree,
//! and for the question a sweep asks of it:
//!  a claimed tree answers that a live process holds it until the claim ends,
//!  a tree that is not there is neither claimed nor held,
//!  and a claim is the kernel's one exclusive lock, given to no second holder.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  A tree a live process claims answers that a process holds it,
///  and the claim ending makes the same tree answer that nobody does.
/// Why:
///  The claim is the whole of what a sweep reads to tell a live tree from an abandoned one across a clock step,
///  so its two answers must follow one another exactly when the holder ends.
#[test]
fn a_claimed_tree_says_a_process_holds_it_until_the_claim_ends() {
    let root: PathBuf = fixture("tree-claim-answers");
    let tree: PathBuf = root.join("a-tree");
    std::fs::create_dir(tree.as_path()).expect("a tree");
    let held: HeldTree = match hold(tree.as_path()) {
        ClaimTaken::Yes(held) => held,
        other => panic!("the claim of a tree nobody holds: {other:?}"),
    };
    assert_eq!(whether_held(tree.as_path()), WhetherHeld::ByALiveProcess, "a claimed tree is one a live process holds");
    drop(held);
    assert_eq!(whether_held(tree.as_path()), WhetherHeld::ByNobody, "the tree is nobody's once the claim ends");
    remove(root.as_path());
}

/// What:
///  A tree that is not there cannot be claimed,
///  and asking about it answers what the volume said rather than a hold.
/// Why:
///  A sweep asks only about a tree it has just listed,
///  and a holder claims a tree the service answered with,
///  which another process may have taken between the answer and the claim:
///  neither may turn that into an answer that a live process holds the tree.
#[test]
fn a_tree_that_is_not_there_is_neither_claimed_nor_held() {
    let root: PathBuf = fixture("tree-claim-missing");
    let gone: PathBuf = root.join("a-tree-that-is-not-there");
    assert!(matches!(hold(gone.as_path()), ClaimTaken::CannotBeAsked), "nothing claims a tree that is not there");
    assert_eq!(whether_held(gone.as_path()), WhetherHeld::CannotBeAsked, "and the volume answered no question about it");
    remove(root.as_path());
}

/// What:
///  A claim a descriptor already holds is not given to a second one,
///  and both the second claim and the sweep's question say a live process holds the tree.
/// Why:
///  The claim is what a sweep reads, so a second holder must never take it from the first;
///  the kernel gives one exclusive `flock`, and the question goes through the same call the claim does.
#[test]
fn a_claim_one_descriptor_holds_is_given_to_no_second() {
    let root: PathBuf = fixture("tree-claim-twice");
    let tree: PathBuf = root.join("a-tree");
    std::fs::create_dir(tree.as_path()).expect("a tree");
    let first: HeldTree = match hold(tree.as_path()) {
        ClaimTaken::Yes(held) => held,
        other => panic!("the first claim: {other:?}"),
    };
    // A share of the claim is held by two holders at once, which is what lets a tree change hands with no gap.
    assert!(matches!(hold(tree.as_path()), ClaimTaken::Yes(_)), "a shared claim is taken by a second holder");
    assert_eq!(whether_held(tree.as_path()), WhetherHeld::ByALiveProcess, "and it says a live process holds it");
    drop(first);
    remove(root.as_path());
}
