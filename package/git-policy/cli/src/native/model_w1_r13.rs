//! What:
//!  Model of revision 13's W1 (the client withdraws) against the publication of one request: the client, the owner, the
//!  `update-ref --stdin` transaction process and the branch tip as four actors whose steps interleave, with the pipe that orders
//!  the instructions, the withdrawal mark, E1's three kinds of result (P published, F positively failed, U unresolved after
//!  submission), PREPARED-HOLD and `WAIT_COMMITTING`.
//! Why:
//!  The owner's contract ("never discard user data", "nothing should ever wait forever or indefinitely", and the exceptions
//!  decisions 7 and 11 allow) is the invariant: after a command returned with the withdrawal error nothing lands; after the
//!  unresolved exit only the one attempt it names may land; a command that said published saw the landing. The model keeps
//!  facts apart that the earlier W1 model merged:
//!  - the transaction process makes progress whether or not its owner lives (no step of it reads `owner`);
//!  - the owner learns what happened only from observations (the report lines Git printed, the exit, a read of the tip that
//!    can fail), never from the ghost fact `landed`, which only the invariants read;
//!  - recovery is not a separate infallible step: a new incarnation reaps only after the child has ended, then classifies with
//!    the same fallible E1 as a living owner, and its evidence may be unreadable.
//!  Labels begin `client:`, `W1:` and `COMMITTING:` for the waiting client's own steps (the register's bound counts these),
//!  `owner:` for the lane, and `environment:` for Git, the person's hook, other writers and deaths, which never count.
//!  The pipe is modelled as the first closing line only: a second closing line makes `update-ref` die after the first has run
//!  and changes nothing that the first did (`update-ref.c:765-768`, a gate), so a second write is a step with no effect.
//!  One narrowing, stated: a request with no client whose attempt is unresolved (class U) is not replayed here; the recovery
//!  classes' landing replay (R5) lands a request that has no client and is outside the contract this model checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules; client; mark; owner; child; pipe; attempt; submitted; closed; tip; landed /* ghost */ };
//! // const steps = (s: State): [string, State][] => [...clientSteps(s), ...ownerSteps(s), ...gitSteps(s), ...environmentSteps(s)];
//! ```

use crate::model_explore::{Limits, Report, explore_bounded_by, push_step};

/// The most attempts the model starts (a retry after a positive failure needs a second).
pub const MAX_ATTEMPT: u8 = 2;

/// What:
///  `enum Rules { ... }`: the rule under test (TS: a union of string literals).
/// Why:
///  The rule lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'twelve' | 'thirteen';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 12: the unresolved exit leaves no mark; E1 infers "did not land" from an ended child and an absent tip.
    Twelve,
    /// Revision 13: every normal exit creates the mark first, `withdraw-unresolved a` forbids any later attempt, E1 has kind U.
    Thirteen,
    /// Revision 14: Revision 13 plus the client's receipt before `published`, a recovery that reads the receipt or `commit: ok` in the
    /// attempt's `out`, and the outcome `withdrawn` for a late positive failure under the unresolved mark.
    Fourteen,
    /// Control: revision 14 whose `out` file is a channel the person's hook descendants can write to (the backend obligation that Git
    /// redirects a hook's standard output to its standard error does not hold), so E1's evidence is not authentic.
    FourteenSharedChannel,
}

/// Whether `rules` read the client's receipt and the attempt's `out` at recovery (revision 14).
pub fn receipts(rules: Rules) -> bool {
    return matches!(rules, Rules::Fourteen | Rules::FourteenSharedChannel);
}

/// The withdrawal mark `lane/decision/<id>`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mark {
    /// No mark.
    Absent,
    /// `withdraw`: no new attempt may start, ever.
    Withdraw,
    /// `withdraw-unresolved <attempt>`: no attempt after the named one may start; the named one is observed only.
    Unresolved(u8),
}

/// The kind of exit a leaving client is on.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    /// W1 at the FOREGROUND bound: nothing was submitted that is still open.
    Plain,
    /// The COMMITTING exit, naming the attempt whose `commit` was submitted.
    Unresolved(u8),
}

