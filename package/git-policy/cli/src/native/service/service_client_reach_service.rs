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

use crate::service_client_settings_and_watches::{Activation, CONNECT_PAUSE, ClientConfig, ManagerAnswer, START_NOTICE_GRACE};use crate::service_client_listen_and_ask_to_retire::{Heard, Reached, listen};
use crate::commit_timing::ReachTrace;
use crate::service_messages::{ClientFailure, UPDATING_NOTICE, replacing_notice, restarting_notice, starting_notice};
use crate::service_messages_of_a_start_that_was_asked_and_did_not_come::{ManagerLastWords, RequestedStart};
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
/// and a unit it calls `inactive` is asked to start at the first answer,
/// because nothing else
///  repairs either state.
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
    let mut started: Instant = Instant::now();
    let mut refused_here: bool = false;
    // Nothing has been asked of the manager yet: the first connection that finds nothing listening asks at once, because a unit
    //  that stopped is unloaded and only a start loads it again (`ManagerAnswer`).
    let mut last_asked: Option<Instant> = None;
    let mut failed_answers: u32 = 0;
    let mut cannot_ask: bool = false;
    let mut last_start_request: Option<Instant> = None;
    let mut repairs: u32 = 0;
    let mut restarts: u32 = 0;
    // What the manager last answered about the unit, which the notice names and the restart is decided on: nothing until it
    //  is asked. The answer itself is kept, not only its state word: a manager that answered that it holds no such unit
    //  answered every poll, and folding that into "nothing" is what made a wait read as a manager that was silent.
    let mut last_answer: Option<ManagerAnswer> = None;
    // What this command asked the manager to do about the unit, which the words at the end name: nothing until it asks.
    let mut requested: RequestedStart = RequestedStart::Nothing;
    // Whether anything took a connection at the address, which is the one fact that tells an address nothing listens on from
    //  one whose connections nothing answers.
    let mut connected: bool = false;
    loop {
        trace.tries += 1;
        // The account-aware call of this file's own name for its wait loop: nothing else asks who took the connection.
        match crate::service_client_listen_and_ask_to_retire::reach(&config.address) {
            Reached::Ours(stream) => {
                connected = true;
                let patience: Duration = match (&config.start_watch, service_seen) {
                    (_, true) => config.heartbeat_silence,
                    (Some(watch), false) => watch.every,
                    (None, false) => greeting_patience_without_a_manager(config, started),
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
                    Heard::Message(_) | Heard::Closed | Heard::Broken | Heard::Silent => {
                        trace.refused += 1;
                        refused_here = true;
                    }
                }
            }
            // What: A connection another account's process took ends the wait at once.
            // Why:  A service is this account's own, and an address no directory protects (an abstract name,
            //       `service_address_of_a_worktree.rs`) is held to the kernel's word about who took the connection. Waiting
            //       out the start would send this commit's request to that process or tell the person nothing.
            Reached::OtherAccount => return Err(ClientFailure::OtherAccount),
            Reached::Nothing => match config.activation {
                Activation::Unsupported(platform) => return Err(ClientFailure::NoActivation(platform)),
                Activation::Managed => {
                    trace.refused += 1;
                    refused_here = true;
                }
            },
        }
        if let Some(watch) = config.start_watch.as_ref()
            && a_poll_is_due(last_asked, watch.every)
        {
            last_asked = Some(Instant::now());
            let answer: ManagerAnswer = (watch.state)(watch.bus.as_path(), watch.unit.as_str());
            last_answer = Some(answer.clone());
            // What: A unit the manager answers `inactive` for, and one it holds no such unit for, is asked to start at
            //       once, then again after each start wait while it stays that way.
            // Why:  The client's connection was refused, so nothing of this worktree listens and nothing is starting it: an
            //       `inactive` unit is the answer a person's `systemctl --user stop` leaves, and provisioning counts the
            //       socket file it kept as in place, so nothing else starts it. Waiting the start wait (ten seconds)
            //       before asking made that commit ten seconds slower than native Git
            //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 5). A unit that
            //       is `activating` is being started and is left alone.
            //       A unit the manager holds no such unit for is the state of every stopped service: the manager unloads
            //       it and `GetUnit` then answers `NoSuchUnit`, so a client that waited for `inactive` waited for an answer
            //       the manager never gives, and the start that loads the unit was never asked for. Every merge's hand-off
            //       of one worktree after its service retired and stopped its own socket then ended with "the commit service
            //       did not start" on 2026-11-14T20:51-05:00, its socket off and its unit unloaded
            //       (`service_own_socket_of_a_retire_that_served_nothing.rs`).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if ((answer.state === 'inactive' || answer.notHeld) && (lastStartRequest === undefined || lastStartRequest.elapsed() >= startWait)) { await start(unit); }
            // ```
            let asks_for_a_start: bool = match &answer {
                // A unit the manager holds no such unit for is one that stopped and was unloaded: the start loads it again.
                ManagerAnswer::NotHeld => true,
                ManagerAnswer::State(state) => state == "inactive",
                ManagerAnswer::Unreachable => false,
            };
            if asks_for_a_start
                && match last_start_request {
                    Some(at) => at.elapsed() >= config.start_wait,
                    None => true,
                }
            {
                let _taken = (watch.start)(watch.bus.as_path(), watch.unit.as_str());
                last_start_request = Some(Instant::now());
                requested = RequestedStart::Start;
            }
            cannot_ask = answer == ManagerAnswer::Unreachable;
            // What: A unit the manager calls `failed` is asked to be reset and started, once, and the wait goes on.
            // Why:  Provisioning finds the unit text current and the socket file there (systemd keeps it) and so
            //       leaves the failed state alone, and a unit in that state never comes back by itself; the
            //       command would fail with "run the commit again", which reproduces the same state. The reset
            //       comes first because a unit that failed too often refuses a start as "repeated too quickly".
            //       One repair only, and a second `failed` answer still ends the wait, so a unit that cannot start fails
            //       the commit in seconds.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (answer === 'failed' && repairs++ === 0) { await reset(unit); await start(unit); failedAnswers = 0; }
            // ```
            if answer.state() == Some("failed") && repairs == 0 {
                repairs += 1;
                // The service unit requires the socket unit, so one start covers both and both states are forgotten.
                let _forgotten = (watch.reset)(watch.bus.as_path(), watch.unit.as_str());
                let _socket_forgotten = (watch.reset)(watch.bus.as_path(), watch.socket_unit.as_str());
                let _taken = (watch.start)(watch.bus.as_path(), watch.unit.as_str());
                last_start_request = Some(Instant::now());
                requested = RequestedStart::ResetAndStart;
                failed_answers = 0;
            }
            failed_answers = if answer.state() == Some("failed") { failed_answers + 1 } else { 0 };
        }
        // What: The wait says what it waits for and on whom, once it has lasted long enough to matter.
        // Why:  A commit with a service that answers nothing printed nothing for ten seconds and then named a cause that was
        //       not true, and an add printed nothing for 613 s (`~/.claude/../report.md`, findings 3 and 5 of the
        //       after-alpha.9 audit): a wait nobody can see is indistinguishable from a hang, and the state the manager
        //       reports is what tells a service that is starting from one that is not.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!announced && started.elapsed() >= START_NOTICE_GRACE) { announced = true; write(startingNotice(addressText(address), who(lastState))) }
        // ```
        if !*announced && started.elapsed() >= START_NOTICE_GRACE {
            *announced = true;
            let address: String = crate::service_server::address_text(&config.address);
            let who: String = match (&config.start_watch, last_answer.as_ref()) {
                (None, _) => String::from("and no service manager can be asked to start it"),
                (Some(watch), Some(ManagerAnswer::State(state))) => format!("the service manager says {} is {state}", watch.unit),
                (Some(watch), Some(ManagerAnswer::NotHeld)) => format!("and the service manager holds no {}, so it was asked to start it", watch.unit),
                (Some(watch), Some(ManagerAnswer::Unreachable)) => format!("and the service manager could not be asked about {}", watch.unit),
                (Some(watch), None) => format!("and the service manager gave no answer about {} yet", watch.unit),
            };
            let _written = writeln!(notices, "{}", starting_notice(address.as_str(), who.as_str()));
        }
        // What: A unit the manager reports as running that has greeted nothing for the whole start ceiling is asked to
        //       restart, once.
        // Why:  A service greets a connection before it restores anything, so a service that has run a whole start ceiling
        //       with nothing to say is one whose program stopped inside its own process (a frozen control group, a debugger,
        //       a suspended machine, a wait in the kernel nothing ends), and no start of a unit the manager believes runs
        //       brings it back. The service survives being ended by design: its journal, its held commits and its work records
        //       are durable, so a commit that needs an answer gets one. What the ceiling separates is the state from a start
        //       that is merely slow, and nothing shorter may stand in for it: on 2026-11-12 a greeting three seconds late was
        //       read as a service stopped inside its own process, and the restart that followed made a commit fail that would
        //       have landed (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`). Three seconds is
        //       not a measurement of a loaded machine, and nothing here may ask the manager for anything but this worktree's
        //       own units.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (restarts === 0 && lastState === 'active' && started.elapsed() >= config.startCeiling) { restarts += 1; await restart(watch.unit); write(restartingNotice(...)); started = Instant.now(); continue; }
        // ```
        if let Some(watch) = config.start_watch.as_ref()
            && restarts == 0
            && !service_seen
            && last_answer.as_ref().and_then(ManagerAnswer::state) == Some("active")
            && started.elapsed() >= config.start_ceiling
        {
            restarts += 1;
            let _restarted = (watch.restart)(watch.bus.as_path(), watch.unit.as_str());
            last_start_request = Some(Instant::now());
            let _written = writeln!(notices, "{}", restarting_notice(watch.unit.as_str(), started.elapsed()));
            started = Instant::now();
            last_answer = None;
            requested = RequestedStart::Restart;
            continue;
        }
        match &config.start_watch {
            // One answer of "failed" can be the unit's last run, before the connection made the manager start it again. A start the
            //  manager reports as under way is waited for until the start ceiling and no longer (owner, 2026-11-03: "nothing
            //  should ever wait forever or indefinitely").
            //  The failure carries which unit the manager was asked about and what it last said, because "the commit service did
            //  not start" alone said nothing about the wait that ended: a start that was never asked for leaves the service's own
            //  log empty, and that is what left the merges of 2026-11-14T20:51-05:00 unexplained.
            Some(watch) if failed_answers >= 2 || (cannot_ask && started.elapsed() >= config.start_wait) || started.elapsed() >= config.start_ceiling => {
                return Err(ClientFailure::DidNotStart {
                    unit: Some(watch.unit.clone()),
                    answer: manager_last_words(last_answer.as_ref()),
                    asked: requested,
                });
            }
            Some(_) => {}
            // Without a manager to ask, nothing can start the service: what the wait comes to is the truth about the address.
            None if started.elapsed() >= config.start_wait => return Err(nothing_answered(config, connected, started.elapsed())),
            None => {}
        }
        std::thread::sleep(CONNECT_PAUSE);
    }
}

