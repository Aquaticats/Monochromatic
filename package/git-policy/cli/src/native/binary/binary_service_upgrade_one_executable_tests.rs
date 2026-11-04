//! What:
//!  The upgrade from the release that shipped the service as a second program (0.5.34) to the release where it is a mode of
//!  the wrapper's own executable, against a stand-in for the user manager that plays the manager's two parts: it owns the
//!  listening socket and passes it to the service it starts (socket activation), and it answers the wrapper's bus calls.
//! Why:
//!  The unit a 0.5.34 wrapper wrote names `<toolchain>/0.5.34/bin/cli-git-service`, and the install of the next release removes
//!  that directory; a start under it ends with `status=203/EXEC` (`doc/troubleshooting/cli-git-service-unit-names-a-removed-
//!  program-status-203-exec.md`). The new wrapper must rewrite the unit, before any activation could use it, to the one that
//!  starts its own executable in service mode, and the old service, still running from the removed file, must retire after
//!  finishing its work (owner: "Commits should never fail." "never discard user data"). The control starts the very
//!  command line the rewritten unit holds, with the socket passed in the way the manager passes it, so the unit text itself
//!  is proved to start a working service.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeUnit(oldText); const old = start(oldService, inheritedSocket); await commit();
//! expect(unit).toContain(wrapperPath + ' cli-git service run'); expect(await exit(old)).toBe(0); expect(await subject()).toBe('subject');
//! ```

use super::commit::wrapped_commit;
use super::service_provision::{PATIENCE, fake_bus_many, finish, finished_batches, prepared};
use super::support::{Fixture, service_command, service_executable, spawn_owned};
use git_policy_cli::service_instance::{instance_of, socket_for};
use git_policy_cli::service_provision::{SERVICE_UNIT_NAME, SOCKET_UNIT, SOCKET_UNIT_NAME};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixListener;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The service unit template of release 0.5.34, as that release wrote it (`git show monochromatic-git-policy-cli-v0.5.34:
///  package/git-policy/cli/src/native/service/service_units/cli-git@.service`), with the program and Git words still to fill in.
const SERVICE_UNIT_0_5_34: &str = "\
# Written by the git wrapper for its commit service (cli-git-unit); the wrapper rewrites this file when it changes.
# One instance per worktree: the instance name is a hash of the worktree's own Git directory (service_instance.rs).
[Unit]
Description=Commit service of one worktree for the git wrapper
Requires=cli-git@%i.socket
After=cli-git@%i.socket

[Service]
# The wrapper replaces the program and Git words below, each written between two at-signs, with absolute paths. The socket
# comes from the socket unit, so the program is never started by the wrapper and is started again by the first
# connection after an exit or a crash. The instance refuses every message about another worktree.
ExecStart=@SERVICE_BINARY@ --journal %S/cli-git/service/%i --git @GIT@ --instance %i
StateDirectory=cli-git/service/%i
Restart=on-failure
";

/// The path 0.5.34 wrote into its unit, in a toolchain directory the next install removed.
const REMOVED_PROGRAM: &str = "/home/a/.local/share/monochromatic-dev/toolchain/cargo/monochromatic-git-policy-cli/0.5.34/bin/cli-git-service";

/// The variable naming the real 0.5.34 `cli-git-service` program, copied out of the toolchain directory before an install
///  removes it; the control with it needs a build whose version differs from 0.5.34, because a service of the same release
///  is not asked to retire.
const OLD_SERVICE_VARIABLE: &str = "CLI_GIT_OLD_SERVICE_0_5_34";

/// The release the stand-in old service announces, which is never this build's.
const STAND_IN_RELEASE: &str = "0.5.34-before-one-executable";

/// The words of an `ExecStart=` line after the equals sign, with the specifiers the control can know filled in, as the manager
///  would fill them: `%S/cli-git/service/%i` is the journal directory, and `%i` the instance.
fn exec_words(unit: &str, journal: &Path, instance: &str) -> Vec<String> {
    let line: &str = unit.lines().find(|line| { return line.starts_with("ExecStart="); }).expect("an ExecStart line");
    return line
        .trim_start_matches("ExecStart=")
        .split_whitespace()
        .map(|word| {
            if word == "%S/cli-git/service/%i" {
                return journal.display().to_string();
            }
            return word.replace("%i", instance);
        })
        .collect();
}

