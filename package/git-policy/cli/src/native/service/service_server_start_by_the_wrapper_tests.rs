//! Controls of the one start of a worktree's service:
//! the starter marker, the greeting, and the takeover of a killed starter's marker.

use super::*;

/// A folder of this test's own, short enough for a socket address.
fn folder(name: &str) -> PathBuf {
    return crate::test_support::short_directory_for_sockets(format!("server-start-{name}").as_str());
}

/// What: One taker of the marker wins and a second finds it held.
/// Why:  Two commands that both found no service must not both start one.
#[test]
fn one_taker_of_the_marker_wins() {
    let root = folder("marker");
    let marker = root.join("3f9a.starting");
    let first = StarterMarker::take(marker.as_path()).expect("the marker is created").expect("the first command takes the marker");
    assert!(StarterMarker::take(marker.as_path()).expect("the marker is there").is_none(), "the second command finds it held");
    drop(first);
    assert!(!marker.exists(), "the marker goes when the start is over");
    assert!(StarterMarker::take(marker.as_path()).expect("the marker is created").is_some(), "and the next start takes it again");
}

/// What: A marker whose owner is gone is nobody's and is taken over.
/// Why:  A command killed between the marker and the greeting must not block every later start.
#[test]
fn a_marker_whose_owner_is_gone_is_taken_over() {
    let root = folder("takeover");
    let marker = root.join("3f9a.starting");
    std::fs::write(marker.as_path(), "1 linux:000000\n").expect("a killed command's marker");
    assert!(!holder_is_alive(marker.as_path()), "the marker names no living process");
    assert!(StarterMarker::take(marker.as_path()).expect("the marker is created").is_some(), "the next start takes it");
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
        // Every connection is greeted, because the probes before the start left connections nobody read. The loop ends after
        // a few, so the control's thread ends: a greeter that accepted for ever would hold this test at its join.
        let line: String = format!("{}\n", crate::service_protocol_encode_messages_to_lines::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version: 1, release: Some(String::from("test")) }));
        let mut greeted: usize = 0;
        while greeted < 6 {
            let Ok((mut stream, _)) = listener.accept() else { return };
            let _sent = stream.write_all(line.as_bytes());
            greeted += 1;
            // The connection stays open while the greeting is read.
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    });
    assert!(waits_for_a_greeting(socket.as_path(), START_WAIT), "a service that greets answers");
    // The greeter's thread ends by itself; a control that waited for it would wait on the socket it still holds.
    std::mem::forget(greeter);
}

/// A wait with no service ends at its bound and does not hang.
#[test]
fn a_wait_with_nothing_at_the_address_ends_at_its_bound() {
    let root = folder("wait");
    let began = std::time::Instant::now();
    assert!(!waits_for_a_greeting(root.join("none.sock").as_path(), Duration::from_millis(50)));
    assert!(began.elapsed() < Duration::from_secs(5), "the bound is the bound");
}

/// What:
///  A command that finds an address accepting but greeting nothing starts no server and keeps waiting.
/// Why:
///  A service that is recovering, starved or stopped in a debugger holds its address and says nothing;
/// a server started beside it could not bind and would exit, and the command would report a start that did not greet
///  while a live service was there all along. The waiting is what makes the address's silence safe
///  (`service_server_socket_binding.rs`): the command's own ceiling bounds it.
#[test]
fn a_command_waits_at_an_address_that_accepts_and_greets_nothing() {
    let root: PathBuf = folder("waits-while-silent");
    let socket: PathBuf = root.join("3f9a.sock");
    // A service that holds the address, accepts every connection, and greets nothing: alive and slow.
    let silent = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("the service's address");
    let holding = std::thread::spawn(move || {
        let mut held: Vec<std::os::unix::net::UnixStream> = Vec::new();
        while let Ok((stream, _)) = silent.accept() {
            held.push(stream);
            if held.len() > 16 {
                break;
            }
        }
    });
    let server = ServerToStart {
        socket: socket.clone(),
        instance: String::from("3f9a"),
        journal: root.join("journal"),
        git: PathBuf::from("/usr/bin/git"),
        // A program that runs and exits without binding, so nothing about this control depends on a missing file: the address
        //  is held by the service that is there.
        program: PathBuf::from("/bin/true"),
        manager: None,
        named_runtime_dir: None,
        bus_address: None,
    };
    let began: std::time::Instant = std::time::Instant::now();
    let attempt: StartAttempt = start(&server);
    assert_eq!(attempt, StartAttempt::AnotherCommandStartsIt, "the command waits for the service that is there");
    // Two greeting reads (the look, and the look after the marker) precede the decision, so the wait is seconds at most and
    //  never the start wait a spawned server would be given.
    assert!(began.elapsed() < Duration::from_secs(4), "it returns in seconds, not after a start wait: {:?}", began.elapsed());
    // Nothing was started: the log the program would have written is not there.
    assert!(!server.log().exists(), "no server was started beside the live one");
    std::mem::forget(holding);
}

/// What:
///  An address whose listening socket is full and never accepts is not answered on, and the answer comes within the bound.
/// Why:
///  A plain connect waits for as long as the program that listens stays stopped, and every deadline of this program sat
/// after the connect returned: the review of `3d5c1a0d8c` measured a commit still waiting after 30 s for exactly that,
///  where a healthy one took 42 ms. The connection is now made within `CONNECT_CEILING`, so the look ends.
#[test]
fn an_address_whose_queue_is_full_is_not_answered_on_within_the_bound() {
    let root = folder("full-queue");
    let socket = root.join("3f9a.sock");
    let listener = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a listening socket");
    let queued = crate::test_support::fill_a_socket_backlog(socket.as_path());
    assert!(!queued.is_empty(), "the kernel queued at least one connection");
    let began = std::time::Instant::now();
    assert!(!answered(socket.as_path()), "a socket that takes no connection is not answered on");
    let took = began.elapsed();
    assert!(
        took < crate::wait_ceilings::CONNECT_CEILING + Duration::from_secs(1),
        "the look ends at its own bound ({took:?}), not when the service moves"
    );
    // And the start of a server there waits rather than spawning one that cannot bind.
    let server = ServerToStart {
        socket: socket.clone(),
        instance: String::from("3f9a"),
        journal: root.join("journal"),
        git: PathBuf::from("/usr/bin/git"),
        program: PathBuf::from("/bin/true"),
        manager: None,
        named_runtime_dir: None,
        bus_address: None,
    };
    let began = std::time::Instant::now();
    assert_eq!(start(&server), StartAttempt::AnotherCommandStartsIt, "a full queue is a service that is there");
    assert!(began.elapsed() < Duration::from_secs(10), "and the start does not wait out its ceiling: {:?}", began.elapsed());
    drop(queued);
    drop(listener);
}
