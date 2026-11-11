//! Controls for what a conversion does about a stage container another command is filling:
//! the swap is refused before the exchange while one stands below the worktree,
//! the old tree is kept while one stands below it after the exchange,
//! a dead owner's container and a folder no claim names refuse nothing,
//! and a claim this release cannot read refuses rather than rules a container out.

use super::*;
use crate::test_support::{dead_copy_owner, fixture, remove};
use crate::worktree_copy_journal::journal_root;
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner};
use crate::worktree_copy_stage_claim::{StageClaim, claim_stage_for_add_as};
use std::path::PathBuf;

/// A parent with a common directory beside the worktree and the converting name an exchange leaves.
fn trees(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let common: PathBuf = parent.join("common");
    let worktree: PathBuf = parent.join("wt");
    std::fs::create_dir_all(common.as_path()).expect("common directory");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    return (parent, common, worktree);
}

/// A claim on a stage beside a destination below `tree`,
/// with the container filled at `stands`.
fn filled_container(common: &Path, tree: &Path, stands: &Path, owner: &CopyOwner) -> StageClaim {
    let claim: StageClaim = claim_stage_for_add_as(common, tree.join("inside/wt").as_path(), owner).expect("claim");
    let relative: PathBuf = claim.container().strip_prefix(tree).expect("below the tree").to_path_buf();
    let payload: PathBuf = stands.join(relative).join("payload/credentials");
    std::fs::create_dir_all(payload.parent().expect("the payload's folder")).expect("the container");
    std::fs::write(payload.as_path(), b"state still being staged\n").expect("fill the stage");
    return claim;
}

/// What:
///  The swap is refused while another command fills a stage container below the worktree,
///  and the reason names the container and the process.
/// Why:
///   The exchange moves the whole worktree aside,
/// so the refusal has to come before it for that command's add to finish.
#[test]
fn a_live_stage_below_the_worktree_refuses_the_swap() {
    let (parent, common, worktree) = trees("filled-stages-swap");
    let owner: CopyOwner = current_copy_owner().expect("this process");
    let claim: StageClaim = filled_container(common.as_path(), worktree.as_path(), worktree.as_path(), &owner);
    let reasons: Vec<String> = swap_refusals(common.as_path(), worktree.as_path());
    assert_eq!(reasons.len(), 1, "{reasons:?}");
    let reason: String = format!("{}", reasons.first().expect("one reason"));
    assert!(reason.contains(claim.container().to_string_lossy().as_ref()), "the reason names the container: {reason}");
    assert!(reason.contains(format!("process {}", owner.pid).as_str()), "the reason names the process filling it: {reason}");
    assert!(reason.contains("nothing is swapped"), "the reason says what did not happen: {reason}");
    remove(parent.as_path());
}

/// What:
///  The old tree is kept while another command fills a stage container the exchange moved into it,
///  and the reason names the container's new place.
/// Why:
///   The claim names the container below the worktree,
/// which is the name the exchange took away,
///  so a reader that trusted the claim's own path alone would find nothing and delete the live stage.
#[test]
fn a_live_stage_below_the_old_tree_keeps_it() {
    let (parent, common, worktree) = trees("filled-stages-removal");
    let old_tree: PathBuf = parent.join("wt.cli-git-converting");
    let claim: StageClaim = filled_container(common.as_path(), worktree.as_path(), old_tree.as_path(), &current_copy_owner().expect("this process"));
    let kept: String = format!("{}", removal_refusal(common.as_path(), worktree.as_path(), old_tree.as_path()).expect_err("the old tree is kept"));
    assert!(kept.contains(old_tree.to_string_lossy().as_ref()), "the reason names the old tree: {kept}");
    assert!(kept.contains(claim.container().file_name().expect("name").to_string_lossy().as_ref()), "the reason names the container: {kept}");
    assert!(kept.contains("nothing is deleted"), "the reason says what did not happen: {kept}");
    remove(parent.as_path());
}

/// What:
///  A dead owner's container and a folder no claim names refuse nothing,
///  in the swap and in the removal alike.
/// Why:
///   A dead owner's claim is an abandoned stage the recovery sweep settles,
/// and a folder no claim names is not a container this wrapper reserved.
#[test]
fn a_dead_owners_container_and_an_unclaimed_folder_refuse_nothing() {
    let (parent, common, worktree) = trees("filled-stages-nobody");
    filled_container(common.as_path(), worktree.as_path(), worktree.as_path(), &dead_copy_owner());
    let unclaimed: PathBuf = worktree.join("inside2").join(".cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543");
    std::fs::create_dir_all(unclaimed.as_path()).expect("a folder named like a container that no claim names");
    assert!(swap_refusals(common.as_path(), worktree.as_path()).is_empty(), "nothing is being filled below the worktree");
    assert!(removal_refusal(common.as_path(), worktree.as_path(), worktree.as_path()).is_ok(), "nothing is being filled below the old tree");
    remove(parent.as_path());
}

/// What:
///  A claim this release cannot read refuses the swap and keeps the old tree,
///  naming the claims and nothing about a container,
///  because the claim is the only record of what it names.
/// Why:
///   Treating an unreadable claim as nobody's would delete the container of a live command
/// the record proved nothing about.
#[test]
fn an_unreadable_claim_refuses_the_swap_and_keeps_the_old_tree() {
    let (parent, common, worktree) = trees("filled-stages-unreadable");
    let root: PathBuf = journal_root(common.as_path());
    std::fs::create_dir_all(root.as_path()).expect("journal root");
    let claim: PathBuf = root.join("8f14e45fceea167a5a36dedd4bea2543.stage");
    std::fs::write(claim.as_path(), b"a line this release does not write\n").expect("a claim it cannot read");
    let reasons: Vec<String> = swap_refusals(common.as_path(), worktree.as_path());
    assert_eq!(reasons.len(), 1, "{reasons:?}");
    assert!(format!("{}", reasons.first().expect("one reason")).contains("cannot be read"), "{reasons:?}");
    let error = removal_refusal(common.as_path(), worktree.as_path(), worktree.as_path()).expect_err("no container can be ruled out");
    assert!(format!("{error}").contains("cannot be read"), "{error}");
    remove(parent.as_path());
}
