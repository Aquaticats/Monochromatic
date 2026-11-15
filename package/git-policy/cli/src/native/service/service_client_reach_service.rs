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
        // What: Every pass of this loop asks the bound before it does anything else, so no path through it can be taken
        //       again without the clock being looked at.
        // Why:  A `continue` after a start went straight back to the connect with no look at the bound and no pause, so a
        //       service that greeted a look's millisecond of read and never the look itself was connected to, started and
        //       connected to again for as long as the machine ran: a command that spun for twelve hours, at half a core, and
        //       was killed by hand (found 2026-11-15 from the process record of that command).
        //       `Greeted` says a look somewhere did see a greeting; it is never a reason to skip the bound.
        // What: A service that is there and has not greeted is a service that is alive and still returning to work, and it is
        //       waited for its own bound; one this command started, or one that answers with something that is not this
        //       protocol, keeps the start's own bound.
        // Why:  The two states have two next steps and two sentences: an address nothing listens on is a start that failed, and
        //       a service that takes the connection and greets nothing is one that is running and recovering.
        let connected_ceiling: Duration = config.recovering_ceiling;
        let start_bound: Duration = match &config.starter {
            Some(_) => config.start_ceiling,
            None => config.start_wait,
        };
        if connected {
            if started.elapsed() >= connected_ceiling {
                return Err(recovering(config, started.elapsed()));
            }
        } else if started.elapsed() >= start_bound {
            return Err(ClientFailure::DidNotStart);
        }
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
                // A greeting a look could not read inside its own patience: the next pass asks again, after the bound.
                StartAttempt::Greeted | StartAttempt::AnotherCommandStartsIt => {}
                StartAttempt::DidNotGreet(log) => return Err(not_greeted(log)),
            }
        }
        // What: The wait says what it waits for and on whom, once it has lasted long enough to matter.
        // Why:  A wait nobody can see is indistinguishable from a hang.
        if !*announced && started.elapsed() >= START_NOTICE_GRACE {
            *announced = true;
            let address: String = crate::service_server::address_text(&config.address);
            let _written = writeln!(
                notices,
                "{}",
                match connected {
                    // A service is there: saying this command starts it would be false.
                    true => crate::service_messages_of_a_wait_for_the_service::recovering_notice(address.as_str()),
                    false => starting_notice(address.as_str(), "and this command starts it when it is not running"),
                }
            );
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
    // What: A look waits as long for a greeting as the look `answered` makes, and no longer than what is left of the start
    //       wait, so the notice of the wait is still printed while it lasts.
    // Why:  A look that read for a shorter time than `answered` did could report a silence about a service `answered` then
    //       called greeted, which is how the command that spun for twelve hours got its `continue`: the two waits disagreed
    //       (`answered` reads for `GREETING_READ`, this read shrank to a millisecond once the start wait had passed).
    // A read timeout of no time at all is refused by the system, so one millisecond is the shortest attempt there is.
    return remaining.min(crate::service_server_start_by_the_wrapper::GREETING_READ).max(Duration::from_millis(1));
}