/// What the person's command is doing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Client {
    /// Attached and waiting for the request's outcome.
    Waiting,
    /// Past its bound, steps of the exit still to run (the mark is created first).
    Leaving(Kind),
    /// Exited 0 saying `published`.
    Published,
    /// Exited non-zero: the request was withdrawn.
    Withdrawn,
    /// Exited non-zero: the instruction was submitted and its result is not known; names the attempt.
    UnresolvedExit(u8),
    /// Killed from outside; its client lock is released and no normal exit ran.
    Killed,
}

/// The transaction process (`update-ref --stdin`) of the current attempt.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Child {
    /// No attempt is running.
    NotStarted,
    /// `start` and `prepare` written; a person's `reference-transaction` hook runs inside.
    Preparing,
    /// `prepare: ok` printed; the branch lock is held; no closing line read yet.
    Prepared,
    /// Prepared and never reading again (a wedge of Git's own); the lock stays held.
    Wedged,
    /// Read `commit`; about to write the ref.
    Applying,
    /// The ref is written; `commit: ok` not printed yet.
    RefWritten,
    /// `commit: ok` printed; the process still lives.
    Reported,
    /// Read `abort`.
    Aborting,
    /// The process ended.
    Ended,
}

/// What Git itself wrote about the attempt (E1's evidence list, item i and ii), the last such line.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Line {
    /// Nothing printed.
    Nothing,
    /// `commit: ok`.
    CommitOk,
    /// `abort: ok`.
    AbortOk,
    /// A `child_exit` of the hook with a non-zero code.
    HookRefused,
    /// `prepare` failed with no `prepare: ok` and a non-zero exit.
    PrepareFailed,
}

/// The first closing line in the pipe.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pipe {
    /// No closing line written.
    Empty,
    /// `commit` queued first.
    Commit,
    /// `abort` queued first.
    Abort,
}

/// Where the branch tip is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Tip {
    /// At the base; nothing published.
    Base,
    /// At `new`.
    New,
    /// Moved by another writer after `new` was there.
    Foreign,
}

/// The owner: alive, or dead until a new incarnation reaps.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Owner {
    /// Alive.
    Alive,
    /// Dead; the child, if any, goes on until it ends.
    Dead,
}

/// E1's result for an attempt, recorded by the owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Class {
    /// P: positively published.
    Published,
    /// P by R3: the reflog shows `new` was written and the branch moved away; saved, not republished.
    RewrittenAway,
    /// F: positively failed before publication; the only kind that allows a later attempt.
    Failed,
    /// U: unresolved after submission; never retried.
    Unresolved,
}

/// What the owner could read of the tip when it classified.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Seen {
    /// The read failed (a store error).
    Unread,
    /// The tip does not contain `new` and no reflog entry shows it did.
    Absent,
    /// The tip equals `new`.
    Present,
    /// The reflog shows `new` was written and the branch moved away.
    Rewritten,
}

