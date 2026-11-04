//! Historical control, not approval evidence for revision 13: capture is enabled only after publication; the refusal and withdrawal cases are reopened in model_capture_r13.
//!
//! What:
//!  Model of finding F9 (sixth review) and its seventh-review follow-ups: requests behind a request whose commit build is
//!  held by a person's hook, with a disjoint and an overlapping successor, the held request later refusing and later
//!  succeeding, the consumption map's release point, and an amendment that supersedes a running hook.
//! Why:
//!  "A request held by a person's hook" (`doc/planning/cli-git-commits-never-fail.md`) promises that commits behind a
//!  hook keep landing without exposing the held request before its hook can refuse.
//!  The model keeps adoption, publication and materialization as separate events and gives each hook version its own
//!  completion; hooks return only by the environment's choice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; overlap; amend; r1; r2; tipX; tipY; indexX; consumedPresent; r3; ... };
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
    /// Revision 6: pending requests publish in adoption order on the logical tail (a successor either waits for the held
    /// request or is built on top of it).
    Six,
    /// Revision 7: a disjoint successor replays its own delta and publishes ahead; an overlapping one is set aside; the
    /// held request's consumption entry is released at publication; a hook's completion belongs to the request.
    Seven,
    /// A candidate: the consumption entry is released at materialization (or voiding) only, and a hook's completion is
    /// tied to the version that started it and ignored when that version was superseded.
    Candidate,
    /// Revision 8 draft: as the candidate, but a voided request releases its consumption entries (M2), a replay of a held
    /// version spawns its hook again, and a superseded version has no terminal outcome.
    Eight,
    /// Revision 9: M2 keeps the entries as `suppress`, B3 records the superseded version terminal and re-keys its entries,
    /// and a hook runs once per version (`hook-ok`).
    Nine,
}

/// What:
///  `struct State { ... }`: the facts the review's schedules turn on.
///  Numeric fields hold small enumerations, documented on each field.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; overlap: boolean; amend: boolean; r1: number; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The successor `R2` changes the same path as the held `R1` (otherwise a disjoint path).
    pub overlap: bool,
    /// The person amends the held request while its hook runs.
    pub amend: bool,
    /// The held request: 0 held, 1 hook succeeded (will re-enter), 2 refused, 3 published, 4 superseded by an amendment.
    pub r1: u8,
    /// The current version of the held request: 1, or 2 after an amendment.
    pub version: u8,
    /// The version whose hook completed and whose commit the lane will publish (0 none yet).
    pub ready: u8,
    /// The first version's hook: 0 running, 1 returned ok, 2 returned refuse.
    pub hook1: u8,
    /// The second version's hook: 0 not started, 1 running, 2 returned ok, 3 returned refuse.
    pub hook2: u8,
    /// The successor: 0 pending, 1 set aside, 2 published.
    pub r2: u8,
    /// The branch tip's content of the held request's path: 0 unchanged, 1 version one's, 2 version two's.
    pub tip_x: u8,
    /// The branch tip contains the successor's change to its path.
    pub tip_y: bool,
    /// The real index entry of the held request's path: 0 old, 1 new.
    pub index_x: u8,
    /// The consumption map still holds the held request's entry for its path.
    pub consumed: bool,
    /// A foreign program holds `index.lock`.
    pub lock_foreign: bool,
    /// The foreign program already took the lock once.
    pub foreign_took: bool,
    /// A foreign program landed a successor on the held request's path.
    pub foreign_x: bool,
    /// M2 voided the materialization.
    pub voided: bool,
    /// How many times a hook ran again for a version whose hook had already succeeded.
    pub hook_runs: u8,
    /// A foreign program moved the tip after the hook succeeded.
    pub tip_moved: bool,
    /// Version one reached a durable terminal outcome (B3 `superseded`).
    pub v1_terminal: bool,
    /// The held request's index delta was materialized.
    pub materialized: bool,
    /// A later plain commit of another path: 0 not made, 1 made cleanly, 2 captured a reversal of the held request.
    pub r3: u8,
}

/// What:
///  The start state: the held request's hook is running, the successor pending.
/// Why:
///  `overlap` and `amend` pick the variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initial(rules: Rules, overlap: boolean, amend: boolean): State;
/// ```
pub fn initial(rules: Rules, overlap: bool, amend: bool) -> State {
    return State {
        rules,
        overlap,
        amend,
        r1: 0,
        version: 1,
        ready: 0,
        hook1: 0,
        hook2: 0,
        r2: 0,
        tip_x: 0,
        tip_y: false,
        index_x: 0,
        consumed: true,
        lock_foreign: false,
        foreign_took: false,
        foreign_x: false,
        voided: false,
        hook_runs: 0,
        tip_moved: false,
        v1_terminal: false,
        materialized: false,
        r3: 0,
    };
}

