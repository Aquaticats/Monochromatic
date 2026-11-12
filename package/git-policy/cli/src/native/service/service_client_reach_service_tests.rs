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

/// What: A service that greeted and then says nothing is the service that stopped.
/// Why:  Its heartbeats are what tell a working batch from a stopped process; a command that saw a greeting knows the difference.
#[test]
fn a_service_that_greeted_and_then_says_nothing_is_the_service_that_stopped() {
    use std::io::Write;
    let socket: String = address("greeted-then-silent");
    let service = listen_at(socket.as_str()).expect("the service listens");
    let serving = std::thread::spawn(move || {
        let (mut first, _) = service.accept().expect("the first connection");
        let _sent = first.write_all(line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice());
        let (mut second, _) = service.accept().expect("the second connection");
        let _also = second.write_all(line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) }).as_slice());
        // The second connection says nothing more the command can use.
        std::thread::sleep(Duration::from_millis(300));
    });
    let mut announced: bool = false;
    let reached = reach(&config(socket.as_str()), &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), true);
    assert_eq!(reached.err(), Some(ClientFailure::ServiceSilent));
    serving.join().expect("the service ends");
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
