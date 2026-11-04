//! What:
//!  Model of revision 18's effective ordered base (the support rule of C1, B1 to B3 and Q1). One path `x`; a tail of pending requests,
//!  each with a live `install` entry on `x` (or none); a last request D whose capture skipped `x` because an entry covered it and so
//!  carries a deferred selection `s` (the person's staged version of `x`). The tail changes by removals, by a predecessor's replay
//!  changing its resulting entry, by a request being superseded by a newer version, by a hook holding the head, by the head
//!  publishing, and by a checkpoint followed by recovery. D's decision (keep `x` deferred, or add it to its delta) is derived state;
//!  when D finally publishes, the tree it lands must hold `s`.
//! Why:
//!  Revision 17 judged support on a predecessor: remove A (`x=1`) from tail P `x=1`, Q `x=2`, A `x=1`, D `x=1` and D re-keys to P,
//!  which publishes `x=1` before Q overrides it, so D lands `x=2` and the person's `x=1` is lost. Revision 18 judges support on the
//!  fold of the live entries before D in tail order and re-evaluates whenever the tail changes. The sequential reference interpreter
//!  recomputes D's delta against the tip D actually lands on, which is the definition of the specification; the model's incremental
//!  decision must equal it in every reachable state. Each rule is a parameter, with single-defect controls (stale after replay, any
//!  equal entry supports) so a green shows the clause it names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const effective = (tip, tail) => tail.reduce((acc, r) => r.entry ?? acc, tip);
//! // const deferred = (tip, tail, s) => effective(tip, tail) === s;
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// How many requests the search may create by superseding (each supersede mints a fresh identity).
pub const MAX_SUPERSEDES: u8 = 2;

/// How many crashes the search may inject (each loses the in-memory decision).
pub const MAX_CRASHES: u8 = 2;

/// The values of `x` the model uses: 0 is the tip's initial content, 1 and 2 are other versions.
pub const VALUES: [u8; 3] = [0, 1, 2];

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 17: D is keyed to one predecessor with an equal entry and re-keys to an earlier equal entry when the key is removed,
    /// ignoring that a later entry overrides it.
    Seventeen,
    /// Revision 18: support is the fold of the live entries before D in tail order, evaluated again at every change of the tail.
    Eighteen,
    /// Control: revision 18 that evaluates the fold only when a request is removed, not when a predecessor's replay or a supersede changes
    /// an entry (the derived decision goes stale).
    EighteenStaleAfterReplay,
    /// Control: revision 18 where any live entry equal to the selection supports it, whatever later entries override.
    EighteenAnyEqualEntry,
    /// Control: revision 18 whose D may publish while its decision is stale (a tail change or a recovery has not been rebuilt yet).
    EighteenPublishesWhileStale,
    /// Control: revision 18 whose recovery trusts the decision persisted by the last checkpoint instead of rebuilding it.
    EighteenTrustsCheckpoint,
}

/// One pending request ahead of D.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Req {
    /// The request's identity (a superseding version has a new one).
    pub id: u8,
    /// The value of `x` in its live `install` entry, if it has an entry on `x`.
    pub entry: Option<u8>,
    /// A hook holds the request: it cannot publish until released.
    pub held: bool,
}

/// The model's state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The content of `x` on the branch tip.
    pub tip: u8,
    /// The pending requests ahead of D, in tail order.
    pub tail: Vec<Req>,
    /// D's deferred selection: the person's staged version of `x`.
    pub selection: u8,
    /// Revision 17: the predecessor D is keyed to.
    pub key: Option<u8>,
    /// Derived: `x` was added to D's delta, so D lands `selection` whatever the tip holds.
    pub added: bool,
    /// Derived state is stale: the tail changed or a recovery happened since the decision was last rebuilt.
    pub stale: bool,
    /// The decision the last checkpoint persisted.
    pub persisted: Option<bool>,
    /// The next fresh identity.
    pub next_id: u8,
    /// How many supersedes happened.
    pub supersedes: u8,
    /// How many crashes happened (each loses the in-memory decision).
    pub crashes: u8,
    /// What D landed for `x` once it published.
    pub landed: Option<u8>,
    /// The sequential reference: a second, separately written execution of the same operations over plain lists.
    pub reference: Reference,
}

