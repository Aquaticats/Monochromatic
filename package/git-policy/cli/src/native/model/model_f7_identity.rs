//! Historical control, not approval evidence for revision 13: the binding file and the global directory counter are retired; EEXIST on the inbox file is reopened in model_barrier_r13.
//!
//! What:
//!  Model of the sixth review's finding F7: two clients accept a request under the same identity with different
//!  content, one before the lane adopts the first and one after.
//! Why:
//!  "Durable acceptance needs no ref write" (`doc/planning/cli-git-commits-never-fail.md`) promises that an accepted
//!  request is immutable and that an identity is bound to its content for good.
//!  The model holds the places a record can be (the inbox, the queue and the permanent ledger) and checks that an
//!  acknowledged request is still the recorded content and that no identity carries two contents.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type State = { rules: Rules; clients: [Client, Client]; inbox: Slot; queue: Slot; ledger: Slot };
//! ```

// What:     `use crate::model_explore::{...}` imports the explorer's names from the sibling module declared in the
//           test root. `explore` runs a search, `Limits` caps it, `Report` is its result.
// Why:      The model hands its steps and checks to the explorer.
//
// In TS you'd write (pseudocode):
// ```ts
// import { explore, Limits, Report } from './model_explore';
// ```
use crate::model_explore::{Limits, Report, explore, push_step};

/// What:
///  `enum Rules { ... }` is a closed set of named choices (TS: a union of string literals).
///  The model runs under the revision-6 rule and under the revision-7 rule, and under two mutations of the
///  revision-7 rule that each leave out one fsync.
/// Why:
///  The rule revision lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'six' | 'seven' | 'candidate' | 'eight' | 'nine' | 'candidate-no-binding-sync' | 'candidate-no-record-sync';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: check the places, write a temporary record, rename it over `inbox/<id>`.
    Six,
    /// Revision 7 (the seventh review's target): create `ids/<xx>/<id>` with `O_EXCL`, write the hash into it, fsync
    /// it, write the record, publish it with a no-replace link, fsync `inbox/` and answer; a same-hash binding
    /// without a record is answered "from the record or the ledger", which cannot be done.
    Seven,
    /// A candidate that answers the seventh review's required changes: the complete binding is published atomically
    /// (a temporary file linked without replacing), and a same-hash binding without a record continues acceptance.
    /// It is a proposal of this model, not a rule of the design.
    Candidate,
    /// Revision 8 draft, C3 and C4: the complete binding is linked, a same-hash resend that finds no inbox record completes
    /// acceptance, and one that finds a record answers from its state (without making it durable).
    Eight,
    /// Revision 9, C2 to C4: each writer spools into a private candidate directory (exclusive mkdir with a random nonce), the
    /// binding routes by the identity's state, and a resend that finds a visible record redoes the fsyncs before answering.
    Nine,
    /// Revision 9 plus one correction found by the model: a no-replace link that fails because the name exists also redoes the
    /// inbox fsyncs before answering (a proposal, not revision 9's text).
    NineFsyncOnExists,
    /// The candidate with the fsync of the binding left out (a mutation that must stay red).
    CandidateNoBindingSync,
    /// The candidate with the fsync of the published record left out (a mutation that must stay red).
    CandidateNoRecordSync,
}

/// What:
///  `struct Slot { ... }` is one place a record of the identity can be: the hash of the immutable request in it,
///  or 0 when nothing is there, and whether it would survive a power loss.
///  `u8` is an unsigned 8-bit integer (siblings: `u32`, `usize`); `bool` is TS `boolean`.
/// Why:
///  `u8` and not `usize` because a hash here is a label of one of two requests, and a small `Copy` type keeps the
///  state cheap to clone and hash; 0 for absent avoids an `Option` in every slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Slot = { hash: number; durable: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Slot {
    /// The hash of the immutable request recorded here, 0 for an empty slot.
    pub hash: u8,
    /// Whether the record has been made durable.
    pub durable: bool,
}

