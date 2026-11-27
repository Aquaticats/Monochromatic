//! One call to a user's manager over that manager's own private socket:
//! the login,
//! the peer check that refuses a socket answered by anything else,
//! the reply read to the call's one deadline,
//! and the error a call that produced no reply returns.

/// Where the user's manager answers,
/// and which process must be the one behind its socket.
use super::endpoint::ManagerEndpoint;
/// The login every D-Bus socket this program opens takes.
use crate::dbus_login::{
    LoginError,
    login,
    peer_pid,
};
/// The message layout.
use crate::dbus_wire::{
    MANAGER_PATH,
    Message,
    MethodCall,
    decode_message,
    encode_call,
    message_length,
};
/// Reading a reply and writing a call.
use std::io::{
    self,
    Read,
    Write,
};
/// Unix stream sockets.
use std::os::unix::net::UnixStream;
/// Bounded waits.
use std::time::{
    Duration,
    Instant,
};

/// How long one call to the manager may take,
/// which also bounds the connect and the login.
pub const CALL_TIMEOUT: Duration = Duration::from_secs(3);
/// Most bytes accepted as a reply.
const MAX_REPLY_BYTES: usize = 1 << 20;

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
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            ManagerError::Failed(text)
            | ManagerError::Refused(text)
            | ManagerError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}

/// Logs in on a private socket as the running process's user,
/// through the login every D-Bus socket takes.
fn authenticate(stream: &mut UnixStream) -> Result<(), ManagerError> {
    return match login(stream) {
        Ok(()) => Ok(()),
        Err(LoginError::TimedOut(text)) => Err(ManagerError::TimedOut(format!(
            "the manager's socket {text}"
        ))),
        Err(LoginError::Failed(text)) => {
            Err(ManagerError::Failed(format!("the manager's socket {text}")))
        }
    };
}

/// Classifies a socket failure as a timeout or a broken connection.
fn socket_failure(
    error: &io::Error,
    what: &str,
) -> ManagerError {
    return match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => ManagerError::TimedOut(format!(
            "{what} did not answer within the time the call allows"
        )),
        _ => ManagerError::Failed(format!("{what} failed: {error}")),
    };
}

/// Reads messages until the reply to `serial` arrives,
/// skipping anything else the manager sends.
///
/// The call has one deadline,
/// and nothing the manager sends extends it:
/// a socket read timeout alone restarts with every read,
/// so a manager that emits an unrelated message more often than the timeout
/// kept an unanswered call blocked for ever.
fn read_reply(
    stream: &mut UnixStream,
    serial: u32,
    deadline: Instant,
) -> Result<Message, ManagerError> {
    let mut buffer: Vec<u8> = Vec::new();
    loop {
        if let Some(length) = message_length(&buffer)
            && buffer.len() >= length
        {
            let message = match decode_message(buffer.get(..length).unwrap_or_default()) {
                Ok(message) => message,
                Err(error) => {
                    return Err(ManagerError::Failed(format!(
                        "the manager's reply could not be read: {error:?}"
                    )));
                }
            };
            if message.reply_serial == Some(serial)
                && (message.kind == crate::dbus_wire::METHOD_RETURN
                    || message.kind == crate::dbus_wire::ERROR)
            {
                return Ok(message);
            }
            buffer.drain(..length);
            continue;
        }
        if buffer.len() >= MAX_REPLY_BYTES {
            return Err(ManagerError::Failed(format!(
                "the manager's reply passed {MAX_REPLY_BYTES} bytes and was dropped"
            )));
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(ManagerError::TimedOut(
                "the manager's socket did not answer within the time the call allows".to_string(),
            ));
        }
        // Each read waits no longer than what is left of the call's own deadline.
        let _read_timeout = stream.set_read_timeout(Some(left));
        let mut chunk = [0u8; 8192];
        match stream.read(&mut chunk) {
            Ok(0) => {
                return Err(ManagerError::Failed(
                    "the manager closed the socket before it answered".to_string(),
                ));
            }
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
        return ManagerClient {
            timeout: CALL_TIMEOUT,
            next_serial: std::cell::Cell::new(1),
        };
    }

    /// Builds a client that gives up after `timeout`,
    /// which a control uses so a silent manager ends the call quickly.
    #[cfg(test)]
    pub fn with_timeout(timeout: Duration) -> ManagerClient {
        return ManagerClient {
            timeout,
            next_serial: std::cell::Cell::new(1),
        };
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
    pub fn call(
        &self,
        endpoint: &ManagerEndpoint,
        call: &MethodCall,
    ) -> Result<Message, ManagerError> {
        // The one deadline of this call, from before the connect, which no answer can extend.
        let deadline = Instant::now() + self.timeout;
        let mut stream = match UnixStream::connect(&endpoint.socket) {
            Ok(stream) => stream,
            Err(error) => {
                return Err(ManagerError::Failed(format!(
                    "the manager's socket {} could not be reached: {error}",
                    endpoint.socket.display()
                )));
            }
        };
        let _read_timeout = stream.set_read_timeout(Some(self.timeout));
        let _write_timeout = stream.set_write_timeout(Some(self.timeout));
        let peer = match peer_pid(&stream) {
            Ok(peer) => peer,
            Err(error) => {
                return Err(ManagerError::Failed(format!(
                    "the process behind the manager's socket could not be read: {error}"
                )));
            }
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
        let bytes = encode_call(
            serial,
            MANAGER_PATH,
            call.interface,
            call.member,
            None,
            &call.arguments,
        );
        if let Err(error) = stream.write_all(&bytes) {
            return Err(socket_failure(&error, "the call to the manager"));
        }
        if let Err(error) = stream.flush() {
            return Err(socket_failure(&error, "the call to the manager"));
        }
        let reply = read_reply(&mut stream, serial, deadline)?;
        if reply.kind == crate::dbus_wire::ERROR {
            let name = match &reply.error_name {
                Some(name) => name.clone(),
                None => "an unnamed error".to_string(),
            };
            let text = reply.first_text().unwrap_or_default();
            return Err(ManagerError::Failed(format!(
                "the manager answered {name}: {text}"
            )));
        }
        return Ok(reply);
    }
}
