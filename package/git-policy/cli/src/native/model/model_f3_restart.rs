//! Historical control, not approval evidence for revision 13: revisions seven to nine; actors are classified by label prefix and its goal exempts a wedged gate holder under every rule.
//!
//! What:
//!  Model of finding F3 (sixth review) and its seventh-review follow-ups: a client restarts a quiet owner while the
//!  owner may dispatch a person's hook, with a gate lock, a pidfd, a restarter that can die, a wedged owner and a
//!  restarter in another PID namespace.
//! Why:
//!  "Restarting a quiet owner" (`doc/planning/cli-git-commits-never-fail.md`) promises that the person's hook is
//!  never ended and that a delayed request never reaches a replacement owner.
//!  The model keeps the observation, the gate, the signal and the owner's death as separate steps, lets the restarter
//!  die after any of them, and assumes no hook or owner ever finishes or runs when it is needed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; owner: Owner; gate: Gate; hookRunning; r: RPhase; ... };
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
/// type Rules = 'six' | 'seven' | 'candidate' | 'eight' | 'nine' | 'candidate-no-gate';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: observe a quiet owner with no hook, then ask the manager to restart the unit by name.
    Six,
    /// Revision 7: the owner holds the shared gate around every hook and ref transaction; the restarter takes the
    /// gate exclusively without waiting, re-reads the incarnation and sends the terminate signal through a pidfd
    /// opened from the pid in `lane/incarnation`.
    Seven,
    /// A candidate: a launcher stub, not the owner, holds the shared gate for exactly as long as the person's program
    /// runs; the restarter sends an uncatchable kill through a pidfd obtained from the manager, so the cancellation
    /// cannot be revoked by the restarter's own death and the pid is never interpreted in another namespace.
    Candidate,
    /// Revision 8 draft, L6 and L7: the gate is held shared around every program; the restarter, in the owner's pid
    /// namespace only, takes it exclusively, re-reads the incarnation, creates a restart-intent, then ends the owner through
    /// the pidfd (read here as a terminate signal, since the intent exists for the owner that is still running); an owner that
    /// finds the intent at an admission point admits nothing and exits.
    Eight,
    /// Revision 9, L6 and L7: as revision 8, and step (5) ends the owner through the pidfd with SIGKILL (uncatchable), so a
    /// wedged owner outside any gate is ended; the shared gate is checked against the restart-intent after it is held.
    Nine,
    /// The candidate without the exclusive gate (a mutation that must stay red).
    CandidateNoGate,
}

/// What:
///  `enum RPhase { ... }`: how far the restarter got.
/// Why:
///  The restarter dying "after any step" needs each step to be a phase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RPhase = 'idle' | 'observed' | 'gate' | 'verified' | 'signalled' | 'done' | 'dead';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RPhase {
    /// Nothing done.
    Idle,
    /// Saw a quiet owner (revision 6) or opened a pidfd (revisions 7 and the candidate).
    Observed,
    /// Holds the gate exclusively.
    Gate,
    /// Re-read the incarnation under the gate and found it unchanged.
    Verified,
    /// Created the restart-intent (revision 8).
    Intent,
    /// Sent the signal; waiting for the owner's death.
    Signalled,
    /// Finished or aborted.
    Done,
    /// The restarter process died.
    Dead,
}

/// What:
///  `struct State { ... }`: the facts the review's schedules turn on.
/// Why:
///  Flat `Copy` fields hash cheaply; the `*_ended` and `*_killed` flags record a harm once it happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; foreignNamespace: boolean; incarnation: number; ownerAlive: boolean; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The restarter runs in another PID namespace than the owner.
    pub foreign_namespace: bool,
    /// The current owner's incarnation: 1, or 2 after a replacement.
    pub incarnation: u8,
    /// The current owner process lives.
    pub owner_alive: bool,
    /// The owner is wedged and takes no further step.
    pub owner_wedged: bool,
    /// An uncatchable kill is pending on the owner (it takes no further step).
    pub kill_pending: bool,
    /// A catchable terminate signal is pending on the owner (it may still take steps).
    pub term_pending: bool,
    /// Who holds the shared gate: 0 nobody, 1 the owner, 2 the launcher stub.
    pub gate_shared: u8,
    /// The restarter holds the gate exclusively.
    pub gate_exclusive: bool,
    /// The person's hook of the current owner is running.
    pub hook_running: bool,
    /// The current owner already dispatched its hook (it dispatches at most once, to bound the search).
    pub dispatched: bool,
    /// The restarter.
    pub restarter: RPhase,
    /// The incarnation the restarter observed.
    pub observed: u8,
    /// The restarter's pidfd names an unrelated process (the pid was read in the wrong namespace).
    pub target_unrelated: bool,
    /// Revision 8: the incarnation named by the restart-intent file (0 for none).
    pub intent: u8,
    /// Revision 6: a restart request is outstanding at the manager, addressed to the unit by name.
    pub stop_outstanding: bool,
    /// Harm: a running person's program was ended.
    pub program_ended: bool,
    /// Harm: an owner of an incarnation other than the observed one was ended.
    pub wrong_owner_ended: bool,
    /// Harm: an unrelated process was killed.
    pub unrelated_killed: bool,
}

