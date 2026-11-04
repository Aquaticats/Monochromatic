//! Historical control, not approval evidence for revision 13: a retired candidate (cancelling a stalled update-ref): identity check and removal are one transition and spawn-time classification is absent.
//!
//! What:
//!  Model of finding F4 (sixth review) and its seventh-review follow-up: a ref transaction whose Git child stalls is
//!  cancelled, and the branch's `.lock` file is cleaned up.
//! Why:
//!  "Cancelling a stalled Git child" (`doc/planning/cli-git-commits-never-fail.md`) removes a lock "created inside the
//!  child's life".
//!  Creation inside a lifetime is not creation by that child: a foreign writer can create the lock while the child
//!  stalls, and a stat followed by an unlink is two steps, not one.
//!  The model has one lock file with a creator and an inode, a Git child that may stall at any point, a foreign writer,
//!  a foreign removal of a stale lock, and an owner that can die; no child is assumed to finish.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; g: GPhase; lock: Lock; owner: OwnerPhase; ... };
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
/// type Rules = 'six' | 'seven' | 'candidate' | 'eight' | 'nine' | 'candidate-no-journal' | 'candidate-removes-any';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: a stalled `update-ref` is killed (or its owner restarted) and nothing cleans the lock it held.
    Six,
    /// Revision 7: terminate, then kill; remove a lock that was absent before launch and whose birth time lies inside
    /// the child's life, after stat-ing its inode.
    Seven,
    /// A candidate: no Git child; the owner opens an `O_TMPFILE`, journals its inode, links it as `branch.lock`
    /// without replacing, and a recovering owner removes the lock only when it is the journaled inode.
    Candidate,
    /// Revision 8 draft, X1: a terminate signal, a wait for the grace, and nothing further; the wrapper never removes a ref lock.
    Eight,
    /// Revision 9, X1: classification is by the kind of command and `update-ref` is exempt, whatever hook is configured now
    /// or installed later; the wrapper never cancels it and never removes a ref lock.
    Nine,
    /// The candidate without the journal write before the link (a mutation that must stay red).
    CandidateNoJournal,
    /// The candidate whose recovery removes whatever lock is present (a mutation that must stay red).
    CandidateRemovesAny,
}

/// What:
///  `enum GPhase { ... }`: the Git child of the transaction.
/// Why:
///  The child can stall before it takes the lock, while it holds it, or after it released it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GPhase = 'absent' | 'launched' | 'locked' | 'released' | 'gone';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GPhase {
    /// Not started.
    Absent,
    /// Started, has not created the lock.
    Launched,
    /// Created the lock and holds it.
    Locked,
    /// Finished the update and released the lock, has not exited.
    Released,
    /// Dead.
    Gone,
}

/// What:
///  `enum OwnerPhase { ... }`: the wrapper's owner, for the transaction.
/// Why:
///  Cancelling is a short script of steps with a death possible between any two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnerPhase = 'idle' | 'journaled' | 'running' | 'term' | 'killed' | 'checked' | 'dead' | 'done';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OwnerPhase {
    /// Nothing started.
    Idle,
    /// Candidate: the tmpfile inode is journaled.
    Journaled,
    /// The transaction runs (a child is launched, or the owner holds the lock itself).
    Running,
    /// Sent the terminate signal.
    Term,
    /// Sent the kill; the child is gone.
    Killed,
    /// Stat-ed the lock and decided to remove it.
    Checked,
    /// The owner process died.
    Dead,
    /// Finished, whatever the outcome.
    Done,
}

/// What:
///  `struct Lock { ... }`: the `branch.lock` path: whether a file is there, who created it (0 nobody, 1 the Git child,
///  2 a foreign writer, 3 the wrapper's own link), its inode and whether it was born inside the Git child's life.
/// Why:
///  The seventh review's counterexamples are exactly the cases where creation inside the child's life is not creation
///  by the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Lock = { present: boolean; creator: number; inode: number; bornInChildLife: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Lock {
    /// A file is at `branch.lock`.
    pub present: bool,
    /// Who created the file there.
    pub creator: u8,
    /// A label of the file, distinct for every creation.
    pub inode: u8,
    /// The creation fell between the child's start and its death.
    pub born_in_child_life: bool,
}

