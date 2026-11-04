//! What:
//!  A minimal D-Bus client for the user session bus: authenticate as the current user, say `Hello`, and call methods of
//!  `org.freedesktop.systemd1.Manager` with string arguments, each bounded by one deadline for the whole exchange.
//! Why:
//!  The wrapper provisions socket activation itself, and may start no program to do it (`systemctl` would be one). The
//!  supported surface of the user manager is its D-Bus API (`org.freedesktop.systemd1(5)`), reached through the session
//!  bus; the manager's own private socket is documented as private to systemd. Only `Reload`, `ResetFailedUnit`,
//!  `StartUnit`, `StopUnit` and the state query (`GetUnit`, then a property read) are needed, all with at most two string
//!  arguments, so the writer and the reader below cover exactly the types those calls and the bus's replies use (`s`, `o`,
//!  `g`, `u`, and a variant holding `s`) and stop at anything else. The reader is total over arbitrary bytes (a
//!  fuzz target holds it to that). The comparison with a D-Bus crate is in `doc/planning/cli-git-resident-service.md`,
//!  "Provisioning".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bus = await connect(address, 5000); await bus.startUnit('cli-git.socket', 'replace'); // throws with the error name
//! ```

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

/// The manager's object path.
pub const MANAGER_PATH: &str = "/org/freedesktop/systemd1";

/// The manager's interface.
pub const MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";

/// How long the whole exchange (connect, authenticate, say hello, make the calls) may take; a stuck bus never holds a
///  commit longer.
pub const EXCHANGE_WAIT: Duration = Duration::from_secs(20);

/// The bus daemon's name, object and interface, for `Hello`.
const BUS_NAME: &str = "org.freedesktop.DBus";
const BUS_PATH: &str = "/org/freedesktop/DBus";

/// The manager's bus name.
pub const MANAGER_NAME: &str = "org.freedesktop.systemd1";

/// The most bytes of one reply this client accepts.
const MAX_REPLY_BYTES: usize = 1 << 20;

/// Message type of a method call.
const METHOD_CALL: u8 = 1;

/// Message type of a successful reply.
const METHOD_RETURN: u8 = 2;

/// Message type of an error reply.
const ERROR: u8 = 3;

/// Header field code of the number of the call a reply answers.
const FIELD_REPLY_SERIAL: u8 = 5;

/// Header field codes.
const FIELD_PATH: u8 = 1;
const FIELD_INTERFACE: u8 = 2;
const FIELD_MEMBER: u8 = 3;
const FIELD_ERROR_NAME: u8 = 4;
const FIELD_DESTINATION: u8 = 6;
const FIELD_SIGNATURE: u8 = 8;

/// Why a call did not succeed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallError {
    /// The socket could not be reached or the connection broke.
    Connection(String),
    /// The manager refused the authentication.
    Authentication(String),
    /// The bus did not answer within the time the exchange was given, which says nothing about whether the manager is there.
    TimedOut,
    /// The manager answered with an error.
    Manager {
        /// The error's name, for example `org.freedesktop.systemd1.NoSuchUnit`.
        name: String,
        /// The error's text, when it gave one.
        text: String,
    },
    /// The reply was not a message this client reads.
    Malformed(String),
}

/// Appends values in D-Bus little-endian wire form.
struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn align(&mut self, boundary: usize) {
        while self.bytes.len() % boundary != 0 {
            self.bytes.push(0);
        }
    }

    fn unsigned(&mut self, value: u32) {
        self.align(4);
        self.bytes.extend(value.to_le_bytes());
    }

    fn text(&mut self, value: &str) {
        self.unsigned(u32::try_from(value.len()).unwrap_or(u32::MAX));
        self.bytes.extend(value.as_bytes());
        self.bytes.push(0);
    }

    fn signature(&mut self, value: &str) {
        self.bytes.push(u8::try_from(value.len()).unwrap_or(u8::MAX));
        self.bytes.extend(value.as_bytes());
        self.bytes.push(0);
    }

    /// One header field: its code, then a variant holding a string-like or signature value.
    fn field(&mut self, code: u8, kind: char, value: &str) {
        self.align(8);
        self.bytes.push(code);
        self.signature(&kind.to_string());
        if kind == 'g' {
            self.signature(value);
        } else {
            self.text(value);
        }
    }
}

