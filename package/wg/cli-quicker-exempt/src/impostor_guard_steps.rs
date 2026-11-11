//! The real steps the guard performs:
//! the system manager for the user manager's main process,
//! the user's session bus for the owner of the manager's name,
//! `SIGKILL` for an impostor,
//! and `systemctl --user daemon-reexec` for the manager.

/// The session bus connection and its socket.
use crate::session_bus::BusConnection;
/// The manager's main process.
use crate::manager_socket::manager_main_pid;
/// Bounded commands and the fixed search path.
use crate::bounded_command::run_bounded;
/// The name the guard watches,
/// the rule it asks for,
/// and what it reads.
use super::{GuardSteps, MANAGER_BUS_NAME, Ownership, match_rule};
/// Argument bytes that need not be text.
use std::ffi::OsString;
/// The bus socket's path.
use std::path::PathBuf;
/// Bounded waits.
use std::time::Duration;

/// How long one `systemctl` call may take.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

/// The session bus socket of a user.
pub fn session_bus_path(uid: u32) -> PathBuf {
    return PathBuf::from(format!("/run/user/{uid}/bus"));
}

/// Asks the bus who owns the name.
pub fn owner_of(connection: &mut BusConnection, name: &str) -> Ownership {
    let unique = match connection.call_owner(name) {
        Ok(Some(unique)) => unique,
        Ok(None) => return Ownership::Nobody,
        Err(_error) => return Ownership::Unknown,
    };
    return match connection.call_owner_pid(&unique) {
        Ok(pid) => Ownership::Owner(pid),
        Err(_error) => Ownership::Unknown,
    };
}

/// Running a bounded `systemctl` as the user.
fn systemctl_user(uid: u32, arguments: &[&str]) -> Result<(), String> {
    let mut full: Vec<OsString> = vec![
        OsString::from("--reuid"),
        OsString::from(uid.to_string()),
        OsString::from("--regid"),
        OsString::from(uid.to_string()),
        OsString::from("--clear-groups"),
        OsString::from("--no-new-privs"),
        OsString::from("systemctl"),
        OsString::from("--user"),
    ];
    for argument in arguments {
        full.push(OsString::from(argument));
    }
    let output = run_bounded("setpriv", &full, COMMAND_TIMEOUT)?;
    if !output.success {
        return Err(format!("systemctl --user {} exited with {}: {}", arguments.join(" "), output.status, output.stderr.trim()));
    }
    return Ok(());
}

/// The real steps:
/// the system manager,
/// the user's session bus,
/// `SIGKILL` and `daemon-reexec`.
pub struct RealSteps {
    /// The bus connection,
    /// opened when the guard starts and again when it breaks.
    connection: Option<BusConnection>,
    /// How long a bus call may take.
    pub timeout: Duration,
}

/// Construction and the connection.
impl RealSteps {
    /// Builds the steps with a bus connection that is opened when it is first needed.
    pub fn new(timeout: Duration) -> RealSteps {
        return RealSteps { connection: None, timeout };
    }

    /// The bus connection,
    /// opened or reopened.
    fn connect(&mut self, uid: u32) -> Result<&mut BusConnection, String> {
        if self.connection.is_none() {
            let connection = match BusConnection::connect(&session_bus_path(uid), self.timeout, &match_rule(MANAGER_BUS_NAME)) {
                Ok(connection) => connection,
                Err(error) => return Err(error.to_string()),
            };
            self.connection = Some(connection);
        }
        return match self.connection.as_mut() {
            Some(connection) => Ok(connection),
            None => Err("the session bus connection is not open".to_string()),
        };
    }
}

/// The real steps of the guard.
impl GuardSteps for RealSteps {
    /// The main process of the user's manager,
    /// as the system manager reports it.
    fn expected_pid(&self, uid: u32) -> Option<i32> {
        return manager_main_pid(uid).ok();
    }

    /// Who holds the name now,
    /// asking the bus again when the connection broke.
    fn ownership(&mut self, uid: u32) -> Ownership {
        let connection = match self.connect(uid) {
            Ok(connection) => connection,
            Err(_error) => return Ownership::Unknown,
        };
        let ownership = owner_of(connection, MANAGER_BUS_NAME);
        if ownership == Ownership::Unknown {
            self.connection = None;
        }
        return ownership;
    }

    /// Waits for the bus's own event that the name's owner changed,
    /// which the guard acts on at once instead of waiting out the poll.
    fn wait_for_change(&mut self, uid: u32, timeout: Duration) -> bool {
        let full = self.timeout;
        let connection = match self.connect(uid) {
            Ok(connection) => connection,
            Err(_error) => {
                std::thread::sleep(timeout);
                return false;
            }
        };
        connection.set_timeout(timeout);
        let changed = matches!(connection.next_signal(), Ok(Some(_signal)));
        connection.set_timeout(full);
        return changed;
    }

    /// Kills the process that holds the name.
    /// `SIGKILL`,
    /// never `SIGTERM`:
    /// an impostor manager stops the units it started when it is asked to stop.
    fn kill(&self, pid: i32) -> Result<(), String> {
        // SAFETY: kill takes a pid and a signal and has no preconditions beyond a valid signal number.
        let result = unsafe { libc::kill(pid, libc::SIGKILL) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        return Ok(());
    }

    /// Asks the manager to take the name again.
    fn reexec(&self, uid: u32) -> Result<(), String> {
        return systemctl_user(uid, &["daemon-reexec"]);
    }
}
