//! Finished requests:
//!  failure classes,
//!  bounded retries,
//!  and where each kind of answer goes.

/// Conversion of each answer against the ticket's text.
use super::convert::{definition_outcome, hover_outcome, references_outcome};
/// The request types this module completes,
///  asking again for what a server still owes,
///  and
/// the hint lines wanted now.
use super::{Answer, Ask, Payload, Ticket, catch_up, hint_ask};
/// Hint shaping.
use crate::language::hints;
/// Replies and their outcomes.
use crate::language::reply::{LanguageReply, RequestFailure, RequestKind, RequestOutcome};
/// The worker whose state these steps change.
use crate::language::worker::{Internal, Worker};
/// The protocol's data types.
use helix_lsp::lsp;
/// What:
///  `Duration` is a time span.
/// Why:
///  Retries wait a fixed time before the request is sent again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RETRY_DELAY_MS = 300;
/// ```
use std::time::Duration;
/// Failures kept per request kind.
use trouble::{Channel, keep_trouble};

/// How often one request is sent again:
///  a superseded one before it is reported as superseded,
/// and a hint or pull-diagnostics request that was superseded or timed out before the server is
/// recorded as owing the answer.
const MAX_RETRIES: u32 = 3;

/// Wait before a superseded position or hint request is sent again.
const RETRY_DELAY: Duration = Duration::from_millis(300);

/// Wait before a pull request is sent again when the server asked for that,
///  as Helix waits.
const PULL_RETRY_DELAY: Duration = Duration::from_millis(500);

/// What:
///  How a failed request is treated.
///  An `enum` with data is a tagged union.
/// Why:
///  Codes `-32801` (content modified) and `-32800` (request cancelled) mean the server's
///      state moved under the request;
///  that is not a failure to show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Failure = { kind: 'superseded' } | { kind: 'failed'; failure: RequestFailure };
/// ```
enum Failure {
    /// The server gave up because its own state changed.
    Superseded,
    /// A real failure,
    ///  with its class.
    Failed(
        /// The class reported to the interface.
        RequestFailure,
    ),
}

/// Name the class of a request error.
fn classify_error(error: &helix_lsp::Error) -> Failure {
    // What: `match` on the borrowed error; `rpc.code.code()` reads the numeric protocol code.
    // Why: A protocol error, a timeout, and an ended server lead to different states.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (error instanceof RpcError) return [-32801, -32800].includes(error.code) ? superseded : failed(error);
    // ```
    return match error {
        helix_lsp::Error::Rpc(rpc) => {
            let code = rpc.code.code();
            if code == -32801 || code == -32800 {
                Failure::Superseded
            } else {
                Failure::Failed(RequestFailure::Rpc {
                    code,
                    // `clone` copies the server's message into the reply.
                    message: rpc.message.clone(),
                })
            }
        }
        helix_lsp::Error::Timeout(_) => Failure::Failed(RequestFailure::Timeout),
        helix_lsp::Error::StreamClosed => Failure::Failed(RequestFailure::StreamClosed),
        other => Failure::Failed(RequestFailure::Other(other.to_string())),
    };
}

/// What:
///  Send the reply of one position request,
///  retrying or dropping a superseded one.
///       `Result<RequestOutcome, helix_lsp::Error>` is the converted outcome or the request error.
/// Why:
///  A superseded answer for text that is no longer displayed is dropped;
///  for displayed text
///      the request is sent again a bounded number of times before it is reported as superseded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replied(worker: Worker, ticket: Ticket, kind: RequestKind, outcome: Outcome | Error): void
/// ```
fn replied(
    worker: &mut Worker,
    ticket: Ticket,
    kind: RequestKind,
    converted: Result<RequestOutcome, helix_lsp::Error>,
) {
    let outcome = match converted {
        Ok(found) => found,
        Err(error) => match classify_error(&error) {
            Failure::Failed(failure) => RequestOutcome::Failed(failure),
            Failure::Superseded => {
                // Only a request for the displayed text is worth asking again. One for older text
                // is replied as superseded with its old stamp, and the interface fence drops it
                // like every other stale result, so stale handling stays in one place.
                let current = worker.session.stamp() == Some(ticket.stamp);
                if current && ticket.attempt < MAX_RETRIES {
                    let mut again = ticket;
                    again.attempt += 1;
                    worker.timer(RETRY_DELAY, Internal::Retry(Box::new(again)));
                    return;
                }
                RequestOutcome::Superseded
            }
        },
    };
    let remaining = worker.session.answered(ticket.number);
    worker.outputs.reply(LanguageReply {
        request: ticket.number,
        stamp: ticket.stamp,
        // `Some(...)` is the "value present" variant: this reply came from a server process.
        server: Some(ticket.server),
        kind,
        position: ticket.position,
        remaining,
        outcome,
    });
}

