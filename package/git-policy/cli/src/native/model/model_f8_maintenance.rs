//! Historical control, not approval evidence for revision 13: revision ten supplies the root by rule selection and the final check, ledger append and release are one step.
//!
//! What:
//!  Model of finding F8 (sixth review) and its seventh-review follow-ups: destructive maintenance the wrapper starts in
//!  one worktree while another worktree of the same object store accepts, publishes and releases a request's spool.
//! Why:
//!  "Retention lifecycle and the spool closure" (`doc/planning/cli-git-commits-never-fail.md`) releases a spool only
//!  when no maintenance can still delete the request's objects.
//!  The model has a registry entry with reserved, running and terminal states, a scope the manager creates and
//!  populates in separate steps, a process group a descendant can leave, a launcher that can die, a prune that takes
//!  its reachability view early and deletes later, and the second worktree's lane.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; manager; entry; scope; launcher; running; escaped; view; deleted; b; lost };
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
/// type Rules = 'six' | 'seven' | 'candidate' | 'eight' | 'nine' | 'ten' | 'candidate-observes-empty';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: each lane records maintenance in its own `lane/maintenance`; another lane does not look.
    Six,
    /// Revision 7: one registry in the common directory; an entry whose scope (or process group) is empty or absent is
    /// closed with an observation, and a closed entry no longer holds the spool.
    Seven,
    /// A candidate: the entry moves reserved, running, terminal by atomic renames; a lane revokes a reserved entry
    /// before it treats the launch as impossible; a scope is populated by the trampoline before the entry is
    /// running; without a manager the launcher is a subreaper and writes terminal only when it has no descendant left.
    Candidate,
    /// Revision 8 draft, G1 to G4: the launcher creates the scope and sits in it before the entry is written; a lane records
    /// terminal only when the scope reports no processes and the launcher is dead (or recorded its end); without a manager
    /// the run is retained for good.
    Eight,
    /// Revision 9, G1 to G4 as revision 8, and T1 as a release intent: observe, write the intent, re-read the registry and
    /// withdraw if a run that is not terminal appeared, then re-check and release. The harm is an actual deletion after release.
    Nine,
    /// Revision 10, T1 and C5: as revision 9, and the request's hold root exists until step 6, so a run's earlier view includes
    /// the closure whatever the branch did meanwhile.
    Ten,
    /// The candidate where a lane also closes a reserved entry whose scope is empty, without revoking (a mutation that
    /// must stay red).
    CandidateObservesEmpty,
}

/// What:
///  `struct State { ... }`: the registry entry, the scope, the launcher, the destructive processes and the second lane.
///  Numeric fields hold small enumerations, documented on each field.
/// Why:
///  Flat `Copy` fields hash cheaply, and each field is one fact the review's schedules turn on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; manager: boolean; entry: number; scope: number; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// A service manager can create scopes (otherwise the fallback applies).
    pub manager: bool,
    /// The registry entry: 0 none, 1 reserved, 2 running, 3 closed by observation, 4 terminal, 5 revoked.
    pub entry: u8,
    /// The scope: 0 absent, 1 created and empty, 2 populated, 3 emptied after it ran.
    pub scope: u8,
    /// The launcher: 0 idle, 1 entry written, 2 scope made, 3 authorized (entry running), 4 launched, 5 finished, 6 dead.
    pub launcher: u8,
    /// The main destructive process lives.
    pub running: bool,
    /// A descendant left the process group (fallback) and still runs.
    pub escaped: bool,
    /// The prune's reachability view: 0 not taken, 1 took it before the request's objects were referenced, 2 after.
    pub view: u8,
    /// The prune deleted the request's objects.
    pub deleted: bool,
    /// The second lane: 0 idle, 1 imported, 2 published, 3 connectivity checked, 4 spool released.
    pub lane_b: u8,
    /// The branch was moved away from the request's commit (a switch or a rewrite) at some point.
    pub branch_elsewhere: bool,
    /// The spool was released while destructive execution could still occur (a registered launch still authorized, or a
    /// destructive process still alive); set by the release step.
    pub lost: bool,
}

/// What:
///  The start state: nothing running, nothing registered.
/// Why:
///  `manager` picks whether a service manager exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initial(rules: Rules, manager: boolean): State;
/// ```
pub fn initial(rules: Rules, manager: bool) -> State {
    return State {
        rules,
        manager,
        entry: 0,
        scope: 0,
        launcher: 0,
        running: false,
        escaped: false,
        view: 0,
        deleted: false,
        lane_b: 0,
        branch_elsewhere: false,
        lost: false,
    };
}

