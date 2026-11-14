//! Controls of reaching the commit service:
//! a greeting is the service,
//! silence at an address is not,
//! and a service of another release is asked to leave.

use super::*;
use crate::service_client_settings_and_watches::Activation;
use crate::service_server::Bind;
use crate::service_protocol_encode_messages_to_lines::encode_service;
use std::io::Read;

/// An abstract address of its own for one control,
/// so two controls of this process never share a socket.
fn address(name: &str) -> String {
    return format!("cli-git-test-reach-{}-{name}", std::process::id());
}

/// The settings of a control:
/// an abstract address,
/// short waits,
/// and no starter,
/// because the controls place their own answer at the address.
fn config(socket: &str) -> ClientConfig {
    return ClientConfig {
        address: Bind::Abstract(String::from(socket)),
        activation: Activation::StartedByTheWrapper,
        start_wait: Duration::from_millis(150),
        heartbeat_silence: Duration::from_millis(100),
        starter: None,
        keep_fresh: None,
        answer_ceiling: Duration::from_secs(5),
        withdraw_ceiling: Duration::from_secs(5),
        waiting_notice_after: Duration::from_secs(60),
        extra_wait: Duration::from_secs(1),
        start_ceiling: Duration::from_millis(400),
        recovering_ceiling: crate::wait_ceilings::RECOVERING_CEILING,
    };
}

/// A listener at an abstract name,
/// or nothing when the name cannot be bound.
fn listen_at(name: &str) -> Option<std::os::unix::net::UnixListener> {
    use std::os::linux::net::SocketAddrExt;
    let address = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).ok()?;
    return std::os::unix::net::UnixListener::bind_addr(&address).ok();
}

/// A line of this protocol, as the service writes it.
fn line(message: &ServiceMessage) -> Vec<u8> {
    let mut bytes: Vec<u8> = encode_service(message).into_bytes();
    bytes.push(b'\n');
    return bytes;
}

/// What: A service that greets in this protocol is reached, with the version and release it announced.
/// Why:  The greeting is the one fact that tells the service from a file at its address.
#[test]
fn a_service_that_greets_is_reached() {
    use std::io::Write;
    let socket: String = address("greets");
    let service = listen_at(socket.as_str()).expect("the service listens");
    let serving = std::thread::spawn(move || {
        let (mut stream, _) = service.accept().expect("a connection");
        let _sent = stream.write_all(line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice());
        std::thread::sleep(Duration::from_millis(200));
    });
    let mut announced: bool = false;
    let reached = reach(&config(socket.as_str()), &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), false);
    let (reader, version, release) = reached.expect("a service that greets is reached");
    assert_eq!(version, PROTOCOL_VERSION);
    assert_eq!(release.as_deref(), Some(RELEASE));
    drop(reader);
    serving.join().expect("the service ends");
}

/// What: An address nothing listens on ends the wait with the plain failure within its bound.
/// Why:  A name of a file at an address is not a service, and nothing may wait forever.
#[test]
fn an_address_nothing_listens_on_ends_with_the_plain_failure() {
    let began = Instant::now();
    let mut announced: bool = false;
    let reached = reach(&config(address("nothing-listens").as_str()), &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), false);
    assert_eq!(reached.err(), Some(ClientFailure::DidNotStart));
    assert!(began.elapsed() < Duration::from_secs(5), "the wait ends at its bound");
}


/// What: A service that greeted earlier in this command and now says nothing is the service that stopped.
/// Why:  Its heartbeats are what tell a working batch from a stopped process; a command that saw a greeting knows the difference,
///       and must not wait out a start ceiling for a process that will never answer.
#[test]
fn a_service_that_greeted_and_then_says_nothing_is_the_service_that_stopped() {
    let socket: String = address("greeted-then-silent");
    let service = listen_at(socket.as_str()).expect("the service listens");
    let serving = std::thread::spawn(move || {
        // Every connection is accepted and answered with nothing: the process this command greeted is stopped inside itself.
        let mut held: Vec<std::os::unix::net::UnixStream> = Vec::new();
        while let Ok((stream, _)) = service.accept() {
            held.push(stream);
            if held.len() > 8 {
                break;
            }
        }
    });
    let mut announced: bool = false;
    let reached = reach(&config(socket.as_str()), &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), true);
    assert_eq!(reached.err(), Some(ClientFailure::ServiceSilent));
    std::mem::forget(serving);
}

