//! What:
//!  Model of revision 13's register of waits: a command that passes through a path of entries (CAPTURE, FOREGROUND, COMMITTING and
//!  the rest), each with one monotonic deadline that retries never renew, and an action at the bound that is a sequence of stages
//!  where every stage can fail or stall instead of completing by assignment.
//! Why:
//!  "Nothing should ever wait forever or indefinitely" (the owner's rule) is a property of the command, not of a count of
//!  protocol steps. The model carries a clock in the state: the timer's ticks are the process's own steps (an assumption stated
//!  here: the process is scheduled and the monotonic clock advances), a retry or a re-read costs a tick instead of renewing the
//!  deadline, and after the deadline the process's own steps reach an exit whatever the other parties do. Stages are of two
//!  kinds: a local one (a read, a link, a write) that succeeds or fails, and a foreign one (the lane records the withdrawal, the
//!  syncer flushes the D-set) whose completion belongs to another party and which has a bound of its own (`WAIT_WITHDRAW`,
//!  `WAIT_DEPART`) after which the command goes on and says that the work is saved and not confirmed. The checked bound is derived
//!  from the table (`path_bound`), and the exit's wording is checked against what the stages did (a failed or escaped stage never
//!  yields "confirmed"). Entries with no bound by the owner's decision 9 (a forwarded `git gc`, the person's own programs before a
//!  commit exists) are listed on their own and checked to be unbounded, not given a bound that would be false.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Entry = { name; waits; retries; continues; stages: [string, 'local' | 'foreign'][] };
//! // const pathBound = (path: Entry[]) => sum(path.map((e) => BUDGET + 1 + e.stages.length + 1));
//! ```

use crate::model_explore::{Limits, Report, explore_bounded_by, push_step};

/// The deadline of one entry, in ticks of the monotonic clock (a model unit; the shipped values are the register's constants).
pub const BUDGET: u8 = 2;

/// Who completes a stage.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Stage {
    /// The command's own operation: it succeeds, fails, or blocks (a filesystem call that does not return); the command's own deadline
    /// is the independently executable escape from a block.
    Local,
    /// The departure barrier's forcing of the final save (owner decision 14: it is assumed never to be cut short). Completes by the disk's
    /// answer only; under revision 15 it has no escape and is not counted as a wait, and a sync that never completes is an outside cause.
    /// Under the earlier rules it behaves as a foreign stage with its own bound.
    Barrier,
    /// A local operation whose failure or block means the stop authority was not recorded: the request may still land, and the exit must
    /// say that and not only that the save is unconfirmed (revision 14).
    Authority,
    /// Another party completes it; revision 13 gives it a bound of its own after which the command goes on unconfirmed.
    Foreign,
}

/// One row of the register.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Entry {
    /// The register's name.
    pub name: &'static str,
    /// Whether the process waits on another party before the action (false: the L6 failed try is itself the action).
    pub waits: bool,
    /// Whether a loop re-reads state while waiting (CAPTURE's C1, C0's re-evaluation, Q1's triggers).
    pub retries: bool,
    /// Whether the action ends in the next entry instead of an exit (ACQUIRE waits as in FOREGROUND; WITHDRAW leads to DEPART).
    pub continues: bool,
    /// The action at the bound, in order.
    pub stages: &'static [(&'static str, Stage)],
}

/// Index of CAPTURE in `ENTRIES`.
pub const CAPTURE: usize = 1;
/// Index of FOREGROUND in `ENTRIES`.
pub const FOREGROUND: usize = 3;
/// Index of COMMITTING in `ENTRIES`.
pub const COMMITTING: usize = 6;
/// Index of WITHDRAW in `ENTRIES`.
pub const WITHDRAW: usize = 7;
/// Index of DEPART in `ENTRIES`.
pub const DEPART: usize = 8;

