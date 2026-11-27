//! Finished requests:
//!  failure classes,
//!  bounded retries,
//!  and where each kind of answer goes.
//!
//! The parts:
//!  `failure` says what a failed request means,
//!  `inlay_hints` stores one server's hints,
//!  `pull_diagnostics` stores a pull-diagnostics answer,
//!  `retries` asks again for what a server still owes,
//!  and `trouble` keeps the failure each server last had for a request kind.

/// What a failed request means, and whether an error is the server's own answer.
mod failure;
/// Storing one server's inlay hints, or asking for them again.
mod inlay_hints;
/// Storing a pull-diagnostics answer, or asking for it again.
mod pull_diagnostics;
/// Sending a request again, and recording what a server still owes.
mod retries;
/// The failure kept per server and request kind.
mod trouble;

/// Conversion of each answer against the ticket's text.
use super::convert::{
    definition_outcome,
    hover_outcome,
    references_outcome,
};
/// The request types this module completes,
///  and asking again for what a server still owes.
use super::{
    Answer,
    Payload,
    Ticket,
    catch_up,
};
/// Replies and their outcomes.
use crate::language::reply::RequestKind;
/// The worker whose state these steps change.
use crate::language::worker::Worker;
/// What a failed request means.
use failure::{
    Failure,
    answered,
    classify_error,
};
/// A stored hint answer.
use inlay_hints::hinted;
/// A stored pull-diagnostics answer.
use pull_diagnostics::pulled;
/// One request sent again.
use retries::replied;
/// Failures kept per request kind.
use trouble::{
    Channel,
    keep_trouble,
};

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
pub(in crate::language) fn finish(
    worker: &mut Worker,
    answer: Answer,
) {
    let Answer {
        ticket,
        payload,
    } = answer;
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
