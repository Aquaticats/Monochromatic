//! What:
//!  A commit that arrives while the commit service is held inside the write of a request's decision record,
//!  and how long that commit took.
//! Why:
//!  Two commits of 2026-11-15 waited 120,560 ms and 78,864 ms in the service's queue with nothing ahead of them
//! (`queued_behind=0`),
//!  while the drive that holds the service's own state directory took 109,241 ms for a 4 KiB write-and-sync and the
//!  repository's drive answered in 10 ms.
//! The landing board publishes one such record for every request before its group is landed
//!  (`service_busy.rs`,
//! `begin_group`),
//! so the commit's place in the queue waits for a write of the service's own bookkeeping on a drive the commit does
//!  not otherwise touch.
//! This control holds every reach of that record, the write and the read that replaced it, where the drive would
//! (`service_decision_record_hold_control_window.rs`,
//! armed by its own variable only)
//!  and asserts the one thing this project requires of a commit:
//! it is answered within the greater of five seconds and twice what real Git takes for the same commit,
//!  measured here on the same fixture.
//! A commit answered later than that waited for the service's own record,
//! not for its commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! giveServiceEnvironment(HOLD_VARIABLE, hold); serve(); const commit = spawn(wrapped('commit')); await reached(hold); holdFor(bound); release(hold); expect(elapsed(commit)).toBeLessThan(bound);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, FixtureService, fixture, give_service_environment, git, repository, start_fixture_service};
use git_policy_cli::service_decision_record_hold_control_window::HOLD_VARIABLE;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The floor of this project's bound on a commit:
/// five seconds
/// (`package/git-policy/cli/SPEC.md`).
const FIVE_SECONDS: Duration = Duration::from_secs(5);

/// The longest this control waits for a state it cannot time,
/// far beyond a healthy run and shorter than the command's own bound.
const PATIENCE: Duration = Duration::from_secs(120);

/// How long past the bound the control keeps the record held,
/// so the commit it measures has waited longer than the bound allows whatever the clock's resolution.
const HELD_PAST_THE_BOUND: Duration = Duration::from_secs(1);

/// What the control's directory holds:
/// the place the board is held at,
/// and the two files the hold leaves and waits for.
struct Hold {
    /// The path the variable names.
    at: PathBuf,
    /// Where the board says it has reached the write.
    reached: PathBuf,
    /// What lets the board go on.
    release: PathBuf,
}

impl Hold {
    /// The hold of one fixture's service,
    /// beside the fixture.
    fn beside(fixture: &Fixture) -> Hold {
        let at: PathBuf = fixture.root.join("record-of-a-decision");
        return Hold {
            reached: at.with_file_name("record-of-a-decision.reached"),
            release: at.with_file_name("record-of-a-decision.release"),
            at,
        };
    }

    /// Let every write through,
    /// including the one the first commit of a fixture reaches.
    fn open(&self) {
        std::fs::write(self.release.as_path(), b"").expect("the release of the record's write");
    }

    /// Let nothing through that has not been reached yet.
    fn close(&self) {
        let _removed = std::fs::remove_file(self.release.as_path());
        let _forgotten = std::fs::remove_file(self.reached.as_path());
    }
}

