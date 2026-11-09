//! Controls for the lines the isolated merge's apply writes when another program holds the index.

use super::*;
/// The fixtures a linked worktree needs,
/// and the identity of a Git context.
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

/// What:
///  A merge that meets an index lock it may not clear says where its result is,
///  that the checkout is still to be finished,
///  what finishes it and what happens to the person's own edits meanwhile.
/// Why:
///  The command exits 0 without the merge in the checkout,
/// so the line must say that plainly
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 3;
/// the owner's answer for a live foreign
///  `index.lock` at `SPEC.md:2187`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aDeferredSaveFinding('feature', lock, 'refs/cli-git/kept/17', Some(60s))).toContain('refs/cli-git/kept/17');
/// ```
#[test]
fn the_line_of_a_saved_merge_names_the_side_ref_and_what_finishes_the_checkout() {
    let said: String =
        a_deferred_save_finding("feature", Path::new("/r/.git/index.lock"), "refs/cli-git/kept/1794200000000", Some(Duration::from_secs(60)));
    assert!(said.starts_with("monochromatic-git-policy-cli: git merge feature: "), "{said}");
    assert!(said.contains("/r/.git/index.lock"), "the lock is named: {said}");
    assert!(said.contains("did not let go within 60 seconds"), "the wait is named: {said}");
    assert!(said.contains("saved on refs/cli-git/kept/1794200000000"), "the side ref is named: {said}");
    assert!(said.contains("exits 0") && said.contains("branch has not moved"), "the state is stated: {said}");
    assert!(said.contains("the next git command in this worktree finishes the checkout"), "{said}");
    assert!(said.contains("edits you make meanwhile to files the merge does not change are kept"), "{said}");
    assert!(said.contains("If no program is using this repository"), "the way out of a lock nothing releases: {said}");
    // A lock this command may not clear at all waited no ceiling, so no seconds are claimed.
    let untouched: String = a_deferred_save_finding("feature", Path::new("/r/.git/index.lock"), "refs/cli-git/kept/1", None);
    assert!(untouched.contains("is not one this command may clear"), "{untouched}");
    assert!(!untouched.contains("seconds"), "no wait is claimed: {untouched}");
}

/// What:
///  A merge whose commit could not be kept on a side ref is refused,
///  with nothing changed,
///  and never promises a later finish it may not reach.
/// Why:
///  Without the name the commit can be collected,
/// so the merge must not exit 0 claiming it is saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aMergeThatCouldNotBeSaved('feature', lock)).toContain('could not be kept on a side ref');
/// ```
#[test]
fn the_line_of_a_merge_that_could_not_be_saved_changes_nothing() {
    let said: String = a_merge_that_could_not_be_saved("feature", Path::new("/r/.git/index.lock"));
    assert!(said.contains("/r/.git/index.lock"), "{said}");
    assert!(said.contains("could not be kept on a side ref"), "{said}");
    assert!(said.contains("Nothing was changed"), "{said}");
    assert!(said.contains("Run the merge again"), "{said}");
    assert!(!said.contains("the next git command"), "it promises no finish: {said}");
}

/// What:
///  A linked worktree's settlement place carries the repository's common Git directory,
///  apart from the worktree's own Git directory it writes through.
/// Why:
///  `RecoveryPlace.common_dir` names the directory every worktree of a repository shares,
///  and `RecoveryPlace.git_dir` the one of the worktree whose index recovery writes;
///  in a linked worktree the first is the main repository's `.git`
///  and the second lives under its `worktrees` directory.
///  A place that filled the shared field with the worktree's own directory would point every reader of
///  it at a directory that goes away with the worktree (`doc/decision/` has no record of the split;
///  `merge_isolation_recovery_steps.rs` states it on the type).
///  The session's common directory is the invocation's own answer,
///  and a session without one reads the worktree's `commondir` file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(recoveryPlace(session, gitDir, root).commonDir).toBe(mainRepositoryGitDir);
/// ```
#[test]
fn a_settlement_place_names_the_repositorys_common_git_directory() {
    let root: PathBuf = fixture("merge-apply-common-dir");
    let repo: PathBuf = repository(root.as_path(), "main");
    let worktree: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", worktree.to_str().expect("a text path"), "--detach"]);
    let git_dir: PathBuf = PathBuf::from(git_text(worktree.as_path(), &["rev-parse", "--absolute-git-dir"]));
    let common_dir: PathBuf = PathBuf::from(git_text(worktree.as_path(), &["rev-parse", "--path-format=absolute", "--git-common-dir"]));
    assert_ne!(git_dir, common_dir, "a linked worktree keeps its own Git directory apart from the shared one");
    let session: Session = Session { context: GitContext { common_dir: Some(common_dir.clone()), ..git_context() } };
    let place: RecoveryPlace = RecoveryPlace::settling(&session.context, git_dir.as_path(), worktree.as_path());
    assert_eq!(place.common_dir, common_dir, "the shared directory is what the session named");
    assert_eq!(place.git_dir, git_dir, "the worktree's own directory is what recovery writes through");
    // A session built without the invocation's answer reads the `commondir` file the worktree's Git directory holds.
    let bare: Session = Session { context: GitContext { common_dir: None, ..git_context() } };
    let read: RecoveryPlace = RecoveryPlace::settling(&bare.context, git_dir.as_path(), worktree.as_path());
    assert_eq!(read.common_dir, common_dir, "the worktree's own Git directory names the common one in its `commondir` file");
    remove(root.as_path());
}