/// What:
///  `struct State { ... }`: every actor's own record and the facts the invariants read.
///  Ghost fields (only invariants read them, no step does): `landed`, `exit_attempt`.
/// Why:
///  Flat `Copy` fields hash cheaply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules; client; mark; owner; child; line; pipe; attempt; submitted; written; write_failed; closed; recorded;
///                 hold_expired; recovered; tip; reflog; landed; exit_attempt };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The command.
    pub client: Client,
    /// The withdrawal mark.
    pub mark: Mark,
    /// The owner.
    pub owner: Owner,
    /// The transaction process.
    pub child: Child,
    /// What Git printed.
    pub line: Line,
    /// The first closing line in the pipe.
    pub pipe: Pipe,
    /// The last attempt started (0 none).
    pub attempt: u8,
    /// The attempt for which the client recorded `submitted` (0 none).
    pub submitted: u8,
    /// The client's `commit` write for that attempt was queued.
    pub written: bool,
    /// The client's `commit` write failed with nothing queued (`EPIPE` or `ENXIO`); the client knows.
    pub write_failed: bool,
    /// The newest attempt the owner recorded `closed` after class F.
    pub closed: u8,
    /// E1's recorded result for the current attempt, if any: (attempt, class).
    pub recorded: Option<(u8, Class)>,
    /// PREPARED-HOLD expired and the owner enqueued `abort`.
    pub hold_expired: bool,
    /// A new incarnation reaped (so the owner's result evidence may be unreadable).
    pub recovered: bool,
    /// The branch tip.
    pub tip: Tip,
    /// The reflog shows `new` was written (survives a foreign reset only when the other writer did not erase it).
    pub reflog: bool,
    /// Ghost: the attempt that wrote the ref (0 none).
    pub landed: u8,
    /// Ghost: the newest attempt started when the command's exit became final.
    pub exit_attempt: u8,
    /// `lane/receipt/<id>`: the client recorded that it saw the landing and is about to print `published` (revision 14).
    pub receipt: bool,
    /// The owner recorded the outcome `withdrawn` for a late positive failure under the unresolved mark (revision 14).
    pub outcome_withdrawn: bool,
    /// The owner left by a planned shutdown (it recorded `clean-shutdown`), not by a crash.
    pub clean_shutdown: bool,
    /// Ghost: the owner decided (wrote `commit`) for this request.
    pub decided_by_owner: bool,
}

/// The start state: a bound request, no attempt, an attached client.
pub fn initial(rules: Rules) -> State {
    return State {
        rules,
        client: Client::Waiting,
        mark: Mark::Absent,
        owner: Owner::Alive,
        child: Child::NotStarted,
        line: Line::Nothing,
        pipe: Pipe::Empty,
        attempt: 0,
        submitted: 0,
        written: false,
        write_failed: false,
        closed: 0,
        recorded: None,
        hold_expired: false,
        recovered: false,
        tip: Tip::Base,
        reflog: false,
        landed: 0,
        exit_attempt: 0,
        receipt: false,
        outcome_withdrawn: false,
        clean_shutdown: false,
        decided_by_owner: false,
    };
}

/// Whether the child process exists (any state but not started or ended).
pub fn child_alive(child: Child) -> bool {
    return child != Child::NotStarted && child != Child::Ended;
}

/// Whether the client's `commit` instruction for the current attempt could still be executed (queued first, child able to read it).
pub fn commit_pending(state: &State) -> bool {
    return state.pipe == Pipe::Commit && matches!(state.child, Child::Preparing | Child::Prepared | Child::Applying | Child::RefWritten);
}

/// Whether the client has submitted `commit` for the current attempt and no positive close was recorded.
pub fn open_submitted(state: &State) -> bool {
    return state.submitted != 0 && state.closed < state.submitted;
}

/// What:
///  E1: the kind of an attempt's end, from evidence Git produced and the decider's record (revision 13) or from the negative
///  inference (revision 12).
///  `submitted: bool` is the decider's record that `commit` was submitted and not known to have failed to queue.
/// Why:
///  Revision 12 read "the child ended and the tip lacks `new`" as "nothing was published"; another writer can publish and reset
///  the branch away before the child dies (the twelfth review, R12-3). Revision 13 returns kind U in that case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classify(rules, submitted, line, seen, childEnded): Class;
/// ```
pub fn classify(rules: Rules, submitted: bool, line: Line, seen: Seen, ended: bool, receipt: bool) -> Class {
    if seen == Seen::Present || line == Line::CommitOk || (receipts(rules) && receipt) {
        return Class::Published;
    }
    if seen == Seen::Rewritten {
        return Class::RewrittenAway;
    }
    let positive_failure: bool = line == Line::AbortOk || line == Line::HookRefused || line == Line::PrepareFailed;
    if positive_failure {
        return Class::Failed;
    }
    if !submitted && ended {
        return Class::Failed;
    }
    if rules == Rules::Twelve && ended && seen == Seen::Absent {
        return Class::Failed;
    }
    return Class::Unresolved;
}

