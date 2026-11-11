//! What:
//!  The built wrapper's choice of route and its own provisioning of socket activation,
//! against a stand-in for the user
//!  session bus that plays the manager's side,
//! and the after-commit push through a running service.
//! Why:
//!  There is no install step and no switch:
//! a socket that exists is used,
//! a session bus and the service program mean the
//!  wrapper sets the service up itself (units in the runtime directory,
//! then a reload and a start through the bus),
//! and
//!  every failure of that is a plain message with the repository unchanged
//!  (`doc/planning/cli-git-resident-service.md`,
//! "Provisioning").
//! The stand-in starts the real service when it is asked
//!  to start the socket,
//! as the manager's socket activation would.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::{Running, start_in, start_in_release};
use super::support::{Fixture, fixture, git, repository};
use git_policy_cli::service_journal::DONE_EXTENSION;
use git_policy_cli::service_messages::{LINGER_COMMAND, UPDATING_NOTICE};
use git_policy_cli::service_provision::{
    LEGACY_SERVICE_UNIT_NAME, LEGACY_SOCKET_UNIT_NAME, OWN_MARKERS, SERVICE_UNIT_NAME, SOCKET_UNIT_NAME,
};
use git_policy_cli::service_instance::{LEGACY_SOCKET_RELATIVE, socket_for};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub(super) const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with one committed file,
/// edited,
/// and a short runtime directory.
pub(super) fn prepared(name: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    let runtime: PathBuf = super::service_commit::runtime_for(name);
    std::fs::create_dir_all(&runtime).expect("runtime directory");
    return (fixture, repo, runtime);
}

pub(super) fn commit(fixture: &Fixture, repo: &Path, runtime: &Path, bus: bool) -> Output {
    let mut command = wrapped_commit(fixture, repo, &["commit", "-m", "subject", "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime);
    if bus {
        command.env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", runtime.join("bus").display()));
    }
    return command.output().expect("the wrapped commit runs");
}

pub(super) fn finished_batches(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
            .count();
    });
}

pub(super) fn subject(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["log", "-1", "--format=%s"]).trim_end().to_string();
}

pub(super) fn finish(fixture: &Fixture, runtime: &Path) {
    let _removed = std::fs::remove_dir_all(runtime);
    remove_settled(fixture);
}

/// The reply a bus gives a call:
/// a method return that names the call's number.
fn return_for(serial: u32) -> Vec<u8> {
    let mut message: Vec<u8> = vec![b'l', 2, 0, 1, 0, 0, 0, 0];
    message.extend(serial.to_le_bytes());
    message.extend(8u32.to_le_bytes());
    message.extend([5, 1, b'u', 0]);
    message.extend(serial.to_le_bytes());
    return message;
}

/// A method return with a signature and a body,
/// naming the call it answers.
fn return_with(serial: u32, signature: &str, body: &[u8]) -> Vec<u8> {
    let mut fields: Vec<u8> = vec![5, 1, b'u', 0];
    fields.extend(serial.to_le_bytes());
    while fields.len() % 8 != 0 {
        fields.push(0);
    }
    fields.extend([8, 1, b'g', 0, u8::try_from(signature.len()).expect("short")]);
    fields.extend(signature.as_bytes());
    fields.push(0);
    let mut message: Vec<u8> = vec![b'l', 2, 0, 1];
    message.extend(u32::try_from(body.len()).expect("short").to_le_bytes());
    message.extend(serial.to_le_bytes());
    message.extend(u32::try_from(fields.len()).expect("short").to_le_bytes());
    message.extend(fields);
    while message.len() % 8 != 0 {
        message.push(0);
    }
    message.extend(body);
    return message;
}

/// The reply to `Hello`:
/// the connection's unique name.
fn unique_name_return(serial: u32) -> Vec<u8> {
    let name: &str = ":1.42";
    let mut body: Vec<u8> = u32::try_from(name.len()).expect("short").to_le_bytes().to_vec();
    body.extend(name.as_bytes());
    body.push(0);
    return return_with(serial, "s", &body);
}

