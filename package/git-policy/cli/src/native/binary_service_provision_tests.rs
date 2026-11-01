//! What:
//!  The built wrapper's choice of route and its own provisioning of socket activation, against a stand-in for the user
//!  session bus that plays the manager's side, and the after-commit push through a running service.
//! Why:
//!  There is no install step and no switch: a socket that exists is used, a session bus and the service program mean the
//!  wrapper sets the service up itself (units in the runtime directory, then a reload and a start through the bus), and
//!  every failure of that is a plain message with the repository unchanged
//!  (`doc/planning/cli-git-resident-service.md`, "Provisioning"). The stand-in starts the real service when it is asked
//!  to start the socket, as the manager's socket activation would.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::{Running, start_in};
use super::support::{Fixture, fixture, git, repository};
use git_policy_cli::service_journal::DONE_EXTENSION;
use git_policy_cli::service_messages::LINGER_COMMAND;
use git_policy_cli::service_provision::{OWN_MARKERS, SERVICE_UNIT_NAME, SOCKET_UNIT_NAME};
use git_policy_cli::service_route::SOCKET_RELATIVE;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SERVICE: &str = env!("CARGO_BIN_EXE_cli-git-service");
const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with one committed file, edited, and a short runtime directory.
fn prepared(name: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    let runtime: PathBuf = super::service_commit::runtime_for(name);
    std::fs::create_dir_all(&runtime).expect("runtime directory");
    // The service program beside the wrapper, as an installed pair sits.
    std::os::unix::fs::symlink(SERVICE, fixture.root.join("wrapper/cli-git-service")).expect("service program link");
    return (fixture, repo, runtime);
}

fn commit(fixture: &Fixture, repo: &Path, runtime: &Path, bus: bool) -> Output {
    let mut command = wrapped_commit(fixture, repo, &["commit", "-m", "subject", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime);
    if bus {
        command.env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", runtime.join("bus").display()));
    }
    return command.output().expect("the wrapped commit runs");
}

fn finished_batches(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
            .count();
    });
}

fn subject(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["log", "-1", "--format=%s"]).trim_end().to_string();
}

fn finish(fixture: &Fixture, runtime: &Path) {
    let _removed = std::fs::remove_dir_all(runtime);
    remove_settled(fixture);
}

/// The reply a bus gives a call: a method return that names the call's number.
fn return_for(serial: u32) -> Vec<u8> {
    let mut message: Vec<u8> = vec![b'l', 2, 0, 1, 0, 0, 0, 0];
    message.extend(serial.to_le_bytes());
    message.extend(8u32.to_le_bytes());
    message.extend([5, 1, b'u', 0]);
    message.extend(serial.to_le_bytes());
    return message;
}

/// A stand-in for the session bus: it authenticates (or refuses), answers every call, and when asked to start a unit it
///  runs `on_start`, as the manager's socket activation would. It returns the names of the methods it was asked for.
fn fake_bus(runtime: &Path, refuse: bool, starter: Box<dyn FnOnce() + Send>) -> std::thread::JoinHandle<Vec<String>> {
    let listener: UnixListener = UnixListener::bind(runtime.join("bus")).expect("the bus socket");
    return std::thread::spawn(move || {
        let mut asked: Vec<String> = Vec::new();
        let Ok((mut stream, _)) = listener.accept() else { return asked };
        let mut one: [u8; 1] = [0];
        let mut line: Vec<u8> = Vec::new();
        while !line.ends_with(b"\r\n") {
            if stream.read_exact(&mut one).is_err() {
                return asked;
            }
            line.push(one[0]);
        }
        if refuse {
            let _sent = stream.write_all(b"REJECTED EXTERNAL\r\n");
            return asked;
        }
        let _sent = stream.write_all(b"OK 00112233445566778899aabbccddeeff\r\n");
        let mut begin: Vec<u8> = Vec::new();
        while !begin.ends_with(b"\r\n") {
            if stream.read_exact(&mut one).is_err() {
                return asked;
            }
            begin.push(one[0]);
        }
        let mut on_start: Option<Box<dyn FnOnce() + Send>> = Some(starter);
        loop {
            let mut header: Vec<u8> = vec![0; 16];
            if stream.read_exact(&mut header).is_err() {
                return asked;
            }
            let body: usize = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
            let fields: usize = u32::from_le_bytes([header[12], header[13], header[14], header[15]]) as usize;
            let serial: u32 = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
            let mut rest: Vec<u8> = vec![0; (16 + fields).div_ceil(8) * 8 + body - 16];
            if stream.read_exact(&mut rest).is_err() {
                return asked;
            }
            header.extend(rest);
            let member: &str = ["Hello", "Reload", "StartUnit"]
                .into_iter()
                .find(|name| { return header.windows(name.len()).any(|window| { return window == name.as_bytes(); }); })
                .unwrap_or("Unknown");
            asked.push(member.to_string());
            if member == "StartUnit"
                && let Some(start) = on_start.take()
            {
                start();
            }
            if stream.write_all(&return_for(serial)).is_err() {
                return asked;
            }
        }
    });
}