fn effective_submission(state: &State) -> bool {
    return state.submitted == state.attempt && state.attempt != 0 && !state.write_failed;
}

fn exit_state(state: &State) -> State {
    let mut after: State = state.clone();
    after.exit_attempt = state.attempt;
    return after;
}

/// The owner's steps.
fn owner_steps(state: &State, found: &mut Vec<(String, State)>) {
    let thirteen: bool = state.rules != Rules::Twelve;
    if state.owner == Owner::Dead {
        if matches!(state.child, Child::NotStarted | Child::Ended) {
            let mut after: State = state.clone();
            after.owner = Owner::Alive;
            after.recovered = true;
            push_step(found, "owner: a new incarnation reaps the previous scope (L2) and takes over", after);
        }
        return;
    }
    let mark_blocks_start: bool = if thirteen { state.mark != Mark::Absent } else { state.mark == Mark::Withdraw };
    let previous_closed: bool = state.attempt == 0 || (state.child == Child::Ended && state.closed == state.attempt);
    // What the owner can observe: its own recorded class, and (revision 14) the client's receipt. The client's actual exit is not
    // observable: a released client lock looks the same after a success and after a kill.
    let finished: bool = matches!(state.recorded, Some((_, Class::Published | Class::RewrittenAway))) || (receipts(state.rules) && state.receipt);
    if !mark_blocks_start && previous_closed && state.attempt < MAX_ATTEMPT && !finished {
        let mut after: State = state.clone();
        after.attempt = state.attempt + 1;
        after.child = Child::Preparing;
        after.line = Line::Nothing;
        after.pipe = Pipe::Empty;
        after.hold_expired = false;
        after.recorded = None;
        push_step(found, "owner: P1 and P2a start an attempt", after);
    }
    // R5 (owner decision 3): a request whose client is gone and whose last attempt is unresolved is landed by a fresh attempt. The client
    // is "gone" when its lock is released with no mark and no outcome, which is what a success exit and a kill both look like.
    let clientless: bool = matches!(state.client, Client::Killed | Client::Published);
    if clientless
        && state.mark == Mark::Absent
        && state.attempt < MAX_ATTEMPT
        && !finished
        && state.child == Child::Ended
        && matches!(state.recorded, Some((attempt, Class::Unresolved)) if attempt == state.attempt)
    {
        let mut after: State = state.clone();
        after.attempt = state.attempt + 1;
        after.child = Child::Preparing;
        after.line = Line::Nothing;
        after.pipe = Pipe::Empty;
        after.hold_expired = false;
        after.recorded = None;
        push_step(found, "owner: R5 lands a request whose client is gone with a fresh attempt", after);
    }
    if matches!(state.child, Child::Prepared | Child::Wedged) && state.pipe == Pipe::Empty && !state.hold_expired {
        let mut after: State = state.clone();
        after.pipe = Pipe::Abort;
        after.hold_expired = true;
        push_step(found, "owner: PREPARED-HOLD expires and it writes abort", after);
    }
    let marked_plain: bool = state.mark == Mark::Withdraw;
    if marked_plain && matches!(state.child, Child::Preparing | Child::Prepared | Child::Wedged) && state.pipe == Pipe::Empty {
        let mut after: State = state.clone();
        after.pipe = Pipe::Abort;
        push_step(found, "owner: finds the mark after the reader exists and writes abort", after);
    }
    let client_gone: bool = state.client == Client::Killed || (!thirteen && matches!(state.client, Client::UnresolvedExit(_)));
    if client_gone && state.mark == Mark::Absent && state.child == Child::Prepared && state.pipe == Pipe::Empty {
        let mut after: State = state.clone();
        after.pipe = Pipe::Commit;
        // The decider records `submitted` before it writes (E1's evidence item iv), whoever the decider is.
        after.submitted = state.attempt;
        after.written = true;
        after.write_failed = false;
        after.decided_by_owner = true;
        push_step(found, "owner: the client lock is gone with no mark; it decides and writes commit", after);
    }
    // Planned shutdown (revision 14, the exit audit): the service stops admitting, writes `abort` only for a request with no client (it
    // never decides for a client that holds the lock), records `clean-shutdown` and exits; the child goes on by its own rules.
    if receipts(state.rules) {
        let mut after: State = state.clone();
        let clientless_now: bool = matches!(state.client, Client::Killed | Client::Published);
        if clientless_now && state.child == Child::Prepared && state.pipe == Pipe::Empty {
            after.pipe = Pipe::Abort;
        }
        after.owner = Owner::Dead;
        after.clean_shutdown = true;
        push_step(found, "owner: planned shutdown records clean-shutdown and exits", after);
    }
    classify_steps(state, found);
}

