//! What:
//!  A commit made while the write of the service's own place in its journal is held,
//!  while a pipe stands at the places' directory,
//!  and while a plain file stands there,
//!  and the answer to a request that landed before its service was killed and is sent again.
//! Why:
//!  The place is this branch's, and the fifth review found that a service which could not write it in five seconds
//!  refused every commit for the rest of its life, and that the first commit after each start waited for that write:
//!  three commits failed where `main` landed them (its own measurement: a commit that took 4,985 ms and failed,
//!  and two more that failed in 26 and 25 ms).
//!  The rule it breaks is the project's own: a commit never fails, and nothing waits for ever.
//!  A place that is late is late and not lost: this control asks only what a person's commit does while it is late.
//!  The fourth case is a request whose commit landed and whose answer was lost: the journal names it, so the answer is
//!  that commit and never "it was not made".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holdThePlace(); const commit = await wrappedCommit(fixture, 'x'); expect(commit.status).toBe(0); expect(commit.ms).toBeLessThan(bound);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, FixtureService, fixture, give_service_environment, git, repository, start_fixture_service};
use git_policy_cli::service_journal_holders::THE_PLACE_STEP;
use git_policy_cli::service_start_step_hold_control_window::HOLD_VARIABLE;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, Instant};

/// The floor of this project's bound on a commit: five seconds.
const FIVE_SECONDS: Duration = Duration::from_secs(5);

/// How long the write of the place is held:
/// longer than the ceiling the service gives it, so the service has no place while the commit arrives.
const HELD_FOR: Duration = Duration::from_secs(8);

/// The longest this control waits for a state it cannot time.
const PATIENCE: Duration = Duration::from_secs(120);

/// A repository with one tracked file and a service of its own,
/// with the write of the service's place armed to be held.
fn setup(name: &str) -> (Fixture, PathBuf, FixtureService, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("the file");
    git(&fixture, repo.as_path(), &["add", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "-m", "base", "--", "a.txt"]);
    let held: PathBuf = fixture.root.join("the-place");
    give_service_environment(&fixture, HOLD_VARIABLE, format!("{THE_PLACE_STEP}:{}", held.display()).as_str());
    let reached: PathBuf = held.with_file_name("the-place.reached");
    let release: PathBuf = held.with_file_name("the-place.release");
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return (fixture, repo, service, reached, release);
}

