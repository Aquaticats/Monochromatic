//! What:
//!  The client's one-way message that work items were written for a repository,
//!  and its wait for the service to take it,
//!  and the one-way message that names the push records that can no longer run.
//! Why:
//!   The wrapper never starts the service or runs the work itself,
//!  so this message is all the service needs to look at the repository's queue;
//! it travels beside the commit request in `service_client.rs`,
//!  which re-exports it so a caller names one module.
//!   A push record is a file and outlives the command that wrote it,
//!  so the programs of that command travel with this message and live only in the service's memory
//!  (`service_push_programs_held_in_memory`).
//!   The message that gives those programs up asks nothing of a service that is not already running:
//!  it is told to the one process that can hold the values,
//!  and a command that settles a record must not start a service to tell it nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await wake(config, gitDir, reviewKey, pushPrograms); await forgetPrograms(config, gitDir, recordIds);
//! ```

use crate::commit_timing::ReachTrace;
use crate::diagnostic_log::debug;
use crate::service_client_listen_and_ask_to_retire::{Heard, Reached, Wrote, listen, reach, read_message, write_within};
use crate::service_client_reach_service::reach_current;
use crate::service_client_settings_and_watches::ClientConfig;
use crate::service_messages::ClientFailure;
use crate::service_protocol::{ClientMessage, PROTOCOL_VERSION, ServiceMessage, encode_client};
use crate::service_review_key::ReviewKey;
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceClientWake";

/// The manager's own word for a unit that is running now.
const RUNNING: &str = "active";

/// What:
///  Tells the service that work items were written for a repository,
/// starting it through the operating system when it
///  is not running,
/// and returns once the service has taken the message or the silence bound has passed since it was
///  written.
/// Why:
///   The wrapper never starts the service or runs the work itself;
/// the message is the only thing the service needs to
///  look at the repository's work directory.
/// The connection closes when the service has read the line,
/// which is the usual
///  confirmation that it will act,
/// and the wait for that close ends at the silence bound whatever the service sends:
///  a service that keeps answering resets the silence of every single read,
/// so a loop that only reads waits for it
///  without end,
/// and the command that woke it waits behind it (owner,
/// 2026-11-03:
/// "nothing should ever wait forever or
///  indefinitely").
/// What is lost by reaching the bound is only the confirmation:
/// the line was written,
/// and the record
///  it names is already durable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await wake(config, gitDir, reviewKey, pushPrograms)
/// ```
pub fn wake(
    config: &ClientConfig,
    git_dir: &std::path::Path,
    review_key: Option<&ReviewKey>,
    push_programs: &[(String, crate::service_push_programs_held_in_memory::PushPrograms)],
    notices: &mut dyn Write,
) -> Result<(), ClientFailure> {
    return send_one(config, &ClientMessage::Work { git_dir: git_dir.to_path_buf(), review_key: review_key.cloned(), push_programs: push_programs.to_vec() }, notices);
}

/// What:
///  Tells the service that the push records `records` can no longer run,
///  so it gives up the programs and transport settings it holds against them,
///  and returns once the service has taken the message or the silence bound has passed.
/// Why:
///   A record the committing wrapper settles or removes after its post-commit gate is not the service's to notice:
/// nothing is queued for it and no round looks at it,
///  while the values it was given live in the service's memory alone,
/// and one of them can hold a password
///  (`service_push_programs_held_in_memory`).
/// The message is best effort,
/// as the wake is:
/// a service of an earlier release holds no values and refuses a message it does not know,
/// and one this command cannot reach holds none of them either.
/// It is told only to a service that is already running:
/// a connection to the socket of a socket-activated unit is what asks the manager to start the service,
/// and the values it would give up live in that service's memory and nowhere else,
/// so a start this message caused would run a commit service to be told nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (managerState(unit) === 'active') await tellRunningService({ type: 'forget-programs', gitDir, recordIds });
/// ```
pub fn forget_programs(config: &ClientConfig, git_dir: &std::path::Path, records: &[String]) -> Result<(), ClientFailure> {
    // Nothing to forget costs no question and no connection, and most commands that look at a repository settle no record at all.
    if records.is_empty() || !a_service_is_running(config) {
        return Ok(());
    }
    return send_to_a_running_service(config, &ClientMessage::ForgetPrograms { git_dir: git_dir.to_path_buf(), records: records.to_vec() });
}

/// What:
///  Whether this worktree's service is running now,
///  asked of the manager and never of the service.
/// Why:
///   The manager's `ActiveState` is the one answer about a service that starts nothing;
///  a connection is what asks a socket-activated unit to start,
///  so the question has to be answered before one is made.
/// Where no manager can be asked, nothing on that platform starts a service for a connection,
///  so the connection is made and finds out.
/// A start between this answer and the connection is possible and is left as it is:
///  it is a service that stopped in that moment, whose values went with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const serviceIsRunning = (config) => !config.managerWatch || config.managerWatch.state(config.managerWatch.bus, config.managerWatch.unit) === 'active';
/// ```
fn a_service_is_running(config: &ClientConfig) -> bool {
    let Some(watch) = config.start_watch.as_ref() else { return true };
    return (watch.state)(watch.bus.as_path(), watch.unit.as_str()).state() == Some(RUNNING);
}

