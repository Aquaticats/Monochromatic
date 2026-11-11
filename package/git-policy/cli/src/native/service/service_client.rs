//! What:
//!  The wrapper's side of a commit through the resident service:
//! connect,
//! wait honestly while the operating system
//!  starts the service,
//! send the request,
//! show the queue position,
//! survive a service that dies mid-request,
//! and turn
//!  every failure into the plain text of `service_messages.rs`.
//! Why:
//!  A failure the system can resolve is resolved here without the user:
//! a service that is starting is waited for,
//! one
//!  that crashed is reconnected to with the same request identity,
//! one that was started from another release of the
//!  wrapper (older or newer,
//! found by the release its welcome names) is asked to retire and is reached again once the
//!  manager has started the current one.
//! Only what cannot be resolved ends the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await submit(config, request, stderr); // landed, or prepare again, or a ClientFailure
//! ```

use crate::service_client_settings_and_watches::RECONNECTS;
pub use crate::service_client_listen_and_ask_to_retire::{START_WAIT, ask_to_retire};
use crate::service_client_listen_and_ask_to_retire::{Heard, listen, read_message};
pub use crate::service_client_settings_and_watches::{Activation, ClientConfig, FreshScratch, ManagerWatch, START_NOTICE_GRACE, manager_watch_for};
use crate::service_client_settings_and_watches::CONNECT_PAUSE;
use crate::diagnostic_log::debug;
use crate::service_client_reach_service::reach_current;
use crate::commit_timing::{ReachTrace, SubmitTrace, holder_kind_after};
use crate::service_client_waiting_words::{place_number, say_the_write_it_is_inside, say_where_it_stands};
use crate::service_messages::{ClientFailure, SILENT_NOTICE};
use crate::service_protocol::{
    ClientMessage, CommitRequest, FailureCode, LandedCapture, ServiceMessage, encode_client,
};
use crate::service_review_key::ReviewKey;
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceClient";

/// What a request came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Submitted {
    /// The commit is on the branch.
    Landed { commit: String, already: bool },
    /// The commit is written and kept,
    /// and a reference lock keeps its branch where it is:
    /// the command exits `0`,
    /// and the next command that finds the lock gone moves the branch.
    Held {
        /// The commit kept on the batch's side reference.
        commit: String,
        /// What the command says about it.
        words: String,
    },
    /// A named path changed on the branch:
    /// prepare again from this base and submit again with the same identity.
    ///  `landed` tells when the bytes of the commits the service landed since were captured,
    /// for the ones it knows.
    PrepareAgain { base: String, landed: Vec<LandedCapture> },
    /// The branch is no longer at the base of a commit that cannot be moved to another tip (an amend,
    /// or a commit Git built).
    ///  The caller decides whether to build it again on the new tip or to stop.
    Moved,
}

/// What:
///  The failure for a repository whose filesystem is read-only:
/// the frozen-move one when the state of a repository move says it froze this repository,
/// the plain read-only one otherwise.
/// Why:
///  The sentence about a move is told only when the move's own record says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readOnlyFailure(repository, stateFolder)
/// ```
pub fn read_only_failure(repository: &str, state_folder: Option<&Path>) -> ClientFailure {
    if let Some(folder) = state_folder
        && crate::repository_move_freeze::freeze_covers(folder, Path::new(repository))
    {
        return ClientFailure::RepositoryFrozen(repository.to_string());
    }
    return ClientFailure::RepositoryReadOnly(repository.to_string());
}

