//! Controls for the stage containers a live command is filling:
//! a live claim's container is reported where it stands,
//! a dead claim's is nobody's,
//! the name a conversion reads after its exchange is the converting one,
//! and a worktree reached through a link still names its own containers.

use super::*;
use crate::test_support::{dead_copy_owner, fixture, remove};
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner};
use crate::worktree_copy_selection::STAGE_PREFIX;
use crate::worktree_copy_stage_claim::{StageClaim, claim_stage_for_add_as};

/// A parent directory with a common directory beside the tree a claim's container stands in.
fn parent_with_a_common_directory(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let common: PathBuf = parent.join("common");
    let tree: PathBuf = parent.join("wt");
    std::fs::create_dir_all(common.as_path()).expect("common directory");
    std::fs::create_dir_all(tree.as_path()).expect("the tree");
    return (parent, common, tree);
}

/// A claim on a stage beside a destination below `tree`,
/// with a container created and filled.
fn filled_container(common: &Path, tree: &Path, owner: &CopyOwner) -> StageClaim {
    let claim: StageClaim = claim_stage_for_add_as(common, tree.join("inside/wt").as_path(), owner).expect("claim");
    let payload: PathBuf = claim.container().join("payload/credentials");
    std::fs::create_dir_all(payload.parent().expect("the payload's folder")).expect("the container");
    std::fs::write(payload.as_path(), b"state still being staged\n").expect("fill the stage");
    return claim;
}

/// What:
///  A container a live command fills below the tree is reported there,
///  with the process filling it.
/// Why:
///   The conversion refuses to swap or delete the tree for exactly these,
/// and a container it did not report would go with the old tree.
#[test]
fn a_live_claims_container_below_the_tree_is_reported() {
    let (parent, common, tree) = parent_with_a_common_directory("stages-being-filled-live");
    let owner: CopyOwner = current_copy_owner().expect("this process");
    let claim: StageClaim = filled_container(common.as_path(), tree.as_path(), &owner);
    let filled: Vec<FilledStage> = filled_stages_below(common.as_path(), tree.as_path(), tree.as_path()).expect("read the claims");
    assert_eq!(filled.len(), 1, "{filled:?}");
    assert_eq!(filled.first().expect("one stage").container, *claim.container(), "the container is reported where it stands");
    assert_eq!(filled.first().expect("one stage").owner, Some(owner.pid), "the process filling it is named");
    remove(parent.as_path());
}

/// What:
///  A container a dead command left is nobody's,
///  and a container no claim names is nobody's either.
/// Why:
///   The recovery sweep settles a dead owner's claim with its own proof,
/// and a folder no claim names is not a stage this wrapper reserved,
/// so neither is another command's work that a conversion must wait for.
#[test]
fn a_dead_claims_container_and_an_unclaimed_one_are_not_reported() {
    let (parent, common, tree) = parent_with_a_common_directory("stages-being-filled-dead");
    filled_container(common.as_path(), tree.as_path(), &dead_copy_owner());
    let unclaimed: PathBuf = tree.join("inside2").join(format!("{STAGE_PREFIX}8f14e45fceea167a5a36dedd4bea2543"));
    std::fs::create_dir_all(unclaimed.as_path()).expect("a folder named like a container that no claim names");
    let filled: Vec<FilledStage> = filled_stages_below(common.as_path(), tree.as_path(), tree.as_path()).expect("read the claims");
    assert!(filled.is_empty(), "neither a dead owner's container nor an unclaimed folder is a command's work: {filled:?}");
    remove(parent.as_path());
}

/// What:
///  A claim written before a conversion's exchange names a container the conversion finds at its
///  converting name afterwards,
///  and it is reported there rather than at the name the claim spells.
/// Why:
///   The claim is written once,
/// before the exchange,
/// so a reader after the exchange that looked only at the claim's own path would find nothing,
///  and the live container below the old tree would go with it.
#[test]
fn a_container_below_the_converting_name_is_reported_there() {
    let (parent, common, tree) = parent_with_a_common_directory("stages-being-filled-exchanged");
    let owner: CopyOwner = current_copy_owner().expect("this process");
    let claim: StageClaim = filled_container(common.as_path(), tree.as_path(), &owner);
    let converting: PathBuf = tree.with_file_name("wt.cli-git-converting");
    let relative: &Path = claim.container().strip_prefix(tree.as_path()).expect("below the tree");
    let moved: PathBuf = converting.join(relative);
    std::fs::create_dir_all(moved.parent().expect("the container's folder")).expect("the converting name");
    std::fs::rename(claim.container(), moved.as_path()).expect("the exchange moves the old tree aside");
    let filled: Vec<FilledStage> = filled_stages_below(common.as_path(), tree.as_path(), converting.as_path()).expect("read the claims");
    assert_eq!(filled.len(), 1, "{filled:?}");
    assert_eq!(filled.first().expect("one stage").container, moved, "the container is reported at the converting name");
    remove(parent.as_path());
}

/// What:
///  A container below a worktree another name reaches through a link is still reported.
/// Why:
///   A repository moved behind a link keeps every spelling of itself in the claims its commands wrote,
/// and a comparison of spellings alone would rule the live container out.
#[test]
fn a_worktree_reached_through_a_link_still_reports_its_container() {
    let (parent, common, tree) = parent_with_a_common_directory("stages-being-filled-link");
    let linked: PathBuf = parent.join("through-a-link");
    std::os::unix::fs::symlink(tree.as_path(), linked.as_path()).expect("a link to the worktree");
    let owner: CopyOwner = current_copy_owner().expect("this process");
    let claim: StageClaim = filled_container(common.as_path(), linked.as_path(), &owner);
    let filled: Vec<FilledStage> = filled_stages_below(common.as_path(), tree.as_path(), tree.as_path()).expect("read the claims");
    assert_eq!(filled.len(), 1, "{filled:?}");
    let reached: PathBuf = tree.join("inside").join(claim.container().file_name().expect("name"));
    assert_eq!(filled.first().expect("one stage").container, reached, "the container is reported below the tree the link reaches");
    remove(parent.as_path());
}
