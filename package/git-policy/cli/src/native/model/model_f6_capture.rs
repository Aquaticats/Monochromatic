//! Historical control, not approval evidence for revision 13: revisions before the state log; the odd-generation marker it checks is retired.
//!
//! What:
//!  Model of finding F6 (sixth review) and its seventh-review follow-up: a plain commit captures the staged state
//!  while the lane materializes a published request.
//! Why:
//!  "Capture consistency" (`doc/planning/cli-git-commits-never-fail.md`) must never let the wrapper's own
//!  materialization look like the person's staging, and acceptance must not wait behind an external program.
//!  One lane mutation writes three facts (the logical head, the index entry, the consumption entry); a capture reads
//!  the same three. A snapshot that mixes before and after of one mutation invents a staged change.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; generation; tip; index; consumed; lane: LanePhase; reader: ReaderPhase; ... };
//! ```

// What:     imports the explorer's types and helpers used here.
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
/// type Rules = 'six' | 'seven' | 'candidate' | 'eight' | 'nine';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: read a state version before and after; the version advances with the last write.
    Six,
    /// Revision 7: intent journaled, generation odd, then every mutation including `update-ref` and its hook, then even.
    Seven,
    /// A candidate: the odd generation brackets only the lane's local state writes; `update-ref` and its hook run
    /// outside it, and a capture reads the lane's logical head, not the raw ref.
    Candidate,
    /// Revision 8 draft, L4: a new owner completes or rolls back any journaled mutation and writes an even generation;
    /// read literally, it does not raise the generation before the writes it repeats.
    Eight,
    /// Revision 9, L4 and C0: a resuming owner writes the generation odd first, and a client that finds a dead odd generation
    /// with no service acts as an inline owner for recovery (C0). Capture reads the state document once (rule S).
    Nine,
}

/// What:
///  `struct State { ... }`: the lane's published facts, the lane's progress and the reader's progress.
///  `u8` fields hold 0 (before the mutation) or 1 (after it) for the three facts.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; generation: number; tip: number; index: number; consumed: number; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// Revision 6: the state version. Revision 7 and the candidate: the generation (odd while a mutation runs).
    pub generation: u8,
    /// The logical head the lane's state shows.
    pub tip: u8,
    /// The real index entry of the path.
    pub index: u8,
    /// The consumption map's entry of the path.
    pub consumed: u8,
    /// How far the lane's one mutation got (0 idle, then one number per step, see `lane_steps`).
    pub lane: u8,
    /// A service exists that starts a new owner after a death (otherwise only a client can).
    pub service: bool,
    /// The lane process died in the middle of its mutation (revisions 7 and the candidate only).
    pub lane_dead: bool,
    /// How far the capture got (0 idle; 1 first generation read; 2 head read; 3 index read; 4 consumption read).
    pub reader: u8,
    /// The generation or version the capture read first.
    pub first: u8,
    /// What the capture read.
    pub seen_tip: u8,
    /// What the capture read of the index.
    pub seen_index: u8,
    /// What the capture read of the consumption map.
    pub seen_consumed: u8,
    /// The capture was accepted.
    pub accepted: bool,
    /// The accepted snapshot mixed before and after of the lane's mutation.
    pub mixed: bool,
}

/// What:
///  The start state: nothing written, nothing read.
/// Why:
///  Every schedule begins here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initial(rules: Rules): State;
/// ```
pub fn initial(rules: Rules) -> State {
    return State {
        rules,
        generation: 0,
        tip: 0,
        index: 0,
        consumed: 0,
        lane: 0,
        service: true,
        lane_dead: false,
        reader: 0,
        first: 0,
        seen_tip: 0,
        seen_index: 0,
        seen_consumed: 0,
        accepted: false,
        mixed: false,
    };
}

