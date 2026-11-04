//! Historical control, not approval evidence for revision 13: revision 11 decision file, infallible recovery, a child that stops with its owner; superseded by model_w1_r13.
//!
//! What:
//!  Model of revision 11's rule B for the one wait that can end a commit: the command's foreground wait (FOREGROUND), W1 (the client
//!  withdraws at the bound) against P2 in three steps (prepare, decide, commit) and the decision file `lane/decision/<id>`, with the
//!  owner dying at any point and a new owner recovering from the decision files first.
//! Why:
//!  The owner's rule "nothing should ever wait forever or indefinitely" and the decision "return with an error, and keep nothing that
//!  lands later" need one fact to be true in every schedule: a request either has the decision `commit` (it lands, and its client is
//!  told so or told it was decided) or the decision `withdraw` (it never lands). The model checks that with the explorer's liveness
//!  bound: the client's wait ends within a stated number of the protocol's own steps, and a person's hook that never returns is an
//!  environment step that no protocol step may depend on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; decision: 0|1|2; stage; ownerAlive; client; landed };
//! ```

use crate::model_explore::{Limits, Report, explore_bounded_by, push_step};

/// What:
///  `enum Rules { ... }`: the rule under test (TS: a union of string literals).
/// Why:
///  The rule lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'ten-waits' | 'ten' | 'eleven';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 10, reading one: the command waits for its request's outcome with no bound.
    TenWaits,
    /// Revision 10, reading two: the command stops waiting at its bound, exits 0 with a pending notice, and the request may land later.
    Ten,
    /// Revision 11: at the bound the client links `withdraw` (W1); P2 decides by the same link race (`commit`); a lost race reports.
    Eleven,
}

/// What:
///  `struct State { ... }`: the decision file, how far P2 got, the owner, the client and whether the ref was written.
///  `stage`: 0 not started, 1 prepare running (a person's hook inside), 2 prepared, 3 decided `commit` (durable, nothing sent),
///  4 acknowledged (`commit: ok` read, P3 recorded), 5 aborted by the decision (W2), 6 owner died before the decision (transaction
///  aborted by update-ref), 7 owner died after the decision (publication status uncertain), 8 the commit instruction sent, 9 the
///  child applied it (the ref is written) and the owner has not read the acknowledgement yet, 10 the commit failed after the decision.
///  `client`: 0 waiting, 1 reported published (exit 0), 2 told withdrawn (error), 3 told pending (exit 0), 4 told decided (exit 0).
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; decision: number; stage: number; ownerAlive: boolean; client: number; landed: boolean };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// `lane/decision/<id>`: 0 absent, 1 `commit`, 2 `withdraw`; created once, by a link, by whoever gets there first.
    pub decision: u8,
    /// How far P2 got.
    pub stage: u8,
    /// The owner lives.
    pub owner_alive: bool,
    /// What the client did.
    pub client: u8,
    /// The ref was written (the request landed).
    pub landed: bool,
    /// W2 recorded the request `withdrawn`.
    pub withdrawn: bool,
    /// The client returned before the ref was written.
    pub returned_before_landing: bool,
}

/// The start state.
pub fn initial(rules: Rules) -> State {
    return State { rules, decision: 0, stage: 0, owner_alive: true, client: 0, landed: false, withdrawn: false, returned_before_landing: false };
}

