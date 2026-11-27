//! What:
//!  Withdraws a request this command sent:
//!  asks the service,
//!  on a connection of its own,
//!  to drop the request of this identity,
//!  and turns its answer into the command's end.
//! Why:
//!  A wait that reached its bound,
//!  or a person who interrupted it,
//!  must not leave a commit that lands later with nobody to read the answer;
//!  the service takes a withdrawal between two landings,
//!  so the command ends either with the commit it landed or with the word that it never will.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const end = await withdraw(config, id, heardBlocker, answerDeadline, why); // 'landed' | 'withdrawn' | not known
//! ```

use crate::commit_timing::ReachTrace;
use crate::service_client_listen_and_ask_to_retire::{
    Heard,
    Wrote,
    listen,
    write_within,
};
use crate::service_client_reach_service::reach_current;
use crate::service_client_settings_and_watches::ClientConfig;
use crate::service_client_what_a_request_came_to::Submitted;
use crate::service_client_why_withdrawn::WhyWithdrawn;
use crate::service_messages::ClientFailure;
use crate::service_protocol::{
    ClientMessage,
    ServiceMessage,
    encode_client,
};
use std::time::{
    Duration,
    Instant,
};
/// What:
///  Withdraws the request:
/// asks the service,
/// on a connection of its own,
/// to drop the request of this
///  identity,
/// and turns its answer into the command's end.
/// Why:
///   The owner's answer is that a wait that reaches its bound returns an error and the commit never lands later by itself,
/// and an interrupted commit is not made at all.
/// The service takes the withdrawal between two landings,
/// so either the commit landed first (the answer is `Landed`,
/// and the command
///  succeeds) or it never will (`Withdrawn`).
/// A service that does not answer within the withdrawal ceiling leaves it unknown.
///  What the request was withdrawn for decides which of the two ends the command reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await askToWithdraw(config, id); // 'landed' | 'withdrawn' | undefined
/// ```
pub(crate) fn withdraw(
    config: &ClientConfig,
    id: &str,
    heard_blocker: Option<String>,
    answer_deadline: Instant,
    why: WhyWithdrawn,
) -> Result<Submitted, ClientFailure> {
    // What the service says it is stuck in on this connection replaces what it said on the first one: it is the newer.
    let mut blocker: Option<String> = heard_blocker;
    let mut announced: bool = true;
    let Ok(mut reader) = reach_current(
        config,
        &mut std::io::sink(),
        &mut announced,
        &mut ReachTrace::default(),
        true,
    ) else {
        return Err(why.not_known(blocker));
    };
    let mut asking: String = encode_client(&ClientMessage::Withdraw {
        id: id.to_string(),
    });
    asking.push('\n');
    // The one line of the withdrawal is written under the same bounds as a request, so a service that stops reading holds nothing.
    let written: Wrote = write_within(
        reader.get_ref(),
        asking.as_bytes(),
        crate::wait_ceilings::REQUEST_WRITE_SILENCE,
        crate::wait_ceilings::REQUEST_WRITE_CEILING,
    );
    if written != Wrote::All {
        return Err(why.not_known(blocker));
    }
    let mut deadline: Instant = Instant::now() + config.withdraw_ceiling;
    while Instant::now() < deadline {
        // No read waits past the withdrawal's ceiling, whatever the silence bound says.
        let _timeout = reader.get_ref().set_read_timeout(Some(
            deadline
                .saturating_duration_since(Instant::now())
                .max(Duration::from_millis(1))
                .min(config.heartbeat_silence),
        ));
        let heard: Heard = {
            let _awake = crate::service_client_interrupted::waking_the_read_of(reader.get_ref());
            listen(&mut reader)
        };
        match heard {
            Heard::Message(ServiceMessage::Landed {
                commit,
                ..
            }) => {
                return Ok(Submitted::Landed {
                    commit,
                    already: true,
                });
            }
            Heard::Message(ServiceMessage::Withdrawn {
                ..
            }) => return Err(why.it_will_not_land(blocker)),
            Heard::Message(ServiceMessage::Waiting {
                text,
            }) => blocker = Some(text.trim().to_string()),
            // The service says the request is inside its stuck write: the withdrawal cannot be answered before the write returns,
            // and the commit may be on the branch, so the wait ends at the same moment it would have ended had the notice come before
            // the ceiling (owner, 2026-11-04, "Short extra wait").
            Heard::Message(ServiceMessage::InsideWrite {
                text,
            }) => {
                blocker = Some(text.trim().to_string());
                deadline = deadline.min(answer_deadline + config.extra_wait);
            }
            Heard::Message(_) => {}
            Heard::Silent | Heard::Closed => return Err(why.not_known(blocker)),
        }
    }
    return Err(why.not_known(blocker));
}