/// What:
///  The steps of the lane's one mutation: publish a request and materialize it.
///  Revision 6: write head, write index, [pause], write consumption and advance the version.
///  Revision 7: journal, odd, `update-ref` (its hook may wait), write head, index, consumption, even.
///  Candidate: journal, `update-ref` (its hook may wait), odd, write head, index, consumption, even.
/// Why:
///  The explorer needs the writes as separate steps so a capture can fall between any two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function laneSteps(s: State): [string, State][];
/// ```
fn lane_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let mut after: State = state.clone();
    if state.lane_dead {
        return found;
    }
    // What:     `match (a, b) { (X, 0) => ..., (Y, 1) | (Z, 1) => ..., _ => {} }` compares a pair against patterns top to
    //           bottom and runs the first arm that fits; `|` joins patterns that share an arm; `_ => {}` is the arm for
    //           everything else and does nothing. Each arm here is one step of the lane's script, found by
    //           (rule, progress).
    // Why:      The script differs by rule only in its order and content, and a table of arms shows each rule's
    //           script next to the others.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (`${rules}:${lane}`) { case 'six:0': ...; case 'seven:0': case 'candidate:0': ...; default: break; }
    // ```
    match (state.rules, state.lane) {
        (Rules::Six, 0) => {
            after.lane = 1;
            after.tip = 1;
            push_step(&mut found, "lane: writes the logical head", after);
        }
        (Rules::Six, 1) => {
            after.lane = 2;
            after.index = 1;
            push_step(&mut found, "lane: installs the index entry", after);
        }
        (Rules::Six, 2) => {
            after.lane = 3;
            after.consumed = 1;
            after.generation += 1;
            push_step(&mut found, "lane: updates the consumption map and advances the state version", after);
        }
        (Rules::Seven, 0) | (Rules::Candidate, 0) | (Rules::Nine, 0) | (Rules::Eight, 0) => {
            after.lane = 1;
            push_step(&mut found, "lane: journals the mutation and fsyncs it", after);
        }
        (Rules::Seven, 1) => {
            after.lane = 2;
            after.generation += 1;
            push_step(&mut found, "lane: writes the generation odd", after);
        }
        (Rules::Seven, 2) | (Rules::Candidate, 1) | (Rules::Nine, 1) | (Rules::Eight, 1) => {
            // `update-ref` runs; a reference-transaction hook may wait for as long as it likes.
            after.lane = if state.rules == Rules::Seven { 3 } else { 2 };
            push_step(&mut found, "lane: invokes update-ref, whose reference-transaction hook waits", after);
        }
        (Rules::Seven, 3) | (Rules::Candidate, 2) | (Rules::Nine, 2) | (Rules::Eight, 2) => {
            // The hook returns only by the environment's choice.
            after.lane = if state.rules == Rules::Seven { 4 } else { 3 };
            push_step(&mut found, "environment: the reference-transaction hook returns", after);
        }
        (Rules::Candidate, 3) | (Rules::Nine, 3) | (Rules::Eight, 3) => {
            after.lane = 4;
            after.generation += 1;
            push_step(&mut found, "lane: writes the generation odd", after);
        }
        (Rules::Seven, 4) | (Rules::Candidate, 4) | (Rules::Nine, 4) | (Rules::Eight, 4) => {
            after.lane = 5;
            after.tip = 1;
            push_step(&mut found, "lane: writes the logical head", after);
        }
        (Rules::Seven, 5) | (Rules::Candidate, 5) | (Rules::Nine, 5) | (Rules::Eight, 5) => {
            after.lane = 6;
            after.index = 1;
            push_step(&mut found, "lane: installs the index entry", after);
        }
        (Rules::Seven, 6) | (Rules::Candidate, 6) | (Rules::Nine, 6) | (Rules::Eight, 6) => {
            after.lane = 7;
            after.consumed = 1;
            push_step(&mut found, "lane: updates the consumption map", after);
        }
        (Rules::Seven, 7) | (Rules::Candidate, 7) | (Rules::Nine, 7) | (Rules::Eight, 7) => {
            after.lane = 8;
            after.generation += 1;
            push_step(&mut found, "lane: writes the generation even", after);
        }
        (Rules::Eight, 9) => {
            after.lane = 10;
            after.tip = 1;
            push_step(&mut found, "L4 resume: writes the logical head with the generation unchanged", after);
        }
        (Rules::Eight, 10) => {
            after.lane = 11;
            after.index = 1;
            push_step(&mut found, "L4 resume: installs the index entry", after);
        }
        (Rules::Eight, 11) => {
            after.lane = 12;
            after.consumed = 1;
            push_step(&mut found, "L4 resume: updates the consumption map", after);
        }
        (Rules::Eight, 12) => {
            after.lane = 8;
            after.generation += 2;
            push_step(&mut found, "L4 resume: writes an even generation", after);
        }
        _ => {}
    }
    // The lane process may die with a mutation half done; a new owner completes it from the journal.
    if state.rules != Rules::Six && state.lane >= 1 && state.lane < 8 {
        let mut died: State = state.clone();
        died.lane_dead = true;
        push_step(&mut found, "environment: the lane process dies", died);
    }
    return found;
}