/// E1 as the owner runs it: the tip read can fail, and a recovered owner may be unable to read the result lines.
fn classify_steps(state: &State, found: &mut Vec<(String, State)>) {
    let ended: bool = state.child == Child::Ended;
    let reported: bool = state.child == Child::Reported;
    if state.attempt == 0 || state.recorded.is_some() || !(ended || reported) {
        return;
    }
    let submitted: bool = effective_submission(state);
    let readings: [(Seen, &str); 2] = [
        (observed_tip(state), "owner: E1 reads the tip and classifies the attempt"),
        (Seen::Unread, "owner: E1 cannot read the tip (store error) and classifies from what it has"),
    ];
    for (seen, label) in readings {
        push_class(state, found, label, classify(state.rules, submitted, state.line, seen, ended, state.receipt));
    }
    if state.recovered {
        let class: Class = classify(state.rules, submitted, Line::Nothing, Seen::Unread, ended, state.receipt);
        push_class(state, found, "owner: E1 after a restart finds the result lines unreadable and classifies from the record", class);
    }
}

fn observed_tip(state: &State) -> Seen {
    return match state.tip {
        Tip::New => Seen::Present,
        Tip::Foreign if state.reflog => Seen::Rewritten,
        Tip::Foreign | Tip::Base => Seen::Absent,
    };
}

fn push_class(state: &State, found: &mut Vec<(String, State)>, label: &str, class: Class) {
    let mut after: State = state.clone();
    after.recorded = Some((state.attempt, class));
    if class == Class::Failed {
        after.closed = state.attempt;
        // A late positive failure of the attempt an unresolved exit named: the request will never land, and revision 14 records
        // the outcome `withdrawn` (revision 13 left the request pending for ever).
        if receipts(state.rules) && matches!(state.mark, Mark::Unresolved(attempt) if attempt == state.attempt) {
            after.outcome_withdrawn = true;
        }
    }
    push_step(found, label, after);
}

