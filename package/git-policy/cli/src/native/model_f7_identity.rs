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
use crate::model_explore::{Limits, Report, explore};

/// What:
///  `enum Rules { Six }` is a closed set of named choices (TS: a union of string literals).
///  The model runs under the revision-6 rule; the revision-7 rule is a second variant added with its fix.
/// Why:
///  The rule revision lives in the state so the explorer's function pointers need no captured variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rules = 'six';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 6: check the places, write a temporary record, rename it over `inbox/<id>`.
    Six,
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
/// type Phase = 'start' | 'checked' | 'temp' | 'published' | 'acked' | 'rejected' | 'resent';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Phase {
    /// Nothing done yet.
    Start,
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
    /// `inbox/<id>`.
    pub inbox: Slot,
    /// `queue/<id>`, where adoption moves the record.
    pub queue: Slot,
    /// The permanent ledger line of the identity.
    pub ledger: Slot,
}

/// A slot with nothing in it.
const EMPTY: Slot = Slot { hash: 0, durable: true };

/// The names used in step labels, by client index.
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
    return State { rules, clients: [a, b], inbox: EMPTY, queue: EMPTY, ledger: EMPTY };
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
    let mut found: Vec<(String, State)> = Vec::new();
    for who in 0..2 {
        let name: &str = NAMES[who];
        let client: Client = state.clients[who];
        if client.phase == Phase::Start {
            let known: u8 = recorded(state);
            let mut after: State = state.clone();
            if known == 0 {
                after.clients[who].phase = Phase::Checked;
                found.push((format!("{}: reads the inbox, the queue and the ledger and finds no record", name), after));
            } else if known == client.hash {
                after.clients[who].phase = Phase::Resent;
                found.push((format!("{}: finds its own content recorded and answers from the record", name), after));
            } else {
                after.clients[who].phase = Phase::Rejected;
                found.push((format!("{}: finds a different content recorded and is refused", name), after));
            }
        }
        if client.phase == Phase::Checked {
            let mut after: State = state.clone();
            after.clients[who].phase = Phase::Temp;
            found.push((format!("{}: writes its record under a temporary name", name), after));
        }
        if client.phase == Phase::Temp {
            // The revision-6 publication: a rename over `inbox/<id>`, which replaces whatever is there.
            let mut after: State = state.clone();
            after.inbox = Slot { hash: client.hash, durable: true };
            after.clients[who].phase = Phase::Published;
            found.push((format!("{}: renames its record over inbox/<id>", name), after));
        }
        if client.phase == Phase::Published {
            let mut after: State = state.clone();
            after.clients[who].phase = Phase::Acked;
            found.push((format!("{}: answers that the request is handed over", name), after));
        }
    }
    if state.inbox.hash != 0 {
        // Adoption moves the record from `inbox/<id>` to `queue/<id>` with a rename, which replaces a record there.
        let mut after: State = state.clone();
        after.queue = state.inbox;
        after.inbox = EMPTY;
        found.push(("lane: adopts the record, moving inbox/<id> to queue/<id>".to_string(), after));
    }
    if state.queue.hash != 0 {
        // Retirement appends the terminal line; the first line of an identity is the one a later lookup reads.
        let mut after: State = state.clone();
        if after.ledger.hash == 0 {
            after.ledger = state.queue;
        }
        after.queue = EMPTY;
        found.push(("lane: retires the request, appending its ledger line".to_string(), after));
    }
    return found;
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
    let mut content: u8 = 0;
    for slot in [state.inbox, state.queue, state.ledger] {
        if slot.hash == 0 {
            continue;
        }
        if content != 0 && content != slot.hash {
            return Some("two different contents are recorded for one identity".to_string());
        }
        content = slot.hash;
    }
    for client in state.clients {
        if client.phase == Phase::Acked && client.hash != content {
            return Some("an acknowledged request is not the recorded content of its identity".to_string());
        }
    }
    return None;
}

/// A state with no enabled step is an acceptable end here: every client finished or was refused.
pub fn acceptable_end(_state: &State) -> Option<String> {
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
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, acceptable_end, limits());
}
