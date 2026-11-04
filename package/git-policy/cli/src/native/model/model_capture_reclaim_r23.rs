//! What:
//!  Model of reclamation during capture: a departure captures the acknowledgment of the record the state log selects, forces the file it
//!  captured and the closure that record retains, confirms against the root the log selects now, and returns; while the lane's owner
//!  concurrently writes a checkpoint candidate and removes the old segment. The facts are the promised record, the candidate's content, its
//!  directory entry and the removal of the old segment's name, each visible and durable apart (`model_observe::World`). The model keeps two
//!  things apart that the first version merged: the root the log selects now (what `state_covers` reads) and the root a power loss leaves
//!  (what survives), and it adds the retained support: what the promised record retains (the files its closure holds) and which candidates
//!  keep it.
//! Why:
//!  The row "reclamation during capture" and the two reviews of steps 1 and 2. The invariant is not "some root exists": a returned
//!  departure's promise is its record and what that record retains, so after any later replacement the root a power loss leaves must keep
//!  that support (or have discharged it, which the component's non-drop check admits only with a ledger proof). A candidate that is
//!  unrelated to the record may legitimately supersede it (a later state) as long as it retains the support. Recapture gives progress before
//!  the return; it does not preserve anything after it. What preserves after the return is the non-drop rule plus the durable publication of
//!  the candidate and the forcing of the dependencies.
//!  Excluded on purpose, and covered elsewhere: more than one replacement of the segment (the component test
//!  `an_acknowledgment_survives_two_successive_seals`), the interleavings of the file force and the directory force (the component tests
//!  count and order the syncs), and a recapture budget larger than the model's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // departure: capture the selected root; force it and its closure (ENOENT or a changed selection: capture again, within the budget);
//! // return only if the root selected now covers the capture; invariant: after a return, the surviving root keeps the promised support
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
/// The files the promised record retains (the order file, the ledger, the requests): forced by the departure.
pub const SUPPORT: u32 = 16;

/// How many power losses the search may inject.
pub const MAX_LOSSES: u8 = 1;

/// How many times a departure may capture (the code's `attempts`, here the model's own small number).
pub const ATTEMPTS: u8 = 3;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// The components' rule: the departure forces the closure, confirms against the root selected now and captures again otherwise within its
    /// attempts; a candidate must keep what its predecessor retained (non-drop); the reclaimer removes the old segment only after a
    /// descending candidate is durable with its directory entry.
    Checked,
    /// Control: the departure never captures again, so a root that changed leaves it with nothing to return on.
    NoSubstitution,
    /// Not a control: the reclaimer removes the old segment for any durable candidate. Green, because the candidates the component can write
    /// all keep the support; this is the model's way of saying that lineage is not what keeps the support after a return.
    ReclaimsAnyCandidate,
    /// Control: the reclaimer removes the old segment as soon as the candidate is visible, before it is durable.
    RemovesBeforeCandidateDurable,
    /// Control, a combined weakening (it allows a return straight from the capture and replaces the confirmation against the selected root
    /// and the forcing by the visible cover): the departure returns without forcing anything.
    ReturnsOnVisibleCover,
    /// Control: the departure confirms against the root a power loss would leave and not the root selected now, so it returns while an
    /// unpublished unrelated candidate is already selected (review of step 2, part B).
    ConfirmsAgainstDurableRoot,
    /// Control: no non-drop rule: the reclaimer may write a candidate that drops the promised support, and the old segment goes after it.
    NoNonDrop,
    /// The departure has a single attempt: the bounded error exit is reachable (a positive control for it).
    OneAttempt,
}

/// What the reclaimer's candidate is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Candidate {
    /// Not created yet.
    None,
    /// A checkpoint whose lineage names the promised record; it keeps the support.
    Descends,
    /// A checkpoint at a higher generation that changed the document: no lineage for the record. It keeps the support.
    Retaining,
    /// A checkpoint at a higher generation that dropped the support (refused by the non-drop check; written only by the control).
    Dropping,
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
    /// Has captured nothing (or is about to capture again).
    Idle,
    /// Has captured the acknowledgment of this root.
    Captured(Root),
    /// Has forced the file of that root and the support.
    Forced(Root),
    /// Returned success to the person.
    Returned,
    /// Ended with an error after its attempts: the bounded exit, no success.
    Failed,
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
    /// Captures left.
    pub attempts_left: u8,
    /// Power losses so far.
    pub losses: u8,
}

