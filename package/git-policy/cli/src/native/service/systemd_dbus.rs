//! What:
//!  A minimal D-Bus client for the user session bus:
//! authenticate as the current user,
//! say `Hello`,
//! and call methods of
//!  `org.freedesktop.systemd1.Manager` with string arguments,
//! each bounded by one deadline for the whole exchange.
//! Why:
//!  The wrapper provisions socket activation itself,
//! and may start no program to do it (`systemctl` would be one).
//! The
//!  supported surface of the user manager is its D-Bus API (`org.freedesktop.systemd1(5)`),
//! reached through the session
//!  bus;
//! the manager's own private socket is documented as private to systemd.
//! Only `Reload`,
//! `ResetFailedUnit`,
//!  `StartUnit`,
//! `StopUnit` and the state query (`GetUnit`,
//! then a property read) are needed,
//! all with at most two string
//!  arguments,
//! so the writer and the reader below cover exactly the types those calls and the bus's replies use (`s`,
//! `o`,
//!  `g`,
//! `u`,
//! and a variant holding `s`) and stop at anything else.
//! The reader is total over arbitrary bytes (a
//!  fuzz target holds it to that).
//! The comparison with a D-Bus crate is in `doc/planning/cli-git-resident-service.md`,
//!  "Provisioning".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bus = await connect(address, 5000); await bus.startUnit('cli-git.socket', 'replace'); // throws with the error name
//! ```

pub use crate::systemd_dbus_wire_format::{CallError, Reply, Target, decode_reply, encode_call};
use crate::systemd_dbus_wire_format::{ERROR, METHOD_RETURN, aligned, read_u32};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

/// The manager's object path.
pub const MANAGER_PATH: &str = "/org/freedesktop/systemd1";

/// The manager's interface.
pub const MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";

/// How long the whole exchange (connect,
/// authenticate,
/// say hello,
/// make the calls) may take;
/// a stuck bus never holds a
///  commit longer.
pub const EXCHANGE_WAIT: Duration = Duration::from_secs(20);

/// The bus daemon's name,
/// object and interface,
/// for `Hello`.
const BUS_NAME: &str = "org.freedesktop.DBus";
const BUS_PATH: &str = "/org/freedesktop/DBus";

/// The manager's bus name.
pub const MANAGER_NAME: &str = "org.freedesktop.systemd1";

/// The most bytes of one reply this client accepts.
const MAX_REPLY_BYTES: usize = 1 << 20;

/// `Hello` on the bus daemon,
/// the first call of every connection.
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

/// `GetUnit` on the user manager:
/// the object path of a loaded unit.
pub const GET_UNIT: Target<'static> = Target { destination: MANAGER_NAME, path: MANAGER_PATH, interface: MANAGER_INTERFACE, member: "GetUnit" };

/// The job mode that applies a stop to the named unit only,
/// leaving the units that require it running.
pub const STOP_ALONE_MODE: &str = "ignore-dependencies";

/// The interface whose `Get` reads one property of an object.
const PROPERTIES_INTERFACE: &str = "org.freedesktop.DBus.Properties";

/// The interface that holds a unit's `ActiveState`.
const UNIT_INTERFACE: &str = "org.freedesktop.systemd1.Unit";

/// The error name of a unit the manager does not hold (never loaded,
/// or unloaded after it stopped).
pub const NO_SUCH_UNIT: &str = "org.freedesktop.systemd1.NoSuchUnit";

/// The error name of an object that is gone by the time a property is read.
const UNKNOWN_OBJECT: &str = "org.freedesktop.DBus.Error.UnknownObject";

/// What:
///  A byte as two lowercase hexadecimal digits.
/// Why:
///  The `EXTERNAL` mechanism sends the user id's ASCII digits written in hexadecimal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hexOfByte = (byte: number) => byte.toString(16).padStart(2, '0');
/// ```
fn hex_of_byte(byte: u8) -> String {
    return format!("{byte:02x}");
}

/// What:
///  A failure to set a socket timeout or to connect or write,
/// as the call's connection error.
/// Why:
///  Every such failure is reported with the operating system's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const connectionFailure = (error: Error) => new CallError('connection', error.message);
/// ```
fn connection_failure(error: std::io::Error) -> CallError {
    return CallError::Connection(error.to_string());
}

/// What:
///  A failure to write during the authentication exchange,
/// as the call's authentication error.
/// Why:
///  The exchange reports the operating system's words,
/// with no timeout meaning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const authenticationFailure = (error: Error) => new CallError('authentication', error.message);
/// ```
fn authentication_failure(error: std::io::Error) -> CallError {
    return CallError::Authentication(error.to_string());
}