/// The register of revision 13 (the entries that wait or act at a bound; the unbounded ones are `EXEMPT`).
pub const ENTRIES: [Entry; 20] = [
    Entry { name: "START", waits: true, retries: false, continues: false, stages: &[("W1, or the plain error when no request exists", Stage::Local)] },
    Entry { name: "CAPTURE", waits: true, retries: true, continues: false, stages: &[("exits non-zero: the lane kept changing, run again", Stage::Local)] },
    Entry { name: "MARKER", waits: true, retries: true, continues: false, stages: &[("exits non-zero: the lane was recovering, run again", Stage::Local)] },
    Entry {
        name: "FOREGROUND",
        waits: true,
        retries: false,
        continues: false,
        stages: &[
            ("W1 looks the identity up by the terminal route", Stage::Local),
            ("W1 creates the withdrawal mark", Stage::Authority),
            ("W1 writes abort to an open attempt", Stage::Local),
            ("W1 waits for W2 to record the withdrawal (WAIT_WITHDRAW)", Stage::Foreign),
            ("W1 forces the D-set (the final save)", Stage::Barrier),
            ("W1 looks the identity up again", Stage::Local),
        ],
    },
    Entry { name: "ACQUIRE", waits: true, retries: false, continues: true, stages: &[("stops being an owner candidate", Stage::Local)] },
    Entry { name: "SERVICE-ACQUIRE", waits: true, retries: false, continues: false, stages: &[("the service exits and the manager restarts it", Stage::Local)] },
    Entry {
        name: "COMMITTING",
        waits: true,
        retries: false,
        continues: false,
        stages: &[
            ("creates the mark withdraw-unresolved a", Stage::Authority),
            ("forces the D-set (the final save)", Stage::Barrier),
            ("exits non-zero: the instruction was submitted and its result is not known", Stage::Local),
        ],
    },
    Entry { name: "WITHDRAW", waits: true, retries: false, continues: true, stages: &[("W2 records the withdrawal (WAIT_WITHDRAW)", Stage::Foreign)] },
    Entry { name: "DEPART", waits: true, retries: false, continues: false, stages: &[("the syncer forces the D-set (the final save)", Stage::Barrier)] },
    Entry {
        name: "REAP",
        waits: true,
        retries: false,
        continues: false,
        stages: &[("an inline client with an unpublished request performs W1 first", Stage::Local), ("releases the lane by exiting", Stage::Local)],
    },
    Entry {
        name: "KILL",
        waits: true,
        retries: false,
        continues: false,
        stages: &[("releases the gate", Stage::Local), ("exits non-zero naming an owner that did not die", Stage::Local)],
    },
    Entry { name: "STALL", waits: true, retries: false, continues: true, stages: &[("cancels the stalled child", Stage::Local)] },
    Entry { name: "PREPARED-HOLD", waits: true, retries: false, continues: true, stages: &[("writes abort to the pipe", Stage::Local)] },
    Entry {
        name: "PUSH",
        waits: true,
        retries: false,
        continues: false,
        stages: &[("ends the push child when its invocation cannot run a protected program", Stage::Local), ("queues the push and exits 0", Stage::Local)],
    },
    Entry {
        name: "L6-FAILED-TRY",
        waits: false,
        retries: false,
        continues: false,
        stages: &[("reads lane/restart-intent", Stage::Local), ("admits nothing and exits", Stage::Local)],
    },
    Entry { name: "ACQUIRE-RECOVERY-HELPER", waits: false, retries: false, continues: false, stages: &[("reports lost and exits", Stage::Local)] },
    Entry { name: "SIGNER", waits: true, retries: false, continues: true, stages: &[("keeps waiting as in FOREGROUND (the person's signer)", Stage::Local)] },
    Entry { name: "STOP", waits: true, retries: false, continues: false, stages: &[("forces the stop receipt: the mark file and lane/decision/", Stage::Barrier)] },
    Entry {
        name: "MATERIALIZE",
        waits: true,
        retries: false,
        continues: false,
        stages: &[("waits for the single attempt at M1 or MG3, then exits 0 published with the install continuing (WAIT_MATERIALIZE)", Stage::Foreign)],
    },
    Entry {
        name: "DRAIN-REQUEST",
        waits: true,
        retries: false,
        continues: false,
        stages: &[("the service finishes the drain (WAIT_DRAIN, unmeasured: no constant is shipped)", Stage::Foreign), ("returns non-zero: draining, upgrade not applied; the drain continues", Stage::Local)],
    },
];

