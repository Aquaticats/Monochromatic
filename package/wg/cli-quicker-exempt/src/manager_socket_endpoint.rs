//! Where a user's manager answers,
//! and which process must be the one behind that socket.
//! The socket is the manager's own private one,
//! which it created and which only it answers,
//! and the expected pid is the main process of `user@<uid>.service`,
//! as the system manager reports it.

/// The search path and the time limit of the system manager query.
use crate::bounded_command::run_bounded;
/// Argument bytes that need not be text.
use std::ffi::OsString;
/// Sockets and paths.
use std::path::PathBuf;
/// Bounded waits.
use std::time::Duration;

/// How long the system manager may take to say which process runs a user's manager.
const MAIN_PID_TIMEOUT: Duration = Duration::from_secs(5);

/// Where a user's manager answers,
/// and which process must be behind that socket.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagerEndpoint {
    /// The manager's private socket.
    pub socket: PathBuf,
    /// The main process of the user's manager unit.
    pub expected_pid: i32,
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
        return Err(format!(
            "systemctl show user@{uid}.service exited with {}: {}",
            output.status,
            output.stderr.trim()
        ));
    }
    let text = output.stdout.trim();
    let pid: i32 = match text.parse() {
        Ok(pid) => pid,
        Err(_error) => {
            return Err(format!(
                "systemctl named no main process of user@{uid}.service ({text:?})"
            ));
        }
    };
    if pid <= 1 {
        return Err(format!(
            "the system manager reports no running main process of user@{uid}.service ({pid})"
        ));
    }
    return Ok(pid);
}

/// Resolving and using the channel.
impl ManagerEndpoint {
    /// Where the user's manager of `uid` answers,
    /// and which process must be there.
    pub fn resolve(uid: u32) -> Result<ManagerEndpoint, String> {
        return Ok(ManagerEndpoint {
            socket: manager_socket_path(uid),
            expected_pid: manager_main_pid(uid)?,
        });
    }
}
