//! What:
//!  Reaches the commit service:
//! connects to its address,
//! starts the server itself when nothing answers,
//! and waits for a
//!  service of the current release.
//! Why:
//!  Submitting a request or a withdrawal begins by reaching a service that speaks this release;
//! the steps have their own file
//!  so the submit loop stays readable.
//! Until this release the operating system started the service
//!  through its unit;
//! the wrapper starts it now,
//! so the wait here starts a server and never asks a service manager.
//! The wait ends when the service greets,
//! when a start this command made sent no greeting within `START_WAIT`,
//! or at `START_CEILING`
//!  (one minute),
//! and a service that greeted before and then greets nothing is the one that stopped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reader = await reachCurrent(config, notices, announced, trace, serviceSeen);
//! ```

use crate::commit_timing::ReachTrace;
use crate::service_client_listen_and_ask_to_retire::{Heard, connect, listen};
use crate::service_client_settings_and_watches::{Activation, CONNECT_PAUSE, ClientConfig, START_NOTICE_GRACE};
use crate::service_messages::{ClientFailure, UPDATING_NOTICE, starting_notice};
use crate::service_protocol::{PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_retire};
use crate::service_server_start_by_the_wrapper::{StartAttempt, start};
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

/// What:
///  Connects,
/// starting the server when nothing answers,
/// and returns the stream with the service's version.
/// Why:
///   A start has its own bound,
/// so a service that starts and dies fails the command in seconds instead of
///  waiting out the ceiling;
/// a start another command of this worktree is making is waited for,
/// which is the same service being made
///  exist;
/// and once a service has greeted during this command (`service_seen`),
/// a service that then greets nothing within the
///  silence bound is the one that stopped,
/// which ends the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stream, version } = await reach(config, notices, serviceSeen)
/// ```
pub(crate) fn reach(config: &ClientConfig, notices: &mut dyn Write, announced: &mut bool, trace: &mut ReachTrace, service_seen: bool) -> Result<(BufReader<UnixStream>, u32, Option<String>), ClientFailure> {
    let started: Instant = Instant::now();
    let mut connected: bool = false;
    loop {
        trace.tries += 1;
        match connect(&config.address) {
            Ok(stream) => {
                connected = true;
                let patience: Duration = if service_seen { config.heartbeat_silence } else { greeting_patience(config, started) };
                let _timeout = stream.set_read_timeout(Some(patience));
                let mut reader: BufReader<UnixStream> = BufReader::new(stream);
                match listen(&mut reader) {
                    Heard::Message(ServiceMessage::Welcome { version, release }) => {
                        let _widened = reader.get_ref().set_read_timeout(Some(config.heartbeat_silence));
                        trace.saw_release(release.as_deref());
                        trace.start_waited += started.elapsed();
                        return Ok((reader, version, release));
                    }
                    // A service that greeted before and now greets nothing is the one that stopped.
                    Heard::Silent if service_seen => return Err(ClientFailure::ServiceSilent),
                    // Connected, but nothing greeted: the service is still starting or is going away.
                    Heard::Message(_) | Heard::Closed | Heard::Silent => trace.refused += 1,
                }
            }
            Err(_) => match config.activation {
                Activation::Unsupported(platform) => return Err(ClientFailure::NoActivation(platform)),
                Activation::StartedByTheWrapper => trace.refused += 1,
            },
        }
        // What: Nothing answered, so the server is started here, once per pass of this loop.
        // Why:  The wrapper is what makes a service exist on a machine with no service manager, and a service that died under a
        //       waiting command is brought back by the same code. A start another command is making is waited for instead.
        if let Some(server) = config.starter.as_ref() {
            match start(server) {
                StartAttempt::Greeted => continue,
                StartAttempt::AnotherCommandStartsIt => {}
                StartAttempt::DidNotGreet(log) => return Err(not_greeted(log)),
            }
        }
        // What: The wait says what it waits for and on whom, once it has lasted long enough to matter.
        // Why:  A wait nobody can see is indistinguishable from a hang.
        if !*announced && started.elapsed() >= START_NOTICE_GRACE {
            *announced = true;
            let address: String = crate::service_server::address_text(&config.address);
            let _written = writeln!(notices, "{}", starting_notice(address.as_str(), "and this command starts it when it is not running"));
        }
        match &config.starter {
            // One start is bounded by its own wait, so reaching this bound means a start is still under way in another command, or
            //  a service keeps answering with something that is not this protocol: both fail the command in a known time.
            Some(_) if started.elapsed() >= config.start_ceiling => {
                return Err(if connected { nothing_answered(config, started.elapsed()) } else { ClientFailure::DidNotStart });
            }
            Some(_) => {}
            // Without an executable to start, nothing can start the service: what the wait comes to is the truth about the address.
            None if started.elapsed() >= config.start_wait => {
                return Err(if connected { nothing_answered(config, started.elapsed()) } else { ClientFailure::DidNotStart });
            }
            None => {}
        }
        std::thread::sleep(CONNECT_PAUSE);
    }
}