/// Index of DRAIN-REQUEST in `ENTRIES`.
pub const DRAIN_REQUEST: usize = 19;

/// Index of STOP in `ENTRIES`.
pub const STOP: usize = 17;
/// Index of MATERIALIZE in `ENTRIES`.
pub const MATERIALIZE: usize = 18;

/// The entries with no bound, each by the owner's decision 9 and each on its own line of the register.
pub const EXEMPT: [(&str, &str); 2] = [
    ("FORWARDED-MAINTENANCE", "a forwarded git gc, prune or maintenance that the wrapper runs in the foreground for the person"),
    ("PERSON-PROGRAMS", "the editor, pre-commit, commit-msg, prepare-commit-msg, a signer's prompt and filters run before any request exists"),
];

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 12: a retry renews the deadline, a foreign stage has no bound of its own, COMMITTING exits 0 promising a landing.
    Twelve,
    /// Revision 13: one deadline per entry, every foreign stage bounded except STOP and MATERIALIZE, which the register did not have.
    Thirteen,
    /// Revision 14: Revision 13 plus the entries STOP and MATERIALIZE and an exit that says the request may still land when the stop
    /// authority could not be recorded.
    Fourteen,
    /// Revision 15 with owner decision 14: the departure barrier (the stop receipt and the D-set) runs to completion and is not a bounded
    /// wait; every other entry is as under revision 14.
    Fifteen,
    /// Control: revision 15 whose ordinary "the wait ends" step still carries a departing command to a normal return without its final save.
    FifteenOrdinaryWait,
    /// Control: revision 15 whose command returns normally after the stop authority could not be recorded (the warning variant).
    FifteenNormalAfterLoss,
    /// Control: revision 15 whose terminal outcome (a refusal, a conflict, a rewritten-away or an already) ends the wait and returns normally
    /// without forcing the absorbing outcome's ledger line (the ordinary-completion bypass of models review 7).
    FifteenOrdinaryBypass,
    /// Control only: revision 13 whose exit wording ignores failed stages (the abstraction the third models review rejected).
    IgnoresFailures,
}

/// Whether `rules` is revision 15 or one of its single-defect controls (they share every mechanism but the one each control breaks).
pub fn is_fifteen(rules: Rules) -> bool {
    return matches!(rules, Rules::Fifteen | Rules::FifteenOrdinaryWait | Rules::FifteenNormalAfterLoss | Rules::FifteenOrdinaryBypass);
}

/// Whether the entry is a departure: its whole action is the final save, so an ordinary "the other party progresses" cannot end it.
pub fn is_departure(entry: Entry) -> bool {
    return entry.name == "DEPART" || entry.name == "STOP";
}

/// Where the command is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Phase {
    /// Waiting on another party in the entry.
    Waiting,
    /// Running the action's stage with this index.
    Acting(usize),
    /// The stages are done; the exit remains.
    Exiting,
    /// The command ended.
    Done,
}

/// What:
///  `struct State { ... }`: the path, the position in it, the phase, the entry's clock and what the stages did.
/// Why:
///  Flat `Copy` fields hash cheaply; the path is a static slice, which hashes by content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules; path: number[]; pos; phase; elapsed; invalidated; unconfirmed; claimsConfirmed; promisedLanding };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The entries the command passes through, in order.
    pub path: &'static [usize],
    /// The index in `path` of the entry now active.
    pub pos: usize,
    /// Where the command is in that entry.
    pub phase: Phase,
    /// Ticks of the entry's monotonic clock since the command first needed the entry.
    pub elapsed: u8,
    /// The state the loop read has changed since the loop last read it.
    pub invalidated: bool,
    /// A stage failed or a foreign stage hit its own bound.
    pub unconfirmed: bool,
    /// A local operation is blocked and has not returned.
    pub stalled: bool,
    /// The stop authority could not be recorded (an authority stage failed, blocked or hit its own bound).
    pub authority_lost: bool,
    /// The exit said that the request may still land.
    pub says_may_land: bool,
    /// The final save (a barrier stage) completed.
    pub saved: bool,
    /// The wait ended by a terminal outcome of the request (revision 15): the absorbing outcome's ledger line is the stop disposition and is
    /// owed to disk before any normal return. A published outcome owes nothing: the receipt forbids a later attempt and is queued unforced.
    pub terminal_outcome: bool,
    /// The wait ended because the request was published: nothing is owed to the stop disposition, and this state records that completion.
    pub published: bool,
    /// The command ran its action (a bound was reached) and did not end by an ordinary completion.
    pub acted: bool,
    /// The exit was an outside-fault exit (revision 17: the stop authority could not be recorded), not a normal return.
    pub outside_fault: bool,
    /// The exit said that everything was confirmed.
    pub claims_confirmed: bool,
    /// The exit was zero and promised a landing later.
    pub promised_landing: bool,
}