/// The reply to `GetUnit`:
/// the object path of the unit,
/// whichever unit was asked for.
fn unit_path_return(serial: u32) -> Vec<u8> {
    let path: &str = "/org/freedesktop/systemd1/unit/stand_2din";
    let mut body: Vec<u8> = u32::try_from(path.len()).expect("short").to_le_bytes().to_vec();
    body.extend(path.as_bytes());
    body.push(0);
    return return_with(serial, "o", &body);
}

/// The reply to the `ActiveState` read:
/// a variant holding `inactive`,
/// because the stand-in's stop is immediate.
fn inactive_state_return(serial: u32) -> Vec<u8> {
    let state: &str = "inactive";
    let mut body: Vec<u8> = vec![1, b's', 0, 0];
    body.extend(u32::try_from(state.len()).expect("short").to_le_bytes());
    body.extend(state.as_bytes());
    body.push(0);
    return return_with(serial, "v", &body);
}

/// One connection to the stand-in bus:
/// it authenticates (or refuses) and answers every call,
/// and when asked to start a
///  unit it runs the starter if one is left.
/// It returns the names of the methods it was asked for.
/// `listens` is what it answers when the wrapper asks where a unit listens,
/// which a manager that made the unit's address says.
fn serve_connection(mut stream: std::os::unix::net::UnixStream, refuse: bool, on_start: &mut Option<Box<dyn FnOnce() + Send>>, listens: &[String]) -> Vec<String> {
    let mut asked: Vec<String> = Vec::new();
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
    // `zbus` pipelines `NEGOTIATE_UNIX_FD` and `BEGIN` after the answer: the stand-in agrees to the first and stops at the second.
    loop {
        let mut begin: Vec<u8> = Vec::new();
        while !begin.ends_with(b"\r\n") {
            if stream.read_exact(&mut one).is_err() {
                return asked;
            }
            begin.push(one[0]);
        }
        if begin == b"NEGOTIATE_UNIX_FD\r\n" {
            let _agreed = stream.write_all(b"AGREE_UNIX_FD\r\n");
        } else {
            break;
        }
    }
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
        // `GetUnit` comes before `Get`: the first name found is the member, and the longer one contains the shorter.
        let member: &str = ["Hello", "Reload", "ResetFailedUnit", "StartUnit", "StopUnit", "GetUnit", "Get"]
            .into_iter()
            .find(|name| { return header.windows(name.len()).any(|window| { return window == name.as_bytes(); }); })
            .unwrap_or("Unknown");
        asked.push(member.to_string());
        if member == "StartUnit"
            && let Some(start) = on_start.take()
        {
            start();
        }
        let answer: Vec<u8> = match member {
            // `zbus` reads the unique name that `Hello` returns, so the reply to it carries one.
            "Hello" => unique_name_return(serial),
            "GetUnit" => unit_path_return(serial),
            // The property the wrapper reads decides the answer: where the unit listens, or the state it is in.
            "Get" if header.windows(6).any(|window| { return window == b"Listen"; }) => addresses_return(serial, listens),
            "Get" => inactive_state_return(serial),
            _ => return_for(serial),
        };
        if stream.write_all(&answer).is_err() {
            return asked;
        }
    }
}

/// A stand-in for the session bus that serves `connections` wrapper connections one after another,
/// as the manager's bus
///  would serve several commits,
/// and returns the methods each was asked for,
/// in the order the connections came.
/// The starter
///  runs at the first start of a unit,
/// as the manager's socket activation would.
pub(super) fn fake_bus_many(runtime: &Path, refuse: bool, connections: usize, starter: Box<dyn FnOnce() + Send>) -> std::thread::JoinHandle<Vec<Vec<String>>> {
    return fake_bus_many_answering(runtime, refuse, connections, starter, Vec::new());
}

/// `fake_bus_many` whose manager also says where a unit listens,
/// which a manager that made the unit's address answers.
pub(super) fn fake_bus_many_answering(runtime: &Path, refuse: bool, connections: usize, starter: Box<dyn FnOnce() + Send>, listens: Vec<String>) -> std::thread::JoinHandle<Vec<Vec<String>>> {
    let listener: UnixListener = UnixListener::bind(runtime.join("bus")).expect("the bus socket");
    return std::thread::spawn(move || {
        let mut on_start: Option<Box<dyn FnOnce() + Send>> = Some(starter);
        let mut all: Vec<Vec<String>> = Vec::new();
        for _ in 0..connections {
            let Ok((stream, _)) = listener.accept() else { return all };
            all.push(serve_connection(stream, refuse, &mut on_start, listens.as_slice()));
        }
        return all;
    });
}