/// Every step enabled in `state`: the lane's P2 steps, W2, the client's, the environment's and recovery.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    let eleven: bool = state.rules == Rules::Eleven;
    if state.owner_alive {
        if state.stage == 0 && !(eleven && state.decision == 2) {
            let mut after: State = state.clone();
            after.stage = 1;
            push_step(&mut found, "P2a prepare: starts update-ref --stdin, writes prepare, a person's hook runs inside", after);
        }
        if state.stage == 0 && eleven && state.decision == 2 {
            let mut after: State = state.clone();
            after.stage = 5;
            after.withdrawn = true;
            push_step(&mut found, "W2: P1's pre finds the decision withdraw and withdraws the request", after);
        }
        if state.stage == 2 && eleven && state.decision == 0 {
            let mut after: State = state.clone();
            after.stage = 3;
            after.decision = 1;
            push_step(&mut found, "P2b decide: links the decision commit", after);
        }
        if state.stage == 2 && eleven && state.decision == 2 {
            let mut after: State = state.clone();
            after.stage = 5;
            after.withdrawn = true;
            push_step(&mut found, "P2b decide: finds withdraw, writes abort and ends as W2", after);
        }
        if state.stage == 2 && !eleven {
            let mut after: State = state.clone();
            after.stage = 3;
            push_step(&mut found, "P2b: nothing decides, the transaction goes on", after);
        }
        if state.stage == 3 {
            let mut after: State = state.clone();
            after.stage = 8;
            push_step(&mut found, "P2c send: writes commit to the transaction", after);
        }
        if state.stage == 8 {
            let mut applied: State = state.clone();
            applied.stage = 9;
            applied.landed = true;
            push_step(&mut found, "update-ref: applies the commit and writes the ref", applied);
            let mut failed: State = state.clone();
            failed.stage = 10;
            push_step(&mut found, "update-ref: the commit fails after the decision (store failure)", failed);
        }
        if state.stage == 9 {
            let mut after: State = state.clone();
            after.stage = 4;
            push_step(&mut found, "P2c ack: reads commit ok (P3 published)", after);
        }
        if state.stage == 10 {
            let mut after: State = state.clone();
            after.stage = 3;
            push_step(&mut found, "Q1: retries the decided commit on a trigger", after);
        }
    }
    if state.stage == 1 && state.owner_alive {
        let mut after: State = state.clone();
        after.stage = 2;
        push_step(&mut found, "environment: the person's hook returns and accepts", after);
    }
    if state.owner_alive && ((state.stage >= 1 && state.stage <= 3) || state.stage == 8 || state.stage == 9 || state.stage == 10) {
        let mut after: State = state.clone();
        after.owner_alive = false;
        // update-ref aborts a prepared transaction when its input ends; after the decision the decision stands, and what the child
        // did with a sent instruction is uncertain until recovery reads the ref.
        after.stage = if state.stage >= 3 && state.stage != 7 { 7 } else { 6 };
        push_step(&mut found, "environment: the owner dies; update-ref aborts what is prepared", after);
    }
    if !state.owner_alive {
        let mut after: State = state.clone();
        after.owner_alive = true;
        if state.stage == 6 && state.decision == 2 && eleven {
            after.stage = 5;
            after.withdrawn = true;
            push_step(&mut found, "L4: the new owner reads the decision files first and withdraws the request", after);
        } else if state.stage == 6 {
            after.stage = 0;
            push_step(&mut found, "L4: the new owner repeats the attempt", after);
        } else if state.stage == 7 && state.landed {
            after.stage = 4;
            push_step(&mut found, "L4: the tip equals new (R2), recovery records published", after);
        } else if state.stage == 7 {
            after.stage = 4;
            after.landed = true;
            push_step(&mut found, "L4: the decision commit stands, recovery lands the commit", after);
        }
    }
    if state.client == 0 {
        if state.stage == 4 {
            let mut after: State = state.clone();
            after.client = 1;
            push_step(&mut found, "client: sees published and exits 0", after);
        }
        if eleven && state.decision == 0 {
            let mut after: State = state.clone();
            after.decision = 2;
            after.client = 2;
            push_step(&mut found, "W1: the bound is reached, the client links the decision withdraw and exits with an error", after);
        }
        if eleven && state.decision == 2 {
            let mut after: State = state.clone();
            after.client = 2;
            push_step(&mut found, "W1: finds an earlier withdrawal and exits with an error", after);
        }
        if eleven && state.decision == 1 && state.stage != 4 {
            let mut after: State = state.clone();
            after.client = 4;
            after.returned_before_landing = !state.landed;
            push_step(&mut found, "COMMITTING: W1 found commit, the bound is reached, exits 0 saying the commit was decided", after);
        }
        if state.rules == Rules::Ten {
            let mut after: State = state.clone();
            after.client = 3;
            push_step(&mut found, "FOREGROUND: the bound is reached, exits 0 with a pending notice", after);
        }
    }
    return found;
}

/// The safety check: a request either lands (and its client was not told it never would) or never lands.
pub fn invariant(state: &State) -> Option<String> {
    if state.client == 2 && state.landed {
        return Some("the request landed after its client was told it never would".to_string());
    }
    if state.client == 4 && state.returned_before_landing && state.landed {
        return Some("the request landed after its command returned exit 0 saying it was decided".to_string());
    }
    if state.client == 3 && state.landed {
        return Some("the request landed after its command returned with a pending notice".to_string());
    }
    if state.withdrawn && state.landed {
        return Some("a withdrawn request landed".to_string());
    }
    return None;
}

/// The wait the rule bounds: the client is still waiting for its request's outcome.
pub fn waiting(state: &State) -> bool {
    return state.client == 0;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 10_000, max_depth: 40 };
}

/// Whether a step is one of the waiting client's own (its W1, its COMMITTING, its report, its pending exit); the lane's retries and
/// the environment's steps are not steps of this wait.
pub fn is_clients_step(label: &str) -> bool {
    return label.starts_with("W1:") || label.starts_with("COMMITTING:") || label.starts_with("client:") || label.starts_with("FOREGROUND:");
}

/// The bound, in the waiting client's own steps, on the client's wait (the bound itself is a step: the clock reaching it).
pub const FOREGROUND_BOUND: usize = 8;

/// Explores every schedule under `rules` with the liveness bound on the client's wait.
pub fn check(rules: Rules) -> Report {
    return explore_bounded_by(vec![initial(rules)], steps, invariant, waiting, is_clients_step, FOREGROUND_BOUND, limits());
}
