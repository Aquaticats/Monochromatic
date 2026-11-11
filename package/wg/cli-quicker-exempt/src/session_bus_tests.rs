//! Verifies the session bus client the guard uses:
//! the login and `Hello`,
//! the match rule it asks for,
//! the owner of a name,
//! the pid behind a unique name,
//! a name that nobody holds,
//! and the signal that makes the guard act at once.
//! A fake bus daemon is a real Unix socket server in the test process,
//! so the client meets an answer a server wrote.

/// The client under test.
use crate::session_bus::{BusConnection, SessionBusError};
/// The message layout the fake daemon answers with.
use crate::dbus_wire::{Argument, decode_message, encode_reply, encode_signal};
/// Reading and writing the fake daemon's socket.
use std::io::{Read, Write};
/// The fake daemon's socket.
use std::os::unix::net::{UnixListener, UnixStream};
/// The server thread's life.
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// The name the fake daemon reports as owned.
const HELD_NAME: &str = "org.freedesktop.systemd1";
/// The unique name the fake daemon gives the owner connection.
const OWNER_UNIQUE: &str = ":1.42";
/// The pid the fake daemon reports for that connection.
const OWNER_PID: u32 = 4242;

/// What the fake daemon answers to `GetNameOwner`.
#[derive(Clone, Copy)]
enum Owner {
    /// The name is held by a connection.
    Held,
    /// No connection holds it.
    Nobody,
}

/// A fake bus daemon.
struct FakeBus {
    /// Its socket.
    socket: PathBuf,
    /// What it received,
    /// one line each.
    received: Arc<std::sync::Mutex<Vec<String>>>,
    /// Whether the server keeps running.
    stop: Arc<AtomicBool>,
    /// Its thread.
    thread: Option<JoinHandle<()>>,
}

/// Reads one whole message.
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

/// Reads one login line.
fn read_line(stream: &mut UnixStream) -> String {
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

/// A directory short enough for a Unix socket path.
fn socket_directory(name: &str) -> PathBuf {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    let base = match runtime {
        Some(directory) => directory,
        None => PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| return "/nonexistent".into())).join("temp/agent"),
    };
    return base.join(format!("wg-bus-{}-{name}", std::process::id()));
}

/// Builds a fake daemon that answers `Hello`,
/// `AddMatch`,
/// `GetNameOwner`
/// and `GetConnectionUnixProcessID`.
fn fake_bus(name: &str, owner: Owner) -> FakeBus {
    let base = socket_directory(name);
    let _removed = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("make the fixture directory");
    let socket = base.join("bus");
    let listener = UnixListener::bind(&socket).expect("bind the fake bus");
    let _nonblocking = listener.set_nonblocking(true);
    let received: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let stop = Arc::new(AtomicBool::new(false));
    let recorded = Arc::clone(&received);
    let stopping = Arc::clone(&stop);
    let thread = std::thread::Builder::new()
        .name("fake-bus".to_string())
        .spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _address)) => {
                        let _timeout = stream.set_read_timeout(Some(Duration::from_secs(5)));
                        let login = read_line(&mut stream);
                        if !login.starts_with("\u{0}AUTH EXTERNAL") {
                            continue;
                        }
                        let _wrote = stream.write_all(b"OK 00000000000000000000000000000000\r\n");
                        let begin = read_line(&mut stream);
                        if begin != "BEGIN\r\n" {
                            continue;
                        }
                        while let Some(bytes) = read_message(&mut stream) {
                            let Ok(message) = decode_message(&bytes) else { break };
                            let member = message.member.clone().unwrap_or_default();
                            recorded.lock().map(|mut calls| return calls.push(member.clone())).ok();
                            let serial = message.serial;
                            let reply = match member.as_str() {
                                "Hello" => encode_reply(serial, None),
                                "AddMatch" => encode_reply(serial, None),
                                "GetNameOwner" => match owner {
                                    Owner::Held => crate::dbus_wire::encode_text_return(serial, OWNER_UNIQUE),
                                    Owner::Nobody => crate::dbus_wire::encode_error(serial, "org.freedesktop.DBus.Error.NameHasNoOwner", "The name is not owned."),
                                },
                                "GetConnectionUnixProcessID" => crate::dbus_wire::encode_number_return(serial, OWNER_PID),
                                _ => encode_reply(serial, None),
                            };
                            let _wrote_reply = stream.write_all(&reply);
                            if member == "AddMatch" {
                                // The change the guard waits for arrives right after the rule is added and answered.
                                let signal = encode_signal(1, "NameOwnerChanged", &[Argument::Text(HELD_NAME.to_string()), Argument::Text(String::new()), Argument::Text(OWNER_UNIQUE.to_string())]);
                                let _wrote_signal = stream.write_all(&signal);
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(2)),
                    Err(_other) => break,
                }
            }
        })
        .expect("start the fake bus");
    return FakeBus { socket, received, stop, thread: Some(thread) };
}

