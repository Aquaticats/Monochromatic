//! Historical control, not approval evidence for revision 13: owner death and protected-child death are one transition and the incarnation read is not retained.
//!
//! What:
//!  Model of finding F2 (sixth review) and its seventh-review follow-ups F2A and F2B: the lifetime of an owner of a
//!  worktree's lane, its descendants, and the cleanup a new owner does.
//! Why:
//!  "One owner per worktree" (`doc/planning/cli-git-commits-never-fail.md`) relies on a lane lock that dies with the
//!  owner and on a new owner that reaps the previous incarnation's scope before it mutates anything.
//!  The model splits fork, descriptor closure, parent-death setup and exec into separate steps, counts who holds the
//!  lock's open file description, makes the cgroup kill asynchronous, and lets a replacement service owner share the
//!  unit's cgroup path.
//!  No hook, descendant or owner is assumed to finish: the descendants have no exit step.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; ownerAlive; recorded; child; childHoldsLock; daemonAlive; killPending; n: Phase; service };
//! ```

// What:     imports the explorer's types and the two helpers this model uses; `Limits` caps a search, `Report` is
//           its result, `explore` runs it, `push_step` adds an enabled step to a list.
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
/// type Rules = 'six' | 'seven' | 'candidate' | 'eight' | 'candidate-no-wait';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: children keep the lock description; the new owner acquires first and reaps while it mutates.
    Six,
    /// Revision 7: close-on-exec, parent-death signal, incarnation recorded before launch, the new owner acquires
    /// first, reaps the recorded scope by pathname, waits for it to be empty, then mutates.
    Seven,
    /// A candidate: the new owner reads the incarnation record without the lock, confirms the old owner dead through
    /// a pidfd, kills the old invocation's own scope (never a scope it is in), waits for it to be empty, then acquires.
    Candidate,
    /// Revision 8 draft, L1 to L5: acquire first, reap the cgroup whose name contains the previous incarnation's identifier
    /// (each incarnation has its own, so a replacement never targets itself), then incarnate and mutate. The draft states that
    /// L1 waits for a child between fork and exec.
    Eight,
    /// The candidate without the wait for the scope to report empty (a mutation that must stay red).
    CandidateNoWait,
}

/// What:
///  `enum ChildPhase { ... }`: the direct child of the owner (a program the owner launches for a hook), step by step.
/// Why:
///  The fork-to-exec window is the seventh review's F2A: the child holds the lock description between fork and exec.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildPhase = 'absent' | 'forked' | 'armed' | 'execd' | 'gone';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChildPhase {
    /// Not started.
    Absent,
    /// Forked: holds a copy of the owner's descriptors, parent-death signal not set.
    Forked,
    /// The parent-death signal is set; the descriptors are still open.
    Armed,
    /// Executed: close-on-exec closed the lock description (revisions 7 and the candidate).
    Execd,
    /// Dead.
    Gone,
}

/// What:
///  `enum NewOwner { ... }`: how far the new owner got.
/// Why:
///  The three rules order the same steps differently, so the phase names the step done last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NewOwner = 'waiting' | 'read' | 'confirmed' | 'killing' | 'empty' | 'acquired' | 'mutating' | 'dead';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NewOwner {
    /// Not yet past its first step.
    Waiting,
    /// Read `lane/incarnation`.
    Read,
    /// Confirmed the previous owner dead through a pidfd (candidate).
    Confirmed,
    /// Wrote `cgroup.kill`; the kernel is still delivering it.
    Killing,
    /// The scope reported no processes (or none needed reaping).
    Empty,
    /// Holds the lane lock.
    Acquired,
    /// Mutating the lane's state.
    Mutating,
    /// Killed by the cleanup it ran itself.
    Dead,
}

/// What:
///  `struct State { ... }`: every fact the schedule depends on, as plain `bool`s and small enums.
/// Why:
///  A flat record of `Copy` values hashes cheaply, and each field is one fact the review's schedules turn on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; service: boolean; ownerAlive: boolean; recorded: boolean; child: ChildPhase; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// True when the owners are the service: the replacement is started by the manager in the same cgroup path.
    pub service: bool,
    /// The previous owner process lives.
    pub owner_alive: bool,
    /// `lane/incarnation` with the scope is written and fsynced.
    pub recorded: bool,
    /// The previous owner's child, step by step.
    pub child: ChildPhase,
    /// The child is stopped or starved by something outside the wrapper and takes no further step of its own
    /// (the model assumes no hook or child finishes or runs when it is needed).
    pub child_frozen: bool,
    /// Revision 6 only: the hook closed its copy of the lock description by itself.
    pub child_closed: bool,
    /// A daemon the child started: it closed the lock description and has no parent-death signal.
    pub daemon_alive: bool,
    /// `cgroup.kill` was written and not every member has died yet.
    pub kill_pending: bool,
    /// The new owner.
    pub new_owner: NewOwner,
}

