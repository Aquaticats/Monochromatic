//! What:
//!  Model of the disposal of saved content: the person authorizes it; the owner appends a `disposed` line to the ledger and forces it,
//!  checkpoints a root that no longer names the item (its closure carries the forced ledger prefix) and makes that root durable, and only
//!  then removes the content. The facts are the ledger line, the removal of the content and the root without the item, each visible and
//!  durable apart (`model_observe::World`); power loss, write-back and the owner's steps are steps.
//! Why:
//!  The row "saved content survives until authorized disposal". The first version accepted a durable `disposed` line as the discharge of a
//!  missing item; the component's recovery does not: it validates every retained saved-content entry of a root, and a root that still
//!  names an item whose file is gone is ineligible whatever the ledger says, so recovery would report the log damaged (review of step 1,
//!  finding 6). The procedure that fits the validation architecture is the one modelled: a durable eligible replacement root, with its
//!  discharge proof, before the unlink.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // remove(saved) only if authorized && durable(disposedLine) && durable(rootWithoutItem)
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};
use crate::model_observe::{World, make_visible, power_loss, sync};

/// The `disposed` line for the item is in the ledger.
pub const LINE: u32 = 1;
/// The saved content's name is removed.
pub const GONE: u32 = 2;
/// The root document that no longer names the item.
pub const DROPPED: u32 = 4;

/// How many power losses the search may inject.
pub const MAX_LOSSES: u8 = 1;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// The procedure the validation architecture needs.
    Ordered,
    /// Control: the content is removed once the line is durable, before the root without the item is (the first version's rule).
    RemovesBeforeRootDurable,
    /// Control: the root without the item is checkpointed and made durable although the line is not, and the content is removed on that root alone.
    RemovesOnRootWithoutLineProof,
    /// Control: retirement of the request removes its saved content, with no authorization from the person.
    RetirementRemoves,
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

/// Whether recovery after a power loss has an eligible root: the old root is eligible only while the item's file exists, and the root
/// without the item only when it is durable together with the ledger prefix its closure holds.
pub fn recovery_has_a_root(state: &State) -> bool {
    let old_root_eligible: bool = state.world.durable & GONE == 0;
    let new_root_eligible: bool = state.world.durable & DROPPED != 0 && state.world.durable & LINE != 0;
    return old_root_eligible || new_root_eligible;
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
    let line_ready: bool = if state.rules == Rules::RemovesOnRootWithoutLineProof { state.world.visible & LINE != 0 } else { state.world.durable & LINE != 0 };
    if line_ready && state.world.visible & DROPPED == 0 && state.world.visible & LINE != 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, DROPPED);
        push_step(&mut found, "owner: checkpoints a root that no longer names the item", after);
    }
    if state.world.visible & DROPPED != 0 && state.world.durable & DROPPED == 0 {
        let mut after: State = state.clone();
        after.world = sync(after.world, DROPPED);
        push_step(&mut found, "owner: forces the root without the item", after);
    }
    let removal_ready: bool = match state.rules {
        Rules::Ordered | Rules::RetirementRemoves => state.world.durable & LINE != 0 && state.world.durable & DROPPED != 0,
        Rules::RemovesBeforeRootDurable => state.world.durable & LINE != 0,
        Rules::RemovesOnRootWithoutLineProof => state.world.durable & DROPPED != 0,
    };
    if removal_ready && state.world.visible & GONE == 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, GONE);
        push_step(&mut found, "owner: removes the saved content", after);
    }
    if state.rules == Rules::RetirementRemoves && state.world.visible & GONE == 0 {
        let mut after: State = state.clone();
        after.world = make_visible(after.world, GONE);
        push_step(&mut found, "owner: retires the request and removes its saved content", after);
    }
    return found;
}

/// The properties: no removal without the person's authorization, and no power loss that leaves recovery with no eligible root.
pub fn invariant(state: &State) -> Option<String> {
    if state.world.visible & GONE != 0 && !state.authorized {
        return Some("saved content was removed although the person did not authorize its disposal".to_string());
    }
    if state.world.durable & GONE != 0 && !recovery_has_a_root(state) {
        return Some("the content is durably gone and neither the old root nor a durable replacement root with its ledger proof is eligible".to_string());
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