/// The user-facing failure for a code the service gave.
pub fn failure_for(code: FailureCode, request: &CommitRequest) -> ClientFailure {
    let repository: String = request.repository.display().to_string();
    return match code {
        FailureCode::RepositoryFull => ClientFailure::RepositoryFull(repository),
        FailureCode::RepositoryReadOnly => read_only_failure(repository.as_str(), crate::repository_move_freeze::move_state_folder().as_deref()),
        FailureCode::PermissionDenied => ClientFailure::PermissionDenied(repository),
        FailureCode::IndexInUse => ClientFailure::IndexInUse(String::from("Another program")),
        FailureCode::ReferenceLocked => ClientFailure::ReferenceLocked {
            branch: request.reference.trim_start_matches("refs/heads/").to_string(),
            words: String::new(),
        },
        FailureCode::Rejected => ClientFailure::Refused,
        FailureCode::BranchKeepsChanging => ClientFailure::BranchKeepsChanging {
            branch: request.reference.trim_start_matches("refs/heads/").to_string(),
            times: crate::service_land::LANDING_ATTEMPTS,
        },
        FailureCode::BranchMoved => ClientFailure::BranchMoved { branch: request.reference.trim_start_matches("refs/heads/").to_string() },
        FailureCode::StepTooSlow => ClientFailure::StepTooSlow,
        FailureCode::Unavailable => ClientFailure::Unavailable,
    };
}

/// What:
///  Sends a commit request and waits until it is answered.
/// Why:
///   The same identity goes with every send,
/// so a service that landed the commit and died before answering reports it
///  as already landed,
/// and nothing lands twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await submit(config, request, notices)
/// ```
pub fn submit(config: &ClientConfig, request: &CommitRequest, notices: &mut dyn Write) -> Result<Submitted, ClientFailure> {
    return submit_traced(config, request, notices, &mut SubmitTrace::default());
}

/// What:
///  `submit`,
/// leaving in `trace` how the service was reached and how long the answer took.
/// Why:
///   The wrapper's timing record states the tries,
/// the retires and the releases of the reach,
/// the queue positions the service
///  reported and the time between the request being written and the answer;
/// the trace is filled as the send proceeds,
/// so a
///  send that fails still reports what it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await submitTraced(config, request, notices, trace)
/// ```
pub fn submit_traced(config: &ClientConfig, request: &CommitRequest, notices: &mut dyn Write, trace: &mut SubmitTrace) -> Result<Submitted, ClientFailure> {
    let began: Instant = Instant::now();
    let answered = submit_loop(config, request, notices, trace);
    trace.total += began.elapsed();
    return answered;
}

/// What:
///  Renews the modification time of the command's scratch directory when it is due:
/// at once the first time,
/// then as often as
///  the config says.
/// Why:
///  A wait longer than the sweep's age must never have its directory swept,
/// whatever the service sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (settings.keepFresh && (lastRenewed === undefined || elapsed(lastRenewed) >= settings.keepFresh.every)) { renew(settings.keepFresh.path); lastRenewed = now(); }
/// ```
fn renew_scratch(settings: &ClientConfig, last_renewed: &mut Option<Instant>) {
    let Some(fresh) = settings.keep_fresh.as_ref() else {
        return;
    };
    let due: bool = match *last_renewed {
        Some(at) => at.elapsed() >= fresh.every,
        None => true,
    };
    if due {
        crate::scratch_location::renew(fresh.path.as_path());
        *last_renewed = Some(Instant::now());
    }
}