/// What:
///  `struct State { ... }`: the facts the review's schedules turn on.
/// Why:
///  Flat `Copy` fields hash cheaply; the harm flags record a harm once it happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; birthTime: boolean; g: GPhase; lock: Lock; owner: OwnerPhase; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The file system reports a birth time (statx), or not.
    pub birth_time: bool,
    /// The Git child.
    pub child: GPhase,
    /// The terminate signal was sent and the child has not yet died.
    pub term_sent: bool,
    /// The lock path.
    pub lock: Lock,
    /// The next inode label to hand out.
    pub next_inode: u8,
    /// The owner.
    pub owner: OwnerPhase,
    /// The owner observed `branch.lock` absent when it launched the child.
    pub absent_before_launch: bool,
    /// The inode the owner stat-ed before it unlinks.
    pub checked_inode: u8,
    /// Candidate: the journaled inode of the owner's own tmpfile (0 for none).
    pub journal_inode: u8,
    /// How many locks the foreign writer has created (at most two, to bound the search).
    pub foreign_creations: u8,
    /// Harm: a lock the wrapper did not create was removed by the wrapper.
    pub foreign_removed: bool,
    /// A person installed a `reference-transaction` hook (at any time, even after the child was launched).
    pub hook_installed: bool,
    /// Git dispatched the hook and it is running.
    pub hook_running: bool,
    /// Harm: a cancellation ended a running person's hook.
    pub hook_ended: bool,
}

/// What:
///  The start state: no lock, nothing running.
/// Why:
///  `birth_time` picks whether the file system reports a birth time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initial(rules: Rules, birthTime: boolean): State;
/// ```
pub fn initial(rules: Rules, birth_time: bool) -> State {
    return State {
        rules,
        birth_time,
        child: GPhase::Absent,
        term_sent: false,
        lock: Lock { present: false, creator: 0, inode: 0, born_in_child_life: false },
        next_inode: 1,
        owner: OwnerPhase::Idle,
        absent_before_launch: false,
        checked_inode: 0,
        journal_inode: 0,
        foreign_creations: 0,
        foreign_removed: false,
        hook_installed: false,
        hook_running: false,
        hook_ended: false,
    };
}

/// Whether the Git child lives.
fn child_alive(state: &State) -> bool {
    return state.child != GPhase::Absent && state.child != GPhase::Gone;
}

/// Whether the rule is one of the candidate's.
fn is_candidate(rules: Rules) -> bool {
    // What:     `|x| expr` is a closure (an unnamed function; TS: `(x) => expr`). It is passed to a method that calls it.
    // Why:      Say in one place what to do for each element or error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (x) => expr
    // ```
    return rules == Rules::Candidate || rules == Rules::CandidateNoJournal || rules == Rules::CandidateRemovesAny;
}

/// What:
///  Every step enabled in `state`.
/// Why:
///  The explorer takes them in every order, so the foreign writer creates the lock at every point of the child's life
///  and the owner dies between any two steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    // The foreign writer: creates the lock when it is free (twice at most) and releases a lock it holds.
    if !state.lock.present && state.foreign_creations < 2 {
        let mut after: State = state.clone();
        after.lock = Lock { present: true, creator: 2, inode: state.next_inode, born_in_child_life: child_alive(state) };
        after.next_inode += 1;
        after.foreign_creations += 1;
        push_step(&mut found, "foreign: creates branch.lock", after);
    }
    if state.lock.present && state.lock.creator == 2 {
        let mut after: State = state.clone();
        after.lock.present = false;
        push_step(&mut found, "foreign: finishes and releases branch.lock", after);
    }
    // A person removing a stale lock by hand: outside the wrapper's control, and the race the unlink must survive.
    if state.lock.present && state.lock.creator != 2 && !is_candidate(state.rules) {
        let mut after: State = state.clone();
        after.lock.present = false;
        push_step(&mut found, "outside: a person removes the stale branch.lock by hand", after);
    }
    if is_candidate(state.rules) {
        found.extend(embedded_steps(state));
    } else {
        found.extend(git_steps(state));
    }
    return found;
}

