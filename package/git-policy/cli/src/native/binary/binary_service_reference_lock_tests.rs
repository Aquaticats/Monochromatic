//! What:
//!  A commit whose branch write meets a reference lock,
//!  which is the file a killed Git leaves beside a reference.
//! Why:
//!  Git refuses a reference update at once when a lock file stands,
//!  and the service read that refusal as a branch that moved:
//!  three refusals later the commit ended with "the branch changed 3 times while this commit was being prepared",
//!  which was false,
//!  and the lock file that blocked it was never named
//!  (2026-11-09,
//!  a `git update-ref --stdin` of the service killed while it held `refs/heads/main.lock`).
//!  A live holder lets go within its step,
//!  so the landing waits for one and lands,
//!  and a lock nothing proves gone is named with the command that clears it
//!  (`SPEC.md`,
//!  "A landing whose branch write a lock refuses").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! plant('refs/heads/main.lock'); const refused = commit(); expect(refused.stderr).toContain('rm -- '); rm(lock); expect(commit().code).toBe(0);
//! ```

use super::commit::{git_text, remove_settled};
use super::support::{Fixture, FixtureService, Observed, bounded_for, fixture, git, observe, repository, start_fixture_service, wrapper_first_path};
use super::support_fixture_service_log::service_log;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The longest a control waits for anything,
/// far beyond a healthy run.
const PATIENCE: Duration = Duration::from_secs(60);

/// The longest a commit that meets a reference lock,
/// and the command that settles it,
/// may run here.
/// A settlement does what the landing that met the lock would have done,
/// installs the index entries and marks the batch done included,
/// so it costs what a landing costs,
/// and a commit that meets a lock pays for the hold on top of that.
/// Measured on the shared development host:
/// with the load average between 85 and 150 these controls took 9 to 54 s each,
/// and at 132 to 162 each commit and each settlement took 60 to 204 s while still finishing
/// (the same controls passed alone at 179 and 204 s).
/// Five minutes is that measurement plus half,
/// and it is a guard against a hang,
/// not a measure of the work.
/// It is deliberately not the text of the shared bound of one Git command:
/// that text is the supervisor's own backstop,
/// and a command carrying it must show work every few seconds,
/// which a commit answered by the service cannot
/// (`shared_bound_misuse`,
/// `binary_supervisor.rs`).
const SETTLE_BOUND_SECONDS: &str = "300";

/// The longest a control waits for a settlement on a shared host.
const SETTLE_PATIENCE: Duration = Duration::from_secs(600);

/// The file the commits change.
const FILE: &str = "f.txt";

/// A repository with one tracked file,
/// a running service,
/// and the branch the commits go to.
fn scene(name: &str) -> (Fixture, PathBuf, FixtureService) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(FILE), "base\n").expect("the file");
    git(&fixture, &repo, &["add", "--", FILE]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return (fixture, repo, service);
}

/// Wait for a condition,
/// at most `PATIENCE`.
fn wait_for(what: &str, condition: impl Fn() -> bool) {
    return wait_for_at_most(what, PATIENCE, condition);
}

