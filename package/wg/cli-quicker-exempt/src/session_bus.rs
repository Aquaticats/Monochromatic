//! A client of a D-Bus daemon's socket,
//! which is how the guard asks who holds a name and hears when that changes.
//! The connection says `Hello`,
//! asks for the `NameOwnerChanged` signal of one name,
//! and can then call methods of the daemon.
//! The guard runs as the watched user,
//! because a user bus refuses a connection from root,
//! and the daemon authorizes every client by its peer credentials.

/// The message layout.
use crate::dbus_wire::{BUS_INTERFACE, BUS_NAME, BUS_PATH, Message, decode_message, encode_call, message_length};
/// The login and the kernel's word on the peer.
use crate::dbus_login::{LoginError, login};
/// Reading and writing the socket.
use std::io::{self, Read, Write};
/// The socket.
use std::os::unix::net::UnixStream;
/// The socket path.
use std::path::Path;
/// Bounded waits.
use std::time::Duration;

/// Most bytes accepted for one message.
const MAX_MESSAGE_BYTES: usize = 1 << 20;
/// The error name the bus daemon answers when no process holds a name.
const NAME_HAS_NO_OWNER: &str = "org.freedesktop.DBus.Error.NameHasNoOwner";

/// Why a bus call did not produce an answer.
#[derive(Debug)]
pub enum SessionBusError {
    /// The socket could not be reached,
    /// the login failed,
    /// or the connection broke.
    Failed(
        /// What happened.
        String,
    ),
    /// The daemon did not answer in time.
    TimedOut(
        /// What was waited for.
        String,
    ),
}

/// Wording for reports.
impl std::fmt::Display for SessionBusError {
    /// Writes the plain text of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            SessionBusError::Failed(text) | SessionBusError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}

/// A bus failure as the client's error.
fn socket_failure(error: &io::Error, what: &str) -> SessionBusError {
    return match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => SessionBusError::TimedOut(format!("{what} did not answer in time")),
        _ => SessionBusError::Failed(format!("{what} failed: {error}")),
    };
}

/// The login's failure as the client's error.
fn login_failure(error: LoginError) -> SessionBusError {
    return match error {
        LoginError::TimedOut(text) => SessionBusError::TimedOut(text),
        LoginError::Failed(text) => SessionBusError::Failed(text),
    };
}

/// A logged-in connection to a bus daemon.
pub struct BusConnection {
    /// The socket.
    stream: UnixStream,
    /// How long one call may take.
    timeout: Duration,
    /// The serial of the next call.
    next_serial: u32,
    /// The unique name the daemon gave this connection.
    pub unique_name: String,
    /// Bytes of a message that arrived with an earlier one,
    /// kept so a reply is not thrown away with the signal before it.
    buffer: Vec<u8>,
}

/// Construction and use.
impl BusConnection {
    /// Connects to the bus socket,
    /// logs in,
    /// says `Hello`,
    /// and asks for the signals `match_rule` names.
    pub fn connect(socket: &Path, timeout: Duration, match_rule: &str) -> Result<BusConnection, SessionBusError> {
        let mut stream = match UnixStream::connect(socket) {
            Ok(stream) => stream,
            Err(error) => return Err(SessionBusError::Failed(format!("the bus socket {} could not be reached: {error}", socket.display()))),
        };
        let _read_timeout = stream.set_read_timeout(Some(timeout));
        let _write_timeout = stream.set_write_timeout(Some(timeout));
        login(&mut stream).map_err(login_failure)?;
        let mut connection = BusConnection { stream, timeout, next_serial: 1, unique_name: String::new(), buffer: Vec::new() };
        let hello = connection.call_bus("Hello", Vec::new())?;
        connection.unique_name = hello.first_text().unwrap_or_default();
        let _added = connection.call_bus("AddMatch", vec![crate::dbus_wire::Argument::Text(match_rule.to_string())])?;
        return Ok(connection);
    }

    /// The serial of the next call.
    fn serial(&mut self) -> u32 {
        let serial = self.next_serial;
        self.next_serial = self.next_serial.wrapping_add(1).max(1);
        return serial;
    }

