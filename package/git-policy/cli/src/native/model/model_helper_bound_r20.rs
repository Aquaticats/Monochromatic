//! What:
//!  A comparison model of five ways to give every receipt a continuing executor while bounding the processes a lane keeps alive, on three
//!  clients: the built release, recheck and exit; the reviewer's singleton retry owner; a client that forces its own receipt inline when the
//!  lane is sealed; a client that forces its own receipt inline whenever the admission token is held (with the token kept to exit); and a baton
//!  between the running service and a singleton retry owner that exists only while no service runs. A sixth rule, the token kept to exit with
//!  nobody to retry, is the control that misses. Each run reports the first schedule where a receipt is left with no executor, the most
//!  processes alive at once, and the most synchronisations a client does in the foreground.
//! Why:
//!  The acceptance target's resource bound and "continuing forcing responsibility" gates are in tension (round 17): the token kept to exit
//!  bounds the helpers and misses a receipt when no service runs. The owner picks among protocols, so the model gives each its misses, its
//!  bound and its foreground cost under the same actors: a service with a planned shutdown, a crash and a later start, clients that read the
//!  lane and either ring the service or go to the flusher path, and receipts that are only visible until forced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // report(protocol) = { miss: firstScheduleWithAnUnownedReceipt, maxLive, maxForegroundSyncs }
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};
use std::cell::Cell;

/// The number of clients that each leave one receipt.
pub const CLIENTS: usize = 3;

/// The protocol under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Protocol {
    /// The built protocol: the token is released after the last empty sweep, the helper lists once more and may take it again.
    Built,
    /// The reviewer's singleton retry owner: it holds the token for its life, starts one one-shot worker at a time, and never retires.
    SingletonOwner,
    /// The candidate: while the lane is sealed a client that finds the token held forces its own receipt; otherwise it leaves the receipt
    /// to the running service's syncer. The token is kept to exit.
    SealedInline,
    /// The candidate: a client that finds the token held forces its own receipt inline, whatever the lane's state. The token is kept to exit.
    TokenHeldInline,
    /// The baton: the service starts the singleton retry owner when it exits and the owner retires when a running service has the lane.
    Baton,
    /// The control: the token is kept to exit and a client that finds it held leaves its receipt to nobody but the running service.
    TokenToExit,
}

/// A receipt.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Receipt {
    /// Not created yet.
    Absent,
    /// Visible and not forced.
    V,
    /// Forced.
    D,
}

/// What a client saw when it read the lane.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Seen {
    /// A running service that is not sealed: ring it.
    Ring,
    /// The lane is sealed or the service is shutting down.
    Sealed,
    /// No service runs.
    NoService,
}

/// A client.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Client {
    /// Not started.
    Idle,
    /// Created its receipt.
    Created,
    /// Read the lane.
    Read(Seen),
    /// Finished.
    Done,
}

/// The service.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Service {
    /// Runs; its syncer forces what it finds.
    Running,
    /// Created the seal; the final sweep is next.
    Sealed,
    /// Enumerated the receipts (the bit set).
    Enumerated(u8),
    /// Forced what it enumerated; the exit is next.
    Forced,
    /// Gone.
    Exited,
    /// A next owner removed the seal; its startup sweep is next.
    Starting,
}

/// A one-shot helper or worker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Helper {
    /// Not running.
    Off,
    /// Lists the receipts.
    Enum,
    /// Forces its listing.
    Force(u8),
    /// Listed nothing unforced.
    Empty,
    /// Released the token (the built protocol); the second listing is next.
    Released,
}

/// The singleton retry owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Owner {
    /// None.
    Off,
    /// Alive, holding the token.
    Idle,
}

/// The model's state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The protocol.
    pub protocol: Protocol,
    /// The clients.
    pub clients: [Client; CLIENTS],
    /// The receipts.
    pub receipts: [Receipt; CLIENTS],
    /// The service.
    pub service: Service,
    /// `lane/sealed` exists.
    pub seal: bool,
    /// The one-shot helpers (workers, when an owner exists).
    pub helpers: [Helper; CLIENTS],
    /// The singleton retry owner.
    pub owner: Owner,
    /// The service crashed once.
    pub crashed: bool,
    /// A next owner has started and swept since the crash.
    pub replaced: bool,
    /// The foreground synchronisations each client did.
    pub syncs: [u8; CLIENTS],
}

/// The start state with no service running and no seal: the lane's service never started (or was stopped without a seal), with nothing
/// crashed. Clients that read the lane find no service and not sealed.
pub fn initial_without_service(protocol: Protocol) -> State {
    let mut state: State = initial(protocol);
    state.service = Service::Exited;
    return state;
}

/// Whether the protocol has a singleton retry owner.
fn has_owner(protocol: Protocol) -> bool {
    return matches!(protocol, Protocol::SingletonOwner | Protocol::Baton);
}

/// The start state.
pub fn initial(protocol: Protocol) -> State {
    return State {
        protocol,
        clients: [Client::Idle; CLIENTS],
        receipts: [Receipt::Absent; CLIENTS],
        service: Service::Running,
        seal: false,
        helpers: [Helper::Off; CLIENTS],
        owner: Owner::Off,
        crashed: false,
        replaced: false,
        syncs: [0; CLIENTS],
    };
}

