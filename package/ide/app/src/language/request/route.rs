//! Routing of position requests to every server of the displayed document,
//!  and their retries.

/// Sending one request to one server,
///  and the hint request for the window reported last.
use super::{Ask, Sent, Ticket, dispatch, hint_ask};
/// Exited servers are started again by an explicit position request.
use crate::language::attach;
/// Replies name the server process that answered.
use crate::language::identity::ServerIdentity;
/// Replies and their outcomes.
use crate::language::reply::{LanguageReply, PositionRequest, RequestFailure, RequestOutcome};
/// The worker whose state these steps change.
use crate::language::worker::Worker;

/// What:
///  Route one position request to every server that serves the displayed document.
/// Why:
///  States that exist before any wire traffic are replied at once:
///  no server,
///  still
///      starting,
///  out of sync,
///  unsupported.
///  Each asked server answers on its own later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function position(worker: Worker, number: number, request: PositionRequest): Promise<void>
/// ```
pub(in crate::language) async fn position(
    worker: &mut Worker,
    number: u64,
    request: PositionRequest,
) {
    if worker.session.stamp() != Some(request.stamp) {
        tracing::debug!(
            ?request,
            "dropped a request for text that is no longer displayed"
        );
        return;
    }
    if attach::revive(worker).await {
        tracing::info!("started exited language servers again for an explicit request");
    }
    let in_bounds = match worker.session.document.as_ref() {
        Some(document) => request.position <= document.text.len_chars(),
        None => false,
    };
    // `Vec::new()` creates an empty growable list for the replies that need no wire traffic.
    let mut immediate: Vec<(Option<ServerIdentity>, RequestOutcome)> = Vec::new();
    let mut spawned = 0;
    if !in_bounds {
        let failure =
            RequestFailure::Other("the position is outside the displayed text".to_string());
        immediate.push((None, RequestOutcome::Failed(failure)));
    } else {
        for index in 0..worker.session.servers.len() {
            if !worker.session.servers[index].attached {
                continue;
            }
            let server = Some(worker.session.servers[index].identity.clone());
            let sent = dispatch(
                worker,
                index,
                number,
                Ask::Position(request.kind),
                request.position,
                0,
            );
            match sent {
                Sent::Spawned => spawned += 1,
                Sent::Starting => immediate.push((server, RequestOutcome::Starting)),
                Sent::Unsynchronized => immediate.push((server, RequestOutcome::Unsynchronized)),
                Sent::Unsupported => immediate.push((server, RequestOutcome::Unsupported)),
                Sent::Skipped => {}
            }
        }
        if spawned == 0 && immediate.is_empty() {
            immediate.push((None, RequestOutcome::NoServer));
        }
    }
    if spawned > 0 {
        worker.session.pending.push((number, spawned));
    }
    for (server, outcome) in immediate {
        worker.outputs.reply(LanguageReply {
            request: number,
            stamp: request.stamp,
            server,
            kind: request.kind,
            position: request.position,
            remaining: spawned,
            outcome,
        });
    }
    return;
}

/// What:
///  Send a request again,
///  if its text is still displayed and its server still runs:
///  a
///       superseded one,
///  or one of the worker's own that timed out.
/// Why:
///  A server answers `-32801` or `-32800` when its own state moved under the request;
///  the
///      same question is then valid a moment later.
///  A timed-out hint or diagnostics request has
///      nobody who would ask again,
///  so the worker does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retry(worker: Worker, ticket: Ticket): void
/// ```
pub(in crate::language) fn retry(worker: &mut Worker, ticket: Ticket) {
    let current = worker.session.stamp() == Some(ticket.stamp);
    let record = worker.session.index_of_identity(&ticket.server);
    // What: A hint request is asked again for the window reported last, which `hint_ask` reads;
    //       every other request is asked again exactly as it was. `Some(...)` is the "value
    //       present" variant of `Option`.
    // Why: A timed-out request is sent again a whole request timeout later; by then the reader
    //      may have scrolled, and hints for the earlier lines must not replace the current ones.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ask = ticket.ask.kind === 'hints' ? hintAsk(worker.session) : ticket.ask;
    // ```
    let ask = match ticket.ask {
        Ask::Hints { .. } => hint_ask(&worker.session),
        other => Some(other),
    };
    let sent = match (current, record, ask) {
        (true, Some(index), Some(wanted)) => dispatch(
            worker,
            index,
            ticket.number,
            wanted,
            ticket.position,
            ticket.attempt,
        ),
        _ => Sent::Skipped,
    };
    if matches!(sent, Sent::Spawned) {
        return;
    }
    // Only a position request has someone waiting for a reply.
    if let Ask::Position(kind) = ticket.ask {
        let remaining = worker.session.answered(ticket.number);
        worker.outputs.reply(LanguageReply {
            request: ticket.number,
            stamp: ticket.stamp,
            server: Some(ticket.server),
            kind,
            position: ticket.position,
            remaining,
            outcome: RequestOutcome::Superseded,
        });
    }
}
