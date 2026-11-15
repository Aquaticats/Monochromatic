//! What:
//!  What one commit through the commit service costs when no drive is slow:
//! its own wall time and the time the service says it waited in the queue,
//! five times over one warm fixture.
//! Why:
//!  The landing board published a decision file for every request before its group began,
//! and a commit's place in the queue waited for that write of the journal's drive
//! (`doc/troubleshooting/git-policy-cli-commit-waits-for-the-services-record-write.md`).
//! The claim is memory now,
//! so the write is gone from the waiting path:
//! this measurement says what that costs a commit when nothing is slow,
//! and it is run against a control arm built from the same source with the write put back,
//! interleaved, as ratios with their ranges.
//! It asserts only that every commit landed and stayed inside this project's bound on a commit.
//! Run it with
//! `mise run //package/git-policy/cli:native:test:binary:host -- --run-ignored only a_commit_costs_no_more --no-capture`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const round of rounds) { const t0 = now(); await commit(`round-${round}`); print(`MEASURE round=${round} commit_ms=${now()-t0}`); }
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, FixtureService, fixture, git, repository, start_fixture_service};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, Instant};

/// How many commits one run makes:
/// five, so a run gives a spread and not a single reading.
const ROUNDS: usize = 5;

/// The floor of this project's bound on a commit: five seconds.
const FIVE_SECONDS: Duration = Duration::from_secs(5);

/// What the measurement is made on.
struct Setup {
    /// The disposable fixture, removed at the end.
    fixture: Fixture,
    /// The working repository.
    repo: PathBuf,
    /// The service, stopped with this value.
    service: FixtureService,
}

/// A repository with one tracked file and a service that serves it.
fn setup(name: &str) -> Setup {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("the file");
    git(&fixture, repo.as_path(), &["add", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "-m", "base", "--", "a.txt"]);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, service };
}

/// Commit `content` through the service and answer how long the command took.
fn commit_and_time(setup: &Setup, content: &str) -> (Output, Duration) {
    std::fs::write(setup.repo.join("a.txt"), content).expect("the edit");
    let mut command: std::process::Command =
        super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let began: Instant = Instant::now();
    let output: Output = command.output().expect("the wrapped commit runs");
    return (output, began.elapsed());
}

/// What real Git takes to make the same commit in the same repository.
fn real_git_commit_ms(fixture: &Fixture, repo: &Path) -> Duration {
    std::fs::write(repo.join("a.txt"), "what real Git takes\n").expect("the edit real Git commits");
    let began: Instant = Instant::now();
    let _made: Output = git(fixture, repo, &["commit", "--quiet", "-m", "what real Git takes", "--", "a.txt"]);
    return began.elapsed();
}

/// The `queued_ms` of every answered landing the service logged, in order.
fn answered_queued(setup: &Setup) -> Vec<String> {
    let log: String = crate::support_fixture_service_log::service_log(setup.fixture.root.as_path());
    return log
        .lines()
        .filter(|line| { return line.contains("answered request="); })
        .map(|line| { return String::from(line.split("queued_ms=").nth(1).unwrap_or("").split_whitespace().next().unwrap_or("")); })
        .collect();
}

/// What:
///  Five commits through one warm service,
///  each with its own wall time and the queue wait the service named for it.
/// Why:
///  A commit's cost when no drive is slow is the only thing the change can have altered,
/// and a run of five gives the spread that says what a difference would have to beat.
#[test]
#[ignore = "a measurement for the record; run it once per arm and compare the two"]
fn a_commit_costs_no_more_than_the_service_record_write_it_no_longer_makes() {
    let setup: Setup = setup("a-commit-costs-no-more-than-the-record-write");
    let git_ms: Duration = real_git_commit_ms(&setup.fixture, setup.repo.as_path());
    let bound: Duration = FIVE_SECONDS.max(git_ms * 2);
    // The commit that wakes the service, so no round pays for a start.
    let (woke, _woke_ms): (Output, Duration) = commit_and_time(&setup, "the commit that wakes it\n");
    assert!(woke.status.success(), "{}", String::from_utf8_lossy(woke.stderr.as_slice()));
    let mut times: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        let content: String = format!("round {round}\n");
        let (output, took): (Output, Duration) = commit_and_time(&setup, content.as_str());
        assert!(output.status.success(), "round {round}: {}", String::from_utf8_lossy(output.stderr.as_slice()));
        times.push(took.as_millis());
        eprintln!("MEASURE round={round} commit_ms={}", took.as_millis());
    }
    let queued: Vec<String> = answered_queued(&setup);
    eprintln!("MEASURE git_ms={} bound_ms={} queued_ms={queued:?}", git_ms.as_millis(), bound.as_millis());
    let longest: u128 = times.iter().copied().max().unwrap_or_default();
    assert!(Duration::from_millis(u64::try_from(longest).unwrap_or(u64::MAX)) < bound, "a commit waited {longest} ms, past this project's bound");
    remove_settled(&setup.fixture);
}
