//! Historical control, not approval evidence for revision 13: a classification model with an atomic recovery decision; it keeps no observations, never deletes future evidence and does not model message delivery.
//!
//! What:
//!  Model of finding F1 (sixth review) and its seventh-review follow-up: what recovery may conclude about a publication
//!  after an owner death, from a recorded swap failure, the branch tip and a reflog anchor.
//! Why:
//!  "Publication and recovery" (`doc/planning/cli-git-commits-never-fail.md`) classifies evidence; the absence of a
//!  positive trace is never evidence of absence, and a failed invocation is not proof the request was never satisfied.
//!  The model has the lane's records, a swap that can fail on a held ref lock, another writer that installs and removes
//!  the identical commit, a legitimate reflog edit that deletes the lines after the anchor, and the owner's death.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; tip; recordedPublishing; swapDone; swapFailedRecord; publishedRecord; heldByBranch; evidence; ... };
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
/// type Rules = 'six' | 'seven' | 'candidate' | 'eight' | 'nine' | 'ten';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: an intact anchor with no old-value line after it means the swap did not complete; publish again.
    Six,
    /// Revision 7: a published record, then a recorded swap failure (before any positive evidence), then the tip, then
    /// a positive old-value line after the anchor, and everything else is unclear and landed with a message.
    Seven,
    /// A candidate: published record, then the tip, then positive old-value evidence (saved, not published again), and
    /// everything else after an owner death, a recorded swap failure included, is landed with a message.
    Candidate,
    /// Revision 8 draft (`~/temp/agent/reviews/commits-never-fail-design-r8.md`, recovery classes R1 to R5): published,
    /// tip or ancestor, positive old-value evidence, then a recorded swap failure replays on the tip, then landing.
    Eight,
    /// Revision 9, R4: a recorded swap failure replays on the tip only when the reflog after the anchor is intact; with
    /// the post-anchor lines missing or rewritten the attempt is class R5 (landing with a notice).
    Nine,
    /// Revision 10, R3 to R5: no intact-reflog check (it is not observable, erasing every line after the anchor leaves the log
    /// equal to its anchor record); everything but a published or rewritten-away attempt is class R5, landing with a notice.
    Ten,
}

/// What:
///  `struct State { ... }`: the facts the review's schedules turn on.
///  `outcome` holds the recovery's decision: 0 none yet, 1 published, 2 published again silently, 3 landed with a
///  message, 4 rewritten away and saved.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; tip: number; ... outcome: number };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The branch tip: 0 the old tip `B`, 1 the lane's commit `D`.
    pub tip: u8,
    /// The lane recorded `publishing` with the reflog anchor.
    pub recorded_publishing: bool,
    /// The lane's own swap completed.
    pub swap_done: bool,
    /// The lane recorded `swap-failed`.
    pub swap_failed_record: bool,
    /// The lane recorded `published`.
    pub published_record: bool,
    /// A writer holds the branch's ref lock.
    pub lock_foreign: bool,
    /// A writer already took the ref lock once.
    pub lock_taken: bool,
    /// The branch held `D` at some time (by the lane's swap or by another writer).
    pub held_d: bool,
    /// A reflog line after the anchor has `D` as its old value (positive evidence the branch moved away from `D`).
    pub evidence: bool,
    /// A legitimate reflog operation deleted the lines after the anchor (the anchor stays).
    pub evidence_deleted: bool,
    /// The owner died.
    pub owner_dead: bool,
    /// The recovery's decision.
    pub outcome: u8,
    /// Read at the instant of a silent republication: 1 the reflog showed the branch moved away, 2 the branch had held the
    /// commit and lost it, 0 neither (later events do not change it).
    pub republish_harm: u8,
}

/// What:
///  The start state: nothing recorded.
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
        tip: 0,
        recorded_publishing: false,
        swap_done: false,
        swap_failed_record: false,
        published_record: false,
        lock_foreign: false,
        lock_taken: false,
        held_d: false,
        evidence: false,
        evidence_deleted: false,
        owner_dead: false,
        outcome: 0,
        republish_harm: 0,
    };
}

