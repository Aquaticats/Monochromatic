//! What:
//!  Which requests of a batch a landing still makes,
//!  given which of their clients went away before their commits landed.
//! Why:
//!  A commit exists for the person who asked for it:
//!  nothing reads the answer of a client that is gone,
//!  and native Git makes no commit for a command that was ended.
//!  The landing asks this where the branch has still not moved,
//!  twice,
//!  and the batch is built again from the requests whose clients are still there,
//!  so one command's interrupt costs no other command's commit
//!  (`service_land_batch_body.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const wanted = placesStillWanted(landing, gone); const left = take(outcomes, gone, landing, announce, requests.length);
//! ```

use crate::service_index_lock::Announcer;
use crate::service_land_outcomes::Outcome;
use crate::service_protocol::CommitRequest;

/// What became of the places of a batch whose client went away.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Wanted {
    /// Every request of the batch still has a client,
    /// so this attempt goes on.
    All,
    /// Some requests lost their client,
    /// and the batch is built again from the rest.
    Some,
    /// No request of the batch has a client left,
    /// so nothing is landed.
    None,
}

/// What:
///  The places of `landing` whose client is not gone,
///  in the batch's own order.
/// Why:
///  A request whose client is gone is not landed and is not built again either:
///  the commits after it are made from the branch as it is,
///  so the batch is one commit shorter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landing.filter(([place]) => !gone.includes(place))
/// ```
pub(crate) fn places_still_wanted<'a>(landing: Vec<(usize, &'a CommitRequest)>, gone: &[usize]) -> Vec<(usize, &'a CommitRequest)> {
    let mut wanted: Vec<(usize, &CommitRequest)> = Vec::new();
    for (place, request) in landing {
        if !gone.contains(&place) {
            wanted.push((place, request));
        }
    }
    return wanted;
}

/// What:
///  Notes the places whose client went away,
///  writes that ending for every place known to be gone,
///  and answers whether any request is left.
/// Why:
///  The landing writes the outcome of the whole batch once per attempt,
///  so every place known to be gone is written again here,
///  and a place that goes away in the wait for the index lock is taken without a second walk of the batch in the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const went = landing.filter(([place]) => announce.clientIsGone(place)); went.forEach(([place]) => { gone.push(place); outcomes[place] = CLIENT_GONE }); ...
/// ```
pub(crate) fn take(outcomes: &mut [Outcome], gone: &mut Vec<usize>, landing: &[(usize, &CommitRequest)], announce: &dyn Announcer, requests: usize) -> Wanted {
    let mut went: bool = false;
    for (place, _request) in landing {
        if !announce.client_is_gone(*place) {
            continue;
        }
        went = true;
        if !gone.contains(place) {
            gone.push(*place);
        }
    }
    for place in gone.iter() {
        outcomes[*place] = Outcome::ClientGone;
    }
    if !went {
        return Wanted::All;
    }
    return if gone.len() == requests { Wanted::None } else { Wanted::Some };
}