/// What:
///  Mark whether one server owes the answer to a request the worker made on its own.
///       `&Ticket` lends the request;
///  `owes` is the new value of its flag.
/// Why:
///  An answer settles what was owed;
///  a request that stayed unanswered through its retries
///      is owed until the server is asked again.
///  Position requests have a waiting reader,
///  who
///      is told the outcome,
///  so nothing is recorded for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function record(worker: Worker, ticket: Ticket, owes: boolean): void {
///   const owed = worker.session.servers.find(row => same(row.identity, ticket.server))?.owed;
///   if (owed && ticket.ask.kind === 'hints') owed.hints = owes;
///   if (owed && ticket.ask.kind === 'pull') owed.pull = owes;
/// }
/// ```
fn record(worker: &mut Worker, ticket: &Ticket, owes: bool) {
    let Some(index) = worker.session.index_of_identity(&ticket.server) else {
        return;
    };
    // `&mut` borrows the record's field so the assignment changes the stored value.
    let owed = &mut worker.session.servers[index].owed;
    match ticket.ask {
        Ask::Hints { .. } => owed.hints = owes,
        Ask::Pull => owed.pull = owes,
        Ask::Position(_) => {}
    }
}

/// What:
///  Continue a failed request the worker made on its own for the displayed text.
///  `wait`
///       is "how long until it is sent again",
///  or nothing for a failure that asking again
///       cannot change.
///  `&helix_lsp::Error` lends the failure for the log.
/// Why:
///  Nobody else asks for hints or pull diagnostics again,
///  so a request that was superseded
///      or timed out is sent again a bounded number of times.
///  When those are used up the
///      server owes the answer,
///  and `request::catch_up` asks once more when it next sends
///      anything.
///  A failure the server stated itself is final for this text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unanswered(worker: Worker, ticket: Ticket, wait: number | undefined, error: Error): void {
///   if (wait === undefined) return;
///   if (ticket.attempt < MAX_RETRIES) worker.timer(wait, { type: 'retry', ticket: { ...ticket, attempt: ticket.attempt + 1 } });
///   else record(worker, ticket, true);
/// }
/// ```
fn unanswered(
    worker: &mut Worker,
    ticket: Ticket,
    wait: Option<Duration>,
    error: &helix_lsp::Error,
) {
    let Some(delay) = wait else {
        tracing::debug!(server = %ticket.server.name, ask = ?ticket.ask, %error, "request failed; asking again would not change that");
        if let (Ask::Hints { .. }, Failure::Failed(failure)) = (ticket.ask, classify_error(error)) {
            keep_trouble(worker, &ticket, Channel::Hints, Some(failure));
        }
        return;
    };
    if ticket.attempt < MAX_RETRIES {
        tracing::debug!(server = %ticket.server.name, ask = ?ticket.ask, attempt = ticket.attempt + 1, %error, "request was superseded or timed out; sending it again");
        let mut again = ticket;
        again.attempt += 1;
        worker.timer(delay, Internal::Retry(Box::new(again)));
        return;
    }
    tracing::debug!(server = %ticket.server.name, ask = ?ticket.ask, %error, "request was superseded or timed out through all its retries; it is asked again when the server next sends anything");
    if let (Ask::Hints { .. }, Failure::Failed(failure)) = (ticket.ask, classify_error(error)) {
        keep_trouble(worker, &ticket, Channel::Hints, Some(failure));
    }
    record(worker, &ticket, true);
}