/// A stand-in for the session bus for one wrapper connection:
/// the methods it was asked for.
fn fake_bus(runtime: &Path, refuse: bool, starter: Box<dyn FnOnce() + Send>) -> std::thread::JoinHandle<Vec<String>> {
    let many = fake_bus_many(runtime, refuse, 1, starter);
    return std::thread::spawn(move || { return many.join().expect("bus").into_iter().flatten().collect(); });
}

/// A method return whose body is a variant holding a list of address pairs (`a(ss)`).
fn addresses_return(serial: u32, addresses: &[String]) -> Vec<u8> {
    // Each pair starts on an eight-byte boundary; each string in it on a four-byte one.
    let mut entries: Vec<u8> = Vec::new();
    for address in addresses {
        while entries.len() % 8 != 0 {
            entries.push(0);
        }
        for text in ["Stream", address.as_str()] {
            while entries.len() % 4 != 0 {
                entries.push(0);
            }
            entries.extend(u32::try_from(text.len()).expect("short").to_le_bytes());
            entries.extend(text.as_bytes());
            entries.push(0);
        }
    }
    let mut body: Vec<u8> = Vec::new();
    body.extend(u32::try_from(entries.len()).expect("short").to_le_bytes());
    while body.len() % 8 != 0 {
        body.push(0);
    }
    body.extend(entries);
    // The variant's signature: its length, `a(ss)`, and the byte that ends it, then padding to eight bytes.
    let mut wrapped: Vec<u8> = vec![5, b'a', b'(', b's', b's', b')', 0, 0];
    wrapped.extend(body);
    return return_with(serial, "v", &wrapped);
}

/// Starts the real service at the socket the wrapper looks for,
/// the way socket activation would,
/// and keeps it alive.
pub(super) fn activation(fixture_root: PathBuf, runtime: PathBuf, keep: Arc<Mutex<Vec<Running>>>) -> Box<dyn FnOnce() + Send> {
    return Box::new(move || {
        let service: Running = start_in(fixture_root.as_path(), runtime.as_path());
        keep.lock().expect("lock").push(service);
    });
}

/// With a socket in place and no bus at all,
/// the commit goes through the service and nothing is set up.
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

/// With a bus and the service program but no socket,
/// the first commit sets the service up (units in the runtime directory,
///  then Hello and StartUnit),
/// the stand-in starts the service as the manager would,
/// and the commit lands through
///  it;
/// the second commit finds the socket and asks the bus nothing.
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
        assert!(!text.contains("@SERVICE_BINARY@") && !text.contains("@GIT@"), "no placeholder is left");
    }
    assert!(socket_unit.contains("ListenStream=%t/monochromatic-git-policy-cli/%i.sock"), "one socket per worktree instance");
    assert!(service_unit.contains(&fixture.root.join("wrapper/monochromatic-git-policy-cli").display().to_string()), "{service_unit}");
    std::fs::write(repo.join("a.txt"), "base\nmore\nand more\n").expect("edit again");
    let second: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(finished_batches(&fixture), 2);
    drop(second);
    // The stand-in's connection ends with the wrapper's; it saw one hello, one start (a new unit file needs no reload), the question about where the service listens, and the two questions about the earlier release's units of the worktree (its service and its socket).
    assert_eq!(bus.join().expect("bus"), vec!["Hello", "StartUnit", "GetUnit", "Get", "GetUnit", "Get", "GetUnit", "Get"]);
    keep.lock().expect("lock").clear();
    finish(&fixture, runtime.as_path());
}

/// What an upgrade leaves behind,
/// rebuilt:
/// the units a release installed,
/// naming a program that was removed when the next
///  release was installed.
/// The text is the installed service unit with its program path replaced.
const REMOVED_PROGRAM: &str = "/home/a/.local/share/toolchain/cargo/cli/0.5.2/bin/cli-git-service";

