//! Storing one server's inlay hints:
//!  the lines they cover,
//!  and asking again when the answer was superseded or timed out.

/// What a failed request means.
use super::failure::{
    Failure,
    classify_error,
};
/// The retry bounds and the two steps that use them.
use super::retries::{
    RETRY_DELAY,
    record,
    unanswered,
};
/// Failures kept per request kind.
use super::trouble::{
    Channel,
    keep_trouble,
};
/// Hint shaping.
use crate::language::hints;
/// The class of a failure the answer keeps.
use crate::language::reply::RequestFailure;
/// The ticket the request was sent under.
use crate::language::request::Ticket;
/// The kind of request a ticket carries,
///  and the hint lines wanted now.
use crate::language::request::{
    Ask,
    hint_ask,
};
/// The worker whose state these steps change.
use crate::language::worker::Worker;
/// The protocol's data types.
use helix_lsp::lsp;
/// What:
///  `Duration` is a time span.
/// Why:
///  A timed-out hint request is sent again at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ZERO_WAIT = 0;
/// ```
use std::time::Duration;

/// What:
///  Store one server's hints for the displayed text,
///  or send a superseded or timed-out
///       request again.
/// Why:
///  Hints are latest-value state:
///  an answer for another revision,
///  or for lines that are no
///      longer wanted,
///  is simply dropped,
///  and so is its failure,
///  because a later request asked
///      afresh.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hinted(worker: Worker, ticket: Ticket, result: InlayHint[] | null | Error): void
/// ```
pub(super) fn hinted(
    worker: &mut Worker,
    ticket: Ticket,
    result: helix_lsp::Result<Option<Vec<lsp::InlayHint>>>,
) {
    if worker.session.stamp() != Some(ticket.stamp) {
        tracing::debug!(server = %ticket.server.name, "dropped inlay hints for text that is no longer displayed");
        return;
    }
    let Ask::Hints {
        first_line,
        last_line,
    } = ticket.ask
    else {
        return;
    };
    // What: `hint_ask` names the lines wanted now; `Some(ticket.ask)` wraps this request's own
    //       lines for the comparison.
    // Why: Two requests for different windows of the same text can be in flight at once, and
    //      the server may answer the older one last. Its answer, and its failure, belong to lines
    //      the reader no longer shows; the request for the current lines settles them, so this
    //      one is neither stored nor sent again, and what the server owes is left unchanged.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!sameAsk(hintAsk(worker.session), ticket.ask)) return;
    // ```
    if hint_ask(&worker.session) != Some(ticket.ask) {
        tracing::debug!(server = %ticket.server.name, first_line, last_line, "dropped inlay hints for lines that are no longer wanted");
        return;
    }
    match result {
        Ok(found) => {
            record(worker, &ticket, false);
            keep_trouble(worker, &ticket, Channel::Hints, None);
            let shaped = hints::shape(
                found.unwrap_or_default(),
                &ticket.text,
                ticket.encoding,
                &ticket.server,
            );
            // This server's earlier hints are dropped before the new ones are stored.
            let earlier = std::mem::take(&mut worker.session.hints);
            for entry in earlier {
                if entry.0 != ticket.server {
                    worker.session.hints.push(entry);
                }
            }
            worker.session.hints.push((ticket.server, shaped));
            worker.session.hint_lines = (first_line, last_line);
        }
        Err(error) => {
            // What: Pick the wait before the request is sent again, by the class of the failure.
            // Why: A superseded request is valid again a moment later. A timed-out request
            //      already waited the server's whole request timeout, so it goes out at once;
            //      without this the hints of the displayed text stay missing until the reader
            //      scrolls or the file changes again. A failure the server stated is final.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const wait = superseded ? RETRY_DELAY : timedOut ? 0 : undefined;
            // ```
            let wait = match classify_error(&error) {
                Failure::Superseded => Some(RETRY_DELAY),
                Failure::Failed(RequestFailure::Timeout) => Some(Duration::ZERO),
                Failure::Failed(_) => None,
            };
            unanswered(worker, ticket, wait, &error);
        }
    }
}