/// One wrapped commit of `content` into `repo`,
/// started and not waited for,
/// so the control can hold the service while it runs.
fn a_commit(fixture: &Fixture, repo: &Path, content: &str) -> Child {
    std::fs::write(repo.join("a.txt"), content).expect("the edit a commit carries");
    let mut command: Command = super::commit::wrapped_commit(fixture, repo, &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    return command.spawn().expect("the wrapped commit starts");
}

/// Commit `content` through the service and wait for it.
fn a_commit_waited_for(fixture: &Fixture, repo: &Path, content: &str) -> Output {
    return a_commit(fixture, repo, content).wait_with_output().expect("the wrapped commit ends");
}

/// What real Git takes to make the same commit in the same repository,
/// which is the other half of this project's bound.
fn real_git_commit_ms(fixture: &Fixture, repo: &Path) -> Duration {
    std::fs::write(repo.join("a.txt"), "what real Git takes\n").expect("the edit real Git commits");
    let began: Instant = Instant::now();
    let _made: Output = git(fixture, repo, &["commit", "--quiet", "-m", "what real Git takes", "--", "a.txt"]);
    return began.elapsed();
}

/// The `queued_ms` of the last request the service answered,
/// from its own log,
/// so a failure names the wait and not only the wall time.
fn last_queued_ms(log: &str) -> String {
    let answered: Option<&str> = log.lines().rev().find(|line| { return line.contains("answered request="); });
    let Some(line) = answered else { return String::from("no answered line") };
    let Some(after) = line.split("queued_ms=").nth(1) else { return String::from("no queued_ms field") };
    return String::from(after.split_whitespace().next().unwrap_or(""));
}

/// What:
///  A commit arrives while the service is held inside the write of a request's decision record,
///  and it is answered within the bound this project puts on a commit.
/// Why:
///  The record is the service's own bookkeeping about a request;
/// a commit's answer needs the commit,
///  and the wait the two commits of 2026-11-15 bore was spent on the write of that record on the drive
///  that stalled,
/// not on anything the commit needed.
#[test]
fn a_commit_is_answered_while_the_record_of_a_decision_is_held() {
    let fixture: Fixture = fixture("a-commit-while-a-decision-record-is-held");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    // The file every commit of this control changes, in the base commit, so a pathspec names a tracked path.
    std::fs::write(repo.join("a.txt"), "base\n").expect("the file");
    git(&fixture, repo.as_path(), &["add", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "-m", "base", "--", "a.txt"]);
    let hold: Hold = Hold::beside(&fixture);
    hold.open();
    // The service reads where the hold is at its start, so the place is given before it runs.
    give_service_environment(&fixture, HOLD_VARIABLE, hold.at.to_str().expect("the hold's place is UTF-8"));
    // Real Git's own commit is measured before the wrapper's service is serving, so nothing of the wrapper is in it.
    let git_ms: Duration = real_git_commit_ms(&fixture, &repo);
    let bound: Duration = FIVE_SECONDS.max(git_ms * 2);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    // The commit that wakes the service: its own record is released before the control holds one.
    let woke: Output = a_commit_waited_for(&fixture, &repo, "the commit that wakes it\n");
    assert!(woke.status.success(), "the waking commit failed: {}\n{}", String::from_utf8_lossy(woke.stderr.as_slice()), crate::support_fixture_service_log::service_log(fixture.root.as_path()));
    hold.close();
    let began: Instant = Instant::now();
    let mut commit: Child = a_commit(&fixture, &repo, "the commit the control measures\n");
    // The board is held once it reaches the write;
    // a service that never reaches it answers the commit instead, which is the state this control's change makes.
    let mut held: bool = false;
    while began.elapsed() < PATIENCE {
        if hold.reached.exists() {
            held = true;
            break;
        }
        if commit.try_wait().expect("the commit can be asked whether it ended").is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if held {
        // A stalled drive holds a write for as long as it likes, and this control holds it past the bound a commit may wait.
        while began.elapsed() < bound + HELD_PAST_THE_BOUND && commit.try_wait().expect("the commit can be asked whether it ended").is_none() {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    hold.open();
    let output: Output = commit.wait_with_output().expect("the measured commit ends");
    let took: Duration = began.elapsed();
    let log: String = crate::support_fixture_service_log::service_log(fixture.root.as_path());
    let queued: String = last_queued_ms(log.as_str());
    eprintln!("MEASURE held={held} commit_ms={} bound_ms={} git_ms={} queued_ms={queued}", took.as_millis(), bound.as_millis(), git_ms.as_millis());
    remove_settled(&fixture);
    assert!(output.status.success(), "the measured commit failed: {}\n{log}", String::from_utf8_lossy(output.stderr.as_slice()));
    assert!(
        took < bound,
        "the commit waited {} ms while the service was held inside the record of a request's decision (a write or a read of it), and this project's bound on a commit is {} ms (twice real Git's {} ms, or five seconds): the service answered it with queued_ms={queued}. Its own log:{log}",
        took.as_millis(),
        bound.as_millis(),
        git_ms.as_millis(),
    );
}
