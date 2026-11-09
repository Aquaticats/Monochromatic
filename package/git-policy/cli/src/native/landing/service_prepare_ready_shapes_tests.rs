//! What:
//!  The commit's scratch directory as the preparation hands it on:
//!  where it is made,
//!  and that the command that made it never removes it.
//! Why:
//!  A recursive removal after the commit landed waited on the person
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`),
//!  so what the command must not do is remove it,
//!  and what must be true is that something else does:
//!  the service that held the connection,
//!  or the sweep of the repository scratch directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const scratch = makeScratch(common, git, name); drop(scratch); expect(exists(scratch.path)).toBe(true);
//! ```

/// The directory under control,
/// and the place it is made.
use super::Scratch;
use crate::scratch_location::REPOSITORY_SCRATCH_NAME;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// The preparation lets go of the tree without removing it:
/// the service that held it removes it,
/// and the sweep removes the one no service held.
#[test]
fn the_commit_hands_its_scratch_tree_on_and_never_removes_it() {
    let root: PathBuf = fixture("ready-shapes-scratch");
    let common: PathBuf = root.join("repo/.git");
    let tree: PathBuf = common.join(REPOSITORY_SCRATCH_NAME).join("monochromatic-git-policy-cli-service-0a1b2c3d");
    std::fs::create_dir_all(&tree).expect("the scratch tree");
    std::fs::write(tree.join("commit.index"), b"index").expect("a file in the tree");
    drop(Scratch(tree.clone()));
    assert!(tree.exists(), "the command that made it must not remove it: {}", tree.display());
    remove(root.as_path());
}

/// The tree is made inside the repository scratch directory,
/// where the sweep covers it.
#[test]
fn the_scratch_tree_is_made_where_the_sweep_covers_it() {
    let root: PathBuf = fixture("ready-shapes-scratch-made");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let scratch: Scratch = Scratch::make(common.as_path(), common.as_path(), "monochromatic-git-policy-cli-service-0a1b2c3d").expect("the scratch tree");
    assert_eq!(scratch.0, common.join(REPOSITORY_SCRATCH_NAME).join("monochromatic-git-policy-cli-service-0a1b2c3d"));
    drop(scratch);
    remove(root.as_path());
}

/// A second attempt of one command gets an empty tree,
/// so nothing an earlier attempt wrote is read as this attempt's.
#[test]
fn a_second_attempt_starts_from_an_empty_tree() {
    let root: PathBuf = fixture("ready-shapes-scratch-again");
    let common: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&common).expect("common directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let first: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree");
    std::fs::write(first.0.join("commit.index"), b"stale").expect("a file in the tree");
    std::fs::create_dir_all(first.0.join("shadow")).expect("a shadow of the earlier attempt");
    drop(first);
    let again: Scratch = Scratch::make(common.as_path(), common.as_path(), name).expect("the scratch tree again");
    assert!(!again.0.join("commit.index").exists(), "the earlier attempt's index is gone");
    assert!(!again.0.join("shadow").exists(), "the earlier attempt's shadow is gone");
    remove(root.as_path());
}

/// A repository whose scratch directory cannot be used still gets a tree,
/// in the Git directory,
/// so a commit never fails over where its scratch files go.
#[test]
fn an_unusable_repository_scratch_directory_falls_back_to_the_git_directory() {
    let root: PathBuf = fixture("ready-shapes-scratch-fallback");
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&git_dir).expect("git directory");
    // A link where the repository scratch directory goes is refused by the privacy check.
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");
    std::os::unix::fs::symlink(&elsewhere, git_dir.join(REPOSITORY_SCRATCH_NAME)).expect("a link in place of the directory");
    let name: &str = "monochromatic-git-policy-cli-service-0a1b2c3d";
    let scratch: Scratch = Scratch::make(git_dir.as_path(), git_dir.as_path(), name).expect("the scratch tree");
    assert_eq!(scratch.0, git_dir.join(name));
    drop(scratch);
    remove(root.as_path());
}
