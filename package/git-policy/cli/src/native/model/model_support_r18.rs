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
    /// The next fresh identity.
    pub next_id: u8,
    /// How many supersedes happened.
    pub supersedes: u8,
    /// What D landed for `x` once it published.
    pub landed: Option<u8>,
}

/// The effective value of `x` before D: the last live entry in tail order, else the tip.
pub fn effective(tip: u8, tail: &[Req]) -> u8 {
    return tail.iter().fold(tip, |value, request| { return request.entry.unwrap_or(value); });
}

/// The start state: a tip, the entries of the requests ahead of D (identities from 1 in tail order) and D's selection. D's capture skipped
/// `x` because an entry covered it; the deferral is kept only when the rule's support test holds at capture.
pub fn initial(rules: Rules, tip: u8, entries: &[Option<u8>], selection: u8) -> State {
    let tail: Vec<Req> = entries.iter().enumerate().map(|(at, entry)| { return Req { id: at as u8 + 1, entry: *entry, held: false }; }).collect();
    let mut state: State = State { rules, tip, next_id: tail.len() as u8 + 1, tail, selection, key: None, added: false, supersedes: 0, landed: None };
    state.key = last_equal(&state.tail, selection).filter(|_| { return rules == Rules::Seventeen; });
    reevaluate(&mut state);
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
        Rules::Eighteen | Rules::EighteenStaleAfterReplay => effective(state.tip, &state.tail) == state.selection,
        Rules::EighteenAnyEqualEntry => state.tail.iter().any(|request| { return request.entry == Some(state.selection); }) || state.tip == state.selection,
    };
}

/// Re-derives D's decision from the tail (revisions 18 and the controls; revision 17 keeps its key).
fn reevaluate(state: &mut State) {
    if state.rules != Rules::Seventeen {
        state.added = !supported(state);
    }
}

/// Revision 17's reaction to the loss of its key: re-key to the latest remaining equal entry, else add the selection.
fn lose_key(state: &mut State, gone: u8) {
    if state.key == Some(gone) {
        state.key = last_equal(&state.tail, state.selection);
        state.added = state.key.is_none();
    }
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    if state.landed.is_some() {
        return found;
    }
    for (at, request) in state.tail.iter().enumerate() {
        let mut removed: State = state.clone();
        removed.tail.remove(at);
        if removed.rules == Rules::Seventeen {
            lose_key(&mut removed, request.id);
        }
        reevaluate(&mut removed);
        push_step(&mut found, &format!("lane: removes request {} from the tail (it ended without publishing)", request.id), removed);
        for value in std::iter::once(None).chain(VALUES.iter().map(|value| { return Some(*value); })) {
            if value != request.entry {
                let mut replayed: State = state.clone();
                replayed.tail[at].entry = value;
                if replayed.rules == Rules::Seventeen && replayed.key == Some(request.id) && value != Some(replayed.selection) {
                    lose_key(&mut replayed, request.id);
                }
                if replayed.rules != Rules::EighteenStaleAfterReplay {
                    reevaluate(&mut replayed);
                }
                push_step(&mut found, &format!("lane: a replay changes request {}'s entry to {:?}", request.id, value), replayed);
            }
        }
        if state.supersedes < MAX_SUPERSEDES {
            for value in [None, request.entry] {
                let mut superseded: State = state.clone();
                superseded.tail[at] = Req { id: state.next_id, entry: value, held: false };
                superseded.next_id = state.next_id + 1;
                superseded.supersedes = state.supersedes + 1;
                if superseded.rules == Rules::Seventeen {
                    lose_key(&mut superseded, request.id);
                }
                if superseded.rules != Rules::EighteenStaleAfterReplay {
                    reevaluate(&mut superseded);
                }
                push_step(&mut found, &format!("person: request {} is superseded by a new version with entry {:?}", request.id, value), superseded);
            }
        }
    }
    if let Some(head) = state.tail.first() {
        let mut toggled: State = state.clone();
        toggled.tail[0].held = !head.held;
        push_step(&mut found, if head.held { "environment: the hook holding the head returns" } else { "environment: a hook holds the head" }, toggled);
        if !head.held {
            let mut published: State = state.clone();
            published.tail.remove(0);
            if let Some(value) = head.entry {
                published.tip = value;
            }
            // The fold of tip and tail is unchanged by the head publishing, so no rule derives anything here.
            push_step(&mut found, &format!("lane: request {} publishes", head.id), published);
        }
    } else {
        // D publishes: its delta holds `x` when the decision added it, else the tree it lands keeps what the tip holds.
        let mut landed: State = state.clone();
        landed.landed = Some(if state.added { state.selection } else { state.tip });
        push_step(&mut found, "lane: D publishes", landed);
    }
    // A checkpoint then recovery: derived state is dropped and rebuilt from the immutable bytes and the recorded removals. Revision 17's key
    // is rebuilt by its own rule; revision 18's decision is recomputed from the tail.
    let mut recovered: State = state.clone();
    if recovered.rules == Rules::Seventeen {
        recovered.key = recovered.key.filter(|key| { return recovered.tail.iter().any(|request| { return request.id == *key; }); });
    } else {
        recovered.added = false;
        reevaluate(&mut recovered);
    }
    if recovered != *state {
        push_step(&mut found, "lane: a checkpoint is followed by recovery, which rebuilds D's decision", recovered);
    }
    return found;
}

/// What the sequential reference interpreter says D lands for `x`. The literal tail and tip are the reference world: every request ahead
/// of D has published or ended, so the tip is what D lands on; the interpreter diffs D's staged version against that tip and applies the
/// difference, which leaves `selection` in the tree whether or not the tip already held it. It reads nothing the rules derive.
pub fn reference_landing(state: &State) -> u8 {
    let difference: Option<u8> = if state.tip == state.selection { None } else { Some(state.selection) };
    return difference.unwrap_or(state.tip);
}

/// D must land the person's staged version.
pub fn invariant(state: &State) -> Option<String> {
    if let Some(landed) = state.landed
        && landed != reference_landing(state)
    {
        return Some(format!("D landed x={} and the person had staged x={}", landed, state.selection));
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 400_000, max_depth: 60 };
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
