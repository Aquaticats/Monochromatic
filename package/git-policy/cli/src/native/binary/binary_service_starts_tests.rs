//! What:
//!  The built wrapper starting its own commit service:
//! the first commit starts it,
//! the next finds it,
//! two starts at once leave one service,
//! a service of another release is replaced,
//! and a start that cannot greet says where its output went.
//! Why:
//!  This is the base the whole design rests on (`doc/planning/cli-git-commit-service-without-a-service-manager.md`):
//! nothing here may need a service manager,
//! a session bus or a unit file,
//! and every control runs with neither.
//! The fixtures are the ones every service control uses,
//! and a service under test is the built executable in its service mode.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::runtime_for;
use super::support::{Fixture, fixture, git, repository, service_executable};
use git_policy_cli::service_instance::{instance_of, socket_for};
use git_policy_cli::service_server_pid_note_of_an_instance::{path as pid_note, read as read_note};
use git_policy_cli::service_server_start_by_the_wrapper::{ServerToStart, StartAttempt, start};
use std::path::{Path, PathBuf};
use std::process::Output;

pub(super) const PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// A repository with one committed file,
/// edited,
/// and a short runtime directory of its own.
pub(super) fn prepared(name: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    let runtime: PathBuf = runtime_for(name);
    std::fs::create_dir_all(&runtime).expect("runtime directory");
    return (fixture, repo, runtime);
}

/// A commit through the wrapper in `repo`,
/// with `XDG_RUNTIME_DIR` set and no session bus,
/// which is the state of a machine that has no service manager for this account.
pub(super) fn commit(fixture: &Fixture, repo: &Path, runtime: &Path) -> Output {
    let mut command = wrapped_commit(fixture, repo, &["commit", "-m", "subject", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime);
    command.env_remove("DBUS_SESSION_BUS_ADDRESS");
    return command.output().expect("the wrapped commit runs");
}

pub(super) fn subject(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["log", "-1", "--format=%s"]).trim_end().to_string();
}

pub(super) fn finish(fixture: &Fixture, runtime: &Path) {
    let _removed = std::fs::remove_dir_all(runtime);
    remove_settled(fixture);
}

/// The folder an instance's files sit in.
fn instance_folder(runtime: &Path) -> PathBuf {
    return runtime.join(git_policy_cli::service_instance::SOCKET_DIRECTORY);
}

/// What:
///  A first commit starts the service itself,
///  and leaves the files the design says it leaves and no unit file at all.
/// Why:
///   This is the whole base in one control:
/// no session bus,
/// no manager,
/// and the commit lands.
#[test]
fn a_first_commit_starts_the_service_itself_and_writes_no_unit_file() {
    let (fixture, repo, runtime) = prepared("starts-first");
    let committed: Output = commit(&fixture, repo.as_path(), runtime.as_path());
    assert!(committed.status.success(), "the commit lands without a manager: {}", String::from_utf8_lossy(committed.stderr.as_slice()));
    assert_eq!(subject(&fixture, repo.as_path()), "subject");
    let socket: PathBuf = socket_for(runtime.as_path(), repo.join(".git").as_path());
    assert!(socket.exists(), "the address the service listened at is there");
    let note: PathBuf = pid_note(instance_folder(runtime.as_path()).as_path(), instance_of(repo.join(".git").as_path()).as_str());
    let server = read_note(note.as_path()).expect("the service wrote the note that names its process");
    assert!(git_policy_cli::service_server_pid_note_of_an_instance::is_still_the_process(&server), "the note names a living process");
    let log: PathBuf = instance_folder(runtime.as_path()).join(format!("{}.log", instance_of(repo.join(".git").as_path())));
    assert!(log.exists(), "the service's output has a home beside its address");
    assert!(!runtime.join("systemd").exists(), "no unit file is written anywhere");
    let record: PathBuf = git_policy_cli::real_git_record::record_file(runtime.as_path());
    assert!(record.exists(), "the real Git the service runs is recorded beside the sockets");
    finish(&fixture, runtime.as_path());
}

