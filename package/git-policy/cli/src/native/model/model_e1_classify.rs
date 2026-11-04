//! Historical control, not approval evidence for revision 13: assumes authentic refusal evidence; E1 kinds P, F and U are modelled in model_w1_r13.
//!
//! What:
//!  Model of the ninth review's R9-8 for E1: a person's `reference-transaction` hook refuses an update, a foreign writer advances
//!  the branch before E1 classifies the exit, and E1 runs.
//! Why:
//!  Revision 9 classified a refusal only when the tip still equals `old`; a foreign advance made E1 read the exit as an outside
//!  kill and run crash recovery, which runs the hook again. Revision 10 classifies from the invocation's exit status and its last
//!  line before reading any state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; hookRefused; tipMoved; classified: 'none'|'refused'|'crash'; hookRuns: number };
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// What:
///  `enum Rules { ... }`: the rule under test (TS: a union of string literals).
/// Why:
///  The rule lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'nine' | 'ten';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 9: the hook-abort text counts only when the tip equals `old` and the lock file is absent.
    Nine,
    /// Revision 10: the class comes from the exit status and the last line, before any state is read.
    Ten,
}

/// What:
///  `struct State { ... }`: the facts of the schedule.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; exited: boolean; tipMoved: boolean; classified: number; hookRuns: number };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The update ended with the hook's abort text and a nonzero status.
    pub exited: bool,
    /// A foreign writer advanced the branch after the exit.
    pub tip_moved: bool,
    /// E1's class: 0 not yet, 1 refused (nothing changed, no retry), 2 treated as a crash (recovery runs).
    pub classified: u8,
    /// How many times the person's hook ran.
    pub hook_runs: u8,
}

/// The start state: the hook has run once.
pub fn initial(rules: Rules) -> State {
    return State { rules, exited: false, tip_moved: false, classified: 0, hook_runs: 1 };
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    if !state.exited {
        let mut after: State = state.clone();
        after.exited = true;
        push_step(&mut found, "P2: update-ref exits nonzero with the reference-transaction hook's abort text", after);
    }
    if state.exited && !state.tip_moved {
        let mut after: State = state.clone();
        after.tip_moved = true;
        push_step(&mut found, "foreign: advances the branch before E1 runs", after);
    }
    if state.exited && state.classified == 0 {
        let mut after: State = state.clone();
        let refused: bool = state.rules == Rules::Ten || !state.tip_moved;
        if refused {
            after.classified = 1;
            push_step(&mut found, "E1: classifies the exit as the hook refusing", after);
        } else {
            // Crash recovery repeats the update, and the hook runs again.
            after.classified = 2;
            after.hook_runs += 1;
            push_step(&mut found, "E1 (d): the evidence matches nothing, so crash recovery repeats the update", after);
        }
    }
    return found;
}

/// The safety check: a refusal is never repeated.
pub fn invariant(state: &State) -> Option<String> {
    if state.hook_runs > 1 {
        return Some("a person's hook that refused ran again".to_string());
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

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, end_state, limits());
}
