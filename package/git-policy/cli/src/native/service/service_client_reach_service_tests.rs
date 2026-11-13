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

/// What: A connection something accepts but never answers is named as that, not as a start that never happened.
/// Why:  The two states have two different next steps, and the old words sent a person to a log that holds nothing.
#[test]
fn an_address_that_answers_nothing_is_named_truly() {
    let socket: String = address("answers-nothing");
    let service = listen_at(socket.as_str()).expect("the service listens");
    let holding = std::thread::spawn(move || {
        let mut held = Vec::new();
        while let Ok((stream, _)) = service.accept() {
            held.push(stream);
            if held.len() > 64 {
                break;
            }
        }
    });
    let mut announced: bool = false;
    let reached = reach(&config(socket.as_str()), &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), false);
    match reached.err() {
        Some(ClientFailure::NothingAnswered { .. }) => {}
        other => panic!("an address whose connections nothing answers must be named so, not {other:?}"),
    }
    std::mem::forget(holding);
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
///  The line a wait prints when nothing has answered is written while the wait still goes on,
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
    let mut notices: TimesOfLines = TimesOfLines::start();
    let mut announced: bool = false;
    let began: std::time::Instant = std::time::Instant::now();
    let failure: ClientFailure = reach(&settings, &mut notices, &mut announced, &mut ReachTrace::default(), false).expect_err("nothing answered, so nothing is reached");
    let waited: Duration = began.elapsed();
    let said: Vec<(Duration, String)> = notices.lines();
    let text: String = said.iter().map(|(_when, line)| { return line.as_str(); }).collect();
    let telling: Vec<&(Duration, String)> = said.iter().filter(|(_when, line)| { return line.contains("nothing has answered at"); }).collect();
    assert_eq!(telling.len(), 1, "one line, and one only: {text:?}");
    let (when, _line): &(Duration, String) = telling[0];
    assert!(text.contains(socket.as_str()), "the line names the address: {text}");
    assert!(*when < Duration::from_millis(1_500), "the line is read while the wait goes on ({when:?} of {waited:?}): {text}");
    assert!(waited >= settings.start_wait, "and the wait still ends at its own bound: {waited:?}");
    assert!(matches!(failure, ClientFailure::NothingAnswered { .. }), "the end says what the wait proved: {failure:?}");
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
