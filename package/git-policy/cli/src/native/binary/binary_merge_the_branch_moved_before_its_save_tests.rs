//! What:
//!  A merge whose branch moves between its preparation and its landing,
//!  while another command holds the index,
//!  through the built executable:
//!  the command refuses and says so,
//!  keeps nothing,
//!  and the next run lands the merge on the branch as it is now.
//! Why:
//!  On 2026-11-25 a `git merge` in the main checkout returned 0 and landed nothing (issue 6473):
//!  the merge commit was made on the head `main` held,
//!  another session's commit landed on `main` in the same second,
//!  and the commit service held `index.lock` while it landed.
//!  Git's own `merge --ff-only` could not take the lock,
//!  so the command saved its merge
//!  on `refs/cli-git/kept/<n>` and exited 0 with a line that said the branch had not moved.
//!  A save is a promise that the next git command finishes this very merge,
//!  and no command can finish one built on a head the branch has moved past,
//!  so a caller read success where nothing had landed: the merge was reported, the branch was deleted on
//!  origin, and it had to be pushed back.
//!  The save therefore reads the branch first and refuses when it moved,
//!  which is the one ending of a merge that changes nothing and leaves nothing to finish.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paused = pausedAt('merge-intent'); movingCommit(); write('.git/index.lock', '');
//! const ran = paused.released('merge-intent'); expect(ran.code).toBe(BRANCH_MOVED_EXIT_CODE);
//! expect(ran.stderr).toContain('main moved while this merge was prepared'); expect(ran.stderr).toContain('running the merge again is safe');
//! expect(sideRefs()).toBe(''); expect(records()).toBe(''); rm(lock); await wrapped(['merge', '--no-edit', 'feature']); expect(merged());
//! ```

use super::commit::{
    git_text,
    remove_settled,
    wrapped_commit,
};
use super::merge_carries_edits::{
    FEATURE_THIRD,
    assert_merged,
    diverged,
    on_main_a,
    records,
    side_refs,
    status,
    text_at,
};
use super::merge_carries_edits_kill::{
    Paused,
    paused_at,
};
use super::support::{
    Fixture,
    Observed,
    fixture,
    git,
    observe,
};
use git_policy_cli::merge_isolation_run_types::BRANCH_MOVED_EXIT_CODE;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Output;

/// The lock file of the repository's index,
/// which the commit service holds while it lands another session's commit.
fn lock_of(repo: &Path) -> PathBuf {
    return repo.join(".git/index.lock");
}

/// What:
///  A commit lands on the branch in the window between this merge's preparation and its landing,
///  and another command holds the index throughout,
///  which is the incident of 2026-11-25 exactly.
/// Why:
///  The merge commit is built on the head the branch held,
/// so neither Git's own fast-forward nor a later command can put it
///  on the branch once another commit is there;
///  the command must say so and exit non-zero,
///  and the run after it must land the merge,
///  because that is what the line promises.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paused = pausedAt('merge-intent'); movingCommit(); write('.git/index.lock', '');
/// const ran = paused.released('merge-intent'); expect(ran.code).toBe(BRANCH_MOVED_EXIT_CODE);
/// expect(sideRefs()).toBe(''); expect(records()).toBe(''); rm(lock); await wrapped(['merge', '--no-edit', 'feature']); expect(merged());
/// ```
#[test]
fn a_merge_whose_branch_moved_before_it_could_be_saved_refuses_and_the_next_run_lands_it() {
    let fixture: Fixture = fixture("merge-branch-moved-before-save");
    let repo: PathBuf = diverged(&fixture, "repo");
    let held: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"])
        .trim()
        .to_owned();
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-intent");
    // Another session's commit, by plumbing, while this merge holds the commit it prepared and has moved nothing.
    let tree: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"])
        .trim()
        .to_owned();
    let made: Vec<u8> = git(
        &fixture,
        repo.as_path(),
        &[
            "commit-tree",
            tree.as_str(),
            "-p",
            held.as_str(),
            "-m",
            "a concurrent commit",
        ],
    )
    .stdout;
    let concurrent: String = String::from_utf8_lossy(made.as_slice()).trim().to_owned();
    git(
        &fixture,
        repo.as_path(),
        &["update-ref", "refs/heads/main", concurrent.as_str()],
    );
    // The command that landed it holds the index while it works, so Git's own fast-forward cannot take the lock either.
    std::fs::write(lock_of(repo.as_path()), b"").expect("the other command holds the index");
    let output: Output = paused.released("merge-intent");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(
        output.status.code(),
        Some(BRANCH_MOVED_EXIT_CODE),
        "a merge that did not land does not report success: {said}"
    );
    assert!(
        said.contains("main moved while this merge was prepared"),
        "the line names the branch and the move: {said}"
    );
    assert!(
        said.contains("running the merge again is safe"),
        "the line says the next run is safe: {said}"
    );
    assert!(
        !said.contains("The branch has not moved"),
        "the line never claims an unmoved branch: {said}"
    );
    assert!(
        !said.contains("saved on refs/cli-git/kept/"),
        "no side ref is written for a merge that cannot land: {said}"
    );
    assert_eq!(
        side_refs(&fixture, repo.as_path()),
        "",
        "no side ref is kept: {said}"
    );
    assert_eq!(records(repo.as_path()), "", "no record stays: {said}");
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"])
            .trim()
            .to_owned(),
        concurrent,
        "the branch is the other session's commit"
    );
    std::fs::remove_file(lock_of(repo.as_path())).expect("the other command lets go");
    assert_eq!(
        status(&fixture, repo.as_path()),
        "",
        "nothing of the merge is in the checkout"
    );
    assert_eq!(
        text_at(repo.as_path(), "a.txt"),
        Some(on_main_a()),
        "the person's file holds what the branch holds"
    );
    // What the line promised: the merge runs again on the branch as it is now, and lands there.
    let again: Observed = observe(
        &mut wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]),
        b"",
    );
    let merged: String = String::from_utf8_lossy(again.stderr.as_slice()).into_owned();
    assert_eq!(
        again.code,
        Some(0),
        "running the merge again is safe: {merged}"
    );
    assert_merged(
        &fixture,
        repo.as_path(),
        "the merge run again after the branch moved",
    );
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(
        after.contains(FEATURE_THIRD),
        "the merge's line is in the file: {after:?}"
    );
    remove_settled(&fixture);
}
