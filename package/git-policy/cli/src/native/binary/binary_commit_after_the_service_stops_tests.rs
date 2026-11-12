//! What:
//!  A commit whose commit service is stopped in the middle of its exchange:
//!  the command makes the commit itself,
//!  the branch names it,
//!  and the service,
//!  allowed to run again,
//!  makes no second commit of the same request.
//! Why:
//!  A service that stops answering is an ordinary event on this machine,
//!  and native Git, on the same repository with no service at all, makes the commit.
//!  The audit of release 0.6.1-alpha.12 stopped a real service during a commit and got a commit that failed and never
//!  appeared,
//!  so the real program is what this control stops, at a moment this control chooses:
//!  the batch's record in the journal says the service has the request,
//!  and another program's hold on the index says it has not moved the branch.
//!  The commit's identity is a function of the request,
//!  so a service that runs again finds either no client, and makes nothing, or its own commit already on the branch.

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, fixture, git, repository, start_fixture_service, FixtureService};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// How long a wait of this control runs before it says what did not happen.
const WAITING: Duration = Duration::from_secs(240);

/// What:
///  A program of this control that holds the repository's index lock and keeps it.
/// Why:
///  The landing takes that lock before it moves the branch,
///  so a live holder is what keeps a service that already has the request from putting the commit on the branch:
///  the moment this control stops the service is then the one it means to, and not a race with a landing that takes
///  milliseconds.
fn hold_the_index(lock: &Path) -> Child {
    let script: String = format!("exec 9>'{}' ; sleep 600", lock.display());
    let mut holding: Command = Command::new("/bin/sh");
    holding.args(["-c", script.as_str()]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    return holding.spawn().expect("the holder of the index lock starts");
}

/// Wait until the batch's record is in the journal,
/// which is the service's own statement that it has the request and is about to move the branch.
fn wait_for_the_batch_record(journal: &Path) {
    let began: Instant = Instant::now();
    while began.elapsed() < WAITING {
        let recorded: bool = std::fs::read_dir(journal)
            .map(|listing| {
                return listing.flatten().any(|entry| {
                    return entry.file_name().to_string_lossy().ends_with(".intent");
                });
            })
            .unwrap_or(false);
        if recorded {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("the service never recorded the batch of the commit at {}", journal.display());
}

/// What:
///  The processes serving this fixture's journal,
///  found by the journal the service is started with.
/// Why:
///  A control that stops the service must stop the one that serves the request,
///  and what a service of this fixture is is the journal its words name.
fn serving(journal: &Path) -> Vec<u32> {
    let mut found: Vec<u32> = Vec::new();
    let Ok(listing) = std::fs::read_dir("/proc") else { return found };
    for entry in listing.flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|name| { return name.parse::<u32>().ok(); }) else { continue };
        let Ok(line) = std::fs::read(format!("/proc/{pid}/cmdline")) else { continue };
        let words: String = String::from_utf8_lossy(line.as_slice()).replace('\0', " ");
        if words.contains("service") && words.contains(journal.to_str().unwrap_or_default()) {
            found.push(pid);
        }
    }
    return found;
}

/// Real Git's answer, as text.
fn real_git_text(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    let output: Output = git(fixture, repo, arguments);
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
}

/// What:
///  A commit whose service is stopped while the service holds the request is made by the command itself,
///  and the service,
///  allowed to run again,
///  makes no second commit of it.
/// Why:
///  The rule is that a commit never fails where native Git would succeed,
///  and the audit's finding is that this one failed and then nothing appeared.
#[test]
fn a_commit_whose_service_is_stopped_is_made_and_the_woken_service_makes_no_second_one() {
    let fixture: Fixture = fixture("commit-after-a-stopped-service");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "one\n").expect("the first content");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let before: String = real_git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    // The next content is staged, and then another program holds the index, so the service has the request and cannot move
    //  the branch.
    std::fs::write(repo.join("a.txt"), "two\n").expect("the second content");
    let holder: Child = hold_the_index(repo.join(".git").join("index.lock").as_path());
    std::thread::sleep(Duration::from_millis(250));
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=the second content", "--", "a.txt"]);
    let child: Child = command.spawn().expect("the wrapped commit starts");
    wait_for_the_batch_record(fixture.root.join("journal").as_path());
    let stopped: Vec<u32> = serving(fixture.root.join("journal").as_path());
    let mut stops: Vec<i32> = Vec::new();
    for service_pid in &stopped {
        // SAFETY: `kill` takes a process number and a signal and touches no memory; the numbers are the services this fixture
        //  started and reaps.
        stops.push(unsafe { libc::kill(*service_pid as libc::pid_t, libc::SIGSTOP) });
    }
    assert!(!stopped.is_empty(), "the fixture service this control stops is running");
    assert!(stops.iter().all(|outcome| return *outcome == 0), "every stop is taken: {stopped:?} {stops:?}");
    // The command this control started is what it waits for, and the bound of `wrapped_commit` is what ends a command that
    //  never returns.
    let ended: Output = child.wait_with_output().expect("the wrapped commit ends");
    let observed: crate::support_observation::Observed = crate::support_observation::Observed { code: ended.status.code(), stdout: ended.stdout, stderr: ended.stderr };
    // The other program lets the index go, and the service runs again, which is what a stopped process does when whatever held
    //  it lets go.
    let _ended = Command::new("kill").args(["-KILL", holder.id().to_string().as_str()]).status();
    let mut reaped: Child = holder;
    let _waited = reaped.wait();
    for service_pid in &stopped {
        // SAFETY: as above, on the same services.
        unsafe { libc::kill(*service_pid as libc::pid_t, libc::SIGCONT) };
    }
    let after: String = real_git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]);
    let stated: String = real_git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]);
    // The service has time to land anything it still held, and to run the after-commit work the commit left it.
    std::thread::sleep(Duration::from_secs(5));
    let later: String = real_git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]);
    let said: String = String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
    assert_eq!(observed.code, Some(0), "the commit is made, not failed: {said}");
    assert_eq!(after, format!("{}", before.parse::<usize>().expect("a count") + 1), "the branch holds the commit the command made: {said}");
    assert_eq!(stated, "two", "the commit holds the person's bytes: {said}");
    assert_eq!(later, after, "a service that ran again makes no second commit of the same request: {said}");
    service.stop_all();
    remove_settled(&fixture);
}