/// Starts `words` as the manager starts a socket-activated service: the listening socket is descriptor 3 and the
///  variables name this process and one descriptor. `sh` only sets the process number in the variable and replaces itself.
fn activate(listener: &UnixListener, words: &[String], root: &Path) -> Child {
    let descriptor: i32 = listener.as_raw_fd();
    let mut command: Command = Command::new("/usr/bin/sh");
    command
        .args(["-c", "LISTEN_PID=$$ LISTEN_FDS=1 exec \"$@\"", "sh"])
        .args(words)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    // SAFETY: between fork and exec only `dup2` and `fcntl`, async-signal-safe calls, run; the descriptor is open in the parent.
    unsafe {
        command.pre_exec(move || {
            // `dup2` onto itself changes nothing and leaves close-on-exec set, so the flag is cleared in either case.
            if libc::dup2(descriptor, 3) < 0 || libc::fcntl(3, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    return spawn_owned(&mut command).expect("the service starts");
}

/// A wrapped scoped commit of `file` with the runtime directory and the bus of the fixture, ready to run.
fn commit_command(fixture: &Fixture, repo: &Path, message: &str, file: &str, runtime: &Path) -> Command {
    let mut command = wrapped_commit(fixture, repo, &["commit", "-m", message, "--", file]);
    command.env("XDG_RUNTIME_DIR", runtime).env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", runtime.join("bus").display()));
    return command;
}

/// Waits for `child` to exit by itself within `limit`.
fn exited_within(child: &mut Child, limit: Duration) -> Option<std::process::ExitStatus> {
    let began: Instant = Instant::now();
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Some(status);
        }
        if began.elapsed() >= limit {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Waits until a connection to `socket` is answered with a greeting, so the service is serving.
fn wait_serving(socket: &Path) {
    use std::io::Read;
    let began: Instant = Instant::now();
    loop {
        if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(socket) {
            let _timeout = stream.set_read_timeout(Some(Duration::from_millis(500)));
            let mut byte: [u8; 1] = [0];
            if stream.read(&mut byte).is_ok_and(|read| { return read > 0; }) {
                return;
            }
        }
        assert!(began.elapsed() < PATIENCE, "the service never greeted");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The upgrade, with the old service started by `old_words`: 0.5.34's unit text names a removed program, an old service of the
///  other release runs on the manager's socket, and two commits arrive together. Both land, the unit was rewritten before the
///  manager was asked to start anything, the old service left by itself with success, and the service that serves them was
///  started from the rewritten unit's own command line.
fn upgrade_with(name: &str, old_words: impl FnOnce(&Fixture, &Path, &str) -> Vec<String>) {
    let (fixture, repo, runtime): (Fixture, PathBuf, PathBuf) = prepared(name);
    std::fs::write(repo.join("b.txt"), "second\n").expect("a second file");
    super::support::git(&fixture, &repo, &["add", "b.txt"]);
    let git_dir: PathBuf = repo.join(".git");
    let instance: String = instance_of(git_dir.as_path());
    let socket: PathBuf = socket_for(runtime.as_path(), git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    let journal: PathBuf = fixture.root.join("journal");
    // The units 0.5.34 wrote: its service template filled with the removed program, and the socket template.
    let units: PathBuf = runtime.join("systemd/user");
    std::fs::create_dir_all(&units).expect("unit directory");
    let old_unit: String = SERVICE_UNIT_0_5_34.replace("@SERVICE_BINARY@", REMOVED_PROGRAM).replace("@GIT@", "/usr/bin/git");
    std::fs::write(units.join(SERVICE_UNIT_NAME), &old_unit).expect("the old service unit");
    std::fs::write(units.join(SOCKET_UNIT_NAME), SOCKET_UNIT).expect("the socket unit");
    // Before the wrapper has run, activation under that unit cannot start anything: the program is gone (203/EXEC).
    let old_command: Vec<String> = exec_words(old_unit.as_str(), journal.as_path(), instance.as_str());
    assert!(Command::new(&old_command[0]).spawn().is_err(), "the removed program must not exist");
    // The manager owns the socket, and the old service serves on it.
    let listener: UnixListener = UnixListener::bind(&socket).expect("the manager's socket");
    let mut old: Child = activate(&listener, old_words(&fixture, journal.as_path(), instance.as_str()).as_slice(), fixture.root.as_path());
    wait_serving(socket.as_path());
    // The bus stand-in checks, when the wrapper asks it to start the socket, that the unit on disk is already the new one.
    let checked: std::sync::Arc<std::sync::Mutex<Option<String>>> = std::sync::Arc::new(std::sync::Mutex::new(None));
    let at_start: Box<dyn FnOnce() + Send> = Box::new({
        let unit_path: PathBuf = units.join(SERVICE_UNIT_NAME);
        let checked = std::sync::Arc::clone(&checked);
        move || { *checked.lock().expect("lock") = std::fs::read_to_string(unit_path).ok(); }
    });
    let bus = fake_bus_many(runtime.as_path(), false, 2, at_start);
    let mut first_command: Command = commit_command(&fixture, &repo, "subject", "a.txt", runtime.as_path());
    let mut second_command: Command = commit_command(&fixture, &repo, "second", "b.txt", runtime.as_path());
    let commits = std::thread::spawn(move || {
        let second = std::thread::spawn(move || { return second_command.output().expect("the second commit runs"); });
        let first: Output = first_command.output().expect("the first commit runs");
        return (first, second.join().expect("second commit"));
    });
    // The part of the manager that starts the next service when the old one has gone: the command line of the rewritten unit,
    //  on the same socket, started from this thread (a helper dies with the thread that started it).
    let status = exited_within(&mut old, PATIENCE).expect("the old service retired when the wrapper asked it to");
    assert!(status.success(), "the old service retired by itself: {status:?}");
    let unit: String = std::fs::read_to_string(units.join(SERVICE_UNIT_NAME)).expect("the rewritten unit");
    let words: Vec<String> = exec_words(unit.as_str(), journal.as_path(), instance.as_str());
    let mut next: Child = activate(&listener, words.as_slice(), fixture.root.as_path());
    let (first, second): (Output, Output) = commits.join().expect("the commits");
    // A connection made and dropped lets the bus stand-in's wait for a second wrapper end when the second found the units in place.
    drop(std::os::unix::net::UnixStream::connect(runtime.join("bus")));
    let calls: Vec<Vec<String>> = bus.join().expect("bus");
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    // The unit was rewritten before the manager was asked to start the socket, and names the wrapper's own executable.
    let at_start_text: String = checked.lock().expect("lock").clone().expect("the socket was started");
    assert!(!at_start_text.contains(REMOVED_PROGRAM) && !at_start_text.contains("cli-git-service "), "{at_start_text}");
    assert!(at_start_text.contains("cli-git service run --journal"), "{at_start_text}");
    assert_eq!(words[0], fixture.root.join("wrapper/cli-git-native").display().to_string(), "the unit starts the wrapper's own executable");
    assert_eq!(&words[1..4], &["cli-git", "service", "run"], "in service mode");
    assert!(calls.iter().flatten().any(|call| { return call == "Reload"; }), "the manager was asked to reload: {calls:?}");
    assert!(calls.iter().flatten().any(|call| { return call == "StartUnit"; }), "{calls:?}");
    // Both commits landed, none was lost, and the service that landed them is the one started from the rewritten unit.
    let log: String = super::commit::git_text(&fixture, &repo, &["log", "--format=%s"]);
    assert!(log.lines().any(|line| { return line == "subject"; }) && log.lines().any(|line| { return line == "second"; }), "{log}");
    assert!(finished_batches(&fixture) >= 1, "the service started from the rewritten unit finished a batch");
    assert!(next.try_wait().expect("state").is_none(), "the successor is still serving");
    let _killed = next.kill();
    let _waited = next.wait();
    drop(listener);
    finish(&fixture, runtime.as_path());
}

/// With this build's own executable standing in for the old service (announcing another release), which is the old service's
///  retirement logic and the unit rewrite proved without needing the old program.
#[test]
fn the_unit_of_the_second_program_is_rewritten_and_the_old_service_retires_without_losing_a_commit() {
    upgrade_with("upgrade-one-executable", |_, journal, instance| {
        let mut words: Vec<String> = vec![service_executable().to_string()];
        words.extend(service_command().get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }));
        words.extend(["--journal", journal.to_str().expect("UTF-8"), "--git", "/usr/bin/git", "--instance", instance, "--release", STAND_IN_RELEASE].map(String::from));
        return words;
    });
}

/// The same with the real 0.5.34 `cli-git-service` program, started with the command line 0.5.34's unit holds.
#[test]
#[ignore = "needs CLI_GIT_OLD_SERVICE_0_5_34, the 0.5.34 cli-git-service program, and a build whose version is not 0.5.34"]
fn the_real_0_5_34_service_retires_when_the_unit_is_rewritten() {
    let program: PathBuf = PathBuf::from(std::env::var_os(OLD_SERVICE_VARIABLE).expect("CLI_GIT_OLD_SERVICE_0_5_34 names the 0.5.34 service program"));
    assert!(program.is_file(), "{} is not a file", program.display());
    assert_ne!(git_policy_cli::service_protocol::RELEASE, "0.5.34", "a service of the same release is not asked to retire; build with another version");
    upgrade_with("upgrade-real-0-5-34", |_, journal, instance| {
        return vec![program.display().to_string(), "--journal".into(), journal.display().to_string(), "--git".into(), "/usr/bin/git".into(), "--instance".into(), instance.into()];
    });
}