/// The entry now active.
pub fn entry_of(state: &State) -> Entry {
    return ENTRIES[state.path[state.pos]];
}

/// The start state of a command whose first entry is `path[0]`.
pub fn initial(rules: Rules, path: &'static [usize]) -> State {
    let first: Entry = ENTRIES[path[0]];
    let phase: Phase = if first.waits { Phase::Waiting } else { Phase::Acting(0) };
    return State {
        rules,
        path,
        pos: 0,
        phase,
        elapsed: if first.waits { 0 } else { BUDGET },
        invalidated: false,
        unconfirmed: false,
        stalled: false,
        authority_lost: false,
        says_may_land: false,
        saved: false,
        acted: false,
        terminal_outcome: false,
        published: false,
        outside_fault: false,
        claims_confirmed: false,
        promised_landing: false,
    };
}

fn advance(state: &State) -> State {
    let mut after: State = state.clone();
    if state.pos + 1 < state.path.len() {
        after.pos = state.pos + 1;
        let next: Entry = entry_of(&after);
        after.phase = if next.waits { Phase::Waiting } else { Phase::Acting(0) };
        after.elapsed = if next.waits { 0 } else { BUDGET };
        after.invalidated = false;
    } else {
        after.phase = Phase::Done;
    }
    return after;
}

fn after_stage(state: &State, index: usize) -> State {
    let mut after: State = state.clone();
    if index + 1 < entry_of(state).stages.len() {
        after.phase = Phase::Acting(index + 1);
    } else if entry_of(state).continues {
        after = advance(state);
    } else {
        after.phase = Phase::Exiting;
    }
    return after;
}