/// What:
///  The start state: owner incarnation 1 alive, nothing dispatched, the restarter idle.
/// Why:
///  `foreign_namespace` picks whether the restarter's pid interpretation is wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initial(rules: Rules, foreignNamespace: boolean): State;
/// ```
pub fn initial(rules: Rules, foreign_namespace: bool) -> State {
    return State {
        rules,
        foreign_namespace,
        incarnation: 1,
        owner_alive: true,
        owner_wedged: false,
        kill_pending: false,
        term_pending: false,
        gate_shared: 0,
        gate_exclusive: false,
        hook_running: false,
        dispatched: false,
        restarter: RPhase::Idle,
        observed: 0,
        target_unrelated: false,
        intent: 0,
        stop_outstanding: false,
        program_ended: false,
        wrong_owner_ended: false,
        unrelated_killed: false,
    };
}

/// What:
///  The state after the owner is gone: its gate share and its hook go with it (the unit's control group is ended).
///  `program_lost` says whether the end was a restart that ended a running program.
/// Why:
///  The manager ends the rest of the unit when the owner dies, so a hook running then is ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerGone(s: State, byRestart: boolean): State;
/// ```
fn owner_gone(state: &State, by_restart: bool) -> State {
    let mut after: State = state.clone();
    if by_restart && state.hook_running {
        after.program_ended = true;
    }
    after.owner_alive = false;
    after.kill_pending = false;
    after.term_pending = false;
    after.hook_running = false;
    after.gate_shared = 0;
    return after;
}

/// Whether the rule is revision 8 or revision 9, which share the gate, the restart-intent and the namespace rule.
fn eight_like(rules: Rules) -> bool {
    // What:     `|x| expr` is a closure (an unnamed function; TS: `(x) => expr`). It is passed to a method that calls it.
    // Why:      Say in one place what to do for each element or error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (x) => expr
    // ```
    return rules == Rules::Eight || rules == Rules::Nine;
}

/// Whether the restarter's pidfd still names the live owner it observed.
fn pidfd_names_live_owner(state: &State) -> bool {
    return !state.target_unrelated && state.owner_alive && state.incarnation == state.observed;
}

/// What:
///  Every step enabled in `state`: the owner's, the hook's, the environment's, the restarter's and the manager's.
/// Why:
///  The explorer takes them in every order, so the restarter's death and the owner's admission of a hook are
///  interleaved at every point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    found.extend(owner_steps(state));
    found.extend(restarter_steps(state));
    if state.owner_alive {
        // The owner can die for reasons outside the wrapper; the manager then starts a replacement.
        push_step(&mut found, "outside: the owner dies", owner_gone(state, false));
    }
    if !state.owner_alive && state.incarnation == 1 {
        let mut after: State = state.clone();
        after.incarnation = 2;
        after.owner_alive = true;
        after.owner_wedged = false;
        after.dispatched = false;
        push_step(&mut found, "manager: starts a replacement owner", after);
    }
    if state.stop_outstanding && state.owner_alive {
        let mut after: State = owner_gone(state, true);
        after.stop_outstanding = false;
        if state.incarnation != state.observed {
            after.wrong_owner_ended = true;
        }
        push_step(&mut found, "manager: executes the restart of the unit", after);
    }
    return found;
}

