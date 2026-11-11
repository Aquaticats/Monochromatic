//! What:
//!  A stop of the commit service while it lands a commit,
//! run as the real service against a real repository with a stand-in Git that stalls the landing at its branch move.
//! Why:
//!  The owner's pick for open choice 12 (2026-11-07T22:19:43-05:00,
//! verbatim "C:
//! shutdown stops its Gits (Recommended)"):
//! the stop handler only records the stop,
//! the service's own shutdown signals the Git processes it is waiting on,
//! every wait after the signal is bounded and then `SIGKILL`,
//! and a commit whose Git is stopped keeps its work on its side ref.
//! The red control the owner asked for is the first one:
//! a stop during a long-running Git of a landing leaves no Git process and no `index.lock` behind within the bound.
//! The owner's later pick (2026-11-08,
//! verbatim "B:
//! let a branch write finish (Recommended)"):
//! a stop signals no Git that is writing the branch,
//! the shutdown waits for that write within its bound,
//! and every other Git is ended as before.
//! The service is started by the control at a socket under the control's own runtime directory,
//! so nothing touches the user's service manager or journal.

use super::client_service::{Held, edited_repository, served, service_with_git};
use super::commit::{git_text, remove_settled};
use super::support::{Fixture, executable, fixture, while_test_lives};
use git_policy_cli::bounded_push::STOP_GRACE;
use git_policy_cli::stop_of_the_service_noticed_by_its_children::{LAST_RESORT, SETTLE_CEILING};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long a control waits for a state it started,
/// far beyond what a healthy run needs.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long a control waits for the end of a service whose shutdown has to use its whole settle bound.
/// Why:
///  A control's own patience is never the product's bound.
/// The service settles a stop within `SETTLE_CEILING`,
/// and a shutdown that cannot run at all is ended by the kernel's last-resort alarm `LAST_RESORT` after the stop,
/// which is the same length,
/// so a control that allowed exactly `SETTLE_CEILING` raced that alarm,
/// and failed a few milliseconds before it under the release gate's load
/// (2026-11-11,
/// `a_stop_during_a_hung_branch_write_ends_the_service_within_the_bound_and_keeps_the_commit_on_its_side_ref`).
/// The grace a kill gets is what the kernel needs to end a process it was just told to end.
const THE_WHOLE_OF_A_STOP: Duration = LAST_RESORT.saturating_add(STOP_GRACE);

/// What the stand-in Git does with the stop once the landing is inside it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Reaction {
    /// It ends at the first `SIGTERM`,
    /// as Git does.
    EndsAtTerm,
    /// It ignores `SIGTERM`,
    /// so only `SIGKILL` ends it.
    IgnoresTerm,
}

/// The step of a landing the stand-in Git stalls in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StallAt {
    /// The import of the commit's objects,
    /// a Git the stop ends,
    /// before the service holds the index.
    TheImport,
    /// The branch move,
    /// the write a stop lets finish,
    /// with the index held by the service.
    TheBranchWrite,
}

impl StallAt {
    /// The Git subcommand the stand-in stalls on.
    fn subcommand(self) -> &'static str {
        return match self {
            StallAt::TheImport => "fast-import",
            StallAt::TheBranchWrite => "update-ref",
        };
    }
}

/// A Git for the service that stalls the landing at one step,
/// until it is ended or released:
/// it writes its own process number to the first returned file when the step starts,
/// and runs the step for real once the second returned file exists.
/// Only the first run of that subcommand stalls,
/// so the step that keeps the commit after a stop runs.
fn stalling_git(fixture: &Fixture, reaction: Reaction, stall_at: StallAt) -> (PathBuf, PathBuf, PathBuf) {
    let entered: PathBuf = fixture.root.join("landing-entered");
    let release: PathBuf = fixture.root.join("landing-release");
    let program: PathBuf = fixture.root.join("stalling-git");
    let trap: &str = match reaction {
        Reaction::EndsAtTerm => "",
        Reaction::IgnoresTerm => "trap '' TERM\n    ",
    };
    executable(
        program.as_path(),
        format!(
            "#!/bin/sh\nfor argument in \"$@\"; do\n  if [ \"$argument\" = {2} ] && [ ! -e '{0}' ]; then\n    {trap}echo $$ > '{0}'\n    while [ ! -e '{3}' ] && {1}; do sleep 0.02; done\n    break\n  fi\ndone\nexec /usr/bin/git \"$@\"\n",
            entered.display(),
            while_test_lives(fixture.root.as_path()),
            stall_at.subcommand(),
            release.display()
        )
        .as_bytes(),
    );
    return (program, entered, release);
}