/// Wait for a condition,
/// at most `patience`.
fn wait_for_at_most(what: &str, patience: Duration, condition: impl Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < patience, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// What:
///  A lock a killed Git left,
///  as old as one is when the next command meets it.
/// Why:
///  A lock younger than the judgement's margin is looked at again rather than judged,
///  so a lock written a moment ago makes the landing poll for the whole wait
///  and leaves the command's own process tree idle for seconds,
///  which the harness's hang guard reads as a deadlock.
///  A lock a killed Git left is seconds to days old,
///  and one two minutes old is judged at once,
///  so the control depends on the code and not on that wait.
fn plant_a_lock(path: &Path) {
    std::fs::create_dir_all(path.parent().expect("the lock's directory")).expect("the lock's directory");
    let file: std::fs::File = std::fs::File::create(path).expect("the lock a killed Git leaves");
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - Duration::from_secs(120);
    file.set_modified(long_ago).expect("the lock's age");
}

/// The command a wrapped run of `arguments` in `repo` is started with.
fn wrapped_at_the_settle_bound(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> std::process::Command {
    let program: PathBuf = fixture.root.join("bin/git");
    let mut command: std::process::Command = bounded_for(fixture, program.as_path(), wrapper_first_path(fixture).as_os_str(), SETTLE_BOUND_SECONDS);
    command.current_dir(repo).args(arguments);
    return command;
}

/// Run a wrapped commit of `FILE` to its end,
/// with the bound a commit that meets a reference lock needs,
/// and return its exit status and what it wrote.
fn commit(fixture: &Fixture, repo: &Path, message: &str) -> (Option<i32>, String) {
    return commit_paths(fixture, repo, message, FILE);
}

/// Run a wrapped commit of one named path to its end,
/// with the bound a commit that meets a reference lock needs.
fn commit_paths(fixture: &Fixture, repo: &Path, message: &str, path: &str) -> (Option<i32>, String) {
    let mut command: std::process::Command = wrapped_at_the_settle_bound(fixture, repo, &["commit", "--quiet", "-m", message, "--", path]);
    let ended: Observed = observe(&mut command, b"");
    return (ended.code, String::from_utf8_lossy(ended.stderr.as_slice()).into_owned());
}

/// What:
///  The one held record of a worktree,
///  as the wrapper wrote it.
/// Why:
///  The record is what a settlement reads and what names the side reference,
///  so a control that asks about a held commit keeps the question to the state the wrapper left.
fn held_record_of(repo: &Path) -> git_policy_cli::held_commit_record::Held {
    let (records, damaged) = git_policy_cli::held_commit_record::list(repo.join(".git").as_path());
    assert_eq!(damaged, Vec::<PathBuf>::new(), "every held record reads");
    assert_eq!(records.len(), 1, "one commit is held");
    return records[0].clone();
}

/// What:
///  A commit that meets a lock a live Git holds waits for it,
///  and lands once the holder lets go,
///  with nothing for the person to do.
/// Why:
///  A lock a live Git holds is released within its own step,
///  so failing the commit on it would fail a commit that only had to wait,
///  which the owner's rule forbids.
#[test]
fn a_commit_that_meets_a_live_reference_lock_waits_for_it_and_lands() {
    let (fixture, repo, service) = scene("reference-lock-waited");
    let lock: PathBuf = repo.join(".git/refs/heads/main.lock");
    std::fs::write(&lock, "").expect("the lock a live Git holds");
    std::fs::write(repo.join(FILE), "the edit\n").expect("the edit");
    let (code, said) = std::thread::scope(|scope| {
        let waiting = scope.spawn(|| { return commit(&fixture, repo.as_path(), "waits for the lock"); });
        // The holder lets go once the landing says it waits for the lock, so the wait under control is the landing's own.
        wait_for("the landing to wait for the reference lock", || {
            return service_log(fixture.root.as_path()).contains("a reference lock stands in the way");
        });
        std::fs::remove_file(&lock).expect("the holder lets go");
        return waiting.join().expect("the commit ends");
    });
    assert_eq!(code, Some(0), "the commit lands once the lock is released: {said}");
    assert_eq!(git_text(&fixture, &repo, &["log", "--format=%s", "-2"]), "waits for the lock\nbase\n");
    let landed: String = format!("HEAD:{FILE}");
    assert_eq!(git_text(&fixture, &repo, &["show", landed.as_str()]), "the edit\n", "the captured bytes landed");
    assert!(!lock.exists(), "no lock is left");
    drop(service);
    remove_settled(&fixture);
}

/// Run one wrapped command whose output a person reads,
/// with the bound a settlement needs,
/// and return its exit status and what it wrote on standard error.
fn command(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> (Option<i32>, String) {
    let mut command: std::process::Command = wrapped_at_the_settle_bound(fixture, repo, arguments);
    let ended: Observed = observe(&mut command, b"");
    return (ended.code, String::from_utf8_lossy(ended.stderr.as_slice()).into_owned());
}

/// The subject of the commit the branch names.
fn head_subject(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["log", "--format=%s", "-1"]);
}

/// What:
///  A commit that meets a lock nothing proves gone is made and held,
///  its command exits `0`,
///  says that the branch has not moved yet,
///  names the lock,
///  and the next command that finds the lock gone moves the branch.
/// Why:
///  The owner chose "Make it,
/// hold until lock goes" (2026-11-09),
///  and the rule it answers is "Commits should never fail.":
///  the commit is written and kept,
///  nothing of the person's is lost,
///  and no lock is removed on a guess.
#[test]
fn a_commit_that_meets_a_lock_nothing_proves_gone_is_held_and_moves_the_branch_when_it_goes() {
    let (fixture, repo, service) = scene("reference-lock-held");
    let lock: PathBuf = repo.join(".git/refs/heads/main.lock");
    plant_a_lock(lock.as_path());
    std::fs::write(repo.join(FILE), "the edit\n").expect("the edit");
    let (code, said) = commit(&fixture, repo.as_path(), "held behind the lock");
    assert_eq!(code, Some(0), "the commit is made and held: {said}");
    assert!(said.contains("has not moved yet"), "the branch has not moved: {said}");
    assert!(said.contains(lock.display().to_string().as_str()), "the lock it waits for is named: {said}");
    assert!(said.contains("moves the branch"), "what the next command does is said: {said}");
    assert!(said.contains("rm -- "), "the command that clears the lock is given: {said}");
    assert_eq!(head_subject(&fixture, repo.as_path()), "base\n", "the branch has not moved yet");
    assert!(lock.exists(), "a lock nothing proves gone is left where it is: {}", lock.display());
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1"]), format!(" M {FILE}\n"), "the edit is still the person's");
    // Every command while the commit is held says what it waits for and names the lock.
    let (still, told) = command(&fixture, repo.as_path(), &["status"]);
    assert_eq!(still, Some(0), "the command itself runs: {told}");
    assert!(told.contains(lock.display().to_string().as_str()), "the held commit's lock is named on the next command: {told}");
    assert!(told.contains("has not moved yet"), "the next command says the branch has not moved: {told}");
    // The lock goes, and the next command moves the branch, once, with the bytes the commit captured.
    std::fs::remove_file(&lock).expect("the holder lets go");
    let (moved, said_now) = command(&fixture, repo.as_path(), &["status"]);
    assert_eq!(moved, Some(0), "the command that resumes the commit runs: {said_now}");
    wait_for_at_most("the held commit to land", SETTLE_PATIENCE, || { return head_subject(&fixture, repo.as_path()) == "held behind the lock\n"; });
    assert_eq!(git_text(&fixture, &repo, &["show", format!("HEAD:{FILE}").as_str()]), "the edit\n", "the captured bytes landed");
    // The settlement installs the entries under Git's index lock, which a landing holds from the index file it saw to the
    //  installation of the entries, so the index catches up even though commands ran while the lock stood.
    assert!(!said_now.contains("the next commit in this worktree sets them"), "nothing is left owed: {said_now}");
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1"]), "", "the index caught up, and the commit's bytes are what the branch and the index hold");
    let (after, said_after) = command(&fixture, repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=after the held commit"]);
    assert_eq!(after, Some(0), "the next commit in the worktree is made: {said_after}");
    assert_eq!(git_text(&fixture, &repo, &["log", "--format=%s", "-2"]), "after the held commit\nheld behind the lock\n");
    assert_eq!(git_text(&fixture, &repo, &["show", format!("HEAD:{FILE}").as_str()]), "the edit\n", "the held commit's bytes stay on the branch");
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1"]), "", "the index caught up, and the commit's own bytes are what the branch and the index hold");
    assert!(!lock.exists(), "no lock is left by the settlement");
    drop(service);
    remove_settled(&fixture);
}

/// What:
///  A held commit lands on the branch another session moved while the lock stood,
///  with both sessions' work.
/// Why:
///  The branch can move before the lock goes,
///  and the rule for a commit that lands on a moved branch is the one in
///  `doc/decision/cli-git-concurrent-commits.md`:
///  the change is carried onto the branch as it is now,
///  and nothing of either session is lost.
#[test]
fn a_held_commit_lands_on_the_branch_another_session_moved() {
    let (fixture, repo, service) = scene("reference-lock-held-moved");
    let lock: PathBuf = repo.join(".git/refs/heads/main.lock");
    plant_a_lock(lock.as_path());
    std::fs::write(repo.join(FILE), "the edit\n").expect("the edit");
    let (code, said) = commit(&fixture, repo.as_path(), "held behind the lock");
    assert_eq!(code, Some(0), "the commit is made and held: {said}");
    // The lock goes, and another program moves the branch before any command of the wrapper finds that out:
    //  the branch can move at any instant, and a held commit must carry its change onto it.
    std::fs::remove_file(&lock).expect("the holder lets go");
    std::fs::write(repo.join("other.txt"), "the other edit\n").expect("another session's file");
    git(&fixture, &repo, &["add", "--", "other.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=another session"]);
    let (resumed, said_now) = command(&fixture, repo.as_path(), &["status"]);
    assert_eq!(resumed, Some(0), "the command that resumes the commit runs: {said_now}");
    wait_for_at_most("the held commit to land on the moved branch", SETTLE_PATIENCE, || {
        return git_text(&fixture, &repo, &["log", "--format=%s", "-3"]) == "held behind the lock\nanother session\nbase\n";
    });
    assert_eq!(git_text(&fixture, &repo, &["show", format!("HEAD:{FILE}").as_str()]), "the edit\n", "the held change landed");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:other.txt"]), "the other edit\n", "the other session's work is kept");
    drop(service);
    remove_settled(&fixture);
}

/// What:
///  A second commit made while the first is held waits beside it,
///  both are said on the commands that follow,
///  and both land in the order they were made once the lock goes.
/// Why:
///  A held commit is not one a restart decides from the branch:
///  dropping it would take a commit the person made.
///  Held commits must not collect without notice,
///  so the count and each one's line are said on every command,
///  and the settled branch carries exactly the two commits with their own bytes.
#[test]
fn two_held_commits_wait_beside_each_other_and_both_land_in_order() {
    let (fixture, repo, service) = scene("reference-lock-two-held");
    let lock: PathBuf = repo.join(".git/refs/heads/main.lock");
    let other: &str = "g.txt";
    plant_a_lock(lock.as_path());
    std::fs::write(repo.join(FILE), "the first edit\n").expect("the first edit");
    let (first, said) = commit(&fixture, repo.as_path(), "held first");
    assert_eq!(first, Some(0), "the first commit is made and held: {said}");
    std::fs::write(repo.join(other), "the second edit\n").expect("the second edit");
    git(&fixture, &repo, &["add", "--", other]);
    let (second, said_second) = commit_paths(&fixture, repo.as_path(), "held second", other);
    assert_eq!(second, Some(0), "the second commit is made and held beside the first: {said_second}");
    assert!(said_second.contains("2 held commits"), "the count of held commits is said: {said_second}");
    assert_eq!(head_subject(&fixture, repo.as_path()), "base\n", "the branch has not moved");
    let (quiet, told) = command(&fixture, repo.as_path(), &["status"]);
    assert_eq!(quiet, Some(0), "the command itself runs: {told}");
    assert!(told.contains("2 held commits"), "the count is said on every command: {told}");
    assert_eq!(told.matches("is held: the branch has not moved yet").count(), 2, "each held commit has its own line: {told}");
    // The lock goes, and the next command lands both, in the order they were made.
    std::fs::remove_file(&lock).expect("the holder lets go");
    let (resumed, said_now) = command(&fixture, repo.as_path(), &["status"]);
    assert_eq!(resumed, Some(0), "the command that resumes them runs: {said_now}");
    wait_for_at_most("both held commits to land", SETTLE_PATIENCE, || {
        return git_text(&fixture, &repo, &["log", "--format=%s", "-3"]) == "held second\nheld first\nbase\n";
    });
    assert_eq!(git_text(&fixture, &repo, &["show", format!("HEAD:{FILE}").as_str()]), "the first edit\n", "the first commit's bytes landed");
    assert_eq!(git_text(&fixture, &repo, &["show", format!("HEAD:{other}").as_str()]), "the second edit\n", "the second commit's bytes landed");
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1"]), "", "the index caught up with both");
    drop(service);
    remove_settled(&fixture);
}

/// What:
///  A commit real Git built, held behind a lock, is on the side reference the answer names,
///  and real Git's maintenance keeps it.
/// Why:
///  The answer says the commit is kept at `refs/worktree/cli-git-service/<batch>` either way,
///  and the owner's rule is "never discard user data":
///  an object no reference names is what `git gc --prune=now` removes,
///  and a settlement that finds it gone cannot move the branch to it.
///  `--verbose` is a line the service does not build itself,
///  so real Git builds this commit (`service_commit_request_of.rs`),
///  which is the arm whose commit has no reference of its own.
#[test]
fn a_held_commit_real_git_built_is_kept_on_the_side_reference_and_survives_a_prune() {
    let (fixture, repo, service) = scene("reference-lock-held-git-built");
    let lock: PathBuf = repo.join(".git/refs/heads/main.lock");
    plant_a_lock(lock.as_path());
    std::fs::write(repo.join(FILE), "the edit\n").expect("the edit");
    let (code, said) = command(&fixture, repo.as_path(), &["commit", "--verbose", "--message=held behind the lock", "--", FILE]);
    assert_eq!(code, Some(0), "the commit is made and held: {said}");
    assert!(said.contains("is held"), "the commit is held: {said}");
    let record: git_policy_cli::held_commit_record::Held = held_record_of(repo.as_path());
    let held: String = record.new.clone();
    let side: String = git_policy_cli::held_commit_settle::kept_reference(record.seq);
    assert_eq!(
        git_text(&fixture, &repo, &["rev-parse", side.as_str()]).trim(),
        held,
        "the side reference the answer names holds the held commit: {said}"
    );
    git(&fixture, &repo, &["gc", "--prune=now"]);
    assert_eq!(git_text(&fixture, &repo, &["cat-file", "-t", held.as_str()]), "commit\n", "the held commit survives a maintenance that drops what no reference names");
    std::fs::remove_file(&lock).expect("the holder lets go");
    let (resumed, said_now) = command(&fixture, repo.as_path(), &["status"]);
    assert_eq!(resumed, Some(0), "the command that settles the commit runs: {said_now}");
    wait_for_at_most("the held commit to land", SETTLE_PATIENCE, || { return head_subject(&fixture, repo.as_path()) == "held behind the lock\n"; });
    assert_eq!(git_text(&fixture, &repo, &["show", format!("HEAD:{FILE}").as_str()]), "the edit\n", "the commit kept on the side reference is the one that landed");
    drop(service);
    remove_settled(&fixture);
}
