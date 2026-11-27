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
//! Only `ResetFailedUnit`,
//!  `RestartUnit`,
//! `StartUnit`,
//! `StopUnit` and the state query (`GetUnit`,
//! then a property read) are needed.
//! `Reload` is deliberately absent:
//! it makes the manager re-read every unit it holds,
//! so its cost grows with
//!  everything else the manager runs and it holds the manager's own event loop while it lasts
//! (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
//! Nothing here may ask for it.
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
//!
//! The names it calls them by are `systemd_dbus_manager_calls_and_error_names.rs`,
//! what a failed exchange means is `systemd_dbus_failures.rs`,
//! and where the bus listens is `systemd_dbus_session_address.rs`.
//!

use super::systemd_dbus_failures::{
    malformed,
    remaining,
    zbus_failure,
};
pub use super::systemd_dbus_manager_calls_and_error_names::{
    EXCHANGE_WAIT,
    GET_UNIT,
    HELLO,
    MANAGER_INTERFACE,
    MANAGER_NAME,
    MANAGER_PATH,
    NO_MANAGER_ON_THE_BUS,
    NO_SUCH_UNIT,
    RESET_FAILED_UNIT,
    RESTART_UNIT,
    START_UNIT,
    STOP_ALONE_MODE,
    STOP_UNIT,
    manager_is_absent,
};
use super::systemd_dbus_manager_calls_and_error_names::{
    PROPERTIES_INTERFACE,
    SOCKET_INTERFACE,
    UNIT_INTERFACE,
    UNKNOWN_OBJECT,
};
pub use super::systemd_dbus_session_address::session_bus_path;
pub use crate::systemd_dbus_wire_format::{
    CallError,
    Reply,
    Target,
    decode_reply,
    encode_call,
};
use std::path::Path;
use std::time::{
    Duration,
    Instant,
};