/// What:
///  The longest one connection attempt may wait for the greeting where no service manager can be asked,
///  which is the notice grace or what is left of the start wait,
/// whichever is shorter.
/// Why:
///   A read that lasts the whole start wait spends the entire wait inside one socket read,
/// so the loop that
///  says what the wait is for never gets to say it:
/// a commit against a service stopped inside its own process printed nothing for ten
///  seconds and then named a cause that was not true
/// (`~/.claude/../sonnet-audits/integration-tip-never-stuck/report.md`,
/// finding 3 of the after-alpha.9 audit).
/// The bound itself does not move:
/// what is left of the start wait caps the last attempt,
/// so the wait still ends at the
///  bound it was granted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// greetingPatienceWithoutAManager(config, started) // min(START_NOTICE_GRACE, startWait - elapsed)
/// ```
fn greeting_patience_without_a_manager(config: &ClientConfig, started: Instant) -> Duration {
    let remaining: Duration = config.start_wait.saturating_sub(started.elapsed());
    // A read timeout of no time at all is refused by the system, so one millisecond is the shortest attempt there is.
    return remaining.min(START_NOTICE_GRACE).max(Duration::from_millis(1));
}

/// What:
///  The end of a wait where no manager could be asked and nothing answered,
/// worded from what the wait proved.
/// Why:
///  `DidNotStart` was printed here where the service had started and then stopped inside its own process,
/// which sent the person to a
///  log that holds nothing about it;
/// an address nothing listens on and an address whose connections go unanswered are two different
///  states with two different next steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nothingAnswered(config, connected, waited) // { nothing-answered } | { did-not-start }
/// ```
fn nothing_answered(config: &ClientConfig, connected: bool, waited: Duration) -> ClientFailure {
    if !connected {
        // No manager can be asked here (`start_watch` is absent), so no unit was named to anyone and none answered.
        return ClientFailure::DidNotStart { unit: None, answer: ManagerLastWords::Silent, asked: RequestedStart::Nothing };
    }
    return ClientFailure::NothingAnswered { address: crate::service_server::address_text(&config.address), waited };
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
pub(crate) fn reach_current(config: &ClientConfig, notices: &mut dyn Write, announced: &mut bool, trace: &mut ReachTrace, service_seen: bool) -> Result<BufReader<UnixStream>, ClientFailure> {
    let began: Instant = Instant::now();
    let reached = reach_current_untimed(config, notices, announced, trace, service_seen);
    trace.elapsed += began.elapsed();
    return reached;
}

/// `reach_current` without the clock around it.
pub(crate) fn reach_current_untimed(config: &ClientConfig, notices: &mut dyn Write, announced: &mut bool, trace: &mut ReachTrace, service_seen: bool) -> Result<BufReader<UnixStream>, ClientFailure> {
    let mut give_up_at: Option<Instant> = None;
    // Whether the service of another release that outlasted the bound was already ended, which happens once.
    let mut restarted: bool = false;
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
            if !restarted
                && let Some(watch) = config.start_watch.as_ref()
            {
                restarted = true;
                let _written = writeln!(notices, "{}", replacing_notice(watch.unit.as_str(), config.start_wait));
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
