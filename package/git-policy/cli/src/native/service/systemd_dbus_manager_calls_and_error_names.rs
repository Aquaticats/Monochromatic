//! What:
//!  The names the user manager is reached by on the session bus:
//! its own name,
//! its object path and interface,
//! the call of each method the wrapper makes,
//! the job mode of a stop that leaves other units alone,
//! and the error names a bus or the manager answers with.
//! Why:
//!  A bus call is a name and a body (`systemd_dbus.rs`),
//! so the names sit beside the client that sends them and not in it:
//! a reader looking for what `StartUnit` is called on finds one list.
//!  The error names matter as much as the calls:
//! a bus that carries no user manager and a unit the manager does not hold are
//!  two states with two remedies,
//! and the client tells them apart by these names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const START_UNIT = { destination: 'org.freedesktop.systemd1', path: '/org/freedesktop/systemd1', interface: 'org.freedesktop.systemd1.Manager', member: 'StartUnit' };
//! ```

use crate::systemd_dbus_wire_format::Target;
use std::time::Duration;

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
pub const HELLO: Target<'static> = Target {
    destination: BUS_NAME,
    path: BUS_PATH,
    interface: BUS_NAME,
    member: "Hello",
};

/// `StartUnit` on the user manager.
pub const START_UNIT: Target<'static> = Target {
    destination: MANAGER_NAME,
    path: MANAGER_PATH,
    interface: MANAGER_INTERFACE,
    member: "StartUnit",
};

/// `RestartUnit` on the user manager.
pub const RESTART_UNIT: Target<'static> = Target {
    destination: MANAGER_NAME,
    path: MANAGER_PATH,
    interface: MANAGER_INTERFACE,
    member: "RestartUnit",
};

/// `StopUnit` on the user manager.
pub const STOP_UNIT: Target<'static> = Target {
    destination: MANAGER_NAME,
    path: MANAGER_PATH,
    interface: MANAGER_INTERFACE,
    member: "StopUnit",
};

/// `ResetFailedUnit` on the user manager.
pub const RESET_FAILED_UNIT: Target<'static> = Target {
    destination: MANAGER_NAME,
    path: MANAGER_PATH,
    interface: MANAGER_INTERFACE,
    member: "ResetFailedUnit",
};

/// `GetUnit` on the user manager:
/// the object path of a loaded unit.
pub const GET_UNIT: Target<'static> = Target {
    destination: MANAGER_NAME,
    path: MANAGER_PATH,
    interface: MANAGER_INTERFACE,
    member: "GetUnit",
};

/// The job mode that applies a stop to the named unit only,
/// leaving the units that require it running.
pub const STOP_ALONE_MODE: &str = "ignore-dependencies";

/// The interface whose `Get` reads one property of an object.
pub(crate) const PROPERTIES_INTERFACE: &str = "org.freedesktop.DBus.Properties";

/// The interface that holds a socket unit's addresses.
pub(crate) const SOCKET_INTERFACE: &str = "org.freedesktop.systemd1.Socket";

/// The interface that holds a unit's `ActiveState`.
pub(crate) const UNIT_INTERFACE: &str = "org.freedesktop.systemd1.Unit";

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
pub const NO_MANAGER_ON_THE_BUS: &[&str] = &[
    "org.freedesktop.DBus.Error.ServiceUnknown",
    "org.freedesktop.DBus.Error.NameHasNoOwner",
];

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
pub(crate) const UNKNOWN_OBJECT: &str = "org.freedesktop.DBus.Error.UnknownObject";
