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
    // A tree that is not there is gone, and not a volume that cannot answer: a maker makes another.
    assert!(matches!(hold(gone.as_path()), ClaimTaken::Gone), "nothing claims a tree that is not there");
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

/// What:
///  A tree whose removal has not finished is still the removal's, and no maker may claim it.
/// Why:
///  `remove_within` stops waiting at its ceiling while its thread goes on deleting the tree,
///  so a claim released at the ceiling would let a commit build in a tree that is being emptied under it
///  (`remove_as_a_sweep`, `scratch_entry_removal.rs`).
///  The claim lives as long as the thread that deletes, not as long as the wait for it.
#[test]
fn a_tree_whose_removal_has_not_finished_is_still_claimed() {
    let root: PathBuf = fixture("tree-claim-unfinished-removal");
    let tree: PathBuf = root.join("a-tree");
    std::fs::create_dir(tree.as_path()).expect("a tree");
    std::fs::write(tree.join("refs"), b"a file of the tree").expect("a file in the tree");
    // The removal's thread waits before it deletes until this control releases it.
    let release: std::sync::mpsc::Sender<()> = crate::scratch_entry_removal::test_hold::hold_the_next_removal_for_a_control();
    let swept: SweptRemoval = remove_as_a_sweep(tree.as_path(), |path: &Path, claim: Option<HeldTree>| {
        return crate::scratch_entry_removal::remove_within(path, std::time::Duration::from_micros(1), claim);
    });
    assert!(
        matches!(swept, SweptRemoval::Removed(crate::scratch_entry_removal::Removal::Unfinished)),
        "the removal is still going when its ceiling passes: {swept:?}"
    );
    // The claim went with the removal, not with the wait for it.
    let offered: Result<ClaimOnAMadeTree, TreeIsGoing> = claim_a_made_tree(tree.clone());
    assert!(offered.is_err(), "a tree a removal has not finished is no maker's: {}", offered.is_ok());
    drop(release);
    // Once the removal ends the path is nobody's: it is gone, so no maker can build in it either.
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        let answer: Result<ClaimOnAMadeTree, TreeIsGoing> = claim_a_made_tree(tree.clone());
        if !matches!(answer, Ok(ClaimOnAMadeTree::Claimed(_))) {
            break;
        }
        assert!(began.elapsed() < std::time::Duration::from_secs(30), "the removal ends and the path is nobody's");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    remove(root.as_path());
}

/// What:
///  The claim is asked on a thread with a ceiling, so a file system that never answers cannot hold the caller.
/// Why:
///  `open` and `flock` of a tree are synchronous kernel calls,
///  and a directory on a mount whose server stopped answering never returns them;
///  `O_NONBLOCK` and `LOCK_NB` limit neither.
///  The package draws that line once, in `work_awaited_within_a_ceiling.rs`,
///  and the claim goes through it: a system that refuses the thread takes no claim and says the volume cannot be asked.
#[test]
fn the_claim_is_asked_within_a_ceiling() {
    let root: PathBuf = fixture("tree-claim-within-a-ceiling");
    let tree: PathBuf = root.join("a-tree");
    std::fs::create_dir(tree.as_path()).expect("a tree");
    let refused: crate::work_awaited_within_a_ceiling::ThreadsRefusedForTheControl = crate::work_awaited_within_a_ceiling::a_control_refuses_every_thread();
    let answer: Result<ClaimOnAMadeTree, TreeIsGoing> = claim_a_made_tree(tree.clone());
    drop(refused);
    assert!(
        matches!(answer, Ok(ClaimOnAMadeTree::TheVolumeCannotSay(_))),
        "a claim that cannot be asked is not taken: {}",
        answer.is_ok()
    );
    remove(root.as_path());
}
