//! What:
//!  Model of rule V's barriers at the filesystem's level of detail: a scenario is a sequence of writes (each makes one fact
//!  visible), write-back that can persist any visible fact in any order, one power loss, and a barrier that makes a stated set
//!  durable before the step that needs it. The scenarios are the departure barrier of every saving exit (W1 withdrawn, the
//!  unresolved COMMITTING exit, `conflicted`, `rewritten-away`, W1 finding a terminal outcome), the reclamation barrier (T1's Pre),
//!  and the preservation barrier (a staged-only version saved before the index entry that held it is replaced), each with the
//!  content of a file and its directory entry as separate facts.
//! Why:
//!  Revision 12 removed the syncs from the waiting path and kept no barrier where a promise is made (the twelfth review, R12-5):
//!  a command answered "saved" with nothing durable, T1 released a request on a ledger line that was only visible, and MG3 replaced
//!  the index entry that was the only copy of a staged-only version before the saved file was durable. Revision 13 names the D-set
//!  of each exit and puts the barrier before the answer, before the release, and before the replacement. The model checks the
//!  promise after the power loss, using only implementable evidence: a fact counts only when it is durable and every fact it
//!  names (a log record names its saved item, a ledger line names the removal record) is durable too, which is what the
//!  hash-chained files check at recovery. Visibility is never durability.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; scenario; visible: number; durable: number; step: number; answer; crashed; replaced };
//! // const recoverable = (durable: number, fact: number, deps) => (durable & fact) !== 0 && deps[fact].every((d) => recoverable(durable, d, deps));
//! ```

use crate::model_explore::{Limits, Report, explore_bounded_by, push_step};
use crate::model_observe::{World, make_visible, power_loss, sync};

/// The request file `inbox/<id>`.
pub const REQUEST: u32 = 1;
/// The directory entry of the request file in `inbox/`.
pub const INBOX_NAME: u32 = 2;
/// The lane log record that names the saved item.
pub const LOG_RECORD: u32 = 4;
/// The mark file.
pub const MARK: u32 = 8;
/// The directory entry of the mark in `lane/decision/`.
pub const MARK_NAME: u32 = 16;
/// The objects of a built commit.
pub const BUILT: u32 = 32;
/// The terminal ledger line.
pub const LEDGER_LINE: u32 = 64;
/// The ledger's directory entry.
pub const LEDGER_NAME: u32 = 128;
/// The log record that removes the request from the state.
pub const REMOVAL: u32 = 256;
/// The content of the preserving file.
pub const SAVED_CONTENT: u32 = 512;
/// The preserving file's directory entry.
pub const SAVED_NAME: u32 = 1024;
/// The log record that names the preserving file.
pub const SAVED_RECORD: u32 = 2048;
/// The index entry replacement (Git's own write, durable as `core.fsync` makes it).
pub const INDEX_NEW: u32 = 4096;
/// The retained recovery copy of the request (the spool and the request file) was released.
pub const RELEASED: u32 = 8192;
/// Content that only another writer's commit holds and that the exit's named built commit needs (its parent's objects).
pub const PARENT: u32 = 16384;
/// The highest fact, for the loops over facts.
pub const LAST_FACT: u32 = PARENT;

/// What a scenario promises and needs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Scenario {
    /// The register of exits and barriers it comes from.
    pub name: &'static str,
    /// The writes, in order; the last may be the gated step.
    pub writes: &'static [u32],
    /// The D-set of the barrier: facts made durable, in this order, before the gated step.
    pub d_set: &'static [u32],
    /// The facts the promise needs recoverable after a power loss once the gated step happened.
    pub promise: u32,
    /// For each fact, the facts it names (recovery drops a record whose reference does not validate).
    pub deps: &'static [(u32, u32)],
    /// Facts another actor made visible before the command began (a resend that finds the request file already linked).
    pub preexisting: u32,
    /// The facts that carry the stop disposition (the mark and the name of the mark): after any answer, no attempt may start, so these
    /// must survive a power loss whatever the wording of the answer.
    pub stop: u32,
    /// Facts the exit's named content needs beyond the lane's own files (revision 14 forces them with the closure).
    pub closure: u32,
}