/// The sends and the waits of `submit_traced`.
///
/// The wait for the answer lasts as long as the service keeps sending,
/// and ends when the service goes
///  silent for the silence bound or at `ANSWER_CEILING` (twenty minutes,
/// `wait_ceilings.rs`),
/// where the request is withdrawn.
/// The request is then sent again with the same identity,
/// so nothing lands twice,
/// and the
///  command ends when every try has met silence or a closed connection.
fn submit_loop(config: &ClientConfig, request: &CommitRequest, notices: &mut dyn Write, trace: &mut SubmitTrace) -> Result<Submitted, ClientFailure> {
    let line: String = encode_client(&ClientMessage::Commit(Box::new(request.clone())));
    let mut announced: bool = false;
    let mut breaks: usize = 0;
    // The place the person was last told.
    let mut told_place: Option<Option<usize>> = None;
    // When the request line first reached the service, which is where the wait the telling answers begins.
    let mut parked: Option<Instant> = None;
    let mut holder: Option<String> = None;
    let mut service_seen: bool = false;
    let mut last_break_was_silence: bool = false;
    // The scratch directory of a command that waits is renewed at once and then as often as the config says, whatever the
    //  service sends, so a wait longer than the sweep's age never has its directory swept.
    let waiting_since: Instant = Instant::now();
    let mut last_blocker: Option<String> = None;
    // Whether the service said this request is inside its stuck branch write, and when the extra wait for that write ends.
    let mut inside_write: bool = false;
    // What the service said about the write this command's own request is inside, held until the wait has mattered, and the
    // words of it the person was told last.
    let mut inside_words: Option<String> = None;
    let mut inside_told: Option<String> = None;
    // The wait for the answer ends here, and the extra wait for a write the request is inside ends the extra wait after it: both
    // are measured from the start of the wait, not from the moment a loop first notices them.
    let answer_deadline: Instant = waiting_since + config.answer_ceiling;
    let mut last_renewed: Option<Instant> = None;
    loop {
        // What:
        //  The place the service named,
        // which belongs to this connection and to no other.
        // Why:
        //  The place is a position in a batch the answering service decided,
        // and a request sent again goes to a service that decides
        //  its own,
        // so a place read here cannot outlive the connection it came from.
        // It is therefore held inside this loop,
        // and
        //  what the person was already told (`told_place`) is what outlives it.
        let mut last_place: Option<(usize, usize)> = None;
        // What:
        //  The connection that can answer this request,
        // or the end of the command when no service can.
        // Why:
        //  A request that was already sent may have landed,
        // and failing to reach a service again cannot say it did not:
        // the service that took it may have landed it and died,
        // or a service of another release may hold it now.
        //  Such a failure therefore ends with the words for a result that is not known,
        // as a silence does,
        // and the landing path looks for the commit before it says anything
        // (`service_commit_looking_for_what_landed.rs`).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const reader = await reachCurrent(config); // after a send: a failure means 'not known', never 'not made'
        // ```
        let mut reader: BufReader<UnixStream> = match reach_current(config, notices, &mut announced, &mut trace.reach, service_seen) {
            Ok(reached) => reached,
            Err(failure) if trace.sends > 0 => {
                debug(TAG, format!("the service could not be reached again after the request was sent ({failure:?}), so the result is not known").as_str());
                return Err(ClientFailure::ServiceSilent);
            }
            Err(failure) => return Err(failure),
        };
        service_seen = true;
        let mut payload: String = line.clone();
        payload.push('\n');
        if reader.get_mut().write_all(payload.as_bytes()).is_err() {
            breaks += 1;
            trace.reconnects += 1;
            if breaks > RECONNECTS {
                return Err(ClientFailure::MayNotHaveLanded);
            }
            continue;
        }
        trace.sends += 1;
        if parked.is_none() {
            parked = Some(Instant::now());
        }
        renew_scratch(config, &mut last_renewed);
        let written_at: Instant = Instant::now();
        let answered: Option<Result<Submitted, ClientFailure>> = loop {
            // Where the commit stands is said here, before each wait, so a service that sends nothing at all is answered
            //  in the same words as one that only beats: neither has started on this commit.
            say_where_it_stands(notices, config, parked, last_place, &mut told_place);
            say_the_write_it_is_inside(notices, config, parked, inside_words.as_deref(), &mut inside_told);
            let heard: Heard = listen(&mut reader);
            if matches!(heard, Heard::Message(_)) {
                renew_scratch(config, &mut last_renewed);
            }
            // A notice that arrives with the ceiling counts, so it is read before the ceiling is judged.
            if matches!(heard, Heard::Message(ServiceMessage::InsideWrite { .. })) {
                inside_write = true;
            }
            // The wait ends at the ceiling whatever the service sends: the request is withdrawn, and the answer to that is
            //  the command's end (landed after all, or the error).
            //  A request the service says is inside its branch write is not withdrawn: its commit may be on the branch already, so the
            //  client waits the short extra wait for that write (owner, 2026-11-04, "Short extra wait"), listening for the landing,
            //  and then reports that the result is not known; the write is left running.
            if waiting_since.elapsed() >= config.answer_ceiling && !matches!(heard, Heard::Message(ServiceMessage::Landed { .. } | ServiceMessage::Held { .. } | ServiceMessage::RetryPrepare { .. } | ServiceMessage::Failed { .. })) {
                if !inside_write {
                    trace.answer_wait += written_at.elapsed();
                    return withdraw(config, request.id.as_str(), last_blocker.take(), answer_deadline);
                }
                if Instant::now() >= answer_deadline + config.extra_wait {
                    trace.answer_wait += written_at.elapsed();
                    return Err(ClientFailure::WithdrawalUnconfirmed { blocker: last_blocker.take() });
                }
            }
            match heard {
                Heard::Message(ServiceMessage::Position { number, of }) => {
                    // Every place the service names is recorded, printed or not; the person is told it once the wait has
                    //  mattered, so a commit that lands at once says nothing new. The record keeps every place every service
                    //  named, which is what `positions` states; what may be printed is only ever this connection's place.
                    if last_place.map(place_number) != Some(number) {
                        last_place = Some((number, of));
                        trace.positions.push(number);
                    }
                }
                Heard::Message(ServiceMessage::Landed { commit, already, landing, .. }) => {
                    // What the landing spent and held, for the record of this command: the service measured it, and only its
                    //  answer can carry it.
                    trace.landing = landing;
                    break Some(Ok(Submitted::Landed { commit, already }));
                }
                Heard::Message(ServiceMessage::Held { commit, words, .. }) => break Some(Ok(Submitted::Held { commit, words })),
                Heard::Message(ServiceMessage::RetryPrepare { base, landed, .. }) => break Some(Ok(Submitted::PrepareAgain { base, landed })),
                Heard::Message(ServiceMessage::Failed { code: FailureCode::BranchMoved, .. }) => break Some(Ok(Submitted::Moved)),
                Heard::Message(ServiceMessage::Failed { code: FailureCode::IndexInUse, .. }) => {
                    break Some(Err(ClientFailure::IndexInUse(match holder.take() {
                        Some(who) => who,
                        None => String::from("Another program"),
                    })));
                }
                Heard::Message(ServiceMessage::Failed { code: FailureCode::ReferenceLocked, .. }) => {
                    // The service named the locks it left and the command that clears each; without that answer the
                    //  message names the lock by the branch's own name.
                    let words: String = match holder.take() {
                        Some(who) => who,
                        None => format!("The reference lock of {} stands in the way, and nothing proves that a Git process still holds it.", request.reference),
                    };
                    break Some(Err(ClientFailure::ReferenceLocked { branch: request.reference.trim_start_matches("refs/heads/").to_string(), words }));
                }
                Heard::Message(ServiceMessage::Failed { code, .. }) => break Some(Err(failure_for(code, request))),
                Heard::Message(ServiceMessage::Waiting { text }) => {
                    last_blocker = Some(text.trim().to_string());
                    trace.waiting_notices += 1;
                    trace.holder_kind = Some(holder_kind_after(trace.holder_kind, text.as_str()));
                    let _written = write!(notices, "{text}");
                }
                // The landing is inside a write: said of the commit, not of a holder, so the kind of holder is left as it was.
                Heard::Message(ServiceMessage::InsideWrite { text }) => {
                    last_blocker = Some(text.trim().to_string());
                    trace.waiting_notices += 1;
                    inside_words = Some(text);
                }
                Heard::Message(ServiceMessage::Holder { who }) => holder = Some(who),
                // Each message, this one too, restarts the silence bound; nothing else is needed.
                Heard::Message(ServiceMessage::Welcome { .. } | ServiceMessage::Heartbeat | ServiceMessage::Withdrawn { .. }) => {}
                Heard::Silent => {
                    last_break_was_silence = true;
                    break None;
                }
                Heard::Closed => {
                    last_break_was_silence = false;
                    break None;
                }
            }
            // Words about the write that came with this message are said at once when they differ from the ones already said,
            // so a write that has stayed stuck a new minute is not held until the beat after it.
            say_the_write_it_is_inside(notices, config, parked, inside_words.as_deref(), &mut inside_told);
        };
        trace.answer_wait += written_at.elapsed();
        if let Some(done) = answered {
            return done;
        }
        // The service went silent or the connection broke before an answer: the service died, hung or was replaced. Send the
        //  same request again.
        breaks += 1;
        if breaks > RECONNECTS {
            return Err(if last_break_was_silence { ClientFailure::ServiceSilent } else { ClientFailure::MayNotHaveLanded });
        }
        if last_break_was_silence {
            let _written = writeln!(notices, "{SILENT_NOTICE}");
        }
        std::thread::sleep(CONNECT_PAUSE * u32::try_from(breaks).unwrap_or(1));
    }
}