/// What:
///  `enum Phase { ... }` is how far a client got.
/// Why:
///  A client is a short script of steps; the phase says which step is next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Phase = 'start' | 'bound' | 'linked' | 'checked' | 'temp' | 'published' | 'acked' | 'rejected' | 'resent' | 'crashed';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Phase {
    /// Nothing done yet.
    Start,
    /// Revision 8: writing the spool into the shared `incoming/<id>.<hash>` directory.
    Writing,
    /// Revisions 8 and 9: the candidate directory is complete and synced (C2 done).
    Cand,
    /// Revision 7: the binding file exists but its content is not yet written (creation is visible before content).
    Creating,
    /// The binding is complete but not yet made durable.
    Bound,
    /// The client met a binding it cannot act on (partial, or its own with no record to answer from) and is stuck.
    Stuck,
    /// Revision 7: the record is linked into `inbox/<id>` but not yet made durable.
    Linked,
    /// The places were read and held no record of the identity.
    Checked,
    /// The client's record is written under a temporary name.
    Temp,
    /// The record is in `inbox/<id>`.
    Published,
    /// The client was told the request is handed over.
    Acked,
    /// The client found a different content under its identity and was refused.
    Rejected,
    /// The client found its own content recorded and was answered from the record.
    Resent,
    /// The client process died; whatever it had written stays.
    Crashed,
}

/// What:
///  `struct Client { ... }`: the content hash this client sends and how far it got.
/// Why:
///  Two clients with one identity and two contents are the whole scenario.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Client = { hash: number; phase: Phase };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Client {
    /// The hash of the request this client sends.
    pub hash: u8,
    /// How far the client got.
    pub phase: Phase,
}

/// What:
///  `struct State { ... }`: the rule revision, the two clients (`[Client; 2]` is a fixed-size array; siblings:
///  `Vec<Client>`, `&[Client]`) and the three places a record can be.
/// Why:
///  A fixed array and not a `Vec` because the scenario has exactly two clients.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = { rules: Rules; clients: [Client, Client]; inbox: Slot; queue: Slot; ledger: Slot };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// Client A (index 0, hash 1) and client B (index 1, hash 2).
    pub clients: [Client; 2],
    /// How many writers are inside one candidate directory at once (revision 8 shares it between same-content writers).
    pub cand_writers: u8,
    /// `ids/<xx>/<id>`: the permanent binding of the identity to a content hash (revision 7 only).
    pub binding: Slot,
    /// `inbox/<id>`.
    pub inbox: Slot,
    /// `queue/<id>`, where adoption moves the record.
    pub queue: Slot,
    /// The permanent ledger line of the identity.
    pub ledger: Slot,
}

/// The hash value of a binding that exists but whose content is not written yet.
const PARTIAL: u8 = 255;

/// A slot with nothing in it.
const EMPTY: Slot = Slot { hash: 0, durable: true };

/// What:
///  `const NAMES: [&str; 2]` is a fixed-size array of two borrowed texts, read as `NAMES[who]`.
///  Sibling: `Vec<&str>`, which could grow.
/// Why:
///  The labels of a schedule name the client; a constant array needs no allocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NAMES = ['A', 'B'] as const;
/// ```
const NAMES: [&str; 2] = ["A", "B"];

/// What:
///  The start state under `rules`: nothing recorded, both clients at the start.
/// Why:
///  Every schedule begins here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function initial(rules: Rules): State;
/// ```
pub fn initial(rules: Rules) -> State {
    let a: Client = Client { hash: 1, phase: Phase::Start };
    let b: Client = Client { hash: 2, phase: Phase::Start };
    return State { rules, clients: [a, b], cand_writers: 0, binding: EMPTY, inbox: EMPTY, queue: EMPTY, ledger: EMPTY };
}

/// What:
///  The hash recorded in the ledger, else the queue, else the inbox, or 0 when no place holds a record.
/// Why:
///  A client that looks for the identity reads all three places.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recorded(s: State): number;
/// ```
fn recorded(state: &State) -> u8 {
    if state.ledger.hash != 0 {
        return state.ledger.hash;
    }
    if state.queue.hash != 0 {
        return state.queue.hash;
    }
    return state.inbox.hash;
}

