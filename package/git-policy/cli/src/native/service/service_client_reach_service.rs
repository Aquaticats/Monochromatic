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
//! In TS you'd write (pseudocode):
//! ```ts
//! const reader = await reachCurrent(config, notices, announced, trace, serviceSeen);
//! ```

use crate::service_client_settings_and_watches::{Activation, CONNECT_PAUSE, ClientConfig, START_NOTICE_GRACE};
use crate::service_client_listen_and_ask_to_retire::{Heard, connect, listen};
use crate::commit_timing::ReachTrace;
use crate::service_messages::{ClientFailure, STARTING_NOTICE, UPDATING_NOTICE};
use crate::service_protocol::{
    PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_retire,
};
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};
/// What:
///  Connects,
/// waiting for a start the operating system performs,
/// and returns the stream with the service's version.
/// Why:
///   With a manager to ask,
/// the wait ends when the service greets,
/// when the manager has said twice in a
///  row that the unit failed,
/// or at `START_CEILING` (one minute).
/// A unit the manager calls `failed` is asked to be
///  reset and started once,
/// because nothing else repairs that state.
/// Where the manager cannot be asked it ends at the start wait,
/// as it always did.
/// Once a service
///  has greeted during this command (`service_seen`),
/// a service that then greets nothing within the silence bound is the
///  one that stopped,
/// and that ends the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stream, version } = await reach(config, notices, serviceSeen)
/// ```
pub(crate) fn reach(
    config: &ClientConfig,
    notices: &mut dyn Write,
    announced: &mut bool,
    trace: &mut ReachTrace,
    service_seen: bool,
) -> Result<(BufReader<UnixStream>, u32, Option<String>), ClientFailure> {
    let started: Instant = Instant::now();
    let mut refused_here: bool = false;
    let mut last_asked: Instant = started;
    let mut failed_answers: u32 = 0;
    let mut cannot_ask: bool = false;
    let mut last_start_request: Option<Instant> = None;
    let mut repairs: u32 = 0;
    loop {
        trace.tries += 1;
        match connect(&config.address) {
            Ok(stream) => {
                let patience: Duration = match (&config.start_watch, service_seen) {
                    (_, true) => config.heartbeat_silence,
                    (Some(watch), false) => watch.every,
                    (None, false) => config.start_wait,
                };
                let _timeout = stream.set_read_timeout(Some(patience));
                let mut reader: BufReader<UnixStream> = BufReader::new(stream);
                match listen(&mut reader) {
                    Heard::Message(ServiceMessage::Welcome { version, release }) => {
                        let _widened = reader.get_ref().set_read_timeout(Some(config.heartbeat_silence));
                        trace.saw_release(release.as_deref());
                        if refused_here {
                            trace.start_waited += started.elapsed();
                        }
                        return Ok((reader, version, release));
                    }
                    // A service that greeted before and now greets nothing is the one that stopped.
                    Heard::Silent if service_seen => return Err(ClientFailure::ServiceSilent),
                    // Connected, but nothing greeted: the service is still starting or is going away.
                    Heard::Message(_) | Heard::Closed | Heard::Silent => {
                        trace.refused += 1;
                        refused_here = true;
                    }
                }
            }
            Err(_) => match config.activation {
                Activation::Unsupported(platform) => return Err(ClientFailure::NoActivation(platform)),
                Activation::Managed => {
                    trace.refused += 1;
                    refused_here = true;
                }
            },
        }
        match &config.start_watch {
            Some(watch) => {
                if last_asked.elapsed() >= watch.every {
                    last_asked = Instant::now();
                    let answer: Option<String> = (watch.state)(watch.bus.as_path(), watch.unit.as_str());
                    // A unit still `inactive` after the start wait, with the client's connection waiting on its socket, is one the
                    //  manager did not start; it is asked to, again after each further start wait, and the client goes on waiting.
                    // A unit that is `activating` is being started and is left alone.
                    if answer.as_deref() == Some("inactive")
                        && started.elapsed() >= config.start_wait
                        && match last_start_request {
                            Some(at) => at.elapsed() >= config.start_wait,
                            None => true,
                        }
                    {
                        let _taken = (watch.start)(watch.bus.as_path(), watch.unit.as_str());
                        last_start_request = Some(Instant::now());
                    }
                    cannot_ask = answer.is_none();
                    // What: A unit the manager calls `failed` is asked to be reset and started, once, and the wait goes on.
                    // Why:  Provisioning finds the unit text current and the socket file there (systemd keeps it) and so
                    //       leaves the failed state alone, and a unit in that state never comes back by itself; the
                    //       command would fail with "run the commit again", which reproduces the same state. The reset
                    //       comes first because a unit that failed too often refuses a start as "repeated too quickly".
                    //       One repair, then the answer of two failed readings still ends the wait.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (answer === 'failed' && repairs++ === 0) { await reset(unit); await start(unit); failedAnswers = 0; }
                    // ```
                    if answer.as_deref() == Some("failed") && repairs == 0 {
                        repairs += 1;
                        // The service unit requires the socket unit, so one start covers both and both states are forgotten.
                        let _forgotten = (watch.reset)(watch.bus.as_path(), watch.unit.as_str());
                        let _socket_forgotten = (watch.reset)(watch.bus.as_path(), watch.socket_unit.as_str());
                        let _taken = (watch.start)(watch.bus.as_path(), watch.unit.as_str());
                        last_start_request = Some(Instant::now());
                        failed_answers = 0;
                    }
                    failed_answers = if answer.as_deref() == Some("failed") { failed_answers + 1 } else { 0 };
                }
                // One answer of "failed" can be the unit's last run, before the connection made the manager start it again. A start the
                //  manager reports as under way is waited for until the start ceiling and no longer (owner, 2026-11-03: "nothing
                //  should ever wait forever or indefinitely").
                if failed_answers >= 2 || (cannot_ask && started.elapsed() >= config.start_wait) || started.elapsed() >= config.start_ceiling {
                    return Err(ClientFailure::DidNotStart);
                }
            }
            None => {
                if started.elapsed() >= config.start_wait {
                    return Err(ClientFailure::DidNotStart);
                }
            }
        }
        if !*announced && started.elapsed() >= START_NOTICE_GRACE {
            *announced = true;
            let _written = writeln!(notices, "{STARTING_NOTICE}");
        }
        std::thread::sleep(CONNECT_PAUSE);
    }
}

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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = await reachCurrent(config, notices, announced)
/// ```
pub(crate) fn reach_current(config: &ClientConfig, notices: &mut dyn Write, announced: &mut bool, trace: &mut ReachTrace, service_seen: bool) -> Result<BufReader<UnixStream>, ClientFailure> {
    let began: Instant = Instant::now();
    let reached = reach_current_untimed(config, notices, announced, trace, service_seen);
    trace.elapsed += began.elapsed();
    return reached;
}

/// `reach_current` without the clock around it.
pub(crate) fn reach_current_untimed(config: &ClientConfig, notices: &mut dyn Write, announced: &mut bool, trace: &mut ReachTrace, service_seen: bool) -> Result<BufReader<UnixStream>, ClientFailure> {
    let mut give_up_at: Option<Instant> = None;
    loop {
        let (mut reader, version, release) = reach(config, notices, announced, trace, service_seen)?;
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
                let fresh: Instant = Instant::now() + config.start_wait;
                give_up_at = Some(fresh);
                fresh
            }
        };
        if Instant::now() >= deadline {
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