/// Where a method call goes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Target<'a> {
    /// The bus name of the receiver.
    pub destination: &'a str,
    /// The object path.
    pub path: &'a str,
    /// The interface.
    pub interface: &'a str,
    /// The method.
    pub member: &'a str,
}

/// `Hello` on the bus daemon, the first call of every connection.
pub const HELLO: Target<'static> = Target { destination: BUS_NAME, path: BUS_PATH, interface: BUS_NAME, member: "Hello" };

/// `Reload` on the user manager.
pub const RELOAD: Target<'static> = Target { destination: MANAGER_NAME, path: MANAGER_PATH, interface: MANAGER_INTERFACE, member: "Reload" };

/// `StartUnit` on the user manager.
pub const START_UNIT: Target<'static> = Target { destination: MANAGER_NAME, path: MANAGER_PATH, interface: MANAGER_INTERFACE, member: "StartUnit" };

/// `StopUnit` on the user manager.
pub const STOP_UNIT: Target<'static> = Target { destination: MANAGER_NAME, path: MANAGER_PATH, interface: MANAGER_INTERFACE, member: "StopUnit" };

/// `ResetFailedUnit` on the user manager.
pub const RESET_FAILED_UNIT: Target<'static> =
    Target { destination: MANAGER_NAME, path: MANAGER_PATH, interface: MANAGER_INTERFACE, member: "ResetFailedUnit" };

/// `GetUnit` on the user manager: the object path of a loaded unit.
pub const GET_UNIT: Target<'static> = Target { destination: MANAGER_NAME, path: MANAGER_PATH, interface: MANAGER_INTERFACE, member: "GetUnit" };

/// The interface whose `Get` reads one property of an object.
const PROPERTIES_INTERFACE: &str = "org.freedesktop.DBus.Properties";

/// The interface that holds a unit's `ActiveState`.
const UNIT_INTERFACE: &str = "org.freedesktop.systemd1.Unit";

/// The error name of a unit the manager does not hold (never loaded, or unloaded after it stopped).
pub const NO_SUCH_UNIT: &str = "org.freedesktop.systemd1.NoSuchUnit";

/// The error name of an object that is gone by the time a property is read.
const UNKNOWN_OBJECT: &str = "org.freedesktop.DBus.Error.UnknownObject";

/// What:
///  The bytes of a method call with string arguments.
/// Why:
///   Kept apart from the socket so a test can read them back and compare them with the specification's layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// encodeCall(serial, START_UNIT, ['cli-git.socket', 'replace'])
/// ```
pub fn encode_call(serial: u32, target: &Target<'_>, arguments: &[&str]) -> Vec<u8> {
    let mut body: Writer = Writer { bytes: Vec::new() };
    for argument in arguments {
        body.text(argument);
    }
    let mut fields: Writer = Writer { bytes: vec![0; 16] };
    fields.field(FIELD_PATH, 'o', target.path);
    fields.field(FIELD_DESTINATION, 's', target.destination);
    fields.field(FIELD_INTERFACE, 's', target.interface);
    fields.field(FIELD_MEMBER, 's', target.member);
    if !arguments.is_empty() {
        fields.field(FIELD_SIGNATURE, 'g', &"s".repeat(arguments.len()));
    }
    // The first sixteen bytes are the fixed header and the array length, filled in below.
    let array_length: u32 = u32::try_from(fields.bytes.len() - 16).unwrap_or(u32::MAX);
    fields.align(8);
    let mut message: Vec<u8> = fields.bytes;
    message[0] = b'l';
    message[1] = METHOD_CALL;
    message[2] = 0;
    message[3] = 1;
    message[4..8].copy_from_slice(&u32::try_from(body.bytes.len()).unwrap_or(u32::MAX).to_le_bytes());
    message[8..12].copy_from_slice(&serial.to_le_bytes());
    message[12..16].copy_from_slice(&array_length.to_le_bytes());
    message.extend(body.bytes);
    return message;
}

