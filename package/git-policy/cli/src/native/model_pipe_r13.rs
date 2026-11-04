//! What:
//!  Model of P2a's pipe discipline: who holds which end of the named pipe to `update-ref --stdin`, one write per line or two,
//!  what a writer sees when the reader is dead, and whether end of input ever reaches the child. Plus the syncer's separation from the
//!  owner's loop (`WAIT_DEPART` and "foreground progress while the syncer flushes").
//! Why:
//!  Revision 12 left the bootstrap unspecified, so an implementation could keep the owner's own copy of the read end (a write to a
//!  dead reader then succeeds and the failure is hidden), let a hook descendant inherit a write end (end of input never arrives, so an
//!  abandoned prepared transaction never ends), or write a line in two parts (two writers interleave into a command Git cannot parse).
//!  Revision 13 specifies the bootstrap: the read end is opened non-blocking, passed as standard input and closed in the owner,
//!  every other descriptor is close-on-exec, and every line is one write of at most `PIPE_BUF` bytes. Twelve is modelled as every
//!  combination of those mistakes being possible; thirteen as the specified bootstrap only. A queued `commit` is not cancelled by
//!  later end of input (the child reads it first), which is why the owner's death does not undo a submitted instruction.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Pipe = { rules; readCopy; strayWrite; splitLines; child; client; owner; queued; partialFrom; garbage; hidden };
//! ```

use crate::model_explore::{Limits, Report, explore, explore_eventually, push_step};

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 12: unspecified; any combination of the mistakes can occur.
    Twelve,
    /// Revision 13: the specified bootstrap and the single-write discipline.
    Thirteen,
}

/// The child.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Child {
    /// Prepared, reading.
    Prepared,
    /// Ended.
    Dead,
}

/// What the pipe holds.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pipe {
    /// The rule under test.
    pub rules: Rules,
    /// The owner kept its own copy of the read end.
    pub read_copy: bool,
    /// A write end is held by a process that never writes (a hook descendant or an owner copy).
    pub stray_write: bool,
    /// Writers write a line in two parts.
    pub split_lines: bool,
    /// The child.
    pub child: Child,
    /// The client (writer one) has exited and closed its write end.
    pub client_exited: bool,
    /// The owner (writer two) has exited and closed its write end.
    pub owner_exited: bool,
    /// A complete closing line is queued.
    pub queued: bool,
    /// The writer (1 client, 2 owner) whose line is half written, 0 none.
    pub partial_from: u8,
    /// Two writers' halves were interleaved into a line Git cannot parse.
    pub garbage: bool,
    /// A write succeeded into a pipe whose only reader is a copy nobody reads.
    pub hidden_failure: bool,
    /// Writers that have written (bit 1 client, 2 owner).
    pub wrote: u8,
}

/// The start states: thirteen has one; twelve has every combination of the three mistakes.
pub fn initials(rules: Rules) -> Vec<Pipe> {
    let mut found: Vec<Pipe> = Vec::new();
    let options: &[bool] = if rules == Rules::Twelve { &[false, true] } else { &[false] };
    for read_copy in options {
        for stray_write in options {
            for split_lines in options {
                found.push(Pipe {
                    rules,
                    read_copy: *read_copy,
                    stray_write: *stray_write,
                    split_lines: *split_lines,
                    child: Child::Prepared,
                    client_exited: false,
                    owner_exited: false,
                    queued: false,
                    partial_from: 0,
                    garbage: false,
                    hidden_failure: false,
                    wrote: 0,
                });
            }
        }
    }
    return found;
}

fn write_steps(state: &Pipe, writer: u8, found: &mut Vec<(String, Pipe)>) {
    let exited: bool = if writer == 1 { state.client_exited } else { state.owner_exited };
    if exited || state.wrote & writer != 0 {
        return;
    }
    let name: &str = if writer == 1 { "client" } else { "owner" };
    if !state.split_lines {
        let mut after: Pipe = state.clone();
        after.wrote |= writer;
        if state.child == Child::Dead {
            // Atomic write to a pipe with no reader: EPIPE, which the writer sees; with a retained read copy it succeeds and hides it.
            after.hidden_failure = state.read_copy;
            after.wrote = state.wrote | if state.read_copy { writer } else { 0 };
            if !state.read_copy {
                after.wrote = state.wrote;
                push_step(found, &format!("{}: the write fails with EPIPE and the writer knows", name), after);
                return;
            }
        } else {
            after.queued = true;
        }
        push_step(found, &format!("{}: writes the line in one write", name), after);
    } else if state.partial_from == 0 {
        let mut after: Pipe = state.clone();
        after.partial_from = writer;
        push_step(found, &format!("{}: writes the first part of the line", name), after);
    } else if state.partial_from == writer {
        let mut after: Pipe = state.clone();
        after.partial_from = 0;
        after.wrote |= writer;
        after.queued = true;
        push_step(found, &format!("{}: writes the rest of the line", name), after);
    } else {
        let mut after: Pipe = state.clone();
        after.garbage = true;
        push_step(found, &format!("{}: writes its first part into the other writer's half line", name), after);
    }
}

