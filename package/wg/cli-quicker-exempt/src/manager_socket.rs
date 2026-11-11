//! The channel to a user's service manager,
//! and the check that the manager is the process on the other end of it.
//! The watcher calls the manager over that manager's own private socket,
//! `/run/user/<uid>/systemd/private`,
//! which the manager itself created and which only the manager answers:
//! no name lookup happens,
//! so no process can take `org.freedesktop.systemd1` and be called instead.
//! On 2026-11-11 a nested `systemd --user` an agent's probe started took that name on the owner's session bus for hours,
//! and every watcher call went to it,
//! which is why the watcher never asks a bus who the manager is.
//! Before the first byte of a call is written,
//! `SO_PEERCRED` gives the pid behind the socket,
//! and it must be the main process of `user@<uid>.service`
//! as the system manager reports it.
//! A mismatch refuses the call loudly,
//! naming both pids,
//! and nothing is moved.
//! The reason the peer check is enough:
//! the pid comes from the kernel for the connected socket,
//! so a socket the user planted at that path answers with its own pid and is refused.

/// The search path and the time limit of the system manager query.
use crate::bounded_command::run_bounded;
/// The message layout.
use crate::dbus_wire::{MANAGER_PATH, Message, MethodCall, decode_message, encode_call, message_length};
/// Reading a reply and writing a call.
use std::io::{self, Read, Write};
/// Argument bytes that need not be text.
use std::ffi::OsString;
/// Unix stream sockets.
use std::os::unix::net::UnixStream;
/// Sockets and paths.
use std::path::PathBuf;
/// Bounded waits.
use std::time::Duration;

/// How long the system manager may take to say which process runs a user's manager.
const MAIN_PID_TIMEOUT: Duration = Duration::from_secs(5);
/// How long one call to the manager may take,
/// which also bounds the connect and the login.
pub const CALL_TIMEOUT: Duration = Duration::from_secs(3);
/// Most bytes accepted as a reply.
const MAX_REPLY_BYTES: usize = 1 << 20;

/// Where a user's manager answers,
/// and which process must be behind that socket.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagerEndpoint {
    /// The manager's private socket.
    pub socket: PathBuf,
    /// The main process of the user's manager unit.
    pub expected_pid: i32,
}

/// Why a call to the user's manager did not produce a reply.
#[derive(Debug)]
pub enum ManagerError {
    /// The socket could not be reached,
    /// the login was refused,
    /// or the connection broke.
    Failed(
        /// What happened.
        String,
    ),
    /// The socket is answered by another process than the user's manager.
    /// Nothing was sent.
    Refused(
        /// The refusal,
        /// naming both pids.
        String,
    ),
    /// The manager did not answer in time.
    TimedOut(
        /// What was waited for.
        String,
    ),
}