/// The process number the stand-in Git wrote,
/// once it has.
fn pid_written_to(entered: &Path) -> Option<u32> {
    return std::fs::read_to_string(entered).ok().and_then(parse_number);
}

/// The number a text holds,
/// when it holds one.
fn parse_number(text: String) -> Option<u32> {
    return text.trim().parse::<u32>().ok();
}

/// Whether a process is alive:
/// a zombie that nobody has collected is not.
fn is_alive(pid: u32) -> bool {
    return match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(stat) => !stat.rsplit(')').next().is_some_and(is_zombie_state),
        Err(_gone) => false,
    };
}

/// Whether the text after a process name in `/proc/<pid>/stat` starts with the zombie state.
fn is_zombie_state(after_the_name: &str) -> bool {
    return after_the_name.trim_start().starts_with('Z');
}

/// A service that is landing a commit through the stand-in Git,
/// the client that asked for the commit,
/// and the process number of the stalled Git.
struct Landing {
    /// The service.
    service: Held,
    /// The wrapped commit waiting for its answer.
    client: Child,
    /// The stalled Git.
    git_pid: u32,
    /// The file whose existence lets the stalled Git run its step.
    release: PathBuf,
}

/// Starts the service with the stand-in Git and a commit that stalls inside it.
fn landing_stalled_in_git(fixture: &Fixture, repo: &Path, name: &str, reaction: Reaction, stall_at: StallAt) -> Landing {
    let (program, entered, release): (PathBuf, PathBuf, PathBuf) = stalling_git(fixture, reaction, stall_at);
    let service: Held = service_with_git(fixture, name, repo.join(".git").as_path(), program.as_path());
    let mut commit: Command = served(fixture, repo, name, &["commit", "--quiet", "-m", "stopped commit", "--", "a.txt"]);
    let client: Child = commit.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("the commit starts");
    let began: Instant = Instant::now();
    let git_pid: u32 = loop {
        if let Some(pid) = pid_written_to(entered.as_path()) {
            break pid;
        }
        assert!(began.elapsed() < PATIENCE, "the landing never reached the branch move");
        std::thread::sleep(Duration::from_millis(10));
    };
    if stall_at == StallAt::TheBranchWrite {
        let lock: PathBuf = repo.join(".git/index.lock");
        assert!(std::fs::read_to_string(&lock).is_ok_and(has_the_service_mark), "the service holds the index while it moves the branch");
    }
    return Landing { service, client, git_pid, release };
}

/// Whether a lock file's text is the service's own mark.
fn has_the_service_mark(text: String) -> bool {
    return text.starts_with("cli-git-service");
}