/// The sequential reference interpreter's world: a tip and a tail of (identity, entry) pairs, advanced by `apply_reference` only.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Reference {
    /// The branch tip's content of `x`.
    pub tip: u8,
    /// The pending requests, in tail order.
    pub tail: Vec<(u8, Option<u8>)>,
    /// What D landed, computed at its publication.
    pub landed: Option<u8>,
}

/// One operation of the lane, applied to the model and, separately, to the reference interpreter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op {
    /// A request ended without publishing and leaves the tail.
    Remove(u8),
    /// A predecessor's replay changes its resulting entry.
    Replay(u8, Option<u8>),
    /// The person supersedes a request by a new version with the given identity and entry.
    Supersede(u8, u8, Option<u8>),
    /// A hook holds or releases the head.
    ToggleHold,
    /// The head publishes.
    PublishHead,
    /// D publishes.
    PublishD,
}

/// The effective value of `x` before D: the last live entry in tail order, else the tip.
pub fn effective(tip: u8, tail: &[Req]) -> u8 {
    return tail.iter().fold(tip, |value, request| { return request.entry.unwrap_or(value); });
}

/// The start state: a tip, the entries of the requests ahead of D (identities from 1 in tail order) and D's selection. D's capture skipped
/// `x` because an entry covered it; the deferral is kept only when the rule's support test holds at capture.
pub fn initial(rules: Rules, tip: u8, entries: &[Option<u8>], selection: u8) -> State {
    let tail: Vec<Req> = entries.iter().enumerate().map(|(at, entry)| { return Req { id: at as u8 + 1, entry: *entry, held: false }; }).collect();
    let reference: Reference = Reference { tip, tail: tail.iter().map(|request| { return (request.id, request.entry); }).collect(), landed: None };
    let mut state: State = State {
        rules,
        tip,
        next_id: tail.len() as u8 + 1,
        tail,
        selection,
        key: None,
        added: false,
        stale: false,
        persisted: None,
        supersedes: 0,
        crashes: 0,
        landed: None,
        reference,
    };
    state.key = last_equal(&state.tail, selection).filter(|_| { return rules == Rules::Seventeen; });
    state.added = !supported(&state);
    return state;
}

/// The identity of the last live entry equal to `selection`.
fn last_equal(tail: &[Req], selection: u8) -> Option<u8> {
    return tail.iter().rev().find(|request| { return request.entry == Some(selection); }).map(|request| { return request.id; });
}

/// Whether the rule counts the selection as supported by the current tail.
fn supported(state: &State) -> bool {
    return match state.rules {
        Rules::Seventeen => state.key.is_some() && !state.added,
        Rules::Eighteen | Rules::EighteenStaleAfterReplay | Rules::EighteenPublishesWhileStale | Rules::EighteenTrustsCheckpoint => {
            effective(state.tip, &state.tail) == state.selection
        }
        Rules::EighteenAnyEqualEntry => state.tail.iter().any(|request| { return request.entry == Some(state.selection); }) || state.tip == state.selection,
    };
}

/// Revision 17's reaction to the loss of its key: re-key to the latest remaining equal entry, else add the selection.
fn lose_key(state: &mut State, gone: u8) {
    if state.key == Some(gone) {
        state.key = last_equal(&state.tail, state.selection);
        state.added = state.key.is_none();
    }
}

/// Whether a tail change of this kind makes the derived decision stale under `rules`.
fn marks_stale(rules: Rules, op: Op) -> bool {
    return match rules {
        Rules::Seventeen => false,
        Rules::EighteenStaleAfterReplay => matches!(op, Op::Remove(_)),
        _ => matches!(op, Op::Remove(_) | Op::Replay(..) | Op::Supersede(..)),
    };
}