/// What:
///  Withdraws the request after the answer ceiling:
/// asks the service,
/// on a connection of its own,
/// to drop the request of this
///  identity,
/// and turns its answer into the command's end.
/// Why:
///   The owner's answer is that a wait that reaches its bound returns an error and the commit never lands later by itself.
/// The
///  service takes the withdrawal between two landings,
/// so either the commit landed first (the answer is `Landed`,
/// and the command
///  succeeds) or it never will (`Withdrawn`).
/// A service that does not answer within the withdrawal ceiling leaves it unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await askToWithdraw(config, id); // 'landed' | 'withdrawn' | undefined
/// ```
fn withdraw(config: &ClientConfig, id: &str, heard_blocker: Option<String>, answer_deadline: Instant) -> Result<Submitted, ClientFailure> {
    let minutes: u64 = config.answer_ceiling.as_secs() / 60;
    // What the service says it is stuck in on this connection replaces what it said on the first one: it is the newer.
    let mut blocker: Option<String> = heard_blocker;
    let mut announced: bool = true;
    let Ok(mut reader) = reach_current(config, &mut std::io::sink(), &mut announced, &mut ReachTrace::default(), true) else {
        return Err(ClientFailure::WithdrawalUnconfirmed { blocker });
    };
    let mut asking: String = encode_client(&ClientMessage::Withdraw { id: id.to_string() });
    asking.push('\n');
    if reader.get_mut().write_all(asking.as_bytes()).is_err() {
        return Err(ClientFailure::WithdrawalUnconfirmed { blocker });
    }
    let mut deadline: Instant = Instant::now() + config.withdraw_ceiling;
    while Instant::now() < deadline {
        // No read waits past the withdrawal's ceiling, whatever the silence bound says.
        let _timeout = reader.get_ref().set_read_timeout(Some(deadline.saturating_duration_since(Instant::now()).max(Duration::from_millis(1)).min(config.heartbeat_silence)));
        match listen(&mut reader) {
            Heard::Message(ServiceMessage::Landed { commit, .. }) => return Ok(Submitted::Landed { commit, already: true }),
            Heard::Message(ServiceMessage::Withdrawn { .. }) => return Err(ClientFailure::StillWithService { minutes, blocker }),
            Heard::Message(ServiceMessage::Waiting { text }) => blocker = Some(text.trim().to_string()),
            // The service says the request is inside its stuck write: the withdrawal cannot be answered before the write returns,
            // and the commit may be on the branch, so the wait ends at the same moment it would have ended had the notice come before
            // the ceiling (owner, 2026-11-04, "Short extra wait").
            Heard::Message(ServiceMessage::InsideWrite { text }) => {
                blocker = Some(text.trim().to_string());
                deadline = deadline.min(answer_deadline + config.extra_wait);
            }
            Heard::Message(_) => {}
            Heard::Silent | Heard::Closed => return Err(ClientFailure::WithdrawalUnconfirmed { blocker }),
        }
    }
    return Err(ClientFailure::WithdrawalUnconfirmed { blocker });
}

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
/// await wake(config, gitDir, reviewKey)
/// ```
pub fn wake(config: &ClientConfig, git_dir: &std::path::Path, review_key: Option<&ReviewKey>, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let mut line: String = encode_client(&ClientMessage::Work { git_dir: git_dir.to_path_buf(), review_key: review_key.cloned() });
    line.push('\n');
    let mut announced: bool = false;
    let mut reader: BufReader<UnixStream> = reach_current(config, notices, &mut announced, &mut ReachTrace::default(), false)?;
    if reader.get_mut().write_all(line.as_bytes()).is_err() {
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

/// The client's bounds and how it reads the service's lines.
#[cfg(test)]
#[path = "service_client_tests.rs"]
mod tests;