/// What:
///  The steps of revisions 6 and 7: the owner launches a Git child and cancels it when it stalls.
/// Why:
///  The child's steps are all optional, so a child that never takes another step is a schedule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitSteps(s: State): [string, State][];
/// ```
fn git_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let seven: bool = state.rules == Rules::Seven;
    if state.owner == OwnerPhase::Idle && !state.lock.present {
        let mut after: State = state.clone();
        after.owner = OwnerPhase::Running;
        after.child = GPhase::Launched;
        after.absent_before_launch = true;
        push_step(&mut found, "owner: observes branch.lock absent and launches update-ref", after);
    }
    if state.child == GPhase::Launched && !state.lock.present {
        let mut after: State = state.clone();
        after.child = GPhase::Locked;
        after.lock = Lock { present: true, creator: 1, inode: state.next_inode, born_in_child_life: true };
        after.next_inode += 1;
        push_step(&mut found, "git: creates branch.lock", after);
    }
    if state.child == GPhase::Locked {
        let mut after: State = state.clone();
        after.child = GPhase::Released;
        after.lock.present = false;
        push_step(&mut found, "git: finishes the update and releases branch.lock", after);
    }
    if state.child == GPhase::Released {
        let mut after: State = state.clone();
        after.child = GPhase::Gone;
        push_step(&mut found, "git: exits", after);
    }
    if state.term_sent && child_alive(state) {
        // A child that handles the signal removes its own lock and exits.
        let mut after: State = state.clone();
        if state.child == GPhase::Locked {
            after.lock.present = false;
        }
        after.child = GPhase::Gone;
        after.term_sent = false;
        push_step(&mut found, "git: handles the terminate signal, removes its own lock and exits", after);
    }
    let hooks_possible: bool = state.rules == Rules::Eight || state.rules == Rules::Nine;
    if hooks_possible && !state.hook_installed {
        let mut after: State = state.clone();
        after.hook_installed = true;
        push_step(&mut found, "person: installs a reference-transaction hook", after);
    }
    if hooks_possible && state.child == GPhase::Locked && state.hook_installed && !state.hook_running {
        let mut after: State = state.clone();
        after.hook_running = true;
        push_step(&mut found, "git: dispatches the reference-transaction hook", after);
    }
    if state.hook_running {
        let mut after: State = state.clone();
        after.hook_running = false;
        push_step(&mut found, "hook: returns", after);
    }
    // Revision 8 classifies at spawn time (no hook was configured then); revision 9 exempts `update-ref` by kind.
    if state.owner == OwnerPhase::Running && child_alive(state) && !state.term_sent && (seven || state.rules == Rules::Eight) {
        let mut after: State = state.clone();
        after.owner = OwnerPhase::Term;
        after.term_sent = true;
        if state.hook_running {
            after.hook_ended = true;
        }
        push_step(&mut found, "owner: sends the terminate signal to the stalled child", after);
    }
    let can_kill: bool = child_alive(state) && ((seven && state.owner == OwnerPhase::Term) || (state.rules == Rules::Six && state.owner == OwnerPhase::Running));
    if can_kill {
        let mut after: State = state.clone();
        after.owner = OwnerPhase::Killed;
        after.child = GPhase::Gone;
        after.term_sent = false;
        push_step(&mut found, "owner: kills the stalled child", after);
    }
    if (state.owner == OwnerPhase::Term || state.owner == OwnerPhase::Running) && state.child == GPhase::Gone {
        // The child went away by itself (it handled the signal or finished): the owner is done.
        let mut after: State = state.clone();
        after.owner = if seven { OwnerPhase::Killed } else { OwnerPhase::Done };
        push_step(&mut found, "owner: sees the child gone", after);
    }
    if state.owner == OwnerPhase::Killed && !seven {
        let mut after: State = state.clone();
        after.owner = OwnerPhase::Done;
        push_step(&mut found, "owner: reports the transaction pending", after);
    }
    if seven && state.owner == OwnerPhase::Killed {
        let removable: bool = state.lock.present && state.absent_before_launch && state.birth_time && state.lock.born_in_child_life;
        let mut after: State = state.clone();
        if removable {
            after.owner = OwnerPhase::Checked;
            after.checked_inode = state.lock.inode;
            push_step(&mut found, "owner: stats branch.lock: absent before launch, born inside the child's life", after);
        } else {
            after.owner = OwnerPhase::Done;
            push_step(&mut found, "owner: treats branch.lock as foreign and reports the transaction pending", after);
        }
    }
    if seven && state.owner == OwnerPhase::Checked {
        let mut after: State = state.clone();
        if state.lock.present && state.lock.creator != 1 {
            after.foreign_removed = true;
        }
        after.lock.present = false;
        after.owner = OwnerPhase::Done;
        push_step(&mut found, "owner: unlinks branch.lock", after);
    }
    return found;
}