/// Every step enabled.
pub fn pipe_steps(state: &Pipe) -> Vec<(String, Pipe)> {
    let mut found: Vec<(String, Pipe)> = Vec::new();
    write_steps(state, 1, &mut found);
    write_steps(state, 2, &mut found);
    if !state.client_exited {
        let mut after: Pipe = state.clone();
        after.client_exited = true;
        push_step(&mut found, "client: exits and closes its write end", after);
    }
    if !state.owner_exited {
        let mut after: Pipe = state.clone();
        after.owner_exited = true;
        push_step(&mut found, "owner: dies or exits and closes its write end", after);
    }
    if state.child == Child::Prepared && state.queued {
        let mut after: Pipe = state.clone();
        after.child = Child::Dead;
        push_step(&mut found, "git: update-ref reads the queued line, executes it and ends", after);
    }
    let eof: bool = state.client_exited && state.owner_exited && !state.stray_write;
    if state.child == Child::Prepared && eof && !state.queued {
        let mut after: Pipe = state.clone();
        after.child = Child::Dead;
        push_step(&mut found, "git: end of input reaches update-ref and it aborts the prepared transaction", after);
    }
    return found;
}

/// A write never hides a dead reader and two writers never interleave.
pub fn pipe_invariant(state: &Pipe) -> Option<String> {
    if state.hidden_failure {
        return Some("a write to a pipe whose child died succeeded because a retained read end hid the failure".to_string());
    }
    if state.garbage {
        return Some("two writers interleaved halves of closing lines into a command Git cannot parse".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 100_000, max_depth: 30 };
}

/// Explores the safety properties.
pub fn check_pipe(rules: Rules) -> Report {
    return explore(initials(rules), pipe_steps, pipe_invariant, |_| { return None; }, limits());
}

/// The child has ended (by a queued line or by end of input).
pub fn child_ended(state: &Pipe) -> bool {
    return state.child == Child::Dead;
}

/// Whether an abandoned prepared transaction can still end: from every reachable state the child can end by the protocol's own steps
/// (the owner and client closing their ends), which a retained write end prevents.
pub fn check_end_of_input(rules: Rules) -> Report {
    return explore_eventually(initials(rules), pipe_steps, child_ended, limits());
}

/// The syncer and the owner's loop.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Loop {
    /// True when the owner's loop waits on the syncer (revision 12's reading of "background" for the barrier).
    pub owner_waits_on_syncer: bool,
    /// The syncer is stalled in a slow flush.
    pub syncer_stalled: bool,
    /// The owner admitted the next request.
    pub admitted: bool,
}

/// Every step of the loop model.
pub fn loop_steps(state: &Loop) -> Vec<(String, Loop)> {
    let mut found: Vec<(String, Loop)> = Vec::new();
    if !state.syncer_stalled {
        let mut after: Loop = state.clone();
        after.syncer_stalled = true;
        push_step(&mut found, "environment: the syncer stalls in a slow flush", after);
    }
    if !state.admitted && !(state.owner_waits_on_syncer && state.syncer_stalled) {
        let mut after: Loop = state.clone();
        after.admitted = true;
        push_step(&mut found, "owner: admits the next request", after);
    }
    return found;
}

/// Foreground progress is possible while the syncer flushes: the owner can always admit.
pub fn loop_goal(state: &Loop) -> bool {
    return state.admitted;
}

/// Explores whether admission stays reachable from every state.
pub fn check_loop(owner_waits_on_syncer: bool) -> Report {
    let start: Loop = Loop { owner_waits_on_syncer, syncer_stalled: false, admitted: false };
    return explore_eventually(vec![start], loop_steps, loop_goal, limits());
}