/// What:
///  Every step enabled in `state`: each client's next step, the lane's adoption and the lane's retirement.
///  `format!("{}: ...", name)` builds an owned `String` with the name in the braces (a standard macro, called like a function).
/// Why:
///  The explorer takes every one of these steps in every order, which is every schedule of the two clients and the lane.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(s: State): [string, State][];
/// ```
pub fn steps(state: &State) -> Vec<(String, State)> {
    // What:     `let mut found: Vec<(String, State)> = Vec::new();` makes a changeable (`mut`) variable typed as a
    //           growable list of pairs (a tuple is `(String, State)`, like a TS `[string, State]`), starting empty.
    // Why:      Collect every enabled step as `(label, next state)`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found: [string, State][] = [];
    // ```
    let mut found: Vec<(String, State)> = Vec::new();
    for who in 0..2 {
        // What:     `found.extend(list)` appends every entry of another list; `client_steps(state, who)` returns
        //           a fresh `Vec` and `&` lends the state read-only.
        // Why:      Add this client's enabled steps to the lane's.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // found.push(...clientSteps(state, who));
        // ```
        found.extend(client_steps(state, who));
    }
    if state.inbox.hash != 0 {
        // Adoption moves the record from `inbox/<id>` to `queue/<id>` with a rename, which replaces a record there.
        let mut after: State = state.clone();
        // What:     `let mut after: State = state.clone();` copies the whole state (`.clone()` makes an independent
        //           copy; `state` is only borrowed here) so the step can change the copy.
        // Why:      Each step is a new state; the old one stays the explorer's record.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const after = structuredClone(state);
        // ```
        // The lane fsyncs what it writes, so the queue entry is durable.
        after.queue = Slot { hash: state.inbox.hash, durable: true };
        after.inbox = EMPTY;
        // What:     `"...".to_string()` turns the borrowed literal into an owned `String`; `found.push((a, b))`
        //           appends the pair.
        // Why:      The step list owns its labels.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // found.push(['lane: adopts the record, moving inbox/<id> to queue/<id>', after]);
        // ```
        found.push(("lane: adopts the record, moving inbox/<id> to queue/<id>".to_string(), after));
    }
    if state.queue.hash != 0 {
        // Retirement appends the terminal line; the first line of an identity is the one a later lookup reads.
        let mut after: State = state.clone();
        if after.ledger.hash == 0 {
            after.ledger = Slot { hash: state.queue.hash, durable: true };
        }
        after.queue = EMPTY;
        found.push(("lane: retires the request, appending its ledger line".to_string(), after));
    }
    if state.rules != Rules::Six && has_volatile(state) {
        // A power loss drops what was never fsynced and ends every client that was not told an answer.
        found.push(("power loss: unsynced files and directory entries are gone".to_string(), after_power_loss(state)));
    }
    return found;
}