/// What: A service of another release is asked to leave in the version it announced, and the wait ends when it does not.
/// Why:  The replacement of another release is a retire message and this release's own start, never a manager.
#[test]
fn a_service_of_another_release_is_asked_to_retire() {
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    let socket: String = address("other-release");
    let service = listen_at(socket.as_str()).expect("the service listens");
    let asked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let marked: Arc<AtomicBool> = Arc::clone(&asked);
    let serving = std::thread::spawn(move || {
        while let Ok((mut stream, _)) = service.accept() {
            let _sent = stream.write_all(line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION - 1, release: Some(String::from("0.0.1-before")) }).as_slice());
            let mut heard = [0u8; 256];
            if let Ok(read) = stream.read(&mut heard) {
                let text = String::from_utf8_lossy(&heard[..read]).to_string();
                if text.contains("retire") {
                    marked.store(true, Ordering::SeqCst);
                }
            }
        }
    });
    let mut announced: bool = false;
    let reached = reach_current_untimed(&config(socket.as_str()), &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), false);
    assert_eq!(reached.err(), Some(ClientFailure::OtherRelease), "a service that will not leave ends the wait");
    assert!(asked.load(Ordering::SeqCst), "the other release was asked to leave");
    std::mem::forget(serving);
}

/// What:
///  The line a wait prints for a service that is there and has not answered is written while the wait still goes on,
///  and the wait still ends at its own bound.
/// Why:
///  A line read only at the end of the wait is a line a person sees after the thing it explains has stopped happening:
/// the start wait is where a commit spent ten seconds in silence.
/// The sink here records the moment of each write, which is the only way to see that the line came early.
#[test]
fn the_notice_of_a_wait_reaches_the_person_before_the_wait_ends() {
    let socket: String = address("client-reach-notice-while-waiting");
    let listener = listen_at(socket.as_str()).expect("the address is bound");
    let (finished, held, serving) = take_connections_and_greet_nothing(listener);
    let mut settings: ClientConfig = config(socket.as_str());
    // Twice the notice grace, so the line and the end of the wait are a second apart at the least.
    settings.start_wait = Duration::from_millis(2_400);
    settings.recovering_ceiling = Duration::from_millis(2_400);
    let mut notices: TimesOfLines = TimesOfLines::start();
    let mut announced: bool = false;
    let began: std::time::Instant = std::time::Instant::now();
    let failure: ClientFailure = reach(&settings, &mut notices, &mut announced, &mut ReachTrace::default(), false).expect_err("nothing answered, so nothing is reached");
    let waited: Duration = began.elapsed();
    let said: Vec<(Duration, String)> = notices.lines();
    let text: String = said.iter().map(|(_when, line)| { return line.as_str(); }).collect();
    let telling: Vec<&(Duration, String)> = said.iter().filter(|(_when, line)| { return line.contains("has not answered yet"); }).collect();
    assert_eq!(telling.len(), 1, "one line, and one only: {text:?}");
    let (when, _line): &(Duration, String) = telling[0];
    assert!(text.contains(socket.as_str()), "the line names the address: {text}");
    assert!(*when < Duration::from_millis(1_500), "the line is read while the wait goes on ({when:?} of {waited:?}): {text}");
    assert!(waited >= settings.start_wait, "and the wait still ends at its own bound: {waited:?}");
    assert!(matches!(failure, ClientFailure::ServiceRecovering { .. }), "the end says what the wait proved: {failure:?}");
    finished.store(true, std::sync::atomic::Ordering::Relaxed);
    serving.join().expect("the stand-in service's thread");
    drop(held);
}

/// A sink that records the moment it was given each line.
struct TimesOfLines {
    /// When the wait began.
    began: std::time::Instant,
    /// Every line,
    /// with the moment it was written.
    lines: std::sync::Arc<std::sync::Mutex<Vec<(Duration, String)>>>,
}

