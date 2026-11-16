//! Controls of `service stop`:
//! a service that answers is asked to leave and leaves,
//! one that does not answer is nothing to stop,
//! and a worktree whose Git directory cannot be resolved stops nothing.

use super::*;
use crate::service_protocol::{PROTOCOL_VERSION, ServiceMessage};
use crate::service_protocol_encode_messages_to_lines::encode_service;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;

/// A socket path of its own for one control, short enough for a socket address.
fn address(name: &str) -> std::path::PathBuf {
    let folder: std::path::PathBuf = crate::test_support::short_directory_for_sockets(format!("stop-{name}").as_str());
    return folder.join("service.sock");
}

/// A listener at a path,
/// which is the shape the command looks for.
fn listen_at(path: &Path) -> Option<UnixListener> {
    return UnixListener::bind(path).ok();
}

/// A line of this protocol, as the service writes it.
fn line(message: &ServiceMessage) -> Vec<u8> {
    let mut bytes: Vec<u8> = encode_service(message).into_bytes();
    bytes.push(b'\n');
    return bytes;
}

/// The standard-error text of a plan that carries one.
fn stderr_of(action: &Action) -> &str {
    return match action {
        Action::Exit { stderr, .. } => stderr.as_str(),
        Action::ExitBytes { stderr, .. } => std::str::from_utf8(stderr).expect("an explanation of text paths"),
        _ => "",
    };
}

/// The standard-output text of a plan that carries one.
fn stdout_of(action: &Action) -> &str {
    return match action {
        Action::Exit { stdout, .. } => stdout.as_str(),
        Action::ExitBytes { stdout, .. } => std::str::from_utf8(stdout).unwrap_or(""),
        _ => "",
    };
}

/// What:
///  A service that answers is asked to leave, and the command says it has.
/// Why:
///  This is the whole of the command: the retire message a replacement sends, over the worktree's own address,
/// with no manager anywhere in it.
#[test]
fn a_service_that_answers_is_asked_to_leave_and_the_command_says_so() {
    let socket: std::path::PathBuf = address("asked");
    let service: UnixListener = listen_at(socket.as_path()).expect("the service listens");
    let (asked, told) = std::sync::mpsc::channel::<bool>();
    let serving = std::thread::spawn(move || {
        while let Ok((mut stream, _)) = service.accept() {
            let _sent = stream.write_all(line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from("test")) }).as_slice());
            let mut heard: [u8; 256] = [0; 256];
            if let Ok(read) = stream.read(&mut heard) {
                let text: String = String::from_utf8_lossy(&heard[..read]).into_owned();
                if text.contains("retire") {
                    // The service leaves after the ask, so the address stops answering.
                    let _told = asked.send(true);
                    return;
                }
            }
        }
    });
    let stopped = stop(&crate::service_server::Bind::Path(socket.clone()), socket.parent().unwrap_or(std::path::Path::new(".")), "3f9a");
    told.recv_timeout(std::time::Duration::from_secs(5)).expect("the service was asked to leave");
    assert_eq!(stdout_of(&stopped), "The commit service of this worktree (instance 3f9a) was asked to leave and has.\n");
    assert_eq!(stderr_of(&stopped), "", "nothing is asked of the person");
    match &stopped {
        Action::Exit { code, .. } => assert_eq!(*code, 0),
        other => panic!("a plain exit was expected: {other:?}"),
    }
    std::mem::forget(serving);
}

/// What:
///  An address nothing answers on is nothing to stop, and that is said without a failure.
/// Why:
///  A worktree whose service has exited is the ordinary state between commands.
#[test]
fn an_address_nothing_answers_on_is_nothing_to_stop() {
    let stopped = stop(&crate::service_server::Bind::Path(address("none")), std::path::Path::new("."), "3f9a");
    assert!(stdout_of(&stopped).starts_with("No commit service answers for this worktree"), "{}", stdout_of(&stopped));
    assert_eq!(stderr_of(&stopped), "");
    match &stopped {
        Action::Exit { code, .. } => assert_eq!(*code, 0, "nothing to stop is not a failure"),
        other => panic!("a plain exit was expected: {other:?}"),
    }
}

/// What:
///  A Git directory that cannot be resolved stops nothing and names the cause.
/// Why:
///  The instance name is a hash of that directory, so a directory that cannot be resolved names no service to stop.
#[test]
fn an_unresolvable_git_directory_stops_nothing() {
    let root: std::path::PathBuf = crate::test_support::fixture("stop-unresolvable");
    let git_dir: std::path::PathBuf = root.join("absent/.git");
    let action = plan_service_stop(Vec::new().as_slice(), git_dir.as_path());
    assert!(stderr_of(&action).contains("cannot be resolved"), "{}", stderr_of(&action));
    assert!(stderr_of(&action).contains("nothing was stopped"), "{}", stderr_of(&action));
    match &action {
        Action::Exit { code, .. } => assert_eq!(*code, 1),
        other => panic!("a plain exit was expected: {other:?}"),
    }
    crate::test_support::remove(root.as_path());
}

/// What:
///  An account with neither a runtime directory nor a home is told so, and nothing is stopped.
/// Why:
///  A socket lives in one of those two places; with neither there is nothing this command could have stopped.
#[test]
fn an_account_with_nowhere_to_listen_stops_nothing() {
    let root: std::path::PathBuf = crate::test_support::fixture("stop-no-home");
    let git_dir: std::path::PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("git directory");
    let action = plan_service_stop(Vec::new().as_slice(), git_dir.as_path());
    assert!(stderr_of(&action).contains("no runtime directory and no home"), "{}", stderr_of(&action));
    crate::test_support::remove(root.as_path());
}