/// A reply: its type, the error name its header carries, and the first string of its body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reply {
    /// Message type: 2 for a return, 3 for an error, 4 for a signal.
    pub kind: u8,
    /// The number of the call this message answers.
    pub reply_serial: Option<u32>,
    /// The error name of an error reply.
    pub error_name: Option<String>,
    /// The first string argument of the body, which an error reply uses for its text.
    pub first_text: Option<String>,
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    return bytes.get(at..at.checked_add(4)?).map(|slice| { return u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]); });
}

/// The string at `at` and the offset after its end byte, or nothing when it is cut short or the byte after the text is not
///  the end byte a D-Bus string carries. Callers pass only the bytes of the region the string must lie in, so an end byte
///  beyond that region is not found.
fn read_text(bytes: &[u8], at: usize) -> Option<(String, usize)> {
    let length: usize = usize::try_from(read_u32(bytes, at)?).ok()?;
    let start: usize = at.checked_add(4)?;
    let end: usize = start.checked_add(length)?;
    let text: &[u8] = bytes.get(start..end)?;
    if *bytes.get(end)? != 0 {
        return None;
    }
    return Some((String::from_utf8_lossy(text).into_owned(), end.saturating_add(1)));
}

/// The string a variant at `at` holds, or nothing when it holds another type, is cut short, or has a signature that does
///  not end with its end byte.
fn read_variant_text(bytes: &[u8], at: usize) -> Option<String> {
    let length: usize = usize::from(*bytes.get(at)?);
    let inner_start: usize = at.checked_add(1)?;
    let inner_end: usize = inner_start.checked_add(length)?;
    let inner: &[u8] = bytes.get(inner_start..inner_end)?;
    if inner != b"s" || *bytes.get(inner_end)? != 0 {
        return None;
    }
    // The signature's end byte is checked; the string follows on a four-byte boundary.
    return read_text(bytes, aligned(inner_end.checked_add(1)?, 4)).map(|(text, _)| { return text; });
}

fn aligned(offset: usize, boundary: usize) -> usize {
    return offset.checked_add(boundary - 1).map_or(usize::MAX, |sum| { return sum / boundary * boundary; });
}

