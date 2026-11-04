//! Historical control, not approval evidence for revision 13: acquisition, publication and application are atomic and the switch changes HEAD only.
//!
//! What:
//!  Model of finding F5 (sixth review) and its seventh-review follow-up: a request is captured on `main`, the person
//!  switches the worktree to another branch, and the lane applies the commit's index delta.
//! Why:
//!  The attachment guard (`doc/planning/cli-git-commits-never-fail.md`, "Adoption order") is a guard, not a fence; the
//!  owner accepted one residue (a switch after the last guard read stages the delta on the new branch, index only,
//!  recoverable).
//!  The seventh review shows the guard applies only to the deferred follow-up, so a first-try `index.lock` skips it.
//!  The model keeps the accepted residue as a reachable state and checks that nothing else stages the delta.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; headOnMain; lockForeign; published; applied; declined; guardBefore; guardAfter; ... };
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
/// type Rules = 'six' | 'seven' | 'candidate' | 'eight';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: the follow-up declines when the `HEAD` reflog grew since capture; the first-try path has no guard.
    Six,
    /// Revision 7: the deferred follow-up reads `HEAD` before and after the index; the first-try `index.lock` path
    /// applies the delta with no guard at all.
    Seven,
    /// A candidate: both paths read `HEAD` first and apply the delta only when it is attached to the request's branch;
    /// the deferred path keeps the bracket.
    Candidate,
    /// Revision 8 draft, M1: one path for every follow-up; the lock is tried after the publication, never before it or
    /// across the swap, and `HEAD` is read before and after the index.
    Eight,
}

/// What:
///  `struct State { ... }`: the facts the review's schedules turn on.
///  `u8` fields hold three-way answers: 0 not read, 1 read attached to `main`, 2 read attached elsewhere.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; headOnMain: boolean; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The worktree's `HEAD` is attached to `main`, the branch the request was captured on.
    pub head_on_main: bool,
    /// The person switched the worktree away from `main` (at most once).
    pub person_switched: bool,
    /// A foreign program holds `index.lock`.
    pub lock_foreign: bool,
    /// The foreign program already took the lock once (to bound the search).
    pub foreign_took: bool,
    /// The lane swapped `main` to the commit.
    pub published: bool,
    /// The index delta was applied.
    pub applied: bool,
    /// The lane declined the follow-up.
    pub declined: bool,
    /// The publication grew the `HEAD` reflog (it does when `HEAD` is attached to the branch).
    pub reflog_grew: bool,
    /// Candidate: the read before the first-try critical section: 0 not read, 1 attached to `main`, 2 elsewhere.
    pub pre_read: u8,
    /// The first read of the deferred bracket: 0 not read, 1 attached to `main`, 2 elsewhere.
    pub guard_before: u8,
    /// The second guard read of the deferred bracket.
    pub guard_after: u8,
    /// Harm: the delta was staged on a worktree attached elsewhere with no guard read that saw `main`.
    pub unguarded_staging: bool,
    /// The accepted residue: the delta was staged after a guard that saw `main`, and `HEAD` had moved since.
    pub accepted_residue: bool,
    /// Harm: the follow-up was declined although `HEAD` never left `main`.
    pub wrongly_declined: bool,
}

/// What:
///  The start state: attached to `main`, nothing published.
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
        head_on_main: true,
        person_switched: false,
        lock_foreign: false,
        foreign_took: false,
        published: false,
        applied: false,
        declined: false,
        reflog_grew: false,
        pre_read: 0,
        guard_before: 0,
        guard_after: 0,
        unguarded_staging: false,
        accepted_residue: false,
        wrongly_declined: false,
    };
}

/// What:
///  The answer a guard read gives now: 1 attached to `main`, 2 elsewhere.
/// Why:
///  A guard read is a snapshot of `HEAD` at one instant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = (s: State) => (s.headOnMain ? 1 : 2);
/// ```
fn read_head(state: &State) -> u8 {
    if state.head_on_main {
        return 1;
    }
    return 2;
}

/// What:
///  Applies the index delta in `after` and classifies it: unguarded staging is a harm, staging after a guard that saw
///  `main` is the accepted residue.
/// Why:
///  One place decides what a delta applied to the index means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function apply(s: State): void;
/// ```
fn apply_delta(after: &mut State, guarded: bool) {
    after.applied = true;
    if !after.head_on_main {
        if guarded {
            after.accepted_residue = true;
        } else {
            after.unguarded_staging = true;
        }
    }
}

