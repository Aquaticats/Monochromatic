//! What:
//!  What `... service status` answers about the commit service of a worktree:
//!  what a socket of the service did when the status asked it,
//!  the words for each of those answers,
//!  whether any service greets at a runtime directory,
//!  and the report line itself,
//!  with the unit that serves the worktree.
//! Why:
//!   A socket that takes a connection is not a service that answers,
//!  and the status is what the message of a failed commit sends a person to,
//!  so it tells a service that works from one that is running and has stopped inside itself.
//!  They live beside the route that names the address,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceRouteWhatTheStatusAnswers.ts
//! type Answered = 'greeted' | 'said-nothing' | 'nothing';
//! export const serviceStatus = (report: StatusReport): string => ...;
//! ```

use crate::service_address_of_a_worktree::socket_address_of;
use crate::service_client_listen_and_ask_to_retire::Reached;
use crate::service_instance::{
    SOCKET_DIRECTORY,
    service_unit_instance,
    try_instance_of,
};
use crate::service_server::Bind;
use crate::wait_ceilings::HAND_OVER_GREETING_WAIT;
use std::path::{
    Path,
    PathBuf,
};

/// What `git monochromatic-git-policy-cli service status` prints and exits with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    /// Exit status:
    /// 0 when the service answered.
    pub code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// What a socket of the commit service did when the status asked it.
/// Why:
///  A socket that takes a connection is not a service that answers:
/// the status is what the message of a failed commit sends a person to,
/// so it tells a service that works from one
///  that is running and has stopped inside itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answered = 'greeted' | 'said-nothing' | 'nothing'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Answered {
    /// A service took the connection and sent its greeting.
    Greeted,
    /// Something took the connection and sent nothing within the bound.
    SaidNothing,
    /// Nothing took the connection.
    Nothing,
}

/// What:
///  What the commit service at `address` does when it is asked,
/// waited for no longer than the wait a hand-over grants a greeting.
/// Why:
///   A service greets a connection before it restores anything,
/// so a connection taken and greeted is a service that runs,
/// and a connection taken and never greeted is a program that stopped inside itself
/// (`wait_ceilings.rs`,
/// `HAND_OVER_GREETING_WAIT`).
/// The address is the one the commit itself uses, a socket file or an abstract name (`service_address_of_a_worktree.rs`),
/// and a connection another account's process took is no service of this command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = asking(address); // 'greeted' | 'said-nothing' | 'nothing'
/// ```
fn asking(address: &Bind) -> Answered {
    let Reached::Ours(stream) = crate::service_client_listen_and_ask_to_retire::reach(address)
    else {
        // Nothing took the connection, or a process of another account did, which is no service of this command.
        return Answered::Nothing;
    };
    let _timeout = stream.set_read_timeout(Some(HAND_OVER_GREETING_WAIT));
    let mut reader: std::io::BufReader<std::os::unix::net::UnixStream> =
        std::io::BufReader::new(stream);
    return match crate::service_client_listen_and_ask_to_retire::listen(&mut reader) {
        crate::service_client_listen_and_ask_to_retire::Heard::Message(_) => Answered::Greeted,
        crate::service_client_listen_and_ask_to_retire::Heard::Silent => Answered::SaidNothing,
        crate::service_client_listen_and_ask_to_retire::Heard::Closed => Answered::Nothing,
    };
}

