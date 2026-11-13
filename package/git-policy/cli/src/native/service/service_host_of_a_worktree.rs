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
//! `$XDG_RUNTIME_DIR`, and the journal is under the state folder
//! (`journal_of`),
//! which is where the unit's `StateDirectory=` used to put it,
//! so an upgrade keeps every record it had.
//! This file replaces the provisioning host, which carried the user's unit folder,
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

    /// What starting this worktree's service needs.
    pub fn server(&self) -> ServerToStart {
        return ServerToStart {
            socket: self.socket(),
            instance: self.instance.clone(),
            journal: self.journal(),
            git: self.git.clone(),
            program: self.service_binary.clone(),
        };
    }
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

/// Controls of the derived paths stay out of the release executable.
#[cfg(test)]
#[path = "service_host_of_a_worktree_tests.rs"]
mod tests;
