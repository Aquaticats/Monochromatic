//! What:
//!  Reaches the commit service:
//! connects to its address,
//! starts it through its unit when nothing answers,
//! and waits for a
//!  service of the current release.
//! Why:
//!  Submitting a request or a withdrawal begins by reaching a service that speaks this release;
//! the steps have their own file
//!  so the submit loop stays readable.
//!
//! What the wait learns,
//! and what it says when it ends
//! (`service_messages_of_a_start_that_was_asked_and_did_not_come.rs`):
//!  every answer of the manager at a poll,
//!  what this command does about it,
//!  what it remembers,
//!  and the words a person reads at the ceiling.
//! Two facts are remembered,
//!  and every sentence is one clause each:
//!  the manager's last answer
//!  (`State` with its own word,
//!  `NotHeld`,
//!  `Silent`),
//!  and what this command asked the manager to do
//!  (`Nothing`,
//!  `Start`,
//!  `ResetAndStart`,
//!  `Restart`).
//! "Asked" means a call this command made of the manager,
//!  never a connection:
//!  connecting to the socket of a socket-activated unit is what asks the manager to start it,
//!  the client cannot tell whose connection asked,
//!  and the manager's own `activating` is the fact that a start is under way,
//!  so no sentence claims who started the unit.
//! The cells,
//! with the words each produces
//!  (`<u>` is the unit this command asked about):
//!
//! - `activating`,
//!   nothing asked:
//!   "the service manager says <u> is activating,
//!   so a start of it is under way,
//!   and this command asked it for nothing itself".
//!   This is the cell a socket-activated start leaves,
//!   and it read "nothing asked it to start that unit" before.
//! - `activating`,
//!   after a start was asked for a unit the manager held no such unit:
//!   the same "a start of it is under way",
//!   "and this command asked it to start <u>".
//! - `inactive`:
//!   a start is asked at once,
//!   and the sentence says the manager's word and that this command asked for the start.
//! - not held:
//!   a start is asked,
//!   and the sentence says the manager answered that it holds no such unit
//!   and that this command asked for the start.
//! - `failed`:
//!   the failed state is forgotten and a start asked,
//!   once,
//!   and a second `failed` answer ends the wait with that cell's words.
//! - `active` with nothing greeting:
//!   one restart is asked at the ceiling,
//!   and the sentence says the unit is active and that this command asked for the restart.
//! - `deactivating`,
//!   or any other state:
//!   nothing is asked,
//!   and the manager's word is what the person reads.
//! - unreachable:
//!   nothing is asked and the wait ends at the start wait,
//!   and the sentence says the manager gave no answer and that this command asked for nothing itself.
//! - no manager can be asked at all (`start_watch` absent):
//!   "no service manager could be asked about this worktree's commit service,
//!   so nothing started it".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reader = await reachCurrent(config, notices, announced, trace, serviceSeen);
//! ```

use super::service_client_reach_service_the_waits_own_steps::reach;
use crate::commit_timing::ReachTrace;
use crate::service_client_settings_and_watches::{
    CONNECT_PAUSE,
    ClientConfig,
};
use crate::service_messages::{
    ClientFailure,
    UPDATING_NOTICE,
    replacing_notice,
};
use crate::service_protocol::{
    PROTOCOL_VERSION,
    RELEASE,
    encode_retire,
};
use std::io::{
    BufReader,
    Write,
};
use std::os::unix::net::UnixStream;
use std::time::Instant;

/// What:
///  Reaches a service that speaks this wrapper's protocol and was started from this wrapper's release,
/// asking any other to
///  retire first.
/// Why:
///   The manager keeps its socket across an upgrade or a downgrade,
/// so a service from the other release may answer:
/// one
///  whose program was since removed,
/// or one that behaves as the other release did.
/// It is asked to finish its batch and
///  exit,
/// and the next connection makes the manager start the program the rewritten unit names.
/// The asking is bounded by
///  the start wait from the first ask (the same wait the manager's start gets),
/// because the retiring service still accepts connections until it has exited,
/// and
///  each connection it accepts in that time is asked again.
/// A service of a newer protocol cannot be asked (it refuses a
///  message it cannot read as its own),
/// so that ends the command.
/// A service that is still there at the bound it was given is ended once there,
/// so the manager starts the program this release installs,
/// and the wait goes on under the start ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = await reachCurrent(config, notices, announced)
/// ```
pub(crate) fn reach_current(
    config: &ClientConfig,
    notices: &mut dyn Write,
    announced: &mut bool,
    trace: &mut ReachTrace,
    service_seen: bool,
) -> Result<BufReader<UnixStream>, ClientFailure> {
    let began: Instant = Instant::now();
    let reached = reach_current_untimed(config, notices, announced, trace, service_seen);
    trace.elapsed += began.elapsed();
    return reached;
}

