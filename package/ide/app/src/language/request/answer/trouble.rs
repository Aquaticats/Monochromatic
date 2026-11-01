//! The failure each server last had for a request kind,
//!  kept so the entry for the displayed file can name it.

/// The ticket of the request that finished.
use super::Ticket;
/// The failure to keep.
use crate::language::reply::RequestFailure;
/// The worker whose state these steps change.
use crate::language::worker::Worker;

/// Which of a server's request kinds a kept failure belongs to.
#[derive(Clone, Copy)]
pub(super) enum Channel {
    /// Inlay hint requests.
    Hints,
    /// Hover requests.
    Hover,
}

/// What:
///  Keep or clear the last failure of one request kind for the server that was asked.
///  `Option<RequestFailure>` is the failure to keep,
///  or nothing to clear it.
/// Why:
///  A server that is running but fails these requests looks like one without the feature;
///  the entry the Language module publishes for the displayed file names the failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepTrouble(worker: Worker, ticket: Ticket, channel: Channel, failure?: RequestFailure): void
/// ```
pub(super) fn keep_trouble(
    worker: &mut Worker,
    ticket: &Ticket,
    channel: Channel,
    failure: Option<RequestFailure>,
) {
    let Some(index) = worker.session.index_of_identity(&ticket.server) else {
        return;
    };
    let trouble = &mut worker.session.servers[index].trouble;
    let slot = match channel {
        Channel::Hints => &mut trouble.hints,
        Channel::Hover => &mut trouble.hover,
    };
    if *slot != failure {
        tracing::debug!(server = %ticket.server.name, failure = ?failure, "the failure kept for a request kind changed");
        *slot = failure;
    }
}