/// What:
///  Every step of client `who` in `state`, under the rule the state names.
/// Why:
///  The two revisions differ only here, so the lane's steps stay shared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clientSteps(s: State, who: number): [string, State][];
/// ```
fn client_steps(state: &State, who: usize) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    // What:     `NAMES[who]` reads entry `who` of the array of borrowed texts (`&str`, sibling: owned `String`).
    // Why:      Name the client in the step labels.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name = NAMES[who];
    // ```
    let name: &str = NAMES[who];
    let client: Client = state.clients[who];
    let seven: bool = state.rules != Rules::Six;
    let candidate: bool = state.rules != Rules::Six && state.rules != Rules::Seven;
    let c2_first: bool = state.rules == Rules::Eight || state.rules == Rules::Nine || state.rules == Rules::NineFsyncOnExists;
    if client.phase == Phase::Start && c2_first {
        let mut after: State = state.clone();
        if state.rules == Rules::Eight {
            after.cand_writers += 1;
            after.clients[who].phase = Phase::Writing;
            push_step(&mut found, &format!("{}: C2 spool: writes the spool and record into the shared incoming/<id>.<hash> directory", name), after);
        } else {
            after.clients[who].phase = Phase::Cand;
            // What:     `&mut x` lends `x` to a function for change: the callee may modify it and the caller sees the change. Sibling: `&x`, a read-only loan.
            // Why:      Let a helper append to the caller's list without returning a new one.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // objects and arrays are passed by reference already
            // ```
            push_step(&mut found, &format!("{}: C2 spool: writes the spool and record into its private candidate directory and fsyncs it", name), after);
        }
    }
    if client.phase == Phase::Writing {
        let mut after: State = state.clone();
        after.cand_writers -= 1;
        after.clients[who].phase = Phase::Cand;
        push_step(&mut found, &format!("{}: C2 spool: fsyncs the shared candidate directory", name), after);
    }
    if client.phase == Phase::Start && !seven {
        let known: u8 = recorded(state);
        let mut after: State = state.clone();
        if known == 0 {
            after.clients[who].phase = Phase::Checked;
            // What:     `format!("{}: ...", name)` builds an owned `String`, putting `name` where `{}` is
            //           (a standard macro, called like a function; `.push((label, after))` appends the pair).
            // Why:      Label the step with the client's name.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // found.push([`${name}: reads the inbox, the queue and the ledger and finds no record`, after]);
            // ```
            found.push((format!("{}: reads the inbox, the queue and the ledger and finds no record", name), after));
        } else if known == client.hash {
            after.clients[who].phase = Phase::Resent;
            found.push((format!("{}: finds its own content recorded and answers from the record", name), after));
        } else {
            after.clients[who].phase = Phase::Rejected;
            found.push((format!("{}: finds a different content recorded and is refused", name), after));
        }
    }
    if ((client.phase == Phase::Start && !c2_first) || client.phase == Phase::Cand) && seven {
        // `O_EXCL` creation of `ids/<xx>/<id>`: exactly one client creates it, the others read it.
        let mut after: State = state.clone();
        if state.binding.hash == 0 && !candidate {
            // Creation is visible before the content is written.
            after.binding = Slot { hash: PARTIAL, durable: false };
            after.clients[who].phase = Phase::Creating;
            found.push((format!("{}: creates ids/<xx>/<id> with O_EXCL, still empty", name), after));
        } else if state.binding.hash == 0 {
            // The candidate links a fully written temporary file without replacing: nothing partial is visible.
            after.binding = Slot { hash: client.hash, durable: false };
            after.clients[who].phase = Phase::Bound;
            found.push((format!("{}: links a complete binding ids/<xx>/<id> without replacing", name), after));
        } else if state.binding.hash == PARTIAL {
            after.clients[who].phase = Phase::Stuck;
            found.push((format!("{}: finds a binding with no content", name), after));
        } else if state.binding.hash == client.hash && recorded(state) == client.hash {
            after.clients[who].phase = Phase::Resent;
            if candidate && state.rules != Rules::CandidateNoRecordSync && state.rules != Rules::Eight && state.inbox.hash == client.hash {
                // The record found may be visible but not durable: the resend makes it durable before answering.
                after.inbox.durable = true;
                found.push((format!("{}: finds its own hash and record, fsyncs the record and answers from it", name), after));
            } else {
                found.push((format!("{}: finds its own hash in the binding and answers from the record", name), after));
            }
        } else if state.binding.hash == client.hash && candidate {
            // The binding is this client's own and no record exists (the earlier attempt died before its record was
            // durable): acceptance continues from the binding step.
            after.clients[who].phase = Phase::Bound;
            found.push((format!("{}: finds its own hash in the binding and no record, and continues", name), after));
        } else if state.binding.hash == client.hash {
            after.clients[who].phase = Phase::Stuck;
            found.push((format!("{}: finds its own hash in the binding and no record to answer from", name), after));
        } else {
            after.clients[who].phase = Phase::Rejected;
            found.push((format!("{}: finds a different hash in the binding and is refused", name), after));
        }
    }
    if client.phase == Phase::Creating {
        let mut after: State = state.clone();
        after.binding.hash = client.hash;
        after.clients[who].phase = Phase::Bound;
        found.push((format!("{}: writes the hash into the binding", name), after));
    }
    if client.phase == Phase::Bound {
        let mut after: State = state.clone();
        if state.rules == Rules::CandidateNoBindingSync {
            // Mutation: the binding is not fsynced before the record is written.
            after.clients[who].phase = Phase::Checked;
            found.push((format!("{}: goes on without syncing the binding", name), after));
        } else {
            after.binding.durable = true;
            after.clients[who].phase = Phase::Checked;
            found.push((format!("{}: fsyncs the binding file and its directory", name), after));
        }
    }
    if client.phase == Phase::Checked {
        let mut after: State = state.clone();
        after.clients[who].phase = Phase::Temp;
        found.push((format!("{}: writes its record under a temporary name", name), after));
    }
    if client.phase == Phase::Temp && !seven {
        // The revision-6 publication: a rename over `inbox/<id>`, which replaces whatever is there.
        let mut after: State = state.clone();
        after.inbox = Slot { hash: client.hash, durable: true };
        after.clients[who].phase = Phase::Published;
        found.push((format!("{}: renames its record over inbox/<id>", name), after));
    }
    if client.phase == Phase::Temp && seven {
        // The revision-7 publication: a link that fails when `inbox/<id>` exists and never replaces it.
        let mut after: State = state.clone();
        if state.inbox.hash == 0 {
            after.inbox = Slot { hash: client.hash, durable: false };
            after.clients[who].phase = Phase::Linked;
            found.push((format!("{}: links its record into inbox/<id> without replacing", name), after));
        } else {
            // What:     `if c { a } else { b }` used as a value, like TS `c ? a : b`.
            // Why:      Answer from the record when it is the client's own content, else refuse.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // after.clients[who].phase = inbox.hash === client.hash ? 'resent' : 'rejected';
            // ```
            if state.rules == Rules::NineFsyncOnExists {
                after.inbox.durable = true;
            }
            after.clients[who].phase = if state.inbox.hash == client.hash { Phase::Resent } else { Phase::Rejected };
            found.push((format!("{}: the link fails because inbox/<id> exists", name), after));
        }
    }
    if client.phase == Phase::Linked {
        let mut after: State = state.clone();
        after.clients[who].phase = Phase::Published;
        if state.rules == Rules::CandidateNoRecordSync {
            // Mutation: the record is answered for without fsyncing `inbox/` and its parent.
            found.push((format!("{}: goes on without syncing inbox/", name), after));
        } else {
            after.inbox.durable = true;
            found.push((format!("{}: fsyncs the record, inbox/ and its parent", name), after));
        }
    }
    let alive_midway: bool = client.phase == Phase::Writing
        || client.phase == Phase::Cand
        || client.phase == Phase::Creating
        || client.phase == Phase::Bound
        || client.phase == Phase::Checked
        || client.phase == Phase::Temp
        || client.phase == Phase::Linked
        || client.phase == Phase::Published;
    if alive_midway {
        // A client process can die between any two of its steps.
        let mut after: State = state.clone();
        if client.phase == Phase::Writing {
            after.cand_writers -= 1;
        }
        after.clients[who].phase = Phase::Crashed;
        found.push((format!("{}: the process dies", name), after));
    }
    if client.phase == Phase::Crashed {
        // A client that died resends later with the same identity and content.
        let mut after: State = state.clone();
        after.clients[who].phase = Phase::Start;
        found.push((format!("{}: resends the same request", name), after));
    }
    if client.phase == Phase::Published {
        let mut after: State = state.clone();
        after.clients[who].phase = Phase::Acked;
        found.push((format!("{}: answers that the request is handed over", name), after));
    }
    return found;
}