/// Wording for reports.
impl std::fmt::Display for ManagerError {
    /// Writes the plain text of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            ManagerError::Failed(text) | ManagerError::Refused(text) | ManagerError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}

/// The private socket of the user manager of `uid`.
pub fn manager_socket_path(uid: u32) -> PathBuf {
    return PathBuf::from(format!("/run/user/{uid}/systemd/private"));
}

/// The main process of `user@<uid>.service`,
/// as the system manager reports it.
/// `systemctl` without `--user` asks the system manager over its own root-owned socket,
/// so neither a user process nor a user bus can answer for it.
pub fn manager_main_pid(uid: u32) -> Result<i32, String> {
    let arguments: Vec<OsString> = vec![
        OsString::from("show"),
        OsString::from("--property=MainPID"),
        OsString::from("--value"),
        OsString::from(format!("user@{uid}.service")),
    ];
    let output = run_bounded("systemctl", &arguments, MAIN_PID_TIMEOUT)?;
    if !output.success {
        return Err(format!("systemctl show user@{uid}.service exited with {}: {}", output.status, output.stderr.trim()));
    }
    let text = output.stdout.trim();
    let pid: i32 = match text.parse() {
        Ok(pid) => pid,
        Err(_error) => return Err(format!("systemctl named no main process of user@{uid}.service ({text:?})")),
    };
    if pid <= 1 {
        return Err(format!("the system manager reports no running main process of user@{uid}.service ({pid})"));
    }
    return Ok(pid);
}

/// Resolving and using the channel.
impl ManagerEndpoint {
    /// Where the user's manager of `uid` answers,
    /// and which process must be there.
    pub fn resolve(uid: u32) -> Result<ManagerEndpoint, String> {
        return Ok(ManagerEndpoint { socket: manager_socket_path(uid), expected_pid: manager_main_pid(uid)? });
    }
}

/// The login and the peer credentials,
/// shared with every other D-Bus socket this program opens.
use crate::dbus_login::{LoginError, login};
/// The pid on the other end of a connected socket.
pub use crate::dbus_login::peer_pid;

/// Logs in on a private socket as the running process's user,
/// through the login every D-Bus socket takes.
fn authenticate(stream: &mut UnixStream) -> Result<(), ManagerError> {
    return match login(stream) {
        Ok(()) => Ok(()),
        Err(LoginError::TimedOut(text)) => Err(ManagerError::TimedOut(format!("the manager's socket {text}"))),
        Err(LoginError::Failed(text)) => Err(ManagerError::Failed(format!("the manager's socket {text}"))),
    };
}

/// Classifies a socket failure as a timeout or a broken connection.
fn socket_failure(error: &io::Error, what: &str) -> ManagerError {
    return match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => ManagerError::TimedOut(format!("{what} did not answer within the time the call allows")),
        _ => ManagerError::Failed(format!("{what} failed: {error}")),
    };
}

/// Reads messages until the reply to `serial` arrives,
/// skipping anything else the manager sends.
fn read_reply(stream: &mut UnixStream, serial: u32) -> Result<Message, ManagerError> {
    let mut buffer: Vec<u8> = Vec::new();
    loop {
        if let Some(length) = message_length(&buffer)
            && buffer.len() >= length
        {
            let message = match decode_message(buffer.get(..length).unwrap_or_default()) {
                Ok(message) => message,
                Err(error) => return Err(ManagerError::Failed(format!("the manager's reply could not be read: {error:?}"))),
            };
            if message.reply_serial == Some(serial) && (message.kind == crate::dbus_wire::METHOD_RETURN || message.kind == crate::dbus_wire::ERROR) {
                return Ok(message);
            }
            buffer.drain(..length);
            continue;
        }
        if buffer.len() >= MAX_REPLY_BYTES {
            return Err(ManagerError::Failed(format!("the manager's reply passed {MAX_REPLY_BYTES} bytes and was dropped")));
        }
        let mut chunk = [0u8; 8192];
        match stream.read(&mut chunk) {
            Ok(0) => return Err(ManagerError::Failed("the manager closed the socket before it answered".to_string())),
            Ok(count) => buffer.extend_from_slice(chunk.get(..count).unwrap_or_default()),
            Err(error) => return Err(socket_failure(&error, "the manager's socket")),
        }
    }
}

/// A client of one user's manager over its private socket.
pub struct ManagerClient {
    /// How long one call may take.
    pub timeout: Duration,
    /// The serial number of the next call,
    /// which tells its reply from anything else the manager sends.
    pub next_serial: std::cell::Cell<u32>,
}

/// Construction and the call.
impl ManagerClient {
    /// Builds a client with the production timeout.
    pub fn new() -> ManagerClient {
        return ManagerClient { timeout: CALL_TIMEOUT, next_serial: std::cell::Cell::new(1) };
    }

    /// Builds a client that gives up after `timeout`,
    /// which a control uses so a silent manager ends the call quickly.
    #[cfg(test)]
    pub fn with_timeout(timeout: Duration) -> ManagerClient {
        return ManagerClient { timeout, next_serial: std::cell::Cell::new(1) };
    }

    /// The serial of the next call.
    fn serial(&self) -> u32 {
        let serial = self.next_serial.get();
        self.next_serial.set(serial.wrapping_add(1).max(1));
        return serial;
    }