/// The scenarios of revision 13.
pub const SCENARIOS: [Scenario; 10] = [
    Scenario {
        name: "W1 withdrawn",
        writes: &[REQUEST, INBOX_NAME, BUILT, LOG_RECORD, MARK, MARK_NAME],
        d_set: &[REQUEST, INBOX_NAME, BUILT, LOG_RECORD, MARK, MARK_NAME],
        promise: REQUEST | INBOX_NAME | BUILT | LOG_RECORD | MARK | MARK_NAME,
        deps: &[(LOG_RECORD, REQUEST | BUILT), (REQUEST, INBOX_NAME), (MARK, MARK_NAME)],
        preexisting: 0,
        stop: MARK | MARK_NAME,
        closure: 0,
    },
    Scenario {
        name: "the unresolved COMMITTING exit",
        writes: &[REQUEST, INBOX_NAME, BUILT, LOG_RECORD, MARK, MARK_NAME],
        d_set: &[REQUEST, INBOX_NAME, BUILT, LOG_RECORD, MARK, MARK_NAME],
        promise: REQUEST | INBOX_NAME | BUILT | LOG_RECORD | MARK | MARK_NAME,
        deps: &[(LOG_RECORD, REQUEST | BUILT), (REQUEST, INBOX_NAME), (MARK, MARK_NAME)],
        preexisting: 0,
        stop: MARK | MARK_NAME,
        closure: 0,
    },
    Scenario {
        name: "conflicted",
        writes: &[BUILT, LOG_RECORD, LEDGER_LINE, LEDGER_NAME],
        d_set: &[BUILT, LOG_RECORD, LEDGER_LINE, LEDGER_NAME],
        promise: BUILT | LOG_RECORD | LEDGER_LINE | LEDGER_NAME,
        deps: &[(LOG_RECORD, BUILT), (LEDGER_LINE, LOG_RECORD | LEDGER_NAME)],
        preexisting: 0,
        stop: 0,
        closure: 0,
    },
    Scenario {
        name: "rewritten-away",
        writes: &[BUILT, LOG_RECORD, LEDGER_LINE, LEDGER_NAME],
        d_set: &[BUILT, LOG_RECORD, LEDGER_LINE, LEDGER_NAME],
        promise: BUILT | LOG_RECORD | LEDGER_LINE | LEDGER_NAME,
        deps: &[(LOG_RECORD, BUILT), (LEDGER_LINE, LOG_RECORD | LEDGER_NAME)],
        preexisting: 0,
        stop: 0,
        closure: 0,
    },
    Scenario {
        name: "W1 finds a terminal outcome",
        writes: &[LEDGER_LINE, LEDGER_NAME],
        d_set: &[LEDGER_LINE, LEDGER_NAME],
        promise: LEDGER_LINE | LEDGER_NAME,
        deps: &[(LEDGER_LINE, LEDGER_NAME)],
        preexisting: 0,
        stop: 0,
        closure: 0,
    },
    Scenario {
        name: "T1 reclamation (the lane releases the request's recovery copy)",
        writes: &[LEDGER_LINE, LEDGER_NAME, REMOVAL, RELEASED],
        d_set: &[LEDGER_LINE, LEDGER_NAME, REMOVAL],
        promise: LEDGER_LINE | LEDGER_NAME | REMOVAL,
        deps: &[(LEDGER_LINE, LEDGER_NAME), (REMOVAL, LEDGER_LINE)],
        preexisting: 0,
        stop: 0,
        closure: 0,
    },
    Scenario {
        name: "MG3 preservation (a staged-only version saved before the index entry is replaced)",
        writes: &[SAVED_CONTENT, SAVED_NAME, SAVED_RECORD, INDEX_NEW],
        d_set: &[SAVED_CONTENT, SAVED_NAME, SAVED_RECORD],
        promise: SAVED_CONTENT | SAVED_NAME | SAVED_RECORD,
        deps: &[(SAVED_RECORD, SAVED_CONTENT | SAVED_NAME), (SAVED_NAME, SAVED_CONTENT)],
        preexisting: 0,
        stop: 0,
        closure: 0,
    },
    Scenario {
        name: "carry-forward (the saved file made durable in the filesystem's order, then the displacing step)",
        writes: &[SAVED_CONTENT, SAVED_NAME, SAVED_RECORD, INDEX_NEW],
        d_set: &[SAVED_CONTENT, SAVED_NAME, SAVED_RECORD],
        promise: SAVED_CONTENT | SAVED_NAME | SAVED_RECORD,
        deps: &[(SAVED_RECORD, SAVED_CONTENT | SAVED_NAME), (SAVED_NAME, SAVED_CONTENT)],
        preexisting: 0,
        stop: 0,
        closure: 0,
    },
    Scenario {
        name: "a resend that finds the request file already linked (EEXIST) and then withdraws",
        writes: &[LOG_RECORD, MARK, MARK_NAME],
        d_set: &[REQUEST, INBOX_NAME, LOG_RECORD, MARK, MARK_NAME],
        promise: REQUEST | INBOX_NAME | LOG_RECORD | MARK | MARK_NAME,
        deps: &[(LOG_RECORD, REQUEST), (REQUEST, INBOX_NAME), (MARK, MARK_NAME)],
        preexisting: REQUEST | INBOX_NAME,
        stop: 0,
        closure: 0,
    },
    Scenario {
        name: "a named built commit whose parent holds content only another writer wrote",
        writes: &[REQUEST, INBOX_NAME, BUILT, LOG_RECORD],
        d_set: &[REQUEST, INBOX_NAME, BUILT, LOG_RECORD],
        promise: REQUEST | INBOX_NAME | BUILT | LOG_RECORD | PARENT,
        deps: &[(LOG_RECORD, REQUEST | BUILT), (REQUEST, INBOX_NAME), (BUILT, PARENT)],
        preexisting: PARENT,
        stop: 0,
        closure: PARENT,
    },
];

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 12: no barrier; the answer, the release and the replacement follow the writes.
    Twelve,
    /// Revision 13: the D-set is made durable in order before the answer, the release or the replacement.
    Thirteen,
    /// Control: revision 13 whose barrier forgets the last directory entry of the D-set.
    MissesLastName,
    /// Control: revision 13 whose barrier skips the facts another actor linked first (the resend's `EEXIST` fast path).
    SkipsExisting,
    /// Revision 14: the stop receipt is forced first and alone, the closure of a named built commit is forced with the request, and
    /// what happens when the stop cannot be confirmed in time is the owner's answer to question 6a (the parameter), never assumed.
    Fourteen(StopPolicy),
}