    /// Reads one message,
    /// or nothing when the timeout passed with nothing to read.
    fn read_message(&mut self) -> Result<Option<Message>, SessionBusError> {
        loop {
            if let Some(length) = message_length(&self.buffer)
                && self.buffer.len() >= length
            {
                let message = match decode_message(self.buffer.get(..length).unwrap_or_default()) {
                    Ok(message) => message,
                    Err(error) => return Err(SessionBusError::Failed(format!("the bus sent a message this client does not read: {error:?}"))),
                };
                self.buffer.drain(..length);
                return Ok(Some(message));
            }
            if self.buffer.len() >= MAX_MESSAGE_BYTES {
                return Err(SessionBusError::Failed(format!("the bus sent more than {MAX_MESSAGE_BYTES} bytes")));
            }
            let mut chunk = [0u8; 8192];
            match self.stream.read(&mut chunk) {
                Ok(0) => return Err(SessionBusError::Failed("the bus closed the socket".to_string())),
                Ok(count) => self.buffer.extend_from_slice(chunk.get(..count).unwrap_or_default()),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock || error.kind() == io::ErrorKind::TimedOut => return Ok(None),
                Err(error) => return Err(socket_failure(&error, "the bus socket")),
            }
        }
    }

    /// Sends one call and waits for its own reply,
    /// skipping the signals and other messages that arrive meanwhile.
    pub fn call(&mut self, member: &str, arguments: Vec<crate::dbus_wire::Argument>) -> Result<Message, SessionBusError> {
        let serial = self.serial();
        let bytes = encode_call(serial, BUS_PATH, BUS_INTERFACE, member, Some(BUS_NAME), &arguments);
        if let Err(error) = self.stream.write_all(&bytes) {
            return Err(socket_failure(&error, "the call to the bus"));
        }
        if let Err(error) = self.stream.flush() {
            return Err(socket_failure(&error, "the call to the bus"));
        }
        loop {
            let Some(message) = self.read_message()? else {
                return Err(SessionBusError::TimedOut(format!("the bus did not answer {member} within {:?}", self.timeout)));
            };
            if message.reply_serial != Some(serial) {
                continue;
            }
            return match message.kind {
                crate::dbus_wire::ERROR => {
                    let name = message.error_name.clone().unwrap_or_default();
                    let text = message.first_text().unwrap_or_default();
                    Err(SessionBusError::Failed(format!("the bus answered {name}: {text}")))
                }
                _ => Ok(message),
            };
        }
    }

    /// Sends one call of the bus daemon's own interface.
    fn call_bus(&mut self, member: &str, arguments: Vec<crate::dbus_wire::Argument>) -> Result<Message, SessionBusError> {
        return self.call(member, arguments);
    }

    /// The unique name of the process that holds `name`,
    /// or nothing when no process holds it.
    pub fn call_owner(&mut self, name: &str) -> Result<Option<String>, SessionBusError> {
        return match self.call("GetNameOwner", vec![crate::dbus_wire::Argument::Text(name.to_string())]) {
            Ok(reply) => Ok(reply.first_text()),
            Err(SessionBusError::Failed(text)) if text.contains(NAME_HAS_NO_OWNER) || text.contains("does not have an owner") => Ok(None),
            Err(error) => Err(error),
        };
    }

    /// The pid of the process that holds the unique name `unique`.
    pub fn call_owner_pid(&mut self, unique: &str) -> Result<i32, SessionBusError> {
        let reply = self.call("GetConnectionUnixProcessID", vec![crate::dbus_wire::Argument::Text(unique.to_string())])?;
        return match reply.first_number() {
            Some(pid) => Ok(i32::from_ne_bytes(pid.to_ne_bytes())),
            None => Err(SessionBusError::Failed("the bus gave no pid for the connection".to_string())),
        };
    }

    /// Gives the calls that follow `timeout`.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
        let _read = self.stream.set_read_timeout(Some(timeout));
        let _write = self.stream.set_write_timeout(Some(timeout));
    }

    /// Waits for the next message the daemon sends,
    /// which the guard uses as the event that the name's owner changed,
    /// or nothing when the wait passed with nothing to read.
    pub fn next_signal(&mut self) -> Result<Option<Message>, SessionBusError> {
        loop {
            let Some(message) = self.read_message()? else { return Ok(None) };
            if message.kind == crate::dbus_wire::SIGNAL {
                return Ok(Some(message));
            }
        }
    }
}