/// Applies `op` to the model state, or `None` when it is not enabled.
pub fn apply_model(state: &State, op: Op) -> Option<State> {
    let mut after: State = state.clone();
    match op {
        Op::Remove(id) => {
            let at: usize = after.tail.iter().position(|request| { return request.id == id; })?;
            after.tail.remove(at);
            if after.rules == Rules::Seventeen {
                lose_key(&mut after, id);
            }
        }
        Op::Replay(id, value) => {
            let at: usize = after.tail.iter().position(|request| { return request.id == id; })?;
            if after.tail[at].entry == value {
                return None;
            }
            after.tail[at].entry = value;
            if after.rules == Rules::Seventeen && after.key == Some(id) && value != Some(after.selection) {
                lose_key(&mut after, id);
            }
        }
        Op::Supersede(id, fresh, value) => {
            let at: usize = after.tail.iter().position(|request| { return request.id == id; })?;
            if after.supersedes >= MAX_SUPERSEDES || fresh != after.next_id {
                return None;
            }
            after.tail[at] = Req { id: fresh, entry: value, held: false };
            after.next_id += 1;
            after.supersedes += 1;
            if after.rules == Rules::Seventeen {
                lose_key(&mut after, id);
            }
        }
        Op::ToggleHold => {
            let head = after.tail.first_mut()?;
            head.held = !head.held;
        }
        Op::PublishHead => {
            let head: Req = *after.tail.first()?;
            if head.held {
                return None;
            }
            after.tail.remove(0);
            if let Some(value) = head.entry {
                after.tip = value;
            }
            // The fold of tip and tail is unchanged by the head publishing, so no rule derives anything here.
        }
        Op::PublishD => {
            if !after.tail.is_empty() {
                return None;
            }
            let may_publish: bool = !after.stale || after.rules == Rules::EighteenPublishesWhileStale;
            if !may_publish {
                return None;
            }
            // D's delta holds `x` when the decision added it, else the tree it lands keeps what the tip holds.
            after.landed = Some(if after.added { after.selection } else { after.tip });
        }
    }
    if marks_stale(after.rules, op) {
        after.stale = true;
    }
    return Some(after);
}

/// Applies `op` to the reference interpreter: plain list operations, and D's landing computed by diffing its staged version against the
/// tip it lands on. It reads nothing the rules derive (no key, no decision, no staleness).
pub fn apply_reference(world: &Reference, selection: u8, op: Op) -> Option<Reference> {
    let mut after: Reference = world.clone();
    match op {
        Op::Remove(id) => {
            after.tail = world.tail.iter().filter(|(other, _)| { return *other != id; }).copied().collect();
            if after.tail.len() == world.tail.len() {
                return None;
            }
        }
        Op::Replay(id, value) => {
            let slot = after.tail.iter_mut().find(|(other, _)| { return *other == id; })?;
            slot.1 = value;
        }
        Op::Supersede(id, fresh, value) => {
            let slot = after.tail.iter_mut().find(|(other, _)| { return *other == id; })?;
            *slot = (fresh, value);
        }
        Op::ToggleHold => {
            world.tail.first()?;
        }
        Op::PublishHead => {
            let (_, value) = *world.tail.first()?;
            after.tail.remove(0);
            if let Some(content) = value {
                after.tip = content;
            }
        }
        Op::PublishD => {
            if !world.tail.is_empty() {
                return None;
            }
            let difference: Option<u8> = if world.tip == selection { None } else { Some(selection) };
            after.landed = Some(difference.unwrap_or(world.tip));
        }
    }
    return Some(after);
}