/// The owner's answer to question 6a (forcing the stop mark when the disk is slow); the model is run for each, never for one assumed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StopPolicy {
    /// Option A: wait up to `WAIT_STOP`, then leave and say the stop is not confirmed on disk.
    BoundedThenLeave,
    /// Option B: wait for the stop without a limit.
    Unbounded,
    /// Option C: do not force the stop at leaving; the background forces it.
    Background,
}

/// The kind of answer the command gave.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Answer {
    /// Not yet.
    No,
    /// The work is saved and durable (confirmed).
    Confirmed,
    /// The work is saved and durability is not confirmed (the barrier's bound passed).
    NotConfirmed,
}

/// What:
///  `struct State { ... }`: the scenario, the world (visible and durable facts), how far the writes and the barrier got.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules; scenario; world: { visible; durable }; written: number; synced: number; answer; crashed };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// Index into `SCENARIOS`.
    pub scenario: usize,
    /// Visible and durable facts.
    pub world: World,
    /// Writes done (the next write is `writes[written]`).
    pub written: usize,
    /// D-set facts made durable by the barrier so far.
    pub synced: usize,
    /// The command's answer.
    pub answer: Answer,
    /// A power loss happened.
    pub crashed: bool,
    /// The gated step (the last write) happened.
    pub gated_done: bool,
}