/// The transaction process's steps: none reads the owner.
fn git_steps(state: &State, found: &mut Vec<(String, State)>) {
    match state.child {
        Child::Preparing => {
            let mut accepted: State = state.clone();
            accepted.child = Child::Prepared;
            push_step(found, "environment: the person's hook returns and accepts; prepare: ok", accepted);
            let mut refused: State = state.clone();
            refused.child = Child::Ended;
            refused.line = Line::HookRefused;
            push_step(found, "environment: the person's hook refuses; update-ref exits non-zero", refused);
            let mut failed: State = state.clone();
            failed.child = Child::Ended;
            failed.line = Line::PrepareFailed;
            push_step(found, "environment: prepare fails in the ref store; update-ref exits non-zero", failed);
        }
        Child::Prepared => {
            if state.owner == Owner::Dead && state.pipe == Pipe::Empty {
                // The owner's keeper write end closed with its death or its shutdown and no writer is open: end of input aborts.
                let mut after: State = state.clone();
                after.pipe = Pipe::Abort;
                push_step(found, "environment: the keeper write end closed and end of input aborts the prepared transaction", after);
            }
            if state.rules == Rules::FourteenSharedChannel && state.line == Line::Nothing {
                let mut forged: State = state.clone();
                forged.line = Line::AbortOk;
                push_step(found, "environment: a hook descendant writes text that looks like abort: ok into the shared output", forged);
            }
            if state.pipe == Pipe::Commit {
                let mut after: State = state.clone();
                after.child = Child::Applying;
                push_step(found, "environment: update-ref reads commit", after);
            }
            if state.pipe == Pipe::Abort {
                let mut after: State = state.clone();
                after.child = Child::Aborting;
                push_step(found, "environment: update-ref reads abort", after);
            }
            let mut wedged: State = state.clone();
            wedged.child = Child::Wedged;
            push_step(found, "environment: update-ref never reads again", wedged);
        }
        Child::Applying => {
            let mut written: State = state.clone();
            written.child = Child::RefWritten;
            written.tip = Tip::New;
            written.reflog = true;
            written.landed = state.attempt;
            push_step(found, "environment: update-ref writes the ref", written);
            let mut failed: State = state.clone();
            failed.child = Child::Ended;
            push_step(found, "environment: the commit fails and update-ref dies with no line on standard output", failed);
        }
        Child::RefWritten => {
            let mut reported: State = state.clone();
            reported.child = Child::Reported;
            reported.line = Line::CommitOk;
            push_step(found, "environment: update-ref prints commit: ok", reported);
            let mut died: State = state.clone();
            died.child = Child::Ended;
            push_step(found, "environment: update-ref dies after writing the ref and before commit: ok", died);
        }
        Child::Aborting => {
            let mut after: State = state.clone();
            after.child = Child::Ended;
            after.line = Line::AbortOk;
            push_step(found, "environment: update-ref prints abort: ok and exits", after);
        }
        Child::Reported => {
            let mut after: State = state.clone();
            after.child = Child::Ended;
            push_step(found, "environment: update-ref exits", after);
        }
        Child::Wedged | Child::NotStarted | Child::Ended => {}
    }
    if child_alive(state.child) {
        let mut after: State = state.clone();
        after.child = Child::Ended;
        push_step(found, "environment: the transaction process is ended (parent-death signal or a kill)", after);
    }
}

/// The client's steps (the ones the register's bound counts) and the client lock's release.
fn client_steps(state: &State, found: &mut Vec<(String, State)>) {
    let thirteen: bool = state.rules != Rules::Twelve;
    match state.client {
        Client::Waiting => {
            let saw_landing: bool = state.line == Line::CommitOk || state.tip == Tip::New;
            if saw_landing && receipts(state.rules) && !state.receipt {
                let mut after: State = state.clone();
                after.receipt = true;
                push_step(found, "client: writes the receipt before it prints published", after);
            }
            if saw_landing && (!receipts(state.rules) || state.receipt) {
                let mut after: State = exit_state(state);
                after.client = Client::Published;
                push_step(found, "client: sees commit: ok or the tip at new, prints published and exits 0", after);
            }
            if state.child == Child::Prepared && state.submitted != state.attempt && state.mark == Mark::Absent {
                let mut after: State = state.clone();
                after.submitted = state.attempt;
                after.written = false;
                after.write_failed = false;
                push_step(found, "client: records submitted for the attempt", after);
            }
            if state.submitted == state.attempt && state.attempt != 0 && !state.written && !state.write_failed {
                if child_alive(state.child) {
                    let mut after: State = state.clone();
                    after.written = true;
                    if after.pipe == Pipe::Empty {
                        after.pipe = Pipe::Commit;
                    }
                    push_step(found, "client: writes commit to the pipe", after);
                } else {
                    let mut after: State = state.clone();
                    after.write_failed = true;
                    push_step(found, "client: the commit write fails (no reader), nothing queued", after);
                }
            }
            if !open_submitted(state) || (state.write_failed && state.submitted == state.attempt) {
                let mut after: State = state.clone();
                after.mark = if state.mark == Mark::Absent { Mark::Withdraw } else { state.mark };
                after.client = Client::Leaving(Kind::Plain);
                push_step(found, "W1: the bound is reached; the client creates the mark withdraw", after);
            }
            if open_submitted(state) && state.written {
                let mut after: State = state.clone();
                if thirteen {
                    after.mark = Mark::Unresolved(state.submitted);
                }
                after.client = Client::Leaving(Kind::Unresolved(state.submitted));
                push_step(found, "COMMITTING: the bound passed with commit submitted; the exit creates its mark", after);
            }
        }
        Client::Leaving(Kind::Plain) => {
            let open_without_line: bool = matches!(state.child, Child::Preparing | Child::Prepared | Child::Wedged) && state.pipe == Pipe::Empty;
            if open_without_line {
                let mut after: State = state.clone();
                after.pipe = Pipe::Abort;
                push_step(found, "W1: writes abort to the open attempt it did not submit", after);
            } else {
                let mut after: State = exit_state(state);
                after.client = Client::Withdrawn;
                push_step(found, "W1: exits non-zero naming the blocker", after);
            }
        }
        Client::Leaving(Kind::Unresolved(attempt)) => {
            let mut after: State = exit_state(state);
            after.client = Client::UnresolvedExit(attempt);
            push_step(found, "COMMITTING: exits non-zero saying the result is not known", after);
        }
        Client::Published | Client::Withdrawn | Client::UnresolvedExit(_) | Client::Killed => {}
    }
    if matches!(state.client, Client::Waiting | Client::Leaving(_)) {
        let mut after: State = state.clone();
        after.client = Client::Killed;
        push_step(found, "environment: the client is killed from outside", after);
    }
}

