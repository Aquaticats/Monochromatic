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
///  A scope that reports no process number does not hold the base's start back, and the server it starts later is
///  greeted by the caller's own wait.
/// Why:
///  The review of `55f236a0a8` (finding 2) found `MANAGER_START_EXTRA`, a shipped second with no measurement and no owner
///  decision behind it, and asked for a measurement or for the wait to go. The wait goes: a scope whose middle process
///  reported no number may still start a server, and the base's start is what makes the commit's service exist: waiting
///  the second spends the command's own wait on a scope that has not answered, and the server that arrives later finds
///  the address taken (`service_server_socket_binding.rs`). This control shows the base starting at once and the
///  scope's server's greeting still being this command's answer.
#[test]
fn a_scope_that_reports_no_process_number_does_not_hold_the_base_start_back() {
    let root: PathBuf = folder("scope-without-a-number");
    let socket: PathBuf = root.join("3f9a.sock");
    let base_started: PathBuf = root.join("base-started.txt");
    // The base's start is a stand-in program that records that it was started, and binds nothing.
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
    // What the wait that went would have hidden: the base's start happened before the scope's server was ever up.
    assert!(base_started.exists(), "the base's start is not held back by a scope that reported no number: {}", std::fs::read_to_string(base_started.as_path()).unwrap_or_default());
}

/// A process group of this control's own, which is what a process number the kernel has given to another command would
/// name.
fn a_process_group_of_this_controls_own(name: &str) -> std::process::Child {
    use std::os::unix::process::CommandExt;
    let mut command: std::process::Command = std::process::Command::new("/usr/bin/tail");
    command.args(["--follow=name", "--retry", "/dev/null"]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    // SAFETY: the child calls `setsid` before it execs, so it is the leader of a group of its own, and nothing else runs
    // between the fork and that call.
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    return command.spawn().unwrap_or_else(|error| { panic!("{name} could not be started: {error}") });
}

/// What:
///  A scope that made nothing answer ends the service this worktree's own note proves, and never a process named by the
///  number the scope reported.
/// Why:
///  The review of `55f236a0a8` (finding 1) found `kill(-pid, SIGKILL)` of a number read up to eleven seconds earlier: a
///  number the kernel has given to another process group by then is somebody else's processes, and ending them is harm
///  outside the repository. The proof that exists is the note the service writes, with its own birth identity, and this
///  control puts a stranger's group where the reused number would be.
#[test]
fn a_scope_that_made_nothing_answer_ends_its_own_service_and_no_other() {
    let root: PathBuf = folder("scope-ends-its-own");
    let socket: PathBuf = root.join("3f9a.sock");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir_all(journal.as_path()).expect("the journal folder");
    let base_started: PathBuf = root.join("base-started.txt");
    let program: PathBuf = root.join("service-program");
    std::fs::write(program.as_path(), format!("#!/bin/sh\nprintf 'started\\n' >> '{}'\n", base_started.display())).expect("the stand-in is written");
    std::fs::set_permissions(program.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o700)).expect("the stand-in runs");
    // The service this worktree's note names: a process of this control's own, with the note written as the service writes it.
    let mut service: std::process::Child = a_process_group_of_this_controls_own("the service");
    let service_pid: i32 = i32::try_from(service.id()).expect("a process number");
    let service_identity: String = crate::process_identity::process_birth_identity(i64::from(service_pid)).expect("a readable identity").expect("the service is alive");
    let note: PathBuf = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    std::fs::write(note.as_path(), format!("{service_pid} {service_identity}\n")).expect("the service's own note");
    // The stranger: another process group, whose leader's number is what the seam reports as the scope's leader.
    let mut stranger: std::process::Child = a_process_group_of_this_controls_own("the stranger");
    let stranger_pid: i32 = i32::try_from(stranger.id()).expect("a process number");
    // The greeting: once the service this control wrote the note for is gone, a server binds the address and greets, so the
    // control cannot wait out the start's own bound, and a scope that never ends its service is named by a deadline here.
    let greeted_socket: PathBuf = socket.clone();
    let greeter = std::thread::spawn(move || {
        // What: The address starts answering only once the service the note names is gone, which is the end attempt's work.
        // Why:  A greeting that came earlier would make the start end before the attempt, and this control is about what the
        //       attempt does. The deadline is longer than the start's own bound, so a start that never ends its service is
        //       named by the greeting that follows it and by the assertion, never by a hang.
        let deadline: std::time::Instant = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while crate::auto_push_reconcile_basics_and_limits::process_is_alive(u64::try_from(service_pid).unwrap_or(0)) && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let Ok(listener) = std::os::unix::net::UnixListener::bind(greeted_socket.as_path()) else { return };
        let line: String = format!("{}\n", crate::service_protocol_encode_messages_to_lines::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version: crate::service_protocol::MIN_SERVED_PROTOCOL, release: Some(String::from("test")) }));
        let began: std::time::Instant = std::time::Instant::now();
        while began.elapsed() < std::time::Duration::from_secs(20) {
            if let Ok((mut stream, _)) = listener.accept() {
                use std::io::Write;
                let _sent = stream.write_all(line.as_bytes());
            }
        }
    });
    let server: ServerToStart = ServerToStart {
        socket: socket.clone(),
        instance: String::from("3f9a"),
        journal,
        git: PathBuf::from("/usr/bin/git"),
        program,
        manager: Some(ManagerStart { program: PathBuf::from("/bin/true") }),
        named_runtime_dir: None,
        bus_address: None,
    };
    let mut asked: Box<dyn FnMut(&ManagerStart, &ServerToStart) -> std::io::Result<Option<i32>>> = Box::new(move |_manager, _server| { return Ok(Some(stranger_pid)); });
    let began: std::time::Instant = std::time::Instant::now();
    let attempt: StartAttempt = start_seamed(&server, asked.as_mut());
    let took: std::time::Duration = began.elapsed();
    // Both children are this control's own, so their state is read by reaping them: a killed child that is not reaped yet is a
    // zombie, and a zombie still answers a signal.
    let service_ended: bool = matches!(service.try_wait(), Ok(Some(_status)));
    let stranger_running: bool = matches!(stranger.try_wait(), Ok(None));
    let _killed: std::io::Result<()> = stranger.kill();
    let _reaped: std::io::Result<std::process::ExitStatus> = stranger.wait();
    if !service_ended {
        let _also_killed: std::io::Result<()> = service.kill();
        let _also_reaped: std::io::Result<std::process::ExitStatus> = service.wait();
    }
    std::mem::forget(greeter);
    assert_eq!(attempt, StartAttempt::Greeted, "the greeting the control started answers the wait: {attempt:?}");
    // What is asked first is the harm the review names: processes this command did not start, ended by a number.
    assert!(stranger_running, "the process group whose number the scope reported is untouched, because a number is no proof of what it names by then: {took:?}");
    assert!(service_ended, "and the service this worktree's note proves is the one that was ended, so the address could be bound by what greets: {took:?}");
}