/// What:
///  Store a pull-diagnostics answer,
///  or send the request again when the server asks for
///       that or stayed silent past its timeout.
/// Why:
///  The store fences the answer by stamp;
///  a failure whose data says `retriggerRequest` is
///      retried as Helix does (`helix-term/src/handlers/diagnostics.rs`),
///  a bounded number of
///      times.
///  A timed-out request is sent again at once,
///  because the wait already happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pulled(worker: Worker, ticket: Ticket, result: Report | Error): void
/// ```
fn pulled(
    worker: &mut Worker,
    ticket: Ticket,
    result: helix_lsp::Result<lsp::DocumentDiagnosticReportResult>,
) {
    if result.is_ok() {
        record(worker, &ticket, false);
    }
    let store = &mut worker.session.diagnostics;
    match result {
        Ok(lsp::DocumentDiagnosticReportResult::Report(lsp::DocumentDiagnosticReport::Full(
            report,
        ))) => {
            let full = report.full_document_diagnostic_report;
            let accepted = store.pulled_full(
                &ticket.server,
                ticket.encoding,
                ticket.stamp,
                full.result_id,
                full.items,
            );
            tracing::debug!(server = %ticket.server.name, accepted, "stored pulled diagnostics");
        }
        Ok(lsp::DocumentDiagnosticReportResult::Report(
            lsp::DocumentDiagnosticReport::Unchanged(report),
        )) => {
            let identifier = report.unchanged_document_diagnostic_report.result_id;
            let accepted = store.pulled_unchanged(&ticket.server, ticket.stamp, identifier);
            tracing::debug!(server = %ticket.server.name, accepted, "kept pulled diagnostics the server reported unchanged");
        }
        Ok(lsp::DocumentDiagnosticReportResult::Partial(_)) => {
            tracing::debug!(server = %ticket.server.name, "ignored a partial pull-diagnostics answer");
        }
        Err(error) => {
            let wants_retry = match &error {
                // `clone` copies the error data so it can be decoded by value.
                helix_lsp::Error::Rpc(rpc) => match rpc.data.clone() {
                    Some(data) => {
                        match serde_json::from_value::<lsp::DiagnosticServerCancellationData>(data)
                        {
                            Ok(decoded) => decoded.retrigger_request,
                            Err(_) => false,
                        }
                    }
                    None => false,
                },
                _ => false,
            };
            if worker.session.stamp() != Some(ticket.stamp) {
                tracing::debug!(server = %ticket.server.name, %error, "pull diagnostics failed for text that is no longer displayed");
                return;
            }
            // What: Pick the wait before the request is sent again. `matches!` tests the error's
            //       variant; `Some(...)` is the "value present" variant of `Option`.
            // Why: The server names its own wait by asking for a retrigger; a timeout already
            //      waited, so that request goes out at once; every other failure is final.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const wait = wantsRetry ? PULL_RETRY_DELAY : error instanceof TimeoutError ? 0 : undefined;
            // ```
            let wait = if wants_retry {
                Some(PULL_RETRY_DELAY)
            } else if matches!(error, helix_lsp::Error::Timeout(_)) {
                Some(Duration::ZERO)
            } else {
                None
            };
            unanswered(worker, ticket, wait, &error);
        }
    }
}

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
fn hinted(
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

/// True when the server itself produced the result,
///  as opposed to a timeout or an ended process.
fn is_server_answer(payload: &Payload) -> bool {
    return match payload {
        // `as_ref().err()` borrows the error of a failed result, or yields "nothing".
        Payload::Definition(result) => answered(result.as_ref().err()),
        Payload::References(result) => answered(result.as_ref().err()),
        Payload::Hover(result) => answered(result.as_ref().err()),
        Payload::Hints(result) => answered(result.as_ref().err()),
        Payload::Pull(result) => answered(result.as_ref().err()),
    };
}

/// What:
///  Whether a request error,
///  borrowed,
///  counts as the server's own answer;
///  `matches!` tests the error's variant.
/// Why:
///  A protocol error is still an answer;
///  silence and a closed stream are not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = (error?: Error) => !error || error instanceof RpcError;
/// ```
fn answered(error: Option<&helix_lsp::Error>) -> bool {
    return match error {
        Some(found) => matches!(found, helix_lsp::Error::Rpc(_)),
        None => true,
    };
}

/// What:
///  Complete one finished request.
///  `Answer` is moved in and taken apart.
/// Why:
///  An answer to a request sent for the displayed text also ends that server's hold on
///      unversioned diagnostics,
///  because it shows the server processed the latest change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finish(worker: Worker, answer: Answer): void
/// ```
pub(in crate::language) fn finish(worker: &mut Worker, answer: Answer) {
    let Answer { ticket, payload } = answer;
    let answered = is_server_answer(&payload);
    if answered
        && worker
            .session
            .diagnostics
            .answered(&ticket.server, ticket.stamp)
    {
        tracing::debug!(server = %ticket.server.name, "diagnostics hold ended by the server's first answer after the reload");
    }
    // What: An answer the server produced shows that it processes its input, so it is asked
    //       again for what it still owes. `if let Some(x) = ...` runs the block only when the
    //       server's record exists.
    // Why: This runs before the answer itself is handled, so a request whose last retry fails
    //      here is not asked again by its own failure, only by a later message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (answered && index !== undefined) catchUp(worker, index);
    // ```
    if answered && let Some(index) = worker.session.index_of_identity(&ticket.server) {
        catch_up(worker, index);
    }
    // `clone` copies the root path so the worker can be changed while targets are converted.
    let root = worker.root.clone();
    match payload {
        Payload::Definition(result) => {
            let outcome = match result {
                Ok(found) => Ok(definition_outcome(&ticket, &root, found)),
                Err(error) => Err(error),
            };
            replied(worker, ticket, RequestKind::Definition, outcome);
        }
        Payload::References(result) => {
            let outcome = match result {
                Ok(found) => Ok(references_outcome(&ticket, &root, found)),
                Err(error) => Err(error),
            };
            replied(worker, ticket, RequestKind::References, outcome);
        }
        Payload::Hover(result) => {
            // A hover that failed is kept for the entry the displayed file shows; an answer clears it.
            match &result {
                Ok(_) => keep_trouble(worker, &ticket, Channel::Hover, None),
                Err(error) => {
                    if let Failure::Failed(failure) = classify_error(error) {
                        keep_trouble(worker, &ticket, Channel::Hover, Some(failure));
                    }
                }
            }
            let outcome = match result {
                Ok(found) => Ok(hover_outcome(&ticket, found)),
                Err(error) => Err(error),
            };
            replied(worker, ticket, RequestKind::Hover, outcome);
        }
        Payload::Hints(result) => hinted(worker, ticket, result),
        Payload::Pull(result) => pulled(worker, ticket, result),
    }
}

/// The failure kept per server and request kind.
mod trouble;