/// What:
///  The start state: the previous owner is alive, nothing recorded or launched, the new owner waiting.
/// Why:
///  `service` picks whether the replacement would share the old owner's cgroup path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initial(rules: Rules, service: boolean): State;
/// ```
pub fn initial(rules: Rules, service: bool) -> State {
    return State {
        rules,
        service,
        owner_alive: true,
        recorded: false,
        child: ChildPhase::Absent,
        child_frozen: false,
        child_closed: false,
        daemon_alive: false,
        kill_pending: false,
        new_owner: NewOwner::Waiting,
    };
}

/// What:
///  Whether the lane lock's open file description has no holder: the previous owner is dead and its child holds no copy.
/// Why:
///  An advisory lock on an open file description lasts until every process holding a copy of the description closes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockFree(s: State): boolean;
/// ```
fn lock_free(state: &State) -> bool {
    return !state.owner_alive && !child_holds_lock(state);
}

/// Whether the child still holds a copy of the lock description.
fn child_holds_lock(state: &State) -> bool {
    if state.child == ChildPhase::Forked || state.child == ChildPhase::Armed {
        return true;
    }
    // Revision 6 never closes the description at exec; only the hook itself can.
    return state.child == ChildPhase::Execd && state.rules == Rules::Six && !state.child_closed;
}

/// Whether the child process lives.
fn child_alive(state: &State) -> bool {
    return state.child != ChildPhase::Absent && state.child != ChildPhase::Gone;
}

/// Whether the new owner is a member of the scope the cleanup kills.
fn new_owner_in_killed_scope(state: &State) -> bool {
    // The replacement service owner runs in the unit's cgroup path, which the recorded name still names; the
    // candidate kills the old invocation's own scope instead.
    return state.service && state.rules != Rules::Candidate && state.rules != Rules::CandidateNoWait && state.rules != Rules::Eight;
}

/// What:
///  Every step enabled in `state`: the previous owner's, its child's, the kernel's and the new owner's.
/// Why:
///  The explorer takes them in every order, which includes the previous owner dying between any two steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let six: bool = state.rules == Rules::Six;
    // The previous owner's steps.
    if state.owner_alive && !state.recorded {
        let mut after: State = state.clone();
        after.recorded = true;
        push_step(&mut found, "owner: writes lane/incarnation and fsyncs it", after);
    }
    if state.owner_alive && state.child == ChildPhase::Absent && (six || state.recorded) {
        let mut after: State = state.clone();
        after.child = ChildPhase::Forked;
        push_step(&mut found, "owner: forks the hook child, which holds the lock description", after);
    }
    if state.owner_alive {
        let mut after: State = state.clone();
        after.owner_alive = false;
        // The parent-death signal ends a child that had set it; a child before that, or a daemon, is not ended.
        if !six && (state.child == ChildPhase::Armed || state.child == ChildPhase::Execd) {
            after.child = ChildPhase::Gone;
        }
        push_step(&mut found, "owner: dies", after);
    }
    // The child's steps; a frozen child takes none.
    if child_alive(state) && !state.child_frozen {
        let mut after: State = state.clone();
        after.child_frozen = true;
        push_step(&mut found, "environment: the child is stopped and never runs again", after);
    }
    if six && state.child == ChildPhase::Execd && !state.child_closed && !state.child_frozen {
        let mut after: State = state.clone();
        after.child_closed = true;
        push_step(&mut found, "hook: closes its copy of the lock description and carries on", after);
    }
    if state.child == ChildPhase::Forked && !six && !state.child_frozen {
        let mut after: State = state.clone();
        after.child = ChildPhase::Armed;
        push_step(&mut found, "child: sets the parent-death signal", after);
    }
    if !state.child_frozen && ((state.child == ChildPhase::Armed) || (state.child == ChildPhase::Forked && six)) {
        let mut after: State = state.clone();
        after.child = ChildPhase::Execd;
        push_step(&mut found, "child: executes the hook (close-on-exec closes the lock description)", after);
    }
    if state.child == ChildPhase::Execd && !state.daemon_alive && !state.child_frozen {
        let mut after: State = state.clone();
        after.daemon_alive = true;
        push_step(&mut found, "hook: leaves a daemon that closed the lock description", after);
    }
    // The kernel's delivery of `cgroup.kill`, one member at a time.
    if state.kill_pending && child_alive(state) {
        let mut after: State = state.clone();
        after.child = ChildPhase::Gone;
        push_step(&mut found, "kernel: delivers the kill to the child", after);
    }
    if state.kill_pending && state.daemon_alive {
        let mut after: State = state.clone();
        after.daemon_alive = false;
        push_step(&mut found, "kernel: delivers the kill to the daemon", after);
    }
    if state.kill_pending && new_owner_in_killed_scope(state) && state.new_owner != NewOwner::Dead {
        let mut after: State = state.clone();
        after.new_owner = NewOwner::Dead;
        push_step(&mut found, "kernel: delivers the kill to the new owner, a member of the same cgroup", after);
    }
    found.extend(new_owner_steps(state));
    return found;
}