/// What:
///  The steps of the capture: read the generation, the head, the index and the consumption entry, read the generation
///  again, accept only an unchanged even value (revisions 7 and the candidate) or an unchanged version (revision 6).
/// Why:
///  The capture is the reader of the seqlock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readerSteps(s: State): [string, State][];
/// ```
fn reader_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let mut after: State = state.clone();
    if state.accepted {
        return found;
    }
    let locks: bool = state.rules != Rules::Six;
    // What:     `match state.reader { 0 => ..., 1 => ..., _ => {} }` runs the arm for the capture's progress number.
    // Why:      The capture is a six-step script; the arm is the step.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (state.reader) { case 0: ...; case 1: ...; default: break; }
    // ```
    match state.reader {
        0 => {
            // A seqlock reader waits while the generation is odd.
            if !locks || state.generation % 2 == 0 {
                after.reader = 1;
                after.first = state.generation;
                push_step(&mut found, "client: reads the generation", after);
            }
        }
        1 => {
            after.reader = 2;
            after.seen_tip = state.tip;
            push_step(&mut found, "client: reads the logical head", after);
        }
        2 => {
            after.reader = 3;
            after.seen_index = state.index;
            push_step(&mut found, "client: reads the index entry", after);
        }
        3 => {
            after.reader = 4;
            after.seen_consumed = state.consumed;
            push_step(&mut found, "client: reads the consumption entry", after);
        }
        4 => {
            if state.generation == state.first {
                after.accepted = true;
                after.mixed = !(state.seen_tip == state.seen_index && state.seen_index == state.seen_consumed);
                push_step(&mut found, "client: reads the generation again, unchanged, and accepts the capture", after);
            } else {
                after.reader = 0;
                push_step(&mut found, "client: reads the generation again, changed, and starts over", after);
            }
        }
        _ => {}
    }
    return found;
}

/// What:
///  Every step enabled in `state`: the lane's, the capture's and a new owner's recovery.
/// Why:
///  The explorer takes them in every order, so the capture falls between any two lane writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = lane_steps(state);
    found.extend(reader_steps(state));
    if state.lane_dead {
        // A new owner resumes the journaled mutation as a writer: it follows the same script, so the generation goes
        // odd before any write it repeats. An `update-ref` that was in flight is invoked again (its evidence rules
        // are outside this model).
        let mut after: State = state.clone();
        after.lane_dead = false;
        if state.rules == Rules::Seven && state.lane == 3 {
            after.lane = 2;
        }
        if (state.rules == Rules::Candidate || state.rules == Rules::Nine) && state.lane == 2 {
            after.lane = 1;
        }
        if state.rules == Rules::Eight && state.lane == 2 {
            after.lane = 1;
        }
        // Read literally, L4 completes the journaled writes when the generation was never raised and then writes an even one.
        if state.rules == Rules::Eight && state.lane <= 2 && state.generation % 2 == 0 {
            after.lane = 9;
        }
        let label: &str = if state.service {
            "lane: a new owner resumes the journaled mutation from its script"
        } else if state.rules == Rules::Nine {
            "C0 ensure the lane: the client acts as an inline owner and resumes the journaled mutation"
        } else {
            "environment: a new owner starts and resumes the journaled mutation from its script"
        };
        push_step(&mut found, label, after);
    }
    return found;
}

/// What:
///  The safety check: an accepted capture never mixes before and after of one lane mutation.
/// Why:
///  A mixed snapshot reads as the person's staging of the entry, which reverts a pending request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    if state.accepted && state.mixed {
        return Some("a capture accepted a snapshot that mixes before and after of one lane mutation".to_string());
    }
    return None;
}

/// The liveness check: a state where only the environment can move must not leave the capture unaccepted.
pub fn end_state(state: &State) -> Option<String> {
    if !state.accepted {
        return Some("acceptance is blocked behind an external program: the generation stays odd".to_string());
    }
    return None;
}

/// The caps: a few hundred states at most.
pub fn limits() -> Limits {
    return Limits { max_states: 100_000, max_depth: 64 };
}

/// What:
///  Explores every schedule under `rules`.
/// Why:
///  One call covers the review's schedules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function check(rules: Rules): Report;
/// ```
pub fn initial_with(rules: Rules, service: bool) -> State {
    let mut state: State = initial(rules);
    state.service = service;
    return state;
}

/// Explores every schedule under `rules`; revisions 8 and 9 are also explored with no service to restart the owner.
pub fn check(rules: Rules) -> Report {
    let mut starts: Vec<State> = vec![initial(rules)];
    if rules == Rules::Eight || rules == Rules::Nine {
        starts.push(initial_with(rules, false));
    }
    return explore(starts, steps, invariant, end_state, limits());
}