/// What:
///  A read failure during the authentication exchange:
/// running out of time stays a timeout,
/// anything else is an authentication error.
/// Why:
///  `read_line` waits within the call's deadline,
/// and a timeout must be told apart from a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const authenticationReadFailure = (error: Error) => isTimeout(error) ? timedOut : new CallError('authentication', error.message);
/// ```
fn authentication_read_failure(error: std::io::Error) -> CallError {
    return match socket_failure(&error) {
        CallError::TimedOut => CallError::TimedOut,
        _ => CallError::Authentication(error.to_string()),
    };
}

/// What:
///  A socket failure taken by value,
/// as the call's error.
/// Why:
///  `map_err` hands over the error itself,
/// and `socket_failure` borrows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const socketFailureOf = (error: Error) => socketFailure(error);
/// ```
fn socket_failure_of(error: std::io::Error) -> CallError {
    return socket_failure(&error);
}

/// The user id as the ASCII digits the `EXTERNAL` mechanism sends,
/// written in hexadecimal.
fn external_identity() -> String {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    let user: u32 = unsafe { libc::geteuid() };
    return user.to_string().bytes().map(hex_of_byte).collect();
}

/// The time left of the exchange,
/// or an error when it ran out.
fn remaining(deadline: Instant) -> Result<Duration, CallError> {
    let left: Duration = deadline.saturating_duration_since(Instant::now());
    return if left.is_zero() { Err(CallError::TimedOut) } else { Ok(left) };
}

/// A socket failure as the call's error:
/// running out of the read or write time is a timeout,
/// anything else a broken connection.
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
        stream.set_read_timeout(Some(remaining(deadline)?)).map_err(connection_failure)?;
        stream.read_exact(&mut one).map_err(authentication_read_failure)?;
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
    stream.set_read_timeout(Some(remaining(deadline)?)).map_err(connection_failure)?;
    stream.read_exact(header.as_mut_slice()).map_err(socket_failure_of)?;
    let body_length: usize = usize::try_from(read_u32(header.as_slice(), 4).unwrap_or(0)).unwrap_or(usize::MAX);
    let fields_length: usize = usize::try_from(read_u32(header.as_slice(), 12).unwrap_or(0)).unwrap_or(usize::MAX);
    let rest: usize = aligned(16usize.saturating_add(fields_length), 8).saturating_add(body_length).saturating_sub(16);
    if rest > MAX_REPLY_BYTES {
        return Err(CallError::Malformed(String::from("the reply is larger than this client reads")));
    }
    let mut remainder: Vec<u8> = vec![0; rest];
    stream.set_read_timeout(Some(remaining(deadline)?)).map_err(connection_failure)?;
    stream.read_exact(remainder.as_mut_slice()).map_err(socket_failure_of)?;
    header.extend(remainder);
    return Ok(header);
}

/// What:
///  The default session bus socket inside a runtime directory:
/// `bus` there.
/// Why:
///  `session_bus_path` falls back to it when the address variable names no usable path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const busInDirectory = (directory: string) => path.join(directory, 'bus');
/// ```
fn bus_in_directory(directory: &Path) -> std::path::PathBuf {
    return directory.join("bus");
}

/// The path of the session bus socket:
/// the `unix:path=` entry of the address variable,
/// else `$XDG_RUNTIME_DIR/bus`.
pub fn session_bus_path(address: Option<&str>, runtime_dir: Option<&Path>) -> Option<std::path::PathBuf> {
    // The first `unix:` entry that names a usable `path=` option wins; an entry that does not is skipped.
    let mut from_address: Option<std::path::PathBuf> = None;
    if let Some(list) = address {
        for entry in list.split(';') {
            let Some(rest) = entry.strip_prefix("unix:") else {
                continue;
            };
            let mut named: Option<&str> = None;
            for option in rest.split(',') {
                if let Some(found) = option.strip_prefix("path=") {
                    named = Some(found);
                    break;
                }
            }
            let Some(path) = named else {
                continue;
            };
            // Escapes (`%xx`) in a path are not read; a path that uses them falls back to the default location.
            if path.contains('%') || path.is_empty() {
                continue;
            }
            from_address = Some(std::path::PathBuf::from(path));
            break;
        }
    }
    if from_address.is_some() {
        return from_address;
    }
    return runtime_dir.map(bus_in_directory);
}

/// An authenticated connection to the session bus,
/// past `Hello`.
pub struct Bus {
    stream: UnixStream,
    deadline: Instant,
    serial: u32,
}

