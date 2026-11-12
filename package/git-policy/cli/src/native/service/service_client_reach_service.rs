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

use crate::service_client_settings_and_watches::{Activation, CONNECT_PAUSE, ClientConfig, START_NOTICE_GRACE};use crate::service_client_listen_and_ask_to_retire::{Heard, connect, listen};
use crate::commit_timing::ReachTrace;
use crate::service_messages::{ClientFailure, UPDATING_NOTICE, replacing_notice, restarting_notice, starting_notice};
use crate::wait_ceilings::SERVICE_GREETING_SILENCE;
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
    let mut last_asked: Instant = started;
    let mut failed_answers: u32 = 0;
    let mut cannot_ask: bool = false;
    let mut last_start_request: Option<Instant> = None;
    let mut repairs: u32 = 0;
    let mut reloads: u32 = 0;
    let mut restarts: u32 = 0;
    // What the manager last answered about the unit, which the notice names and the restart is decided on: nothing until it
    //  is asked.
    let mut last_state: Option<String> = None;
    // Whether anything took a connection at the address, which is the one fact that tells an address nothing listens on from
    //  one whose connections nothing answers.
    let mut connected: bool = false;
    loop {
        trace.tries += 1;
        match connect(&config.address) {
            Ok(stream) => {
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
        if let Some(watch) = config.start_watch.as_ref()
            && last_asked.elapsed() >= watch.every
        {
            last_asked = Instant::now();
            let answer: Option<String> = (watch.state)(watch.bus.as_path(), watch.unit.as_str());
            last_state = answer.clone();
            // What: A unit the manager answers `inactive` for is asked to start at once, then again after each start
            //       wait while it stays inactive.
            // Why:  The client's connection was refused, so nothing of this worktree listens and nothing is starting it: an
            //       `inactive` unit is the answer a person's `systemctl --user stop` leaves, and provisioning counts the
            //       socket file it kept as in place, so nothing else starts it. Waiting the start wait (ten seconds)
            //       before asking made that commit ten seconds slower than native Git
            //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 5). A unit that
            //       is `activating` is being started and is left alone.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (answer === 'inactive' && (lastStartRequest === undefined || lastStartRequest.elapsed() >= startWait)) { await start(unit); }
            // ```
            if answer.as_deref() == Some("inactive")
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
            //       One repair only, and a second `failed` answer still ends the wait, so a unit that cannot start fails
            //       the commit in seconds.
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
            let who: String = match (&config.start_watch, last_state.as_deref()) {
                (None, _) => String::from("and no service manager can be asked to start it"),
                (Some(watch), Some(state)) => format!("the service manager says {} is {state}", watch.unit),
                (Some(watch), None) => format!("and the service manager does not hold {}", watch.unit),
            };
            let _written = writeln!(notices, "{}", starting_notice(address.as_str(), who.as_str()));
        }
        // What: A unit the manager reports as running that has greeted nothing for the greeting silence is asked to
        //       restart, once.
        // Why:  A service greets a connection before it restores anything, so a running unit that greets nothing is one whose
        //       program stopped inside its own process (a frozen control group, a debugger, a suspended machine, a wait in
        //       the kernel nothing ends), and no start of a unit the manager believes runs brings it back. The service
        //       survives being ended by design: its journal, its held commits and its work records are durable, so a commit
        //       that needs an answer gets one instead of failing (`wait_ceilings.rs`, `SERVICE_GREETING_SILENCE`).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (restarts === 0 && !serviceSeen && lastState === 'active' && started.elapsed() >= SERVICE_GREETING_SILENCE) { restarts += 1; await restart(watch.unit); write(restartingNotice(watch.unit, started.elapsed())); continue; }
        // ```
        if let Some(watch) = config.start_watch.as_ref()
            && restarts == 0
            && !service_seen
            && last_state.as_deref() == Some("active")
            && started.elapsed() >= SERVICE_GREETING_SILENCE
        {
            restarts += 1;
            let _restarted = (watch.restart)(watch.bus.as_path(), watch.unit.as_str());
            last_start_request = Some(Instant::now());
            let _written = writeln!(notices, "{}", restarting_notice(watch.unit.as_str(), started.elapsed()));
            continue;
        }
        match &config.start_watch {
            // One answer of "failed" can be the unit's last run, before the connection made the manager start it again. A start the
            //  manager reports as under way is waited for until the start ceiling and no longer (owner, 2026-11-03: "nothing
            //  should ever wait forever or indefinitely").
            Some(_) if failed_answers >= 2 || (cannot_ask && started.elapsed() >= config.start_wait) || started.elapsed() >= config.start_ceiling => {
                // What: A service that never greets, in a command that provisioned, is repaired by one reload of the
                //       manager's units before the command gives up.
                // Why:  The manager may hold a unit whose executable an upgrade or a downgrade removed: the files and
                //       the record of what the manager was told can agree while the manager runs the other release, and
                //       then no start brings a greeting, because the program the unit names is not there. Resetting the
                //       failed state and starting again cannot repair that; only reading the units again replaces the
                //       program the unit names. One reload, then one more ceiling, so a service that still does not greet
                //       fails the command in a bounded time (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
                //       finding 2 of the round-4 review).
                if reloads == 0
                    && let (Some(asking), Some(directory)) = (&config.start_watch, config.units_record_dir.as_ref())
                    && (asking.reload)(asking.bus.as_path())
                {
                    reloads += 1;
                    // The record certified the files against a manager that holds another unit, so it is dropped: the
                    // next command provisions and reloads instead of trusting the fast path.
                    crate::service_provision_loaded_unit_texts::forget(directory.as_path());
                    let _forgotten = (asking.reset)(asking.bus.as_path(), asking.unit.as_str());
                    let _socket_forgotten = (asking.reset)(asking.bus.as_path(), asking.socket_unit.as_str());
                    let _taken = (asking.start)(asking.bus.as_path(), asking.unit.as_str());
                    started = Instant::now();
                    last_start_request = Some(Instant::now());
                    failed_answers = 0;
                    continue;
                }
                return Err(ClientFailure::DidNotStart);
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
        return ClientFailure::DidNotStart;
    }
    return ClientFailure::NothingAnswered { address: crate::service_server::address_text(&config.address), waited };
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
                // What: The record of the unit texts the manager was told about is left as it is.
                // Why:  Another release answering used to be proof that the unit the manager loaded and the files describe
                //       were not the same, and the record was dropped so the next commit wrote the units and reloaded. A unit
                //       text no longer names a release: it names a link the wrapper points at its own executable
                //       (`service_provision_program_pointer.rs`), so the manager's text is the one on disk until a template
                //       itself changes, and a template that changed is repaired by the missing record alone
                //       (`service_provision_loaded_unit_texts.rs`). Dropping the record here made every commit after a
                //       release change, in every worktree, read every unit again: 3,521 to 20,002 ms of provisioning each on
                //       2026-11-11T20:01-20:12-05:00, and two commits that ended without a commit.
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
            //       unit names the program link rather than an executable (`service_provision_program_pointer.rs`). One ending
            //       only: a unit the manager starts from a text an upgrade left behind comes back as the other release again,
            //       and the next commit's provisioning is what repairs that.
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
