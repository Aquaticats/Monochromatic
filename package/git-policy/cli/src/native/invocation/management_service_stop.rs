//! What:
//!  `git monochromatic-git-policy-cli service stop`:
//! asks this worktree's commit service to finish what it holds and leave,
//! and says what became of it.
//! Why:
//!  The service has no unit to stop and no manager to ask, so a person's way to end it is this command:
//! it sends the same retire message a command sends when it replaces a service of another release
//! (`service_client_listen_and_ask_to_retire.rs`),
//! and waits, bounded, for the address to be free.
//! A command never needs this: the service ends by itself after three hours without work,
//! and the next commit starts it again.
//!  It is the answer for a person who wants it gone now, and for a script that wants no server of the wrapper running.
//! Nothing here reaches a service manager or a session bus.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const action = planServiceStop(environment, gitDir); // { code, stdout, stderr }
//! ```

use crate::action::Action;
use crate::service_client_listen_and_ask_to_retire::{ask_to_retire, START_WAIT};
use crate::service_host_of_a_worktree::runtime_directory;
use crate::service_instance::{instance_of, socket_path};
use crate::service_route::non_empty_path_variable;
use crate::service_server_start_by_the_wrapper::answered;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The command as a person types it,
/// named in every message about it.
const COMMAND: &str = "git monochromatic-git-policy-cli service stop";

/// How often the command looks for the address to be free.
const POLL: Duration = Duration::from_millis(25);

/// What:
///  Plans `service stop` for the worktree this command runs in.
/// Why:
///  The worktree is what names the address,
/// so a directory whose Git directory cannot be resolved is refused before anything is connected to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planServiceStop(environment, gitDir) // exit 0 when the service left or none answered
/// ```
pub fn plan_service_stop(environment: &[(std::ffi::OsString, std::ffi::OsString)], git_dir: &Path) -> Action {
    let instance: String = match crate::service_instance::try_instance_of(git_dir) {
        Ok(name) => name,
        Err(unreachable) => {
            return Action::Exit {
                code: 1,
                stdout: String::new(),
                stderr: format!("{COMMAND}: {unreachable}, so this command cannot tell which service to stop, and nothing was stopped.\n"),
            };
        }
    };
    let named: Option<PathBuf> = crate::child_environment::environment_value(environment, "XDG_RUNTIME_DIR").filter(|value| !value.is_empty()).map(PathBuf::from);
    let home: Option<PathBuf> = crate::child_environment::environment_value(environment, "HOME").filter(|value| !value.is_empty()).map(PathBuf::from);
    let _unused: Option<PathBuf> = non_empty_path_variable("XDG_STATE_HOME");
    let Some(runtime_dir) = runtime_directory(named.as_deref(), home.as_deref()) else {
        return Action::Exit {
            code: 1,
            stdout: String::new(),
            stderr: format!("{COMMAND}: this account names no runtime directory and no home, so there is nowhere its commit services could listen.\n"),
        };
    };
    let socket: PathBuf = socket_path(runtime_dir.as_path(), instance.as_str());
    return stop(socket.as_path(), instance.as_str());
}

/// What:
///  Asks the service at `socket` to leave and waits, bounded, for the address to be free.
/// Why:
///  A service that is not there is not a failure: there is nothing to stop.
/// A service that was asked and is still there at `START_WAIT` is one that could not finish what it holds in time,
/// and the answer says so instead of waiting longer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopAt(socket) // 'left' | 'none' | 'did-not-leave'
/// ```
fn stop(socket: &Path, instance: &str) -> Action {
    if !answered(socket) {
        return Action::Exit {
            code: 0,
            stdout: format!("No commit service answers for this worktree (instance {instance}), so there was nothing to stop.\n"),
            stderr: String::new(),
        };
    }
    ask_to_retire(socket);
    let deadline: Instant = Instant::now() + START_WAIT;
    loop {
        if !answered(socket) {
            return Action::Exit {
                code: 0,
                stdout: format!("The commit service of this worktree (instance {instance}) was asked to leave and has.\n"),
                stderr: String::new(),
            };
        }
        if Instant::now() >= deadline {
            return Action::Exit {
                code: 1,
                stdout: String::new(),
                stderr: format!(
                    "{COMMAND}: the commit service of this worktree (instance {instance}) was asked to leave and is still there after {} seconds. It finishes the batch it holds first; run this command again once that batch has landed.\n",
                    START_WAIT.as_secs()
                ),
            };
        }
        std::thread::sleep(POLL);
    }
}

/// Controls of the command stay out of the release executable.
#[cfg(test)]
#[path = "management_service_stop_tests.rs"]
mod tests;