/// Methods over a connection.
impl Bus {
    /// What:
    ///  Connects to the bus socket,
    /// authenticates as the current user and says `Hello`,
    /// all within `wait`.
    /// Why:
    ///   The deadline of the whole exchange starts here,
    /// so every later call shares what is left of it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const bus = await Bus.connect(path, 20000)
    /// ```
    pub fn connect(socket: &Path, wait: Duration) -> Result<Bus, CallError> {
        let deadline: Instant = Instant::now() + wait;
        let mut stream: UnixStream = UnixStream::connect(socket).map_err(connection_failure)?;
        stream.set_write_timeout(Some(wait)).map_err(connection_failure)?;
        let mut hello: Vec<u8> = vec![0];
        hello.extend(format!("AUTH EXTERNAL {}\r\n", external_identity()).as_bytes());
        stream.write_all(hello.as_slice()).map_err(authentication_failure)?;
        let answer: String = read_line(&mut stream, deadline)?;
        if !answer.starts_with("OK ") {
            return Err(CallError::Authentication(answer));
        }
        stream.write_all(b"BEGIN\r\n").map_err(authentication_failure)?;
        let mut bus: Bus = Bus { stream, deadline, serial: 0 };
        bus.call(&HELLO, &[])?;
        return Ok(bus);
    }

    /// What:
    ///  Gives the calls that follow `wait` from now,
    /// replacing what was left of the time the connection was given.
    /// Why:
    ///   The time of the connection bounds a stuck bus,
    /// but a caller that spends time between calls (waiting for a service to
    ///  stop) must bound each phase by its own budget:
    /// a slow phase must not make the next one fail,
    /// and a query must not run
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
    ///  Makes one method call and waits for the reply that answers it,
    /// skipping the signals the bus sends meanwhile.
    /// Why:
    ///   A return is the receiver's confirmation (a reload returns after the units are read);
    /// an error reply comes back
    ///  with its name and text so the caller can say what to do.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await bus.call(RELOAD, [])
    /// ```
    pub fn call(&mut self, target: &Target<'_>, arguments: &[&str]) -> Result<Reply, CallError> {
        self.serial += 1;
        let serial: u32 = self.serial;
        self.stream.write_all(encode_call(serial, target, arguments).as_slice()).map_err(socket_failure_of)?;
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

    /// Clears a unit's failed state and its start-rate counter,
    /// so a start is not refused as "repeated too quickly".
    pub fn reset_failed(&mut self, name: &str) -> Result<(), CallError> {
        return self.call(&RESET_FAILED_UNIT, &[name]).map(drop);
    }

    /// Stops a unit,
    /// replacing a queued job for it.
    pub fn stop_unit(&mut self, name: &str) -> Result<(), CallError> {
        return self.call(&STOP_UNIT, &[name, "replace"]).map(drop);
    }

    /// What:
    ///  Stops one unit and nothing that depends on it (job mode `ignore-dependencies`).
    /// Why:
    ///   A service that `Requires=` a socket is stopped by the manager when that socket is stopped in the `replace` mode
    ///  (`systemd.unit(5)`,
    /// `Requires=`),
    /// which would cut the batch of a helper that is running.
    /// This mode applies the job to
    ///  the named unit only.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await bus.stopUnitAlone('cli-git@3f9a.socket') // the service of that socket keeps running
    /// ```
    pub fn stop_unit_alone(&mut self, name: &str) -> Result<(), CallError> {
        return self.call(&STOP_UNIT, &[name, STOP_ALONE_MODE]).map(drop);
    }

    /// What:
    ///  The unit's `ActiveState` (`active`,
    /// `activating`,
    /// `deactivating`,
    /// `inactive`,
    /// `failed`),
    /// or nothing when the manager
    ///  does not hold the unit.
    /// Why:
    ///   A stop call returns when the job is queued,
    /// so only this state says the unit and its processes are gone.
    /// `GetUnit`
    ///  answers the unit's object path and the property read answers a variant holding the state text.
    /// A unit that is not
    ///  loaded,
    /// or that unloads between the two calls,
    /// is not running.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await bus.activeState('cli-git.service') // 'deactivating' | 'inactive' | undefined
    /// ```
    pub fn active_state(&mut self, name: &str) -> Result<Option<String>, CallError> {
        let path: String = match self.call(&GET_UNIT, &[name]) {
            Ok(reply) => match reply.first_text {
                Some(text) => text,
                None => return Err(CallError::Malformed(String::from("GetUnit answered without a path"))),
            },
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT => return Ok(None),
            Err(error) => return Err(error),
        };
        let property: Target<'_> = Target { destination: MANAGER_NAME, path: path.as_str(), interface: PROPERTIES_INTERFACE, member: "Get" };
        return match self.call(&property, &[UNIT_INTERFACE, "ActiveState"]) {
            Ok(reply) => match reply.first_text {
                Some(text) => Ok(Some(text)),
                None => Err(CallError::Malformed(String::from("the state property answered without text"))),
            },
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT || error == UNKNOWN_OBJECT => Ok(None),
            Err(error) => Err(error),
        };
    }

    /// Starts a unit,
    /// replacing a queued job for it.
    pub fn start_unit(&mut self, name: &str) -> Result<(), CallError> {
        return self.call(&START_UNIT, &[name, "replace"]).map(drop);
    }
}

/// Client controls stay out of the release executable.
#[cfg(test)]
#[path = "systemd_dbus_tests.rs"]
mod tests;