/// What:
///  A service that is there and does not answer is ended by its own note, and the command says what it did.
/// Why:
///  This command is the remedy every failure text names for that state; before the fix it answered "there was nothing to
///  stop" and exited 0 while the process kept the address (the review of `b896c60d97`, finding 1).
#[test]
fn a_service_that_is_there_and_does_not_answer_is_ended_by_the_note_that_names_it() {
    use std::os::unix::process::CommandExt;
    let root: std::path::PathBuf = crate::test_support::short_directory_for_sockets("stop-silent");
    let socket: std::path::PathBuf = root.join("3f9a.sock");
    // The service: a listener that takes connections and greets nothing, and a process of this control's own that its note
    // names with the process's birth identity.
    let service: UnixListener = listen_at(socket.as_path()).expect("the service listens");
    let holding = std::thread::spawn(move || {
        let mut held: Vec<std::os::unix::net::UnixStream> = Vec::new();
        while let Ok((stream, _)) = service.accept() {
            held.push(stream);
            if held.len() > 32 {
                return;
            }
        }
    });
    let mut process: std::process::Child = {
        let mut command: std::process::Command = std::process::Command::new("/usr/bin/tail");
        command.args(["--follow=name", "--retry", "/dev/null"]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        // SAFETY: the child calls `setsid` before it execs, and nothing else runs between the fork and that call.
        unsafe {
            command.pre_exec(|| {
                libc::setsid();
                return Ok(());
            });
        }
        command.spawn().expect("the service's process")
    };
    let pid: i32 = i32::try_from(process.id()).expect("a process number");
    let identity: String = crate::process_identity::process_birth_identity(i64::from(pid)).expect("a readable identity").expect("the process is alive");
    std::fs::write(crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a").as_path(), format!("{pid} {identity}\n")).expect("the service's own note");
    let stopped = stop(&crate::service_server::Bind::Path(socket.clone()), root.as_path(), "3f9a");
    let ended: bool = matches!(process.try_wait(), Ok(Some(_status)));
    if !ended {
        let _killed: std::io::Result<()> = process.kill();
        let _reaped: std::io::Result<std::process::ExitStatus> = process.wait();
    }
    std::mem::forget(holding);
    assert_eq!(stdout_of(&stopped), format!("The commit service of this worktree (instance 3f9a) was running and answered nothing, so it was ended (pid {pid}) and its address is free.\n"));
    assert_eq!(stderr_of(&stopped), "");
    assert!(ended, "the process the note named is gone");
}

/// What:
///  A service that greets, is asked to leave, and does not leave is ended by the process its own note proves.
/// Why:
///  The review of `e347be3115` (part two-a, finding 5): `service stop` is the command every failure names for a service that does
///  not answer, and a service held inside a landing that never ends answered "it finishes the batch it holds first; run this
///  command again" for ever. Nothing the words named ended it.
#[test]
fn a_service_that_greets_and_does_not_leave_is_ended_by_the_note_that_names_it() {
    use std::os::unix::process::CommandExt;
    let root: std::path::PathBuf = crate::test_support::short_directory_for_sockets("stop-stuck");
    let socket: std::path::PathBuf = root.join("3f9a.sock");
    // A service that greets (so it is asked to leave) and then holds its connection for ever, as one inside a landing that
    // never ends does: the ask is read and answered by nothing.
    let greeting = crate::service_protocol_encode_messages_to_lines::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version: crate::service_protocol::PROTOCOL_VERSION, release: Some(String::from("control")) });
    let service: UnixListener = listen_at(socket.as_path()).expect("the service listens");
    let holding = std::thread::spawn(move || {
        use std::io::Write;
        let mut held: Vec<std::os::unix::net::UnixStream> = Vec::new();
        loop {
            match service.accept() {
                Ok((mut stream, _)) => {
                    let _sent = stream.write_all(format!("{greeting}\n").as_bytes());
                    held.push(stream);
                }
                // A connection that was closed before this side took it is no reason to stop listening, which is the state
                //  this control is about: the service is still there.
                Err(_closed) => std::thread::sleep(std::time::Duration::from_millis(1)),
            }
            if held.len() > 64 {
                held.remove(0);
            }
        }
    });
    let mut process: std::process::Child = {
        let mut command: std::process::Command = std::process::Command::new("/usr/bin/tail");
        command.args(["--follow=name", "--retry", "/dev/null"]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        // SAFETY: the child calls `setsid` before it execs, and nothing else runs between the fork and that call.
        unsafe {
            command.pre_exec(|| {
                libc::setsid();
                return Ok(());
            });
        }
        command.spawn().expect("the service's process")
    };
    let pid: i32 = i32::try_from(process.id()).expect("a process number");
    let identity: String = crate::process_identity::process_birth_identity(i64::from(pid)).expect("a readable identity").expect("the process is alive");
    std::fs::write(crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a").as_path(), format!("{pid} {identity}\n")).expect("the service's own note");
    let stopped = stop(&crate::service_server::Bind::Path(socket.clone()), root.as_path(), "3f9a");
    let ended: bool = matches!(process.try_wait(), Ok(Some(_status)));
    if !ended {
        let _killed: std::io::Result<()> = process.kill();
        let _reaped: std::io::Result<std::process::ExitStatus> = process.wait();
    }
    std::mem::forget(holding);
    assert!(ended, "the process the note named is ended when the service does not leave: stdout [{}] stderr [{}]", stdout_of(&stopped), stderr_of(&stopped));
    // What the command says: that the service was asked, did not leave, and was ended. The address is still taken here, because
    //  the listener of this control is not the process the note names, so the words are the ones for that state.
    let said: String = format!("{}{}", stdout_of(&stopped), stderr_of(&stopped));
    assert!(said.contains("did not within") && said.contains("it was ended"), "the command says what it did: {said}");
    crate::test_support::remove(root.as_path());
}