/// What:
///  The owner's and the hook's enabled steps.
/// Why:
///  Revision 6 has no gate; revision 7's owner holds the shared gate itself; the candidate's launcher stub holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownerSteps(s: State): [string, State][];
/// ```
fn owner_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let stub: bool = state.rules == Rules::Candidate || state.rules == Rules::CandidateNoGate;
    let gated: bool = state.rules != Rules::Six;
    let free: bool = state.owner_alive && !state.owner_wedged && !state.kill_pending;
    let intent_set: bool = eight_like(state.rules) && state.intent == state.incarnation;
    if free && intent_set && !state.hook_running {
        push_step(&mut found, "L7 intent: the owner finds the restart-intent at an admission point, admits nothing and exits", owner_gone(state, false));
    }
    if free && !intent_set && !state.dispatched && gated && !stub && state.gate_shared == 0 && !state.gate_exclusive {
        let mut after: State = state.clone();
        after.gate_shared = 1;
        push_step(&mut found, "owner: takes the shared gate", after);
    }
    if free && !state.dispatched && !gated {
        let mut after: State = state.clone();
        after.hook_running = true;
        after.dispatched = true;
        push_step(&mut found, "owner: dispatches the person's hook", after);
    }
    if free && !state.dispatched && gated && !stub && state.gate_shared == 1 {
        let mut after: State = state.clone();
        after.hook_running = true;
        after.dispatched = true;
        push_step(&mut found, "owner: dispatches the person's hook", after);
    }
    if free && !state.dispatched && stub && (!state.gate_exclusive || state.rules == Rules::CandidateNoGate) {
        let mut after: State = state.clone();
        after.hook_running = true;
        after.dispatched = true;
        after.gate_shared = 2;
        push_step(&mut found, "owner: launches the stub, which takes the shared gate and executes the hook", after);
    }
    if state.hook_running && state.owner_alive {
        let mut after: State = state.clone();
        after.hook_running = false;
        if state.gate_shared == 2 {
            after.gate_shared = 0;
        }
        push_step(&mut found, "hook: returns", after);
    }
    if free && state.gate_shared == 1 && !state.hook_running && state.dispatched {
        let mut after: State = state.clone();
        after.gate_shared = 0;
        push_step(&mut found, "owner: releases the shared gate", after);
    }
    if state.owner_alive && !state.owner_wedged {
        let mut after: State = state.clone();
        after.owner_wedged = true;
        push_step(&mut found, "environment: the owner wedges and takes no further step", after);
    }
    if state.owner_alive && state.term_pending && !state.owner_wedged && !state.kill_pending {
        push_step(&mut found, "owner: handles the terminate signal and exits", owner_gone(state, true));
    }
    if state.owner_alive && state.kill_pending {
        push_step(&mut found, "kernel: completes the kill of the owner", owner_gone(state, true));
    }
    return found;
}

/// What:
///  The restarter's enabled steps (and its death) under the rule the state names.
/// Why:
///  This is where revision 6 (unit name), revision 7 (gate, pidfd, terminate signal) and the candidate (gate, manager
///  pidfd, uncatchable kill) differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restarterSteps(s: State): [string, State][];
/// ```
fn restarter_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let phase: RPhase = state.restarter;
    let six: bool = state.rules == Rules::Six;
    let candidate: bool = state.rules == Rules::Candidate || state.rules == Rules::CandidateNoGate;
    if phase == RPhase::Idle && six && state.owner_alive && !state.hook_running {
        let mut after: State = state.clone();
        after.restarter = RPhase::Observed;
        after.observed = state.incarnation;
        push_step(&mut found, "restarter: sees a quiet owner and no hook", after);
    }
    if phase == RPhase::Observed && six {
        let mut after: State = state.clone();
        after.restarter = RPhase::Done;
        after.stop_outstanding = true;
        push_step(&mut found, "restarter: asks the manager to restart the unit", after);
    }
    if phase == RPhase::Idle && eight_like(state.rules) && state.foreign_namespace {
        let mut after: State = state.clone();
        after.restarter = RPhase::Done;
        push_step(&mut found, "L7 precondition: the pid namespaces differ, no restart", after);
    } else if phase == RPhase::Idle && !six && state.owner_alive {
        let mut after: State = state.clone();
        after.restarter = RPhase::Observed;
        after.observed = state.incarnation;
        // The pid read from the file names an unrelated process when the restarter is in another namespace, unless
        // the pidfd comes from the manager (candidate).
        after.target_unrelated = state.foreign_namespace && !candidate;
        push_step(&mut found, "restarter: reads lane/incarnation and opens a pidfd", after);
    }
    if phase == RPhase::Observed && !six {
        if state.rules == Rules::CandidateNoGate {
            let mut after: State = state.clone();
            after.restarter = RPhase::Verified;
            push_step(&mut found, "restarter: skips the gate", after);
        } else if state.gate_shared == 0 && !state.gate_exclusive {
            let mut after: State = state.clone();
            after.restarter = RPhase::Gate;
            after.gate_exclusive = true;
            push_step(&mut found, "restarter: takes the gate exclusively without waiting", after);
        }
    }
    if phase == RPhase::Gate {
        let mut after: State = state.clone();
        if state.incarnation == state.observed {
            after.restarter = RPhase::Verified;
            push_step(&mut found, "restarter: re-reads lane/incarnation and finds it unchanged", after);
        } else {
            after.restarter = RPhase::Done;
            after.gate_exclusive = false;
            push_step(&mut found, "restarter: re-reads lane/incarnation, finds it changed and aborts", after);
        }
    }
    if phase == RPhase::Verified && eight_like(state.rules) {
        let mut after: State = state.clone();
        after.restarter = RPhase::Intent;
        after.intent = state.observed;
        push_step(&mut found, "L7 (4): the restarter creates the restart-intent and fsyncs it", after);
    }
    if (phase == RPhase::Verified && !candidate && state.rules != Rules::Eight) || phase == RPhase::Intent {
        let mut after: State = state.clone();
        after.restarter = RPhase::Signalled;
        if state.target_unrelated {
            after.unrelated_killed = true;
        } else if pidfd_names_live_owner(state) && state.rules == Rules::Nine {
            after.kill_pending = true;
        } else if pidfd_names_live_owner(state) {
            after.term_pending = true;
        }
        let label: &str = if state.rules == Rules::Nine {
            "L7 (5): the restarter ends the owner with SIGKILL through the pidfd"
        } else {
            "restarter: sends the terminate signal through the pidfd"
        };
        push_step(&mut found, label, after);
    }
    if phase == RPhase::Verified && candidate {
        let mut after: State = state.clone();
        after.restarter = RPhase::Signalled;
        if pidfd_names_live_owner(state) {
            after.kill_pending = true;
        }
        push_step(&mut found, "restarter: sends the uncatchable kill through the manager's pidfd", after);
    }
    if phase == RPhase::Signalled && !pidfd_names_live_owner(state) {
        let mut after: State = state.clone();
        after.restarter = RPhase::Done;
        after.gate_exclusive = false;
        push_step(&mut found, "restarter: sees the owner dead and releases the gate", after);
    }
    let alive: bool = phase == RPhase::Observed
        || phase == RPhase::Gate
        || phase == RPhase::Verified
        || phase == RPhase::Intent
        || phase == RPhase::Signalled;
    if (phase == RPhase::Dead || phase == RPhase::Done) && !six {
        // A client that finds the owner still quiet starts a new restarter.
        let mut after: State = state.clone();
        after.restarter = RPhase::Idle;
        push_step(&mut found, "restarter: a new client starts", after);
    }
    if alive && !six {
        // The kernel releases the advisory lock when the restarter's last description closes; a signal already
        // sent stays pending.
        let mut after: State = state.clone();
        after.restarter = RPhase::Dead;
        after.gate_exclusive = false;
        push_step(&mut found, "environment: the restarter dies", after);
    }
    return found;
}

