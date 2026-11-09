//! What:
//!  A commit that concludes a merge, a cherry-pick or a revert, and an amend,
//! whose branch another command moved while the commit waited.
//! Why:
//!  The owner, on this exact failure (2026-11-09):
//! "I would consider this to be a defect in our wrapper.
//! Agents should never wait on each other on commits or merges.",
//! beside "Commits should never fail." and "never discard user data".
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
use super::service_commit::{Running, runtime_for, start, through_service};
use super::support::{Fixture, bounded_for, fixture, git, repository, wrapper_first_path};
use super::supervisor::{IdleHold, hold_idle_judgment};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The longest a control waits for a marker,
/// in seconds,
/// far beyond a healthy run.
/// Why:
///   The preparation of a conclusion runs several Git starts and a policy pass,
/// and this host under a load average above a hundred took most of a minute for one of them;
/// the wait is bounded, never a hang,
/// and the command's own bound is the one that ends work that really is stuck.
const PATIENCE_SECONDS: u64 = 300;

/// The time bound on the held commit of a control,
/// in seconds.
/// Why:
///   A conclusion prepared again builds itself twice and replays a tree between the builds,
/// and a host under a load average above a hundred took more than the commit bound of every other control,
/// which ended the command with `SIGKILL` and hid what the commit came to.
const HELD_BOUND_SECONDS: &str = "600";

/// How many arrivals at the phase marker the control releases in advance:
/// the bound on how often a commit is prepared again.
const LATER_ARRIVALS: u32 = 4;

/// What a finished wrapped commit came to.
struct Ended {
    code: Option<i32>,
    stderr: String,
}

/// A wrapped commit held where it has prepared its commit and has not yet sent it.
struct Held {
    child: Child,
    /// Everything the commit wrote to its standard error,
    /// once it has ended.
    stderr: Option<std::thread::JoinHandle<String>>,
    markers: PathBuf,
    /// The hold that suspends the hang guard's idle rule while this control keeps the commit waiting on purpose.
    /// The commit waits for a file this control writes,
    /// which the guard cannot tell from an endless wait on something outside the tree.
    hold: Option<IdleHold>,
}

impl Held {
    /// Everything the commit wrote to its standard error,
    /// read once the commit has ended.
    fn ended(&mut self) -> String {
        let Some(reader) = self.stderr.take() else { return String::new() };
        return reader.join().expect("the reader of standard error");
    }
}

/// The directory a control's phase markers live in,
/// with every later arrival released in advance.
fn marker_directory(fixture: &Fixture) -> PathBuf {
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("the marker directory");
    for number in 2..=LATER_ARRIVALS {
        std::fs::write(markers.join(format!("service-prepared-{number}.release")), "").expect("release of a later arrival");
    }
    return markers;
}

/// What:
///  Starts a wrapped commit and holds it at the marker between its preparation and the service.
/// Why:
///  The other command must land while this one has captured its commit and not landed it,
/// which is the one instant the failure needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = startPaused({ arguments: ['commit', '-m', 'conclude'], markers });
/// await holdsAt(held, 'service-prepared-1');
/// ```
fn held_commit(fixture: &Fixture, repo: &Path, name: &str, markers: &Path, arguments: &[&str]) -> Held {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        HELD_BOUND_SECONDS,
    );
    command.current_dir(repo).args(arguments);
    command
        .env("XDG_RUNTIME_DIR", runtime_for(name))
        .env(PHASE_SIGNAL_VARIABLE, format!("service-prepared:pause:{}", markers.display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child: Child = command.spawn().expect("the wrapped commit starts");
    let stderr = child.stderr.take().expect("piped standard error");
    let reader = std::thread::spawn(move || {
        let mut all: String = String::new();
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            all.push_str(&line);
            all.push('\n');
        }
        return all;
    });
    let hold: IdleHold = hold_idle_judgment(child.id());
    return Held { child, stderr: Some(reader), markers: markers.to_path_buf(), hold: Some(hold) };
}

/// Waits until the commit is held at the marker between its preparation and its landing,
/// and fails the control with the commit's own words when it instead ended.
fn wait_held(held: &mut Held) {
    let reached: PathBuf = held.markers.join("service-prepared-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        if let Some(status) = held.child.try_wait().expect("the commit can be asked whether it ended") {
            let words: String = held.ended();
            panic!("the commit ended ({status:?}) before it was held: {words}");
        }
        assert!(
            began.elapsed() < Duration::from_secs(PATIENCE_SECONDS),
            "never happened: the commit prepared and is held before the service"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Lets the held commit go on to the service and returns what it came to.
fn release(mut held: Held) -> Ended {
    std::fs::write(held.markers.join("service-prepared-1.release"), "").expect("release");
    let status = held.child.wait().expect("the commit ends");
    // The guard judges the commit again once this control is no longer holding it.
    let _held_again: Option<IdleHold> = held.hold.take();
    let stderr: String = held.ended();
    return Ended { code: status.code(), stderr };
}

/// What one commit of a branch is,
/// so a control can say a commit another command made was left as it is.
fn stored(fixture: &Fixture, repo: &Path, revision: &str) -> Vec<String> {
    let formats: [&str; 6] = ["%H", "%P", "%T", "%B", "%an <%ae>|%cn <%ce>", "%aI|%cI"];
    return formats.map(|format| { return git_text(fixture, repo, &["log", "-1", "--date=iso-strict", format!("--format={format}").as_str(), revision]); }).to_vec();
}

/// The author of a commit,
/// with the date the commit records,
/// as the person reads it.
fn author_of(fixture: &Fixture, repo: &Path, revision: &str) -> String {
    return git_text(fixture, repo, &["log", "-1", "--date=iso-strict", "--format=%an <%ae>|%aI", revision]);
}

/// The parents of `HEAD`,
/// as object names.
fn parents(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return git_text(fixture, repo, &["rev-list", "--parents", "-n", "1", "HEAD"]).split_whitespace().skip(1).map(String::from).collect();
}

/// Writes one file of the repository.
fn write(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("the file");
}

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
    let mut held: Held = held_commit(&fixture, &repo, name.as_str(), markers.as_path(), &["commit", "--message=conclude the merge"]);
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
    let mut held: Held = held_commit(&fixture, &repo, name.as_str(), markers.as_path(), &["commit", "--message=pick it"]);
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
    let mut held: Held = held_commit(&fixture, &repo, name.as_str(), markers.as_path(), &["commit", "--message=revert it"]);
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
///   The owner chose exactly this (verbatim, 2026-11-09):
/// "A: land as a new commit (Recommended)",
/// rejecting both the refusal and a rewrite of the commit another session landed.
/// The result is the replaced commit, then the commit that moved the branch,
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
    let mut held: Held = held_commit(&fixture, &repo, name.as_str(), markers.as_path(), &["commit", "--amend", "--message=amended", "--", "a.txt"]);
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
