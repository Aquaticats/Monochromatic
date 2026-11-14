//! What:
//!  The client's one-way request to a commit service that already runs:
//! compile this repository's forbidden-strings rules.
//! Why:
//!  A checkout,
//! merge or pull that changes the rules text leaves the next commit to compile it,
//! 1.7 s of processor time and about
//! 20 s on a fresh cache under load.
//! The service compiles on its own time (`landing/service_rules_precompile.rs`),
//! but this request must never start a service of its own:
//! the client writes no unit,
//! asks no manager for a start,
//! and starts no process (the owner:
//! "We should never invoke ourselves recursively"),
//! so the service's process is looked for before the connection is made,
//! by its start arguments (`service run`,
//! `--instance <instance>`),
//! among the processes of this user.
//!  What that look cannot promise is a start by the manager:
//! the socket unit keeps listening after its service was killed,
//! so a service killed between the look and the connection is started by the connection itself,
//! which is what socket activation is for.
//! No check before a connection can be one with it,
//! and nothing after it can undo the start,
//! so this file promises the look and the two ceilings,
//! never that no service ever starts.
//!  Both steps are bounded:
//! the look is made within `WARM_SCAN_CEILING`,
//! because reading another process's command line takes that process's address-space lock,
//! which a process stuck in the kernel holds without end;
//! and the connection is made within `WARM_REQUEST_CONNECT_CEILING`,
//! because the standard library's connect waits in the kernel while the listening socket's queue of unaccepted
//! connections is full,
//! and a service stopped inside its own process accepts nothing,
//! so that wait has no end.
//! The request is a help and never a part of the command that makes it:
//! a request that cannot be written is not sent,
//! and the command compiles the rules itself (`policy/policy_forbidden_strings.rs`,
//! `cache_access`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (serviceRunsWithin(WARM_SCAN_CEILING, instanceOf(gitDir))) { connectWithinFor(Bind.Path(socketFor(runtime, gitDir)), WARM_REQUEST_CONNECT_CEILING).write(encode({ type: 'warm', gitDir, repository })); }
//! ```

use crate::service_client_connect_within_a_bound::connect_within_for;
use crate::service_route::non_empty_path_variable;
use crate::service_instance::{instance_of, socket_for};
use crate::service_protocol_encode_messages_to_lines::encode_client;
use crate::service_protocol_messages_and_name_checks::ClientMessage;
use crate::service_server::Bind;
use crate::wait_ceilings::WARM_REQUEST_CONNECT_CEILING;
#[cfg(unix)]
use crate::earlier_service_command_line_read::{CommandLine, command_line_of};
#[cfg(unix)]
use crate::earlier_service_process_look::effective_user;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Whether a process of the table is the commit service of `instance`:
/// its start arguments hold `service`,
/// `run`,
/// and
///  `--instance` followed by the instance.
fn is_service_of(arguments: &[&[u8]], instance: &str) -> bool {
    let mut previous: Option<&[u8]> = None;
    let mut after_run: bool = false;
    let mut has_service: bool = false;
    let mut has_instance: bool = false;
    for argument in arguments {
        if *argument == b"service" {
            has_service = true;
        }
        if has_service && *argument == b"run" {
            after_run = true;
        }
        if previous == Some(b"--instance".as_slice()) && *argument == instance.as_bytes() {
            has_instance = true;
        }
        previous = Some(argument);
    }
    return has_service && after_run && has_instance;
}

