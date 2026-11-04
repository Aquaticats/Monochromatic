//! Historical control, not approval evidence for revision 13: covers R8-2 only (two lanes, one repairer).
//!
//! What:
//!  Model of the revision 8 finding R8-2 ("two lanes, one repairer"): two worktrees' lanes use one ledger file, and one lane
//!  repairs the torn tail that the other lane's append in progress shows.
//! Why:
//!  Revision 9 (`doc/planning/cli-git-commits-never-fail.md`, "Named entities" and L4) gives each lane its own ledger with one
//!  appender, repairs only the own ledger, and has a reader of another lane's ledger treat a torn tail as not yet written.
//!  The model has an append that takes two steps (the line is visible partly, then whole), a repair that truncates a partial
//!  tail, a reader, and an appender that can die.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; xLine: 'none' | 'partial' | 'whole'; truncated: boolean; ... };
//! ```

// What:     imports the explorer's types and the step helper.
// Why:      The model hands its steps and checks to the explorer.
//
// In TS you'd write (pseudocode):
// ```ts
// import { explore, Limits, Report, pushStep } from './model_explore';
// ```
use crate::model_explore::{Limits, Report, explore, push_step};

/// What:
///  `enum Rules { ... }`: the rule under test (TS: a union of string literals).
/// Why:
///  The rule lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'eight' | 'nine';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 8: one ledger shared by the lanes of an object store; every lane's L4 repairs its torn tail.
    Eight,
    /// Revision 9: one ledger per lane; a lane repairs only its own; a reader of another lane's ledger ignores a torn tail.
    Nine,
}

/// What:
///  `struct State { ... }`: lane X's append, lane Y's repair and read.
///  `x_line` is 0 for no line, 1 for a partly written line, 2 for a whole line.
/// Why:
///  Flat `Copy` fields hash cheaply; the harms are flags set when they happen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; xLine: number; xAcknowledged: boolean; xLost: boolean; yRepairedOwn: boolean; yRead: number };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// Lane X's line in the ledger it writes: 0 none, 1 partly written, 2 whole.
    pub x_line: u8,
    /// Lane X is writing (between the first and the second write of its append).
    pub x_writing: bool,
    /// Lane X told its caller the line is durable.
    pub x_acknowledged: bool,
    /// Lane Y truncated lane X's partial line.
    pub y_truncated_x: bool,
    /// What lane Y read of X's line: 0 not read, 1 treated as not written, 2 read as whole.
    pub y_read: u8,
    /// Harm: X acknowledged a line that is not in the ledger.
    pub x_lost: bool,
}

/// The start state.
pub fn initial(rules: Rules) -> State {
    return State { rules, x_line: 0, x_writing: false, x_acknowledged: false, y_truncated_x: false, y_read: 0, x_lost: false };
}

/// What:
///  Every step enabled in `state`.
/// Why:
///  The explorer takes them in every order, so Y's repair falls between X's two writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    if state.x_line == 0 && !state.x_writing {
        let mut after: State = state.clone();
        after.x_line = 1;
        after.x_writing = true;
        push_step(&mut found, "lane X: appends the first part of its ledger line", after);
    }
    if state.x_writing {
        let mut after: State = state.clone();
        after.x_writing = false;
        if state.x_line == 1 {
            after.x_line = 2;
            after.x_acknowledged = true;
        } else {
            // The line was truncated under the append: the second write lands after a hole and the line is not valid.
            after.x_acknowledged = true;
            after.x_lost = true;
        }
        push_step(&mut found, "lane X: appends the rest, fsyncs and acknowledges", after);
    }
    if state.x_writing {
        let mut after: State = state.clone();
        after.x_writing = false;
        push_step(&mut found, "environment: lane X dies in the middle of its append", after);
    }
    if state.rules == Rules::Eight && state.x_line == 1 && !state.y_truncated_x {
        let mut after: State = state.clone();
        after.x_line = 0;
        after.y_truncated_x = true;
        push_step(&mut found, "L4: lane Y repairs the torn tail of the shared ledger", after);
    }
    if state.x_line == 1 && !state.x_writing && state.rules == Rules::Nine {
        // Lane X's own L4 repairs its own ledger after its own death.
        let mut after: State = state.clone();
        after.x_line = 0;
        push_step(&mut found, "L4: lane X repairs the torn tail of its own ledger", after);
    }
    if state.y_read == 0 {
        let mut after: State = state.clone();
        after.y_read = if state.x_line == 2 { 2 } else { 1 };
        push_step(&mut found, "lane Y: reads lane X's ledger and treats a torn tail as not yet written", after);
    }
    return found;
}

/// The safety check: a line a lane acknowledged is in its ledger, and a lane never truncates another lane's ledger.
pub fn invariant(state: &State) -> Option<String> {
    if state.y_truncated_x {
        return Some("a lane truncated the tail of another lane's append in progress".to_string());
    }
    if state.x_lost {
        return Some("a lane acknowledged a ledger line that is not in the ledger".to_string());
    }
    return None;
}

/// A state with no enabled step needs no extra check here.
pub fn end_state(_state: &State) -> Option<String> {
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 10_000, max_depth: 32 };
}

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, end_state, limits());
}