/// What:
///  The next command finds the service the first one started,
///  and does not start a second one.
/// Why:
///   One service per worktree is what keeps two commits from landing at once,
/// and a start per command would put two servers on one address.
#[test]
fn the_next_command_finds_the_service_the_first_one_started() {
    let (fixture, repo, runtime) = prepared("starts-finds");
    let first: Output = commit(&fixture, repo.as_path(), runtime.as_path());
    assert!(first.status.success(), "{}", String::from_utf8_lossy(first.stderr.as_slice()));
    let note: PathBuf = pid_note(instance_folder(runtime.as_path()).as_path(), instance_of(repo.join(".git").as_path()).as_str());
    let served = read_note(note.as_path()).expect("the first commit's service is named");
    std::fs::write(repo.join("a.txt"), "base\nmore\nagain\n").expect("second edit");
    let second: Output = commit(&fixture, repo.as_path(), runtime.as_path());
    assert!(second.status.success(), "{}", String::from_utf8_lossy(second.stderr.as_slice()));
    assert_eq!(subject(&fixture, repo.as_path()), "subject");
    let after = read_note(note.as_path()).expect("the same service is still named");
    assert_eq!(after.pid, served.pid, "the second commit reached the service the first one started");
    finish(&fixture, runtime.as_path());
}

/// What:
///  Two commands that both find no service end with exactly one.
/// Why:
///   The start is coordinated by one exclusive file creation,
/// and the address itself is the ownership:
/// a lost race costs a start and never two services.
#[test]
fn two_starts_at_once_leave_one_service() {
    let (fixture, repo, runtime) = prepared("starts-race");
    let instance: String = instance_of(repo.join(".git").as_path());
    let server = ServerToStart {
        socket: socket_for(runtime.as_path(), repo.join(".git").as_path()),
        instance: instance.clone(),
        journal: fixture.root.join("journal"),
        git: PathBuf::from("/usr/bin/git"),
        program: PathBuf::from(service_executable()),
    };
    let one: ServerToStart = server.clone();
    let two: ServerToStart = server.clone();
    let first = std::thread::spawn(move || { return start(&one); });
    let second = std::thread::spawn(move || { return start(&two); });
    let outcomes: Vec<StartAttempt> = vec![first.join().expect("the first start ends"), second.join().expect("the second start ends")];
    assert!(outcomes.iter().any(|outcome| { return matches!(outcome, StartAttempt::Greeted); }), "one of the two starts greeted: {outcomes:?}");
    let note: PathBuf = pid_note(instance_folder(runtime.as_path()).as_path(), instance.as_str());
    let served = read_note(note.as_path()).expect("one service is named");
    assert!(git_policy_cli::service_server_pid_note_of_an_instance::is_still_the_process(&served));
    assert!(!server.starter_marker().exists(), "the start's marker is gone once a start is over");
    let log: PathBuf = instance_folder(runtime.as_path()).join(format!("{instance}.log"));
    let lines: String = std::fs::read_to_string(log).unwrap_or_default();
    assert_eq!(lines.matches("service started").count(), 1, "one server's lines are in the log, not two: {lines}");
    finish(&fixture, runtime.as_path());
}

/// What:
///  A service of another release is asked to retire and the commit lands on this release's service.
/// Why:
///   After an upgrade the address may still belong to the release that was running,
/// and the replacement is the greeting's release plus the retire message,
/// never a call to a manager.
#[test]
fn a_service_of_another_release_is_replaced_by_a_commit() {
    let (fixture, repo, runtime) = prepared("starts-other-release");
    let mut earlier: super::service_commit::Running = super::service_commit::start_in_release(fixture.root.as_path(), runtime.as_path(), Some("0.0.1-before"));
    let committed: Output = commit(&fixture, repo.as_path(), runtime.as_path());
    assert!(committed.status.success(), "a commit through this release's own service: {}", String::from_utf8_lossy(committed.stderr.as_slice()));
    assert_eq!(subject(&fixture, repo.as_path()), "subject");
    let ended = earlier.exited_within(PATIENCE);
    assert!(ended.is_some(), "the earlier release's service was asked to leave and left");
    finish(&fixture, runtime.as_path());
}

/// What:
///  A start that cannot greet fails the commit with a message that names the service's output.
/// Why:
///   The one failure left on the commit path is a service the wrapper started that never came up,
/// and the only record of why is the file beside its address.
#[test]
fn a_start_that_cannot_greet_names_its_log() {
    let (fixture, repo, runtime) = prepared("starts-no-greeting");
    // A folder where the service would bind its address: the bind fails, so nothing can greet.
    let socket: PathBuf = socket_for(runtime.as_path(), repo.join(".git").as_path());
    std::fs::create_dir_all(socket.as_path()).expect("a directory where the address goes");
    let committed: Output = commit(&fixture, repo.as_path(), runtime.as_path());
    assert!(!committed.status.success(), "a service that never comes up does not land a commit");
    let stderr: String = String::from_utf8_lossy(committed.stderr.as_slice()).to_string();
    assert!(stderr.contains(".log"), "the message names the file the service's output went to: {stderr}");
    assert_eq!(subject(&fixture, repo.as_path()), "base", "nothing was committed");
    finish(&fixture, runtime.as_path());
}
