//! What:
//!  One group of a batch, landed:
//!  its trees,
//!  its admission to the landing board,
//!  the landing itself,
//!  the removal of its trees,
//!  and the marker of the next batch.
//! Why:
//!   A group is what lands as one commit on one branch,
//!   and everything around that landing belongs to it:
//!   what the shutdown must wait for,
//!   which requests a client took back while this thread was stuck in a write,
//!   which answer each of them gets,
//!   and when a tree may be removed.
//!  The shutdown can refuse a landing that would begin after the stop,
//!  which is the one answer that ends the whole loop instead of this group,
//!  so the call answers which of the two happened.
//!  A group that is empty once the withdrawn requests are dropped lands nothing,
//!  takes its trees away and leaves the loop to the next group.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const group of groups) if (landTheGroup(group) === 'the-stop-began') break;
//! ```
use crate::service_busy::{
    GroupAdmission,
    LandingBoard,
};
use crate::service_captures::CaptureMemory;
use crate::service_index_prestage::start as start_prestage;
use crate::service_land::LandContext;
use crate::service_protocol::{
    FailureCode,
    ServiceMessage,
};
use crate::service_server_land_group::land_group;
use crate::service_server_queue_types_and_send::{
    Queued,
    id_of_queued,
    index_of_queued,
};
use crate::service_server_scratch_trees_of_a_group::{
    remove_the_trees,
    the_trees_a_group_names,
};
use crate::service_shadow_draw::DrawnShadows;
use crate::stop_of_the_service_noticed_by_its_children::begin_unfinished_work;
use std::collections::{
    HashMap,
    HashSet,
};
use std::sync::Arc;
use std::time::Duration;

/// What one group's landing ended in.
/// Why:
///  The shutdown waits for a landing and refuses one that would begin after the stop,
///  which is the only answer that ends the loop rather than the group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheGroupLanding = 'landed' | 'the-stop-began';
/// ```
pub enum TheGroupLanding {
    /// The group was landed or dropped, and the loop goes on with the next group.
    Landed,
    /// The shutdown began, so nothing more lands.
    TheStopBegan,
}

/// What:
///  Lands one group of a batch and answers every client of it.
/// Why:
///   The trees are read before the landing and removed after it,
///   the requests their clients withdrew in the meantime are dropped and remembered,
///   and a request the landing board could not admit is answered with a failure rather than landed.
///  The marker of the next batch is made and flushed here,
///  while the service is idle,
///  so the next commit only places it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landTheGroup(context, instance, seq, group, landed, captures, placeAfter, noteCeiling, drawn, board) // 'landed' | 'the-stop-began'
/// ```
pub fn land_the_group(
    context: &LandContext,
    instance: Option<&str>,
    seq: &mut u64,
    queued_group: Vec<Queued>,
    landed: &mut HashMap<String, String>,
    captures: &mut CaptureMemory,
    place_after: Duration,
    note_ceiling: Duration,
    drawn: &Arc<DrawnShadows>,
    board: &Arc<LandingBoard>,
) -> TheGroupLanding {
    let trees: Vec<(String, std::path::PathBuf)> = the_trees_a_group_names(queued_group.as_slice());
    // The shutdown waits for this landing and refuses one that would begin after the stop.
    let Ok(_landing_in_flight) = begin_unfinished_work(false, "a landing") else {
        return TheGroupLanding::TheStopBegan;
    };
    let index: Option<std::path::PathBuf> = queued_group.first().map(index_of_queued);
    // A request whose client was answered "withdrawn" while this thread was stuck in a write is dropped here, and the
    //  rest are marked as being landed, under one lock (`service_busy.rs`).
    let ids: Vec<String> = queued_group.iter().map(id_of_queued).collect();
    let admission: GroupAdmission = board.begin_group(ids.as_slice());
    for id in &admission.dropped {
        crate::service_log!(
            "dropped the request {id} that its client withdrew while the service was stuck in a write"
        );
    }
    for id in &admission.unavailable {
        crate::service_error!(
            "the request {id} could not be admitted to its group (the record of the landing thread cannot be written), so it is answered with a failure and not landed"
        );
    }
    // A dropped request is not answered here; an unavailable one is answered with a failure; the rest are landed.
    let mut unavailable: Vec<Queued> = Vec::new();
    let mut admitted: Vec<Queued> = Vec::new();
    for queued in queued_group {
        if admission.dropped.contains(&queued.0.id) {
            continue;
        }
        if admission.unavailable.contains(&queued.0.id) {
            unavailable.push(queued);
        } else {
            admitted.push(queued);
        }
    }
    for (request, stream, _) in &unavailable {
        stream.send(&ServiceMessage::Failed {
            id: request.id.clone(),
            code: FailureCode::Unavailable,
        });
    }
    let group: Vec<Queued> = admitted;
    if group.is_empty() {
        board.end_group(ids.as_slice());
        remove_the_trees(trees.as_slice(), &HashSet::new());
        return TheGroupLanding::Landed;
    }
    let mut handed_back: HashSet<String> = HashSet::new();
    land_group(
        context,
        instance,
        seq,
        group,
        landed,
        captures,
        place_after,
        note_ceiling,
        &mut handed_back,
        drawn,
    );
    board.end_group(ids.as_slice());
    // The requests are landed and answered, so no landing reads these any more; each removal runs beside this loop, bounded,
    //  and says so in the journal when it cannot finish (`service_scratch_release_removal.rs`).
    remove_the_trees(trees.as_slice(), &handed_back);
    // The batch after this one is numbered one more and lands on this index (a worktree has one), so its marker is
    //  made and flushed now, while the service is idle, and the next commit only places it.
    if let Some(next_index) = index {
        start_prestage(next_index.as_path(), *seq + 1);
    }
    return TheGroupLanding::Landed;
}