/// Whether some destructive process of the run still lives.
fn destructive_alive(state: &State) -> bool {
    // What:     `|x| expr` is a closure (an unnamed function; TS: `(x) => expr`). It is passed to a method that calls it.
    // Why:      Say in one place what to do for each element or error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (x) => expr
    // ```
    return state.running || state.escaped;
}

/// What:
///  The launcher's enabled steps under the rule the state names.
/// Why:
///  The three rules differ in how a run is registered, contained and ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function launcherSteps(s: State): [string, State][];
/// ```
fn launcher_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let candidate: bool = state.rules == Rules::Candidate || state.rules == Rules::CandidateObservesEmpty;
    let mut after: State = state.clone();
    if state.rules == Rules::Eight || state.rules == Rules::Nine || state.rules == Rules::Ten {
        return eight_launcher_steps(state);
    }
    if state.rules == Rules::Six && state.launcher == 0 {
        after.launcher = 4;
        after.running = true;
        push_step(&mut found, "launcher: records the run in its own lane and starts the destructive process", after);
    } else if state.launcher == 0 {
        after.launcher = 1;
        after.entry = 1;
        push_step(&mut found, "launcher: creates the registry entry with O_EXCL and fsyncs it", after);
    } else if state.launcher == 1 && state.manager {
        after.launcher = 2;
        after.scope = 1;
        push_step(&mut found, "launcher: asks the manager to create the scope", after);
    } else if state.launcher == 1 && !state.manager && !candidate {
        after.launcher = 2;
        push_step(&mut found, "launcher: chooses a new process group as the scope", after);
    } else if state.launcher == 1 && !state.manager && candidate {
        after.launcher = 2;
        push_step(&mut found, "launcher: becomes a subreaper", after);
    } else if state.launcher == 2 && candidate && state.entry == 1 {
        // The trampoline is placed in the scope before the entry says running; moving to running is one atomic rename
        // that fails if a lane revoked the entry.
        after.launcher = 3;
        after.entry = 2;
        if state.manager {
            after.scope = 2;
        }
        push_step(&mut found, "launcher: moves the entry from reserved to running with one rename, the trampoline in the scope", after);
    } else if state.launcher == 2 && candidate && state.entry == 5 {
        after.launcher = 5;
        push_step(&mut found, "launcher: the rename fails because a lane revoked the entry, and it does not launch", after);
    } else if state.launcher == 2 && !candidate {
        after.launcher = 4;
        after.running = true;
        if state.manager {
            after.scope = 2;
        }
        after.entry = 2;
        push_step(&mut found, "launcher: launches the destructive process into the scope", after);
    } else if state.launcher == 3 {
        after.launcher = 4;
        after.running = true;
        push_step(&mut found, "trampoline: executes the destructive process", after);
    }
    let waited: bool = !state.running && (!candidate || !state.escaped);
    if state.launcher == 4 && waited && state.rules != Rules::Six {
        let mut done: State = state.clone();
        done.launcher = 5;
        done.entry = 4;
        push_step(&mut found, "launcher: waits for the process, finds no descendant left and writes terminal", done);
    }
    if state.launcher >= 1 && state.launcher <= 4 && state.rules != Rules::Six {
        let mut died: State = state.clone();
        died.launcher = 6;
        push_step(&mut found, "environment: the launcher dies", died);
    }
    return found;
}

/// What:
///  The launcher's steps under the revision 8 draft: G1 reserve (scope first with the launcher in it, then the entry), G2 run,
///  G4 without a manager, the end record, and the launcher's death.
/// Why:
///  The launcher inside the scope is the launch authority, so an empty scope with a live launcher never means finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function eightLauncherSteps(s: State): [string, State][];
/// ```
fn eight_launcher_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let mut after: State = state.clone();
    if state.launcher == 0 && state.manager {
        after.launcher = 1;
        after.scope = 2;
        push_step(&mut found, "G1 reserve: the launcher creates the scope and places itself in it", after);
    } else if state.launcher == 0 {
        // G4: no manager, the run starts and is registered as one that cannot be contained.
        after.launcher = 4;
        after.running = true;
        after.entry = 2;
        push_step(&mut found, "G4 no manager: the launcher starts the run and registers it as uncontained", after);
    } else if state.launcher == 1 {
        after.launcher = 2;
        after.entry = 1;
        push_step(&mut found, "G1 reserve: the launcher writes the entry reserved by link", after);
    } else if state.launcher == 2 {
        after.launcher = 4;
        after.running = true;
        after.entry = 2;
        push_step(&mut found, "G2 run: the launcher starts the destructive process from inside the scope", after);
    }
    if state.launcher == 4 && !state.running && state.manager {
        let mut done: State = state.clone();
        done.launcher = 5;
        done.scope = 3;
        push_step(&mut found, "G3 terminal: the launcher waits for the process and records end", done);
    }
    if state.launcher >= 1 && state.launcher <= 4 {
        let mut died: State = state.clone();
        died.launcher = 6;
        if !state.running {
            died.scope = 3;
        }
        push_step(&mut found, "environment: the launcher dies", died);
    }
    return found;
}