/// Starts the real service at the socket the wrapper looks for, the way socket activation would, and keeps it alive.
fn activation(fixture_root: PathBuf, runtime: PathBuf, keep: Arc<Mutex<Vec<Running>>>) -> Box<dyn FnOnce() + Send> {
    return Box::new(move || {
        let service: Running = start_in(fixture_root.as_path(), runtime.as_path());
        keep.lock().expect("lock").push(service);
    });
}

/// With a socket in place and no bus at all, the commit goes through the service and nothing is set up.
#[test]
fn an_existing_socket_is_used_with_no_bus() {
    let (fixture, repo, runtime) = prepared("provision-socket");
    let service: Running = start_in(fixture.root.as_path(), runtime.as_path());
    let output: Output = commit(&fixture, &repo, runtime.as_path(), false);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(finished_batches(&fixture), 1, "the service landed it");
    assert!(!runtime.join("systemd").exists(), "no unit was written");
    drop(service);
    finish(&fixture, runtime.as_path());
}

/// With neither a socket nor a bus, the commit takes the path it took before the service existed.
#[test]
fn no_socket_and_no_bus_keep_the_earlier_path() {
    let (fixture, repo, runtime) = prepared("provision-none");
    let output: Output = commit(&fixture, &repo, runtime.as_path(), false);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(subject(&fixture, &repo), "subject");
    assert_eq!(finished_batches(&fixture), 0);
    finish(&fixture, runtime.as_path());
}

/// With a bus and the service program but no socket, the first commit sets the service up (units in the runtime directory,
///  then Hello, Reload and StartUnit), the stand-in starts the service as the manager would, and the commit lands through
///  it; the second commit finds the socket and asks the bus nothing.
#[test]
fn the_first_commit_provisions_and_the_second_does_not() {
    let (fixture, repo, runtime) = prepared("provision-first");
    let keep: Arc<Mutex<Vec<Running>>> = Arc::new(Mutex::new(Vec::new()));
    let bus = fake_bus(runtime.as_path(), false, activation(fixture.root.clone(), runtime.clone(), Arc::clone(&keep)));
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(finished_batches(&fixture), 1, "the service landed it");
    let units: PathBuf = runtime.join("systemd/user");
    let socket_unit: String = std::fs::read_to_string(units.join(SOCKET_UNIT_NAME)).expect("socket unit");
    let service_unit: String = std::fs::read_to_string(units.join(SERVICE_UNIT_NAME)).expect("service unit");
    for text in [&socket_unit, &service_unit] {
        assert!(OWN_MARKERS.iter().any(|marker| { return text.contains(marker); }), "the marker names the writer");
        assert!(!text.contains('@'), "no placeholder is left");
    }
    assert!(socket_unit.contains(&format!("ListenStream=%t/{SOCKET_RELATIVE}")));
    assert!(service_unit.contains(&fixture.root.join("wrapper/cli-git-service").display().to_string()), "{service_unit}");
    std::fs::write(repo.join("a.txt"), "base\nmore\nand more\n").expect("edit again");
    let second: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(finished_batches(&fixture), 2);
    drop(second);
    // The stand-in's connection ends with the wrapper's; it saw one hello, one reload and one start.
    assert_eq!(bus.join().expect("bus"), vec!["Hello", "Reload", "StartUnit"]);
    keep.lock().expect("lock").clear();
    finish(&fixture, runtime.as_path());
}

