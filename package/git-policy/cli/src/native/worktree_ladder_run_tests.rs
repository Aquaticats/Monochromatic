//! Controls for the choice of rung: a plan on an eligible source claims a stage and runs Git without a checkout; a plan on
//!  any other source prepares an empty subvolume; no plan prepares nothing; and a failed Git is settled quietly.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::test_support::{git_context, remove, repository};
use crate::worktree_copy_stage_claim::stage_claims_exist;
use crate::worktree_ladder_test_support::{main_identity, plan_to, subvolume_repository};

/// A failed Git run, as the lifecycle reports it.
const FAILED: GitStatus = GitStatus { code: Some(128) };

/// A runner that must not be called.
fn no_runner() -> impl FnMut(Option<&str>, &GitStep<'_>) -> Result<GitStatus, super::super::worktree_copy_error::WorktreeCopyError> {
    return |_token, _step| { panic!("no Git run is expected"); };
}

/// Without a plan the ladder asks for the person's own run and settles to Git's status.
#[test]
fn no_plan_means_the_persons_run() {
    let ladder: Ladder = begin_ladder(None, &git_context(), &WorktreeIdentity::OutsideWorktree);
    assert_eq!(ladder.first_step(), GitStep::Person);
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(GitStatus { code: Some(0) }, None, &mut runner, &git_context(), &mut std::io::sink());
    assert_eq!(result, LadderResult { git: GitStatus { code: Some(0) }, carried: None });
}

/// A source that cannot be snapshotted gets an empty subvolume at the destination, removed again when Git failed.
#[test]
fn btrfs_a_refused_source_gets_a_subvolume_that_a_failed_git_leaves_no_trace_of() {
    let Some(root) = btrfs_fixture("run-bootstrap") else { return };
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::Person);
    assert!(is_subvolume_root(plan.destination.as_path()).expect("probe"));
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, None, &mut runner, &git_context(), &mut std::io::sink());
    assert_eq!(result, LadderResult { git: FAILED, carried: None });
    assert!(!plan.destination.exists());
    remove(root.as_path());
}

/// An eligible source claims its stage name and runs Git without a checkout; a failed Git releases the claim and leaves
///  the destination as Git left it.
#[test]
fn btrfs_an_eligible_source_runs_git_without_a_checkout_and_a_failure_releases_the_claim() {
    let Some(root) = btrfs_fixture("run-snapshot") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at));
    assert!(!plan.destination.exists(), "a snapshot rung makes no subvolume before Git");
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the stage name is claimed durably");
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, None, &mut runner, &git_context(), &mut std::io::sink());
    assert_eq!(result, LadderResult { git: FAILED, carried: None });
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the claim is released");
    remove(root.as_path());
}