/// What:
///  Every step enabled in `state`: the environment's (hook returns, foreign lock), the person's, and the lane's.
/// Why:
///  The explorer takes them in every order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = hook_steps(state);
    found.extend(lane_steps(state));
    if !state.lock_foreign && !state.foreign_took && !state.materialized {
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
    if state.amend && state.version == 1 && state.r1 == 0 {
        let mut after: State = state.clone();
        after.version = 2;
        after.hook2 = 1;
        if state.rules == Rules::Nine {
            // B3: version one is recorded superseded and its entries are re-keyed to version two.
            after.v1_terminal = true;
        }
        if state.rules == Rules::Candidate {
            after.r1 = 0;
        }
        push_step(&mut found, "person: amends the held request, a new version supersedes it and runs its hook", after);
    }
    return found;
}

/// What:
///  The hooks' returns, and what the lane does with them.
/// Why:
///  A hook returns only by the environment's choice; its completion is either the request's or a version's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hookSteps(s: State): [string, State][];
/// ```
fn hook_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let candidate: bool = state.rules == Rules::Candidate || state.rules == Rules::Eight || state.rules == Rules::Nine;
    if state.hook1 == 0 {
        let mut ok: State = state.clone();
        ok.hook1 = 1;
        // Revision 6 and 7 attribute the completion to the request, whatever version started the hook.
        if state.r1 == 0 && (!candidate || state.version == 1) {
            ok.r1 = 1;
            ok.ready = 1;
        }
        push_step(&mut found, "environment: the hook of version one returns ok", ok);
        let mut refused: State = state.clone();
        refused.hook1 = 2;
        if state.r1 == 0 && (!candidate || state.version == 1) {
            refused.r1 = 2;
        }
        push_step(&mut found, "environment: the hook of version one refuses", refused);
    }
    if state.hook2 == 1 {
        let mut ok: State = state.clone();
        ok.hook2 = 2;
        ok.r1 = 1;
        ok.ready = 2;
        push_step(&mut found, "environment: the hook of version two returns ok", ok);
        let mut refused: State = state.clone();
        refused.hook2 = 3;
        refused.r1 = 2;
        push_step(&mut found, "environment: the hook of version two refuses", refused);
    }
    return found;
}

