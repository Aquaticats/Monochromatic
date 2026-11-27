//! Sending one request again:
//!  how often it is sent,
//!  how long it waits,
//!  the reply a position request gets,
//!  and the record of what a server still owes.

/// What a failed request means.
use super::failure::{
    Failure,
    classify_error,
};
/// Failures kept per request kind.
use super::trouble::{
    Channel,
    keep_trouble,
};
/// The class reported to the interface, for a failure an answer keeps.
use crate::language::reply::{
    LanguageReply,
    RequestKind,
    RequestOutcome,
};
/// The ticket a request was sent under.
use crate::language::request::{
    Ask,
    Ticket,
};
/// The worker whose state these steps change,
/// and the internal message a retry sends.
use crate::language::worker::{
    Internal,
    Worker,
};
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

/// How often one request is sent again:
///  a superseded one before it is reported as superseded,
/// and a hint or pull-diagnostics request that was superseded or timed out before the server is
/// recorded as owing the answer.
pub(super) const MAX_RETRIES: u32 = 3;

/// Wait before a superseded position or hint request is sent again.
pub(super) const RETRY_DELAY: Duration = Duration::from_millis(300);

/// Wait before a pull request is sent again when the server asked for that,
///  as Helix waits.
pub(super) const PULL_RETRY_DELAY: Duration = Duration::from_millis(500);

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
pub(super) fn replied(
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
pub(super) fn record(
    worker: &mut Worker,
    ticket: &Ticket,
    owes: bool,
) {
    let Some(index) = worker.session.index_of_identity(&ticket.server) else {
        return;
    };
    // `&mut` borrows the record's field so the assignment changes the stored value.
    let owed = &mut worker.session.servers[index].owed;
    match ticket.ask {
        Ask::Hints {
            ..
        } => owed.hints = owes,
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
pub(super) fn unanswered(
    worker: &mut Worker,
    ticket: Ticket,
    wait: Option<Duration>,
    error: &helix_lsp::Error,
) {
    let Some(delay) = wait else {
        tracing::debug!(server = %ticket.server.name, ask = ?ticket.ask, %error, "request failed; asking again would not change that");
        if let (
            Ask::Hints {
                ..
            },
            Failure::Failed(failure),
        ) = (ticket.ask, classify_error(error))
        {
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
    if let (
        Ask::Hints {
            ..
        },
        Failure::Failed(failure),
    ) = (ticket.ask, classify_error(error))
    {
        keep_trouble(worker, &ticket, Channel::Hints, Some(failure));
    }
    record(worker, &ticket, true);
}
