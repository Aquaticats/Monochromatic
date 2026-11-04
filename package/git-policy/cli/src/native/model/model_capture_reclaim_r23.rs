//! What:
//!  Model of reclamation during capture: a departure captures the acknowledgment of the record the state log selects, forces the file it
//!  captured, and returns success, while the lane's owner concurrently writes a checkpoint candidate (one that descends from the record or
//!  an unrelated one at a higher generation) and removes the old segment. The facts are the promised record, the candidate's content, its
//!  directory entry and the removal of the old segment's name, each visible and durable apart (`model_observe::World`). Recovery selects the
//!  root: the candidate once its content and name are durable, else the old segment while its record and name are durable.
//! Why:
//!  The row "reclamation during capture". The first version of this model counted the promise as covered whenever the old durable file
//!  remained; the component's `state_covers` checks the state recovery selects (review of step 1, finding 5), and an unrelated higher
//!  generation candidate is selected and does not cover. So the model has root selection, and the departure's handling of the three things
//!  that can happen to its capture: the selected root changes (it captures the new root again), the old segment is gone when it opens the
//!  path (`sync_prefix` captured `Selected` before `File::open`: ENOENT, it selects again), and a return before anything is durable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // departure: capture root; force it (ENOENT -> select again); return only if the durable root is or descends from the capture, else capture again
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
    /// The components' rule: the departure returns on a durable root that is or descends from its capture and captures again otherwise;
    /// the reclaimer removes the old segment only after a descending candidate is durable with its directory entry.
    Checked,
    /// Control: the departure never captures again, so a root that changed to an unrelated candidate leaves it with nothing to return on.
    NoSubstitution,
    /// Not a control: the reclaimer removes the old segment for any durable candidate. With the departure's substitution this is green, so
    /// the lineage restriction on removal is not what keeps a departure's promise (it keeps `state_covers` true for an acknowledgment taken
    /// before the removal); the test says so.
    ReclaimsAnyCandidate,
    /// Control: the reclaimer removes the old segment as soon as the candidate is visible, before it is durable.
    RemovesBeforeCandidateDurable,
    /// Control, a combined weakening (it allows a return straight from the capture and replaces the durable root by the visible one): the
    /// departure returns when the log as a reader sees it covers its capture, without forcing anything.
    ReturnsOnVisibleCover,
}

/// What the reclaimer's candidate is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Candidate {
    /// Not created yet.
    None,
    /// A checkpoint whose lineage names the promised record.
    Descends,
    /// A checkpoint at a higher generation that changed the document: its lineage does not name the promised record.
    Unrelated,
}

/// A root of the log.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Root {
    /// No segment is a root.
    Nothing,
    /// The old segment.
    Old,
    /// The candidate.
    Newer,
}

/// Where the departure is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Departure {
    /// Has captured nothing.
    Idle,
    /// Has captured the acknowledgment of this root.
    Captured(Root),
    /// Has forced the file of that root.
    Forced(Root),
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

/// The root recovery selects among the facts in `facts`.
fn root_of(facts: u32) -> Root {
    if facts & CANDIDATE != 0 && facts & CANDIDATE_NAME != 0 {
        return Root::Newer;
    }
    if facts & PROMISE != 0 && facts & REMOVED == 0 {
        return Root::Old;
    }
    return Root::Nothing;
}

/// The root a reader sees now.
pub fn visible_root(state: &State) -> Root {
    return root_of(state.world.visible);
}

/// The root a power loss would leave.
pub fn durable_root(state: &State) -> Root {
    return root_of(state.world.durable);
}

/// Whether `root` is the captured root or descends from it.
fn covers(state: &State, root: Root, captured: Root) -> bool {
    return match (root, captured) {
        (Root::Nothing, _) => false,
        (found, wanted) if found == wanted => true,
        (Root::Newer, Root::Old) => state.candidate == Candidate::Descends,
        _ => false,
    };
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
        if !matches!(after.departure, Departure::Returned) {
            after.departure = Departure::Dead;
        }
        // A candidate none of whose facts survived is no candidate; the next owner may write another.
        if after.world.visible & (CANDIDATE | CANDIDATE_NAME) == 0 {
            after.candidate = Candidate::None;
        }
        push_step(&mut found, "environment: power is lost", after);
    }
    departure_steps(state, &mut found);
    reclaimer_steps(state, &mut found);
    return found;
}