/// The steps nobody in the protocol causes: the owner's death and another writer's reset.
fn environment_steps(state: &State, found: &mut Vec<(String, State)>) {
    if state.owner == Owner::Alive {
        let mut after: State = state.clone();
        after.owner = Owner::Dead;
        push_step(found, "environment: the owner dies", after);
    }
    if state.tip == Tip::New {
        let mut erased: State = state.clone();
        erased.tip = Tip::Foreign;
        erased.reflog = false;
        push_step(found, "environment: another writer resets the branch away and the reflog entry is lost", erased);
        let mut kept: State = state.clone();
        kept.tip = Tip::Foreign;
        push_step(found, "environment: another writer resets the branch away and the reflog keeps the entry", kept);
    }
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    client_steps(state, &mut found);
    owner_steps(state, &mut found);
    git_steps(state, &mut found);
    environment_steps(state, &mut found);
    return found;
}

/// What C3 answers to a query or a resend: whether it says that nothing can land. A plain mark says so; the unresolved mark must say that
/// the original attempt may still complete (revision 14). Revisions 12 and 13 answered `withdrawn` for either mark.
pub fn c3_says_nothing_can_land(rules: Rules, mark: Mark) -> bool {
    return match mark {
        Mark::Absent => false,
        Mark::Withdraw => true,
        Mark::Unresolved(_) => !receipts(rules),
    };
}

/// Every clause of the owner's contract that `state` breaks, in a fixed order.
pub fn violations(state: &State) -> Vec<String> {
    let mut broken: Vec<String> = Vec::new();
    let left: bool = matches!(state.client, Client::Leaving(_) | Client::Withdrawn | Client::UnresolvedExit(_));
    if left && state.mark == Mark::Absent {
        broken.push("a normal exit left no withdrawal mark".to_string());
    }
    if state.mark == Mark::Withdraw && (state.landed != 0 || commit_pending(state)) {
        broken.push("a request with the withdrawal mark has published or can still publish".to_string());
    }
    match state.client {
        Client::Withdrawn if state.landed != 0 || commit_pending(state) => {
            broken.push("the command said the request was withdrawn and the request landed or can still land".to_string());
        }
        Client::UnresolvedExit(attempt) | Client::Leaving(Kind::Unresolved(attempt)) => {
            if state.attempt > attempt {
                broken.push("an attempt started after the unresolved exit named an earlier one".to_string());
            }
            if state.landed != 0 && state.landed != attempt {
                broken.push("an attempt other than the one the unresolved exit names landed".to_string());
            }
        }
        Client::Published => {
            if state.landed == 0 {
                broken.push("the command said published and nothing landed".to_string());
            }
            if state.attempt > state.exit_attempt {
                broken.push("an attempt started after the command exited published".to_string());
            }
        }
        _ => {}
    }
    let late_failure_unrecorded: bool = matches!(state.mark, Mark::Unresolved(attempt) if state.closed == attempt)
        && matches!(state.client, Client::UnresolvedExit(_))
        && !state.outcome_withdrawn;
    if late_failure_unrecorded {
        broken.push("an unresolved attempt ended in a positive failure and the request stayed pending".to_string());
    }
    if state.decided_by_owner && matches!(state.client, Client::Waiting | Client::Leaving(_)) {
        broken.push("the owner decided a request whose client holds the lock".to_string());
    }
    if c3_says_nothing_can_land(state.rules, state.mark) && matches!(state.mark, Mark::Unresolved(_)) && commit_pending(state) {
        broken.push("a query was answered withdrawn while the original attempt may still complete".to_string());
    }
    if state.closed != 0 && state.landed == state.closed {
        broken.push("an attempt that wrote the ref was closed as positively failed".to_string());
    }
    return broken;
}