impl TimesOfLines {
    /// A sink whose clock starts now.
    fn start() -> Self {
        return Self { began: std::time::Instant::now(), lines: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())) };
    }

    /// Every line written so far, with when.
    fn lines(&self) -> Vec<(Duration, String)> {
        return self.lines.lock().expect("the sink's lines").clone();
    }
}

impl std::io::Write for TimesOfLines {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let when: Duration = self.began.elapsed();
        self.lines.lock().expect("the sink's lines").push((when, String::from_utf8_lossy(bytes).into_owned()));
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// A service that accepts every connection and greets nothing, held open until the control says it is done.
fn take_connections_and_greet_nothing(listener: std::os::unix::net::UnixListener) -> (std::sync::Arc<std::sync::atomic::AtomicBool>, std::sync::Arc<std::sync::Mutex<Vec<std::os::unix::net::UnixStream>>>, std::thread::JoinHandle<()>) {
    let finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let serving_finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&finished);
    let held: std::sync::Arc<std::sync::Mutex<Vec<std::os::unix::net::UnixStream>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let serving_held: std::sync::Arc<std::sync::Mutex<Vec<std::os::unix::net::UnixStream>>> = std::sync::Arc::clone(&held);
    let serving = std::thread::spawn(move || {
        let _set = listener.set_nonblocking(true);
        while !serving_finished.load(std::sync::atomic::Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _address)) => serving_held.lock().expect("held").push(stream),
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    return (finished, held, serving);
}

/// The settings of a control that meets a service of this release at a real address,
/// with the gate the retiring of an earlier release's units passes through open:
/// the runtime directory the environment named is this one,
/// and a bus address sits inside it.
fn config_with_a_gate(socket: &std::path::Path, runtime: &std::path::Path) -> ClientConfig {
    let mut settings: ClientConfig = config("gate-is-open");
    settings.address = Bind::Path(socket.to_path_buf());
    settings.starter = Some(crate::service_server_start_by_the_wrapper::ServerToStart {
        socket: socket.to_path_buf(),
        instance: String::from("3f9a"),
        journal: runtime.join("journal"),
        git: std::path::PathBuf::from("/usr/bin/git"),
        program: std::path::PathBuf::from("/bin/true"),
        manager: None,
        named_runtime_dir: Some(runtime.to_path_buf()),
        bus_address: Some(format!("unix:path={}", runtime.join("bus").display())),
    });
    return settings;
}

/// What:
///  A command served by its own release reads no unit folder and removes no earlier release's unit file.
/// Why:
///  The retiring of an earlier release's units exists for a service of an earlier release that holds this worktree's
/// address, and a command that meets only its own release touches nothing: it ran for every commit and wake of this
/// release's own service, which read the unit folder and asked the manager to stop units no earlier release ever wrote
///  (the review of `77f5d525e3`, finding 1).
#[test]
fn a_command_served_by_its_own_release_removes_no_earlier_releases_unit_file() {
    use std::io::Write;
    let runtime: std::path::PathBuf = crate::test_support::short_directory_for_sockets("reach-own-release");
    let socket: std::path::PathBuf = runtime.join("monochromatic-git-policy-cli").join("3f9a.sock");
    std::fs::create_dir_all(socket.parent().unwrap_or(runtime.as_path())).expect("the socket folder");
    // An earlier release's unit file, as it wrote it, in the folder the retiring reads.
    let units: std::path::PathBuf = runtime.join("systemd").join("user");
    std::fs::create_dir_all(units.as_path()).expect("the unit folder");
    let earlier_unit: std::path::PathBuf = units.join("monochromatic-git-policy-cli@.service");
    std::fs::write(
        earlier_unit.as_path(),
        "# Written by the git wrapper for its commit service (monochromatic-git-policy-cli-unit)\n[Unit]\nDescription=Commit service of one worktree for the git wrapper\n",
    )
    .expect("the earlier release's unit file");
    // This release's own service at the address.
    let service = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("the service listens");
    let serving = std::thread::spawn(move || {
        let (mut stream, _) = service.accept().expect("a connection");
        let _sent = stream.write_all(line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice());
        std::thread::sleep(Duration::from_millis(200));
    });
    let mut announced: bool = false;
    // The retiring of an earlier release's units is in `reach_current`, which is what the commit and the hand-over call.
    let reader = reach_current_untimed(&config_with_a_gate(socket.as_path(), runtime.as_path()), &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), false)
        .expect("this release's own service is reached");
    drop(reader);
    serving.join().expect("the service ends");
    assert!(earlier_unit.exists(), "a command that meets only its own release leaves an earlier release's unit file where it is");
}