/// The departure's steps.
fn departure_steps(state: &State, found: &mut Vec<(String, State)>) {
    match state.departure {
        Departure::Idle => {
            let root: Root = visible_root(state);
            if root != Root::Nothing {
                let mut after: State = state.clone();
                after.departure = Departure::Captured(root);
                push_step(found, "departure: captures the acknowledgment of the record the log selects now", after);
            }
        }
        Departure::Captured(root) => {
            if root == Root::Old && state.world.visible & REMOVED != 0 {
                let mut after: State = state.clone();
                after.departure = Departure::Idle;
                push_step(found, "departure: opening the old segment finds it gone and the log is selected again", after);
            } else {
                let mut after: State = state.clone();
                after.world = if root == Root::Old { sync(after.world, PROMISE) } else { sync(sync(after.world, CANDIDATE), CANDIDATE_NAME) };
                after.departure = Departure::Forced(root);
                push_step(found, "departure: forces the file it captured", after);
            }
            if state.rules == Rules::ReturnsOnVisibleCover && covers(state, visible_root(state), root) {
                let mut after: State = state.clone();
                after.departure = Departure::Returned;
                push_step(found, "departure: verifies the log by name and returns success", after);
            }
        }
        Departure::Forced(root) => {
            if state.rules == Rules::ReturnsOnVisibleCover {
                return;
            }
            if covers(state, durable_root(state), root) {
                let mut after: State = state.clone();
                after.departure = Departure::Returned;
                push_step(found, "departure: verifies the log by name and returns success", after);
            } else if state.rules != Rules::NoSubstitution {
                let mut after: State = state.clone();
                after.departure = Departure::Idle;
                push_step(found, "departure: the selected root is not the one it captured, so it captures again", after);
            }
        }
        Departure::Returned | Departure::Dead => {}
    }
}

/// The reclaimer's steps.
fn reclaimer_steps(state: &State, found: &mut Vec<(String, State)>) {
    if state.candidate == Candidate::None && state.world.visible & PROMISE != 0 {
        for (kind, label) in [(Candidate::Descends, "owner: writes a checkpoint candidate that descends from the promised record"), (Candidate::Unrelated, "owner: writes a checkpoint candidate that changed the document")] {
            let mut after: State = state.clone();
            after.world = make_visible(make_visible(after.world, CANDIDATE), CANDIDATE_NAME);
            after.candidate = kind;
            push_step(found, label, after);
        }
    }
    if state.candidate == Candidate::None {
        return;
    }
    if state.world.durable & CANDIDATE == 0 && state.world.visible & CANDIDATE != 0 {
        let mut after: State = state.clone();
        after.world = sync(after.world, CANDIDATE);
        push_step(found, "owner: forces the candidate", after);
    }
    if state.world.durable & CANDIDATE_NAME == 0 && state.world.visible & CANDIDATE_NAME != 0 {
        let mut after: State = state.clone();
        after.world = sync(after.world, CANDIDATE_NAME);
        push_step(found, "owner: forces the candidate's directory entry", after);
    }
    let durable_candidate: bool = state.world.durable & CANDIDATE != 0 && state.world.durable & CANDIDATE_NAME != 0;
    let allowed: bool = match state.rules {
        Rules::Checked | Rules::NoSubstitution | Rules::ReturnsOnVisibleCover => durable_candidate && state.candidate == Candidate::Descends,
        Rules::ReclaimsAnyCandidate => durable_candidate,
        Rules::RemovesBeforeCandidateDurable => state.candidate == Candidate::Descends,
    };
    if allowed && state.world.visible & REMOVED == 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, REMOVED);
        push_step(found, "owner: removes the old segment", after);
    }
}

/// The property: a departure that returned success left a log with a root, as a power loss would leave it. (That the root it relied on at
/// the moment of the return covered its capture is a precondition of the return step; a later unrelated checkpoint supersedes it.)
pub fn invariant(state: &State) -> Option<String> {
    if state.departure == Departure::Returned && durable_root(state) == Root::Nothing {
        return Some("a departure returned success but a power loss leaves the log with no root".to_string());
    }
    return None;
}

/// At quiescence a departure that is still alive has returned or is dead: none is left with a capture and nothing to do.
pub fn terminal(state: &State) -> Option<String> {
    if matches!(state.departure, Departure::Idle | Departure::Captured(_) | Departure::Forced(_)) {
        return Some("a departure that captured a record can never return".to_string());
    }
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
