//! Historical control, not approval evidence for revision 13: a vocabulary and one demonstration; its durable evidence is a specification predicate, not a protocol that obtains the evidence.
//!
//! What:
//!  A vocabulary for observations: what a reader can see of another actor's effects, which of those effects would survive a
//!  power loss, and a claim that rests on a named piece of evidence. Plus one model that uses it, a lane that releases a
//!  request's retention on the strength of observing its terminal outcome.
//! Why:
//!  The reviews keep finding schedules where a model treated a completed logical operation as an observation ("root exists",
//!  "rename happened", "hook result is durable", "terminal outcome is discoverable") when a concurrent reader sees only a prefix
//!  of the operation's effects, and visibility is not durability. A model built from `World` has every effect as its own
//!  step, so a reader can fall between any two, and a claim names whether it rests on what is visible or on what is durable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type World = { visible: number /* bit set of facts */; durable: number };
//! // const holds = (w: World, needs: number, via: 'visible' | 'durable') => (w[via] & needs) === needs;
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// The fact that the request's terminal outcome (its ledger line) is written.
pub const TERMINAL_OUTCOME: u32 = 1;
/// The fact that the hold root of the request exists.
pub const ROOT_EXISTS: u32 = 2;
/// The fact that a hook's result file is written.
pub const HOOK_RESULT: u32 = 4;
/// The fact that a rename over a name happened.
pub const RENAME_DONE: u32 = 8;

/// What:
///  `struct World { ... }`: two bit sets of facts, the ones a reader can see now and the ones a power loss would keep.
///  `u32` is an unsigned 32-bit integer used as a set of single-bit constants (TS: a number used as flags).
/// Why:
///  A fact is made visible by one step and durable by another, so a model can place a crash or a reader between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type World = { visible: number; durable: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct World {
    /// The facts a concurrent reader sees.
    pub visible: u32,
    /// The facts that survive a power loss.
    pub durable: u32,
}

/// The world with no fact.
pub const NOTHING: World = World { visible: 0, durable: 0 };

/// The world after `fact` becomes visible (a link, a rename or an append has happened), not yet durable.
pub fn make_visible(world: World, fact: u32) -> World {
    return World { visible: world.visible | fact, durable: world.durable };
}

/// The world after a sync of `fact`: it becomes durable if it is visible, and a sync of a fact that is not there does nothing.
pub fn sync(world: World, fact: u32) -> World {
    return World { visible: world.visible, durable: world.durable | (fact & world.visible) };
}

/// The world after a power loss: only what was durable remains visible.
pub fn power_loss(world: World) -> World {
    return World { visible: world.durable, durable: world.durable };
}

/// What:
///  `enum Evidence { ... }`: what a claim rests on.
/// Why:
///  "Observed" is not one thing: a reader that saw a fact visible may be wrong after a power loss; a claim states which it uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Evidence = 'visible' | 'durable';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Evidence {
    /// The fact is visible to this reader now.
    Visible,
    /// The fact is durable (the reader synced it or knows its writer did).
    Durable,
}

/// Whether the facts in `needs` are all present in `world` under `evidence`.
pub fn holds(world: World, needs: u32, evidence: Evidence) -> bool {
    let present: u32 = if evidence == Evidence::Visible { world.visible } else { world.durable };
    return present & needs == needs;
}

/// What:
///  `struct State { ... }`: a writer retiring a request (append the terminal line, then sync it), a lane that releases the
///  retention once it has observed the terminal outcome, and a power loss.
/// Why:
///  The lane's observation is a claim with evidence: visible (revision 8 and 9 style "the line is in the ledger") or durable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { evidence: Evidence; world: World; released: boolean; crashed: boolean };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The evidence the lane's claim rests on.
    pub evidence: Evidence,
    /// What is visible and what is durable.
    pub world: World,
    /// The lane released the request's retention.
    pub released: bool,
    /// A power loss happened.
    pub crashed: bool,
}

/// The start state.
pub fn initial(evidence: Evidence) -> State {
    return State { evidence, world: NOTHING, released: false, crashed: false };
}

/// What:
///  Every step enabled in `state`: the writer's two effects, the lane's observation-based release, and a power loss.
/// Why:
///  The lane can fall between the writer's append and its sync, and the power can fail on either side of the release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    if state.world.visible & TERMINAL_OUTCOME == 0 && !state.crashed {
        let mut after: State = state.clone();
        after.world = make_visible(state.world, TERMINAL_OUTCOME);
        push_step(&mut found, "writer: appends the terminal line to the ledger", after);
    }
    if state.world.visible & TERMINAL_OUTCOME != 0 && state.world.durable & TERMINAL_OUTCOME == 0 {
        let mut after: State = state.clone();
        after.world = sync(state.world, TERMINAL_OUTCOME);
        push_step(&mut found, "writer: fsyncs the ledger and its directory", after);
    }
    if !state.released && holds(state.world, TERMINAL_OUTCOME, state.evidence) {
        let mut after: State = state.clone();
        after.released = true;
        let label: &str = if state.evidence == Evidence::Visible {
            "lane: observes the terminal line visible and releases the retention"
        } else {
            "lane: observes the terminal line durable and releases the retention"
        };
        push_step(&mut found, label, after);
    }
    if !state.crashed {
        let mut after: State = state.clone();
        after.world = power_loss(state.world);
        after.crashed = true;
        push_step(&mut found, "environment: power is lost", after);
    }
    return found;
}

/// The safety check: a request whose retention was released has its terminal outcome discoverable after a power loss.
pub fn invariant(state: &State) -> Option<String> {
    if state.released && state.crashed && state.world.visible & TERMINAL_OUTCOME == 0 {
        return Some("the retention was released and the terminal outcome is not discoverable after the power loss".to_string());
    }
    return None;
}

/// A state with no enabled step needs no extra check here.
pub fn end_state(_state: &State) -> Option<String> {
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 1_000, max_depth: 16 };
}

/// Explores every schedule under `evidence`.
pub fn check(evidence: Evidence) -> Report {
    return explore(vec![initial(evidence)], steps, invariant, end_state, limits());
}