/// What:
///  A service that is there and greets only after the start's own bound is waited for, and served.
/// Why:
///  A live service can be slow to greet while it returns to work: it recovers the batches an earlier release left open,
/// which can wait on a foreign index lock for a minute, and it builds its own runtime.
/// The start's bound (`START_WAIT`) is for a start this command made, and a service that took the connection is not one:
///  waiting only that long failed a commit the service would have served (the review of `77f5d525e3`, finding 3).
#[test]
fn a_service_that_greets_after_the_starts_own_bound_is_still_served() {
    use std::io::Write;
    let socket: String = address("greets-late");
    let service = listen_at(socket.as_str()).expect("the service listens");
    let serving = std::thread::spawn(move || {
        // What a service that is recovering looks like: it takes every connection and says nothing for a while, then it
        // greets every one. The while is a state of this control, not a speed of the machine: the bound it must outlast is
        // the start's own, a tenth of it.
        let began: Instant = Instant::now();
        let recovering_for: Duration = Duration::from_millis(400);
        while let Ok((mut stream, _)) = service.accept() {
            if began.elapsed() < recovering_for {
                continue;
            }
            let _sent = stream.write_all(line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice());
        }
    });
    let mut announced: bool = false;
    let mut settings: ClientConfig = config(socket.as_str());
    settings.start_wait = Duration::from_millis(150);
    settings.start_ceiling = Duration::from_millis(150);
    settings.recovering_ceiling = Duration::from_secs(5);
    let reached = reach(&settings, &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), false);
    let (reader, version, release) = reached.expect("a service that is there is waited for, and it greets");
    assert_eq!(version, PROTOCOL_VERSION);
    assert_eq!(release.as_deref(), Some(RELEASE));
    drop(reader);
    // The greeter ends when its listener is dropped, which is when the connection it greets closes.
    std::mem::forget(serving);
}

/// What:
///  A service that takes the connection and never greets ends the wait with the failure that says it is alive and
///  returning to work, within its own stated bound, and never with the failure of a start.
/// Why:
///  "Nothing has answered" about a live service sends the person to look for a service that exists, and says nothing about
///  what the service was doing; the sentence that says it is recovering names its log, which is where its recovery and the
///  lock it waits for are written (the review of `77f5d525e3`, finding 3).
#[test]
fn a_service_that_takes_the_connection_and_greets_nothing_is_said_to_be_recovering() {
    let socket: String = address("silent-but-there");
    let service = listen_at(socket.as_str()).expect("the service listens");
    let serving = std::thread::spawn(move || {
        let mut held: Vec<std::os::unix::net::UnixStream> = Vec::new();
        while let Ok((stream, _)) = service.accept() {
            held.push(stream);
            if held.len() > 16 {
                return;
            }
        }
    });
    let mut announced: bool = false;
    let mut settings: ClientConfig = config(socket.as_str());
    settings.start_ceiling = Duration::from_millis(150);
    settings.recovering_ceiling = Duration::from_millis(600);
    let began: Instant = Instant::now();
    let reached = reach(&settings, &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), false);
    let waited: Duration = began.elapsed();
    let failure: ClientFailure = reached.expect_err("a service that greets nothing fails the wait");
    assert!(matches!(failure, ClientFailure::ServiceRecovering { .. }), "a service that is there is not said to be absent: {failure:?}");
    assert!(waited >= settings.start_ceiling, "and the start's own bound was not the end of it: {waited:?}");
    assert!(waited < settings.recovering_ceiling + Duration::from_millis(600), "it ends within the bound a live service is given: {waited:?}");
    std::mem::forget(serving);
}