/// A bus that refuses the wrapper stops the commit with the plain remedy, quickly, and changes nothing.
#[test]
fn a_refusing_bus_stops_the_commit_plainly() {
    let (fixture, repo, runtime) = prepared("provision-refused");
    let bus = fake_bus(runtime.as_path(), true, Box::new(|| {}));
    let before: String = subject(&fixture, &repo);
    let began: Instant = Instant::now();
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(!output.status.success(), "the commit must not land behind the person's back");
    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(stderr.contains(LINGER_COMMAND), "{stderr}");
    assert!(began.elapsed() < PATIENCE / 2);
    assert_eq!(subject(&fixture, &repo), before, "nothing changed");
    assert!(!runtime.join("systemd").exists(), "no unit was written for a manager that cannot be reached");
    bus.join().expect("bus");
    finish(&fixture, runtime.as_path());
}

/// A unit of the same name that something else wrote is never touched; the commit stops and names the file.
#[test]
fn a_foreign_unit_in_the_way_stops_the_commit_and_is_left_alone() {
    let (fixture, repo, runtime) = prepared("provision-foreign");
    let units: PathBuf = fixture.root.join("home/.config/systemd/user");
    std::fs::create_dir_all(&units).expect("unit directory");
    let foreign: PathBuf = units.join(SOCKET_UNIT_NAME);
    std::fs::write(&foreign, "[Socket]\nListenStream=/somewhere/else\n").expect("foreign unit");
    let bus = fake_bus(runtime.as_path(), false, Box::new(|| {}));
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(foreign.to_string_lossy().as_ref()), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(std::fs::read_to_string(&foreign).expect("still there"), "[Socket]\nListenStream=/somewhere/else\n");
    drop(output);
    bus.join().expect("bus");
    finish(&fixture, runtime.as_path());
}

/// Units an earlier release wrote into the user's own directory would outrank the runtime ones; they are recognized by
///  their content and removed, and the commit then lands.
#[test]
fn units_of_an_earlier_release_are_removed() {
    let (fixture, repo, runtime) = prepared("provision-earlier");
    let units: PathBuf = fixture.root.join("home/.config/systemd/user");
    std::fs::create_dir_all(units.join("sockets.target.wants")).expect("unit directory");
    std::fs::write(units.join(SOCKET_UNIT_NAME), "[Unit]\nDescription=Commit service socket for the git wrapper\n").expect("earlier socket unit");
    std::fs::write(units.join(SERVICE_UNIT_NAME), "[Unit]\nDescription=Commit service for the git wrapper\n").expect("earlier service unit");
    std::os::unix::fs::symlink("../cli-git.socket", units.join("sockets.target.wants").join(SOCKET_UNIT_NAME)).expect("enabling link");
    let keep: Arc<Mutex<Vec<Running>>> = Arc::new(Mutex::new(Vec::new()));
    let bus = fake_bus(runtime.as_path(), false, activation(fixture.root.clone(), runtime.clone(), Arc::clone(&keep)));
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(!units.join(SOCKET_UNIT_NAME).exists() && !units.join(SERVICE_UNIT_NAME).exists());
    assert!(std::fs::symlink_metadata(units.join("sockets.target.wants").join(SOCKET_UNIT_NAME)).is_err(), "the enabling link is gone");
    assert_eq!(finished_batches(&fixture), 1);
    drop(output);
    bus.join().expect("bus");
    keep.lock().expect("lock").clear();
    finish(&fixture, runtime.as_path());
}

