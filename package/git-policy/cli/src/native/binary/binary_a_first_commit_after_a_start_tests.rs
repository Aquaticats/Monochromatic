//! What:
//!  What the first commit after a start of the commit service waits for:
//!  the removal of old finished records, which no landing needs, and a start step that does stand between a commit and
//!  its landing, which must say which step it is.
//! Why:
//!  On 2026-11-15 at 07:58:09 the main checkout's service was replaced by a new release and the first commit after it
//!  took 5,577 ms,
//! where the five installs before it took 1,366 to 2,452 ms in all;
//! the instance's own lines carry `at_ms`,
//! and 3,628 ms passed between the line that warmed the rules and the line that removed ten old finished batches,
//! where the two starts before took 106 ms (47 records) and 40 ms (13 records).
//! The start reads and removes records of its state directory on the home drive before it reads its queue,
//! and which of those steps held that commit is not in the journal:
//! no line said it.
//! This control holds one named start step where a stalled drive would
//! (`service_start_step_hold_control_window.rs`, armed by its own variable only)
//! and measures the first commit against the bound this project puts on a commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! hold('the step', mark); serve(); const first = spawn(wrapped('commit')); await reached(mark); holdFor(bound+1); release(mark); expect(elapsed(first)).toBeLessThan(bound);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, FixtureService, fixture, give_service_environment, git, repository, start_fixture_service};
use git_policy_cli::service_prune_old_records_beside_the_loop::PRUNE_STEP;
use git_policy_cli::service_start_step_hold_control_window::HOLD_VARIABLE;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The floor of this project's bound on a commit: five seconds.
const FIVE_SECONDS: Duration = Duration::from_secs(5);

/// The longest this control waits for a state it cannot time.
const PATIENCE: Duration = Duration::from_secs(120);

/// How long past the bound the control keeps the step held,
/// so the commit it measures has waited longer than the bound allows whatever the clock's resolution.
const HELD_PAST_THE_BOUND: Duration = Duration::from_secs(1);

/// The step the service's start names the reading of the newest finished records by,
/// which does stand between a commit and its landing.
const LANDED_STEP: &str = "it reads the records of the newest finished batches";

/// The place one named start step is held at,
/// and the two files the hold leaves and waits for.
struct Hold {
    /// The value the service reads: the step's name and the place.
    value: String,
    /// Where the service says it has reached the step.
    reached: PathBuf,
    /// What lets the service go on.
    release: PathBuf,
}

impl Hold {
    /// The hold of the start step `step` of this fixture,
    /// beside the fixture,
    /// and nothing releases it until the control says.
    fn beside(fixture: &Fixture, step: &str) -> Hold {
        let at: PathBuf = fixture.root.join("a-start-step");
        return Hold {
            value: format!("{step}:{}", at.display()),
            reached: at.with_file_name("a-start-step.reached"),
            release: at.with_file_name("a-start-step.release"),
        };
    }

    /// Let the step through.
    fn open(&self) {
        std::fs::write(self.release.as_path(), b"").expect("the release of the step");
    }
}

/// A repository with one tracked file,
/// and a service whose start reads it,
/// with the start step of `hold` armed before the service runs.
fn setup(name: &str, step: &str) -> (Fixture, PathBuf, FixtureService, Hold) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("the file");
    git(&fixture, repo.as_path(), &["add", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "-m", "base", "--", "a.txt"]);
    let hold: Hold = Hold::beside(&fixture, step);
    give_service_environment(&fixture, HOLD_VARIABLE, hold.value.as_str());
    let service: FixtureService = start_fixture_service(&fixture);
    // The service listens at once and then takes its start steps, one of which is held: the commit this control makes
    //  arrives while it is inside that step.
    service.serve(&fixture, repo.join(".git").as_path());
    return (fixture, repo, service, hold);
}