/// What:
///  Reads a complete reply message.
/// Why:
///   Only the type, the error name and the first string of the body matter; a header field of a kind this reader does not
///  know ends the walk of the header, which is safe because the fields it needs come first in what the manager sends or
///  are looked for before any unknown kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decodeReply(bytes)
/// ```
pub fn decode_reply(bytes: &[u8]) -> Result<Reply, CallError> {
    let malformed = |what: &str| { return CallError::Malformed(what.to_string()); };
    if bytes.len() < 16 || bytes[0] != b'l' {
        return Err(malformed("the reply is not a little-endian message"));
    }
    let kind: u8 = bytes[1];
    let body_length: usize = usize::try_from(read_u32(bytes, 4).ok_or_else(|| { return malformed("no body length"); })?).unwrap_or(usize::MAX);
    let fields_length: usize = usize::try_from(read_u32(bytes, 12).ok_or_else(|| { return malformed("no header length"); })?).unwrap_or(usize::MAX);
    let fields_end: usize = 16usize.saturating_add(fields_length);
    let body_start: usize = aligned(fields_end, 8);
    if bytes.len() < body_start.saturating_add(body_length) || fields_end > bytes.len() {
        return Err(malformed("the reply is shorter than it says"));
    }
    let mut error_name: Option<String> = None;
    let mut reply_serial: Option<u32> = None;
    let mut signature: String = String::new();
    // A header field lies within the declared header and a body value within the declared body, whatever follows them.
    let header: &[u8] = bytes.get(..fields_end).unwrap_or_default();
    let body: &[u8] = bytes.get(..body_start.saturating_add(body_length)).unwrap_or_default();
    let mut at: usize = 16;
    while at < fields_end {
        at = aligned(at, 8);
        let (Some(&code), Some(&signature_length)) = (header.get(at), header.get(at.saturating_add(1))) else { break };
        let kind_at: usize = at.saturating_add(2);
        let Some(&value_kind) = header.get(kind_at) else { break };
        let value_at: usize = kind_at.saturating_add(usize::from(signature_length)).saturating_add(1);
        match (value_kind, code) {
            (b's' | b'o', _) => {
                let Some((text, next)) = read_text(header, aligned(value_at, 4)) else { break };
                if code == FIELD_ERROR_NAME {
                    error_name = Some(text);
                }
                at = next;
            }
            (b'g', _) => {
                let length: usize = usize::from(*header.get(value_at).unwrap_or(&0));
                if code == FIELD_SIGNATURE {
                    let start: usize = value_at.saturating_add(1);
                    // A signature whose end byte is missing is not read, so the body is not interpreted by it.
                    if header.get(start.saturating_add(length)) == Some(&0) {
                        signature = String::from_utf8_lossy(header.get(start..start.saturating_add(length)).unwrap_or_default()).into_owned();
                    }
                }
                at = value_at.saturating_add(length).saturating_add(2);
            }
            (b'u', _) => {
                let number_at: usize = aligned(value_at, 4);
                if code == FIELD_REPLY_SERIAL {
                    reply_serial = read_u32(header, number_at);
                }
                at = number_at.saturating_add(4);
            }
            _ => break,
        }
    }
    // A string and an object path share one wire form; a variant holding a string is its own signature, then that form.
    let first_text: Option<String> = if signature.starts_with('s') || signature.starts_with('o') {
        read_text(body, aligned(body_start, 4)).map(|(text, _)| { return text; })
    } else if signature == "v" {
        read_variant_text(body, body_start)
    } else {
        None
    };
    return Ok(Reply { kind, reply_serial, error_name, first_text });
}

/// The user id as the ASCII digits the `EXTERNAL` mechanism sends, written in hexadecimal.
fn external_identity() -> String {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    let user: u32 = unsafe { libc::geteuid() };
    return user.to_string().bytes().map(|byte| { return format!("{byte:02x}"); }).collect();
}

/// The time left of the exchange, or an error when it ran out.
fn remaining(deadline: Instant) -> Result<Duration, CallError> {
    let left: Duration = deadline.saturating_duration_since(Instant::now());
    return if left.is_zero() { Err(CallError::TimedOut) } else { Ok(left) };
}

/// A socket failure as the call's error: running out of the read or write time is a timeout, anything else a broken connection.
fn socket_failure(error: &std::io::Error) -> CallError {
    return match error.kind() {
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => CallError::TimedOut,
        _ => CallError::Connection(error.to_string()),
    };
}

fn read_line(stream: &mut UnixStream, deadline: Instant) -> Result<String, CallError> {
    let mut line: Vec<u8> = Vec::new();
    let mut one: [u8; 1] = [0];
    while line.len() < 4096 {
        stream.set_read_timeout(Some(remaining(deadline)?)).map_err(|error| { return CallError::Connection(error.to_string()); })?;
        stream.read_exact(&mut one).map_err(|error| {
            return match socket_failure(&error) {
                CallError::TimedOut => CallError::TimedOut,
                _ => CallError::Authentication(error.to_string()),
            };
        })?;
        if one[0] == b'\n' {
            return Ok(String::from_utf8_lossy(line.as_slice()).trim_end_matches('\r').to_string());
        }
        line.push(one[0]);
    }
    return Err(CallError::Authentication(String::from("the answer is too long")));
}

