//! Verifies the channel to a user's service manager:
//! the login,
//! the peer check that refuses a socket answered by anything but the manager,
//! the call and its reply,
//! the error reply,
//! and the timeout.
//! A fake manager is a real Unix socket server in the test process,
//! so the reader meets bytes a server wrote,
//! and the peer the kernel reports is this process.
//! Nothing here talks to a real manager or bus,
//! and no manager is started.

/// The login and the peer credentials.
use crate::dbus_login::{external_token, peer_pid};
/// The channel under test.
use crate::manager_socket::{ManagerCaller, ManagerClient, ManagerEndpoint, ManagerError, PrivateSocketManager};
/// The message layout.
use crate::dbus_wire::{Argument, MethodCall, MANAGER_INTERFACE, decode_message, encode_reply};
/// Reading and writing the fake manager's socket.
use std::io::{Read, Write};
/// A real Unix socket the fake manager listens on.
use std::os::unix::net::{UnixListener, UnixStream};
/// Fixture paths.
use std::path::PathBuf;
/// The server thread and the waits.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

/// The method the tests call.
fn a_call() -> MethodCall {
    return MethodCall { interface: MANAGER_INTERFACE, member: "StopUnit", arguments: vec![Argument::Text("wg-quicker-update-x.scope".to_string()), Argument::Text("replace".to_string())] };
}

/// One fake manager:
/// a listening socket,
/// the calls it received,
/// and what it answers.
struct FakeManager {
    /// The socket the client connects to.
    socket: PathBuf,
    /// The calls received,
    /// one rendered line each.
    received: Arc<std::sync::Mutex<Vec<String>>>,
    /// Whether the server read the login only and then hung up.
    stop: Arc<AtomicBool>,
    /// The server thread.
    thread: Option<JoinHandle<()>>,
}

/// The answer a fake manager gives to a call.
#[derive(Clone, Copy)]
enum Answer {
    /// A successful return.
    Return,
    /// An error reply with this name and text.
    Error(&'static str, &'static str),
    /// Nothing at all,
    /// so the client waits until its timeout.
    Silence,
}

/// Reads one whole message from a stream.
fn read_message(stream: &mut UnixStream) -> Option<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    loop {
        if let Some(length) = crate::dbus_wire::message_length(&buffer)
            && buffer.len() >= length
        {
            return Some(buffer.get(..length)?.to_vec());
        }
        let mut chunk = [0u8; 4096];
        match stream.read(&mut chunk) {
            Ok(0) => return None,
            Ok(count) => buffer.extend_from_slice(chunk.get(..count)?),
            Err(_error) => return None,
        }
    }
}

/// Reads one login line of a connection.
fn read_login_line(stream: &mut UnixStream) -> String {
    let mut line: Vec<u8> = Vec::new();
    let mut byte = [0u8; 1];
    while !line.ends_with(b"\r\n") && line.len() < 512 {
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_read) => line.push(byte[0]),
            Err(_error) => break,
        }
    }
    return String::from_utf8_lossy(&line).into_owned();
}

/// A directory short enough for a Unix socket path,
/// which the kernel limits far below a build directory's depth.
fn socket_directory(name: &str) -> PathBuf {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let base = match runtime {
        Some(directory) => directory,
        None => match home {
            Some(directory) => directory.join("temp/agent"),
            None => crate::test_scratch::scratch_root(),
        },
    };
    return base.join(format!("wg-mgr-{}-{name}", std::process::id()));
}

/// Builds a fake manager that answers every call with `answer`.
fn fake_manager(name: &str, answer: Answer) -> FakeManager {
    let base = socket_directory(name);
    let _removed = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("make the fixture directory");
    let socket = base.join("private");
    let listener = UnixListener::bind(&socket).expect("bind the fake manager");
    let _nonblocking = listener.set_nonblocking(true);
    let received: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let stop = Arc::new(AtomicBool::new(false));
    let recorded = Arc::clone(&received);
    let stopping = Arc::clone(&stop);
    let thread = std::thread::Builder::new()
        .name("fake-manager".to_string())
        .spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _address)) => {
                        let _timeout = stream.set_read_timeout(Some(Duration::from_secs(5)));
                        let login = read_login_line(&mut stream);
                        if !login.starts_with("\u{0}AUTH EXTERNAL") {
                            continue;
                        }
                        let _wrote = stream.write_all(b"OK 00000000000000000000000000000000\r\n");
                        // The client sends BEGIN only after the login is agreed.
                        let begin = read_login_line(&mut stream);
                        if begin != "BEGIN\r\n" {
                            continue;
                        }
                        let Some(call) = read_message(&mut stream) else { continue };
                        let Ok(message) = decode_message(&call) else { continue };
                        recorded.lock().map(|mut calls| return calls.push(message.member.clone().unwrap_or_default())).ok();
                        match answer {
                            Answer::Return => {
                                let reply = encode_reply(message.serial, None);
                                let _wrote_return = stream.write_all(&reply);
                            }
                            Answer::Error(error_name, error_text) => {
                                let reply = encode_reply(message.serial, Some((error_name, error_text)));
                                let _wrote_error = stream.write_all(&reply);
                            }
                            Answer::Silence => std::thread::sleep(Duration::from_millis(400)),
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(2)),
                    Err(_other) => break,
                }
            }
        })
        .expect("start the fake manager");
    return FakeManager { socket, received, stop, thread: Some(thread) };
}

/// Stops the fake manager and waits for its thread.
impl FakeManager {
    /// Stops the server.
    fn close(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _joined = thread.join();
        }
    }

    /// The calls the fake manager received.
    fn calls(&self) -> Vec<String> {
        return self.received.lock().map(|calls| return calls.clone()).unwrap_or_default();
    }
}