/// Whether a byte is an ASCII digit.
fn is_digit(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Whether a byte is the separator of a process's start arguments.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether the service of `instance` runs,
///  from the start arguments of the processes of this user under `proc_root`,
///  for no longer than `WARM_SCAN_CEILING`.
/// Why:
///   A look that cannot answer inside its ceiling is a request that is not made:
/// the command compiles the rules itself,
/// which is what the request being a help means.
///  The read that can hold the look is another process's command line,
/// which takes that process's address-space lock
/// (`landing/earlier_service_process_look.rs`,
/// `wait_ceilings.rs` `PROCESS_LOOK_CEILING`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runs = workWithin(WARM_SCAN_CEILING, () => scanProc(instance, procRoot)) ?? false;
/// ```
pub fn service_process_runs(instance: &str, proc_root: &Path) -> bool {
    return service_runs_in(instance, proc_root);
}

/// What:
///  The scan itself, and the answer a control may hold at
///  (`service_warm_request_control_window.rs`).
/// Why:
///   A control needs an instant between the answer and the connection,
/// and the ceiling has to cover that instant,
/// so both sit inside the work the ceiling wraps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = aServiceOfTheInstanceRuns(instance, procRoot); return theAnswerWaitsWhereAControlSays(found);
/// ```
fn service_runs_in(instance: &str, proc_root: &Path) -> bool {
    let found: bool = a_service_of_the_instance_runs(instance, proc_root);
    return the_controls_window::the_answer_waits_where_a_control_says(found);
}

/// What:
///  Whether any process of this user under `proc_root` has the start arguments of the service of `instance`.
/// Why:
///   The command line is read through the package's own reader,
/// which asks the kernel about the owner before it reads and reads no more than any program can carry
/// (`landing/earlier_service_command_line_read.rs`),
/// so another user's process is neither waited for nor taken for this user's service,
/// and one process cannot fill memory with its arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of readdir(procRoot)) { if (isServiceOf(commandLineOf(entry, ownUser), instance)) return true } return false
/// ```
#[cfg(unix)]
fn a_service_of_the_instance_runs(instance: &str, proc_root: &Path) -> bool {
    let own_user: u32 = effective_user();
    let Ok(listing) = std::fs::read_dir(proc_root) else { return false };
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.is_empty() || !name.bytes().all(is_digit) {
            continue;
        }
        let Ok(pid) = name.parse::<u32>() else { continue };
        let CommandLine::Words(bytes) = command_line_of(&entry, pid, own_user) else { continue };
        let words: Vec<&[u8]> = bytes.split(is_nul).collect();
        if is_service_of(words.as_slice(), instance) {
            return true;
        }
    }
    return false;
}

/// A platform whose processes cannot be looked at this way answers that no service runs,
/// and the command compiles the rules itself.
#[cfg(not(unix))]
fn a_service_of_the_instance_runs(_instance: &str, _proc_root: &Path) -> bool {
    return false;
}

/// The instant a control may hold the answer at,
/// which a request without the control's variable passes at once.
#[path = "service_warm_request_control_window.rs"]
pub mod the_controls_window;

/// What:
///  Asks the commit service of the worktree whose own Git directory is `git_dir` to compile the rules of `repository`,
/// when that
///  service runs;
/// returns whether the request was sent.
/// Why:
///   Nothing is waited for beyond the two ceilings:
/// the line is written and the connection dropped,
/// and every failure is only a request that was not sent.
///  The look at the processes is made within `WARM_SCAN_CEILING`
/// and the connection within `WARM_REQUEST_CONNECT_CEILING`,
/// so a process stuck in the kernel and a service whose queue of unaccepted connections is full each cost a ceiling and never
/// the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// askRunningServiceToWarm(gitDir, repository) // boolean
/// ```
pub fn ask_running_service_to_warm(git_dir: &Path, repository: &Path) -> bool {
    let Some(runtime) = non_empty_path_variable("XDG_RUNTIME_DIR") else { return false };
    let socket: PathBuf = socket_for(runtime.as_path(), git_dir);
    if !socket.exists() || !service_process_runs(instance_of(git_dir).as_str(), Path::new("/proc")) {
        return false;
    }
    return ask_the_service_at(socket.as_path(), git_dir, repository);
}

/// What:
///  Writes the one warm line into the service at `socket`,
///  and says whether it was written.
/// Why:
///   A connect waits in the kernel while the listening socket's queue of unaccepted connections is full,
///  and a service stopped inside its own process accepts nothing,
///  so the connection is where this request must not wait without an end:
///  it is made with the ceiling that belongs to a request whose answer the command does not need
///  (`wait_ceilings.rs`,
///  `WARM_REQUEST_CONNECT_CEILING`).
///  The line that follows waits for nothing either:
///  it is a few hundred bytes into a socket whose send buffer is 212,992 bytes
///  (`wait_ceilings.rs`,
///  `REQUEST_WRITE_SILENCE`),
///  so it is taken even by a service that reads nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// askTheServiceAt(socket, gitDir, repository) // boolean
/// ```
fn ask_the_service_at(socket: &Path, git_dir: &Path, repository: &Path) -> bool {
    let Ok(mut connection) = connect_within_for(&Bind::Path(socket.to_path_buf()), WARM_REQUEST_CONNECT_CEILING) else { return false };
    let mut line: String = encode_client(&ClientMessage::Warm { git_dir: git_dir.to_path_buf(), repository: repository.to_path_buf() });
    line.push('\n');
    return connection.write_all(line.as_bytes()).is_ok();
}

/// The start arguments are read as controls in the test module.
#[cfg(test)]
#[path = "service_warm_request_tests.rs"]
mod tests;