/// The start state: the promised record and its support are in the old segment and visible only.
pub fn initial(rules: Rules) -> State {
    let attempts_left: u8 = if rules == Rules::OneAttempt { 1 } else { ATTEMPTS };
    return State { rules, world: make_visible(make_visible(World { visible: 0, durable: 0 }, PROMISE), SUPPORT), candidate: Candidate::None, departure: Departure::Idle, attempts_left, losses: 0 };
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

/// The root a reader selects now (what `state_covers` reads).
pub fn visible_root(state: &State) -> Root {
    return root_of(state.world.visible);
}

/// The root a power loss would leave (what survives).
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

/// Whether the root a power loss would leave keeps the promised support (the old segment's closure, or a candidate that retains it).
pub fn durable_support_kept(state: &State) -> bool {
    return match durable_root(state) {
        Root::Nothing => false,
        Root::Old => state.world.durable & SUPPORT != 0,
        Root::Newer => state.candidate != Candidate::Dropping && state.world.durable & SUPPORT != 0,
    };
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    for (fact, name) in [(PROMISE, "the promised record"), (CANDIDATE, "the candidate's content"), (CANDIDATE_NAME, "the candidate's directory entry"), (REMOVED, "the removal of the old segment"), (SUPPORT, "the files the record retains")] {
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
        if !matches!(after.departure, Departure::Returned | Departure::Failed) {
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
            if state.attempts_left == 0 {
                let mut after: State = state.clone();
                after.departure = Departure::Failed;
                push_step(found, "departure: its attempts are spent and it ends with an error", after);
                return;
            }
            let root: Root = visible_root(state);
            if root != Root::Nothing {
                let mut after: State = state.clone();
                after.departure = Departure::Captured(root);
                after.attempts_left -= 1;
                push_step(found, "departure: captures the acknowledgment of the record the log selects now", after);
            }
        }
        Departure::Captured(root) => {
            let unlinked: bool = root == Root::Old && state.world.visible & REMOVED != 0;
            if unlinked {
                let mut after: State = state.clone();
                after.departure = Departure::Idle;
                push_step(found, "departure: opening the old segment finds it gone and the log is selected again", after);
            }
            // The open succeeded before a concurrent unlink (or no unlink happened): the descriptor is forced either way.
            let mut forced: State = state.clone();
            forced.world = if root == Root::Old { sync(sync(forced.world, PROMISE), SUPPORT) } else { sync(sync(sync(forced.world, CANDIDATE), CANDIDATE_NAME), SUPPORT) };
            forced.departure = Departure::Forced(root);
            push_step(found, if unlinked { "departure: forces the descriptor it opened before the unlink" } else { "departure: forces the file it captured and the closure" }, forced);
            if state.rules == Rules::ReturnsOnVisibleCover && covers(state, visible_root(state), root) {
                let mut returned: State = state.clone();
                returned.departure = Departure::Returned;
                push_step(found, "departure: verifies the log by name and returns success", returned);
            }
        }
        Departure::Forced(root) => {
            if state.rules == Rules::ReturnsOnVisibleCover {
                return;
            }
            let confirming: Root = if state.rules == Rules::ConfirmsAgainstDurableRoot { durable_root(state) } else { visible_root(state) };
            if covers(state, confirming, root) {
                let mut after: State = state.clone();
                after.departure = Departure::Returned;
                push_step(found, "departure: verifies the log by name and returns success", after);
            } else if state.rules != Rules::NoSubstitution {
                let mut after: State = state.clone();
                after.departure = Departure::Idle;
                push_step(found, "departure: the selected root is not the one it captured, so it captures again", after);
            }
        }
        Departure::Returned | Departure::Failed | Departure::Dead => {}
    }
}

/// The reclaimer's steps.
fn reclaimer_steps(state: &State, found: &mut Vec<(String, State)>) {
    if state.candidate == Candidate::None && state.world.visible & PROMISE != 0 {
        let mut kinds: Vec<(Candidate, &str)> = vec![
            (Candidate::Descends, "owner: writes a checkpoint candidate that descends from the promised record"),
            (Candidate::Retaining, "owner: writes a checkpoint candidate that changed the document and keeps the support"),
        ];
        if state.rules == Rules::NoNonDrop {
            kinds.push((Candidate::Dropping, "owner: writes a checkpoint candidate that drops the support"));
        }
        for (kind, label) in kinds {
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
        // Completion forces the closure the candidate retains together with the candidate.
        after.world = sync(sync(after.world, CANDIDATE), SUPPORT);
        push_step(found, "owner: forces the candidate and the closure it retains", after);
    }
    if state.world.durable & CANDIDATE_NAME == 0 && state.world.visible & CANDIDATE_NAME != 0 {
        let mut after: State = state.clone();
        after.world = sync(after.world, CANDIDATE_NAME);
        push_step(found, "owner: forces the candidate's directory entry", after);
    }
    let durable_candidate: bool = state.world.durable & CANDIDATE != 0 && state.world.durable & CANDIDATE_NAME != 0;
    let allowed: bool = match state.rules {
        Rules::Checked | Rules::NoSubstitution | Rules::ReturnsOnVisibleCover | Rules::ConfirmsAgainstDurableRoot | Rules::OneAttempt => durable_candidate && state.candidate == Candidate::Descends,
        Rules::ReclaimsAnyCandidate | Rules::NoNonDrop => durable_candidate,
        Rules::RemovesBeforeCandidateDurable => state.candidate == Candidate::Descends,
    };
    if allowed && state.world.visible & REMOVED == 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, REMOVED);
        push_step(found, "owner: removes the old segment", after);
    }
}

/// The properties: after a return, a power loss leaves a root that keeps the promised support (never none, never a candidate that dropped
/// it). That the root the departure relied on covered its capture when it returned is the precondition of the return step; a later
/// unrelated checkpoint that keeps the support supersedes the record legitimately.
pub fn invariant(state: &State) -> Option<String> {
    if state.departure == Departure::Returned && !durable_support_kept(state) {
        return Some(if durable_root(state) == Root::Nothing { "a departure returned success but a power loss leaves the log with no root".to_string() } else { "a departure returned success but the root a power loss leaves does not keep the support the record retained".to_string() });
    }
    return None;
}

/// At quiescence a departure that is still alive has returned, failed with its error or is dead: none is left with a capture and nothing
/// to do. The error exit is a bounded end, not a success.
pub fn terminal(state: &State) -> Option<String> {
    if matches!(state.departure, Departure::Idle | Departure::Captured(_) | Departure::Forced(_)) {
        return Some("a departure that captured a record can never return".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 60 };
}

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, terminal, limits());
}
