//! What:
//!  The client's one-way request to a commit service that already runs:
//! compile this repository's forbidden-strings rules.
//! Why:
//!  A checkout,
//! merge or pull that changes the rules text leaves the next commit to compile it,
//! 1.7 s of processor time and about
//! 20 s on a fresh cache under load.
//! The service compiles on its own time (`landing/service_rules_precompile.rs`),
//! but only a service that runs may be asked:
//! the client never starts a service for this (the owner:
//! "We should never invoke ourselves recursively"),
//! and the socket of a
//! stopped service is socket-activated,
//! so a connection to it would start one.
//! The service's process is therefore looked for before the connection is made,
//! by its start arguments (`service run`,
//! `--instance <instance>`),
//! among the processes of the same user.
//!  The connection is made within `WARM_REQUEST_CONNECT_CEILING`,
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
//! if (serviceRuns(instanceOf(gitDir))) { connectWithinFor(Bind.Path(socketFor(runtime, gitDir)), WARM_REQUEST_CONNECT_CEILING).write(encode({ type: 'warm', gitDir, repository })); }
//! ```

use crate::service_client_connect_within_a_bound::connect_within_for;
use crate::service_route::non_empty_path_variable;
use crate::service_instance::{instance_of, socket_for};
use crate::service_protocol_encode_messages_to_lines::encode_client;
use crate::service_protocol_messages_and_name_checks::ClientMessage;
use crate::service_server::Bind;
use crate::wait_ceilings::WARM_REQUEST_CONNECT_CEILING;
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

/// Whether the service of `instance` runs,
/// from the start arguments of the processes under `proc_root`.
pub fn service_process_runs(instance: &str, proc_root: &Path) -> bool {
    let Ok(listing) = std::fs::read_dir(proc_root) else { return false };
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.is_empty() || !name.bytes().all(is_digit) {
            continue;
        }
        let Ok(bytes) = std::fs::read(entry.path().join("cmdline")) else { continue };
        let words: Vec<&[u8]> = bytes.split(is_nul).collect();
        if is_service_of(words.as_slice(), instance) {
            return true;
        }
    }
    return false;
}

/// What:
///  Asks the commit service of the worktree whose own Git directory is `git_dir` to compile the rules of `repository`,
/// when that
///  service runs;
/// returns whether the request was sent.
/// Why:
///   Nothing is waited for:
/// the line is written and the connection dropped,
/// and every failure is only a request that was not sent.
///  The connection is made within `WARM_REQUEST_CONNECT_CEILING`,
/// so a service whose queue of unaccepted connections is full costs that bound and never the command.
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