/// What:
///  The second lane's closing and release steps under the revision 8 draft.
/// Why:
///  G3: terminal needs the scope empty and the launcher dead (or its end recorded); G4: nothing closes an uncontained run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function eightReleaseSteps(s: State): [string, State][];
/// ```
fn eight_release_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let open: bool = state.entry == 1 || state.entry == 2;
    let scope_empty: bool = state.scope == 3 && !state.running;
    let ended: bool = state.launcher == 6 || state.launcher == 5;
    if open && state.manager && scope_empty && ended {
        let mut after: State = state.clone();
        after.entry = 4;
        push_step(&mut found, "G3 terminal: lane B finds the scope empty and the launcher dead and records terminal", after);
    }
    if state.rules == Rules::Eight && state.lane_b == 3 && !open {
        let mut after: State = state.clone();
        after.lane_b = 6;
        push_step(&mut found, "T1 pre: lane B observes no open run and no uncompleted entry", after);
    }
    if state.rules == Rules::Eight && state.lane_b == 6 {
        let mut after: State = state.clone();
        after.lane_b = 4;
        let launch_possible: bool = state.launcher >= 1 && state.launcher <= 3 && state.entry >= 1;
        after.lost = destructive_alive(state) || launch_possible;
        push_step(&mut found, "T1 retire: lane B releases the spool", after);
    }
    let nine_like: bool = state.rules == Rules::Nine || state.rules == Rules::Ten;
    if nine_like && state.lane_b >= 2 && state.lane_b != 4 && !state.branch_elsewhere {
        let mut after: State = state.clone();
        after.branch_elsewhere = true;
        push_step(&mut found, "person: moves the branch away from the request's commit", after);
    }
    if nine_like && state.branch_elsewhere {
        let mut after: State = state.clone();
        after.branch_elsewhere = false;
        push_step(&mut found, "person: returns the branch to the request's commit", after);
    }
    if nine_like && state.lane_b == 3 && !open {
        let mut after: State = state.clone();
        after.lane_b = 5;
        push_step(&mut found, "T1 (1): lane B writes the release intent", after);
    }
    if nine_like && state.lane_b == 5 && open {
        let mut after: State = state.clone();
        after.lane_b = 3;
        push_step(&mut found, "T1 (2): lane B re-reads the registry, finds a run that is not terminal and withdraws the intent", after);
    }
    if nine_like && state.lane_b == 5 && !open {
        let mut after: State = state.clone();
        after.lane_b = 7;
        push_step(&mut found, "T1 (2): lane B re-reads the registry and finds no run that is not terminal", after);
    }
    // Step (3) re-runs the connectivity check from the tip: a commit the tip no longer reaches is not released.
    if nine_like && state.lane_b == 7 && !state.branch_elsewhere {
        let mut after: State = state.clone();
        if state.deleted {
            after.lane_b = 1;
            after.deleted = false;
            push_step(&mut found, "T1 (3): the connectivity check finds an object missing, the intent is withdrawn and the object imported again", after);
        } else {
            after.lane_b = 4;
            push_step(&mut found, "T1 (3) to (5): lane B re-runs the check, appends the ledger line and releases the spool", after);
        }
    }
    return found;
}

/// What:
///  The destructive process's enabled steps and the second lane's.
/// Why:
///  The prune takes its view early and deletes later; the second lane imports, publishes, checks and releases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function workSteps(s: State): [string, State][];
/// ```
fn work_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    if destructive_alive(state) && state.view == 0 {
        let mut after: State = state.clone();
        // The closure is referenced when the request is published and either the branch holds it or a hold root does (revision 10,
        // until T1 step 6).
        let rooted: bool = state.rules == Rules::Ten && state.lane_b != 4;
        after.view = if state.lane_b >= 2 && (rooted || !state.branch_elsewhere) { 2 } else { 1 };
        push_step(&mut found, "prune: takes its reachability view", after);
    }
    if destructive_alive(state) && state.view == 1 && !state.deleted {
        let mut after: State = state.clone();
        after.deleted = true;
        push_step(&mut found, "prune: deletes the objects its view found unreachable", after);
    }
    if state.running {
        let mut after: State = state.clone();
        after.running = false;
        if state.scope == 2 {
            after.scope = 3;
        }
        push_step(&mut found, "prune: the main process ends", after);
    }
    if state.running && !state.manager && !state.escaped && state.rules != Rules::Six {
        let mut after: State = state.clone();
        after.escaped = true;
        push_step(&mut found, "prune: a descendant calls setsid and leaves the process group", after);
    }
    if state.escaped {
        let mut after: State = state.clone();
        after.escaped = false;
        push_step(&mut found, "prune: the escaped descendant ends", after);
    }
    if state.lane_b == 0 {
        let mut after: State = state.clone();
        after.lane_b = 1;
        push_step(&mut found, "lane B: accepts the request and imports its objects", after);
    }
    if state.lane_b == 1 {
        let mut after: State = state.clone();
        after.lane_b = 2;
        push_step(&mut found, "lane B: publishes the request", after);
    }
    // After the run's end a connectivity check must pass before the spool is released: while an entry is open the
    // check is not yet the final one (revision 6 has no entries to wait for).
    let entry_open: bool = state.entry == 1 || state.entry == 2;
    if state.lane_b == 2 && (state.rules == Rules::Six || !entry_open) {
        let mut after: State = state.clone();
        if state.deleted {
            after.lane_b = 1;
            after.deleted = false;
            push_step(&mut found, "lane B: the connectivity check finds an object missing and imports it again", after);
        } else {
            after.lane_b = 3;
            push_step(&mut found, "lane B: the connectivity check passes", after);
        }
    }
    return found;
}

