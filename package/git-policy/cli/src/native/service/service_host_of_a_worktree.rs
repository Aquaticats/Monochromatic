//! What:
//!  What one wrapper command knows about the commit service of the worktree it runs in:
//! the runtime and state folders,
//! the worktree's instance,
//! its socket and its journal,
//! the wrapper's own program and the real Git the service runs.
//! Why:
//!  A command that reaches the service,
//! starts it or hands work to it all need the same paths,
//! and each of them is derived from the worktree and the account's folders,
//! never from a service manager:
//! the socket is beside the worktree's other service files under
//! `$XDG_RUNTIME_DIR`,
//! and the journal is under the state folder
//! (`journal_of`),
//! which is where the unit's `StateDirectory=` used to put it,
//! so an upgrade keeps every record it had.
//! A runtime directory of the command's own is a boundary of its own:
//! nothing of such a run is written under the account's real state directory
//! (`state_dir_of_the_command`,
//! and issue 6293).
//! This file replaces the provisioning host,
//! which carried the user's unit folder,
//! the session bus and the manager's wait;
//! nothing here names a manager.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const host = { runtimeDir, stateDir, serviceBinary, git, instance, gitDir };
//! host.socket; host.journal; host.serverToStart;
//! ```

use crate::service_instance::{socket_path, SOCKET_DIRECTORY};
use crate::service_journal_folder_of_an_instance::journal_of;
use crate::service_server_start_by_the_wrapper::ServerToStart;
use crate::service_session_of_the_runtime_directory::is_another_session;
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{Path, PathBuf};

/// What one command needs to reach or start a worktree's commit service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceHost {
    /// The account's runtime folder (`XDG_RUNTIME_DIR`),
    /// where the sockets live and which is emptied at logout.
    pub runtime_dir: PathBuf,
    /// The account's state folder (`XDG_STATE_HOME`,
    /// else `~/.local/state`),
    /// where the services' journals live.
    pub state_dir: PathBuf,
    /// The wrapper's own executable,
    /// which is also the service's program.
    pub service_binary: PathBuf,
    /// The real Git the service runs.
    pub git: PathBuf,
    /// The worktree's instance name (`service_instance::instance_of`).
    pub instance: String,
    /// The worktree's own Git directory,
    /// written beside the socket so the service and the sweep know whose it is.
    pub git_dir: PathBuf,
    /// What:
    ///  The runtime directory the environment named,
    ///  and nothing when the command fell back to a folder of its own (`runtime_directory`).
    /// Why:
    ///  The enhancement asks a service manager only for the session whose bus this is,
    /// and a folder the wrapper fell back to is no session at all
    /// (`service_server_start_through_the_manager.rs`,
    /// and issue 6293).
    pub named_runtime_dir: Option<PathBuf>,
    /// What:
    ///  The session bus address the environment gave this command,
    ///  which is the manager's if its socket sits in the runtime directory.
    /// Why:
    ///  A bus that belongs to another directory is another session's manager,
    /// and asking it would put this run's service in the account's manager.
    pub bus_address: Option<String>,
}

impl ServiceHost {
    /// The folder every worktree's socket sits in.
    pub fn socket_directory(&self) -> PathBuf {
        return self.runtime_dir.join(SOCKET_DIRECTORY);
    }

    /// The address this worktree's service listens at.
    pub fn socket(&self) -> PathBuf {
        return socket_path(self.runtime_dir.as_path(), self.instance.as_str());
    }

    /// The folder this worktree's service keeps its journal in.
    pub fn journal(&self) -> PathBuf {
        return journal_of(&self.state_dir.join(SERVICE_UNIT.current), self.instance.as_str());
    }

    /// What:
    ///  The manager this command may ask for a scope,
    ///  and nothing when the base starts the server itself.
    /// Why:
    ///  The decision reads two facts of the environment and the presence of the manager's program,
    /// so a caller never repeats it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// host.manager() // ManagerStart | undefined
    /// ```
    pub fn manager(&self) -> Option<crate::service_server_start_through_the_manager::ManagerStart> {
        return crate::service_server_start_through_the_manager::ManagerStart::in_this_session(self);
    }