/// What:
///  Whether any commit service of the account greets a connection at a runtime directory.
/// Why:
///   Without a worktree to ask about,
/// any service of the account counts,
/// and one that takes a connection and says nothing does not.
/// A service holds either a socket file under the folder of the sockets or, where that folder's path is too long for one
/// socket address, an abstract name, which has no file to be found by
/// (`service_address_of_a_worktree.rs`).
/// The instances of an account are the ones the notes name, which provisioning writes for every instance and the sweep
/// reads for the same reason (`instances_with_units`), and each names its own address;
/// the files still answer for an instance whose note is gone, which a process killed in provisioning leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdir(directory).some((name) => name.endsWith('.sock') && asking(join(directory, name)) === 'greeted')
///   || instancesWithUnits(runtime).some((instance) => asking(socketAddressOf(runtime, instance)) === 'greeted')
/// ```
fn any_service_greets(runtime_dir: &Path) -> bool {
    // What: A folder of sockets that is not there holds no socket file, and is not the end of the question.
    // Why:  An account served at abstract names has no such folder until something makes it (the fixture of a control does,
    //       a person's runtime directory of a long path never does), and the names below are where its services are found.
    if let Ok(entries) = std::fs::read_dir(runtime_dir.join(SOCKET_DIRECTORY)) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let is_socket_file: bool = match path.extension() {
                Some(extension) => extension == "sock",
                None => false,
            };
            if is_socket_file && asking(&Bind::Path(path)) == Answered::Greeted {
                return true;
            }
        }
    }
    // What: The names this runtime directory's services hold, asked at each of them.
    // Why:  A name has no file to be found by, and the table of unix sockets is where it is visible without connecting;
    //       `listening_names_of` reads only the names whose account and place are this runtime directory's, so a service of
    //       another account, or of another runtime directory, is never counted. A path-form instance is the socket file
    //       asked above and is not in that list.
    for name in
        crate::service_address_of_a_worktree::listening_names_of(runtime_dir, Path::new("/proc"))
    {
        if asking(&Bind::Abstract(name)) == Answered::Greeted {
            return true;
        }
    }
    return false;
}

/// What:
///  Says what the commit service of a worktree does,
///  names the unit that serves it,
///  and gives the step that ends a service which stopped inside itself.
/// Why:
///   A diagnostic a person reaches for when a commit failed:
/// connecting is what a commit does,
/// so a service the manager would start on demand is started by this check too,
/// and the unit its restart
///  names is the one of this worktree and no other.
/// A restart of every service of the account is what the failure this check answers must never ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceStatus(runtimeDir, gitDir)
/// ```
pub fn service_status(
    runtime_dir: Option<&Path>,
    git_dir: Option<&Path>,
) -> StatusReport {
    let what: &str = if git_dir.is_some() {
        "The commit service of this worktree"
    } else {
        "A commit service"
    };
    let instance: Option<String> = match (runtime_dir, git_dir) {
        (Some(_runtime), Some(git)) => match try_instance_of(git) {
            Ok(found) => Some(found),
            Err(unreachable) => {
                return StatusReport {
                    code: 1,
                    stdout: String::new(),
                    stderr: format!(
                        "The commit service of this worktree cannot be asked: {unreachable}. Nothing was connected to or started; \
                         if the repository is on a drive that is not mounted, mount it and run this again.\n"
                    ),
                };
            }
        },
        _ => None,
    };
    let answered: Answered = match (runtime_dir, instance.as_deref()) {
        (Some(runtime), Some(found)) => asking(&socket_address_of(runtime, found)),
        (Some(runtime), None) => {
            if any_service_greets(runtime) {
                Answered::Greeted
            } else {
                Answered::Nothing
            }
        }
        (None, _) => Answered::Nothing,
    };
    let unit: String = match instance.as_deref() {
        Some(found) => format!(" Its unit is `{}`.", service_unit_instance(found)),
        None => String::new(),
    };
    return match answered {
        Answered::Greeted => StatusReport {
            code: 0,
            stdout: format!("{what} is running and answers.{unit}\n"),
            stderr: String::new(),
        },
        Answered::SaidNothing => StatusReport {
            code: 1,
            stdout: String::new(),
            stderr: format!(
                "{what} took a connection and sent nothing for {} seconds, so it is running and has stopped inside itself: it was \
                 frozen, a debugger holds it, or it waits for a disk that never answers. Starting it again does not help, because the \
                 service manager believes it runs.{unit} Restart that unit with `systemctl --user restart` and the unit name printed here, \
                 then run the commit again.\n",
                HAND_OVER_GREETING_WAIT.as_secs()
            ),
        },
        Answered::Nothing => StatusReport {
            code: 1,
            stdout: String::new(),
            stderr: format!(
                "{what} does not answer here. It sets itself up the first time a commit needs it, so there is nothing to \
                 install; if a commit fails, its message says what to do.{unit}\n"
            ),
        },
    };
}