/// What:
///  The end of a wait for a service that is there and has not greeted within its own bound.
/// Why:
///  It is not the failure of a start: the address takes connections, so a service process runs.
/// Saying that nothing answered about it sends the person to look for a service that does not exist, and fails a commit
///  the service would have served when its recovery or its runtime build ended
/// (the review of `77f5d525e3`, finding 3).
/// The service's own log names what it was doing, which is where a recovery's waiting is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recovering(config, waited) // { service-recovering, address, waited, log }
/// ```
fn recovering(config: &ClientConfig, waited: Duration) -> ClientFailure {
    return ClientFailure::ServiceRecovering {
        address: crate::service_server::address_text(&config.address),
        waited,
        log: config.starter.as_ref().map(|server| { return server.log(); }),
    };
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
///  Whether the service manager is due to be asked again,
///  which is the first time and then once every `every`.
/// Why:
///  A named function keeps the condition of the wait readable and keeps no closure over the watch it belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aPollIsDue(lastAsked, every) // lastAsked === undefined || now - lastAsked >= every
/// ```
fn a_poll_is_due(last_asked: Option<Instant>, every: Duration) -> bool {
    return match last_asked {
        Some(at) => at.elapsed() >= every,
        None => true,
    };
}

/// What:
///  The last answer of the service manager as the words at the end name it,
///  keeping "it holds no such unit" and "it gave no answer" apart.
/// Why:
///   A unit the manager unloaded is one it answered every poll that it holds no such unit for,
///   and a manager whose exchange failed is one that answered nothing:
///   the two have different next steps,
///   and the wait records the answer itself so the words can say which happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// managerLastWords(lastAnswer) // 'state' | 'not-held' | 'silent'
/// ```
fn manager_last_words(answer: Option<&ManagerAnswer>) -> ManagerLastWords {
    return match answer {
        None | Some(ManagerAnswer::Unreachable) => ManagerLastWords::Silent,
        Some(ManagerAnswer::NotHeld) => ManagerLastWords::NotHeld,
        Some(ManagerAnswer::State(state)) => ManagerLastWords::State(state.clone()),
    };
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
        // What: A service that announces any protocol this build still serves is served by this command.
        // Why:  The greeting names the oldest protocol the service speaks (`MIN_SERVED_PROTOCOL`), so a service of this build
        //       and a service of an earlier build both greet 11; a question the older one does not know is refused by it and read
        //       here as no answer, which is the path that exists for that.
        if (crate::service_protocol::MIN_SERVED_PROTOCOL..=PROTOCOL_VERSION).contains(&version) {
            // What: A service of this protocol serves this command, whatever release it announces; one of an older release is
            //       asked to leave so this release's own program replaces it.
            // Why:  The protocol is the contract between the two programs, and the release check existed because the service
            //       manager started the unit's program, which an upgrade could have removed. With the wrapper starting its own
            //       service there is nothing else to check, and asking a *newer* service to leave is what made a changeover cost
            //       a commit: it exits, the manager starts the newer program again, and the older client asks again
            //       (`service_release_order.rs`).
            let newer_or_equal: bool = match release.as_deref() {
                Some(theirs) => matches!(crate::service_release_order::compare(theirs, RELEASE), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
                // A service that announces no release is this build's own, as every service this release starts is.
                None => true,
            };
            if newer_or_equal {
                if release.as_deref() != Some(RELEASE) {
                    let _written = writeln!(notices, "{}", other_release_notice(release.as_deref()));
                }
                trace.answered_by = release;
                return Ok(reader);
            }
            // What: A service of an earlier release is met, so an earlier release's units are what hold this worktree's address;
            //       they are retired once in this command, before it asks the service to leave.
            // Why:  A loaded socket unit of that release keeps the name bound, so a server of this release could never own it, and
            //       the manager would start the old program again on every connection (`service_server_earlier_release_units.rs`).
            //       A command that meets only its own release touches nothing, which is why this is here and not above the return:
            //       measured against the branch's own service, every commit read the unit folder and asked the manager to stop
            //       units no earlier release ever wrote (the review of `77f5d525e3`, finding 1).
            let _retired: Vec<std::path::PathBuf> = retire_earlier_units_once(config);
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

/// What:
///  Retires an earlier release's units once in this command,
///  and nothing on every later look.
/// Why:
///  The removal and the manager's stop cost a read of the unit directory and one program call,
/// so they are paid once, at the first service of another release this command meets;
/// the fact is the process's, because a command is one process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!retiredOnce) retireEarlierUnits(host)
/// ```
fn retire_earlier_units_once(config: &ClientConfig) -> Vec<std::path::PathBuf> {
    static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if DONE.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Vec::new();
    }
    let Some(server) = config.starter.as_ref() else { return Vec::new() };
    // The socket sits one folder below the runtime directory the units live in, and the manager is the gate's own answer:
    //  a command that may not ask a manager passes none.
    let Some(runtime_dir) = server.directory().parent().map(std::path::Path::to_path_buf) else { return Vec::new() };
    return crate::service_server_earlier_release_units::retire_earlier_units(
        runtime_dir.as_path(),
        server.named_runtime_dir.as_deref(),
        server.bus_address.as_deref(),
        server.instance.as_str(),
    );
}

/// What:
///  The line a command prints when a service of another release serves it.
/// Why:
///  A person reading a test's log or a slow commit should know which program served it;
/// the line is one, on standard error, and it asks nothing of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// otherReleaseNotice('0.6.1-alpha.99') // '... served by release 0.6.1-alpha.99 ...'
/// ```
fn other_release_notice(release: Option<&str>) -> String {
    return match release {
        Some(theirs) => format!("this commit was served by the commit service of release {theirs}, which speaks the same protocol as this git wrapper ({RELEASE})"),
        None => String::from("this commit was served by a commit service that announces no release"),
    };
}

/// Controls of the wait stay out of the release executable.
#[cfg(test)]
#[path = "service_client_reach_service_tests.rs"]
mod tests;