/// Sends `SIGTERM` to the service and waits for its end,
/// within `limit`.
fn stop_and_wait(landing: &mut Landing, limit: Duration) -> (std::process::ExitStatus, Duration) {
    let pid: libc::pid_t = libc::pid_t::try_from(landing.service.child.id()).expect("a process number");
    let sent_at: Instant = Instant::now();
    // SAFETY: `kill` takes a process number and a signal number and touches no memory of this process.
    assert_eq!(unsafe { libc::kill(pid, libc::SIGTERM) }, 0);
    loop {
        if let Ok(Some(status)) = landing.service.child.try_wait() {
            return (status, sent_at.elapsed());
        }
        assert!(sent_at.elapsed() < limit, "the service did not end within {limit:?} of the stop");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Ends a process the control left running,
/// as the manager's final kill would.
fn end_the_process(pid: u32) {
    let number: libc::pid_t = libc::pid_t::try_from(pid).expect("a process number");
    // SAFETY: `kill` takes a process number and a signal number and touches no memory of this process.
    let _sent = unsafe { libc::kill(number, libc::SIGKILL) };
}

/// Ends the client the control started.
fn end_client(client: &mut Child) {
    let _killed = client.kill();
    let _waited = client.wait();
}

/// The side refs that keep a commit the service did not land,
/// as `refname subject` lines.
fn kept_commits(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["for-each-ref", "--format=%(refname) %(subject)", "refs/cli-git/kept"]);
}

/// A stop during a landing whose Git is inside the import of the commit's objects ends that Git,
/// leaves no lock,
/// and ends the service by the signal,
/// all within the bound.
/// This is the owner's red control for choice 12:
/// before the shutdown signalled the Gits it waits on,
/// the service died at once and left the Git running.
#[test]
fn a_stop_during_a_landing_leaves_no_git_and_no_index_lock_within_the_bound() {
    let fixture: Fixture = fixture("stop-ends-its-git");
    let repo: PathBuf = edited_repository(&fixture);
    let name: String = format!("stop-ends-its-git-{}", std::process::id());
    let mut landing: Landing = landing_stalled_in_git(&fixture, &repo, name.as_str(), Reaction::EndsAtTerm, StallAt::TheImport);
    assert!(is_alive(landing.git_pid), "the stand-in Git is running when the stop comes");
    let (status, took): (std::process::ExitStatus, Duration) = stop_and_wait(&mut landing, SETTLE_CEILING);
    assert_eq!(status.signal(), Some(libc::SIGTERM), "the service ends by the signal that stopped it: {status:?}");
    assert!(!is_alive(landing.git_pid), "no Git is left running after {took:?}");
    assert!(!repo.join(".git/index.lock").exists(), "no index lock is left");
    end_client(&mut landing.client);
    remove_settled(&fixture);
}

/// A stop during the branch write does not signal that write:
/// the Git runs on,
/// the service waits for it,
/// and when it finishes the commit lands on the branch,
/// the lock is given up,
/// the client is answered,
/// and the service ends by the signal that stopped it.
/// This is the owner's pick "B:
/// let a branch write finish (Recommended)" (2026-11-08).
#[test]
fn a_stop_during_a_slow_branch_write_lets_it_finish_and_lands_the_commit() {
    let fixture: Fixture = fixture("stop-lets-the-write-finish");
    let repo: PathBuf = edited_repository(&fixture);
    let before: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let name: String = format!("stop-lets-the-write-finish-{}", std::process::id());
    let mut landing: Landing = landing_stalled_in_git(&fixture, &repo, name.as_str(), Reaction::EndsAtTerm, StallAt::TheBranchWrite);
    let pid: libc::pid_t = libc::pid_t::try_from(landing.service.child.id()).expect("a process number");
    // SAFETY: `kill` takes a process number and a signal number and touches no memory of this process.
    assert_eq!(unsafe { libc::kill(pid, libc::SIGTERM) }, 0);
    // Long enough for the shutdown to have told every waiter (the bound of the grace is far above it).
    std::thread::sleep(Duration::from_millis(500));
    assert!(is_alive(landing.git_pid), "the branch write is not signalled by the stop");
    assert!(landing.service.child.try_wait().expect("the wait works").is_none(), "the service waits for the branch write");
    std::fs::write(&landing.release, b"go").expect("release the write");
    let began: Instant = Instant::now();
    let status: std::process::ExitStatus = loop {
        if let Ok(Some(status)) = landing.service.child.try_wait() {
            break status;
        }
        assert!(began.elapsed() < SETTLE_CEILING, "the service did not end within the bound after the write finished");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(status.signal(), Some(libc::SIGTERM), "the service ends by the signal that stopped it: {status:?}");
    let after: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    assert_ne!(after, before, "the commit landed on the branch");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s"]).trim(), "stopped commit");
    assert!(kept_commits(&fixture, &repo).trim().is_empty(), "nothing was kept on a side ref: the commit landed");
    assert!(!repo.join(".git/index.lock").exists(), "no index lock is left");
    end_client(&mut landing.client);
    remove_settled(&fixture);
}

/// A stop during a branch write that never ends ends the service within the bound and not before it,
/// leaves the write to the manager's final kill (nothing here signals it),
/// leaves the branch where it was,
/// and keeps the commit on its side ref (the scratch branch the landing built it on),
/// with the open intent on disk for recovery to read.
#[test]
fn a_stop_during_a_hung_branch_write_ends_the_service_within_the_bound_and_keeps_the_commit_on_its_side_ref() {
    let fixture: Fixture = fixture("stop-during-a-hung-write");
    let repo: PathBuf = edited_repository(&fixture);
    let before: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let name: String = format!("stop-during-a-hung-write-{}", std::process::id());
    let mut landing: Landing = landing_stalled_in_git(&fixture, &repo, name.as_str(), Reaction::EndsAtTerm, StallAt::TheBranchWrite);
    let limit: Duration = THE_WHOLE_OF_A_STOP;
    let (status, took): (std::process::ExitStatus, Duration) = stop_and_wait(&mut landing, limit);
    assert_eq!(status.signal(), Some(libc::SIGTERM), "the service's own shutdown ended it, not the last-resort alarm: {status:?}");
    assert!(took >= SETTLE_CEILING.saturating_sub(Duration::from_secs(1)), "the service waited for the write up to the bound: ended after {took:?}");
    assert!(is_alive(landing.git_pid), "nothing of the service signalled the branch write; the manager's final kill ends it");
    assert_eq!(git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim(), before, "the branch did not move");
    let side: String = git_text(&fixture, &repo, &["for-each-ref", "--format=%(refname) %(subject)", "refs/worktree/cli-git-service"]);
    assert!(side.contains("stopped commit"), "the commit is on its side ref: {side:?}");
    end_the_process(landing.git_pid);
    end_client(&mut landing.client);
    remove_settled(&fixture);
}

/// A Git that ignores `SIGTERM` is ended by `SIGKILL` after the grace,
/// not at once and not never:
/// the service ends after the grace and within the bound,
/// with no Git and no lock left.
#[test]
fn a_git_that_ignores_the_stop_is_killed_after_the_grace_and_within_the_bound() {
    let fixture: Fixture = fixture("stop-kills-after-grace");
    let repo: PathBuf = edited_repository(&fixture);
    let name: String = format!("stop-kills-after-grace-{}", std::process::id());
    let mut landing: Landing = landing_stalled_in_git(&fixture, &repo, name.as_str(), Reaction::IgnoresTerm, StallAt::TheImport);
    let (status, took): (std::process::ExitStatus, Duration) = stop_and_wait(&mut landing, SETTLE_CEILING);
    assert_eq!(status.signal(), Some(libc::SIGTERM), "{status:?}");
    assert!(took >= STOP_GRACE, "the Git was given its grace before the kill: ended after {took:?}");
    assert!(!is_alive(landing.git_pid), "the Git that ignored the stop is gone");
    assert!(!repo.join(".git/index.lock").exists(), "no index lock is left");
    end_client(&mut landing.client);
    remove_settled(&fixture);
}

/// A stop of a service with nothing in flight ends it at once by the signal.
#[test]
fn a_stop_of_an_idle_service_ends_it_by_the_signal_without_waiting() {
    let fixture: Fixture = fixture("stop-idle");
    let repo: PathBuf = edited_repository(&fixture);
    let name: String = format!("stop-idle-{}", std::process::id());
    let service: Held = service_with_git(&fixture, name.as_str(), repo.join(".git").as_path(), Path::new("/usr/bin/git"));
    let mut landing: Landing = Landing { service, client: Command::new("/bin/true").spawn().expect("a stand-in client"), git_pid: 0, release: PathBuf::new() };
    let (status, took): (std::process::ExitStatus, Duration) = stop_and_wait(&mut landing, SETTLE_CEILING);
    assert_eq!(status.signal(), Some(libc::SIGTERM), "{status:?}");
    assert!(took < STOP_GRACE, "an idle service has nothing to wait for: ended after {took:?}");
    end_client(&mut landing.client);
    remove_settled(&fixture);
}

/// The commit of a branch write that outlived the stop bound is kept on `refs/cli-git/kept/<number>` by the next
/// commit of the worktree:
/// the service ended inside the write,
/// the branch never moved,
/// and the batch's intent was still open,
/// so the next landing settles it and keeps the commit where the wrapper says work it did not land is kept.
/// Without that,
/// the commit is only on the landing's scratch branch,
/// which no message and no documented surface name.
#[test]
fn the_commit_of_a_branch_write_that_outlived_the_stop_is_kept_on_a_kept_ref_by_the_next_commit() {
    let fixture: Fixture = fixture("stop-hung-write-kept-ref");
    let repo: PathBuf = edited_repository(&fixture);
    let before: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let name: String = format!("stop-hung-write-kept-ref-{}", std::process::id());
    let mut landing: Landing = landing_stalled_in_git(&fixture, &repo, name.as_str(), Reaction::EndsAtTerm, StallAt::TheBranchWrite);
    let limit: Duration = THE_WHOLE_OF_A_STOP;
    let (status, took): (std::process::ExitStatus, Duration) = stop_and_wait(&mut landing, limit);
    assert_eq!(status.signal(), Some(libc::SIGTERM), "the service's own shutdown ended it, not the last-resort alarm: {status:?}");
    assert!(took >= SETTLE_CEILING.saturating_sub(Duration::from_secs(1)), "the service waited for the write up to the bound: ended after {took:?}");
    end_the_process(landing.git_pid);
    end_client(&mut landing.client);
    assert_eq!(git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim(), before, "the branch did not move");
    assert!(kept_commits(&fixture, &repo).trim().is_empty(), "nothing is on a kept ref yet: the service ended inside the write");
    // The next commit of this worktree settles the batch the stopped service left open, before it lands its own.
    let after_the_stop: String = format!("{name}-after");
    let service: Held = service_with_git(&fixture, after_the_stop.as_str(), repo.join(".git").as_path(), Path::new("/usr/bin/git"));
    let output: std::process::Output = served(&fixture, &repo, after_the_stop.as_str(), &["commit", "--quiet", "-m", "after the stop", "--", "a.txt"])
        .output()
        .expect("the next commit runs");
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    let kept: String = kept_commits(&fixture, &repo);
    assert!(kept.contains("stopped commit"), "the stopped commit is kept on a kept ref: {kept:?}");
    drop(service);
    remove_settled(&fixture);
}