fn point_unit_at_removed_program(runtime: &Path, fixture: &Fixture) {
    let unit: PathBuf = runtime.join("systemd/user").join(SERVICE_UNIT_NAME);
    let text: String = std::fs::read_to_string(&unit).expect("service unit");
    let program: String = fixture.root.join("wrapper/monochromatic-git-policy-cli").display().to_string();
    assert!(text.contains(program.as_str()), "{text}");
    std::fs::write(&unit, text.replace(program.as_str(), REMOVED_PROGRAM)).expect("older unit");
}

/// An upgrade removes the program the installed service unit names,
/// while the manager's socket stays.
/// The next commit
///  finds the unit differs from what the wrapper would write,
/// rewrites it,
/// has the manager reload,
/// clears failed state,
/// and
///  starts the socket,
/// and the commit lands without the person doing anything.
/// The bus is asked for those and nothing else.
#[test]
fn a_unit_naming_a_removed_program_heals_on_the_next_commit() {
    let (fixture, repo, runtime) = prepared("provision-heal");
    let keep: Arc<Mutex<Vec<Running>>> = Arc::new(Mutex::new(Vec::new()));
    let bus = fake_bus_many(runtime.as_path(), false, 2, activation(fixture.root.clone(), runtime.clone(), Arc::clone(&keep)));
    let first: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    point_unit_at_removed_program(runtime.as_path(), &fixture);
    std::fs::write(repo.join("a.txt"), "base\nmore\nand more\n").expect("edit again");
    let second: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(finished_batches(&fixture), 2, "both commits landed through the service");
    let unit: String = std::fs::read_to_string(runtime.join("systemd/user").join(SERVICE_UNIT_NAME)).expect("service unit");
    assert!(!unit.contains(REMOVED_PROGRAM), "the unit names the program that exists: {unit}");
    assert!(unit.contains(&fixture.root.join("wrapper/monochromatic-git-policy-cli").display().to_string()), "{unit}");
    assert_eq!(
        bus.join().expect("bus"),
        vec![
            vec!["Hello", "StartUnit", "GetUnit", "Get", "GetUnit", "Get", "GetUnit", "Get"],
            vec!["Hello", "Reload", "ResetFailedUnit", "ResetFailedUnit", "StartUnit", "GetUnit", "Get", "GetUnit", "Get", "GetUnit", "Get"],
        ]
    );
    keep.lock().expect("lock").clear();
    finish(&fixture, runtime.as_path());
}

/// When the installed units are what the wrapper would write and the socket is there,
/// a commit rewrites nothing,
/// reloads
///  nothing and does not connect to the bus at all:
/// the stand-in saw one connection,
/// the first commit's.
#[test]
fn matching_units_cost_no_rewrite_no_reload_and_no_bus_call() {
    let (fixture, repo, runtime) = prepared("provision-fast");
    let keep: Arc<Mutex<Vec<Running>>> = Arc::new(Mutex::new(Vec::new()));
    let bus = fake_bus_many(runtime.as_path(), false, 1, activation(fixture.root.clone(), runtime.clone(), Arc::clone(&keep)));
    let first: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let units: PathBuf = runtime.join("systemd/user");
    let stamps = |names: [&str; 2]| -> Vec<std::time::SystemTime> {
        return names.map(|name| { return std::fs::metadata(units.join(name)).expect("unit").modified().expect("time"); }).to_vec();
    };
    let before = stamps([SOCKET_UNIT_NAME, SERVICE_UNIT_NAME]);
    for round in 0..3 {
        std::fs::write(repo.join("a.txt"), format!("base\nmore\nround {round}\n")).expect("edit again");
        let next: Output = commit(&fixture, &repo, runtime.as_path(), true);
        assert!(next.status.success(), "{}", String::from_utf8_lossy(&next.stderr));
    }
    assert_eq!(finished_batches(&fixture), 4);
    assert_eq!(stamps([SOCKET_UNIT_NAME, SERVICE_UNIT_NAME]), before, "no unit was rewritten");
    // Had a later commit connected, the stand-in would have been asked again, or the connection refused.
    assert_eq!(bus.join().expect("bus"), vec![vec!["Hello", "StartUnit", "GetUnit", "Get", "GetUnit", "Get", "GetUnit", "Get"]]);
    keep.lock().expect("lock").clear();
    finish(&fixture, runtime.as_path());
}

