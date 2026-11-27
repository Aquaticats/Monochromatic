//! What:
//!  The words a client of one landing batch reads while it waits:
//!  what the landing is waiting for,
//!  and the place each request of the batch has.
//! Why:
//!   The landing takes its announcer as a value,
//!  so the streams of the batch are the state each announcement is sent through;
//!  a request that is not the batch's first is told how many commits go first however short its wait,
//!  and the first only once its own wait has mattered,
//!  so a batch that begins at once says nothing new to anybody.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class AnnounceToStreams implements Announcer { announce(note) { for (const [, stream, ] of fresh) stream.send(note); } }
//! ```

use crate::service_index_lock::{
    Announce,
    Announcer,
};
use crate::service_protocol::ServiceMessage;
use crate::service_server_queue_types_and_send::Queued;
use std::time::Duration;
/// What:
///  Tells every client of a batch what the landing waits for.
/// Why:
///   The landing takes its announcer as a value;
/// the streams of the batch are the state each announcement is sent through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AnnounceToStreams implements Announcer { announce(note) { for (const stream of streams) stream.send(note); } }
/// ```
pub(crate) struct AnnounceToStreams<'a> {
    /// The requests of the batch,
    /// each with the stream its client listens on.
    pub(crate) fresh: &'a [Queued],
}

impl Announcer for AnnounceToStreams<'_> {
    fn announce(
        &self,
        note: Announce,
    ) {
        for (_, stream, _) in self.fresh {
            match &note {
                Announce::Waiting(text) => stream.send(&ServiceMessage::Waiting {
                    text: text.clone(),
                }),
                Announce::Holder(who) => stream.send(&ServiceMessage::Holder {
                    who: who.clone(),
                }),
            }
        }
    }

    /// Whether the client of the request at `place` of the batch is gone:
    /// its channel is closed when the task that writes its
    /// connection ends,
    /// which it does when the connection breaks.
    fn client_is_gone(
        &self,
        place: usize,
    ) -> bool {
        return match self.fresh.get(place) {
            Some((_, stream, _)) => stream.client_is_gone(),
            None => false,
        };
    }
}

/// The place each request of a batch is told,
/// as `(number, of)`,
/// in the order of the batch.
/// Why:
///   A request that is not the batch's first goes behind the ones before it,
/// so it is told how many commits go first however
///  short its wait;
/// the first is next,
/// and is told so only once its own wait has mattered,
/// so a batch that begins at once says
///  nothing new to anybody (and a client of an earlier release,
/// which prints every place it is told,
/// sees no new line for a quick commit).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placeNotices(3, 12_000, 10_000) // [[1,3],[2,3],[3,3]]
/// ```
pub(crate) fn place_notices(
    size: usize,
    waited: Duration,
    threshold: Duration,
) -> Vec<(usize, usize)> {
    let mut places: Vec<(usize, usize)> = Vec::new();
    for index in 0..size {
        if index == 0 && waited < threshold {
            continue;
        }
        places.push((index + 1, size));
    }
    return places;
}

/// How long the first request of a batch has waited since its connection was accepted,
/// which is the wait the telling answers.
/// Nothing when the batch holds no request,
/// which lands nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitedSinceAccepted(group[0]) // milliseconds
/// ```
pub(crate) fn waited_since_accepted(queued: &Queued) -> Duration {
    return queued.2.accepted.elapsed();
}