/// The owner's contract, as a property of every reachable state: the first clause `violations` finds.
pub fn invariant(state: &State) -> Option<String> {
    return violations(state).into_iter().next();
}

/// The text of revision 12 about PREPARED-HOLD, as a check: after `abort` is enqueued the branch lock is released. It is false
/// when the transaction process never reads again (revision 13 states the residue and claims only the enqueue).
pub fn invariant_twelve_text_lock_released(state: &State) -> Option<String> {
    if state.hold_expired && state.child == Child::Wedged {
        return Some("abort was enqueued and the branch lock is still held by a process that never reads".to_string());
    }
    return invariant(state);
}

/// PREPARED-HOLD's claim as revision 13 states it: while the owner lives and a prepared transaction has no closing line, the
/// owner has its own step that enqueues `abort`.
pub fn hold_waiting(state: &State) -> bool {
    return state.owner == Owner::Alive && matches!(state.child, Child::Prepared | Child::Wedged) && state.pipe == Pipe::Empty;
}

/// The wait the rule bounds: the client is still waiting for its request's outcome.
pub fn waiting(state: &State) -> bool {
    return state.client == Client::Waiting;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 3_000_000, max_depth: 60 };
}

/// Whether a step is one of the waiting client's own (the register counts these; the lane's and the environment's do not).
pub fn is_clients_step(label: &str) -> bool {
    return label.starts_with("W1:") || label.starts_with("COMMITTING:") || label.starts_with("client:");
}

/// Whether a step is the owner's own.
pub fn is_owners_step(label: &str) -> bool {
    return label.starts_with("owner:");
}

/// The bound, in the waiting client's own steps: two attempts at most, each with `records` and `writes`, then the step that leaves.
pub const FOREGROUND_BOUND: usize = 5;

/// The bound, in the owner's own steps, on a prepared transaction without a closing line (one: PREPARED-HOLD).
pub const HOLD_BOUND: usize = 1;

/// Explores every schedule under `rules` with the liveness bound on the client's wait and the contract as the invariant.
pub fn check(rules: Rules) -> Report {
    return explore_bounded_by(vec![initial(rules)], steps, invariant, waiting, is_clients_step, FOREGROUND_BOUND, limits());
}

/// Explores with another invariant (the text of revision 12 about PREPARED-HOLD).
pub fn check_with(rules: Rules, invariant: fn(&State) -> Option<String>) -> Report {
    return explore_bounded_by(vec![initial(rules)], steps, invariant, waiting, is_clients_step, FOREGROUND_BOUND, limits());
}

/// Explores the owner's bound on a prepared transaction: PREPARED-HOLD enqueues `abort` within the owner's own steps.
pub fn check_hold(rules: Rules) -> Report {
    return explore_bounded_by(vec![initial(rules)], steps, invariant, hold_waiting, is_owners_step, HOLD_BOUND, limits());
}
