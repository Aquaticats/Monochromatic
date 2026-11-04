//! Historical control, not approval evidence for revision 13: revision 11 register whose actions complete by assignment; superseded by model_register_r13.
//!
//! What:
//!  Model of revision 11's register of waits: each entry (START, FOREGROUND, MARKER, ACQUIRE, REAP, KILL, COMMITTING, STALL, PUSH) is
//!  a process blocked on another party's progress, with a bound and an action at the bound.
//! Why:
//!  "Nothing should ever wait forever or indefinitely": for every entry, the wait ends by the protocol's own step (the bound reached,
//!  then the action) within a stated number of steps, whether or not the other party ever makes progress. Revision 10 had no bound for
//!  these waits, so a party that never progresses leaves the process with nothing of its own that can end the wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const entry of REGISTER) check(entry, rules);
//! ```

use crate::model_explore::{Limits, Report, explore_bounded, push_step};

/// The nine entries of the register and the action each takes at its bound.
pub const ENTRIES: [(&str, &str); 9] = [
    ("START", "W1, or the plain error when no request exists yet"),
    ("FOREGROUND", "W1: withdraw and exit non-zero naming the blocker"),
    ("MARKER", "exit non-zero: the lane was busy recovering, run again"),
    ("ACQUIRE", "stop being an owner candidate and wait as in FOREGROUND"),
    ("REAP", "release the lane by exiting; the next trigger tries again"),
    ("KILL", "release the gate and exit non-zero naming an owner that did not die"),
    ("COMMITTING", "exit 0 saying the commit was decided and will complete"),
    ("STALL", "cancel the stalled child; the owner never blocks its event loop"),
    ("PUSH", "bound the push and keep retrying"),
];

/// What:
///  `enum Rules { ... }`: the rule under test.
/// Why:
///  The rule lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'ten' | 'eleven';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 10: the waits have no bound.
    Ten,
    /// Revision 11: each wait has a bound and an action at it.
    Eleven,
}

/// What:
///  `struct State { ... }`: which entry, whether the process is still waiting, and whether the other party ever progressed.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; entry: number; waiting: boolean; progressed: boolean };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// Which entry of `ENTRIES`.
    pub entry: usize,
    /// The process is blocked.
    pub waiting: bool,
    /// The other party made progress (the environment's choice).
    pub progressed: bool,
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    if state.waiting && !state.progressed {
        let mut after: State = state.clone();
        after.progressed = true;
        after.waiting = false;
        push_step(&mut found, "environment: the other party makes progress and the wait ends", after);
    }
    if state.waiting && state.rules == Rules::Eleven {
        let mut after: State = state.clone();
        after.waiting = false;
        push_step(&mut found, ENTRIES[state.entry].1, after);
    }
    return found;
}

/// No safety property beyond the bound.
pub fn invariant(_state: &State) -> Option<String> {
    return None;
}

/// The wait the register bounds.
pub fn waiting(state: &State) -> bool {
    return state.waiting;
}

/// The bound, in own steps, that every entry must meet (the bound is itself one step: the clock reaching it).
pub const BOUND: usize = 2;

/// Explores entry `entry` under `rules`.
pub fn check(rules: Rules, entry: usize) -> Report {
    let start: State = State { rules, entry, waiting: true, progressed: false };
    return explore_bounded(vec![start], steps, invariant, waiting, BOUND, Limits { max_states: 100, max_depth: 10 });
}