    /// Runs one call against the manager at `endpoint`,
    /// after the peer behind the socket is established as the manager itself.
    /// A refusal or a timeout is its own error,
    /// so the policy says which happened:
    /// a refusal is reported loudly and moves nothing,
    /// and a timeout pauses the moves for a few seconds.
    pub fn call(&self, endpoint: &ManagerEndpoint, call: &MethodCall) -> Result<Message, ManagerError> {
        let mut stream = match UnixStream::connect(&endpoint.socket) {
            Ok(stream) => stream,
            Err(error) => return Err(ManagerError::Failed(format!("the manager's socket {} could not be reached: {error}", endpoint.socket.display()))),
        };
        let _read_timeout = stream.set_read_timeout(Some(self.timeout));
        let _write_timeout = stream.set_write_timeout(Some(self.timeout));
        let peer = match peer_pid(&stream) {
            Ok(peer) => peer,
            Err(error) => return Err(ManagerError::Failed(format!("the process behind the manager's socket could not be read: {error}"))),
        };
        if peer != endpoint.expected_pid {
            return Err(ManagerError::Refused(format!(
                "the socket {} is answered by pid {peer}, but the user's service manager is pid {}; \
nothing was sent, and a process that is not that manager is never listened to",
                endpoint.socket.display(),
                endpoint.expected_pid
            )));
        }
        authenticate(&mut stream)?;
        let serial = self.serial();
        let bytes = encode_call(serial, MANAGER_PATH, call.interface, call.member, None, &call.arguments);
        if let Err(error) = stream.write_all(&bytes) {
            return Err(socket_failure(&error, "the call to the manager"));
        }
        if let Err(error) = stream.flush() {
            return Err(socket_failure(&error, "the call to the manager"));
        }
        let reply = read_reply(&mut stream, serial)?;
        if reply.kind == crate::dbus_wire::ERROR {
            let name = match &reply.error_name {
                Some(name) => name.clone(),
                None => "an unnamed error".to_string(),
            };
            let text = reply.first_text().unwrap_or_default();
            return Err(ManagerError::Failed(format!("the manager answered {name}: {text}")));
        }
        return Ok(reply);
    }
}

/// The seam that runs one call against a user's manager,
/// so the policy is testable without a manager and the production path stays in one place.
pub trait ManagerCaller {
    /// Runs one call for the watched user `uid`.
    fn call(&self, uid: u32, call: &MethodCall) -> Result<Message, ManagerError>;
}

/// The production seam:
/// the manager's private socket,
/// with the endpoint resolved once and again after a refusal or a lost connection.
pub struct PrivateSocketManager {
    /// The client of that socket.
    pub client: ManagerClient,
    /// The endpoint of the watched user's manager,
    /// kept between calls.
    pub endpoint: std::cell::RefCell<Option<ManagerEndpoint>>,
}

/// Construction and the call.
impl PrivateSocketManager {
    /// Builds the production seam.
    pub fn new() -> PrivateSocketManager {
        return PrivateSocketManager { client: ManagerClient::new(), endpoint: std::cell::RefCell::new(None) };
    }

    /// The endpoint of the user's manager,
    /// resolved on the first call and kept until a refusal shows the resolved pid is no longer the manager's.
    fn endpoint(&self, uid: u32) -> Result<ManagerEndpoint, ManagerError> {
        let mut slot = self.endpoint.borrow_mut();
        if let Some(endpoint) = slot.as_ref() {
            return Ok(endpoint.clone());
        }
        let endpoint = ManagerEndpoint::resolve(uid).map_err(ManagerError::Failed)?;
        *slot = Some(endpoint.clone());
        return Ok(endpoint);
    }

    /// Forgets the endpoint,
    /// so the next call asks the system manager which process runs the user's manager again.
    fn forget_endpoint(&self) {
        self.endpoint.borrow_mut().take();
    }
}

/// The production seam calls over the manager's private socket.
impl ManagerCaller for PrivateSocketManager {
    /// Runs one call,
    /// with the peer check that refuses a socket answered by anything but the user's manager.
    fn call(&self, uid: u32, call: &MethodCall) -> Result<Message, ManagerError> {
        let endpoint = self.endpoint(uid)?;
        let outcome = self.client.call(&endpoint, call);
        if matches!(outcome, Err(ManagerError::Refused(_))) {
            self.forget_endpoint();
        }
        return outcome;
    }
}