/// Reads exactly one message from the stream.
fn read_message(stream: &mut UnixStream, deadline: Instant) -> Result<Vec<u8>, CallError> {
    let mut header: Vec<u8> = vec![0; 16];
    stream.set_read_timeout(Some(remaining(deadline)?)).map_err(|error| { return CallError::Connection(error.to_string()); })?;
    stream.read_exact(header.as_mut_slice()).map_err(|error| { return socket_failure(&error); })?;
    let body_length: usize = usize::try_from(read_u32(header.as_slice(), 4).unwrap_or(0)).unwrap_or(usize::MAX);
    let fields_length: usize = usize::try_from(read_u32(header.as_slice(), 12).unwrap_or(0)).unwrap_or(usize::MAX);
    let rest: usize = aligned(16usize.saturating_add(fields_length), 8).saturating_add(body_length).saturating_sub(16);
    if rest > MAX_REPLY_BYTES {
        return Err(CallError::Malformed(String::from("the reply is larger than this client reads")));
    }
    let mut remainder: Vec<u8> = vec![0; rest];
    stream.set_read_timeout(Some(remaining(deadline)?)).map_err(|error| { return CallError::Connection(error.to_string()); })?;
    stream.read_exact(remainder.as_mut_slice()).map_err(|error| { return socket_failure(&error); })?;
    header.extend(remainder);
    return Ok(header);
}

/// The path of the session bus socket: the `unix:path=` entry of the address variable, else `$XDG_RUNTIME_DIR/bus`.
pub fn session_bus_path(address: Option<&str>, runtime_dir: Option<&Path>) -> Option<std::path::PathBuf> {
    let from_address: Option<std::path::PathBuf> = address.and_then(|list| {
        return list.split(';').find_map(|entry| {
            let rest: &str = entry.strip_prefix("unix:")?;
            let path: &str = rest.split(',').find_map(|option| { return option.strip_prefix("path="); })?;
            // Escapes (`%xx`) in a path are not read; a path that uses them falls back to the default location.
            return if path.contains('%') || path.is_empty() { None } else { Some(std::path::PathBuf::from(path)) };
        });
    });
    return from_address.or_else(|| { return runtime_dir.map(|directory| { return directory.join("bus"); }); });
}

/// An authenticated connection to the session bus, past `Hello`.
pub struct Bus {
    stream: UnixStream,
    deadline: Instant,
    serial: u32,
}

/// Methods over a connection.
impl Bus {
    /// What:
    ///  Connects to the bus socket, authenticates as the current user and says `Hello`, all within `wait`.
    /// Why:
    ///   The deadline of the whole exchange starts here, so every later call shares what is left of it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const bus = await Bus.connect(path, 20000)
    /// ```
    pub fn connect(socket: &Path, wait: Duration) -> Result<Bus, CallError> {
        let deadline: Instant = Instant::now() + wait;
        let mut stream: UnixStream = UnixStream::connect(socket).map_err(|error| { return CallError::Connection(error.to_string()); })?;
        stream.set_write_timeout(Some(wait)).map_err(|error| { return CallError::Connection(error.to_string()); })?;
        let mut hello: Vec<u8> = vec![0];
        hello.extend(format!("AUTH EXTERNAL {}\r\n", external_identity()).as_bytes());
        stream.write_all(hello.as_slice()).map_err(|error| { return CallError::Authentication(error.to_string()); })?;
        let answer: String = read_line(&mut stream, deadline)?;
        if !answer.starts_with("OK ") {
            return Err(CallError::Authentication(answer));
        }
        stream.write_all(b"BEGIN\r\n").map_err(|error| { return CallError::Authentication(error.to_string()); })?;
        let mut bus: Bus = Bus { stream, deadline, serial: 0 };
        bus.call(&HELLO, &[])?;
        return Ok(bus);
    }

    /// What:
    ///  Gives the calls that follow `wait` from now, replacing what was left of the time the connection was given.
    /// Why:
    ///   The time of the connection bounds a stuck bus, but a caller that spends time between calls (waiting for a service to
    ///  stop) must bound each phase by its own budget: a slow phase must not make the next one fail, and a query must not run
    ///  past the phase it belongs to.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bus.within(10000) // the next calls may take ten seconds in all
    /// ```
    pub fn within(&mut self, wait: Duration) {
        self.deadline = Instant::now() + wait;
    }

