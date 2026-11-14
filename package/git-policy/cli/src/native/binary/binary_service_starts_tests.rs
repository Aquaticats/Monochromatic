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
        manager: None,
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
    assert_eq!(lines.matches("started release=").count(), 1, "one server's lines are in the log, not two: {lines}");
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

/// What:
///  A stand-in for the account's manager, in the fixture's own `bin`, which records every word it is given and then runs the
///  command it was asked to run.
/// Why:
///  The enhancement's gain is that the server is started as a scope of the manager, and the only honest way to see that
///  without this machine's manager is a program of the run's own that plays it: it records the words, then does what
///  `systemd-run --user --scope` does, which is to run the command as its child in the caller's environment.
fn stand_in_manager(fixture: &Fixture, sleeps_seconds: u32) -> PathBuf {
    let calls: PathBuf = fixture.root.join("manager-calls.txt");
    let programme: PathBuf = fixture.root.join("bin/systemd-run");
    std::fs::write(
        programme.as_path(),
        format!(
            "#!/bin/sh\n# The words this start asked the manager for.\nprintf '%s\\n' \"$*\" >> '{calls}'\n# Past the `--`, the command the manager would run as its scope's own child.\nwhile [ \"$#\" -gt 0 ] && [ \"$1\" != \"--\" ]; do shift; done\n[ \"$#\" -gt 0 ] && shift\nsleep {sleeps_seconds}\nexec \"$@\"\n",
            calls = calls.display()
        ),
    )
    .expect("the stand-in is written");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(programme.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the stand-in runs");
    return calls;
}

/// A commit whose command may ask a manager: the runtime directory is named, and the bus is inside it, which is what makes
/// it this session's own.
fn commit_with_a_manager_of_this_session(fixture: &Fixture, repo: &Path, runtime: &Path) -> Output {
    let mut command = wrapped_commit(fixture, repo, &["commit", "-m", "subject", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime);
    // Nothing listens on this file: the gate compares the bus's directory with the runtime directory, and the stand-in
    //  interpreter never speaks the protocol. No bus of this machine is named or reached.
    command.env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", runtime.join("bus").display()));
    return command.output().expect("the wrapped commit runs");
}

/// What:
///  With a manager of this session there, the server is started as a scope of it, and the commit lands.
/// Why:
///  This is the whole of what the enhancement gives: the service's cgroup is the scope's, so it is not killed with, nor charged
///  to, the command that started it;
///  and the words prove the base's own service words travel inside the manager's.
#[test]
fn with_a_manager_of_this_session_the_service_is_started_as_its_scope() {
    let (fixture, repo, runtime) = prepared("starts-through-the-manager");
    let calls: PathBuf = stand_in_manager(&fixture, 0);
    let met: std::time::Instant = std::time::Instant::now();
    let committed: Output = commit_with_a_manager_of_this_session(&fixture, repo.as_path(), runtime.as_path());
    assert!(committed.status.success(), "the commit lands through a scope: {}", String::from_utf8_lossy(committed.stderr.as_slice()));
    assert_eq!(subject(&fixture, repo.as_path()), "subject");
    let asked: String = std::fs::read_to_string(calls.as_path()).unwrap_or_default();
    assert!(asked.contains("--user --scope"), "the manager's own words are used: {asked}");
    assert!(asked.contains("-- /bin/true") || asked.contains("-- true"), "the probe comes first: {asked}");
    assert!(
        asked.contains("service run --path"),
        "and the start it then makes carries the base's own service words: {asked}"
    );
    // The service that answers is this release's, started with the arguments the design names.
    let socket: PathBuf = socket_for(runtime.as_path(), repo.join(".git").as_path());
    assert!(socket.exists(), "the service listens where the base says");
    assert!(met.elapsed() < PATIENCE, "the whole start is bounded: {:?}", met.elapsed());
    finish(&fixture, runtime.as_path());
}

/// What:
///  With a manager that does not answer in time, the result is exactly the base's: one server, the same files, one commit.
/// Why:
///  An enhancement that can fail a commit is not one;
/// the probe's bound is what makes a busy manager (one re-reading its units answers after the reload, which took 139,770 ms
///  once on this machine) cost a first commit at most `PROBE_WAIT` and nothing at all afterwards.
#[test]
fn with_a_manager_that_does_not_answer_in_time_the_base_starts_the_service() {
    let (fixture, repo, runtime) = prepared("starts-manager-too-slow");
    // The probe outlasts its bound, so the manager is treated as not there and its command is never run.
    let calls: PathBuf = stand_in_manager(&fixture, 30);
    let began: std::time::Instant = std::time::Instant::now();
    let committed: Output = commit_with_a_manager_of_this_session(&fixture, repo.as_path(), runtime.as_path());
    let took: std::time::Duration = began.elapsed();
    assert!(committed.status.success(), "the commit lands with a manager that does not answer: {}", String::from_utf8_lossy(committed.stderr.as_slice()));
    assert_eq!(subject(&fixture, repo.as_path()), "subject");
    assert!(took < PATIENCE, "and within the bound: {took:?}");
    // The manager was asked once and never ran the service: its record holds the probe alone.
    let asked: String = std::fs::read_to_string(calls.as_path()).unwrap_or_default();
    assert!(!asked.contains("service run"), "the manager that did not answer was not asked to start the service: {asked}");
    // Exactly the base's files, and exactly one server.
    let instance: String = instance_of(repo.join(".git").as_path());
    let note: PathBuf = pid_note(runtime.join(git_policy_cli::service_instance::SOCKET_DIRECTORY).as_path(), instance.as_str());
    let served = read_note(note.as_path()).expect("the base started the service, which wrote its note");
    assert!(git_policy_cli::service_server_pid_note_of_an_instance::is_still_the_process(&served), "and it is one living service");
    assert!(socket_for(runtime.as_path(), repo.join(".git").as_path()).exists(), "the address is the base's");
    finish(&fixture, runtime.as_path());
}

/// What:
///  A client keeps a service of a newer release, serves every command through it, and asks it to leave not once.
/// Why:
///  The audit of 2026-08-... measured one client sending `retire` a hundred times in ten seconds to a healthy service of a
/// newer release, and failing after ten seconds with the edit still staged. Each ask made the newer service leave and the
///  account's manager start the newer program again, which is a changeover spending a commit on a program that was there
///  and better. `service_release_order.rs` is the client's half of the fix and the service's one-ask rule the other.
/// The count is the service's own log, which is where an ask that was honoured leaves its line.
#[test]
fn a_client_keeps_a_service_of_a_newer_release_and_asks_it_to_leave_not_once() {
    let (fixture, repo, runtime) = prepared("starts-newer-release-kept");
    let mut newer: super::service_commit::Running = super::service_commit::start_in_release(fixture.root.as_path(), runtime.as_path(), Some("0.6.1-alpha.99"));
    let instance: String = instance_of(repo.join(".git").as_path());
    let note: PathBuf = pid_note(runtime.join(git_policy_cli::service_instance::SOCKET_DIRECTORY).as_path(), instance.as_str());
    let served = read_note(note.as_path()).expect("the newer service wrote its note");
    // Three commands, each of which must be served by the service that was already there.
    for round in 1..=3 {
        std::fs::write(repo.join("a.txt"), format!("base\nmore\n{round}\n")).expect("an edit");
        let committed: Output = commit(&fixture, repo.as_path(), runtime.as_path());
        let said: String = String::from_utf8_lossy(committed.stderr.as_slice()).into_owned();
        assert!(committed.status.success(), "command {round} is served by the newer service: {said}");
        assert!(
            said.contains("served by the commit service of release 0.6.1-alpha.99"),
            "command {round} says which release served it: {said}"
        );
    }
    let after = read_note(note.as_path()).expect("the newer service is still the one serving this worktree");
    assert_eq!(after.pid, served.pid, "not one command replaced the newer service");
    let log: PathBuf = runtime.join(git_policy_cli::service_instance::SOCKET_DIRECTORY).join(format!("{instance}.log"));
    let written: String = std::fs::read_to_string(log.as_path()).unwrap_or_default();
    assert!(!written.contains("retire requested"), "no command asked the newer service to leave: {written}");
    // The service this control started is the control's own to end, and its runtime folder goes with it.
    let _ended = newer.kill();
    drop(newer);
    finish(&fixture, runtime.as_path());
}

/// The most a control waits for the service it started to reach the state it is about,
/// far beyond what the state needs, so only a service that never gets there reaches it.
const SETTLED: std::time::Duration = std::time::Duration::from_secs(60);

/// What:
///  A service of this worktree that is alive and has not greeted yet is waited for by the next command and never started
///  over, and a commit reaches it once its recovery ends.
/// Why:
///  The fourth review of issue 6293 (finding 1):
/// a service in its recovery before its first greeting answered nothing,
/// so the next start took its address away,
/// the worktree had two live services,
/// and the one that ended first deleted the other's address and note.
/// The recovery is held by the service's own hold, which this control releases,
/// so nothing here is a wait made of load.
#[test]
fn a_live_service_that_has_not_greeted_yet_is_waited_for_and_never_started_over() {
    let (fixture, repo, runtime) = prepared("starts-prepares");
    let socket: PathBuf = socket_for(runtime.as_path(), repo.join(".git").as_path());
    let instance: String = instance_of(repo.join(".git").as_path());
    let note: PathBuf = pid_note(instance_folder(runtime.as_path()).as_path(), instance.as_str());
    std::fs::create_dir_all(instance_folder(runtime.as_path()).as_path()).expect("the folder of the address");
    let hold: PathBuf = instance_folder(runtime.as_path()).join("before-serving.pipe");
    let made: std::ffi::CString = std::ffi::CString::new(hold.as_os_str().as_encoded_bytes()).expect("no NUL in the pipe's path");
    // SAFETY: `mkfifo` takes a path this control made and a mode, and touches no memory of this process.
    let created: libc::c_int = unsafe { libc::mkfifo(made.as_ptr(), 0o600) };
    assert_eq!(created, 0, "the control's pipe is made: {}", std::io::Error::last_os_error());
    // A service of this build, started by hand, held in the window between its address being bound and its first greeting.
    let mut service: std::process::Child = {
        let words: Vec<&str> = git_policy_cli::service_main::SERVICE_WORDS.to_vec();
        let mut command = std::process::Command::new(service_executable());
        command
            .args(words)
            .arg("--path")
            .arg(socket.as_path())
            .arg("--journal")
            .arg(fixture.root.join("journal"))
            .arg("--instance")
            .arg(instance.as_str())
            .env(git_policy_cli::service_fault::HOLD_BEFORE_SERVING_VARIABLE, hold.as_os_str())
            .env_remove("XDG_RUNTIME_DIR")
            .env_remove("DBUS_SESSION_BUS_ADDRESS")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        command.spawn().expect("the service starts")
    };
    let service_pid: i32 = i32::try_from(service.id()).expect("a process number fits");
    let settled: std::time::Instant = std::time::Instant::now();
    while !(note.exists() && socket.exists()) && settled.elapsed() < SETTLED {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(read_note(note.as_path()).expect("the service claimed the instance").pid, service_pid, "the instance was claimed before the address was bound");
    assert!(!git_policy_cli::service_server_start_by_the_wrapper::answered(socket.as_path()), "the premise: the service has bound its address and has not greeted yet");
    let address_before: u64 = {
        use std::os::unix::fs::MetadataExt as _;
        std::fs::metadata(socket.as_path()).expect("the address").ino()
    };
    // The next command starts the service, which is what a commit does first.
    let to_start = ServerToStart {
        socket: socket.clone(),
        instance: instance.clone(),
        journal: fixture.root.join("journal"),
        git: PathBuf::from("/usr/bin/git"),
        program: PathBuf::from(service_executable()),
        manager: None,
    };
    let attempt = start(&to_start);
    assert!(
        matches!(attempt, StartAttempt::ServiceIsPreparing { .. }),
        "a live service that has not greeted is waited for, not started over: {attempt:?}"
    );
    let address_after: u64 = {
        use std::os::unix::fs::MetadataExt as _;
        std::fs::metadata(socket.as_path()).expect("the address is still there").ino()
    };
    assert_eq!(address_after, address_before, "the address was not taken over");
    assert_eq!(read_note(note.as_path()).expect("the note still reads").pid, service_pid, "and the service at the address is still the one that was there");
    // The hold is released: the service that was already there greets, so a command that waited is served by it.
    {
        use std::io::Write as _;
        let mut writing: std::fs::File = std::fs::OpenOptions::new().write(true).open(hold.as_path()).expect("the control holds the pipe's other end");
        let _released = writing.write_all(b"go");
    }
    assert!(
        git_policy_cli::service_server_start_by_the_wrapper::waits_for_a_greeting(socket.as_path(), PATIENCE),
        "the service that was preparing answers once it has finished"
    );
    let _killed = service.kill();
    let _reaped = service.wait();
    finish(&fixture, runtime.as_path());
}