/// The start state.
pub fn initial(rules: Rules, scenario: usize) -> State {
    let preexisting: u32 = SCENARIOS[scenario].preexisting;
    return State { rules, scenario, world: World { visible: preexisting, durable: 0 }, written: 0, synced: 0, answer: Answer::No, crashed: false, gated_done: false };
}

fn gated(scenario: &Scenario) -> bool {
    return scenario.name.starts_with("MG3") || scenario.name.starts_with("carry-forward") || scenario.name.starts_with("T1");
}

/// The facts the barrier forces, in order, under the state's rule.
fn barrier(state: &State) -> Vec<u32> {
    let scenario: Scenario = SCENARIOS[state.scenario];
    let mut facts: Vec<u32> = scenario.d_set.to_vec();
    if let Rules::Fourteen(policy) = state.rules {
        // The stop receipt first and alone; then the closure the exit names; then the rest of the D-set.
        let stop_facts: Vec<u32> = facts.iter().copied().filter(|fact| { return scenario.stop & *fact != 0; }).collect();
        let rest: Vec<u32> = facts.iter().copied().filter(|fact| { return scenario.stop & *fact == 0; }).collect();
        let mut ordered: Vec<u32> = Vec::new();
        if policy != StopPolicy::Background {
            ordered.extend(stop_facts);
        }
        let mut bit: u32 = 1;
        while bit <= LAST_FACT {
            if scenario.closure & bit != 0 && scenario.preexisting & bit != 0 {
                ordered.push(bit);
            }
            bit <<= 1;
        }
        ordered.extend(rest);
        facts = ordered;
    }
    if state.rules == Rules::MissesLastName {
        facts.pop();
    }
    if state.rules == Rules::SkipsExisting {
        facts.retain(|fact| { return scenario.preexisting & *fact == 0; });
    }
    return facts;
}

fn barrier_len(state: &State) -> usize {
    return barrier(state).len();
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let scenario: Scenario = SCENARIOS[state.scenario];
    let last: usize = scenario.writes.len() - 1;
    let gate: bool = gated(&scenario);
    if !state.crashed {
        let barrier_done: bool = state.rules == Rules::Twelve || state.synced == barrier_len(state);
        let next_is_gated: bool = gate && state.written == last;
        // The gated step (index replacement, release) waits for the barrier under revision 13.
        if state.written < scenario.writes.len() && (!next_is_gated || barrier_done) {
            let mut after: State = state.clone();
            after.world = make_visible(state.world, scenario.writes[state.written]);
            if scenario.writes[state.written] == INDEX_NEW {
                // Git's own index write is durable as core.fsync makes it; the saved file is what rule V must make durable.
                after.world = sync(after.world, INDEX_NEW);
            }
            after.written = state.written + 1;
            after.gated_done = next_is_gated;
            push_step(&mut found, "client: writes the next fact", after);
        }
        let writes_before_barrier: usize = if gate { last } else { scenario.writes.len() };
        if state.rules != Rules::Twelve && state.written >= writes_before_barrier && state.synced < barrier_len(state) {
            let fact: u32 = barrier(state)[state.synced];
            if state.world.visible & fact != 0 {
                let mut after: State = state.clone();
                after.world = sync(state.world, fact);
                after.synced = state.synced + 1;
                push_step(&mut found, "client: forces the next fact of the D-set", after);
            }
        }
        let ready: bool = !gate && state.written == scenario.writes.len();
        if state.answer == Answer::No && ready && (state.rules == Rules::Twelve || state.synced == barrier_len(state)) {
            let mut after: State = state.clone();
            after.answer = Answer::Confirmed;
            push_step(&mut found, "client: answers confirmed", after);
        }
        let stop_forced: bool = scenario.stop == 0 || state.world.durable & scenario.stop == scenario.stop;
        let may_leave_unconfirmed: bool = match state.rules {
            // Option B waits for the stop without a limit; only the saved copy has a bound.
            Rules::Fourteen(StopPolicy::Unbounded) => stop_forced,
            _ => true,
        };
        if state.answer == Answer::No && ready && state.rules != Rules::Twelve && state.synced < barrier_len(state) && may_leave_unconfirmed {
            let mut after: State = state.clone();
            after.answer = Answer::NotConfirmed;
            push_step(&mut found, "client: the departure bound passed; answers saved and not confirmed", after);
        }
        // Write-back persists any visible fact, in any order, at any time.
        let mut bit: u32 = 1;
        while bit <= LAST_FACT {
            if state.world.visible & bit != 0 && state.world.durable & bit == 0 {
                let mut after: State = state.clone();
                after.world = sync(state.world, bit);
                push_step(&mut found, "environment: write-back persists one visible fact", after);
            }
            bit <<= 1;
        }
        let mut lost: State = state.clone();
        lost.world = power_loss(state.world);
        lost.crashed = true;
        push_step(&mut found, "environment: power is lost", lost);
    }
    return found;
}

