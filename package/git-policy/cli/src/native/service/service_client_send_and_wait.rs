//! What:
//!  Sends a commit request to the resident service and waits until it is answered:
//!  the one identity every send carries,
//!  the tries a service that went silent or closed its connection is given,
//!  the renewal of the command's scratch directory while it waits,
//!  and the trace the command's timing record is filled from.
//! Why:
//!   The same identity goes with every send,
//!  so a service that landed the commit and died before answering reports it
//!  as already landed,
//!  and nothing lands twice.
//!  What a wait ends with is another file's question (`service_client_wait_for_the_answer.rs`);
//!  this one only decides whether another try follows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await submitTraced(config, request, notices, trace); // landed, or prepare again, or a ClientFailure
//! ```

use crate::commit_timing::SubmitTrace;
use crate::diagnostic_log::debug;
use crate::service_client_listen_and_ask_to_retire::{
    Wrote,
    write_within,
};
use crate::service_client_reach_service::reach_current;
use crate::service_client_scratch_renewal::renew_scratch;
use crate::service_client_settings_and_watches::{
    CONNECT_PAUSE,
    ClientConfig,
    RECONNECTS,
};
use crate::service_client_wait_for_the_answer::{
    TheEndOfTheWait,
    TheWaitForTheAnswer,
};
use crate::service_client_what_a_request_came_to::Submitted;
use crate::service_client_why_withdrawn::WhyWithdrawn;
use crate::service_client_withdraw_a_request::withdraw;
use crate::service_messages::{
    ClientFailure,
    silent_notice,
};
use crate::service_protocol::{
    ClientMessage,
    CommitRequest,
    encode_client,
};
use std::io::{
    BufReader,
    Write,
};
use std::os::unix::net::UnixStream;
use std::time::Instant;