/// `reach_current` without the clock around it.
pub(crate) fn reach_current_untimed(
    config: &ClientConfig,
    notices: &mut dyn Write,
    announced: &mut bool,
    trace: &mut ReachTrace,
    service_seen: bool,
) -> Result<BufReader<UnixStream>, ClientFailure> {
    let mut give_up_at: Option<Instant> = None;
    // Whether the service of another release that outlasted the bound was already ended, which happens once.
    let mut restarted: bool = false;
    loop {
        let (mut reader, version, release) =
            reach(config, notices, announced, trace, service_seen)?;
        if version == PROTOCOL_VERSION && release.as_deref() == Some(RELEASE) {
            trace.answered_by = release;
            return Ok(reader);
        }
        if version > PROTOCOL_VERSION {
            return Err(ClientFailure::NoCommonVersion);
        }
        // The give-up time is fixed the first time an other-release service is met, and the person is told then.
        let deadline: Instant = match give_up_at {
            Some(fixed) => fixed,
            None => {
                let _written = writeln!(notices, "{UPDATING_NOTICE}");
                // What: Nothing is done to the manager here but the ending at the bound below.
                // Why:  Another release's answer is repaired by that service's own retirement: it finishes its batch and
                //       exits, the manager unloads the unit once nothing runs in it, and the next start reads the unit file as
                //       it is now. A manager asked to re-read every unit it holds would stall for every other client of it,
                //       which is the operation no Git command may ask for
                //       (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
                let fresh: Instant = Instant::now() + config.start_wait;
                give_up_at = Some(fresh);
                fresh
            }
        };
        if Instant::now() >= deadline {
            // What: A service of another release that is still there at the bound is ended once,
            //       said once,
            //       and waited for again.
            // Why:  The bound is the commit's, and a service that finishes a batch or a push it planned to try again outlasts
            //       it; the commit that ended here left the person's change uncommitted (five of six changes of release in the
            //       two-build reproduction of 2026-11-12, 10,032 to 10,052 ms of a wait that then failed, and 1 of 16 on
            //       another agent's fixture). Ending it lets the manager start the program this release installs, because the
            //       unit names the program link rather than an executable (`service_instance.rs`, `program_link`). One ending
            //       only: a unit the manager starts from a text an upgrade left behind comes back as the other release again,
            //       and the next commit's provisioning is what repairs that (`service_provision_instance_unloaded.rs`).
            //       Ending a unit is this worktree's own stop and start; nothing here makes the manager read every unit it
            //       holds, which no Git command may ask for
            //       (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
            if !restarted && let Some(watch) = config.start_watch.as_ref() {
                restarted = true;
                let _written = writeln!(
                    notices,
                    "{}",
                    replacing_notice(watch.unit.as_str(), config.start_wait)
                );
                let _ended = (watch.restart)(watch.bus.as_path(), watch.unit.as_str());
                give_up_at = Some(Instant::now() + config.start_ceiling);
                continue;
            }
            return Err(ClientFailure::OtherRelease);
        }
        // Written in the version the service announced: a service of an earlier release reads a retire in no other.
        let mut retire: String = encode_retire(version);
        retire.push('\n');
        let _sent = reader.get_mut().write_all(retire.as_bytes());
        trace.retires_sent += 1;
        std::thread::sleep(CONNECT_PAUSE * 4);
    }
}

/// Controls of the wait and of the repair it asks the manager for stay out of the release executable.
#[cfg(test)]
#[path = "service_client_reach_service_tests.rs"]
mod tests;