/// Whether the single fact `bit` can be used at recovery: it is there and every fact it names is usable (a bounded walk over the
/// scenario's dependency table, which has no cycle).
pub fn recoverable(world: World, bit: u32, deps: &[(u32, u32)]) -> bool {
    if world.visible & bit == 0 {
        return false;
    }
    for (name, needs) in deps {
        if *name != bit {
            continue;
        }
        let mut need_bit: u32 = 1;
        while need_bit <= LAST_FACT {
            if needs & need_bit != 0 && !recoverable(world, need_bit, deps) {
                return false;
            }
            need_bit <<= 1;
        }
    }
    return true;
}

fn all_recoverable(world: World, promise: u32, deps: &[(u32, u32)]) -> bool {
    let mut bit: u32 = 1;
    while bit <= LAST_FACT {
        if promise & bit != 0 && !recoverable(world, bit, deps) {
            return false;
        }
        bit <<= 1;
    }
    return true;
}

/// The promise after the power loss: a confirmed answer, a release of the recovery copy, or a replaced index entry each leave
/// the scenario's promised facts recoverable.
pub fn invariant(state: &State) -> Option<String> {
    let scenario: Scenario = SCENARIOS[state.scenario];
    if !state.crashed {
        return None;
    }
    // Whatever the wording, an answer that returns the command means no attempt may start later: the stop must survive the loss.
    if state.answer != Answer::No && scenario.stop != 0 && !all_recoverable(state.world, scenario.stop, scenario.deps) {
        return Some(format!("after the power loss the stop disposition of `{}` is lost and the request is live", scenario.name));
    }
    let promised: bool = state.answer == Answer::Confirmed || state.gated_done;
    if promised && !all_recoverable(state.world, scenario.promise, scenario.deps) {
        return Some(format!("after the power loss the promise of `{}` is not recoverable", scenario.name));
    }
    return None;
}

/// The command (or the lane) has not finished its scenario.
pub fn waiting(state: &State) -> bool {
    let scenario: Scenario = SCENARIOS[state.scenario];
    return !state.crashed && state.written < scenario.writes.len();
}

/// Steps the waiting command takes itself: its writes and its forced facts.
pub fn is_own(label: &str) -> bool {
    return label.starts_with("client:");
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 2_000_000, max_depth: 40 };
}

/// Explores one scenario under `rules`: every interleaving of writes, forced facts, write-back and one power loss.
pub fn check(rules: Rules, scenario: usize) -> Report {
    let bound: usize = SCENARIOS[scenario].writes.len() + SCENARIOS[scenario].d_set.len() + 1;
    return explore_bounded_by(vec![initial(rules, scenario)], steps, invariant, waiting, is_own, bound, limits());
}
