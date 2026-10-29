//! A process start that fails at any point of a wrapped commit must not strand the repository.
//! Every control fails one `clone3` of a real commit (processes and threads, the wrapper's and Git's) with `EAGAIN`,
//!  the way a host that ran out of processes does, and requires three things of every outcome:
//!  the command ends within its bound with a message when it fails,
//!  the next ordinary command succeeds,
//!  and the repository is then consistent, either with the commit landed and the index clean
//!  or with the commit not made and the staged file intact, and nothing left behind.
//! Which process the Nth `clone3` hits shifts from run to run because Git's threads interleave,
//!  so no case is pinned to an index; the invariants must hold for whichever one fails.
//! The sweep needs `strace`; a host without it reports that and checks nothing, so the host suite is where it runs.

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, run_commit,
    wrapped_commit};
use super::support::{Fixture, fixture, git, repository};
use std::ffi::OsStr;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many `clone3` calls the sweep fails, one per case; a commit makes fewer than this many in total.
const SWEEP_CASES: usize = 60;

/// How many cases run at once.
const SWEEP_WIDTH: usize = 6;

/// Whether `strace` can be started.
fn strace_available() -> bool {
    return Command::new("strace").arg("--version").output().is_ok();
}

/// A repository with one commit and `a.txt` staged, ready for an explicit-path commit.
fn staged_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// The same command run under `strace`, with the Nth `clone3` of the whole process tree failing with `EAGAIN`.
fn failing_nth_clone3(inner: &Command, nth: usize) -> Command {
    let mut outer: Command = Command::new("strace");
    outer
        .arg("--follow-forks")
        .arg("--quiet")
        .arg("--quiet")
        .arg("--output=/dev/null")
        .arg("--trace=clone3")
        .arg(format!("--inject=clone3:error=EAGAIN:when={nth}"))
        .arg(inner.get_program())
        .args(inner.get_args());
    outer.env_clear();
    for (name, setting) in inner.get_envs() {
        if let Some(text) = setting {
            outer.env(name, text);
        }
    }
    if let Some(directory) = inner.get_current_dir() {
        outer.current_dir(directory);
    }
    return outer;
}

/// Fail the Nth `clone3` of one explicit-path commit and check the three invariants.
///  Returns whether the injected failure made the commit fail, so the sweep can tell that injection took effect.
fn check_case(fixture: &Fixture, nth: usize) -> bool {
    let repo: PathBuf = staged_repository(fixture, &format!("case-{nth}"));
    let inner: Command = wrapped_commit(fixture, repo.as_path(), &["commit", "--quiet", "--message=x", "--", "a.txt"]);
    let failed: Output = failing_nth_clone3(&inner, nth).output().expect("strace starts");
    assert_ne!(failed.status.signal(), Some(9), "case {nth}: the commit hit its time bound");
    let code: i32 = failed.status.code().unwrap_or(-1);
    assert!(matches!(code, 0 | 1 | 2), "case {nth}: unexpected exit {code}");
    if code != 0 {
        let text: String = String::from_utf8_lossy(&[failed.stdout.as_slice(), failed.stderr.as_slice()].concat()).into_owned();
        assert!(!text.trim().is_empty(), "case {nth}: a failed commit must say why");
    }
    let next = run_commit(fixture, repo.as_path(), &["status", "--porcelain"]);
    assert_eq!(next.code, Some(0), "case {nth}: the next command must succeed: {}", String::from_utf8_lossy(&next.stderr));
    let commits: String = git_text(fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]);
    let status: String = String::from_utf8_lossy(next.stdout.as_slice()).into_owned();
    match commits.trim() {
        "1" => assert_eq!(status, "A  a.txt\n", "case {nth}: the commit was not made, so the staged file stays"),
        "2" => assert_eq!(status, "", "case {nth}: the commit landed, so the index must be clean"),
        other => panic!("case {nth}: {other} commits"),
    }
    let outcome: String = format!(
        "case {nth}: injected run exit {code}: {}{}; commits {}; status {status:?}",
        String::from_utf8_lossy(failed.stdout.as_slice()).chars().take(300).collect::<String>(),
        String::from_utf8_lossy(failed.stderr.as_slice()).chars().take(300).collect::<String>(),
        commits.trim(),
    );
    let leftovers = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { assert_nothing_left(fixture, repo.as_path()); }));
    if leftovers.is_err() {
        panic!("{outcome}");
    }
    return code != 0;
}

/// Whichever `clone3` fails, the command ends with a message, the next command recovers, and the repository is consistent.
#[test]
fn a_failed_process_start_never_strands_a_commit() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no process start was failed");
        return;
    }
    let fixture: Fixture = fixture("fork-failures");
    let cases: Vec<usize> = (1..=SWEEP_CASES).collect();
    let failed_commits: AtomicUsize = AtomicUsize::new(0);
    for batch in cases.chunks(SWEEP_WIDTH) {
        std::thread::scope(|scope| {
            for nth in batch {
                let shared_fixture: &Fixture = &fixture;
                let counter: &AtomicUsize = &failed_commits;
                scope.spawn(move || {
                    if check_case(shared_fixture, *nth) {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }
        });
    }
    // Positive control: a sweep whose injection never took effect would pass without having tested anything.
    assert!(failed_commits.load(Ordering::SeqCst) > 0, "no injected process start made a commit fail, so nothing was tested");
    remove_settled(&fixture);
}

/// With no failure injected the same command succeeds, so a passing sweep is not a sweep that never reached the commit.
#[test]
fn the_uninjected_commit_lands_and_leaves_nothing() {
    let fixture: Fixture = fixture("fork-failures-control");
    let repo: PathBuf = staged_repository(&fixture, "control");
    let landed = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=x", "--", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "2");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}
