//! What:
//!  Model of reclamation during capture: a departure captures the acknowledgment of a record in the state log's current segment, forces
//!  it, and returns success, while the lane's owner concurrently replaces that segment by a checkpoint and removes the segment. The facts
//!  are the promised record, the candidate's content, the candidate's directory entry and the removal of the old segment's name, each
//!  visible and durable apart (`model_observe::World`); power loss, write-back, the reclaimer's and the departure's steps are steps.
//! Why:
//!  The row "reclamation during capture" and the clause of "captured authority valid through completion or verified substitution" that
//!  said survival under a concurrent reclaimer was not shown. A departure that forces a descriptor of a segment the reclaimer has already
//!  unlinked syncs bytes no name leads to; a reclaimer that removes the segment before its candidate is durable, or when the candidate does
//!  not descend from the promised record, leaves a durable log that no longer covers a promise the departure reported as kept. The model
//!  states the rule the components follow (`complete_checkpoint`, `state_covers`) and one single-defect control per clause.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // departure returns only if durableCover(world); reclaimer removes the old name only if candidate durable and descends from the promise
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};
use crate::model_observe::{World, make_visible, power_loss, sync};

/// The promised record is in the old segment's content.
pub const PROMISE: u32 = 1;
/// The candidate segment's first record (its content).
pub const CANDIDATE: u32 = 2;
/// The candidate segment's directory entry.
pub const CANDIDATE_NAME: u32 = 4;
/// The removal of the old segment's directory entry.
pub const REMOVED: u32 = 8;

/// How many power losses the search may inject.
pub const MAX_LOSSES: u8 = 1;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// The components' rule: the departure returns on a durable cover; the reclaimer removes the old segment only after its candidate
    /// descends from the promised record and is durable with its directory entry.
    Checked,
    /// Control: the reclaimer removes the old segment whatever its candidate is (a checkpoint that changed the document and recorded no
    /// lineage replaces the segment that holds the promise).
    ReclaimsAnyCandidate,
    /// Control: the reclaimer removes the old segment as soon as the candidate is visible, before it is durable.
    RemovesBeforeCandidateDurable,
    /// Control: the departure returns when the log covers the promise as a reader sees it now, without making the cover durable.
    ReturnsOnVisibleCover,
}

/// What the reclaimer's candidate is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Candidate {
    /// Not created yet.
    None,
    /// A checkpoint whose lineage names the promised record.
    Descends,
    /// A checkpoint that changed the document: its lineage does not name the promised record.
    Unrelated,
}

/// Where the departure is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Departure {
    /// Has captured nothing.
    Idle,
    /// Has captured the acknowledgment (the file, offset and hash of the promised record).
    Captured,
    /// Has forced the descriptor it captured.
    Forced,
    /// Returned success to the person.
    Returned,
    /// Power was lost before it returned.
    Dead,
}

/// The model's state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The facts of the log.
    pub world: World,
    /// The reclaimer's candidate.
    pub candidate: Candidate,
    /// Where the departure is.
    pub departure: Departure,
    /// Power losses so far.
    pub losses: u8,
}

/// The start state: the promised record is in the old segment and visible only.
pub fn initial(rules: Rules) -> State {
    return State { rules, world: make_visible(World { visible: 0, durable: 0 }, PROMISE), candidate: Candidate::None, departure: Departure::Idle, losses: 0 };
}

/// Whether the log as a reader sees it covers the promise: the old name is there with the record, or the candidate descends from it.
fn visible_cover(state: &State) -> bool {
    let old_there: bool = state.world.visible & PROMISE != 0 && state.world.visible & REMOVED == 0;
    let replaced: bool = state.world.visible & CANDIDATE != 0 && state.world.visible & CANDIDATE_NAME != 0 && state.candidate == Candidate::Descends;
    return old_there || replaced;
}