/// Tag used for diagnostics from this file.
const TAG: &str = "serviceClient";
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
pub fn submit(
    config: &ClientConfig,
    request: &CommitRequest,
    notices: &mut dyn Write,
) -> Result<Submitted, ClientFailure> {
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
pub fn submit_traced(
    config: &ClientConfig,
    request: &CommitRequest,
    notices: &mut dyn Write,
    trace: &mut SubmitTrace,
) -> Result<Submitted, ClientFailure> {
    let began: Instant = Instant::now();
    // What: The programs this shell finds its programs through, and whether it chose the transport they reach the remote
    //  through, go with the request that asks for the push.
    // Why:  The service takes the environment the user manager started it with, which the person cannot see or change; only
    //  this process knows the shell's, and a push must not fail where the same push typed in the same shell would succeed
    //  (`service_push_programs_held_in_memory`). They travel in the request and live in the service's memory alone: the two
    //  command lines can hold a password, so nothing of them is written anywhere.
    //  A request that already carries a list keeps it, so a caller that gave one is not overruled by this process's environment.
    let mut carried: CommitRequest = request.clone();
    if carried.programs.is_none() {
        carried.programs = Some(crate::service_push_programs_held_in_memory::of_this_process());
    }
    // The signals a waiting commit answers are watched only while it waits: a Ctrl-C before the commit exists, or after it
    //  landed, keeps the ending it has, and the actions that were in place come back with the watch.
    let stop_watch: crate::service_client_interrupted::StopWatch =
        crate::service_client_interrupted::watch_for_a_stop();
    let answered = submit_loop(config, &carried, notices, trace);
    drop(stop_watch);
    trace.total += began.elapsed();
    return answered;
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
fn submit_loop(
    config: &ClientConfig,
    request: &CommitRequest,
    notices: &mut dyn Write,
    trace: &mut SubmitTrace,
) -> Result<Submitted, ClientFailure> {
    let line: String = encode_client(&ClientMessage::Commit(Box::new(request.clone())));
    let mut announced: bool = false;
    let mut breaks: usize = 0;
    // The place the person was last told.
    let mut told_place: Option<Option<usize>> = None;
    // When the request line first reached the service, which is where the wait the telling answers begins.
    let mut parked: Option<Instant> = None;
    let mut holder: Option<String> = None;
    let mut service_seen: bool = false;
    let mut last_break_was_silence: bool;
    // The scratch directory of a command that waits is renewed at once and then as often as the config says, whatever the
    //  service sends, so a wait longer than the sweep's age never has its directory swept.
    let waiting_since: Instant = Instant::now();
    let mut last_blocker: Option<String> = None;
    // Whether the service said this request is inside its stuck branch write, and when the extra wait for that write ends.
    let mut inside_write: bool = false;
    // The words of the write this command's own request is inside that the person was told last.
    let mut inside_told: Option<String> = None;
    // The wait for the answer ends here, and the extra wait for a write the request is inside ends the extra wait after it: both
    // are measured from the start of the wait, not from the moment a loop first notices them.
    let answer_deadline: Instant = waiting_since + config.answer_ceiling;
    let mut last_renewed: Option<Instant> = None;
    loop {
        // What:
        //  A person who interrupts the commit withdraws its request.
        // Why:
        //  Native Git makes no commit when it is interrupted,
        //  and the request this command sent is the service's,
        //  so ending here would leave it to land with nobody to read the answer.
        //  The withdrawal is asked for and its answer is the command's end:
        //  a commit that landed before the signal came stays landed and says so,
        //  and one the service gives up was not made.
        if let Some((signal, sender)) = crate::service_client_interrupted::stop() {
            trace.answer_wait += waiting_since.elapsed();
            return withdraw(
                config,
                request.id.as_str(),
                last_blocker.take(),
                Instant::now(),
                WhyWithdrawn::Interrupted {
                    signal,
                    sender,
                },
            );
        }
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
        let mut reader: BufReader<UnixStream> = match reach_current(
            config,
            notices,
            &mut announced,
            &mut trace.reach,
            service_seen,
        ) {
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
        // What:
        //  The request is written within a bound, and the wait is on progress rather than on a clock; a give-up says
        //  "never taken" only when this command never put one whole request line on the wire.
        // Why:
        //  One `write_all` of a request waits in the kernel until the service reads it,
        //  and a service that stopped reading never does:
        // a request of one mebibyte fills both buffers of the connection and then waits with no end,
        //  so no ceiling of this client was ever looked at again
        //  (the audit of release 0.6.1-alpha.12 that stopped the service during the write of its request).
        //  `trace.sends` counts the attempts that put the whole line on the wire, so it is what tells the two ends apart:
        // with none, no line ever reached the service whole and, the service refusing a line that ends without its newline
        //  (`service_server_connection_handling.rs`), nothing of this commit can be taken or land later;
        // with one, an earlier attempt may already be the service's, so the result is not known and the commit's scratch
        //  tree stays with the service's claim on it (the review of this change, finding 1).
        let before_the_write: usize = trace.sends;
        match write_within(
            reader.get_ref(),
            payload.as_bytes(),
            crate::wait_ceilings::REQUEST_WRITE_SILENCE,
            crate::wait_ceilings::REQUEST_WRITE_CEILING,
        ) {
            Wrote::All => {}
            Wrote::Ended => {
                breaks += 1;
                trace.reconnects += 1;
                if breaks > RECONNECTS {
                    return Err(ClientFailure::MayNotHaveLanded);
                }
                continue;
            }
            Wrote::NothingTaken | Wrote::TooSlow if before_the_write > 0 => {
                // An earlier attempt delivered this request whole, so it may be the service's however this one ended.
                return Err(ClientFailure::MayNotHaveLanded);
            }
            Wrote::NothingTaken => {
                return Err(ClientFailure::ServiceNotReading {
                    took_nothing: true,
                });
            }
            Wrote::TooSlow => {
                return Err(ClientFailure::ServiceNotReading {
                    took_nothing: false,
                });
            }
        }
        trace.sends += 1;
        if parked.is_none() {
            parked = Some(Instant::now());
        }
        renew_scratch(config, &mut last_renewed);
        let written_at: Instant = Instant::now();
        // What: The wait for an answer to the request just written, with everything it remembers between two messages.
        // Why:  It is a step of its own (`service_client_wait_for_the_answer.rs`): it ends at the ceiling or on a stop and
        //  tells the person where the commit stands, while this loop only decides whether another try follows its end.
        let end: TheEndOfTheWait = {
            let mut the_wait: TheWaitForTheAnswer<'_> = TheWaitForTheAnswer {
                config,
                request,
                notices,
                trace,
                parked,
                told_place: &mut told_place,
                inside_told: &mut inside_told,
                last_blocker: &mut last_blocker,
                last_renewed: &mut last_renewed,
                inside_write: &mut inside_write,
                holder: &mut holder,
                waiting_since,
                answer_deadline,
                written_at,
            };
            the_wait.the_answer(&mut reader)
        };
        match end {
            // The wait itself counted this wait's time in the trace before it ended the command.
            TheEndOfTheWait::DecidedByTheWait(done) => return done,
            TheEndOfTheWait::Answered(done) => {
                trace.answer_wait += written_at.elapsed();
                return done;
            }
            TheEndOfTheWait::WentSilent => {
                trace.answer_wait += written_at.elapsed();
                last_break_was_silence = true;
            }
            TheEndOfTheWait::Closed => {
                trace.answer_wait += written_at.elapsed();
                last_break_was_silence = false;
            }
        } // What:
        //  The service went silent or the connection broke before an answer,
        //  and the same request is sent again with the same identity.
        // Why:
        //  A silence is the service saying nothing for the whole heartbeat silence,
        //  and the connection made again is what tells a
        //  service that stopped from one whose beats were late:
        // a service that still runs greets a new connection at once,
        //  however busy its
        //  landing is,
        // and a service that stopped greets nothing,
        // which ends the command at this bound rather than after another
        //  request.
        //  A commit never lands twice,
        // because the identity goes with every send and the service answers a repeated
        //  identity as already landed,
        // so sending it again costs nothing that was not already spent.
        breaks += 1;
        if breaks > RECONNECTS {
            return Err(if last_break_was_silence {
                ClientFailure::ServiceSilent
            } else {
                ClientFailure::MayNotHaveLanded
            });
        }
        if last_break_was_silence {
            let _written = writeln!(notices, "{}", silent_notice(config.heartbeat_silence));
        }
        std::thread::sleep(CONNECT_PAUSE * u32::try_from(breaks).unwrap_or(1));
    }
}
