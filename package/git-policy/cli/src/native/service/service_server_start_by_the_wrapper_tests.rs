//! Controls of the one start of a worktree's service:
//! the starter marker, the greeting, and the takeover of a killed starter's marker.

use super::*;

fn folder(name: &str) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture_root().join(format!("server-start-{name}"));
    std::fs::create_dir_all(&root).expect("fixture folder");
    return root;
}

/// What: One taker of the marker wins and a second finds it held.
/// Why:  Two commands that both found no service must not both start one.
#[test]
fn one_taker_of_the_marker_wins() {
    let root = folder("marker");
    let marker = root.join("3f9a.starting");
    let first = StarterMarker::take(marker.as_path()).expect("the first command takes the marker");
    assert!(StarterMarker::take(marker.as_path()).is_none(), "the second command finds it held");
    drop(first);
    assert!(!marker.exists(), "the marker goes when the start is over");
    assert!(StarterMarker::take(marker.as_path()).is_some(), "and the next start takes it again");
}

/// What: A marker whose owner is gone is nobody's and is taken over.
/// Why:  A command killed between the marker and the greeting must not block every later start.
#[test]
fn a_marker_whose_owner_is_gone_is_taken_over() {
    let root = folder("takeover");
    let marker = root.join("3f9a.starting");
    std::fs::write(marker.as_path(), "1 linux:000000\n").expect("a killed command's marker");
    assert!(!holder_is_alive(marker.as_path()), "the marker names no living process");
    assert!(StarterMarker::take(marker.as_path()).is_some(), "the next start takes it");
}

/// What: Nothing at an address answers, and a service that greets does.
/// Why:  The greeting is what tells this service from a file someone left at its address.
#[test]
fn only_a_service_that_greets_answers() {
    let root = folder("greeting");
    let socket = root.join("3f9a.sock");
    assert!(!answered(socket.as_path()), "nothing at a path that is not there answers");
    let listener = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a socket");
    assert!(!answered(socket.as_path()), "a connection nothing answers on is not a greeting");
    let greeter = std::thread::spawn(move || {
        use std::io::Write;
        let (mut stream, _) = listener.accept().expect("a connection");
        let line: String = format!("{}\n", crate::service_protocol_encode_messages_to_lines::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version: 1, release: Some(String::from("test")) }));
        let _sent = stream.write_all(line.as_bytes());
        // The connection stays open while the greeting is read.
        std::thread::sleep(std::time::Duration::from_millis(200));
    });
    assert!(waits_for_a_greeting(socket.as_path(), START_WAIT), "a service that greets answers");
    greeter.join().expect("the greeter ends");
}

/// A wait with no service ends at its bound and does not hang.
#[test]
fn a_wait_with_nothing_at_the_address_ends_at_its_bound() {
    let root = folder("wait");
    let began = std::time::Instant::now();
    assert!(!waits_for_a_greeting(root.join("none.sock").as_path(), Duration::from_millis(50)));
    assert!(began.elapsed() < Duration::from_secs(5), "the bound is the bound");
}
