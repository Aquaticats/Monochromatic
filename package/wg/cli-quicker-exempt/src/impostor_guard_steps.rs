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
/// Turning a descriptor into an owned one.
use std::os::fd::FromRawFd;
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

/// Asks the bus who owns the name,
/// and keeps the unique connection name as well as the pid:
/// the pid alone is not an identity the kernel keeps,
/// and the kill proves the unique name again before it signals.
pub fn owner_of(connection: &mut BusConnection, name: &str) -> Ownership {
    let unique = match connection.call_owner(name) {
        Ok(Some(unique)) => unique,
        Ok(None) => return Ownership::Nobody,
        Err(_error) => return Ownership::Unknown,
    };
    return match connection.call_owner_pid(&unique) {
        Ok(pid) => Ownership::Owner { pid, unique },
        Err(_error) => Ownership::Unknown,
    };
}

/// Asks the bus which pid a unique connection name has.
pub trait OwnerLookup {
    /// The pid the bus reports for `unique`,
    /// or `None` when the bus does not name one,
    /// which is no proof of anything.
    fn pid_of(&mut self, unique: &str) -> Option<i32>;
}

/// Signals a process through a handle that keeps it that process.
pub trait SignalSend {
    /// Signals `pid`,
    /// and only the process the handle was opened for.
    fn send(&self, pid: i32) -> Result<(), String>;
}

/// The bus, as the proof asks it.
struct BusLookup<'a> {
    /// The open connection.
    connection: &'a mut BusConnection,
}

/// Asking the bus.
impl OwnerLookup for BusLookup<'_> {
    /// The pid the bus reports for a unique connection name.
    fn pid_of(&mut self, unique: &str) -> Option<i32> {
        return self.connection.call_owner_pid(unique).ok();
    }
}

/// A process file descriptor opened for one pid,
/// so a pid the kernel gives to another process is never signalled.
pub struct PidfdSignal;

/// Opening the handle and signalling through it.
impl SignalSend for PidfdSignal {
    /// Signals the process the handle was opened for.
    fn send(&self, pid: i32) -> Result<(), String> {
        // SAFETY: pidfd_open takes a pid and flags and returns a new descriptor or a negative value.
        let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
        if raw < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENOSYS) {
                // A kernel without process descriptors: the bus has just named this pid,
                // and the window the descriptor would close stays open on such a kernel.
                return kill_directly(pid);
            }
            return Err(format!("pid {pid} has no process descriptor: {error}"));
        }
        // SAFETY: a successful pidfd_open returned a descriptor owned by nobody else.
        let descriptor = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw as libc::c_int) };
        // SAFETY: `descriptor` is a live process descriptor and the signal number is valid.
        let result = unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                std::os::fd::AsRawFd::as_raw_fd(&descriptor),
                libc::SIGKILL,
                std::ptr::null::<libc::siginfo_t>(),
                0,
            )
        };
        if result < 0 {
            return Err(format!("pid {pid} could not be signalled through its process descriptor: {}", std::io::Error::last_os_error()));
        }
        return Ok(());
    }
}

/// Signals a pid directly,
/// for a kernel with no process descriptors.
fn kill_directly(pid: i32) -> Result<(), String> {
    // SAFETY: kill takes a pid and a signal and has no preconditions beyond a valid signal number.
    let result = unsafe { libc::kill(pid, libc::SIGKILL) };
    if result != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    return Ok(());
}

/// Signals `pid` only when the bus still says that `unique` names it.
///
/// A pid the bus named is not an identity the kernel keeps:
/// the owner can exit and the kernel can give its number to another process of the same user between the answer and the signal.
/// Asking the bus again about the unique connection name,
/// which only that connection can hold,
/// is the proof that the pid still names the owner,
/// and the process descriptor opened for it keeps the signal on that process for the instant between the proof and the signal.
///
/// Every answer that is not exactly this pid is a refusal, and nothing is signalled.
pub fn kill_proven(
    lookup: &mut dyn OwnerLookup,
    signal: &dyn SignalSend,
    unique: &str,
    pid: i32,
) -> Result<(), String> {
    let Some(named) = lookup.pid_of(unique) else {
        return Err(format!("the bus no longer names a process for {unique}, so pid {pid} is not signalled"));
    };
    if named != pid {
        return Err(format!("{unique} now names pid {named} and not pid {pid}, so pid {pid} is not signalled"));
    }
    return signal.send(pid);
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

    /// Kills the process that holds the name,
    /// after proving through the bus that the unique connection name still names it.
    /// `SIGKILL`,
    /// never `SIGTERM`:
    /// an impostor manager stops the units it started when it is asked to stop.
    fn kill(&mut self, uid: u32, unique: &str, pid: i32) -> Result<(), String> {
        let connection = match self.connect(uid) {
            Ok(connection) => connection,
            Err(error) => return Err(format!("the bus cannot be asked whether {unique} still names pid {pid}: {error}")),
        };
        let mut lookup = BusLookup { connection };
        return kill_proven(&mut lookup, &PidfdSignal, unique, pid);
    }

    /// Asks the manager to take the name again.
    fn reexec(&self, uid: u32) -> Result<(), String> {
        return systemctl_user(uid, &["daemon-reexec"]);
    }
}