/// What:
///  The lane's enabled steps: publishing the successor, publishing the held request, materializing it, and the plain
///  commit of another path that reads the consumption map.
/// Why:
///  This is where the ordering rule and the release point of the consumption entry are written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function laneSteps(s: State): [string, State][];
/// ```
fn lane_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let six: bool = state.rules == Rules::Six;
    let candidate: bool = state.rules == Rules::Candidate || state.rules == Rules::Eight || state.rules == Rules::Nine;
    let r1_resolved: bool = state.r1 == 2 || state.r1 == 3 || state.r1 == 4;
    if state.r2 == 0 && six {
        // Revision 6, reading one: strict adoption order, so the successor waits for the held request.
        if state.r1 == 3 || state.r1 == 2 {
            let mut after: State = state.clone();
            after.r2 = 2;
            after.tip_y = true;
            push_step(&mut found, "lane: publishes the successor in adoption order after the held request", after);
        }
        // Revision 6, reading two: the successor is built on the logical tail, which includes the held request.
        let mut after: State = state.clone();
        after.r2 = 2;
        after.tip_y = true;
        if state.r1 == 0 {
            after.tip_x = state.version;
        }
        push_step(&mut found, "lane: builds the successor on the logical tail, which includes the held request, and publishes it", after);
    }
    if state.r2 == 0 && !six && !state.overlap {
        let mut after: State = state.clone();
        after.r2 = 2;
        after.tip_y = true;
        push_step(&mut found, "lane: publishes the disjoint successor by replaying its own delta on the tip", after);
    }
    if state.r2 == 0 && !six && state.overlap && !r1_resolved && state.r1 != 3 {
        let mut after: State = state.clone();
        after.r2 = 1;
        push_step(&mut found, "lane: sets the overlapping successor aside until the held request's hook decides", after);
    }
    if (state.r2 == 1 || (state.r2 == 0 && state.overlap)) && !six && r1_resolved {
        let mut after: State = state.clone();
        after.r2 = 2;
        after.tip_y = true;
        push_step(&mut found, "lane: publishes the overlapping successor now that the held request is decided", after);
    }
    if state.r1 == 1 && state.rules == Rules::Eight && state.tip_moved {
        let mut again: State = state.clone();
        again.r1 = 0;
        again.hook1 = 0;
        again.hook_runs += 1;
        again.tip_moved = false;
        push_step(&mut found, "B1: the version is replayed on the moved tip and its hook is spawned again", again);
    }
    if (state.rules == Rules::Eight || state.rules == Rules::Nine) && state.r1 == 1 && !state.tip_moved {
        let mut moved: State = state.clone();
        moved.tip_moved = true;
        push_step(&mut found, "foreign: moves the tip after the hook succeeded", moved);
    }
    if (state.rules == Rules::Eight || state.rules == Rules::Nine) && state.r1 == 3 && !state.foreign_x && !state.materialized {
        let mut after: State = state.clone();
        after.foreign_x = true;
        push_step(&mut found, "foreign: lands a successor on the held request's path", after);
    }
    if (state.rules == Rules::Eight || state.rules == Rules::Nine) && state.r1 == 3 && state.foreign_x && !state.voided && !state.materialized {
        let mut after: State = state.clone();
        after.voided = true;
        if state.rules == Rules::Eight {
            // Revision 8: voiding releases the consumption entries.
            after.consumed = false;
        }
        push_step(&mut found, "M2 void: the entry rule declined because a foreign successor changed the path", after);
    }
    if state.r1 == 1 {
        let mut after: State = state.clone();
        after.r1 = 3;
        after.tip_x = state.ready;
        if !candidate {
            // Revisions 6 and 7 release the consumption entry when the held request is published.
            after.consumed = false;
        }
        push_step(&mut found, "lane: re-enters the held request at the tail and publishes it by replaying its delta on the tip", after);
    }
    if state.r1 == 3 && !state.materialized && !state.lock_foreign && !state.foreign_x {
        let mut after: State = state.clone();
        after.materialized = true;
        after.index_x = 1;
        after.consumed = false;
        push_step(&mut found, "lane: materializes the held request into the index", after);
    }
    if state.r1 == 3 && state.r3 == 0 {
        // A plain commit of another path: x is staged when the index entry differs from the consumption entry if one
        // exists, otherwise from the physical head's tree.
        let reference: u8 = if state.consumed {
            0
        } else if state.foreign_x {
            3
        } else {
            1
        };
        let mut after: State = state.clone();
        after.r3 = if state.index_x != reference { 2 } else { 1 };
        push_step(&mut found, "client: a plain commit stages only another path", after);
    }
    return found;
}

/// What:
///  The safety checks: the held request is not exposed before its hook succeeds, a refused request leaves no trace, a
///  superseded version is never published, and a plain commit never captures a reversal the person did not stage.
/// Why:
///  These are the review's properties for the hook-bypass ordering and the consumption map.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    let decided_ok: bool = state.r1 == 1 || state.r1 == 3;
    if state.tip_x != 0 && !decided_ok {
        return Some("the held request was exposed on the branch before its hook succeeded".to_string());
    }
    if state.tip_x != 0 && state.tip_x != state.version {
        return Some("a superseded version of the held request was published".to_string());
    }
    if state.hook_runs > 0 {
        return Some("a hook ran again for a version whose hook had already succeeded".to_string());
    }
    if state.r3 == 2 {
        return Some("a plain commit captured a reversal of the held request that the person did not stage".to_string());
    }
    // What:     `None` is the absent case of `Option` (TS: `undefined`).
    // Why:      Say there is no value, or no failure, without a sentinel.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return undefined;
    // ```
    return None;
}

/// The liveness check: when only the environment can move, an independent request must not still be waiting.
pub fn end_state(state: &State) -> Option<String> {
    let eight_like: bool = state.rules == Rules::Eight || state.rules == Rules::Nine;
    if eight_like && state.amend && state.version == 2 && !state.v1_terminal {
        return Some("a superseded version never reached a durable terminal outcome".to_string());
    }
    let r1_resolved: bool = state.r1 == 2 || state.r1 == 3 || state.r1 == 4;
    if state.r2 != 2 && (!state.overlap || r1_resolved) {
        return Some("a request behind a held hook is still waiting although it need not".to_string());
    }
    return None;
}

/// The caps: a few thousand states at most.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 64 };
}

/// What:
///  Explores every schedule under `rules`, for a disjoint and an overlapping successor, with and without an amendment.
/// Why:
///  One call covers the review's schedules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function check(rules: Rules): Report;
/// ```
pub fn check(rules: Rules) -> Report {
    return explore(
        vec![
            initial(rules, false, false),
            initial(rules, true, false),
            initial(rules, false, true),
            initial(rules, true, true),
        ],
        steps,
        invariant,
        end_state,
        limits(),
    );
}