    /// What starting this worktree's service needs.
    pub fn server(&self) -> ServerToStart {
        return ServerToStart {
            socket: self.socket(),
            instance: self.instance.clone(),
            journal: self.journal(),
            git: self.git.clone(),
            program: self.service_binary.clone(),
            manager: self.manager(),
        };
    }
}

/// What:
///  The state folder this command's commit service keeps its records in:
///  the account's (`XDG_STATE_HOME`,
/// else `~/.local/state`),
///  or the runtime directory itself when that directory is provably the command's own.
/// Why:
///   A run given a runtime directory of its own is a boundary of its own
/// (issue 6293,
/// the owner's decision of 2026-11-13):
/// its service is not the account's session's service,
/// so nothing of it may be written under the account's real state directory,
/// and its journal,
/// its retry plans and its scratch files live under that runtime directory instead.
/// A session whose own bus lives in its runtime directory is unaffected:
/// its service state stays exactly where earlier releases kept it.
/// An account that named no runtime directory (`~/.run`) keeps the account's state folder too:
/// no session was named,
/// so nothing here belongs to another session.
/// What is not proven stays where a logout does not clear it:
/// an abstract bus address,
/// a bus that is not a Unix socket,
/// and no address at all prove neither this session nor another,
/// so such a command keeps the account's state folder and reads the earlier records there,
/// because the alternative is a real session whose retry plans vanish with its logout
/// (the review of issue 6293,
/// finding 6).
/// The trade is the private directory's own:
/// its records do not outlive a logout or a reboot,
/// and an earlier release's journal under the account's state directory is not adopted by such a run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stateDirOfTheCommand(env) // isAnotherSession(env) ? runtimeDir : accountStateDir
/// ```
pub fn state_dir_of_the_command(
    named_runtime_directory: Option<&Path>,
    runtime_dir: &Path,
    bus_address: Option<&str>,
    state_home: Option<&Path>,
    home: Option<&Path>,
) -> Option<PathBuf> {
    if is_another_session(named_runtime_directory, runtime_dir, bus_address) {
        return Some(runtime_dir.to_path_buf());
    }
    return state_dir_of(state_home, home);
}

/// What:
///  The account's state folder,
/// as the environment names it.
/// Why:
///   The journal's home must be the one earlier releases used,
/// which is `XDG_STATE_HOME` when it is set and not empty,
/// else `~/.local/state`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stateDirOf(stateHome, home) // stateHome || home && join(home, '.local/state')
/// ```
pub fn state_dir_of(state_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    if let Some(named) = state_home
        && !named.as_os_str().is_empty()
    {
        return Some(named.to_path_buf());
    }
    let named: &Path = home?;
    return Some(named.join(".local").join("state"));
}

/// What:
///  The folder this account's commit services keep their sockets in when the environment names no runtime folder:
///  `.run` in the home directory.
/// Why:
///   A machine with no login session sets no `XDG_RUNTIME_DIR` (a container,
/// a service's own child,
/// a cron job),
/// and a commit must land there as it does anywhere else:
/// real Git commits there,
/// so this wrapper does.
/// The home is the one place an account has of its own that survives a reboot,
/// and `.run` is four characters,
/// which leaves a socket address the room it needs.
/// A folder the wrapper makes is private (0700);
/// `ensure_private` sets that at every start.
/// An account with neither a named folder nor a home has nowhere of its own to put a socket,
/// and says so instead of guessing a folder that is not its.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runtimeDirectory({ named: process.env.XDG_RUNTIME_DIR, home: process.env.HOME }) // `${home}/.run` or undefined
/// ```
pub fn runtime_directory(named: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    if let Some(directory) = named
        && !directory.as_os_str().is_empty()
    {
        return Some(directory.to_path_buf());
    }
    // The home is the only folder an account certainly has; a folder named by nothing would be anyone's.
    let place: &Path = home?;
    return Some(place.join(".run"));
}

/// Controls of the derived paths stay out of the release executable.
#[cfg(test)]
#[path = "service_host_of_a_worktree_tests.rs"]
mod tests;