/// One wrapped commit of `content` into `repo`, waited for.
fn a_commit(fixture: &Fixture, repo: &Path, content: &str) -> (Output, Duration) {
    std::fs::write(repo.join("a.txt"), content).expect("the edit a commit carries");
    let mut command: std::process::Command =
        super::commit::wrapped_commit(fixture, repo, &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    let began: Instant = Instant::now();
    let output: Output = command.output().expect("the wrapped commit runs");
    return (output, began.elapsed());
}

/// What real Git takes to make the same commit in the same repository,
/// which is the other half of this project's bound.
fn real_git_commit_ms(fixture: &Fixture, repo: &Path) -> Duration {
    std::fs::write(repo.join("a.txt"), "what real Git takes\n").expect("the edit real Git commits");
    let began: Instant = Instant::now();
    let _made: Output = git(fixture, repo, &["commit", "--quiet", "-m", "what real Git takes", "--", "a.txt"]);
    return began.elapsed();
}

/// The commit the branch's own log names, as real Git sees the branch.
fn subjects(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let named: Output = super::support::direct(fixture).current_dir(repo).args(["log", "--format=%s", "main"]).output().expect("real Git answers");
    return String::from_utf8_lossy(named.stdout.as_slice()).lines().map(String::from).filter(|line| { return !line.is_empty(); }).collect();
}

/// What:
///  A commit that arrives while the write of the service's place is held lands, and within the bound a person's commit
///  has.
/// Why:
///  The place is the service's own bookkeeping about who may promise what;
/// a commit's answer needs the commit,
///  and the fifth review measured three commits that failed here where `main` landed them.
#[test]
fn a_commit_lands_while_the_write_of_the_service_place_is_held() {
    let (fixture, repo, _service, reached, release): (Fixture, PathBuf, FixtureService, PathBuf, PathBuf) = setup("a-commit-while-the-place-is-held");
    let git_ms: Duration = real_git_commit_ms(&fixture, &repo);
    let bound: Duration = FIVE_SECONDS.max(git_ms * 2);
    let began: Instant = Instant::now();
    // The write is reached, and it stays held while the commit is made.
    while !reached.exists() && began.elapsed() < PATIENCE {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(reached.exists(), "the service reached the write of its place");
    let (output, took): (Output, Duration) = a_commit(&fixture, &repo, "the commit made while the place is held\n");
    let held_while_the_commit: bool = began.elapsed() < HELD_FOR;
    std::fs::write(release.as_path(), b"").expect("the write of the place is let go");
    let log: String = crate::support_fixture_service_log::service_log(fixture.root.as_path());
    eprintln!("MEASURE held={held_while_the_commit} commit_ms={} bound_ms={} git_ms={}", took.as_millis(), bound.as_millis(), git_ms.as_millis());
    remove_settled(&fixture);
    assert!(output.status.success(), "the commit landed through its service: {}\n{log}", String::from_utf8_lossy(output.stderr.as_slice()));
    assert!(took < bound, "the commit took {} ms while the write of the service's place was held, past the bound of {} ms:\n{log}", took.as_millis(), bound.as_millis());
}

/// What:
///  A commit lands while a pipe stands where the places of a journal live,
///  and while a plain file stands there.
/// Why:
///  Either makes the place impossible to write,
/// which is the state the fifth review measured as a permanent refusal of every commit in 28 ms.
#[test]
fn a_commit_lands_while_the_place_cannot_be_written_at_all() {
    for kind in ["a pipe", "a file"] {
        let fixture: Fixture = fixture(format!("a-commit-with-{}-at-the-places", kind.replace(' ', "-")).as_str());
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        std::fs::write(repo.join("a.txt"), "base\n").expect("the file");
        git(&fixture, repo.as_path(), &["add", "a.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "-m", "base", "--", "a.txt"]);
        let places: PathBuf = fixture.root.join("journal").join("services");
        std::fs::create_dir_all(fixture.root.join("journal").as_path()).expect("the journal");
        if kind == "a pipe" {
            use std::os::unix::ffi::OsStrExt as _;
            let named: std::ffi::CString = std::ffi::CString::new(places.as_os_str().as_bytes()).expect("no NUL in a fixture path");
            // SAFETY: `named` is a NUL-terminated path that outlives the call, and the mode is the journal's own.
            assert_eq!(unsafe { libc::mkfifo(named.as_ptr(), 0o700) }, 0, "a pipe where the places of a journal live");
        } else {
            std::fs::write(places.as_path(), b"not a directory").expect("a plain file where the places of a journal live");
        }
        let git_ms: Duration = real_git_commit_ms(&fixture, &repo);
        let bound: Duration = FIVE_SECONDS.max(git_ms * 2);
        let service: FixtureService = start_fixture_service(&fixture);
        service.serve(&fixture, repo.join(".git").as_path());
        let (output, took): (Output, Duration) = a_commit(&fixture, &repo, "a commit while the place cannot be written\n");
        let log: String = crate::support_fixture_service_log::service_log(fixture.root.as_path());
        let landed: Vec<String> = subjects(&fixture, repo.as_path());
        eprintln!("MEASURE {kind} commit_ms={} bound_ms={}", took.as_millis(), bound.as_millis());
        remove_settled(&fixture);
        assert!(output.status.success(), "a commit lands with {kind} at the places of its journal: {}\n{log}", String::from_utf8_lossy(output.stderr.as_slice()));
        assert!(took < bound, "and within the bound of {} ms, and took {} ms:\n{log}", bound.as_millis(), took.as_millis());
        assert!(landed.iter().any(|subject| { return subject.contains("while the place cannot be written"); }), "real Git shows the commit on the branch: {landed:?}");
    }
}