/// What:
///  Writes one client message into a service that is already running,
///  and returns once it has taken the line,
///  when it greets nothing,
///  or at the silence bound.
/// Why:
///   Nothing here waits for a start:
/// the message carries no work a person waits for,
///  and a service that is not there holds none of the values it would give up.
/// A greeting that names no protocol this release writes ends it,
///  because an earlier service would refuse the line and only say so in its own log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stream = connect(address); if (greeting(stream)?.version !== PROTOCOL_VERSION) return; writeWithin(stream, line); await closed(stream);
/// ```
fn send_to_a_running_service(config: &ClientConfig, message: &ClientMessage) -> Result<(), ClientFailure> {
    let mut line: String = encode_client(message);
    line.push('\n');
    let mut reader: BufReader<UnixStream> = match reach(&config.address) {
        Reached::Ours(stream) => {
            let _timeout = stream.set_read_timeout(Some(config.start_wait));
            BufReader::new(stream)
        }
        // Nothing listens at the address, or a process of another account took the connection: neither is this worktree's service.
        Reached::Nothing | Reached::OtherAccount => return Ok(()),
    };
    match listen(&mut reader) {
        Heard::Message(ServiceMessage::Welcome { version, .. }) if version == PROTOCOL_VERSION => {}
        Heard::Message(_) | Heard::Closed | Heard::Silent => return Ok(()),
    }
    // The one line the message carries is written under the bounds of any other, so a service that stopped reading costs the
    //  confirmation and never the command.
    let written: Wrote = write_within(reader.get_ref(), line.as_bytes(), crate::wait_ceilings::REQUEST_WRITE_SILENCE, crate::wait_ceilings::REQUEST_WRITE_CEILING);
    if written == Wrote::Ended {
        return Err(ClientFailure::Unavailable);
    }
    wait_for_the_close(config, &mut reader);
    return Ok(());
}

/// What:
///  Writes one client message and waits for the service to take it,
///  no longer than the silence bound,
///  starting the service when it is not running.
/// Why:
///   The wake is the message that may start a service,
/// and the close of the connection is the usual confirmation
///  that the service read the line (`wake`).
/// The message that gives a record's programs up asks a service that already runs (`forget_programs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = encodeClient(message) + '\n'; const reader = await reachCurrent(config); writeWithin(reader, line); await closed(reader);
/// ```
fn send_one(config: &ClientConfig, message: &ClientMessage, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let mut line: String = encode_client(message);
    line.push('\n');
    let mut announced: bool = false;
    let mut reader: BufReader<UnixStream> = reach_current(config, notices, &mut announced, &mut ReachTrace::default(), false)?;
    // The one line the message carries is written under the bounds of any other, so a service that stopped reading costs the
    //  confirmation and never the command.
    let written: Wrote = write_within(reader.get_ref(), line.as_bytes(), crate::wait_ceilings::REQUEST_WRITE_SILENCE, crate::wait_ceilings::REQUEST_WRITE_CEILING);
    if written == Wrote::Ended {
        return Err(ClientFailure::Unavailable);
    }
    // The service closes the connection after it has read the line, and that close is the confirmation.
    wait_for_the_close(config, &mut reader);
    return Ok(());
}

/// What:
///  Waits, no longer than the silence bound, for the service to close the connection it was written on,
///  which is its confirmation that it read the line.
/// Why:
///   A service that keeps answering resets the silence of every single read,
///  so a loop that only reads waits without end and the command waits behind it;
/// the close is what this waits for and nothing else, and what is lost by reaching the bound is only the confirmation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const closedBy = now() + silence; while (now() < closedBy && read(stream) !== undefined) {}
/// ```
fn wait_for_the_close(config: &ClientConfig, reader: &mut BufReader<UnixStream>) {
    let closed_by: Instant = Instant::now() + config.heartbeat_silence;
    loop {
        let left: Duration = closed_by.saturating_duration_since(Instant::now());
        if left.is_zero() {
            debug(TAG, format!("the service did not close the connection within {:?} of the line being written", config.heartbeat_silence).as_str());
            break;
        }
        // No read waits past the bound, whatever the service sends.
        let _timeout = reader.get_ref().set_read_timeout(Some(left));
        if read_message(reader).is_none() {
            break;
        }
    }
}

/// The controls of what each message waits for and whom it asks stay out of the release executable.
#[cfg(test)]
#[path = "service_client_wake_tests.rs"]
mod tests;