/// Every step enabled in `state`, with its operation when it is one the reference also executes.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    if state.landed.is_some() {
        return found;
    }
    let mut ops: Vec<(Op, String)> = Vec::new();
    for request in &state.tail {
        ops.push((Op::Remove(request.id), format!("lane: removes request {} from the tail (it ended without publishing)", request.id)));
        for value in std::iter::once(None).chain(VALUES.iter().map(|value| { return Some(*value); })) {
            if value != request.entry {
                ops.push((Op::Replay(request.id, value), format!("lane: a replay changes request {}'s entry to {:?}", request.id, value)));
            }
        }
        if state.supersedes < MAX_SUPERSEDES {
            for value in [None, request.entry] {
                ops.push((Op::Supersede(request.id, state.next_id, value), format!("person: request {} is superseded by a new version with entry {:?}", request.id, value)));
            }
        }
    }
    if let Some(head) = state.tail.first() {
        ops.push((Op::ToggleHold, (if head.held { "environment: the hook holding the head returns" } else { "environment: a hook holds the head" }).to_string()));
        ops.push((Op::PublishHead, format!("lane: request {} publishes", head.id)));
    } else {
        ops.push((Op::PublishD, "lane: D publishes".to_string()));
    }
    for (op, label) in ops {
        let Some(mut next) = apply_model(state, op) else { continue };
        let Some(world) = apply_reference(&state.reference, state.selection, op) else { continue };
        next.reference = world;
        next.landed = next.landed.or(None);
        push_step(&mut found, &label, next);
    }
    // The derived decision has steps of its own (revisions 18 and the controls; revision 17 keeps its key and has none).
    if state.rules != Rules::Seventeen {
        if state.stale {
            let mut rebuilt: State = state.clone();
            rebuilt.added = !supported(&rebuilt);
            rebuilt.stale = false;
            push_step(&mut found, "lane: rebuilds D's decision from the tail and the recorded removals", rebuilt);
        }
        if state.persisted != Some(state.added) || state.stale {
            let mut checkpointed: State = state.clone();
            checkpointed.persisted = Some(state.added);
            push_step(&mut found, "lane: a checkpoint persists D's current decision", checkpointed);
        }
        if state.crashes < MAX_CRASHES {
            let mut recovered: State = state.clone();
            recovered.crashes += 1;
            recovered.stale = true;
            recovered.added = false;
            if recovered.rules == Rules::EighteenTrustsCheckpoint {
                // The control trusts what the checkpoint persisted, whatever changed after it.
                recovered.added = state.persisted.unwrap_or(false);
                recovered.stale = false;
            }
            push_step(&mut found, "environment: a crash loses the in-memory decision and recovery starts", recovered);
        }
    }
    return found;
}

/// D must land the person's staged version, the model's landing must equal the reference interpreter's, and the literal worlds must agree.
pub fn invariant(state: &State) -> Option<String> {
    if state.tip != state.reference.tip || state.tail.iter().map(|request| { return (request.id, request.entry); }).collect::<Vec<_>>() != state.reference.tail {
        return Some("the model's tail and the reference interpreter's tail differ".to_string());
    }
    if let Some(landed) = state.landed {
        if state.reference.landed != Some(landed) {
            return Some(format!("D landed x={} and the reference interpreter says x={:?}", landed, state.reference.landed));
        }
        if landed != state.selection {
            return Some(format!("D landed x={} and the person had staged x={}", landed, state.selection));
        }
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 600_000, max_depth: 80 };
}

/// No quiescent state is a failure: D publishing is the end.
fn never(_state: &State) -> Option<String> {
    return None;
}

/// Explores one start state.
pub fn check(initial_state: State) -> Report {
    return explore(vec![initial_state], steps, invariant, never, limits());
}

/// The reviewer's schedule: tail P `x=1`, Q `x=2`, A `x=1`; D's selection `x=1`.
pub fn pqad(rules: Rules) -> State {
    return initial(rules, 0, &[Some(1), Some(2), Some(1)], 1);
}