/// Whether the log that a power loss would leave covers the promise.
pub fn durable_cover(state: &State) -> bool {
    let old_there: bool = state.world.durable & PROMISE != 0 && state.world.durable & REMOVED == 0;
    let replaced: bool = state.world.durable & CANDIDATE != 0 && state.world.durable & CANDIDATE_NAME != 0 && state.candidate == Candidate::Descends;
    return old_there || replaced;
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    for (fact, name) in [(PROMISE, "the promised record"), (CANDIDATE, "the candidate's content"), (CANDIDATE_NAME, "the candidate's directory entry"), (REMOVED, "the removal of the old segment")] {
        if state.world.visible & fact != 0 && state.world.durable & fact == 0 {
            let mut after: State = state.clone();
            after.world = sync(after.world, fact);
            push_step(&mut found, &format!("environment: write-back persists {}", name), after);
        }
    }
    if state.losses < MAX_LOSSES {
        let mut after: State = state.clone();
        after.world = power_loss(after.world);
        after.losses += 1;
        if matches!(after.departure, Departure::Captured | Departure::Forced | Departure::Idle) {
            after.departure = Departure::Dead;
        }
        // A candidate none of whose facts survived is no candidate; the next owner may write another. One of which only a part survived stays
        // as it is, and cannot cover anything (a cover needs both the content and the name).
        if after.world.visible & (CANDIDATE | CANDIDATE_NAME) == 0 {
            after.candidate = Candidate::None;
        }
        push_step(&mut found, "environment: power is lost", after);
    }
    // The departure.
    match state.departure {
        Departure::Idle => {
            let mut after: State = state.clone();
            after.departure = Departure::Captured;
            push_step(&mut found, "departure: captures the acknowledgment of the promised record in the current segment", after);
        }
        Departure::Captured => {
            let mut after: State = state.clone();
            after.world = sync(after.world, PROMISE);
            after.departure = Departure::Forced;
            push_step(&mut found, "departure: forces the segment it captured", after);
        }
        Departure::Forced | Departure::Returned | Departure::Dead => {}
    }
    if matches!(state.departure, Departure::Captured | Departure::Forced) && state.candidate == Candidate::Descends && state.world.visible & CANDIDATE != 0 && state.world.visible & CANDIDATE_NAME != 0 {
        let mut after: State = state.clone();
        after.world = sync(sync(after.world, CANDIDATE), CANDIDATE_NAME);
        push_step(&mut found, "departure: forces the candidate that replaced the segment", after);
    }
    if state.departure == Departure::Forced || (state.rules == Rules::ReturnsOnVisibleCover && state.departure == Departure::Captured) {
        let covered: bool = if state.rules == Rules::ReturnsOnVisibleCover { visible_cover(state) } else { durable_cover(state) };
        if covered {
            let mut after: State = state.clone();
            after.departure = Departure::Returned;
            push_step(&mut found, "departure: verifies the log by name and returns success", after);
        }
    }
    // The reclaimer.
    if state.candidate == Candidate::None && state.world.visible & PROMISE != 0 {
        for (kind, label) in [(Candidate::Descends, "owner: writes a checkpoint candidate that descends from the promised record"), (Candidate::Unrelated, "owner: writes a checkpoint candidate that changed the document")] {
            let mut after: State = state.clone();
            after.world = make_visible(make_visible(after.world, CANDIDATE), CANDIDATE_NAME);
            after.candidate = kind;
            push_step(&mut found, label, after);
        }
    }
    if state.candidate != Candidate::None {
        if state.world.durable & CANDIDATE == 0 && state.world.visible & CANDIDATE != 0 {
            let mut after: State = state.clone();
            after.world = sync(after.world, CANDIDATE);
            push_step(&mut found, "owner: forces the candidate", after);
        }
        if state.world.durable & CANDIDATE_NAME == 0 && state.world.visible & CANDIDATE_NAME != 0 {
            let mut after: State = state.clone();
            after.world = sync(after.world, CANDIDATE_NAME);
            push_step(&mut found, "owner: forces the candidate's directory entry", after);
        }
        let durable_candidate: bool = state.world.durable & CANDIDATE != 0 && state.world.durable & CANDIDATE_NAME != 0;
        let allowed: bool = match state.rules {
            Rules::Checked | Rules::ReturnsOnVisibleCover => durable_candidate && state.candidate == Candidate::Descends,
            Rules::ReclaimsAnyCandidate => durable_candidate,
            Rules::RemovesBeforeCandidateDurable => state.candidate == Candidate::Descends,
        };
        if allowed && state.world.visible & REMOVED == 0 {
            let mut after: State = state.clone();
            after.world = make_visible(after.world, REMOVED);
            push_step(&mut found, "owner: removes the old segment", after);
        }
    }
    return found;
}

/// The property: a departure that returned success left a log that covers its promise, as a power loss would leave it.
pub fn invariant(state: &State) -> Option<String> {
    if state.departure == Departure::Returned && !durable_cover(state) {
        return Some("a departure returned success but the log a power loss would leave does not cover the record it acknowledged".to_string());
    }
    return None;
}

/// Nothing to check at quiescence beyond the invariant.
pub fn terminal(_state: &State) -> Option<String> {
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 100_000, max_depth: 40 };
}

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, terminal, limits());
}