/// Stops the fake manager on the way out,
/// so no test leaves a listening socket or a thread behind.
impl Drop for FakeManager {
    /// Stops and joins the server.
    fn drop(&mut self) {
        self.close();
    }
}

/// The endpoint of a fake manager,
/// with the pid the test process has,
/// because the server runs in it.
fn endpoint_of(manager: &FakeManager) -> ManagerEndpoint {
    return ManagerEndpoint { socket: manager.socket.clone(), expected_pid: i32::try_from(std::process::id()).unwrap_or(0) };
}

/// A client with a short timeout,
/// so a silent manager is a timeout and not a long wait.
fn quick_client() -> ManagerClient {
    return ManagerClient::with_timeout(Duration::from_millis(200));
}

/// The pid the kernel reports for a socket is the peer's,
/// which is how the watcher learns who answers before it says anything.
#[test]
fn the_peer_pid_is_the_process_that_listens() {
    let manager = fake_manager("peer-pid", Answer::Return);
    let stream = UnixStream::connect(&manager.socket).expect("connect the fake manager");
    assert_eq!(peer_pid(&stream).ok(), Some(i32::try_from(std::process::id()).unwrap_or(0)));
}

/// A socket answered by a process that is not the user's manager is refused,
/// the refusal names both pids,
/// and the call is never sent:
/// this is the control that fails on the commit before the peer check,
/// where every call went to whoever owned the bus name.
#[test]
fn a_socket_answered_by_another_process_is_refused_and_nothing_is_sent() {
    let manager = fake_manager("refused", Answer::Return);
    let endpoint = ManagerEndpoint { socket: manager.socket.clone(), expected_pid: 999_999 };
    let error = quick_client().call(&endpoint, &a_call()).expect_err("a stranger answers");
    match &error {
        ManagerError::Refused(text) => {
            assert!(text.contains("999999"), "the refusal names the pid the manager should have: {text}");
            assert!(text.contains(&std::process::id().to_string()), "the refusal names the pid that answered: {text}");
            assert!(text.contains(&manager.socket.display().to_string()), "the refusal names the socket: {text}");
        }
        other => panic!("not refused: {other:?}"),
    }
    std::thread::sleep(Duration::from_millis(100));
    assert!(manager.calls().is_empty(), "a message was sent to a process that is not the manager: {:?}", manager.calls());
}

/// The same socket,
/// with the right pid expected,
/// is called and answered.
#[test]
fn the_manager_is_called_when_it_is_the_peer() {
    let manager = fake_manager("called", Answer::Return);
    let reply = quick_client().call(&endpoint_of(&manager), &a_call()).expect("the manager answers");
    assert_eq!(reply.kind, crate::dbus_wire::METHOD_RETURN);
    assert_eq!(manager.calls(), ["StopUnit"]);
}

/// An error reply keeps the manager's error name and text.
#[test]
fn an_error_reply_keeps_its_name_and_text() {
    let manager = fake_manager("error", Answer::Error("org.freedesktop.systemd1.NoSuchProcess", "No such process"));
    let error = quick_client().call(&endpoint_of(&manager), &a_call()).expect_err("the manager refuses");
    match error {
        ManagerError::Failed(text) => {
            assert!(text.contains("org.freedesktop.systemd1.NoSuchProcess"), "{text}");
            assert!(text.contains("No such process"), "{text}");
        }
        other => panic!("not a manager error: {other:?}"),
    }
}

/// A manager that says nothing is a timeout and not a failure of the manager's answer,
/// so the policy pauses the moves instead of retrying at once.
#[test]
fn a_silent_manager_is_a_timeout() {
    let manager = fake_manager("silent", Answer::Silence);
    let error = quick_client().call(&endpoint_of(&manager), &a_call()).expect_err("nobody answers");
    assert!(matches!(error, ManagerError::TimedOut(_)), "{error:?}");
}

/// A socket that is not there is a failure naming the socket,
/// never a silent skip.
#[test]
fn a_missing_socket_is_a_failure_naming_it() {
    let endpoint = ManagerEndpoint { socket: PathBuf::from("/nonexistent/directory/systemd/private"), expected_pid: 1 };
    let error = quick_client().call(&endpoint, &a_call()).expect_err("no socket");
    match error {
        ManagerError::Failed(text) => assert!(text.contains("/nonexistent/directory/systemd/private"), "{text}"),
        other => panic!("not a failure: {other:?}"),
    }
}

/// The login token is the hexadecimal spelling of the decimal user id,
/// the form the EXTERNAL mechanism takes.
#[test]
fn the_login_token_spells_the_user_id_in_hexadecimal() {
    assert_eq!(external_token(0), "30");
    assert_eq!(external_token(1000), "31303030");
}

/// The production seam resolves the endpoint first and calls the manager,
/// and a refusal makes it resolve again instead of keeping a pid that is no longer the manager's.
#[test]
fn the_production_seam_refuses_a_stranger_and_keeps_a_refusal_from_hiding() {
    let manager = fake_manager("seam", Answer::Return);
    let seam = PrivateSocketManager { client: quick_client(), endpoint: std::cell::RefCell::new(Some(ManagerEndpoint { socket: manager.socket.clone(), expected_pid: 1 })) };
    assert!(matches!(seam.call(0, &a_call()), Err(ManagerError::Refused(_))));
    assert!(manager.calls().is_empty());
    assert!(seam.endpoint.borrow().is_none(), "a refusal must make the endpoint be resolved again");
}
