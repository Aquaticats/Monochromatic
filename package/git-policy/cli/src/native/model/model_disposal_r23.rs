//! What:
//!  Model of the disposal of saved content: the person authorizes the disposal of a saved item; the owner appends a `disposed` line to the
//!  ledger, forces it, and only then removes the saved content; a reclaiming checkpoint may drop the item from the root document. The facts
//!  are the ledger line, the removal of the content and the root document's claim on the item, each visible and durable apart
//!  (`model_observe::World`); power loss, write-back and the owner's steps are steps.
//! Why:
//!  The row "saved content survives until authorized disposal" (design: "a `disposed` ledger line is made D before the saved content it
//!  discharges is deleted, so a power loss cannot leave a durable root that claims a saved item whose only copy was released"). Two clauses:
//!  nothing deletes saved content that the person did not authorize (retirement never does), and the removal never becomes durable ahead of
//!  the line that discharges it. Each has a single-defect control.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // remove(saved) only if authorized && durable(disposedLine); root may drop saved only if durable(disposedLine)
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};
use crate::model_observe::{World, make_visible, power_loss, sync};

/// The `disposed` line for the item is in the ledger.
pub const LINE: u32 = 1;
/// The saved content's name is removed.
pub const GONE: u32 = 2;
/// The root document no longer names the item.
pub const DROPPED: u32 = 4;

/// How many power losses the search may inject.
pub const MAX_LOSSES: u8 = 1;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// The design's rule.
    Ordered,
    /// Control: the content is removed as soon as the line is appended, before the line is durable.
    RemovesBeforeLineDurable,
    /// Control: retirement of the request removes its saved content, with no authorization from the person.
    RetirementRemoves,
    /// Control: the root drops the item before the line is durable.
    RootDropsBeforeLineDurable,
}

/// The model's state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The facts of the disposal.
    pub world: World,
    /// The person's explicit disposal command was given (a monitor: never reset by a power loss).
    pub authorized: bool,
    /// The owner has been told of the authorization (the command's request reached the lane before a power loss ended it).
    pub requested: bool,
    /// Power losses so far.
    pub losses: u8,
}

/// The start state: the item is saved, the root names it, nothing is authorized.
pub fn initial(rules: Rules) -> State {
    return State { rules, world: World { visible: 0, durable: 0 }, authorized: false, requested: false, losses: 0 };
}

/// Whether the root, as a recovery after a power loss would read it, names an item whose content is gone and that no durable line discharges.
pub fn durable_orphan(state: &State) -> bool {
    let root_names_item: bool = state.world.durable & DROPPED == 0;
    return root_names_item && state.world.durable & GONE != 0 && state.world.durable & LINE == 0;
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    for (fact, name) in [(LINE, "the disposed line"), (GONE, "the removal of the saved content"), (DROPPED, "the root without the item")] {
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
        after.requested = after.world.visible & LINE != 0;
        push_step(&mut found, "environment: power is lost", after);
    }
    if !state.authorized {
        let mut after: State = state.clone();
        after.authorized = true;
        after.requested = true;
        push_step(&mut found, "person: disposes of the saved content explicitly", after);
    }
    if state.requested && state.world.visible & LINE == 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, LINE);
        push_step(&mut found, "owner: appends the disposed line to the ledger", after);
    }
    if state.world.visible & LINE != 0 && state.world.durable & LINE == 0 {
        let mut after: State = state.clone();
        after.world = sync(after.world, LINE);
        push_step(&mut found, "owner: forces the disposed line", after);
    }
    let line_ready: bool = if state.rules == Rules::RemovesBeforeLineDurable { state.world.visible & LINE != 0 } else { state.world.durable & LINE != 0 };
    if line_ready && state.world.visible & GONE == 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, GONE);
        push_step(&mut found, "owner: removes the saved content", after);
    }
    if state.rules == Rules::RetirementRemoves && state.world.visible & GONE == 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, GONE);
        push_step(&mut found, "owner: retires the request and removes its saved content", after);
    }
    let root_ready: bool = if state.rules == Rules::RootDropsBeforeLineDurable { state.world.visible & LINE != 0 } else { state.world.durable & LINE != 0 };
    if root_ready && state.world.visible & DROPPED == 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, DROPPED);
        push_step(&mut found, "owner: checkpoints a root that no longer names the item", after);
    }
    return found;
}

/// The properties: no removal without the person's authorization, and no durable root that names an item whose content is gone with no
/// durable line to discharge it.
pub fn invariant(state: &State) -> Option<String> {
    if state.world.visible & GONE != 0 && !state.authorized {
        return Some("saved content was removed although the person did not authorize its disposal".to_string());
    }
    if durable_orphan(state) {
        return Some("a durable root names a saved item whose content is gone and no durable disposed line discharges it".to_string());
    }
    if state.world.durable & DROPPED != 0 && state.world.durable & LINE == 0 {
        return Some("a durable root dropped a saved item with no durable disposed line to discharge it".to_string());
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