/// Whether some file or directory entry has not been made durable.
fn has_volatile(state: &State) -> bool {
    return !state.binding.durable || !state.inbox.durable;
}

/// What:
///  The state after a power loss: unsynced entries vanish and every client without a final answer starts over
///  (a client that was never told retries with the same identity and content).
/// Why:
///  This is the model's fsync ordering: a step that is not synced when the power goes is not there afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function afterPowerLoss(s: State): State;
/// ```
fn after_power_loss(state: &State) -> State {
    let mut after: State = state.clone();
    if !after.binding.durable {
        after.binding = EMPTY;
    }
    if !after.inbox.durable {
        after.inbox = EMPTY;
    }
    // `iter_mut()` lends each client for change in turn (`&mut` is a borrow that may modify).
    for client in after.clients.iter_mut() {
        let phase: Phase = client.phase;
        if phase != Phase::Acked && phase != Phase::Rejected && phase != Phase::Resent && phase != Phase::Crashed {
            client.phase = Phase::Start;
        }
    }
    after.cand_writers = 0;
    return after;
}

/// What:
///  The safety check: an identity carries one content across the inbox, the queue and the ledger, and every
///  acknowledged request is that content.
/// Why:
///  An acknowledged request that is not the recorded content was replaced or lost, which "immutable" forbids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function invariant(s: State): string | undefined;
/// ```
pub fn invariant(state: &State) -> Option<String> {
    if state.cand_writers > 1 {
        return Some("two writers write into one candidate directory".to_string());
    }
    let mut content: u8 = 0;
    // What:     `for slot in [a, b, c]` loops over a three-entry array, giving each `Slot` by value (`Slot` is `Copy`,
    //           a small type duplicated on use, so no `&` is needed).
    // Why:      Compare the content in every place a record can be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const slot of [state.inbox, state.queue, state.ledger]) {
    // ```
    for slot in [state.inbox, state.queue, state.ledger] {
        if slot.hash == 0 {
            continue;
        }
        if content != 0 && content != slot.hash {
            // What:     `Some(text)` is the present case of `Option`; the check returns it to say "violated, because ...".
            // Why:      Tell the explorer which property failed.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return 'two different contents are recorded for one identity';
            // ```
            return Some("two different contents are recorded for one identity".to_string());
        }
        content = slot.hash;
    }
    if state.binding.hash != 0 && state.binding.hash != PARTIAL && content != 0 && state.binding.hash != content {
        return Some("the binding of the identity names a different content than its record".to_string());
    }
    for client in state.clients {
        if client.phase == Phase::Resent && client.hash != content {
            return Some("a client was answered from a record that does not exist".to_string());
        }
        if client.phase == Phase::Acked && client.hash != content {
            return Some("an acknowledged request is not the recorded content of its identity".to_string());
        }
    }
    // What:     `None` is the absent case of `Option`: no property failed.
    // Why:      Tell the explorer the state is acceptable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return undefined;
    // ```
    return None;
}

/// A state with no enabled step is an acceptable end unless a client is stuck on a binding it cannot act on.
pub fn acceptable_end(state: &State) -> Option<String> {
    for client in state.clients {
        if client.phase == Phase::Stuck {
            return Some("an identity is reserved with no recoverable request and a client cannot go on".to_string());
        }
    }
    return None;
}

/// The caps for this model: a few hundred states at most.
pub fn limits() -> Limits {
    return Limits { max_states: 100_000, max_depth: 64 };
}

/// What:
///  Explores every schedule under `rules` and returns the explorer's report.
/// Why:
///  The tests call this one function for each rule revision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function check(rules: Rules): Report;
/// ```
pub fn initial_same_content(rules: Rules) -> State {
    let mut state: State = initial(rules);
    state.clients[1].hash = 1;
    return state;
}

/// Explores every schedule under `rules`; revisions 8 and 9 also with both clients sending the same content.
pub fn check(rules: Rules) -> Report {
    let mut starts: Vec<State> = vec![initial(rules)];
    if rules == Rules::Eight || rules == Rules::Nine || rules == Rules::NineFsyncOnExists {
        starts.push(initial_same_content(rules));
    }
    return explore(starts, steps, invariant, acceptable_end, limits());
}