/// Stopping the fake bus.
impl FakeBus {
    /// The members the fake daemon answered.
    fn received(&self) -> Vec<String> {
        return self.received.lock().map(|calls| return calls.clone()).unwrap_or_default();
    }
}

/// Stops the fake daemon on the way out.
impl Drop for FakeBus {
    /// Stops and joins the server.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _joined = thread.join();
        }
    }
}

/// The client logs in,
/// says `Hello`,
/// asks for the signal of the name it guards,
/// and finds the owner and its pid.
#[test]
fn the_owner_of_a_name_is_found_through_the_bus() {
    let bus = fake_bus("held", Owner::Held);
    let mut connection = BusConnection::connect(&bus.socket, Duration::from_secs(1), "type='signal',member='NameOwnerChanged'").expect("the bus answers");
    assert_eq!(connection.call_owner(HELD_NAME).expect("the bus answers"), Some(OWNER_UNIQUE.to_string()));
    assert_eq!(connection.call_owner_pid(OWNER_UNIQUE).expect("the bus answers"), i32::try_from(OWNER_PID).unwrap_or(0));
    assert!(bus.received().iter().any(|member| return member == "Hello"));
    assert!(bus.received().iter().any(|member| return member == "AddMatch"));
}

/// A name nobody holds is no owner and not a failure.
#[test]
fn a_name_without_an_owner_is_no_owner() {
    let bus = fake_bus("nobody", Owner::Nobody);
    let mut connection = BusConnection::connect(&bus.socket, Duration::from_secs(1), "type='signal'").expect("the bus answers");
    assert_eq!(connection.call_owner(HELD_NAME).expect("the bus answers"), None);
}

/// The event that makes the guard act at once is the daemon's signal,
/// which the client hands back rather than swallowing,
/// so the guard does not wait out its poll after an impostor took the name.
#[test]
fn the_change_of_an_owner_arrives_as_a_signal() {
    let bus = fake_bus("signal", Owner::Held);
    let mut connection = BusConnection::connect(&bus.socket, Duration::from_secs(1), "type='signal'").expect("the bus answers");
    let signal = connection.next_signal().expect("the bus answers").expect("the change arrives");
    assert_eq!(signal.kind, crate::dbus_wire::SIGNAL);
    assert_eq!(signal.member.as_deref(), Some("NameOwnerChanged"));
    assert_eq!(signal.first_text().as_deref(), Some(HELD_NAME));
}

/// A socket that is not there is a failure naming it,
/// never a silent no-owner.
#[test]
fn a_missing_bus_socket_is_a_failure() {
    let text = match BusConnection::connect(&socket_directory("missing").join("bus"), Duration::from_millis(200), "type='signal'") {
        Err(SessionBusError::Failed(text)) => text,
        Err(other) => panic!("not a failure: {other}"),
        Ok(_connection) => panic!("a socket that does not exist accepted a connection"),
    };
    assert!(text.contains("could not be reached"), "{text}");
}

/// A daemon that refuses the login is a failure and not a timeout,
/// so the guard says what happened.
#[test]
fn a_refused_login_is_a_failure() {
    let base = socket_directory("refused-login");
    let _removed = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("make the fixture directory");
    let socket = base.join("bus");
    let listener = UnixListener::bind(&socket).expect("bind the refusing socket");
    let refusing = std::thread::spawn(move || {
        if let Ok((mut stream, _address)) = listener.accept() {
            let _read = stream.write_all(b"REJECTED EXTERNAL\r\n");
            std::thread::sleep(Duration::from_millis(200));
        }
    });
    let error = match BusConnection::connect(&socket, Duration::from_millis(500), "type='signal'") {
        Err(SessionBusError::Failed(text)) => text,
        Err(other) => panic!("not a failure: {other}"),
        Ok(_connection) => panic!("a refusing peer accepted the login"),
    };
    refusing.join().ok();
    assert!(error.contains("refused the login"), "{error}");
}
