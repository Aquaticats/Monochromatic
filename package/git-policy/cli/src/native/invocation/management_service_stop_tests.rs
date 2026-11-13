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
    let stopped = stop(socket.as_path(), "3f9a");
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
    let stopped = stop(address("none").as_path(), "3f9a");
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
