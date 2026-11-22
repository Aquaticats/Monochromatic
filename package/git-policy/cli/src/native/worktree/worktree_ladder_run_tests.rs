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
    let Rung::Snapshot(source, _claim, _fence) = &ladder.rung else { panic!("the snapshot rung is expected") };
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

/// What:
///  A source whose attributes convert more than a quarter of its paths is declined before the add's record is claimed,
///  so no copy and no snapshot is paid for an add that Git's own checkout will write.
/// Why:
///   The decline needs `git check-attr` and the count over its answer and nothing else,
/// so deciding it after the copies made a declined add pay the copy and then Git's whole checkout over it
///  (the 0.644 fixture measured 1.67 to 1.75 of `main`,
/// `doc/planning/cli-git-clone-rung-conversion-fence.md`,
///  "What it costs").
/// The count is one past the floor and a quarter,
/// so the fixture is the smallest that reaches the rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(plan, context, identity);
/// expect(ladder.rung).toBe('git'); expect(ladder.slow.word).toBe('declines'); expect(claims()).toBe(0);
/// ```
#[test]
fn a_converting_source_is_declined_before_anything_is_claimed_or_copied() {
    let root: PathBuf = crate::test_support::fixture("run-decline-before-the-copy");
    let repo: PathBuf = repository(root.as_path(), "repo");
    crate::worktree_ladder_test_support::tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    // One path past the floor and more than a quarter of the tracked files.
    for index in 0..=crate::worktree_clone_conversion_fence::DECLINE_PATHS_FLOOR {
        crate::worktree_ladder_test_support::tracked(repo.as_path(), format!("file-{index}.txt").as_str(), b"line\n");
    }
    crate::worktree_ladder_test_support::commit_all(repo.as_path(), "fixture");
    let plan = plan_to(root.join("topic"));
    let identity = main_identity(repo.as_path());
    // The fence is only reached where a fast rung applies: a destination whose filesystem cannot clone refuses the clone
    // rung before the fence runs, and the gate image's fixture root is a tmpfs, so this control names its own skip there.
    if let Some(reason) = crate::worktree_snapshot_source::clone_refusal(&identity, &plan) {
        eprintln!("CLONE RUNG NOT RUN: a_converting_source_is_declined_before_anything_is_claimed_or_copied ({reason})");
        remove(root.as_path());
        return;
    }
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &identity);
    assert_eq!(ladder.first_step(), GitStep::Person, "the person's own run checks the worktree out");
    assert!(matches!(ladder.rung, Rung::Git(_)), "a source above the quarter takes Git's checkout");
    assert_eq!(ladder.slow.as_ref().map(|route| { return route.word(); }), Some("declines"), "{:?}", ladder.slow);
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "nothing was claimed, so nothing can have been copied into or out of a stage");
    // The Git rung prepares an empty subvolume at the destination when the filesystem allows one, so what says nothing was copied is that it holds no entry.
    let prepared: usize = if plan.destination.exists() { std::fs::read_dir(plan.destination.as_path()).expect("the destination").count() } else { 0 };
    assert_eq!(prepared, 0, "the rung copied nothing into the destination");
    remove(root.as_path());
}

/// What:
///  An add at a commit whose tree lacks a path the attributes convert is skipped before the add's record is claimed,
///  so no copy is paid for a worktree that Git's own checkout writes.
/// Why:
///   The index this rung installs holds no stat data for a path the fence hands to Git,
/// and the tree difference refuses such an entry,
///  so the rung would end after its copies and pay Git's whole checkout over them,
///  as the "Limitations" section of
///  `doc/planning/cli-git-clone-rung-conversion-fence.md` says.
/// The attributes are read before the claim,
///  so the fence's own answer and one name-only read settle it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(planAt('other'), context, identity);
/// expect(ladder.rung).toBe('git'); expect(ladder.slow.line()).toContain('extra.txt'); expect(claims()).toBe(0);
/// ```
#[test]
fn a_target_tree_that_changes_a_fenced_path_is_skipped_before_anything_is_claimed_or_copied() {
    let root: PathBuf = crate::test_support::fixture("run-target-tree-skip");
    let repo: PathBuf = repository(root.as_path(), "repo");
    crate::worktree_ladder_test_support::tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    crate::worktree_ladder_test_support::tracked(repo.as_path(), "a.txt", b"line one\nline two\n");
    crate::worktree_ladder_test_support::commit_all(repo.as_path(), "conversion");
    crate::test_support::git(repo.as_path(), &["update-ref", "refs/heads/other", "HEAD"]);
    crate::worktree_ladder_test_support::tracked(repo.as_path(), "extra.txt", b"line one\nline two\n");
    crate::worktree_ladder_test_support::commit_all(repo.as_path(), "second");
    let plan = crate::worktree_ladder_test_support::plan_to_at(root.join("topic"), Some("other"));
    let identity = main_identity(repo.as_path());
    // A destination whose filesystem cannot clone refuses the clone rung before the fence runs, so this control names its
    // own skip there, as the decline control beside it does.
    if let Some(reason) = crate::worktree_snapshot_source::clone_refusal(&identity, &plan) {
        eprintln!("CLONE RUNG NOT RUN: a_target_tree_that_changes_a_fenced_path_is_skipped_before_anything_is_claimed_or_copied ({reason})");
        remove(root.as_path());
        return;
    }
    let ladder: Ladder = begin_ladder(Some(&plan), &git_context(), &identity);
    assert_eq!(ladder.first_step(), GitStep::Person, "Git's own checkout writes the worktree");
    assert!(matches!(ladder.rung, Rung::Git(_)), "the rung steps aside before the copies");
    let route: &SlowRoute = ladder.slow.as_ref().expect("the sentence that says why");
    assert_eq!(route.word(), "declines", "{route:?}");
    assert!(route.line().contains("differs from the source's index at extra.txt"), "{}", route.line());
    assert!(!stage_claims_exist(repo.join(".git").as_path()).expect("claims"), "nothing was claimed, so nothing can have been copied into or out of a stage");
    let prepared: usize = if plan.destination.exists() { std::fs::read_dir(plan.destination.as_path()).expect("the destination").count() } else { 0 };
    assert_eq!(prepared, 0, "the rung copied nothing into the destination");
    remove(root.as_path());
}
