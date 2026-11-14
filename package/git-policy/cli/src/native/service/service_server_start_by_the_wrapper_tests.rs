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
    let start_began = std::time::Instant::now();
    assert_eq!(start(&server), StartAttempt::AnotherCommandStartsIt, "a full queue is a service that is there");
    assert!(start_began.elapsed() < Duration::from_secs(10), "and the start does not wait out its ceiling: {:?}", start_began.elapsed());
    drop(queued);
    drop(listener);
}

/// What:
///  A marker is never readable without its owner:
///  a command that publishes at the same instant either finds a whole marker naming a live command, or takes the place.
/// Why:
///  The review of `76a08c9fac` (finding 3, part 1) found the marker created and its owner written afterwards, so a second
///  command read an empty marker in that window, took it for a dead command's, removed it and started its own server:
///  two starts for one worktree. The hold point here is between the marker's making and its publication.
#[test]
fn a_marker_is_never_readable_without_its_owner() {
    let root: PathBuf = folder("whole-marker");
    let marker: PathBuf = root.join("3f9a.starting");
    let publishers: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(2));
    let second_go: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(2));
    let first_marker: PathBuf = marker.clone();
    let first_publishers: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::clone(&publishers);
    let first_second_go: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::clone(&second_go);
    let first = std::thread::spawn(move || {
        return StarterMarker::take_seamed(first_marker.as_path(), &mut || {
            first_publishers.wait();
            first_second_go.wait();
        });
    });
    publishers.wait();
    // What a second command finds at the marker's name at that instant: nothing at all, or a whole marker naming its owner.
    let found: Option<String> = std::fs::read_to_string(marker.as_path()).ok();
    if let Some(text) = found.as_ref() {
        assert!(crate::service_server_pid_note_of_an_instance::parse(text).is_some(), "a marker that is there names its owner, never nobody: {text:?}");
        assert!(holder_is_alive(marker.as_path()), "and that owner is a live command");
    }
    // The second command runs its own take there, and exactly one of the two ends holding the marker.
    let second = StarterMarker::take(marker.as_path()).expect("the second command asks for the marker");
    let second_taken: bool = second.is_some();
    second_go.wait();
    let first_taken: bool = first.join().expect("the first command ends").expect("the first command answers").is_some();
    let held: usize = [first_taken, second_taken].iter().filter(|taken| { return **taken; }).count();
    assert_eq!(held, 1, "exactly one command holds the marker: the first {first_taken}, the second {second_taken}");
    let text: String = std::fs::read_to_string(marker.as_path()).expect("the one holder's marker is there");
    assert!(crate::service_server_pid_note_of_an_instance::parse(text.as_str()).is_some(), "and it names its owner: {text:?}");
}

/// What:
///  A start that ends removes the marker only while it still holds its own text.
/// Why:
///  Removed by its path alone, a finishing start removed the marker another command had published by then, and a third
///  command took that place: two starts beside one address (the review of `76a08c9fac`, finding 3).
#[test]
fn a_start_that_ends_does_not_remove_a_marker_another_command_published() {
    let root: PathBuf = folder("marker-of-another");
    let marker: PathBuf = root.join("3f9a.starting");
    let taken: StarterMarker = StarterMarker::take(marker.as_path()).expect("the marker is made").expect("the first command takes it");
    // The marker this command holds is replaced by another command's, as a command that took over a dead marker would.
    let another: String = format!("1 linux:000000\n");
    std::fs::write(marker.as_path(), another.as_str()).expect("another command's marker");
    drop(taken);
    assert_eq!(std::fs::read_to_string(marker.as_path()).ok(), Some(another), "the marker of another command is left where it is");
}

/// What:
///  A scope that leaves no process number is given its own wait before a base server is started beside it.
/// Why:
///  The review of `76a08c9fac` (finding 3, part 2) found that a `systemd-run` whose middle process reported no number can
///  start a server seconds later, beside the one the base start had already made. The scope's own start is what this
///  command waits for, and the base's start is still there when the scope starts nothing.
#[test]
fn a_scope_that_leaves_no_process_number_is_given_its_own_wait_before_a_base_server_starts() {
    let root: PathBuf = folder("scope-without-a-number");
    let socket: PathBuf = root.join("3f9a.sock");
    let base_started: PathBuf = root.join("base-started.txt");
    // The base's start is a stand-in program that records that it was started.
    let program: PathBuf = root.join("service-program");
    std::fs::write(program.as_path(), format!("#!/bin/sh\nprintf 'started\\n' >> '{}'\n", base_started.display())).expect("the stand-in is written");
    std::fs::set_permissions(program.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("the stand-in runs");
    let server: ServerToStart = ServerToStart {
        socket: socket.clone(),
        instance: String::from("3f9a"),
        journal: root.join("journal"),
        git: PathBuf::from("/usr/bin/git"),
        program,
        manager: Some(ManagerStart { program: PathBuf::from("/bin/true") }),
        named_runtime_dir: None,
        bus_address: None,
    };
    let late_socket: PathBuf = socket.clone();
    let mut asked: Box<dyn FnMut(&ManagerStart, &ServerToStart) -> std::io::Result<Option<i32>>> = Box::new(move |_manager, _server| {
        // What the scope does: a server of its own greets the address a moment later, and no number came back for it.
        let scope_socket: PathBuf = late_socket.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let Ok(listener) = std::os::unix::net::UnixListener::bind(scope_socket.as_path()) else { return };
            let line: String = format!("{}\n", crate::service_protocol_encode_messages_to_lines::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version: crate::service_protocol::MIN_SERVED_PROTOCOL, release: Some(String::from("test")) }));
            // The scope's server greets every connection for a few seconds, and ends with this control rather than holding the
            // address for the rest of the suite.
            let began: std::time::Instant = std::time::Instant::now();
            while began.elapsed() < std::time::Duration::from_secs(5) {
                if let Ok((mut stream, _)) = listener.accept() {
                    use std::io::Write;
                    let _sent = stream.write_all(line.as_bytes());
                }
            }
        });
        return Ok(None);
    });
    let attempt: StartAttempt = start_seamed(&server, asked.as_mut());
    assert_eq!(attempt, StartAttempt::Greeted, "the scope's own server greets, and that is this command's answer");
    assert!(!base_started.exists(), "no base server was started beside the scope: {}", std::fs::read_to_string(base_started.as_path()).unwrap_or_default());
}