/// What:
///  The steps of the candidate: the owner takes the lock itself through a journaled tmpfile link.
/// Why:
///  The lock's inode is known and journaled before the name exists, so recovery can tell its own lock from a
///  foreign one by identity, not by time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function embeddedSteps(s: State): [string, State][];
/// ```
fn embedded_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let journals: bool = state.rules != Rules::CandidateNoJournal;
    if state.owner == OwnerPhase::Idle && journals {
        let mut after: State = state.clone();
        after.owner = OwnerPhase::Journaled;
        // The unnamed file exists now and has its inode; the journal records it before the name exists.
        after.journal_inode = state.next_inode;
        after.next_inode += 1;
        push_step(&mut found, "owner: opens an unnamed file and journals its inode (fsync)", after);
    }
    if (state.owner == OwnerPhase::Journaled) || (state.owner == OwnerPhase::Idle && !journals) {
        let mut after: State = state.clone();
        if state.lock.present {
            after.owner = OwnerPhase::Done;
            push_step(&mut found, "owner: the link fails because branch.lock exists, the request becomes pending", after);
        } else {
            after.owner = OwnerPhase::Running;
            // Without the journal the file's inode is only known now, and nobody wrote it down.
            let inode: u8 = if journals { state.journal_inode } else { state.next_inode };
            after.lock = Lock { present: true, creator: 3, inode, born_in_child_life: false };
            if !journals {
                after.next_inode += 1;
            }
            push_step(&mut found, "owner: links the file as branch.lock without replacing", after);
        }
    }
    if state.owner == OwnerPhase::Running {
        let mut after: State = state.clone();
        after.lock.present = false;
        after.owner = OwnerPhase::Done;
        push_step(&mut found, "owner: renames branch.lock over the ref", after);
    }
    if state.owner == OwnerPhase::Idle || state.owner == OwnerPhase::Journaled || state.owner == OwnerPhase::Running {
        let mut after: State = state.clone();
        after.owner = OwnerPhase::Dead;
        push_step(&mut found, "environment: the owner dies", after);
    }
    if state.owner == OwnerPhase::Dead {
        let mut after: State = state.clone();
        after.owner = OwnerPhase::Done;
        let ours: bool = state.lock.present && state.lock.creator == 3 && state.lock.inode == state.journal_inode && state.journal_inode != 0;
        if ours || (state.rules == Rules::CandidateRemovesAny && state.lock.present) {
            if state.lock.creator != 3 {
                after.foreign_removed = true;
            }
            after.lock.present = false;
            push_step(&mut found, "owner: a new owner finds the journaled inode at branch.lock and removes it", after);
        } else {
            push_step(&mut found, "owner: a new owner finds no journaled lock to remove", after);
        }
    }
    return found;
}

/// What:
///  The safety checks: the wrapper never removes a lock it did not create, and a finished transaction leaves no lock
///  the wrapper itself made.
/// Why:
///  These are "no foreign lock is removed" and "the wrapper's own cancellation does not create a permanent pending state".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    if state.hook_ended {
        return Some("a cancellation ended a running person's hook".to_string());
    }
    if state.foreign_removed {
        return Some("a lock the wrapper did not create was removed".to_string());
    }
    let own_creator: bool = state.lock.creator == 1 || state.lock.creator == 3;
    if state.owner == OwnerPhase::Done && state.lock.present && own_creator && !child_alive(state) {
        return Some("the wrapper's own cancellation left a lock that nothing will ever remove".to_string());
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

/// A state with no enabled step needs no extra check here: the harms are safety properties.
pub fn end_state(_state: &State) -> Option<String> {
    return None;
}

/// The caps: a few thousand states at most.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 64 };
}

/// What:
///  Explores every schedule under `rules`, with and without a file system birth time.
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