/// Every step enabled in `state`. Labels `timer:` and `client:` are the process's own steps; `environment:` are other parties'.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let entry: Entry = entry_of(state);
    match state.phase {
        Phase::Waiting => {
            // Ordinary completion of the wait. A departure is its own action: under revision 15 it cannot be ended by this step.
            let barrier: Option<usize> = entry.stages.iter().position(|stage| { return stage.1 == Stage::Barrier; });
            if let Some(at) = barrier
                && is_fifteen(state.rules)
                && !is_departure(entry)
            {
                // The ordinary end of a wait is one of two typed completions (models review 7, finding 1): the request was published (the
                // receipt, queued without forcing, forbids a later attempt, so no final save is owed) or it reached a terminal outcome
                // (the ledger line is the stop disposition, and the departure barrier forces it before any normal return).
                let mut published: State = advance(state);
                published.published = true;
                push_step(&mut found, "environment: the other party publishes the request and the wait ends", published);
                if state.rules == Rules::FifteenOrdinaryBypass {
                    let mut bypass: State = advance(state);
                    bypass.terminal_outcome = true;
                    push_step(&mut found, "environment: the other party reaches a terminal outcome and the wait ends", bypass);
                } else {
                    let mut owed: State = state.clone();
                    owed.phase = Phase::Acting(at);
                    owed.terminal_outcome = true;
                    push_step(&mut found, "environment: the other party reaches a terminal outcome and the wait ends", owed);
                }
            } else if !(state.rules == Rules::Fifteen || state.rules == Rules::FifteenNormalAfterLoss) || !is_departure(entry) {
                push_step(&mut found, "environment: the other party progresses and the wait ends", advance(state));
            }
            if state.elapsed < BUDGET {
                let mut after: State = state.clone();
                after.elapsed = state.elapsed + 1;
                push_step(&mut found, "timer: the monotonic clock advances one tick", after);
            }
            if entry.retries && !state.invalidated {
                let mut after: State = state.clone();
                after.invalidated = true;
                push_step(&mut found, "environment: the lane's state changes under the loop", after);
            }
            // A retry costs a tick and never renews the deadline (revision 13); after the deadline the loop stops and the action starts.
            // Revision 12's loop drew a fresh deadline on each pass, so it could repeat for ever.
            if entry.retries && state.invalidated && (state.elapsed < BUDGET || state.rules == Rules::Twelve) {
                let mut after: State = state.clone();
                after.invalidated = false;
                after.elapsed = if state.rules == Rules::Twelve { 0 } else { (state.elapsed + 1).min(BUDGET) };
                push_step(&mut found, "client: the loop reads the state again", after);
            }
            if state.elapsed >= BUDGET {
                let mut after: State = state.clone();
                after.phase = Phase::Acting(0);
                after.acted = true;
                push_step(&mut found, "client: the deadline is reached and the action starts", after);
            }
        }
        Phase::Acting(index) => {
            let (_, kind) = entry.stages[index];
            let has_escape: bool = match state.rules {
                Rules::Twelve => false,
                Rules::Thirteen => !(entry.name == "STOP" || entry.name == "MATERIALIZE"),
                Rules::Fourteen | Rules::IgnoresFailures => true,
                Rules::Fifteen | Rules::FifteenOrdinaryWait | Rules::FifteenNormalAfterLoss | Rules::FifteenOrdinaryBypass => true,
            };
            match kind {
                Stage::Local | Stage::Authority | Stage::Barrier if state.stalled => {
                    if has_escape {
                        let mut escaped: State = after_stage(state, index);
                        escaped.stalled = false;
                        escaped.unconfirmed = true;
                        escaped.authority_lost = state.authority_lost || kind == Stage::Authority;
                        push_step(&mut found, "client: the stage's own deadline expires while the operation is blocked and the command goes on", escaped);
                    }
                }
                Stage::Local | Stage::Authority => {
                    push_step(&mut found, "client: the stage succeeds", after_stage(state, index));
                    let mut failed: State = after_stage(state, index);
                    failed.unconfirmed = true;
                    failed.authority_lost = state.authority_lost || kind == Stage::Authority;
                    push_step(&mut found, "client: the stage fails", failed);
                    let mut blocked: State = state.clone();
                    blocked.stalled = true;
                    push_step(&mut found, "client: the operation blocks and does not return", blocked);
                }
                Stage::Foreign | Stage::Barrier => {
                    let mut completed: State = after_stage(state, index);
                    if state.terminal_outcome && kind == Stage::Barrier {
                        // A terminal outcome has no later stages: the saved ledger line is the whole action, and the exit follows.
                        completed.phase = Phase::Exiting;
                    }
                    completed.saved = completed.saved || kind == Stage::Barrier;
                    push_step(&mut found, "environment: the other party completes the stage", completed);
                    if has_escape && !(kind == Stage::Barrier && is_fifteen(state.rules)) {
                        let mut escaped: State = after_stage(state, index);
                        escaped.unconfirmed = true;
                        push_step(&mut found, "client: the stage's own bound is reached and the command goes on unconfirmed", escaped);
                    }
                }
            }
        }
        Phase::Exiting => {
            let mut after: State = state.clone();
            after.phase = Phase::Done;
            after.claims_confirmed = state.rules == Rules::IgnoresFailures || !state.unconfirmed;
            after.outside_fault = state.authority_lost && matches!(state.rules, Rules::Fifteen | Rules::FifteenOrdinaryWait | Rules::FifteenOrdinaryBypass);
            after.says_may_land = state.authority_lost && (matches!(state.rules, Rules::Fourteen) || is_fifteen(state.rules));
            after.promised_landing = state.rules == Rules::Twelve && state.path[state.pos] == COMMITTING;
            push_step(&mut found, "client: the command exits", after);
        }
        Phase::Done => {}
    }
    return found;
}

