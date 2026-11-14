//! What:
//!  A push that delivers to an upstream branch named differently from the local branch,
//!  and fails at another address of the same remote,
//!  is not called a failed push.
//! Why:
//!  Git's status line names the reference the push wrote to,
//! which is the branch's upstream and not always the branch's own name:
//! a local `topic` that tracks `origin/main` under `push.default=upstream` is pushed as
//! `<commit>:refs/heads/main`,
//! and Git prints `... <oid> -> main`.
//! A reading that looks for `topic` finds no line there,
//! and the verdict called a delivered push one that reached nowhere,
//! with a final notice that said the commit was saved only in this repository and a plan ended
//! (the fifth review of issue 6293,
//! finding 1).
//! Everything here is real:
//! the remote is a bare repository,
//! the second address is the reserved word Git refuses,
//! the service is a real process and the commit is the built wrapper's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await commitThroughService(); await untilRecorded(); expect(remoteAMain).toBe(commit); expect(stderrOf(status)).not.toContain('saved in this repository');
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, fixture, git, repository, run_wrapped, start_fixture_service, FixtureService};
use git_policy_cli::push_failure_notice::notice_directory;
use git_policy_cli::service_work::work_directory;
use git_policy_cli::work_item::{WorkItem, WorkKind};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// The longest a control waits for the service's push,
/// far beyond what a healthy run needs.
const PATIENCE: Duration = Duration::from_secs(60);

/// The address Git refuses,
/// which no machine state can make a remote.
const DISABLED: &str = "DISABLED";

/// A repository on a local `topic` that tracks `origin/main`,
/// a bare remote that has `main`,
/// a second push address Git refuses,
/// and the service the commit goes through.
struct Setup {
    fixture: Fixture,
    repo: PathBuf,
    remote: PathBuf,
    service: FixtureService,
}

fn setup(name: &str) -> Setup {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    git(&fixture, &repo, &["checkout", "--quiet", "-b", "topic"]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    // The upstream is `main` while the local branch is `topic`, which is the configuration this control is about.
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "topic:main"]);
    // Two push addresses of one remote: the bare repository first, the reserved word second.
    for address in [remote.to_str().expect("UTF-8"), DISABLED] {
        git(&fixture, &repo, &["config", "--add", "remote.origin.pushurl", address]);
    }
    git(&fixture, &repo, &["config", "push.default", "upstream"]);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, remote, service };
}

/// Commits `content` to `a.txt` through the service and returns the new commit's name.
fn commit_through_service(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(output.stderr.as_slice()));
    let named = git(&setup.fixture, setup.repo.as_path(), &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(named.stdout.as_slice()).trim().to_string();
}

/// Waits until the condition holds;
/// the service's push has no signal a control could block on.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Waits until the service settled the push of `commit` on `branch`,
/// in either way it can settle one.
fn wait_until_settled(setup: &Setup, branch: &str, commit: &str) {
    let work: PathBuf = work_directory(setup.repo.join(".git").as_path());
    let record: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: PathBuf::new(),
        git_dir: PathBuf::new(),
        branch_ref: String::from(branch),
        oid: String::from(commit),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    };
    let id: String = record.id();
    wait_until("the service settled the push", &|| {
        return ["done", "failed"].iter().any(|state| { return work.join(state).join(id.as_str()).exists(); });
    });
}

/// The commit a reference of the bare remote names,
/// and nothing when it names none.
fn remote_tip(setup: &Setup, reference: &str) -> Option<String> {
    let named = git(&setup.fixture, setup.remote.as_path(), &["rev-parse", "--verify", "--quiet", reference]);
    let text: String = String::from_utf8_lossy(named.stdout.as_slice()).trim().to_string();
    return match text.is_empty() {
        true => None,
        false => Some(text),
    };
}

/// What:
///  A push that delivered to a differently named upstream is recorded as a delivery and shown as one,
///  and never as a push that was not made.
/// Why:
///  The fifth review of issue 6293 (finding 1) measured the opposite:
/// the reading looked for the local branch's name in a status line that named the upstream's,
/// so a commit the first address held was reported as not pushed,
/// saved only in this repository,
/// and its retry plan was ended.
#[test]
fn a_delivered_push_to_a_differently_named_upstream_is_not_called_failed() {
    let setup: Setup = setup("push-renamed-upstream");
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, "refs/heads/topic", commit.as_str());
    assert_eq!(remote_tip(&setup, "refs/heads/main"), Some(commit.clone()), "the first address received the commit, on the upstream's name");
    let work: PathBuf = work_directory(setup.repo.join(".git").as_path());
    let record: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: PathBuf::new(),
        git_dir: PathBuf::new(),
        branch_ref: String::from("refs/heads/topic"),
        oid: commit.clone(),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    };
    let id: String = record.id();
    assert!(work.join("done").join(id.as_str()).exists(), "the push settled as a delivery");
    assert!(!work.join("failed").join(id.as_str()).exists(), "and never as a failure");
    // The next command shows what the service said about this push: where the commit is, and no failure.
    let shown: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    let text: String = String::from_utf8_lossy(shown.stderr.as_slice()).into_owned();
    assert_eq!(shown.code, Some(0), "{text}");
    assert!(text.contains(setup.remote.to_str().expect("UTF-8")), "the words name the address that holds the commit: {text}");
    assert!(!text.contains("saved in this repository"), "a delivered push is never said to be saved only here: {text}");
    // The delivery's own sentence says which address was not pushed to, which is true; what must never appear is the failure's
    //  claim that the commit went nowhere.
    assert!(!text.contains("was not pushed"), "and never said the commit was not pushed: {text}");
    // The delivery's record is kept for the person rather than the branch's retry plan being ended in failure.
    let notices: usize = std::fs::read_dir(notice_directory(setup.repo.join(".git").as_path())).map_or(0, |listing| { return listing.flatten().count(); });
    assert_eq!(notices, 0, "the record is shown once and is gone from the waiting notices");
    remove_settled(&setup.fixture);
}