/// What:
///  Every step enabled in `state`.
/// Why:
///  The explorer takes them in every order, so the switch falls before the publication, between the guard reads and
///  after the last one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let six: bool = state.rules == Rules::Six;
    let candidate: bool = state.rules == Rules::Candidate;
    if state.head_on_main && !state.person_switched {
        let mut after: State = state.clone();
        after.head_on_main = false;
        after.person_switched = true;
        push_step(&mut found, "person: switches the worktree to topic, carrying an edit", after);
    }
    if !state.lock_foreign && !state.foreign_took && !state.applied {
        let mut after: State = state.clone();
        after.lock_foreign = true;
        after.foreign_took = true;
        push_step(&mut found, "foreign: takes index.lock", after);
    }
    if state.lock_foreign {
        let mut after: State = state.clone();
        after.lock_foreign = false;
        push_step(&mut found, "foreign: releases index.lock", after);
    }
    // The first-try path: publication and materialization in one critical section.
    if candidate && !state.published && state.pre_read == 0 {
        let mut after: State = state.clone();
        after.pre_read = read_head(state);
        push_step(&mut found, "lane: reads HEAD before its critical section", after);
    }
    let eight: bool = state.rules == Rules::Eight;
    // What:     `|x| expr` is a closure (an unnamed function; TS: `(x) => expr`). It is passed to a method that calls it.
    // Why:      Say in one place what to do for each element or error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (x) => expr
    // ```
    if !state.published && !state.lock_foreign && !eight && (!candidate || state.pre_read != 0) {
        let mut after: State = state.clone();
        after.published = true;
        after.reflog_grew = state.head_on_main;
        if !candidate {
            apply_delta(&mut after, false);
            push_step(&mut found, "lane: takes index.lock on the first try, swaps the ref and applies the delta", after);
        } else if state.pre_read == 1 {
            apply_delta(&mut after, true);
            push_step(&mut found, "lane: takes index.lock on the first try, swaps the ref and applies the delta", after);
        } else {
            after.declined = true;
            push_step(&mut found, "lane: swaps the ref without the index because HEAD is attached elsewhere", after);
        }
    }
    // The deferred path: publication without the index while a foreign program holds the lock.
    if !state.published && (state.lock_foreign || eight) {
        let mut after: State = state.clone();
        after.published = true;
        after.reflog_grew = state.head_on_main;
        push_step(&mut found, "lane: swaps the ref without the index lock", after);
    }
    let pending: bool = state.published && !state.applied && !state.declined && !state.lock_foreign;
    if pending && six {
        let mut after: State = state.clone();
        if state.reflog_grew || !state.head_on_main {
            after.declined = true;
            if state.head_on_main && !state.person_switched {
                after.wrongly_declined = true;
            }
            push_step(&mut found, "lane: declines the follow-up because the HEAD reflog grew since capture", after);
        } else {
            apply_delta(&mut after, false);
            push_step(&mut found, "lane: applies the follow-up with no attachment guard", after);
        }
    }
    if pending && !six && state.guard_before == 0 {
        // The bracket: the first read, then the second read after the index was read.
        let mut after: State = state.clone();
        after.guard_before = read_head(state);
        push_step(&mut found, "lane: reads HEAD before reading the index", after);
    }
    if pending && !six && state.guard_before != 0 && state.guard_after == 0 {
        let mut after: State = state.clone();
        after.guard_after = read_head(state);
        push_step(&mut found, "lane: reads the index and then HEAD again", after);
    }
    if pending && !six && state.guard_before != 0 && state.guard_after != 0 {
        let mut after: State = state.clone();
        if state.guard_before == 1 && state.guard_after == 1 {
            apply_delta(&mut after, true);
            push_step(&mut found, "lane: applies the follow-up", after);
        } else {
            after.declined = true;
            push_step(&mut found, "lane: declines the follow-up because HEAD is attached elsewhere", after);
        }
    }
    return found;
}

/// What:
///  The safety checks: no delta is staged without a guard read that saw `main`, and no follow-up is declined while
///  `HEAD` never left `main`.
/// Why:
///  The first is "the immediate path is not exempt from the guard"; the second is "growth of the reflog is neither a
///  complete switch detector nor evidence of a switch".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    if state.unguarded_staging {
        return Some("the delta was staged on a worktree attached to another branch with no guard read".to_string());
    }
    if state.wrongly_declined {
        return Some("a valid follow-up was declined although HEAD never left the branch".to_string());
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