/// The exit's wording against what the stages did, and the owner's rule that no exit promises a later landing.
pub fn invariant(state: &State) -> Option<String> {
    if state.claims_confirmed && state.unconfirmed {
        return Some("the exit said everything was confirmed after a stage failed or hit its own bound".to_string());
    }
    if is_fifteen(state.rules) && state.phase == Phase::Done {
        if state.terminal_outcome && !state.saved {
            return Some("a normal return after a terminal outcome without the final save of its ledger line".to_string());
        }
        let entry: Entry = entry_of(state);
        let has_barrier: bool = entry.stages.iter().any(|stage| { return stage.1 == Stage::Barrier; });
        if has_barrier && !state.saved && !state.authority_lost && !state.published {
            return Some("a normal return without the final save of its departure or a recorded published completion".to_string());
        }
        if state.authority_lost && !state.outside_fault {
            return Some("a normal return after the stop authority could not be recorded".to_string());
        }
    }
    if state.authority_lost && state.phase == Phase::Done && !state.says_may_land {
        return Some("the stop could not be recorded and the exit did not say that the request may still land".to_string());
    }
    if state.promised_landing {
        return Some("the command exited 0 promising a landing after it returned".to_string());
    }
    return None;
}

/// The command has not ended and is not in the departure barrier under revision 15: the final save is assumed to complete (decision 14),
/// so waiting for the disk is not a wait the register bounds. Every other phase of every entry is.
pub fn waiting(state: &State) -> bool {
    if let Phase::Acting(index) = state.phase
        && is_fifteen(state.rules)
        && entry_of(state).stages[index].1 == Stage::Barrier
    {
        return false;
    }
    return state.phase != Phase::Done;
}

/// The entries whose barrier stage is exempt from the bound by owner decision 14, for the report.
pub const BARRIER_ENTRIES: [&str; 4] = ["FOREGROUND", "COMMITTING", "DEPART", "STOP"];

/// Whether a step is the command's own (its timer included).
pub fn is_own(label: &str) -> bool {
    return label.starts_with("client:") || label.starts_with("timer:");
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 200_000, max_depth: 80 };
}

/// The bound on one entry, derived from the table: the deadline's ticks and retries, the step that starts the action, one step per
/// stage, and the exit.
pub fn entry_bound(entry: Entry) -> usize {
    return usize::from(BUDGET) + 1 + 2 * entry.stages.len() + 1;
}

/// The bound on a command: the sum of its entries' bounds (the register's rule that the upper bound is the sum).
pub fn path_bound(path: &[usize]) -> usize {
    return path.iter().map(|index| { return entry_bound(ENTRIES[*index]); }).sum();
}

/// Explores a command that passes through `path` under `rules`.
pub fn check(rules: Rules, path: &'static [usize]) -> Report {
    return explore_bounded_by(vec![initial(rules, path)], steps, invariant, waiting, is_own, path_bound(path), limits());
}

/// An entry with no bound by decision 9: the command waits with no deadline, so the search reports a wait with nothing of the
/// protocol's own to end it. This is the stated exemption and not a defect of the model.
pub fn exempt_state() -> State {
    return State {
        rules: Rules::Thirteen,
        path: &[CAPTURE],
        pos: 0,
        phase: Phase::Waiting,
        elapsed: 0,
        invalidated: false,
        unconfirmed: false,
        stalled: false,
        authority_lost: false,
        says_may_land: false,
        saved: false,
        acted: false,
        terminal_outcome: false,
        published: false,
        outside_fault: false,
        claims_confirmed: false,
        promised_landing: false,
    };
}

/// Steps of a command in an exempt wait: only the other party's progress (no timer, no action).
pub fn exempt_steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let mut done: State = state.clone();
    done.phase = Phase::Done;
    push_step(&mut found, "environment: the program or maintenance ends", done);
    return found;
}

/// Explores the exempt wait.
pub fn check_exempt() -> Report {
    return explore_bounded_by(vec![exempt_state()], exempt_steps, invariant, waiting, is_own, 1, limits());
}
