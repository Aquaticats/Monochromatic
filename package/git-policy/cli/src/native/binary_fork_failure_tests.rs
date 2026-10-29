//! A refused process start, anywhere in a wrapped commit, must neither strand the repository nor escape the wrapper.
//! Every control fails `clone3` calls of a real commit (processes and threads, the wrapper's and Git's) with `EAGAIN`,
//!  the way a host that ran out of processes does, and requires of every outcome:
//!  the command ends within its bound with a message when it fails,
//!  the next ordinary command succeeds,
//!  the repository is then consistent, either with the commit landed and the index clean
//!  or with the commit not made and the staged file intact, and nothing is left behind,
//!  and no start of the wrapper's own is reported as refused, because a short run of refusals is ridden out by the
//!  retry (only Git failing inside itself may still surface).
//! Which process the Nth `clone3` hits shifts from run to run because Git's threads interleave,
//!  so no case is pinned to an index; the invariants must hold for whichever one fails.
//! The sweeps need `strace`; a host without it reports that and checks nothing, so the host suite is where they run.

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, run_commit,
    wrapped_commit};
use super::support::{Fixture, fixture, git, repository};
use std::ffi::OsStr;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many `clone3` calls the sweep fails, one start index per case; a commit makes fewer than this many in total.
const SWEEP_CASES: usize = 60;

/// How many cases run at once.
const SWEEP_WIDTH: usize = 6;

/// How many consecutive `clone3` calls a burst refuses; far inside the retry budget.
const BURST_LENGTH: usize = 3;

/// Whether `strace` can be started.
pub(super) fn strace_available() -> bool {
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

/// The same command run under `strace`, with `length` consecutive `clone3` calls of the whole process tree, starting
///  at the `first`th, failing with `EAGAIN`, and the trace written to `trace`.
fn failing_clone3(inner: &Command, first: usize, length: usize, trace: &std::path::Path) -> Command {
    let when: String = if length == 1 { format!("{first}") } else { format!("{first}..{}", first + length - 1) };
    let mut outer: Command = Command::new("strace");
    outer
        .arg("--follow-forks")
        .arg("--quiet")
        .arg("--quiet")
        .arg(format!("--output={}", trace.display()))
        .arg("--trace=clone3")
        .arg(format!("--inject=clone3:error=EAGAIN:when={when}"))
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

/// Fail `length` consecutive `clone3` calls from the `first`th of one explicit-path commit and check the invariants.
///  Returns whether the trace shows that a call was really refused, so the sweep can tell that injection took effect.
fn check_case(fixture: &Fixture, first: usize, length: usize) -> bool {
    let repo: PathBuf = staged_repository(fixture, &format!("case-{length}-{first}"));
    let trace: PathBuf = fixture.root.join(format!("trace-{length}-{first}.txt"));
    let inner: Command = wrapped_commit(fixture, repo.as_path(), &["commit", "--quiet", "--message=x", "--", "a.txt"]);
    let failed: Output = failing_clone3(&inner, first, length, trace.as_path()).output().expect("strace starts");
    let injected: bool = std::fs::read_to_string(&trace).is_ok_and(|text| { return text.contains("(INJECTED)"); });
    assert_ne!(failed.status.signal(), Some(9), "case {first}: the commit hit its time bound");
    let code: i32 = failed.status.code().unwrap_or(-1);
    assert!(matches!(code, 0 | 1 | 2), "case {first}: unexpected exit {code}");
    let text: String = String::from_utf8_lossy(&[failed.stdout.as_slice(), failed.stderr.as_slice()].concat()).into_owned();
    if code != 0 {
        assert!(!text.trim().is_empty(), "case {first}: a failed commit must say why");
    }
    assert!(
        !text.contains("could not start"),
        "case {first}: a refused start of the wrapper's own escaped the retry: {text}"
    );
    let next = run_commit(fixture, repo.as_path(), &["status", "--porcelain"]);
    assert_eq!(next.code, Some(0), "case {first}: the next command must succeed: {}", String::from_utf8_lossy(&next.stderr));
    let commits: String = git_text(fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]);
    let status: String = String::from_utf8_lossy(next.stdout.as_slice()).into_owned();
    match commits.trim() {
        "1" => assert_eq!(status, "A  a.txt\n", "case {first}: the commit was not made, so the staged file stays"),
        "2" => assert_eq!(status, "", "case {first}: the commit landed, so the index must be clean"),
        other => panic!("case {first}: {other} commits"),
    }
    let outcome: String = format!(
        "case {first}: injected run exit {code}: {}; commits {}; status {status:?}",
        text.chars().take(600).collect::<String>(),
        commits.trim(),
    );
    let leftovers = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { assert_nothing_left(fixture, repo.as_path()); }));
    if leftovers.is_err() {
        panic!("{outcome}");
    }
    return injected;
}

/// Run the whole sweep for one burst length.
fn sweep(label: &str, length: usize) {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no process start was failed");
        return;
    }
    let fixture: Fixture = fixture(label);
    let cases: Vec<usize> = (1..=SWEEP_CASES).collect();
    let injected_cases: AtomicUsize = AtomicUsize::new(0);
    for batch in cases.chunks(SWEEP_WIDTH) {
        std::thread::scope(|scope| {
            for first in batch {
                let shared_fixture: &Fixture = &fixture;
                let counter: &AtomicUsize = &injected_cases;
                scope.spawn(move || {
                    if check_case(shared_fixture, *first, length) {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }
        });
    }
    // Positive control: a sweep whose injection never took effect would pass without having tested anything.
    assert!(injected_cases.load(Ordering::SeqCst) > 0, "no clone3 was refused, so nothing was tested");
    remove_settled(&fixture);
}

/// Whichever single `clone3` fails, the wrapper's own starts are retried, the next command recovers,
///  and the repository is consistent.
#[test]
fn a_failed_process_start_never_strands_a_commit() {
    sweep("fork-failures", 1);
}

/// A short run of consecutive refusals is ridden out the same way.
#[test]
fn a_short_burst_of_refused_starts_is_ridden_out() {
    sweep("fork-failure-bursts", BURST_LENGTH);
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
