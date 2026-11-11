//! What:
//!  A client of the user session bus,
//! on `zbus` 6 and the process's runtime:
//! connect,
//! authenticate as the current user,
//! say
//!  `Hello`,
//! and call methods of `org.freedesktop.systemd1.Manager`,
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
//! then a property read) are needed.
//! The owner chose `zbus` 6 over the
//!  hand-written client that stood here (2026-11-07:
//! "zbus 6,
//! and I suppose the lack of a blocking api can be worked around."):
//!  `zbus` 6 has no blocking client,
//! so the calls of this synchronous face enter the process's runtime
//!  (`foundation/lazy_runtime.rs`),
//! which builds on the first need,
//! until their callers are asynchronous (plan step T11).
//! A
//!  connection kept between calls keeps its reading tasks on the runtime's workers,
//! so a `Bus` is usable across the waits of a
//!  caller.
//! The hand-written wire format (`systemd_dbus_wire_format.rs`) is used by the controls' fake bus and the fuzz target
//!  until step T13 removes them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bus = await connect(address, 5000); await bus.startUnit('cli-git.socket', 'replace'); // throws with the error name
//! ```

pub use crate::systemd_dbus_wire_format::{CallError, Reply, Target, decode_reply, encode_call};
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

/// The interface that holds a socket unit's addresses.
const SOCKET_INTERFACE: &str = "org.freedesktop.systemd1.Socket";

/// The interface that holds a unit's `ActiveState`.
const UNIT_INTERFACE: &str = "org.freedesktop.systemd1.Unit";

/// The error name of a unit the manager does not hold (never loaded,
/// or unloaded after it stopped).
pub const NO_SUCH_UNIT: &str = "org.freedesktop.systemd1.NoSuchUnit";

/// What:
///  The error names a bus answers a call of the user manager with when the bus carries no user service manager.
/// Why:
///   A session bus without `systemd --user` is the one case where nothing about the units matters:
/// the bus itself says no manager owns the name,
/// and the person needs the remedy for an account without user services (`loginctl enable-linger`),
/// not the bus's own words.
/// `ServiceUnknown` is what a bus says when no unit and no program owns the name;
/// `NameHasNoOwner` is what it says when the name was owned and has gone.
pub const NO_MANAGER_ON_THE_BUS: [&str; 2] = ["org.freedesktop.DBus.Error.ServiceUnknown", "org.freedesktop.DBus.Error.NameHasNoOwner"];

/// Whether an error name says the bus that answered carries no user service manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const managerIsAbsent = (name: string) => NO_MANAGER_ON_THE_BUS.includes(name);
/// ```
pub fn manager_is_absent(error_name: &str) -> bool {
    return NO_MANAGER_ON_THE_BUS.contains(&error_name);
}

/// The error name of an object that is gone by the time a property is read.
const UNKNOWN_OBJECT: &str = "org.freedesktop.DBus.Error.UnknownObject";

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
    /// The connection;
    /// its reading tasks run on the process's runtime for as long as this value lives.
    connection: zbus::Connection,
    /// When the calls that follow must have ended.
    deadline: Instant,
}

/// A reply that did not have the expected shape,
/// as the call's `Malformed` error.
fn malformed(error: impl std::fmt::Display) -> CallError {
    return CallError::Malformed(error.to_string());
}

/// What:
///  A `zbus` failure as the call's error:
/// a method error keeps its name and text,
/// a handshake error is an authentication error,
/// and
///  running out of time on the socket is a timeout.
/// Why:
///  The callers tell a refusal of the manager,
/// a failed login,
/// a missing bus and a timeout apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const callError = (error: ZbusError): CallError => ...
/// ```
fn zbus_failure(error: zbus::Error) -> CallError {
    return match error {
        zbus::Error::MethodError(name, text, _message) => CallError::Manager { name: name.to_string(), text: text.unwrap_or_default() },
        zbus::Error::InputOutput(io) => socket_failure(&io),
        zbus::Error::Handshake(text) => CallError::Authentication(text),
        zbus::Error::Address(text) => CallError::Connection(text),
        other if other.to_string().starts_with("failed to connect") => CallError::Connection(other.to_string()),
        other => CallError::Malformed(other.to_string()),
    };
}