/// What:
///  The longest one connection attempt may wait for the greeting,
///  which is the notice grace or what is left of the start wait,
/// whichever is shorter.
/// Why:
///   A read that lasts the whole start wait spends the entire wait inside one socket read,
/// so the loop that
///  says what the wait is for never gets to say it,
/// and the server would not be started until that read ends.
/// The bound itself does not move:
/// what is left of the start wait caps the last attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// greetingPatience(config, started) // min(START_NOTICE_GRACE, startWait - elapsed)
/// ```
fn greeting_patience(config: &ClientConfig, started: Instant) -> Duration {
    let remaining: Duration = config.start_wait.saturating_sub(started.elapsed());
    // A read timeout of no time at all is refused by the system, so one millisecond is the shortest attempt there is.
    return remaining.min(START_NOTICE_GRACE).max(Duration::from_millis(1));
}

/// What:
///  The end of a wait where something took the connection but nothing answered.
/// Why:
///  `DidNotStart` was printed here where the service had started and then stopped inside its own process,
/// which sent the person to a
///  log that holds nothing about it;
/// an address nothing listens on and an address whose connections go unanswered are two different
///  states with two different next steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nothingAnswered(config, waited) // { nothing-answered } | { did-not-start }
/// ```
fn nothing_answered(config: &ClientConfig, waited: Duration) -> ClientFailure {
    return ClientFailure::NothingAnswered { address: crate::service_server::address_text(&config.address), waited };
}

/// What:
///  A start that sent no greeting,
/// worded with where its output went.
/// Why:
///   The one reason a start of this release does not greet is the program's own failure,
/// and the only record of it is the file beside the socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// notGreeted(log) // CouldNotStart with the log path
/// ```
pub fn not_greeted(log: std::path::PathBuf) -> ClientFailure {
    return ClientFailure::CouldNotStart(format!(
        "the commit service was started and sent no greeting within {} seconds. Its output is in {}; if it says nothing, remove that \
         file and the address beside it and run the commit again",
        crate::service_client_listen_and_ask_to_retire::START_WAIT.as_secs(),
        log.display()
    ));
}

/// What:
///  Reaches a service that speaks this wrapper's protocol and was started from this wrapper's release,
/// asking any other to
///  retire first.
/// Why:
///   A service of the other release may answer after an upgrade or a downgrade:
/// one whose program was since
///  removed,
/// or one that behaves as the other release did.
/// It is asked to finish its batch and exit,
/// and the next connection makes this wrapper start its
///  own server.
/// The asking is bounded by
///  the start wait from the first ask,
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
        // A service that was asked to leave and still answers is asked again; once its address is free the loop's own start
        // replaces it, which is a start this wrapper makes and never a call to a manager.
        std::thread::sleep(CONNECT_PAUSE * 4);
    }
}

/// Controls of the wait stay out of the release executable.
#[cfg(test)]
#[path = "service_client_reach_service_tests.rs"]
mod tests;