/// Whether the admission token is held.
fn token_held(state: &State) -> bool {
    if has_owner(state.protocol) {
        return state.owner == Owner::Idle;
    }
    return state.helpers.iter().any(|helper| { return matches!(helper, Helper::Enum | Helper::Force(_) | Helper::Empty); });
}

fn visible(receipts: &[Receipt; CLIENTS]) -> u8 {
    let mut mask: u8 = 0;
    for (at, receipt) in receipts.iter().enumerate() {
        if *receipt == Receipt::V {
            mask |= 1 << at;
        }
    }
    return mask;
}

fn force(receipts: &mut [Receipt; CLIENTS], mask: u8) {
    for (at, receipt) in receipts.iter_mut().enumerate() {
        if mask & (1 << at) != 0 && *receipt == Receipt::V {
            *receipt = Receipt::D;
        }
    }
}

/// The processes alive for the lane: helpers and workers, and the owner.
pub fn live(state: &State) -> usize {
    return state.helpers.iter().filter(|helper| { return **helper != Helper::Off; }).count() + usize::from(state.owner == Owner::Idle);
}

/// Every step enabled in `state`.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    for at in 0..CLIENTS {
        client_steps(state, at, &mut found);
    }
    service_steps(state, &mut found);
    helper_steps(state, &mut found);
    return found;
}

fn client_steps(state: &State, at: usize, found: &mut Vec<(String, State)>) {
    match state.clients[at] {
        Client::Idle => {
            let mut after: State = state.clone();
            after.receipts[at] = Receipt::V;
            after.clients[at] = Client::Created;
            push_step(found, &format!("client {}: creates its receipt", at), after);
        }
        Client::Created => {
            let seen: Seen = if state.service == Service::Running && !state.seal {
                Seen::Ring
            } else if state.seal || matches!(state.service, Service::Sealed | Service::Enumerated(_) | Service::Forced) {
                Seen::Sealed
            } else {
                Seen::NoService
            };
            let mut after: State = state.clone();
            after.clients[at] = Client::Read(seen);
            push_step(found, &format!("client {}: reads the seal and whether a service runs", at), after);
        }
        Client::Read(Seen::Ring) => {
            let mut after: State = state.clone();
            after.clients[at] = Client::Done;
            push_step(found, &format!("client {}: rings the service", at), after);
        }
        Client::Read(seen) => {
            let mut after: State = state.clone();
            after.clients[at] = Client::Done;
            if !token_held(state) {
                if has_owner(state.protocol) {
                    after.owner = Owner::Idle;
                } else if let Some(slot) = after.helpers.iter_mut().find(|helper| { return **helper == Helper::Off; }) {
                    *slot = Helper::Enum;
                }
                push_step(found, &format!("client {}: finds the token free and starts the retry owner or the flusher", at), after);
                return;
            }
            let inline: bool = match state.protocol {
                Protocol::TokenHeldInline => true,
                Protocol::SealedInline => seen == Seen::Sealed,
                _ => false,
            };
            if inline {
                force(&mut after.receipts, 1 << at);
                after.syncs[at] += 2;
                push_step(found, &format!("client {}: finds the token held and forces its own receipt inline", at), after);
            } else {
                push_step(found, &format!("client {}: finds the token held and leaves its receipt enrolled", at), after);
            }
        }
        Client::Done => {}
    }
}

fn service_steps(state: &State, found: &mut Vec<(String, State)>) {
    match state.service {
        Service::Running => {
            if visible(&state.receipts) != 0 {
                let mut after: State = state.clone();
                force(&mut after.receipts, visible(&state.receipts));
                push_step(found, "owner: the running syncer sweeps lane/receipt/ and forces what it finds", after);
            }
            let mut sealed: State = state.clone();
            sealed.seal = true;
            sealed.service = Service::Sealed;
            push_step(found, "environment: a planned shutdown creates lane/sealed", sealed);
            if !state.crashed {
                let mut crash: State = state.clone();
                crash.service = Service::Exited;
                crash.crashed = true;
                crash.replaced = false;
                push_step(found, "environment: the service crashes", crash);
            }
            if state.protocol == Protocol::Baton && state.owner == Owner::Idle && state.helpers.iter().all(|helper| { return *helper == Helper::Off; }) && visible(&state.receipts) == 0 {
                let mut retired: State = state.clone();
                retired.owner = Owner::Off;
                push_step(found, "owner: the retry owner retires after the running service acknowledged the transfer", retired);
            }
        }
        Service::Sealed => {
            let mut after: State = state.clone();
            after.service = Service::Enumerated(visible(&state.receipts));
            push_step(found, "owner: the final sweep enumerates lane/receipt/", after);
        }
        Service::Enumerated(mask) => {
            let mut after: State = state.clone();
            force(&mut after.receipts, mask);
            after.service = Service::Forced;
            push_step(found, "owner: the final sweep forces what it enumerated", after);
        }
        Service::Forced => {
            let mut after: State = state.clone();
            after.service = Service::Exited;
            if state.protocol == Protocol::Baton && !token_held(state) {
                after.owner = Owner::Idle;
            }
            push_step(found, "owner: the service hands the baton and exits", after);
        }
        Service::Exited => {
            let mut after: State = state.clone();
            after.seal = false;
            after.service = Service::Starting;
            push_step(found, "environment: a later wrapper command starts a next owner", after);
        }
        Service::Starting => {
            let mut after: State = state.clone();
            for receipt in after.receipts.iter_mut() {
                if *receipt == Receipt::V {
                    *receipt = Receipt::D;
                }
            }
            after.service = Service::Running;
            after.replaced = true;
            push_step(found, "owner: the next owner's syncer sweeps lane/receipt/ at startup", after);
        }
    }
}

