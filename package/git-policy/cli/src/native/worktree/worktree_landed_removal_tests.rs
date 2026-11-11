//! What:
//!  Controls for the removal of the worktrees whose branches a merge landed.
//! Why:
//!   The removal happens without a person asking for it,
//!  so each condition that keeps a worktree needs a control that would catch it going,
//!  and the clean case needs one that proves the worktree really goes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = aRepositoryWithAWorktree(); const sweep = sweepLandedWorktrees(...); expect(sweep.removed).toHaveLength(1);
//! ```

use super::{LandedPlace, LandedSweep, sweep_landed_worktrees};
use crate::test_support::{REAL_GIT, age, fixture, git, git_context, git_text, repository, spawn_owned};
use crate::worktree_added_by_the_wrapper::record;
use std::path::{Path, PathBuf};

/// A repository with one linked worktree whose branch holds a commit of its own.
struct Landed {
    /// The fixture directory that holds both.
    root: PathBuf,
    /// The main checkout,
    /// where the merge runs.
    main: PathBuf,
    /// Its common Git directory.
    common: PathBuf,
    /// The linked worktree,
    /// on branch `feature`.
    worktree: PathBuf,
}

/// What:
///  A repository whose branch `feature` is checked out in a linked worktree and holds one commit.
/// Why:
///   Every control starts from a repository where one merge lands one branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landed = aRepositoryWithAWorktreeWhoseBranchHasACommit(name);
/// ```
fn a_repository_with_a_worktree(name: &str) -> Landed {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    let tracked: PathBuf = main.join("README.md");
    std::fs::write(tracked.as_path(), b"# fixture\n").expect("a tracked file");
    // A file as new as the index proves nothing by its stat fields, and the cleanliness proof would then refuse
    // every worktree below (`test_support::age`).
    age(tracked.as_path());
    git(main.as_path(), &["add", "--", "README.md"]);
    git(main.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let worktree: PathBuf = root.join("worktree");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "feature", worktree.to_str().expect("a UTF-8 fixture path")]);
    git(worktree.as_path(), &["commit", "--quiet", "--allow-empty", "--message=the feature work"]);
    let common: PathBuf = main.join(".git");
    return Landed { root, main, common, worktree };
}

/// The place one control sweeps,
/// with the real process table.
fn place_of<'a>(landed: &'a Landed, context: &'a crate::transaction_git::GitContext) -> LandedPlace<'a> {
    return LandedPlace {
        context,
        cwd: landed.main.as_path(),
        common_dir: landed.common.as_path(),
        current_root: Some(landed.main.as_path()),
        proc_root: Path::new("/proc"),
    };
}

/// The head of the main checkout now.
fn head_of(main: &Path) -> String {
    return git_text(main, &["rev-parse", "HEAD"]);
}

/// What:
///  Land `feature` on `main` and answer the sweep of the worktrees it landed.
/// Why:
///   The merge is the event the sweep hangs on,
///  and its two heads are what the sweep compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sweep = landFeature(landed);
/// ```
fn land_feature(landed: &Landed) -> LandedSweep {
    let before: String = head_of(landed.main.as_path());
    git(landed.main.as_path(), &["merge", "--quiet", "--no-ff", "--message=land the feature", "feature"]);
    let after: String = head_of(landed.main.as_path());
    let context: crate::transaction_git::GitContext = git_context();
    let place: LandedPlace<'_> = place_of(landed, &context);
    return sweep_landed_worktrees(&place, before.as_str(), after.as_str(), "main");
}

#[test]
fn a_landed_clean_worktree_is_removed_and_its_branch_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-clean");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.failed.is_none(), "{:?}", sweep.failed);
    assert_eq!(sweep.removed.len(), 1, "one landed worktree is removed: {sweep:?}");
    assert!(sweep.kept.is_empty() && sweep.unproven.is_empty(), "{sweep:?}");
    assert!(!landed.worktree.exists(), "the worktree directory is gone");
    let branch: String = git_text(landed.main.as_path(), &["rev-parse", "--verify", "refs/heads/feature"]);
    assert_eq!(branch.len(), 40, "the branch the worktree held stays: {branch}");
    assert!(landed.root.join("main").exists(), "the repository itself stays");
}

#[test]
fn a_landed_worktree_with_uncommitted_work_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-dirty");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let notes: PathBuf = landed.worktree.join("notes.md");
    std::fs::write(notes.as_path(), b"what I was in the middle of\n").expect("a file the person wrote");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "nothing is removed while the worktree holds work: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(!sweep.kept[0].why.is_empty(), "the reason is named: {sweep:?}");
    assert!(landed.worktree.exists(), "the worktree stays");
    assert_eq!(std::fs::read(notes.as_path()).expect("the file is still there"), b"what I was in the middle of\n");
}

#[test]
fn a_landed_worktree_with_a_process_inside_stays() {
    let landed: Landed = a_repository_with_a_worktree("landed-busy");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let mut child: std::process::Child = spawn_owned(std::process::Command::new("sleep").arg("120").current_dir(landed.worktree.as_path()))
        .expect("a helper working in the worktree");
    let sweep: LandedSweep = land_feature(&landed);
    let _ended = child.kill();
    let _waited = child.wait();
    assert!(sweep.removed.is_empty(), "nothing is removed while a process works there: {sweep:?}");
    assert_eq!(sweep.kept.len(), 1, "{sweep:?}");
    assert!(sweep.kept[0].why.contains(&format!("process {} is working in it", child.id())), "{}", sweep.kept[0].why);
    assert!(landed.worktree.exists(), "the worktree stays");
}

#[test]
fn a_landed_worktree_this_program_cannot_prove_it_made_is_left() {
    let landed: Landed = a_repository_with_a_worktree("landed-unproven");
    let sweep: LandedSweep = land_feature(&landed);
    assert!(sweep.removed.is_empty(), "nothing is removed without the record: {sweep:?}");
    assert_eq!(sweep.unproven.len(), 1, "{sweep:?}");
    assert_eq!(sweep.unproven[0].branch, "refs/heads/feature");
    assert!(landed.worktree.exists(), "the worktree stays");
}

#[test]
fn a_branch_that_merely_sat_on_an_older_commit_is_no_candidate() {
    let landed: Landed = a_repository_with_a_worktree("landed-older");
    record(landed.common.as_path(), landed.worktree.as_path()).expect("the record of a worktree this program made");
    let older: PathBuf = landed.root.join("older");
    let first: String = git_text(landed.main.as_path(), &["rev-parse", "HEAD"]);
    git(
        landed.main.as_path(),
        &["worktree", "add", "--quiet", "-b", "older", older.to_str().expect("a UTF-8 fixture path"), first.as_str()],
    );
    record(landed.common.as_path(), older.as_path()).expect("the record of a worktree this program made");
    let sweep: LandedSweep = land_feature(&landed);
    assert_eq!(sweep.removed.len(), 1, "only the branch the merge landed is removed: {sweep:?}");
    assert_eq!(sweep.removed[0].worktree, landed.worktree);
    assert!(older.exists(), "a branch that sat on an older commit is no work this merge brought in");
}

/// The path of the host's Git,
/// named so a control can say which Git it used.
#[test]
fn the_fixture_git_is_the_host_git() {
    assert!(Path::new(REAL_GIT).exists(), "{REAL_GIT}");
}
