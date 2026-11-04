//! What:
//!  Model of the submission window of P2b and of what a drain may infer from the FIFO: a client records that it has not submitted, writes
//!  `commit`, and records that it has, and can die between any two of those; Git may read the written line ahead into its own buffer, so
//!  the FIFO looks empty while the instruction is held; the owner's classification (E1) and a planned shutdown's drain decide what they
//!  conclude from the records and from the FIFO.
//! Why:
//!  The fifteenth review (controls 7 and 13): a full `commit` write that succeeded before the client recorded it leaves an earlier,
//!  non-final `not-submitted` record as the last word, and reading that as "positively failed" lets a second attempt land beside the
//!  first; and a drain that finds the FIFO empty after Git's read-ahead concludes that nothing was submitted while a buffered `commit`
//!  still runs. Revision 15 reads a non-final record as unknown (class U) and never infers absence of a submission from the FIFO.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Sub = { rules; record: 'none' | 'not-submitted' | 'submitted'; fifoLine; buffered; clientAlive; landed; attempts; drainConcludedNone };
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 14: the last record decides (`not-submitted` is read as positively failed); a drain reads the FIFO.
    Fourteen,
    /// Revision 15: a non-final record is unknown; no conclusion about absence is drawn from the FIFO.
    Fifteen,
}

/// The client's last record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Record {
    /// None yet.
    None,
    /// `not-submitted`, written before the `commit` write; non-final.
    NotSubmitted,
    /// `submitted`, written after the `commit` write; final.
    Submitted,
}

/// The state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Sub {
    /// The rule under test.
    pub rules: Rules,
    /// The client's last record.
    pub record: Record,
    /// Ghost: the client wrote `commit` (whole line) into the FIFO.
    pub wrote: bool,
    /// The FIFO holds the line.
    pub fifo_line: bool,
    /// Git read the line ahead into its own buffer (the FIFO looks empty).
    pub buffered: bool,
    /// The client lives.
    pub client_alive: bool,
    /// Ghost: the first attempt's `commit` was executed and the ref written.
    pub landed: bool,
    /// The owner classified the first attempt as positively failed and started a second.
    pub second_attempt: bool,
    /// A drain concluded that nothing was submitted.
    pub drain_concluded_none: bool,
}

/// The start state.
pub fn initial(rules: Rules) -> Sub {
    return Sub {
        rules,
        record: Record::None,
        wrote: false,
        fifo_line: false,
        buffered: false,
        client_alive: true,
        landed: false,
        second_attempt: false,
        drain_concluded_none: false,
    };
}

/// Every step enabled.
pub fn steps(state: &Sub) -> Vec<(String, Sub)> {
    let mut found: Vec<(String, Sub)> = Vec::new();
    if state.client_alive {
        if state.record == Record::None {
            let mut after: Sub = state.clone();
            after.record = Record::NotSubmitted;
            push_step(&mut found, "client: records not-submitted (non-final)", after);
        }
        if state.record == Record::NotSubmitted && !state.wrote {
            let mut after: Sub = state.clone();
            after.wrote = true;
            after.fifo_line = true;
            push_step(&mut found, "client: writes commit", after);
        }
        if state.record == Record::NotSubmitted && state.wrote {
            let mut after: Sub = state.clone();
            after.record = Record::Submitted;
            push_step(&mut found, "client: records submitted", after);
        }
        let mut dies: Sub = state.clone();
        dies.client_alive = false;
        push_step(&mut found, "environment: the client dies", dies);
    }
    if state.fifo_line && !state.buffered {
        let mut after: Sub = state.clone();
        after.fifo_line = false;
        after.buffered = true;
        push_step(&mut found, "git: reads the line ahead into its buffer; the FIFO looks empty", after);
    }
    if (state.fifo_line || state.buffered) && !state.landed {
        let mut after: Sub = state.clone();
        after.landed = true;
        after.fifo_line = false;
        after.buffered = false;
        push_step(&mut found, "git: executes commit and writes the ref", after);
    }
    // E1 for a client that is gone: the last record decides under revision 14; under revision 15 only a final record decides, and a
    // non-final one is unknown (class U, never retried).
    if !state.client_alive && !state.second_attempt && state.record == Record::NotSubmitted {
        let failed: bool = state.rules == Rules::Fourteen;
        if failed {
            let mut after: Sub = state.clone();
            after.second_attempt = true;
            push_step(&mut found, "owner: E1 reads not-submitted as positively failed and starts a second attempt", after);
        }
    }
    // A planned shutdown's drain: revision 14 reads an empty FIFO as "nothing was submitted"; revision 15 draws no such conclusion.
    if state.rules == Rules::Fourteen && !state.fifo_line && !state.drain_concluded_none {
        let mut after: Sub = state.clone();
        after.drain_concluded_none = true;
        push_step(&mut found, "owner: the drain finds the FIFO empty and concludes that nothing was submitted", after);
    }
    return found;
}

/// A second attempt never starts beside a submitted `commit`, and a drain's conclusion is never contradicted by a landing.
pub fn invariant(state: &Sub) -> Option<String> {
    if state.second_attempt && state.wrote {
        return Some("a second attempt was started although the first attempt's commit was written".to_string());
    }
    if state.drain_concluded_none && state.wrote {
        return Some("the drain concluded that nothing was submitted and a commit was written (queued, buffered or run)".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 10_000, max_depth: 20 };
}

/// Explores every schedule.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, |_| { return None; }, limits());
}
