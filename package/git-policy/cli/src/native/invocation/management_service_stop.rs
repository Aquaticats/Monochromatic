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
use crate::service_address_of_a_worktree::socket_address_of;
use crate::service_route::non_empty_path_variable;
use crate::service_server::Bind;
use crate::service_server_start_by_the_wrapper::{answered, accepts};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The command as a person types it,
/// named in every message about it.
const COMMAND: &str = "git monochromatic-git-policy-cli service stop";

/// How often the command looks for the address to be free.
const POLL: Duration = Duration::from_millis(25);

/// What:
///  The one bound of this command:
///  how long a service it asked to leave is waited for,
///  and how long a service it ended is waited for to stop taking connections.
/// Why:
///  It is one start's bound (`START_WAIT`): a service finishing the batch it holds does so inside the time a start is
///  given, and the same number is what a person pays at most for a service that cannot be ended.
///  One constant, because the two waits are one question: is the address free now?
pub const STOP_BOUND: Duration = START_WAIT;

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
    let named: Option<PathBuf> = crate::child_environment::environment_value(environment, "XDG_RUNTIME_DIR").filter(|value| {
        return !value.is_empty();
    }).map(PathBuf::from);
    let home: Option<PathBuf> = crate::child_environment::environment_value(environment, "HOME").filter(|value| {
        return !value.is_empty();
    }).map(PathBuf::from);
    let _unused: Option<PathBuf> = non_empty_path_variable("XDG_STATE_HOME");
    let Some(runtime_dir) = runtime_directory(named.as_deref(), home.as_deref()) else {
        return Action::Exit {
            code: 1,
            stdout: String::new(),
            stderr: format!("{COMMAND}: this account names no runtime directory and no home, so there is nowhere its commit services could listen.\n"),
        };
    };
    let address: Bind = socket_address_of(runtime_dir.as_path(), instance.as_str());
    let directory: PathBuf = crate::service_instance::socket_path(runtime_dir.as_path(), instance.as_str()).parent().map(Path::to_path_buf).unwrap_or_else(|| {
        return runtime_dir.clone();
    });
    return stop(&address, directory.as_path(), instance.as_str());
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
/// What:
///  Ends this worktree's service:
///  asks one that answers to leave,
///  and ends one that is alive and does not answer,
///  by the process its own note proves,
///  and waits for the address to be free within `STOP_BOUND`.
/// Why:
///  This command is the remedy every failure text of this program names for a service that is there and does not answer,
///  so it has to end that service too: a service stopped inside its own process, under a debugger, or in a kernel wait takes
///  connections and greets nothing, and "there was nothing to stop" about it sent the person back to the same failure
///  (the review of `b896c60d97`, finding 1).
///  What ends it is the note the service wrote with its own birth identity, never a number this command guessed
///  (`service_server_start_through_the_manager.rs`, `end_the_service_the_note_names`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = greet(address); if (answered) askToRetire(address)
/// else if (accepts(address)) { const note = readNote(dir, instance); if (proven(note)) kill(note.pid) }
/// await freeWithin(STOP_BOUND)
/// ```
fn stop(address: &Bind, directory: &Path, instance: &str) -> Action {
    let greeted: bool = answered(address);
    if greeted {
        ask_to_retire(address);
    } else if accepts(address) {
        // What: A service that takes connections and greets nothing is ended by the note its own process wrote.
        // Why:  Nothing else proves which process serves this address, and a number read from a file with no identity is a
        //       stranger's process (`end_the_service_the_note_names`).
        let note: PathBuf = crate::service_server_pid_note_of_an_instance::path(directory, instance);
        let service: Option<crate::service_server_pid_note_of_an_instance::ServerProcess> = crate::service_server_pid_note_of_an_instance::read(note.as_path());
        let proven: bool = service.as_ref().is_some_and(crate::service_server_pid_note_of_an_instance::is_still_the_process);
        if !proven {
            return Action::Exit {
                code: 1,
                stdout: String::new(),
                stderr: format!(
                    "{COMMAND}: the commit service of this worktree (instance {instance}) is there at {} and does not answer, and the note beside its address names no process that is still running, so this command cannot end it. Look at what holds that address (its note is {}) and end that process, or remove the address and the note once nothing is left. Nothing else was changed.\n",
                    crate::service_server::address_text(address),
                    note.display()
                ),
            };
        }
        let ended: bool = crate::service_server_start_through_the_manager::end_the_service_the_note_names(directory, instance);
        let pid: i32 = service.map_or(0, |named| { return named.pid; });
        if ended && wait_until_free(address, STOP_BOUND) {
            return Action::Exit {
                code: 0,
                stdout: format!(
                    "The commit service of this worktree (instance {instance}) was running and answered nothing, so it was ended (pid {pid}) and its address is free.\n"
                ),
                stderr: String::new(),
            };
        }
        return Action::Exit {
            code: 1,
            stdout: String::new(),
            stderr: format!(
                "{COMMAND}: the commit service of this worktree (instance {instance}, pid {pid}) does not answer, and it was ended but its address was still taken after {} seconds. Look at {} and at its output beside it.\n",
                STOP_BOUND.as_secs(),
                crate::service_server::address_text(address)
            ),
        };
    } else {
        // Nothing takes a connection at the address: no service of this worktree is there.
        let note: PathBuf = crate::service_server_pid_note_of_an_instance::path(directory, instance);
        let service: Option<crate::service_server_pid_note_of_an_instance::ServerProcess> = crate::service_server_pid_note_of_an_instance::read(note.as_path());
        if service.as_ref().is_some_and(crate::service_server_pid_note_of_an_instance::is_still_the_process) {
            // A service whose address is gone but whose process runs: it is ended by its note, as one that answers nothing is.
            let _ended: bool = crate::service_server_start_through_the_manager::end_the_service_the_note_names(directory, instance);
            return Action::Exit {
                code: 0,
                stdout: format!("The commit service of this worktree (instance {instance}) held no address and was ended by the note that names it.\n"),
                stderr: String::new(),
            };
        }
        return Action::Exit {
            code: 0,
            stdout: format!("No commit service answers for this worktree (instance {instance}), so there was nothing to stop.\n"),
            stderr: String::new(),
        };
    }
    if wait_until_free(address, STOP_BOUND) {
        return Action::Exit {
            code: 0,
            stdout: format!("The commit service of this worktree (instance {instance}) was asked to leave and has.\n"),
            stderr: String::new(),
        };
    }
    // What: A service that was asked to leave and is still there after its bound is ended by the process its own note proves.
    // Why:  `service stop` is the command every failure of this program names for a service that does not answer, and a service
    //       held inside a landing that never ends (a branch write on a hung mount, a read of a file a person left as a pipe)
    //       answered "it finishes the batch it holds first; run this command again" for ever: the review of `e347be3115`
    //       (part two-a, finding 5) measured the same words 30 s later, and only `kill -TERM` to the number in the note ended it.
    //       The batch such a service held is not lost: it is in the journal, and the next service recovers it (the fault points
    //       of the landing are the same state, and every one of them recovers).
    let note: PathBuf = crate::service_server_pid_note_of_an_instance::path(directory, instance);
    let proven: Option<crate::service_server_pid_note_of_an_instance::ServerProcess> = crate::service_server_pid_note_of_an_instance::read(note.as_path());
    let named: Option<i32> = proven.as_ref().filter(|service| { return crate::service_server_pid_note_of_an_instance::is_still_the_process(service); }).map(|service| { return service.pid; });
    if let Some(pid) = named {
        let ended: crate::service_ended_through_a_pinned_handle::Ended = crate::service_ended_through_a_pinned_handle::end_the_service(directory, instance, None);
        let free: bool = wait_until_free(address, STOP_BOUND);
        return match (ended, free) {
            (crate::service_ended_through_a_pinned_handle::Ended::Sent, true) | (crate::service_ended_through_a_pinned_handle::Ended::Gone, true) => Action::Exit {
                code: 0,
                stdout: format!(
                    "The commit service of this worktree (instance {instance}, pid {pid}) was asked to leave and did not within {} seconds, so it was ended and its address is free. The batch it held is in its journal, and the next service recovers it.\n",
                    STOP_BOUND.as_secs()
                ),
                stderr: String::new(),
            },
            _ => Action::Exit {
                code: 1,
                stdout: String::new(),
                stderr: format!(
                    "{COMMAND}: the commit service of this worktree (instance {instance}, pid {pid}) was asked to leave and did not within {} seconds, and it was ended ({ended:?}) but its address is still taken; look at {} and at the service's log beside it.\n",
                    STOP_BOUND.as_secs(),
                    crate::service_server::address_text(address)
                ),
            },
        };
    }
    return Action::Exit {
        code: 1,
        stdout: String::new(),
        stderr: format!(
            "{COMMAND}: the commit service of this worktree (instance {instance}) was asked to leave and is still there after {} seconds, and the note beside its address ({}) names no process that is still running, so this command cannot end it. Look at what holds that address and end that process.\n",
            STOP_BOUND.as_secs(),
            note.display()
        ),
    };
}

/// What:
///  Waits, bounded, until nothing takes a connection at the address.
/// Why:
///   A service that was asked to leave finishes the batch it holds first,
/// and one that was ended stops taking connections a moment later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitUntilFree(address, bound) // true when nothing accepts
/// ```
fn wait_until_free(address: &Bind, bound: Duration) -> bool {
    let deadline: Instant = Instant::now() + bound;
    loop {
        if !accepts(address) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(POLL);
    }
}

/// Controls of the command stay out of the release executable.
#[cfg(test)]
#[path = "management_service_stop_tests.rs"]
mod tests;
