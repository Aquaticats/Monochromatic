//! What:
//!  A commit that concludes a merge,
//! a cherry-pick or a revert,
//! and an amend,
//! whose branch another command moved while the commit waited.
//! Why:
//!  The owner,
//! on this exact failure (2026-11-09):
//! "I would consider this to be a defect in our wrapper.
//! Agents should never wait on each other on commits or merges.",
//! beside "Commits should never fail."
//! and "never discard user data".
//! A merge is open in a checkout several sessions share,
//! another session lands a commit of its own paths,
//! and the commit that concludes the merge must land on the branch as it is now:
//! its first parent is the new tip,
//! the merged commits follow,
//! and the tree is the replay of the prepared one.
//! An amend whose branch moved lands its changes as a new commit on top,
//! leaving the commits already there as they are.
//! Every control runs the built executable against real Git in a disposable repository,
//! held at the phase marker between preparation and landing,
//! so the other command lands in exactly that window.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = commitPausedAtPrepare('commit -m conclude'); await anotherSessionCommits(); release(held);
//! expect(await held).toLandOnTheNewTip();
//! ```

use super::commit::{git_text, remove_settled};
use super::hold_at_prepare::{Ended, Hold, Held, author_of, held_commit, marker_directory, parents, release, stored, wait_held, write};
use super::service_commit::{Running, start, through_service};
use super::support::{Fixture, fixture, git, repository};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A repository with three tracked files where `git merge --no-commit --no-ff other` is unfinished:
/// `y.txt` is staged by the merge,
/// `z.txt` is the last commit of `main`,
/// and `x.txt` is a tracked file the merge does not touch.
fn unfinished_merge(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    for file in ["x.txt", "y.txt", "z.txt"] {
        write(&repo, file, "base\n");
    }
    git(fixture, &repo, &["add", "--", "x.txt", "y.txt", "z.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(fixture, &repo, &["switch", "--quiet", "--create", "other"]);
    write(&repo, "y.txt", "other\n");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=other"]);
    git(fixture, &repo, &["switch", "--quiet", "main"]);
    write(&repo, "z.txt", "main\n");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=main"]);
    git(fixture, &repo, &["merge", "--no-commit", "--no-ff", "other"]);
    return repo;
}

/// The commit the merge in progress merges,
/// read from the state the merge left.
fn merge_head(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["rev-parse", "MERGE_HEAD"]).trim().to_owned();
}

/// What:
///  A merge open in the checkout whose branch moves before the commit that concludes it lands.
/// Why:
///   The first parent is the new tip,
/// the merged commits follow it,
/// and the tree holds what the merge resolved and what the other session landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mergeNoCommit(); const held = commitPaused('conclude'); await otherSessionCommits('x.txt'); release(held);
/// ```
#[test]
fn a_merge_conclusion_lands_on_a_branch_that_moved_while_it_was_prepared() {
    let fixture: Fixture = fixture("merge-conclusion-moved");
    let repo: PathBuf = unfinished_merge(&fixture);
    let merged: String = merge_head(&fixture, &repo);
    let before: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let name: String = format!("mmb-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(&fixture, &repo, name.as_str(), markers.as_path(), &Hold { arguments: &["commit", "--message=conclude the merge"], stdin: None, environment: &[] });
    wait_held(&mut held);
    write(&repo, "x.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "x.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    assert_ne!(moved, before, "the branch moved while the merge was prepared");
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(parents(&fixture, &repo), vec![moved, merged], "the new tip first, then the merged commit");
    assert!(!repo.join(".git/MERGE_HEAD").exists(), "the merge is concluded");
    assert!(!repo.join(".git/MERGE_MSG").exists(), "the merge message is gone with the merge");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:x.txt"]), "landed by another session\n", "nothing the moved commit landed is lost");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:y.txt"]), "other\n", "the merged commit's change");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:z.txt"]), "main\n");
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1", "--untracked-files=all"]), "", "the index and worktree agree with the commit");
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// Runs Git inside a fixture and requires the status the control expects,
/// for a command whose nonzero exit is the state the control wants.
fn git_expecting(fixture: &Fixture, repo: &Path, arguments: &[&str], expected: i32) {
    let output: std::process::Output = super::support::direct(fixture).current_dir(repo).args(arguments).output().expect("native Git fixture command");
    assert_eq!(
        output.status.code(),
        Some(expected),
        "git {arguments:?} exited {}: {}",
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// What:
///  A repository where `picked.txt` was changed on `other` and again on `main`,
///  with `git cherry-pick other` stopped on that conflict and the conflicted file
///  resolved and staged,
/// and `x.txt` as a tracked file the pick does not touch.
/// Why:
///   `git cherry-pick --no-commit` leaves no `CHERRY_PICK_HEAD` and makes no conclusion;
/// a pick stopped on a conflict does,
/// and it is the case whose concluding commit waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cherryPickThatConflicts(); write('picked.txt', 'resolved\n'); await git('add', 'picked.txt');
/// ```
fn unfinished_cherry_pick(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    for file in ["x.txt", "picked.txt"] {
        write(&repo, file, "base\n");
    }
    git(fixture, &repo, &["add", "--", "x.txt", "picked.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(fixture, &repo, &["switch", "--quiet", "--create", "other"]);
    write(&repo, "picked.txt", "other\n");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=pick me"]);
    git(fixture, &repo, &["switch", "--quiet", "main"]);
    write(&repo, "picked.txt", "main\n");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=main"]);
    git_expecting(fixture, &repo, &["cherry-pick", "other"], 1);
    write(&repo, "picked.txt", "resolved\n");
    git(fixture, &repo, &["add", "--", "picked.txt"]);
    assert!(repo.join(".git/CHERRY_PICK_HEAD").exists(), "the pick waits for its commit");
    return repo;
}

/// A cherry-pick conclusion lands on the branch as it is now,
/// as one commit on the new tip.
#[test]
fn a_cherry_pick_conclusion_lands_on_a_branch_that_moved_while_it_was_prepared() {
    let fixture: Fixture = fixture("cherry-pick-conclusion-moved");
    let repo: PathBuf = unfinished_cherry_pick(&fixture);
    let name: String = format!("cpm-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(&fixture, &repo, name.as_str(), markers.as_path(), &Hold { arguments: &["commit", "--message=pick it"], stdin: None, environment: &[] });
    wait_held(&mut held);
    write(&repo, "x.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "x.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(parents(&fixture, &repo), vec![moved], "one commit on the new tip");
    assert!(!repo.join(".git/CHERRY_PICK_HEAD").exists(), "the cherry-pick is concluded");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:picked.txt"]), "resolved\n", "the resolution the pick stopped for");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:x.txt"]), "landed by another session\n", "nothing the moved commit landed is lost");
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1", "--untracked-files=all"]), "", "the index and worktree agree with the commit");
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// What:
///  A repository with `reverted.txt` changed by its last commit on `main`,
///  a `git revert --no-commit` of that commit,
/// and `x.txt` as a tracked file the revert does not touch.
/// Why:
///   A revert conclusion is a conclusion whose state lives in `REVERT_HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await revertNoCommit('HEAD'); expect(join(repo, '.git/REVERT_HEAD')).toExist();
/// ```
fn unfinished_revert(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    for file in ["x.txt", "reverted.txt"] {
        write(&repo, file, "base\n");
    }
    git(fixture, &repo, &["add", "--", "x.txt", "reverted.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    write(&repo, "reverted.txt", "changed\n");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=change it"]);
    git(fixture, &repo, &["revert", "--no-commit", "HEAD"]);
    assert!(repo.join(".git/REVERT_HEAD").exists(), "the revert waits for its commit");
    return repo;
}

/// What:
///  A revert whose branch moved under it,
/// where the command that moved the branch is the one Git lets conclude a revert:
/// the commit of another session's paths.
/// Why:
///   Real Git 2.56.0 concludes a revert with a partial commit and removes `REVERT_HEAD` as it does
/// (measured 2026-11-09:
/// `git revert --no-commit HEAD` then `git commit -- x.txt` leaves no `REVERT_HEAD`),
/// and a `git merge` beside it removes the state too and fails.
/// So no commit can move a revert's branch and leave the revert waiting,
/// and what the wrapper owes the person is to say that the operation it was concluding is gone,
/// instead of a line about a branch it could not move or a commit it invented.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = commitPaused('commit -m revert it'); await otherSessionCommits('x.txt'); release(held);
/// expect(held.stderr).toSayTheRevertIsGone();
/// ```
#[test]
fn a_revert_whose_branch_moved_says_the_operation_it_concluded_is_gone() {
    let fixture: Fixture = fixture("revert-conclusion-moved");
    let repo: PathBuf = unfinished_revert(&fixture);
    let name: String = format!("rvm-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(&fixture, &repo, name.as_str(), markers.as_path(), &Hold { arguments: &["commit", "--message=revert it"], stdin: None, environment: &[] });
    wait_held(&mut held);
    write(&repo, "x.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "x.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    assert!(!repo.join(".git/REVERT_HEAD").exists(), "the other session's commit concluded the revert, as real Git does");
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(1), "{}", ended.stderr);
    assert!(ended.stderr.contains("changed while it was being prepared"), "the person is told the operation is gone: {}", ended.stderr);
    assert!(!ended.stderr.contains("cannot be moved onto a different last commit"), "no line about a commit that cannot be moved: {}", ended.stderr);
    assert_eq!(git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim(), moved, "nothing else landed");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:x.txt"]), "landed by another session\n");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:reverted.txt"]), "changed\n", "the other session took only its own path");
    assert_eq!(git_text(&fixture, &repo, &["diff", "--cached", "--name-only"]), "reverted.txt\n", "the reverted change stays staged for its owner");
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// What:
///  A repository whose last commit `replaced` has its own author,
/// the repository's committer being the fixture's,
/// with an edit of `a.txt` the amend takes by naming it,
/// and `b.txt` as a tracked file the amend does not touch.
/// Why:
///   The amended commit's author must survive the amend that became a new commit,
/// so the control gives the replaced commit another author than the committer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await commitAs('Original Author <original@example.com>', 'replaced'); write('a.txt', 'one\ntwo\n'); await git('add', 'a.txt');
/// ```
fn repository_for_an_amend(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    write(&repo, "a.txt", "one\n");
    write(&repo, "b.txt", "base\n");
    git(fixture, &repo, &["add", "--", "a.txt", "b.txt"]);
    git(
        fixture,
        &repo,
        &["commit", "--quiet", "--author=Original Author <original@example.com>", "--message=replaced"],
    );
    write(&repo, "a.txt", "one\ntwo\n");
    return repo;
}

/// What:
///  An amend whose branch moved lands its changes as a new commit on top,
/// with the amended message and author,
/// and says so.
/// Why:
///   The owner chose exactly this (verbatim,
/// 2026-11-09):
/// "A:
/// land as a new commit (Recommended)",
/// rejecting both the refusal and a rewrite of the commit another session landed.
/// The result is the replaced commit,
/// then the commit that moved the branch,
/// then the amendment as a commit of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const x = head(); const held = commitPaused(['commit', '--amend', '-m', 'amended']); await otherSessionCommits('b.txt'); release(held);
/// expect(head().parents).toEqual([y]); expect(head().message).toBe('amended');
/// ```
#[test]
fn an_amend_lands_as_a_new_commit_when_the_branch_moved_while_it_was_prepared() {
    let fixture: Fixture = fixture("amend-on-a-moved-branch");
    let repo: PathBuf = repository_for_an_amend(&fixture);
    let replaced: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let replaced_fields: Vec<String> = stored(&fixture, &repo, "HEAD");
    let replaced_author: String = author_of(&fixture, &repo, replaced.as_str());
    let name: String = format!("amb-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let markers: PathBuf = marker_directory(&fixture);
    let mut held: Held = held_commit(&fixture, &repo, name.as_str(), markers.as_path(), &Hold { arguments: &["commit", "--amend", "--message=amended", "--", "a.txt"], stdin: None, environment: &[] });
    wait_held(&mut held);
    write(&repo, "b.txt", "landed by another session\n");
    let landed = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "b.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    let moved: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let moved_fields: Vec<String> = stored(&fixture, &repo, "HEAD");
    let ended: Ended = release(held);
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(parents(&fixture, &repo), vec![moved.clone()], "the amendment sits on the branch as it is now");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]), "amended\n");
    assert_eq!(author_of(&fixture, &repo, "HEAD"), replaced_author, "the author of the replaced commit survives");
    assert!(replaced_author.starts_with("Original Author <original@example.com>|"), "the control gave the replaced commit its own author: {replaced_author}");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:a.txt"]), "one\ntwo\n", "the amendment");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:b.txt"]), "landed by another session\n", "nothing the moved commit landed is lost");
    assert_eq!(stored(&fixture, &repo, replaced.as_str()), replaced_fields, "the replaced commit is left as it is");
    assert_eq!(stored(&fixture, &repo, moved.as_str()), moved_fields, "the commit that moved the branch is left as it is");
    assert_eq!(git_text(&fixture, &repo, &["rev-list", "--count", "HEAD"]), "4\n", "the first commit, the replaced commit, the one that moved it, and the amendment");
    assert!(ended.stderr.contains("new commit"), "the person is told what the amend became: {}", ended.stderr);
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1", "--untracked-files=all"]), "", "the index and worktree agree with the commit");
    git(&fixture, &repo, &["fsck", "--strict", "--no-dangling"]);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