/// A bare repository the work repository pushes to, set as the branch's upstream.
fn with_remote(fixture: &Fixture, repo: &Path) -> PathBuf {
    let remote: PathBuf = fixture.root.join("remote.git");
    git(fixture, &fixture.root, &["init", "--quiet", "--bare", remote.to_str().expect("UTF-8")]);
    git(fixture, repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(fixture, repo, &["push", "--quiet", "--set-upstream", "origin", "HEAD"]);
    return remote;
}

/// The commit a branch names in a repository, or empty text when it names none.
fn tip(fixture: &Fixture, directory: &Path, branch: &str) -> String {
    return git_text(fixture, directory, &["rev-parse", "--quiet", "--verify", branch]).trim_end().to_string();
}

fn states(repo: &Path, state: &str) -> usize {
    let directory: PathBuf = repo.join(".git").join(git_policy_cli::work_item::WORK_DIRECTORY).join(state);
    return std::fs::read_dir(directory).map_or(0, Iterator::count);
}

/// Waits for a condition the service brings about after the commit returned, up to the patience of these controls.
fn eventually(what: &str, mut condition: impl FnMut() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "{what} never happened");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A commit through the service returns, and the service then pushes it: the upstream has the commit, both work records
///  are done, and nothing was pushed by the wrapper.
#[test]
fn the_service_pushes_after_the_commit_returned() {
    let (fixture, repo, runtime) = prepared("service-push");
    let remote: PathBuf = with_remote(&fixture, &repo);
    let service: Running = start_in(fixture.root.as_path(), runtime.as_path());
    let output: Output = commit(&fixture, &repo, runtime.as_path(), false);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let landed: String = tip(&fixture, &repo, "HEAD");
    let branch: String = format!("refs/heads/{}", git_text(&fixture, &repo, &["branch", "--show-current"]).trim_end());
    eventually("the push", || { return tip(&fixture, &remote, branch.as_str()) == landed; });
    eventually("the records settling", || { return states(&repo, "done") == git_policy_cli::service_handoff::HANDED_OVER.len(); });
    assert_eq!(states(&repo, "pending") + states(&repo, "claimed") + states(&repo, "failed"), 0);
    drop(service);
    finish(&fixture, runtime.as_path());
}

/// A push that cannot succeed does not fail the commit; the next commit through the service says so once, plainly.
#[test]
fn a_failed_push_is_reported_by_the_next_commit() {
    let (fixture, repo, runtime) = prepared("service-push-fails");
    let remote: PathBuf = with_remote(&fixture, &repo);
    let service: Running = start_in(fixture.root.as_path(), runtime.as_path());
    std::fs::remove_dir_all(&remote).expect("the remote disappears");
    let first: Output = commit(&fixture, &repo, runtime.as_path(), false);
    assert!(first.status.success(), "the commit stands: {}", String::from_utf8_lossy(&first.stderr));
    eventually("the failed record", || { return states(&repo, "failed") > 0; });
    std::fs::write(repo.join("a.txt"), "base\nmore\nand more\n").expect("edit again");
    let second: Output = commit(&fixture, &repo, runtime.as_path(), false);
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    let stderr: String = String::from_utf8_lossy(&second.stderr).to_string();
    assert_eq!(stderr.matches("backup push of your commit").count(), 1, "said once: {stderr}");
    drop(service);
    finish(&fixture, runtime.as_path());
}

/// A service with nothing to do exits on its own after the idle time, with success, so the manager does not restart it;
///  a connection before then keeps it for a further full idle time.
#[test]
fn an_idle_service_exits_with_success() {
    let (fixture, _repo, runtime) = prepared("service-idle");
    let socket: PathBuf = runtime.join(SOCKET_RELATIVE);
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    let idle: Duration = Duration::from_secs(2);
    let mut child = std::process::Command::new(SERVICE)
        .args(["--journal", fixture.root.join("journal").to_str().expect("UTF-8"), "--path", socket.to_str().expect("UTF-8")])
        .args(["--idle-seconds", idle.as_secs().to_string().as_str()])
        .env_clear()
        .spawn()
        .expect("the service starts");
    let began: Instant = Instant::now();
    while std::os::unix::net::UnixStream::connect(&socket).is_err() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    // That connection was the service's last activity; it must still be alive just after it.
    assert!(child.try_wait().expect("status").is_none(), "alive just after a connection");
    let waited: Instant = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("status") {
            break status;
        }
        assert!(waited.elapsed() < PATIENCE, "the idle service never exited");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(status.success(), "{status:?}");
    assert!(waited.elapsed() >= idle / 2, "it did not leave at once");
    finish(&fixture, runtime.as_path());
}