/// What:
///  Runs `work` on the process's runtime and gives up when `deadline` has passed.
/// Why:
///  One deadline covers the connect (the handshake and `Hello` are inside the connection's build) and each call,
/// so a bus that
///  accepts and says nothing never holds a commit longer.
/// A call from inside the runtime is refused with a plain connection error
///  rather than waiting for the runtime from within it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([work, delay(deadline - now)]).catch(() => { throw timedOut });
/// ```
fn bounded<T, F>(deadline: Instant, work: F) -> Result<T, CallError>
where
    F: std::future::Future<Output = Result<T, CallError>>,
{
    let left: Duration = remaining(deadline)?;
    let outcome = crate::lazy_runtime::run_on_runtime(async move {
        return match tokio::time::timeout(left, work).await {
            Ok(result) => result,
            Err(_elapsed) => Err(CallError::TimedOut),
        };
    });
    return match outcome {
        Ok(result) => result,
        Err(refusal) => Err(CallError::Connection(refusal.to_string())),
    };
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
        let address: String = format!("unix:path={}", socket.display());
        let connection: zbus::Connection = bounded(deadline, async move {
            return zbus::connection::Builder::address(address.as_str()).build().await.map_err(zbus_failure);
        })?;
        return Ok(Bus { connection, deadline });
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
    ///  One method call of the user manager with `body` as its arguments,
    /// answered by the reply that belongs to it.
    /// Why:
    ///   A return is the receiver's confirmation (a reload returns after the units are read);
    /// an error reply comes back
    ///  with its name and text so the caller can say what to do.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await manager.call('StartUnit', [name, 'replace'])
    /// ```
    fn call_manager<B>(&self, member: &'static str, body: B) -> Result<zbus::Message, CallError>
    where
        B: zbus::export::serde::Serialize + zbus::wire::Type + Send + 'static,
    {
        let connection: zbus::Connection = self.connection.clone();
        return bounded(self.deadline, async move {
            return connection.call_method(Some(MANAGER_NAME), MANAGER_PATH, Some(MANAGER_INTERFACE), member, &body).await.map_err(zbus_failure);
        });
    }

    /// Reloads the user manager's unit files.
    pub fn reload(&mut self) -> Result<(), CallError> {
        return self.call_manager("Reload", ()).map(drop);
    }

    /// Clears a unit's failed state and its start-rate counter,
    /// so a start is not refused as "repeated too quickly".
    pub fn reset_failed(&mut self, name: &str) -> Result<(), CallError> {
        return self.call_manager("ResetFailedUnit", (name.to_string(),)).map(drop);
    }

    /// Stops a unit,
    /// replacing a queued job for it.
    pub fn stop_unit(&mut self, name: &str) -> Result<(), CallError> {
        return self.call_manager("StopUnit", (name.to_string(), String::from("replace"))).map(drop);
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
        return self.call_manager("StopUnit", (name.to_string(), String::from(STOP_ALONE_MODE))).map(drop);
    }

    /// Starts a unit,
    /// replacing a queued job for it.
    pub fn start_unit(&mut self, name: &str) -> Result<(), CallError> {
        return self.call_manager("StartUnit", (name.to_string(), String::from("replace"))).map(drop);
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
        let path: String = match self.call_manager("GetUnit", (name.to_string(),)) {
            Ok(reply) => reply
                .body()
                .deserialize::<zbus::OwnedObjectPath>()
                .map_err(malformed)?
                .to_string(),
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT => return Ok(None),
            Err(error) => return Err(error),
        };
        let connection: zbus::Connection = self.connection.clone();
        let reply = bounded(self.deadline, async move {
            return connection
                .call_method(Some(MANAGER_NAME), path.as_str(), Some(PROPERTIES_INTERFACE), "Get", &(UNIT_INTERFACE, "ActiveState"))
                .await
                .map_err(zbus_failure);
        });
        return match reply {
            Ok(message) => {
                let value: zbus::OwnedVariant = message.body().deserialize().map_err(malformed)?;
                match String::try_from(value) {
                    Ok(text) => Ok(Some(text)),
                    Err(error) => Err(CallError::Malformed(error.to_string())),
                }
            }
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT || error == UNKNOWN_OBJECT => Ok(None),
            Err(error) => Err(error),
        };
    }

    /// What:
    ///  The addresses the manager made one unit listen at (its `Listen`),
    /// or nothing when the manager holds no such unit.
    /// Why:
    ///   The unit names its address with `%t`,
    /// which the manager reads as its own runtime directory,
    /// and this client connects to a path under the runtime directory it was given.
    /// The manager's answer is the only fact that says the two agree;
    /// a command whose runtime directory the manager does not use waits for a service that answers somewhere else,
    /// and this answer names where.
    /// An answer that does not read as a list of address pairs says nothing rather than guessing,
    /// and a caller then has no proof either way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await bus.listenAddresses('cli-git@3f9a.socket') // ['/run/user/1000/cli-git/3f9a.sock']
    /// ```
    pub fn listen_addresses(&mut self, name: &str) -> Result<Option<Vec<String>>, CallError> {
        let path: String = match self.call_manager("GetUnit", (name.to_string(),)) {
            Ok(reply) => reply
                .body()
                .deserialize::<zbus::OwnedObjectPath>()
                .map_err(malformed)?
                .to_string(),
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT => return Ok(None),
            Err(error) => return Err(error),
        };
        let connection: zbus::Connection = self.connection.clone();
        let reply = bounded(self.deadline, async move {
            return connection
                .call_method(Some(MANAGER_NAME), path.as_str(), Some(PROPERTIES_INTERFACE), "Get", &(SOCKET_INTERFACE, "Listen"))
                .await
                .map_err(zbus_failure);
        });
        let message = match reply {
            Ok(answered) => answered,
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT || error == UNKNOWN_OBJECT => return Ok(None),
            Err(error) => return Err(error),
        };
        let value: zbus::OwnedVariant = message.body().deserialize().map_err(malformed)?;
        // `Listen` is one entry per address, each a kind (`Stream`, `Datagram`, ...) and the address text.
        let pairs: Vec<(String, String)> = match Vec::<(String, String)>::try_from(zbus::Variant::from(value)) {
            Ok(read) => read,
            Err(error) => return Err(CallError::Malformed(error.to_string())),
        };
        let mut addresses: Vec<String> = Vec::new();
        for (_kind, address) in pairs {
            addresses.push(address);
        }
        return Ok(Some(addresses));
    }
}

/// Client controls stay out of the release executable.
#[cfg(test)]
#[path = "systemd_dbus_tests.rs"]
mod tests;
