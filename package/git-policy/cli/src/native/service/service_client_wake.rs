//! What:
//!  The client's one-way message that work items were written for a repository,
//!  and its wait for the service to take it.
//! Why:
//!   The wrapper never starts the service or runs the work itself,
//!  so this message is all the service needs to look at the repository's queue;
//! it travels beside the commit request in `service_client.rs`,
//!  which re-exports it so a caller names one module.
//!   A push record is a file and outlives the command that wrote it,
//!  so the programs of that command travel with this message and live only in the service's memory
//!  (`service_push_programs_held_in_memory`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await wake(config, gitDir, reviewKey, pushPrograms);
//! ```

use crate::commit_timing::ReachTrace;
use crate::diagnostic_log::debug;
use crate::service_client_listen_and_ask_to_retire::{Wrote, read_message, write_within};
use crate::service_client_reach_service::reach_current;
use crate::service_client_settings_and_watches::ClientConfig;
use crate::service_messages::ClientFailure;
use crate::service_protocol::{ClientMessage, encode_client};
use crate::service_review_key::ReviewKey;
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceClientWake";

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
    let mut line: String = encode_client(&ClientMessage::Work { git_dir: git_dir.to_path_buf(), review_key: review_key.cloned(), push_programs: push_programs.to_vec() });
    line.push('\n');
    let mut announced: bool = false;
    let mut reader: BufReader<UnixStream> = reach_current(config, notices, &mut announced, &mut ReachTrace::default(), false)?;
    // The one line the wake carries is written under the bounds of any other, so a service that stopped reading costs the
    //  confirmation and never the command.
    let written: Wrote = write_within(reader.get_ref(), line.as_bytes(), crate::wait_ceilings::REQUEST_WRITE_SILENCE, crate::wait_ceilings::REQUEST_WRITE_CEILING);
    if written == Wrote::Ended {
        return Err(ClientFailure::Unavailable);
    }
    // The service closes the connection after it has read the line, and that close is the confirmation.
    let closed_by: Instant = Instant::now() + config.heartbeat_silence;
    loop {
        let left: Duration = closed_by.saturating_duration_since(Instant::now());
        if left.is_zero() {
            debug(TAG, format!("the service did not close the connection within {:?} of the line being written", config.heartbeat_silence).as_str());
            break;
        }
        // No read waits past the bound, whatever the service sends.
        let _timeout = reader.get_ref().set_read_timeout(Some(left));
        if read_message(&mut reader).is_none() {
            break;
        }
    }
    return Ok(());
}
