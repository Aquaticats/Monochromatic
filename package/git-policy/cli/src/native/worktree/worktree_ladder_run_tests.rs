//! Controls for the choice of rung:
//! a plan on an eligible source claims a stage and runs Git without a checkout;
//! a plan on
//!  any other source prepares an empty subvolume;
//! no plan prepares nothing;
//! and a failed Git is settled quietly.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::is_subvolume_root;
use crate::test_support::{git_context, remove, repository};
use crate::worktree_copy_stage_claim::stage_claims_exist;
use crate::worktree_ladder_test_support::{main_identity, plan_to, subvolume_repository};

/// A failed Git run,
/// as the lifecycle reports it.
const FAILED: GitStatus = GitStatus { code: Some(128) };

/// A runner that must not be called.
fn no_runner() -> impl FnMut(&GitStep<'_>) -> Result<GitStatus, crate::worktree_copy_error::WorktreeCopyError> {
    return |_step| { panic!("no Git run is expected"); };
}

/// Without a plan the ladder asks for the person's own run and settles to Git's status.
#[test]
fn no_plan_means_the_persons_run() {
    let ladder: Ladder = begin_ladder(None, &git_context(), &WorktreeIdentity::OutsideWorktree);
    assert_eq!(ladder.first_step(), GitStep::Person);
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(GitStatus { code: Some(0) }, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, GitStatus { code: Some(0) });
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None, "no rung put an object at the destination");
    assert!(result.claim.is_none(), "no plan means no record");
}

/// A source that cannot be snapshotted gets an empty subvolume at the destination,
/// removed again when Git failed.
#[test]
fn btrfs_a_refused_source_gets_a_subvolume_that_a_failed_git_leaves_no_trace_of() {
    let Some(root) = btrfs_fixture("run-bootstrap") else { return };
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    std::fs::write(repo.join("untracked.txt"), b"u").expect("an untracked file rules every source out");
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::Person);
    assert!(is_subvolume_root(plan.destination.as_path()).expect("probe"));
    assert_eq!(ladder.slow.as_ref().map(|route| { return route.word(); }), Some("not-subvolume"), "an ordinary directory on btrfs is told to be converted: {:?}", ladder.slow);
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, FAILED);
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None);
    assert!(result.claim.is_none(), "Git never ran, so no rung took a claim");
    assert!(!plan.destination.exists());
    remove(root.as_path());
}

/// A clean source that is an ordinary directory gets the clone rung:
/// Git runs without a checkout,
/// into an empty subvolume.
#[test]
fn btrfs_a_clean_ordinary_source_gets_the_clone_rung_and_a_subvolume() {
    let Some(root) = btrfs_fixture("run-clone") else { return };
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "ordinary")).expect("canonical");
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at));
    assert!(is_subvolume_root(plan.destination.as_path()).expect("probe"));
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the clone rung records the add durably, so a killed add can be removed");
    assert!(!plan.destination.with_file_name(".cli-git-worktree-copy-stage").exists(), "the clone rung stages nothing");
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, FAILED);
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None);
    assert!(!plan.destination.exists(), "the empty subvolume is removed after a failed Git");
    assert!(result.claim.is_some(), "the record of the add is handed to the lifecycle rather than dropped");
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the record stands until its owner settles it");
    result.claim.expect("the record").settle().expect("settled");
    remove(root.as_path());
}

/// An eligible source claims its stage name and runs Git without a checkout;
/// a failed Git releases the claim and leaves
///  the destination as Git left it.
#[test]
fn btrfs_an_eligible_source_runs_git_without_a_checkout_and_a_failure_releases_the_claim() {
    let Some(root) = btrfs_fixture("run-snapshot") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at));
    assert_eq!(ladder.slow, None, "a snapshot has nothing to explain");
    assert!(!plan.destination.exists(), "a snapshot rung makes no subvolume before Git");
    assert!(stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the stage name is claimed durably");
    let mut runner = no_runner();
    let result: LadderResult = ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    assert_eq!(result.git, FAILED);
    assert_eq!(result.carried, None);
    assert_eq!(result.placed, None);
    assert!(result.claim.is_none(), "a Git that failed before the snapshot released the claim");
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "the claim is released");
    remove(root.as_path());
}

/// A subvolume main checkout with an uncommitted change gets the snapshot rung with its list of changes and nothing to explain;
///  the same checkout in the middle of a merge keeps its reason for the person and gets Git's checkout.
#[test]
fn btrfs_a_dirty_subvolume_source_is_snapshotted_and_a_merging_one_keeps_its_reason() {
    let Some(root) = btrfs_fixture("run-dirty-reason") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    std::fs::write(repo.join("shared.txt"), b"edited\n").expect("edit");
    let plan = plan_to(root.join("topic"));
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(ladder.first_step(), GitStep::WithoutCheckout(plan.insert_at), "a snapshot serves a source with a change");
    assert_eq!(ladder.slow, None, "a snapshot has nothing to explain");
    let Rung::Snapshot(source, _claim) = &ladder.rung else { panic!("the snapshot rung is expected") };
    let changed = source.dirty.as_ref().expect("the changes are listed");
    assert_eq!(changed.first.as_deref(), Some("shared.txt"));
    let mut runner = no_runner();
    ladder.finish(FAILED, &mut runner, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    let head: String = String::from_utf8_lossy(&crate::test_support::git(repo.as_path(), &["rev-parse", "HEAD"]).stdout).trim().to_string();
    std::fs::write(repo.join(".git/MERGE_HEAD"), format!("{head}\n")).expect("merge mark");
    let merging: Ladder = begin_ladder(Some(&plan), &git_context(), &main_identity(repo.as_path()));
    assert_eq!(merging.first_step(), GitStep::Person, "an unfinished merge stays a refusal");
    let line: String = merging.slow.as_ref().expect("a reason").line();
    assert!(line.contains("MERGE_HEAD"), "{line}");
    let mut second = no_runner();
    merging.finish(FAILED, &mut second, &git_context(), &mut std::io::sink(), &mut std::io::sink());
    remove(root.as_path());
}

/// A stage is destroyed only while the name still holds the object this process made:
/// another directory at the name,
/// and a
///  stage that was never identified,
/// are left in place.
#[test]
fn btrfs_a_stage_is_discarded_only_while_it_is_the_one_that_was_made() {
    let Some(root) = btrfs_fixture("run-discard-identity") else { return };
    let base: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    let made = |name: &str| -> (PathBuf, crate::fs_beneath::ObjectId) {
        let stage: PathBuf = base.join(name);
        crate::fs_btrfs_ioctl::subvolume_create_at(stage.as_path()).expect("stage");
        std::fs::write(stage.join("file"), b"x").expect("content");
        let parent = crate::fs_beneath::open_root(base.as_path()).expect("parent");
        let id = crate::fs_beneath::identity_at(&parent, std::ffi::OsStr::new(name)).expect("identity");
        return (stage, id);
    };
    let (stage, id) = made("stage-one");
    discard_stage(stage.as_path(), Some(id));
    assert!(!stage.exists(), "the stage that was made goes");
    let (other, id_two) = made("stage-two");
    std::fs::create_dir(base.join("stage-three")).expect("somebody's directory");
    std::fs::write(base.join("stage-three/theirs"), b"theirs").expect("their file");
    discard_stage(base.join("stage-three").as_path(), Some(id_two));
    assert!(base.join("stage-three/theirs").exists(), "a directory that is not the one made stays");
    discard_stage(other.as_path(), None);
    assert!(other.join("file").exists(), "an unidentified stage stays");
    remove(root.as_path());
}
