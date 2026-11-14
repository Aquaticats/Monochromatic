//! What:
//!  A lock that another program takes in the window between a merge's look at the index and its last step,
//!  through the built executable,
//!  and what the command ends in.
//! Why:
//!  The owner's rules (`SPEC.md`,
//! "Merges and commits never fail"):
//! "nothing the machine or other processes do ends them with a
//!  nonzero status",
//! which covers a held `index.lock`,
//! and "the index and the worktree hold either what they held before the
//!  command or the whole result".
//! A merge that carries the person's own edit clears the paths the merge changes and then runs Git's own
//!  update;
//! a lock taken in that window made Git's update fail,
//! and the putting back of the person's bytes met the same
//!  lock,
//! so the command exited 1 with a finding that said the worktree was as it was (`merge-isolation/not-applied`).
//! The lock is put in place at the instant the review's reproductions use:
//! the phase marker `merge-cleared`,
//! with the clear written and Git's update not yet asked for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paused = pausedAt('merge-cleared'); write('.git/index.lock', ''); const ran = paused.released('merge-cleared');
//! expect(ran.code).toBe(0);
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::merge_carries_edits::{
    FEATURE_THIRD, PERSON_LINE, assert_merged, index_entry, on_main_a, records, side_refs, status, text_at,
};
use super::merge_carries_edits_kill::{Paused, edited, paused_at};
use super::support::{Fixture, Observed, fixture, observe, git};
use std::path::{Path, PathBuf};
use std::process::Output;

/// The lock file of the repository's index.
fn lock_of(repo: &Path) -> PathBuf {
    return repo.join(".git/index.lock");
}

/// The refs a merge keeps its commit on,
/// which a merge that cannot be applied writes none of.
fn kept_refs(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["for-each-ref", "--format=%(refname)", "refs/cli-git/kept"]);
}

/// The PID file `core.lockfilePid` leaves beside the lock,
/// naming a process that does not run.
fn a_pid_file_of_a_gone_process(repo: &Path) {
    // A number no process of this host has, which is what a killed Git's PID file can name.
    std::fs::write(repo.join(".git/index~pid.lock"), b"pid 4294967\n").expect("the PID file of a process that is gone");
}

/// What:
///  A lock a killed Git left in the window after the clear is cleared,
///  the merge lands in the same command,
///  and the person's edit is carried into it.
/// Why:
///  A number this process table does not have is what a killed Git leaves,
///  and nothing here releases it,
/// so the command that meets it while putting the person's bytes back must clear it on the same proof the
///  merge's own path uses,
///  instead of exiting 1 with a line that promises a clearing it never makes.
///  The merge is made again on the person's restored bytes,
///  so the edit is carried by the merge that lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paused = pausedAt('merge-cleared'); write('.git/index.lock', ''); write('.git/index~pid.lock', 'pid 4294967\n');
/// const ran = paused.released('merge-cleared'); expect(ran.code).toBe(0); expect(existsSync(lock)).toBe(false);
/// expect(read('a.txt')).toContain('the person line'); expect(records()).toBe('');
/// ```
#[test]
fn a_lock_a_gone_git_left_after_the_clear_is_cleared_and_the_merge_lands() {
    let fixture: Fixture = fixture("merge-lock-after-clear-dead");
    let repo: PathBuf = edited(&fixture, "repo");
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-cleared");
    assert_eq!(text_at(repo.as_path(), "a.txt").as_deref(), Some(on_main_a().as_str()), "the clear wrote the branch's content over the edit");
    // The lock appears after the clear's look and after the pause was reached, which is the review's instant.
    std::fs::write(lock_of(repo.as_path()), b"").expect("a Git left the lock");
    a_pid_file_of_a_gone_process(repo.as_path());
    let output: Output = paused.released("merge-cleared");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "a leftover lock after the clear does not end the merge: {said}");
    assert!(!lock_of(repo.as_path()).exists(), "the lock a gone process left is cleared: {said}");
    assert!(said.contains("the merge clears that lock"), "the line says what the merge does with it: {said}");
    assert!(!said.contains("the branch, the index and the worktree are as they were"), "no line claims a checkout the clear changed: {said}");
    assert_merged(&fixture, repo.as_path(), "a leftover lock after the clear");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(FEATURE_THIRD), "the merge's line is in the file: {after:?}");
    assert!(after.contains(PERSON_LINE), "the person's line is in the file, not lost: {after:?}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    remove_settled(&fixture);
}