fn helper_steps(state: &State, found: &mut Vec<(String, State)>) {
    let polls: bool = has_owner(state.protocol) && state.owner == Owner::Idle && visible(&state.receipts) != 0;
    if polls && state.helpers.iter().all(|helper| { return *helper == Helper::Off; }) {
        let mut after: State = state.clone();
        after.helpers[0] = Helper::Enum;
        push_step(found, "owner: the retry owner polls without waiting and starts one worker", after);
    }
    for slot in 0..CLIENTS {
        let phase: Helper = state.helpers[slot];
        let with = |next: Helper| -> State {
            let mut after: State = state.clone();
            after.helpers[slot] = next;
            return after;
        };
        match phase {
            Helper::Off => {}
            Helper::Enum => {
                let mask: u8 = visible(&state.receipts);
                let next: Helper = if mask != 0 { Helper::Force(mask) } else { Helper::Empty };
                push_step(found, &format!("owner: helper {} lists lane/receipt/", slot), with(next));
            }
            Helper::Force(mask) => {
                let mut after: State = with(Helper::Enum);
                force(&mut after.receipts, mask);
                push_step(found, &format!("owner: helper {} forces its listing", slot), after);
            }
            Helper::Empty => {
                let next: Helper = if state.protocol == Protocol::Built { Helper::Released } else { Helper::Off };
                push_step(found, &format!("owner: helper {} finishes its last listing", slot), with(next));
            }
            Helper::Released => {
                let other: bool = state.helpers.iter().enumerate().any(|(index, helper)| { return index != slot && matches!(helper, Helper::Enum | Helper::Force(_) | Helper::Empty); });
                let next: Helper = if visible(&state.receipts) != 0 && !other { Helper::Enum } else { Helper::Off };
                push_step(found, &format!("owner: helper {} lists once more without the token", slot), with(next));
            }
        }
    }
}

thread_local! {
    /// The most processes alive at once in the search under way.
    static MAX_LIVE: Cell<usize> = const { Cell::new(0) };
    /// The most foreground synchronisations of one client in the search under way.
    static MAX_SYNCS: Cell<u8> = const { Cell::new(0) };
}

/// Records the metrics of every state the search visits; no safety property.
pub fn invariant(state: &State) -> Option<String> {
    MAX_LIVE.with(|cell| { cell.set(cell.get().max(live(state))); });
    MAX_SYNCS.with(|cell| { cell.set(cell.get().max(state.syncs.iter().copied().max().unwrap_or(0))); });
    return None;
}

/// A quiescent state (only the environment can move) must hold no receipt with nothing left to force it. A crash of the service excuses
/// EVERY visible receipt until a next owner has swept, including receipts created after the crash; that is broader than a crash justifies
/// (review 13 section 3) and the comparison is exploration, not evidence for a ranking.
pub fn terminal(state: &State) -> Option<String> {
    let excused: bool = state.crashed && !state.replaced;
    if visible(&state.receipts) != 0 && !excused {
        return Some("a receipt is visible and nothing is left that will force it".to_string());
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 3_000_000, max_depth: 120 };
}

/// What a run found.
#[derive(Debug)]
pub struct Findings {
    /// The first schedule that leaves a receipt with no executor, if any.
    pub miss: Option<Vec<String>>,
    /// The most processes alive for the lane at once. Where `miss` is set the search stopped at it, and this is a lower bound.
    pub max_live: usize,
    /// The most synchronisations one client did in the foreground (a lower bound where `miss` is set; and a synthetic count, not a time).
    pub max_client_syncs: u8,
    /// Whether the search covered every state (otherwise the numbers are lower bounds).
    pub exhausted: bool,
}

/// Explores every schedule under `protocol`.
pub fn report(protocol: Protocol) -> Findings {
    MAX_LIVE.with(|cell| { cell.set(0); });
    MAX_SYNCS.with(|cell| { cell.set(0); });
    let run: Report = explore(vec![initial(protocol), initial_without_service(protocol)], steps, invariant, terminal, limits());
    return Findings {
        miss: run.violation.map(|found| { return found.trace; }),
        max_live: MAX_LIVE.with(Cell::get),
        max_client_syncs: MAX_SYNCS.with(Cell::get),
        exhausted: run.exhausted,
    };
}
