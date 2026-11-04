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
//! The steps labelled `git:` (reading the queued line, ending on end of input) are Git's documented reactions to its input
//! (`update-ref.c:712-770`, a backend gate that is still to be confirmed against the installed Git); they are labelled apart from
//! `environment:` because the check `explore_eventually` counts only the protocol's own steps. The green of `check_end_of_input` rests
//! on that assumption about Git, and not on any help from the environment.
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

/// The rule of the descriptor model.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GapRules {
    /// Revision 13: a writer holds its write end only for its one write, never idle (the idle-writer rule).
    Thirteen,
    /// Revision 14: the owner holds a keeper write end from before the first instruction until the closing line has run or the child
    /// has ended; other writers still hold theirs only for one write.
    Fourteen,
}

/// What:
///  The writer-lifetime state of one transaction's pipe: how many instruction lines the owner has written (`start`, `update`, `prepare`),
///  which write ends are open, what Git has read, and what end of input did. Descriptor acquisition and closure are separate from
///  process life, so the intervals between instructions exist.
/// Why:
///  The previous pipe model made every live process a continuous writer, so end of input was possible only after both exited. The
///  intervals are where revision 13 failed: the owner writes a line and closes, Git consumes it and finds no writer and no buffered byte,
///  and end of input aborts a transaction that the protocol meant to close itself (R13-3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gap = { rules; owner_lines: number; owner_end_open; client_end_open; git_read: number; prepared; closing: 'none'|'commit'|'abort'; eof_abort; ... };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Gap {
    /// The rule under test.
    pub rules: GapRules,
    /// Instruction lines the owner has written: 0 to 3 (`start`, `update`, `prepare`).
    pub owner_lines: u8,
    /// The owner's (keeper or per-line) write end is open.
    pub owner_end_open: bool,
    /// The client's write end is open (only for its one write).
    pub client_end_open: bool,
    /// Lines Git has consumed.
    pub git_read: u8,
    /// `prepare: ok` printed.
    pub prepared: bool,
    /// 0 none, 1 `commit` queued first, 2 `abort` queued first.
    pub closing: u8,
    /// The closing line ran (Git executed it).
    pub closing_ran: bool,
    /// End of input aborted the transaction while the owner lives and no closing line was queued.
    pub eof_abort: bool,
    /// The owner lives.
    pub owner_alive: bool,
    /// The client believes its `commit` is queued.
    pub client_believes_queued: bool,
    /// The owner's `abort` write failed with nothing queued (`EAGAIN` after the bounded retry, or the reader is gone).
    pub abort_enqueue_failed: bool,
    /// The child ended.
    pub child_ended: bool,
}

/// The start state: the owner opened the pipe and holds its end.
pub fn gap_initial(rules: GapRules) -> Gap {
    return Gap {
        rules,
        owner_lines: 0,
        owner_end_open: true,
        client_end_open: false,
        git_read: 0,
        prepared: false,
        closing: 0,
        closing_ran: false,
        eof_abort: false,
        owner_alive: true,
        client_believes_queued: false,
        abort_enqueue_failed: false,
        child_ended: false,
    };
}

/// Every step of the descriptor model.
pub fn gap_steps(state: &Gap) -> Vec<(String, Gap)> {
    let mut found: Vec<(String, Gap)> = Vec::new();
    if state.child_ended {
        return found;
    }
    if state.owner_alive && state.owner_lines < 3 && state.owner_end_open {
        let mut after: Gap = state.clone();
        after.owner_lines = state.owner_lines + 1;
        // Revision 13 closes the write end after each line; revision 14's keeper stays open.
        if state.rules == GapRules::Thirteen {
            after.owner_end_open = false;
        }
        push_step(&mut found, "owner: writes the next instruction line", after);
    }
    if state.owner_alive && state.owner_lines < 3 && !state.owner_end_open {
        let mut after: Gap = state.clone();
        after.owner_end_open = true;
        push_step(&mut found, "owner: opens its write end for the next line", after);
    }
    if state.git_read < state.owner_lines {
        let mut after: Gap = state.clone();
        after.git_read = state.git_read + 1;
        after.prepared = after.git_read == 3;
        push_step(&mut found, "git: reads the next instruction line", after);
    }
    if state.prepared && state.closing == 0 && !state.client_end_open {
        let mut after: Gap = state.clone();
        after.client_end_open = true;
        push_step(&mut found, "client: opens a write end after prepare: ok", after);
    }
    if state.client_end_open && state.closing == 0 {
        let mut queued: Gap = state.clone();
        queued.closing = 1;
        queued.client_end_open = false;
        queued.client_believes_queued = true;
        push_step(&mut found, "client: writes commit and closes its write end", queued);
        let mut failed: Gap = state.clone();
        failed.client_end_open = false;
        push_step(&mut found, "client: the commit write fails (EAGAIN after the bounded retry, or SIGPIPE) and nothing is queued", failed);
    }
    if state.prepared && state.closing == 0 && state.owner_alive && state.owner_end_open {
        let mut queued: Gap = state.clone();
        queued.closing = 2;
        push_step(&mut found, "owner: PREPARED-HOLD writes abort", queued);
        let mut failed: Gap = state.clone();
        failed.abort_enqueue_failed = true;
        push_step(&mut found, "owner: PREPARED-HOLD attempts abort and the write fails, nothing is queued", failed);
    }
    if state.prepared && state.closing != 0 && !state.closing_ran {
        let mut after: Gap = state.clone();
        after.closing_ran = true;
        // Revision 14's keeper is released once the closing line has run.
        after.owner_end_open = state.owner_end_open && state.rules == GapRules::Thirteen;
        push_step(&mut found, "git: executes the closing line", after);
    }
    if state.owner_alive {
        let mut after: Gap = state.clone();
        after.owner_alive = false;
        after.owner_end_open = false;
        push_step(&mut found, "environment: the owner dies and its write end closes", after);
    }
    // End of input: Git has consumed everything written, no write end is open, and nothing closes the transaction.
    let no_writer: bool = !state.owner_end_open && !state.client_end_open;
    let consumed: bool = state.git_read == state.owner_lines;
    let unfinished: bool = !state.closing_ran && state.closing == 0;
    if no_writer && consumed && unfinished && state.git_read > 0 {
        let mut after: Gap = state.clone();
        after.child_ended = true;
        after.eof_abort = state.owner_alive;
        push_step(&mut found, "git: end of input reaches update-ref and it aborts", after);
    }
    return found;
}

/// End of input never aborts a transaction the live owner meant to close itself, and a client never believes a failed write queued.
pub fn gap_invariant(state: &Gap) -> Option<String> {
    if state.eof_abort {
        return Some("end of input aborted a transaction while its owner lived and no closing line was queued".to_string());
    }
    if state.client_believes_queued && state.closing != 1 {
        return Some("the client believes its commit is queued and the first closing line is not commit".to_string());
    }
    return None;
}

/// Explores the descriptor model.
pub fn check_gap(rules: GapRules) -> Report {
    return explore(vec![gap_initial(rules)], gap_steps, gap_invariant, |_| { return None; }, limits());
}
