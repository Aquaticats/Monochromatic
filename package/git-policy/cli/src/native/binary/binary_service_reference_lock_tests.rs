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

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, FixtureService, fixture, git, repository, start_fixture_service};
use super::support_fixture_service_log::service_log;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

/// The longest a control waits for anything,
/// far beyond a healthy run.
const PATIENCE: Duration = Duration::from_secs(60);

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
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Run a wrapped commit of `FILE` to its end,
/// and return its exit status and what it wrote.
fn commit(fixture: &Fixture, repo: &Path, message: &str) -> (Option<i32>, String) {
    let ended = wrapped_commit(fixture, repo, &["commit", "--quiet", "-m", message, "--", FILE])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .output()
        .expect("the wrapped commit runs");
    return (ended.status.code(), String::from_utf8_lossy(ended.stderr.as_slice()).into_owned());
}

/// What:
///  A lock that nothing proves gone is left where it is and named,
///  with the command that clears it,
///  and the commit does not claim that the branch changed.
/// Why:
///  The lock is what a killed Git leaves,
///  removing it on a guess would take the lock of a Git that is running,
///  and the sentence about a branch that changed is only ever true of a branch that changed.
#[test]
fn a_commit_that_meets_a_reference_lock_names_it_with_the_command_that_clears_it() {
    let (fixture, repo, service) = scene("reference-lock-named");
    let lock: PathBuf = repo.join(".git/refs/heads/main.lock");
    std::fs::write(&lock, "").expect("the lock a killed Git leaves");
    std::fs::write(repo.join(FILE), "the edit\n").expect("the edit");
    let (code, said) = commit(&fixture, repo.as_path(), "meets the lock");
    assert_eq!(code, Some(1), "the commit was not made: {said}");
    assert!(said.contains(lock.display().to_string().as_str()), "the lock is named: {said}");
    assert!(said.contains("rm -- "), "the command that clears it is given: {said}");
    assert!(!said.contains("changed 3 times"), "a lock is not a branch that changed: {said}");
    assert!(said.contains("Nothing in the repository changed"), "the commit changed nothing: {said}");
    assert!(lock.exists(), "a lock nothing proves gone is left where it is: {}", lock.display());
    assert_eq!(git_text(&fixture, &repo, &["log", "--format=%s", "-1"]), "base\n", "nothing landed");
    assert_eq!(git_text(&fixture, &repo, &["status", "--porcelain=v1"]), format!(" M {FILE}\n"), "the edit is still the person's");
    drop(service);
    remove_settled(&fixture);
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