/// What:
///  Whether lane B may release the spool now, and the closing steps lane B can take on the registry.
/// Why:
///  This is where the release rule of each revision is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function releaseSteps(s: State): [string, State][];
/// ```
fn release_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let candidate: bool = state.rules == Rules::Candidate || state.rules == Rules::CandidateObservesEmpty;
    if state.rules == Rules::Eight || state.rules == Rules::Nine || state.rules == Rules::Ten {
        return eight_release_steps(state);
    }
    let scope_empty: bool = state.scope == 0 || state.scope == 1 || state.scope == 3;
    let group_empty: bool = !state.running;
    if state.rules == Rules::Seven || state.rules == Rules::CandidateObservesEmpty {
        let observes_empty: bool = if state.manager { scope_empty } else { group_empty };
        let applies: bool = state.entry == 1 || (state.entry == 2 && state.rules == Rules::Seven);
        if applies && observes_empty {
            let mut after: State = state.clone();
            after.entry = 3;
            push_step(&mut found, "lane B: finds the scope empty or absent and closes the entry with an observation", after);
        }
    }
    if candidate && state.entry == 1 {
        let mut after: State = state.clone();
        after.entry = 5;
        push_step(&mut found, "lane B: revokes the reserved entry with one rename", after);
    }
    if candidate && state.entry == 2 && state.manager && state.scope == 3 {
        let mut after: State = state.clone();
        after.entry = 4;
        push_step(&mut found, "lane B: finds the populated scope emptied and records the entry terminal", after);
    }
    if state.lane_b == 3 {
        let open: bool = state.entry == 1 || state.entry == 2;
        let allowed: bool = state.rules == Rules::Six || !open;
        if allowed {
            let mut after: State = state.clone();
            after.lane_b = 4;
            // A launch is still possible when the launcher lives, has registered and nobody revoked its entry.
            let launch_possible: bool = state.launcher >= 1 && state.launcher <= 3 && state.entry >= 1 && state.entry != 5;
            after.lost = destructive_alive(state) || launch_possible;
            push_step(&mut found, "lane B: releases the spool", after);
        }
    }
    return found;
}

/// What:
///  Every step enabled in `state`.
/// Why:
///  The explorer takes them in every order, so lane B observes the registry between any two launcher steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = launcher_steps(state);
    found.extend(work_steps(state));
    found.extend(release_steps(state));
    return found;
}

/// What:
///  The safety checks: no recovery copy is released while destructive execution can still occur, and no objects are
///  deleted after the copy is released.
/// Why:
///  Releasing the only copy, and then losing the objects, is the data loss the retention lifecycle exists to prevent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    if state.lane_b == 4 && state.deleted {
        return Some("a request's recovery copy was released and its objects were then deleted by wrapper-started maintenance".to_string());
    }
    // The review's property: no recovery copy is released while relevant destructive execution can still occur,
    // which includes a launch that is still authorized.
    if state.lost {
        return Some("a recovery copy was released while destructive execution could still occur".to_string());
    }
    return None;
}

/// A state with no enabled step needs no extra check here: retention that never ends costs disk only.
pub fn end_state(_state: &State) -> Option<String> {
    return None;
}

/// The caps: a few thousand states at most.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 64 };
}

/// What:
///  Explores every schedule under `rules`, with and without a manager.
/// Why:
///  One call covers the review's schedules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function check(rules: Rules): Report;
/// ```
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules, true), initial(rules, false)], steps, invariant, end_state, limits());
}