/// What:
///  The safety checks: no running person's program is ended, no owner of another incarnation is ended, no unrelated
///  process is killed.
/// Why:
///  These are the promises "the person's hook is never ended" and "a delayed request cannot reach a replacement".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    if state.program_ended {
        return Some("a running person's program was ended by a restart".to_string());
    }
    if state.wrong_owner_ended {
        return Some("a restart request ended an owner of an incarnation it did not observe".to_string());
    }
    if state.unrelated_killed {
        return Some("a process in another PID namespace was killed through a misread pid".to_string());
    }
    return None;
}

/// The liveness check: a wedged owner that no restarter can act on, with no program of the person's running.
pub fn end_state(state: &State) -> Option<String> {
    // Revision 8, owner question B: an owner wedged while it holds the gate is not restarted automatically.
    if eight_like(state.rules) && state.owner_wedged && state.gate_shared != 0 {
        return None;
    }
    if state.owner_alive && state.owner_wedged && !state.hook_running {
        return Some("a wedged owner with no running program of the person's can never be restarted".to_string());
    }
    return None;
}

/// The caps: a few thousand states at most.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 64 };
}

/// What:
///  Explores every schedule under `rules`, for a restarter in the owner's PID namespace and in another one.
/// Why:
///  One call covers the review's schedules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function check(rules: Rules): Report;
/// ```
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules, false), initial(rules, true)], steps, invariant, end_state, limits());
}

/// The goal of the liveness check: the owner is not wedged, is dead, or is wedged holding the gate (the accepted B-hand narrowing).
pub fn owner_is_not_stuck(state: &State) -> bool {
    return !state.owner_alive || !state.owner_wedged || state.gate_shared != 0;
}

/// Explores `rules` for the general liveness property: from every state the protocol's own steps can reach a state where the owner is
/// not stuck. Quiescence alone misses a cycle of own steps (a restarter that starts again and again).
pub fn check_eventually(rules: Rules) -> Report {
    return crate::model_explore::explore_eventually(vec![initial(rules, false), initial(rules, true)], steps, owner_is_not_stuck, limits());
}