    /// What:
    ///  Makes one method call and waits for the reply that answers it, skipping the signals the bus sends meanwhile.
    /// Why:
    ///   A return is the receiver's confirmation (a reload returns after the units are read); an error reply comes back
    ///  with its name and text so the caller can say what to do.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await bus.call(RELOAD, [])
    /// ```
    pub fn call(&mut self, target: &Target<'_>, arguments: &[&str]) -> Result<Reply, CallError> {
        self.serial += 1;
        let serial: u32 = self.serial;
        self.stream.write_all(encode_call(serial, target, arguments).as_slice()).map_err(|error| { return socket_failure(&error); })?;
        loop {
            let reply: Reply = decode_reply(read_message(&mut self.stream, self.deadline)?.as_slice())?;
            if reply.reply_serial != Some(serial) || !matches!(reply.kind, METHOD_RETURN | ERROR) {
                continue;
            }
            return match (reply.kind, reply.error_name.clone()) {
                (METHOD_RETURN, _) => Ok(reply),
                (_, Some(name)) => Err(CallError::Manager { name, text: reply.first_text.unwrap_or_default() }),
                _ => Err(CallError::Malformed(String::from("an error reply without a name"))),
            };
        }
    }

    /// Reloads the user manager's unit files.
    pub fn reload(&mut self) -> Result<(), CallError> {
        return self.call(&RELOAD, &[]).map(drop);
    }

    /// Clears a unit's failed state and its start-rate counter, so a start is not refused as "repeated too quickly".
    pub fn reset_failed(&mut self, name: &str) -> Result<(), CallError> {
        return self.call(&RESET_FAILED_UNIT, &[name]).map(drop);
    }

    /// Stops a unit, replacing a queued job for it.
    pub fn stop_unit(&mut self, name: &str) -> Result<(), CallError> {
        return self.call(&STOP_UNIT, &[name, "replace"]).map(drop);
    }

    /// What:
    ///  The unit's `ActiveState` (`active`, `activating`, `deactivating`, `inactive`, `failed`), or nothing when the manager
    ///  does not hold the unit.
    /// Why:
    ///   A stop call returns when the job is queued, so only this state says the unit and its processes are gone. `GetUnit`
    ///  answers the unit's object path and the property read answers a variant holding the state text. A unit that is not
    ///  loaded, or that unloads between the two calls, is not running.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await bus.activeState('cli-git.service') // 'deactivating' | 'inactive' | undefined
    /// ```
    pub fn active_state(&mut self, name: &str) -> Result<Option<String>, CallError> {
        let path: String = match self.call(&GET_UNIT, &[name]) {
            Ok(reply) => reply.first_text.ok_or_else(|| { return CallError::Malformed(String::from("GetUnit answered without a path")); })?,
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT => return Ok(None),
            Err(error) => return Err(error),
        };
        let property: Target<'_> = Target { destination: MANAGER_NAME, path: path.as_str(), interface: PROPERTIES_INTERFACE, member: "Get" };
        return match self.call(&property, &[UNIT_INTERFACE, "ActiveState"]) {
            Ok(reply) => reply
                .first_text
                .map(Some)
                .ok_or_else(|| { return CallError::Malformed(String::from("the state property answered without text")); }),
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT || error == UNKNOWN_OBJECT => Ok(None),
            Err(error) => Err(error),
        };
    }

    /// Starts a unit, replacing a queued job for it.
    pub fn start_unit(&mut self, name: &str) -> Result<(), CallError> {
        return self.call(&START_UNIT, &[name, "replace"]).map(drop);
    }
}

/// Client controls stay out of the release executable.
#[cfg(test)]
#[path = "systemd_dbus_tests.rs"]
mod tests;