/// What:
///  The new owner's enabled steps under the rule the state names.
/// Why:
///  The rules differ in the order of acquire, read, confirm, kill, wait and mutate; this is where they are written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newOwnerSteps(s: State): [string, State][];
/// ```
fn new_owner_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let candidate: bool = state.rules == Rules::Candidate || state.rules == Rules::CandidateNoWait;
    let phase: NewOwner = state.new_owner;
    let members_left: bool = child_alive(state) || state.daemon_alive || (new_owner_in_killed_scope(state) && phase != NewOwner::Dead);
    if !candidate && phase == NewOwner::Waiting && lock_free(state) {
        let mut after: State = state.clone();
        after.new_owner = NewOwner::Acquired;
        push_step(&mut found, "new owner: acquires the lane lock", after);
    }
    if !candidate && phase == NewOwner::Acquired {
        let mut after: State = state.clone();
        // With no incarnation record nothing was ever launched, so there is nothing to reap.
        after.new_owner = if state.recorded { NewOwner::Read } else { NewOwner::Empty };
        push_step(&mut found, "new owner: reads lane/incarnation", after);
    }
    if candidate && phase == NewOwner::Waiting {
        let mut after: State = state.clone();
        after.new_owner = NewOwner::Read;
        push_step(&mut found, "new owner: reads lane/incarnation without the lock", after);
    }
    if candidate && phase == NewOwner::Read && !state.owner_alive {
        let mut after: State = state.clone();
        after.new_owner = if state.recorded { NewOwner::Confirmed } else { NewOwner::Empty };
        push_step(&mut found, "new owner: the pidfd of the recorded owner reports it dead", after);
    }
    if (phase == NewOwner::Read && !candidate) || phase == NewOwner::Confirmed {
        let mut after: State = state.clone();
        after.new_owner = NewOwner::Killing;
        after.kill_pending = true;
        push_step(&mut found, "new owner: writes cgroup.kill on the recorded scope", after);
    }
    if phase == NewOwner::Killing && (!members_left || state.rules == Rules::CandidateNoWait) {
        let mut after: State = state.clone();
        after.new_owner = NewOwner::Empty;
        // The kill, once written, keeps being delivered; it is finished only when no member is left.
        after.kill_pending = members_left;
        push_step(&mut found, "new owner: the scope reports no processes", after);
    }
    if candidate && phase == NewOwner::Empty && lock_free(state) {
        let mut after: State = state.clone();
        after.new_owner = NewOwner::Acquired;
        push_step(&mut found, "new owner: acquires the lane lock", after);
    }
    let may_mutate_early: bool = state.rules == Rules::Six
        && (phase == NewOwner::Acquired || phase == NewOwner::Read || phase == NewOwner::Killing || phase == NewOwner::Empty);
    let may_mutate: bool = ((state.rules == Rules::Seven || state.rules == Rules::Eight) && phase == NewOwner::Empty)
        || (candidate && phase == NewOwner::Acquired);
    if may_mutate_early || may_mutate {
        let mut after: State = state.clone();
        after.new_owner = NewOwner::Mutating;
        push_step(&mut found, "new owner: begins to mutate the lane", after);
    }
    return found;
}

/// What:
///  The safety checks: the new owner never mutates beside a live descendant of the previous incarnation, is never
///  killed by its own cleanup, and no live owner's scope is targeted.
/// Why:
///  These are the properties "the lock proves no obsolete mutator exists" and "never target a live owner" claim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    if state.new_owner == NewOwner::Dead {
        return Some("the replacement owner was killed by the cleanup it ran".to_string());
    }
    if state.new_owner == NewOwner::Mutating && (child_alive(state) || state.daemon_alive) {
        return Some("the new owner mutates while a descendant of the previous incarnation lives".to_string());
    }
    if state.kill_pending && state.owner_alive {
        return Some("a live owner's scope was targeted for the kill".to_string());
    }
    return None;
}

/// The liveness check: a state with no enabled step must have a new owner that is mutating.
pub fn end_state(state: &State) -> Option<String> {
    // Revision 8, L1: the wait for a child between fork and exec is a stated narrowing, bounded only by that child's progress.
    let before_exec: bool = state.child == ChildPhase::Forked || state.child == ChildPhase::Armed;
    if state.rules == Rules::Eight && state.new_owner == NewOwner::Waiting && before_exec {
        return None;
    }
    if state.new_owner != NewOwner::Mutating {
        return Some("the new owner can never proceed: the lane lock or the cleanup is unreachable".to_string());
    }
    return None;
}

/// The caps: a few thousand states at most.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 64 };
}

/// What:
///  Explores every schedule under `rules` for both the inline owner (`service` false) and the service (`service` true).
/// Why:
///  One call covers the review's schedules for a client-started owner and a manager-started one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function check(rules: Rules): Report;
/// ```
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules, false), initial(rules, true)], steps, invariant, end_state, limits());
}
