//! Controls for the lines the isolated merge's apply writes when another program holds the index.

use super::*;
/// The fixtures a linked worktree needs,
/// and the identity of a Git context.
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
/// The words a saved merge prints,
/// and what the endings state about the bytes the settle left where they are.
use crate::merge_isolation_apply_findings::{
    BytesAfterAFailedMerge, a_deferred_save_finding, a_merge_in_the_branch_whose_carry_waits, a_merge_partly_installed_behind_the_lock,
    a_merge_that_could_not_be_kept, a_merge_the_branch_moved_under,
};

/// What:
///  A merge that meets an index lock it may not clear says where its result is,
///  that the checkout is still to be finished,
///  what finishes it,
///  what the files hold and where the person's own bytes are.
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
/// expect(aDeferredSaveFinding('feature', lock, 'refs/cli-git/kept/17', why, bytes)).toContain('refs/cli-git/kept/17');
/// ```
#[test]
fn the_line_of_a_saved_merge_names_the_side_ref_and_what_finishes_the_checkout() {
    let bytes: BytesAfterAFailedMerge<'_> = BytesAfterAFailedMerge { the_paths_were_cleared: true, kept_on: Some("refs/cli-git/carried/1") };
    let said: String = a_deferred_save_finding(
        "feature",
        Path::new("/r/.git/index.lock"),
        "refs/cli-git/kept/1794200000000",
        &WhyTheMergeIsSaved::TheWaitEnded(std::time::Duration::from_secs(60)),
        &bytes,
    );
    assert!(said.starts_with("monochromatic-git-policy-cli: git merge feature: "), "{said}");
    assert!(said.contains("/r/.git/index.lock"), "the lock is named: {said}");
    assert!(said.contains("did not let go within 60 seconds"), "the wait is named: {said}");
    assert!(said.contains("saved on refs/cli-git/kept/1794200000000"), "the side ref is named: {said}");
    assert!(said.contains("The branch has not moved and nothing of the merge is in the checkout"), "the state is stated: {said}");
    assert!(said.contains("hold the branch's content where your own bytes were"), "the files' state is stated: {said}");
    assert!(said.contains("refs/cli-git/carried/1"), "where the bytes are is stated: {said}");
    assert!(said.contains("exits 0"), "{said}");
    assert!(said.contains("The next git command in this worktree finishes the checkout"), "{said}");
    assert!(said.contains("If no program is using this repository"), "the way out of a lock nothing releases: {said}");
    // A lock this command may not clear at all waited no ceiling, so no seconds are claimed.
    let untouched: String = a_deferred_save_finding(
        "feature",
        Path::new("/r/.git/index.lock"),
        "refs/cli-git/kept/1",
        &WhyTheMergeIsSaved::NotTheCommandsToClear,
        &BytesAfterAFailedMerge { the_paths_were_cleared: false, kept_on: None },
    );
    assert!(untouched.contains("is not one this command may clear"), "{untouched}");
    assert!(!untouched.contains("seconds"), "no wait is claimed: {untouched}");
    assert!(!untouched.contains("hold the branch's content"), "a merge that changed nothing claims no cleared path: {untouched}");
}

/// What:
///  A merge whose commit could not be kept on a side ref says that it is not saved,
///  what the files hold,
///  and that the next command puts the person's own bytes back,
///  and never claims that nothing was changed.
/// Why:
///  Without the name the commit can be collected,
/// so the merge cannot promise a later finish;
/// the clear may already have written the branch's content over the person's edit,
/// so "Nothing was changed" would be false
///  (the review of `ce51914d2a`,
/// its first finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = aMergeThatCouldNotBeKept('feature', lock, bytes); expect(said).not.toContain('Nothing was changed');
/// ```
#[test]
fn the_line_of_a_merge_that_could_not_be_kept_says_what_stands() {
    let bytes: BytesAfterAFailedMerge<'_> = BytesAfterAFailedMerge { the_paths_were_cleared: true, kept_on: Some("refs/cli-git/carried/1") };
    let said: String = a_merge_that_could_not_be_kept("feature", Path::new("/r/.git/index.lock"), &bytes);
    assert!(said.contains("/r/.git/index.lock"), "{said}");
    assert!(said.contains("not saved for a later command"), "{said}");
    assert!(said.contains("The branch has not moved and nothing of the merge is in the checkout"), "{said}");
    assert!(said.contains("hold the branch's content where your own bytes were") && said.contains("refs/cli-git/carried/1"), "{said}");
    assert!(said.contains("The next git command in this worktree puts your own bytes back"), "{said}");
    assert!(said.contains("`git merge feature`"), "the command that repeats the merge is named: {said}");
    assert!(!said.contains("Nothing was changed"), "no line claims a checkout the clear changed: {said}");
}

/// What:
///  The line for a merge whose branch moved says the branch moved,
///  and never that it has not moved.
/// Why:
///  The same reading of `IndexHeld` reached this instant,
/// and the line it used to print claimed an unmoved branch and a finish the moved branch breaks
///  (the review of `ce51914d2a`,
/// its second finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = aMergeTheBranchMovedUnder('feature', lock, bytes); expect(said).not.toContain('has not moved');
/// ```
#[test]
fn the_line_of_a_merge_whose_branch_moved_states_the_move() {
    let bytes: BytesAfterAFailedMerge<'_> = BytesAfterAFailedMerge { the_paths_were_cleared: true, kept_on: Some("refs/cli-git/carried/1") };
    let said: String = a_merge_the_branch_moved_under("feature", Path::new("/r/.git/index.lock"), &bytes);
    assert!(said.contains("the branch moved while this merge was prepared"), "{said}");
    assert!(said.contains("this merge does not land"), "{said}");
    assert!(!said.contains("has not moved"), "the line never claims an unmoved branch: {said}");
    assert!(said.contains("refs/cli-git/carried/1"), "{said}");
    assert!(said.contains("`git merge feature`"), "the command that repeats the merge is named: {said}");
}

/// What:
///  The lines for a merge already in the branch and for one partly installed say which of the two stands.
/// Why:
///  A landed merge has only its carry left,
/// and a partly installed one is already in the checkout,
/// so neither may say that nothing of the merge is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aMergeInTheBranch(...)).toContain('the merge is in the branch'); expect(aMergePartlyInstalled(...)).toContain('part of the merge');
/// ```
#[test]
fn the_lines_of_a_landed_merge_and_a_partly_installed_one_say_which_stands() {
    let bytes: BytesAfterAFailedMerge<'_> = BytesAfterAFailedMerge { the_paths_were_cleared: false, kept_on: Some("refs/cli-git/carried/1") };
    let landed: String = a_merge_in_the_branch_whose_carry_waits("feature", Path::new("/r/.git/index.lock"), &bytes);
    assert!(landed.contains("the merge is in the branch"), "{landed}");
    assert!(landed.contains("carried into the files yet"), "{landed}");
    assert!(!landed.contains("nothing of the merge"), "a landed merge is in the checkout: {landed}");
    let partly: String = a_merge_partly_installed_behind_the_lock("feature", Path::new("/r/.git/index.lock"), &bytes);
    assert!(partly.contains("part of the merge is in the index and the worktree"), "{partly}");
    assert!(partly.contains("The next git command in this worktree finishes it"), "{partly}");
    assert!(!partly.contains("nothing of the merge"), "a partly installed merge is in the checkout: {partly}");
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