/// A service that is running from another release (here one left from before an upgrade,
/// with its socket in place) is never
///  the one that serves this wrapper's commit:
/// the wrapper asks it to retire,
/// the next connection reaches the current
///  release,
/// and the commit lands there,
/// with the one notice and nothing else for the person to do.
#[test]
fn a_running_service_of_another_release_is_retired_and_the_commit_lands() {
    let (fixture, repo, runtime) = prepared("provision-retire");
    let earlier: Running = start_in_release(fixture.root.as_path(), runtime.as_path(), Some("0.0.1-earlier"));
    let old_socket: PathBuf = socket_for(runtime.as_path(), repo.join(".git").as_path());
    // The manager starts the next service on the socket when the old one has gone; the test plays that part.
    let replacement = std::thread::spawn({
        let root: PathBuf = fixture.root.clone();
        let socket_directory: PathBuf = runtime.clone();
        move || {
            let mut old: Running = earlier;
            let Some(status) = old.exited_within(PATIENCE / 6) else { return (None, None) };
            std::fs::remove_file(old_socket).expect("the old socket goes");
            // Dropping the old handle would remove the runtime directory the next service listens in. The process has
            //  exited and been waited for, so there is nothing left for the handle to release.
            std::mem::forget(old);
            return (Some(status), Some(start_in(root.as_path(), socket_directory.as_path())));
        }
    });
    let output: Output = commit(&fixture, &repo, runtime.as_path(), false);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let (exit, replaced) = replacement.join().expect("replacement");
    let status = exit.expect("the old service exited when it was asked to retire");
    assert!(status.success(), "the old service retired by itself: {status:?}");
    let current: Running = replaced.expect("the replacement started");
    assert!(String::from_utf8_lossy(&output.stderr).contains(UPDATING_NOTICE), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(subject(&fixture, &repo), "subject");
    assert_eq!(finished_batches(&fixture), 1, "the current service landed it");
    drop(current);
    finish(&fixture, runtime.as_path());
}

/// A commit the service does not take (here one with nothing to commit,
/// which the transaction answers) sets nothing up:
///  no unit is written and the bus is never asked.
#[test]
fn a_commit_the_service_does_not_take_sets_nothing_up() {
    let (fixture, repo, runtime) = prepared("provision-unused");
    std::fs::write(repo.join("a.txt"), "base\n").expect("undo the edit");
    let bus = fake_bus(runtime.as_path(), false, Box::new(|| {}));
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(!output.status.success(), "there was nothing to commit");
    assert!(!runtime.join("systemd").exists(), "no unit was written for a commit the service never saw");
    // A connection made and dropped here lets the stand-in's wait for the wrapper end.
    drop(std::os::unix::net::UnixStream::connect(runtime.join("bus")));
    assert_eq!(bus.join().expect("bus"), Vec::<String>::new(), "the bus was never asked");
    finish(&fixture, runtime.as_path());
}

/// A bus that refuses the wrapper stops the commit with the plain remedy,
/// quickly,
/// and changes nothing.
#[test]
fn a_refusing_bus_stops_the_commit_plainly() {
    let (fixture, repo, runtime) = prepared("provision-refused");
    let bus = fake_bus(runtime.as_path(), true, Box::new(|| {}));
    let before: String = subject(&fixture, &repo);
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(!output.status.success(), "the commit must not land behind the person's back");
    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(stderr.contains(LINGER_COMMAND), "{stderr}");
    assert_eq!(subject(&fixture, &repo), before, "nothing changed");
    assert!(!runtime.join("systemd").exists(), "no unit was written for a manager that cannot be reached");
    bus.join().expect("bus");
    finish(&fixture, runtime.as_path());
}

/// A unit of the same name that something else wrote is never touched;
/// the commit stops and names the file.
#[test]
fn a_foreign_unit_in_the_way_stops_the_commit_and_is_left_alone() {
    let (fixture, repo, runtime) = prepared("provision-foreign");
    let units: PathBuf = fixture.root.join("home/.config/systemd/user");
    std::fs::create_dir_all(&units).expect("unit directory");
    let foreign: PathBuf = units.join(LEGACY_SOCKET_UNIT_NAME);
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

/// Units an earlier release wrote into the user's own directory would outrank the runtime ones;
/// they are recognized by
///  their content and removed,
/// and the commit then lands.
#[test]
fn units_of_an_earlier_release_are_removed() {
    let (fixture, repo, runtime) = prepared("provision-earlier");
    let units: PathBuf = fixture.root.join("home/.config/systemd/user");
    std::fs::create_dir_all(units.join("sockets.target.wants")).expect("unit directory");
    std::fs::write(units.join(LEGACY_SOCKET_UNIT_NAME), "[Unit]\nDescription=Commit service socket for the git wrapper\n").expect("earlier socket unit");
    std::fs::write(units.join(LEGACY_SERVICE_UNIT_NAME), "[Unit]\nDescription=Commit service for the git wrapper\n").expect("earlier service unit");
    std::os::unix::fs::symlink("../cli-git.socket", units.join("sockets.target.wants").join(LEGACY_SOCKET_UNIT_NAME)).expect("enabling link");
    let keep: Arc<Mutex<Vec<Running>>> = Arc::new(Mutex::new(Vec::new()));
    let bus = fake_bus(runtime.as_path(), false, activation(fixture.root.clone(), runtime.clone(), Arc::clone(&keep)));
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(!units.join(LEGACY_SOCKET_UNIT_NAME).exists() && !units.join(LEGACY_SERVICE_UNIT_NAME).exists());
    assert!(std::fs::symlink_metadata(units.join("sockets.target.wants").join(LEGACY_SOCKET_UNIT_NAME)).is_err(), "the enabling link is gone");
    assert_eq!(finished_batches(&fixture), 1);
    drop(output);
    bus.join().expect("bus");
    keep.lock().expect("lock").clear();
    finish(&fixture, runtime.as_path());
}

/// A bare repository the work repository pushes to,
/// set as the branch's upstream.
fn with_remote(fixture: &Fixture, repo: &Path) -> PathBuf {
    let remote: PathBuf = fixture.root.join("remote.git");
    git(fixture, &fixture.root, &["init", "--quiet", "--bare", remote.to_str().expect("UTF-8")]);
    git(fixture, repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(fixture, repo, &["push", "--quiet", "--set-upstream", "origin", "HEAD"]);
    return remote;
}

/// The commit a branch names in a repository,
/// or empty text when it names none.
fn tip(fixture: &Fixture, directory: &Path, branch: &str) -> String {
    return git_text(fixture, directory, &["rev-parse", "--quiet", "--verify", branch]).trim_end().to_string();
}

fn states(repo: &Path, state: &str) -> usize {
    let directory: PathBuf = repo.join(".git").join(git_policy_cli::work_item::WORK_DIRECTORY).join(state);
    return std::fs::read_dir(directory).map_or(0, Iterator::count);
}

/// Waits for a condition the service brings about after the commit returned,
/// up to the patience of these controls.
fn eventually(what: &str, mut condition: impl FnMut() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "{what} never happened");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A commit through the service returns,
/// and the service then pushes it:
/// the upstream has the commit,
/// both work records
///  are done,
/// and nothing was pushed by the wrapper.
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
    settle_every_work_record(&repo);
    drop(service);
    finish(&fixture, runtime.as_path());
}

/// Waits until every work record the commit left has settled to `done`:
/// the maintenance item the hand-off names,
/// and the push record the gate promoted,
/// which the service runs in its own time after the commit returned.
/// Why:
///  The records settle after the answer,
/// so a control that reads them once races the service's own work;
///  a record that never settles has to name the state it stayed in,
/// and the wait is bounded so it never waits for ever.
/// A `done` record is retained for a long time (`DONE_RETAINED`),
/// so the wait is for at least one per handed-over kind and never for exactly that many.
fn settle_every_work_record(repo: &Path) {
    let wanted: usize = git_policy_cli::service_handoff::HANDED_OVER.len();
    let began: Instant = Instant::now();
    loop {
        let unfinished: Vec<&str> = ["pending", "claimed", "recovering", "failed"]
            .into_iter()
            .filter(|state| { return states(repo, state) > 0; })
            .collect();
        if unfinished.is_empty() && states(repo, "done") >= wanted {
            return;
        }
        assert!(
            began.elapsed() < PATIENCE,
            "the work records settling: done {} of {wanted}, and records left in {unfinished:?}",
            states(repo, "done")
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A push that cannot succeed does not fail the commit;
/// the next commit through the service says so once,
/// plainly.
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
    assert_eq!(stderr.matches(super::support::push_unfinished_words()).count(), 1, "said once, and as a push that did not finish: {stderr}");
    drop(service);
    finish(&fixture, runtime.as_path());
}

/// A service with nothing to do exits on its own after the idle time,
/// with success,
/// so the manager does not restart it;
///  a connection before then keeps it for a further full idle time.
#[test]
fn an_idle_service_exits_with_success() {
    let (fixture, _repo, runtime) = prepared("service-idle");
    let socket: PathBuf = runtime.join("monochromatic-git-policy-cli/idle.sock");
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    let idle: Duration = Duration::from_secs(2);
    let mut child = super::support::service_command()
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
    let status = loop {
        if let Some(status) = child.try_wait().expect("status") {
            break status;
        }
        assert!(began.elapsed() < PATIENCE, "the idle service never exited");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(status.success(), "{status:?}");
    // Measured from before the connection that was its last activity, so a control delayed after it cannot shorten the time.
    assert!(began.elapsed() >= idle / 2, "it did not leave at once");
    finish(&fixture, runtime.as_path());
}

/// Starts the single per-user service of the release before one service per worktree,
/// as the manager's socket activation left
///  it:
/// this build's program announcing the earlier release on the one socket every worktree shared.
fn start_per_user_service(fixture: &Fixture, runtime: &Path) -> std::process::Child {
    let socket: PathBuf = runtime.join(LEGACY_SOCKET_RELATIVE);
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("socket directory");
    // Owned: its idle exit is hours away, so a control that fails before the wrapper retires it would leave it running.
    let child: std::process::Child = super::support::spawn_owned(
        super::support::service_command()
            .args(["--journal", fixture.root.join("journal").to_str().expect("UTF-8"), "--path", socket.to_str().expect("UTF-8")])
            .args(["--release", "0.5.5"])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", fixture.root.join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit()),
    )
    .expect("the per-user service starts");
    let began: Instant = Instant::now();
    while std::os::unix::net::UnixStream::connect(&socket).is_err() {
        assert!(began.elapsed() < PATIENCE, "the per-user service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return child;
}

/// A per-user service of the earlier release is running with its two units installed.
/// The first commit of this release asks it
///  to retire,
/// has the manager stop its socket,
/// waits for the service to leave by itself (the manager is asked about it,
/// and the
///  stop of the service goes after it left,
/// so a landing in progress is not cut),
/// and reload,
/// removes its units and its socket file,
/// starts the
///  worktree's own instance,
/// and the commit lands there.
/// The old service exits by itself with success.
#[test]
fn a_running_per_user_service_is_retired_and_the_worktrees_own_service_lands_the_commit() {
    let (fixture, repo, runtime) = prepared("provision-per-user");
    let units: PathBuf = runtime.join("systemd/user");
    std::fs::create_dir_all(&units).expect("unit directory");
    for name in [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME] {
        std::fs::write(units.join(name), "# Written by the git wrapper for its commit service (cli-git-unit)\n[Unit]\n").expect("earlier unit");
    }
    let mut old: std::process::Child = start_per_user_service(&fixture, runtime.as_path());
    let keep: Arc<Mutex<Vec<Running>>> = Arc::new(Mutex::new(Vec::new()));
    let bus = fake_bus_many(runtime.as_path(), false, 1, activation(fixture.root.clone(), runtime.clone(), Arc::clone(&keep)));
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let began: Instant = Instant::now();
    let status = loop {
        if let Some(status) = old.try_wait().expect("status") {
            break status;
        }
        assert!(began.elapsed() < PATIENCE, "the earlier service never retired");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(status.success(), "the earlier service retired by itself: {status:?}");
    assert_eq!(finished_batches(&fixture), 1, "the worktree's own service landed it");
    assert!(!units.join(LEGACY_SOCKET_UNIT_NAME).exists() && !units.join(LEGACY_SERVICE_UNIT_NAME).exists(), "its units are gone");
    assert!(!runtime.join(LEGACY_SOCKET_RELATIVE).exists(), "its socket file is gone");
    assert!(socket_for(runtime.as_path(), repo.join(".git").as_path()).exists(), "the worktree has its own socket");
    assert_eq!(bus.join().expect("bus"), vec![vec!["Hello", "StopUnit", "GetUnit", "Get", "StopUnit", "GetUnit", "Get", "GetUnit", "Get", "Reload", "StartUnit", "GetUnit", "Get", "GetUnit", "Get", "GetUnit", "Get"]]);
    keep.lock().expect("lock").clear();
    finish(&fixture, runtime.as_path());
}

/// What:
///  A runtime directory that leaves no room for one socket address is refused at once,
/// with the path,
/// the room this system has and the remedy.
/// Why:
///   A person whose `XDG_RUNTIME_DIR` is longer than 40 bytes (an agent workspace,
/// a container's own runtime directory)
/// waited two minutes before the command said only that the service did not start,
/// and the remedy it named (turning on user services)
/// had nothing to do with the cause.
#[test]
fn a_runtime_directory_no_address_can_hold_is_refused_at_once() {
    let (fixture, repo, _short) = prepared("provision-too-long");
    let scratch: PathBuf = super::service_commit::runtime_for("provision-too-long").join("x");
    std::fs::create_dir_all(&scratch).expect("the long runtime directory");
    let runtime: PathBuf = scratch.join("x".repeat(80));
    std::fs::create_dir_all(&runtime).expect("the runtime directory");
    // The route needs a session bus, and this one is never reached: the address is refused before that.
    std::fs::write(runtime.join("bus"), b"").expect("a bus that is never asked");
    let began: Instant = Instant::now();
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    let took: Duration = began.elapsed();
    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(took < PATIENCE / 6, "the refusal takes {took:?}, not the start ceiling: {stderr}");
    assert!(stderr.contains("XDG_RUNTIME_DIR"), "the setting that has to change is named: {stderr}");
    assert!(stderr.contains("107"), "the room this system has is named: {stderr}");
    assert!(!stderr.contains(LINGER_COMMAND), "the remedy for an account without user services is not the cause: {stderr}");
    assert_eq!(subject(&fixture, &repo), "base", "nothing was committed");
    assert!(!runtime.join("systemd").exists(), "no unit was written for a service that could never answer");
    finish(&fixture, runtime.as_path());
}

/// What:
///  A service the manager made listen at another address than this command waits at is named at once,
/// with both addresses and both runtime directories.
/// Why:
///   The unit's address is built from the manager's own runtime directory,
/// so a commit run with one of its own (under a unit,
/// in a container)
/// reaches a service that is running and healthy,
/// and waited two minutes before it said only that the service did not start.
#[test]
fn a_service_started_where_this_command_does_not_look_is_named_at_once() {
    let (fixture, repo, runtime) = prepared("provision-elsewhere");
    let elsewhere: String = String::from("/run/user/1000/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock");
    let bus = fake_bus_many_answering(runtime.as_path(), false, 1, Box::new(|| {}), vec![elsewhere.clone()]);
    let began: Instant = Instant::now();
    let output: Output = commit(&fixture, &repo, runtime.as_path(), true);
    let took: Duration = began.elapsed();
    let stderr: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(took < PATIENCE / 6, "the refusal takes {took:?}, not the start ceiling: {stderr}");
    assert!(stderr.contains(elsewhere.as_str()), "the address the service listens at is named: {stderr}");
    assert!(stderr.contains("/run/user/1000"), "the manager's runtime directory is named: {stderr}");
    assert!(stderr.contains(runtime.to_string_lossy().as_ref()), "the runtime directory this command used is named: {stderr}");
    assert!(stderr.contains("XDG_RUNTIME_DIR"), "the setting that has to change is named: {stderr}");
    assert_eq!(subject(&fixture, &repo), "base", "nothing was committed");
    assert_eq!(finished_batches(&fixture), 0, "no service landed anything");
    bus.join().expect("bus");
    finish(&fixture, runtime.as_path());
}