/// What:
///  A lock taken after the clear and held past the bound saves the merge,
///  exits 0,
///  and says what the checkout holds and which command puts the person's bytes back;
///  the next command lands the merge and carries them.
/// Why:
///  The lock that stopped Git's own update stops the putting back of the person's bytes as well,
/// so the worktree
///  cannot hold what it held before the command when the command returns.
/// The merge is therefore saved on a side ref and the command
///  exits 0 (`SPEC.md`,
/// "Merges and commits never fail"),
/// and the line names what the files hold,
/// where the bytes are,
/// and the command that puts them back,
/// because a line that said the checkout was as it was would be false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paused = pausedAt('merge-cleared'); write('.git/index.lock', ''); const ran = paused.released('merge-cleared');
/// expect(ran.code).toBe(0); expect(ran.stderr).toSay('hold the branch's content where your own bytes were');
/// rm(lock); await wrapped(['status']); expect(read('a.txt')).toContain('the person line');
/// ```
#[test]
fn a_lock_taken_after_the_clear_and_held_past_the_bound_saves_the_merge() {
    let fixture: Fixture = fixture("merge-lock-after-clear-held");
    let repo: PathBuf = edited(&fixture, "repo");
    let before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-cleared");
    assert_eq!(text_at(repo.as_path(), "a.txt").as_deref(), Some(on_main_a().as_str()), "the clear wrote the branch's content over the edit");
    std::fs::write(lock_of(repo.as_path()), b"").expect("another program takes the index and keeps it");
    let output: Output = paused.released("merge-cleared");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "a lock held past the bound does not end the merge: {said}");
    // The merge is saved, and the line states the checkout's real state instead of claiming the old one.
    assert!(said.contains("saved on refs/cli-git/kept/"), "the side ref is named: {said}");
    assert!(said.contains("hold the branch's content where your own bytes were"), "the line says what the files hold: {said}");
    assert!(said.contains("refs/cli-git/carried/"), "the line names where the person's bytes are: {said}");
    assert!(said.contains("those bytes are kept on refs/cli-git/carried/"), "the line says where the bytes are: {said}");
    assert!(said.contains("The next git command in this worktree finishes the checkout"), "the line names the command that finishes it: {said}");
    assert!(!said.contains("the branch, the index and the worktree are as they were"), "no line claims a checkout the clear changed: {said}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned(), before, "the branch has not moved");
    assert_eq!(text_at(repo.as_path(), "a.txt").as_deref(), Some(on_main_a().as_str()), "the path holds the branch's content until the next command");
    assert!(lock_of(repo.as_path()).exists(), "a lock another program holds is left for that program");
    // The next command lands the saved merge and puts the person's bytes back into the merge.
    std::fs::remove_file(lock_of(repo.as_path())).expect("the other program lets go");
    let landed: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]), b"");
    let finished: String = String::from_utf8_lossy(landed.stderr.as_slice()).into_owned();
    assert_eq!(landed.code, Some(0), "the next command goes on: {finished}");
    assert_merged(&fixture, repo.as_path(), "a held lock after the clear");
    let after: String = text_at(repo.as_path(), "a.txt").expect("a.txt is there");
    assert!(after.contains(FEATURE_THIRD), "the merge's line is in the file: {after:?}");
    assert!(after.contains(PERSON_LINE), "the person's line came back into the merge: {after:?}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "no ref stays behind: {finished}");
    assert!(!index_entry(&fixture, repo.as_path(), "a.txt").is_empty(), "the index holds the merge's entry: {finished}");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

/// What:
///  A merge whose side ref cannot be kept says it is not saved,
///  what the files hold,
///  where the person's own bytes are and which command puts them back,
///  and the next command does put them back.
/// Why:
///  The clear has already written the branch's content over the person's edit when the save fails,
///  so a line that said nothing was changed was false,
///  and a merge on another program's lock exited 2 where `SPEC.md`,
/// "Merges and commits never fail" says it ends 0
///  (the review of `ce51914d2a`,
/// its first finding).
///  The side ref is made to fail the way that review made it fail:
/// `refs/cli-git/kept` is a regular file,
/// so every `update-ref refs/cli-git/kept/<n>` cannot create its lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paused = pausedAt('merge-cleared'); write('.git/index.lock', ''); write('.git/refs/cli-git/kept', 'not a directory');
/// const ran = paused.released('merge-cleared'); expect(ran.code).toBe(0);
/// expect(ran.stderr).toContain('not saved for a later command'); expect(ran.stderr).not.toContain('Nothing was changed');
/// rm(lock); await wrapped(['status']); expect(read('a.txt')).toContain('the person line');
/// ```
#[test]
fn a_merge_whose_side_ref_cannot_be_kept_says_what_stands_and_the_next_command_puts_the_bytes_back() {
    let fixture: Fixture = fixture("merge-lock-after-clear-no-side-ref");
    let repo: PathBuf = edited(&fixture, "repo");
    let before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-cleared");
    std::fs::write(lock_of(repo.as_path()), b"").expect("another program takes the index and keeps it");
    // Everything the merge could be kept on is a path that cannot hold a ref, which is the review's own fault.
    std::fs::write(repo.join(".git/refs/cli-git/kept"), b"not a directory\n").expect("the side ref cannot be made");
    let output: Output = paused.released("merge-cleared");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "a lock after the clear does not fail the merge: {said}");
    assert!(said.contains("not saved for a later command"), "the line says the merge is not saved: {said}");
    assert!(said.contains("The branch has not moved and nothing of the merge is in the checkout"), "{said}");
    assert!(said.contains("hold the branch's content where your own bytes were"), "the files' state is stated: {said}");
    assert!(said.contains("refs/cli-git/carried/"), "where the bytes are is stated: {said}");
    assert!(said.contains("The next git command in this worktree puts your own bytes back"), "{said}");
    assert!(!said.contains("Nothing was changed"), "no line claims a checkout the clear changed: {said}");
    assert_eq!(text_at(repo.as_path(), "a.txt").as_deref(), Some(on_main_a().as_str()), "the clear's content stands until the next command");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned(), before, "the branch has not moved");
    // The next command does what the line said: it puts the person's own bytes back, and lands no merge.
    std::fs::remove_file(lock_of(repo.as_path())).expect("the other program lets go");
    std::fs::remove_file(repo.join(".git/refs/cli-git/kept")).expect("the fault goes");
    let after: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]), b"");
    let next: String = String::from_utf8_lossy(after.stderr.as_slice()).into_owned();
    assert_eq!(after.code, Some(0), "the next command goes on: {next}");
    assert_eq!(text_at(repo.as_path(), "a.txt").as_deref(), Some(format!("{}{PERSON_LINE}\n", on_main_a()).as_str()), "the person's bytes are back: {next}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned(), before, "the merge did not land, as the line said");
    assert_eq!(records(repo.as_path()), "", "no record stays: {next}");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "no ref stays: {next}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

/// What:
///  A lock taken after the clear while another session moves the branch says the branch moved,
///  promises no finish,
///  keeps nothing on a side ref,
///  and the next command puts the person's own bytes back.
/// Why:
///  That merge cannot land as it was prepared,
/// so the line that said the branch had not moved and that the next command finishes the checkout was false in
///  both clauses,
/// and the next command abandoned the merge to a side ref the line had promised would finish it
///  (the review of `ce51914d2a`,
/// its second finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paused = pausedAt('merge-cleared'); movingCommit(); write('.git/index.lock', '');
/// const ran = paused.released('merge-cleared'); expect(ran.code).toBe(0);
/// expect(ran.stderr).toContain('the branch moved while this merge was prepared'); expect(ran.stderr).not.toContain('has not moved');
/// expect(sideRefs()).toBe(''); rm(lock); await wrapped(['status']); expect(read('a.txt')).toContain('the person line');
/// ```
#[test]
fn a_merge_whose_branch_moved_behind_the_lock_says_so_and_lands_nothing() {
    let fixture: Fixture = fixture("merge-lock-after-clear-moved");
    let repo: PathBuf = edited(&fixture, "repo");
    let paused: Paused = paused_at(&fixture, repo.as_path(), "merge-cleared");
    // The second session: an empty commit on the branch, by plumbing, while this merge is paused.
    let parent: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let tree: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]).trim().to_owned();
    let made: Vec<u8> = git(&fixture, repo.as_path(), &["commit-tree", tree.trim(), "-p", parent.as_str(), "-m", "a concurrent commit"]).stdout;
    let concurrent: String = String::from_utf8_lossy(made.as_slice()).trim().to_owned();
    git(&fixture, repo.as_path(), &["update-ref", "refs/heads/main", concurrent.as_str()]);
    std::fs::write(lock_of(repo.as_path()), b"").expect("another program takes the index and keeps it");
    let output: Output = paused.released("merge-cleared");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "a lock after the clear does not fail the merge: {said}");
    assert!(said.contains("the branch moved while this merge was prepared"), "the line states the move: {said}");
    assert!(said.contains("this merge does not land"), "the line promises no landing: {said}");
    assert!(!said.contains("has not moved"), "the line never claims an unmoved branch: {said}");
    assert!(said.contains("refs/cli-git/carried/"), "where the bytes are is stated: {said}");
    assert!(!said.contains("saved on refs/cli-git/kept/"), "no side ref is written for a merge that cannot land: {said}");
    assert_eq!(kept_refs(&fixture, repo.as_path()), "", "no side ref is left behind: {said}");
    assert_eq!(text_at(repo.as_path(), "a.txt").as_deref(), Some(on_main_a().as_str()), "the clear's content stands until the next command");
    // The next command does what the line said: it puts the person's own bytes back, and the branch keeps the other commit.
    std::fs::remove_file(lock_of(repo.as_path())).expect("the other program lets go");
    let after: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["status", "--porcelain=v1"]), b"");
    let next: String = String::from_utf8_lossy(after.stderr.as_slice()).into_owned();
    assert_eq!(after.code, Some(0), "the next command goes on: {next}");
    assert_eq!(text_at(repo.as_path(), "a.txt").as_deref(), Some(format!("{}{PERSON_LINE}\n", on_main_a()).as_str()), "the person's bytes are back: {next}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned(), concurrent, "the branch keeps the other session's commit");
    assert_eq!(records(repo.as_path()), "", "no record stays: {next}");
    assert_eq!(side_refs(&fixture, repo.as_path()), "", "no ref stays: {next}");
    assert_eq!(status(&fixture, repo.as_path()), " M a.txt\n", "the edit is still unsaved");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}