/// past `Hello`.
pub struct Bus {
    /// The connection;
    /// its reading tasks run on the process's runtime for as long as this value lives.
    connection: zbus::Connection,
    /// When the calls that follow must have ended.
    deadline: Instant,
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
fn bounded<T, F>(
    deadline: Instant,
    work: F,
) -> Result<T, CallError>
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
    pub fn connect(
        socket: &Path,
        wait: Duration,
    ) -> Result<Bus, CallError> {
        let deadline: Instant = Instant::now() + wait;
        let address: String = format!("unix:path={}", socket.display());
        let connection: zbus::Connection = bounded(deadline, async move {
            return zbus::connection::Builder::address(address.as_str())
                .build()
                .await
                .map_err(zbus_failure);
        })?;
        return Ok(Bus {
            connection,
            deadline,
        });
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
    pub fn within(
        &mut self,
        wait: Duration,
    ) {
        self.deadline = Instant::now() + wait;
    }

    /// What:
    ///  One method call of the user manager with `body` as its arguments,
    /// answered by the reply that belongs to it.
    /// Why:
    ///   A return is the receiver's confirmation (a start returns once the job is queued);
    /// an error reply comes back
    ///  with its name and text so the caller can say what to do.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await manager.call('StartUnit', [name, 'replace'])
    /// ```
    fn call_manager<B>(
        &self,
        member: &'static str,
        body: B,
    ) -> Result<zbus::Message, CallError>
    where
        B: zbus::export::serde::Serialize + zbus::wire::Type + Send + 'static,
    {
        let connection: zbus::Connection = self.connection.clone();
        return bounded(self.deadline, async move {
            return connection
                .call_method(
                    Some(MANAGER_NAME),
                    MANAGER_PATH,
                    Some(MANAGER_INTERFACE),
                    member,
                    &body,
                )
                .await
                .map_err(zbus_failure);
        });
    }

    /// Clears a unit's failed state and its start-rate counter,
    /// so a start is not refused as "repeated too quickly".
    pub fn reset_failed(
        &mut self,
        name: &str,
    ) -> Result<(), CallError> {
        return self
            .call_manager("ResetFailedUnit", (name.to_string(),))
            .map(drop);
    }

    /// Stops a unit,
    /// replacing a queued job for it.
    pub fn stop_unit(
        &mut self,
        name: &str,
    ) -> Result<(), CallError> {
        return self
            .call_manager("StopUnit", (name.to_string(), String::from("replace")))
            .map(drop);
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
    pub fn stop_unit_alone(
        &mut self,
        name: &str,
    ) -> Result<(), CallError> {
        return self
            .call_manager(
                "StopUnit",
                (name.to_string(), String::from(STOP_ALONE_MODE)),
            )
            .map(drop);
    }

    /// Starts a unit,
    /// replacing a queued job for it.
    pub fn start_unit(
        &mut self,
        name: &str,
    ) -> Result<(), CallError> {
        return self
            .call_manager("StartUnit", (name.to_string(), String::from("replace")))
            .map(drop);
    }

    /// What:
    ///  Stops a unit and starts it again,
    /// whether it runs or not,
    /// leaving the socket that activates it running.
    /// Why:
    ///  A unit the manager reports `active` whose program has stopped inside its own process is not repaired by a
    /// start
    /// (`systemctl` and the manager both leave a running unit alone),
    /// and the socket keeps its file either
    ///  way,
    /// so a connection waits in its backlog and nothing ever greets it.
    /// The service survives a kill by design (its journal,
    ///  its held commits and its work records are durable),
    /// so ending it is how a command that needs its answer gets one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await manager.call('RestartUnit', [name, 'replace'])
    /// ```
    pub fn restart_unit(
        &mut self,
        name: &str,
    ) -> Result<(), CallError> {
        return self
            .call_manager("RestartUnit", (name.to_string(), String::from("replace")))
            .map(drop);
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
    pub fn active_state(
        &mut self,
        name: &str,
    ) -> Result<Option<String>, CallError> {
        let path: String = match self.call_manager("GetUnit", (name.to_string(),)) {
            Ok(reply) => reply
                .body()
                .deserialize::<zbus::OwnedObjectPath>()
                .map_err(malformed)?
                .to_string(),
            Err(CallError::Manager {
                name: error,
                ..
            }) if error == NO_SUCH_UNIT => return Ok(None),
            Err(error) => return Err(error),
        };
        let connection: zbus::Connection = self.connection.clone();
        let reply = bounded(self.deadline, async move {
            return connection
                .call_method(
                    Some(MANAGER_NAME),
                    path.as_str(),
                    Some(PROPERTIES_INTERFACE),
                    "Get",
                    &(UNIT_INTERFACE, "ActiveState"),
                )
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
            Err(CallError::Manager {
                name: error,
                ..
            }) if error == NO_SUCH_UNIT || error == UNKNOWN_OBJECT => Ok(None),
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
    pub fn listen_addresses(
        &mut self,
        name: &str,
    ) -> Result<Option<Vec<String>>, CallError> {
        let path: String = match self.call_manager("GetUnit", (name.to_string(),)) {
            Ok(reply) => reply
                .body()
                .deserialize::<zbus::OwnedObjectPath>()
                .map_err(malformed)?
                .to_string(),
            Err(CallError::Manager {
                name: error,
                ..
            }) if error == NO_SUCH_UNIT => return Ok(None),
            Err(error) => return Err(error),
        };
        let connection: zbus::Connection = self.connection.clone();
        let reply = bounded(self.deadline, async move {
            return connection
                .call_method(
                    Some(MANAGER_NAME),
                    path.as_str(),
                    Some(PROPERTIES_INTERFACE),
                    "Get",
                    &(SOCKET_INTERFACE, "Listen"),
                )
                .await
                .map_err(zbus_failure);
        });
        let message = match reply {
            Ok(answered) => answered,
            Err(CallError::Manager {
                name: error,
                ..
            }) if error == NO_SUCH_UNIT || error == UNKNOWN_OBJECT => return Ok(None),
            Err(error) => return Err(error),
        };
        let value: zbus::OwnedVariant = message.body().deserialize().map_err(malformed)?;
        // `Listen` is one entry per address, each a kind (`Stream`, `Datagram`, ...) and the address text.
        let pairs: Vec<(String, String)> =
            match Vec::<(String, String)>::try_from(zbus::Variant::from(value)) {
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