/// What:
///  The decision recovery takes after the owner died, under the rule the state names.
/// Why:
///  This is the one place the evidence order is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decide(s: State): number;
/// ```
fn decide(state: &State) -> u8 {
    if state.published_record {
        return 1;
    }
    if state.rules == Rules::Seven && state.swap_failed_record {
        return 2;
    }
    // Revision 8, R4: a recorded failure, with no R1 to R3 evidence, replays on the tip.
    if state.rules == Rules::Eight && !(state.tip == 1 || state.evidence) && state.swap_failed_record {
        return 2;
    }
    // Revision 9, R4: only when the reflog after the anchor matches the journal's record of the anchor (line count and last line).
    // That is all the lane can see: erasing every line after the anchor leaves a log equal to its anchor record, so it passes.
    if state.rules == Rules::Nine && !(state.tip == 1 || state.evidence) && state.swap_failed_record {
        return 2;
    }
    if state.tip == 1 {
        return 1;
    }
    if state.evidence {
        return 4;
    }
    if state.rules == Rules::Six {
        // An intact anchor with no old-value line: "the swap did not complete".
        return 2;
    }
    return 3;
}

/// What:
///  Every step enabled in `state`.
/// Why:
///  The explorer takes them in every order, so the owner dies between any two records and the reflog edit falls anywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let live: bool = !state.owner_dead;
    if live && !state.recorded_publishing {
        let mut after: State = state.clone();
        after.recorded_publishing = true;
        push_step(&mut found, "lane: records publishing with the reflog anchor", after);
    }
    if live && state.recorded_publishing && !state.swap_done && !state.swap_failed_record && state.tip == 0 && !state.lock_foreign {
        let mut after: State = state.clone();
        after.tip = 1;
        after.swap_done = true;
        after.held_d = true;
        push_step(&mut found, "lane: the swap succeeds", after);
    }
    if live && state.recorded_publishing && !state.swap_done && !state.swap_failed_record && state.lock_foreign {
        let mut after: State = state.clone();
        after.swap_failed_record = true;
        push_step(&mut found, "lane: the swap fails because another writer holds the ref lock, and records swap-failed", after);
    }
    if live && state.swap_done && !state.published_record {
        let mut after: State = state.clone();
        after.published_record = true;
        push_step(&mut found, "lane: records published", after);
    }
    if !state.lock_foreign && !state.lock_taken {
        let mut after: State = state.clone();
        after.lock_foreign = true;
        after.lock_taken = true;
        push_step(&mut found, "foreign: takes the ref lock", after);
    }
    if state.lock_foreign {
        let mut after: State = state.clone();
        after.lock_foreign = false;
        push_step(&mut found, "foreign: releases the ref lock", after);
    }
    if !state.lock_foreign && state.tip == 0 && !state.held_d && state.recorded_publishing {
        let mut after: State = state.clone();
        after.tip = 1;
        after.held_d = true;
        push_step(&mut found, "foreign: installs exactly the lane's commit", after);
    }
    if !state.lock_foreign && state.tip == 1 {
        let mut after: State = state.clone();
        after.tip = 0;
        after.evidence = !state.evidence_deleted;
        push_step(&mut found, "foreign: resets the branch away from the commit", after);
    }
    if state.recorded_publishing && !state.evidence_deleted {
        let mut after: State = state.clone();
        after.evidence = false;
        after.evidence_deleted = true;
        push_step(&mut found, "reflog operation: deletes the lines after the anchor, keeping the anchor", after);
    }
    if live && state.recorded_publishing && !state.published_record {
        let mut after: State = state.clone();
        after.owner_dead = true;
        push_step(&mut found, "environment: the owner dies without a durable published record", after);
    }
    if state.owner_dead && state.outcome == 0 {
        let mut after: State = state.clone();
        after.outcome = decide(state);
        if after.outcome == 2 && state.evidence {
            after.republish_harm = 1;
        } else if after.outcome == 2 && state.held_d && state.tip == 0 {
            after.republish_harm = 2;
        }
        push_step(&mut found, "recovery: decides from the evidence", after);
    }
    return found;
}

/// What:
///  The safety checks: recovery never publishes silently a commit the branch held, and never publishes again when the
///  branch is positively known to have moved away from it.
/// Why:
///  Absence of evidence is not evidence of absence, and who installed the identical commit does not matter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    if state.republish_harm == 1 {
        return Some("a commit was published again although the reflog shows the branch moved away from it".to_string());
    }
    if state.republish_harm == 2 {
        return Some("a commit the branch had held and lost was published again silently".to_string());
    }
    return None;
}

/// A state with no enabled step needs no extra check here.
pub fn end_state(_state: &State) -> Option<String> {
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
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, end_state, limits());
}