/// One wrapped commit of `content` into `repo`, started and not waited for.
fn a_commit(fixture: &Fixture, repo: &Path, content: &str) -> Child {
    std::fs::write(repo.join("a.txt"), content).expect("the edit a commit carries");
    let mut command: Command = super::commit::wrapped_commit(fixture, repo, &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    return command.spawn().expect("the wrapped commit starts");
}

/// What real Git takes to make the same commit in the same repository,
/// which is the other half of this project's bound.
fn real_git_commit_ms(fixture: &Fixture, repo: &Path) -> Duration {
    std::fs::write(repo.join("a.txt"), "what real Git takes\n").expect("the edit real Git commits");
    let began: Instant = Instant::now();
    let _made: Output = git(fixture, repo, &["commit", "--quiet", "-m", "what real Git takes", "--", "a.txt"]);
    return began.elapsed();
}

/// Wait, bounded, until the service says it has reached the step this control holds, or the commit ends.
fn wait_for_the_step(hold: &Hold, commit: &mut Child, began: Instant) -> bool {
    while began.elapsed() < PATIENCE {
        if hold.reached.exists() {
            return true;
        }
        if commit.try_wait().expect("the commit can be asked whether it ended").is_some() {
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    return false;
}

/// What:
///  The first commit after a start is answered inside this project's bound while a start step that no landing needs is
///  held.
/// Why:
///  The removal of old finished records is one of the steps the journal's own lines put between the warm line and the
/// batch of 2026-11-15's first commit after a start,
/// and no landing reads those records.
#[test]
fn a_first_commit_after_a_start_is_answered_while_the_old_records_are_removed() {
    let (fixture, repo, _service, hold): (Fixture, PathBuf, FixtureService, Hold) = setup("a-first-commit-after-a-start", PRUNE_STEP);
    let git_ms: Duration = real_git_commit_ms(&fixture, &repo);
    let bound: Duration = FIVE_SECONDS.max(git_ms * 2);
    let began: Instant = Instant::now();
    let mut commit: Child = a_commit(&fixture, &repo, "the first commit after a start\n");
    let held: bool = wait_for_the_step(&hold, &mut commit, began);
    if held {
        while began.elapsed() < bound + HELD_PAST_THE_BOUND && commit.try_wait().expect("the commit can be asked whether it ended").is_none() {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    hold.open();
    let output: Output = commit.wait_with_output().expect("the commit ends");
    let took: Duration = began.elapsed();
    let log: String = crate::support_fixture_service_log::service_log(fixture.root.as_path());
    eprintln!("MEASURE held={held} first_commit_ms={} bound_ms={} git_ms={}", took.as_millis(), bound.as_millis(), git_ms.as_millis());
    assert!(output.status.success(), "the first commit failed: {}\n{log}", String::from_utf8_lossy(output.stderr.as_slice()));
    remove_settled(&fixture);
    assert!(
        took < bound,
        "the first commit after a start waited {} ms while the removal of old finished records was held, and this project's bound on a commit is {} ms (twice real Git's {} ms, or five seconds). The service's own log:{log}",
        took.as_millis(),
        bound.as_millis(),
        git_ms.as_millis(),
    );
}

/// What:
///  A start step that does stand between a commit and its landing says which step it is,
///  in the service's log and to the client that waits,
///  when it takes longer than a person would expect.
/// Why:
///  The journal of 2026-11-15 held 3,628 ms between two start steps and no line that said which one took it,
/// so the wait could not be attributed to anything.
#[test]
fn a_start_step_a_commit_waits_for_names_itself() {
    let (fixture, repo, _service, hold): (Fixture, PathBuf, FixtureService, Hold) = setup("a-start-step-names-itself", LANDED_STEP);
    let began: Instant = Instant::now();
    let mut commit: Child = a_commit(&fixture, &repo, "the commit a start step holds\n");
    let reached: bool = wait_for_the_step(&hold, &mut commit, began);
    assert!(reached, "the service reached the step this control holds");
    // Longer than the bound a waiting client is told at, so the service's notice and the client's own words both come.
    while began.elapsed() < FIVE_SECONDS + HELD_PAST_THE_BOUND * 2 && commit.try_wait().expect("the commit can be asked whether it ended").is_none() {
        std::thread::sleep(Duration::from_millis(5));
    }
    hold.open();
    let output: Output = commit.wait_with_output().expect("the commit ends");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    // The line is written when the step ends, which is after the release, so the log is read until it carries it.
    let until: Instant = Instant::now() + PATIENCE;
    let mut log: String = crate::support_fixture_service_log::service_log(fixture.root.as_path());
    while !log.contains(LANDED_STEP) && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(20));
        log = crate::support_fixture_service_log::service_log(fixture.root.as_path());
    }
    eprintln!("MEASURE stderr_names_the_step={} log_names_the_step={}", said.contains(LANDED_STEP), log.contains(LANDED_STEP));
    remove_settled(&fixture);
    assert!(log.contains(LANDED_STEP), "the service's log names the step it was inside, and how long: {log}");
    assert!(said.contains(LANDED_STEP), "and the client that waited is told the same: {said}");
}
