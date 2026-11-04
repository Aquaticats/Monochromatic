//! What:
//!  Model of revision 18's receipt responsibility: two clients that each create `lane/receipt/<id>` (V), read the seal and the lane, and
//!  either ring the running service or start a one-shot flusher; a service whose planned shutdown creates `lane/sealed`, enumerates
//!  `lane/receipt/` once more (the final sweep), forces what it found and exits; a flusher holding `lane/flusher.lock` that sweeps, may
//!  meet a forcing error, and after its last empty sweep releases the lock and sweeps once more; a next owner that removes the seal
//!  (L3) and sweeps at startup; an unrelated command whose dispatch (L4) must not wait for any receipt. Enumeration and forcing are
//!  separate steps, so a receipt created between them is the case the protocol has to cover.
//! Why:
//!  Revision 17 let the client read whether a service held the lane and ring it; a client delayed past the service's final sweep rang a
//!  service that was gone and its receipt was registered with no executor. Revision 18 makes the receipt directory the registration
//!  and orders seal before sweep, so for every receipt either the client sees the seal or the sweep sees the receipt. The check is
//!  quiescence: a state where only the environment can move must hold no receipt that is V with no executor responsible (a forced
//!  disk stall and a launch refused by someone else's cap are the stated outside causes), and no unrelated dispatch still waiting.
//!  Each rule is a parameter with single-defect controls, so a green shows the clause it names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // client: create(receipt) -> read(seal, lane) -> (seal absent && service running ? ring : launchFlusher)
//! // service: seal() -> enumerate() -> force(snapshot) -> exit()
//! ```

use crate::model_explore::{Limits, Report, explore, push_step};

/// The number of clients that each leave one receipt.
pub const CLIENTS: usize = 2;

/// The rule under test.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Rules {
    /// Revision 17: no seal; a client rings a service it saw holding the lane; L4 forces outstanding receipts before dispatching.
    Seventeen,
    /// Revision 18: the seal protocol, the one-shot flusher with a lock and a recheck sweep, a startup sweep, and L4 dispatching without
    /// waiting for receipts.
    Eighteen,
    /// Control: the service seals only after its final sweep.
    EighteenSealAfterSweep,
    /// Control: the flusher exits after its last empty sweep without releasing the lock and sweeping once more.
    EighteenNoRecheck,
    /// Control: no flusher lock; every launch starts another flusher.
    EighteenNoFlusherLock,
    /// Control: the flusher lives in the inline owner's scope and ends when the owner exits.
    EighteenFlusherInOwnerScope,
    /// Control: the next owner does not sweep `lane/receipt/` at startup.
    EighteenNoStartupSweep,
    /// Control: L4 forces outstanding receipts in the foreground before it dispatches an unrelated command.
    EighteenForegroundForcing,
}

/// A receipt's durability.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Receipt {
    /// Not created yet.
    Absent,
    /// Created and visible (V).
    V,
    /// Forced (D).
    D,
    /// A forcing error was recorded in `lane/receipt-errors`; the next owner's syncer retries it.
    Err,
}

/// Where a client is.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Client {
    /// Has not started.
    Idle,
    /// Created its receipt.
    Created,
    /// Read the seal and the lane: `true` means a service can be rung (no seal and a service holds the lane).
    Read(bool),
    /// Finished: rang the service or started or joined a flusher.
    Done,
    /// Finished: no flusher could be started because of a cap set by someone else; the command said the receipt is not yet forced.
    NotYet,
}

/// The service.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Service {
    /// Holds the lane and sweeps.
    Running,
    /// Created the seal; the final sweep is next.
    Sealed,
    /// Enumerated the receipts: the snapshot (one bit per client) is what it will force.
    Enumerated(u8),
    /// Forced its snapshot; the exit (and, in the late-seal control, the seal) is next.
    Forced,
    /// Gone.
    Exited,
    /// A next owner removed the seal; its startup sweep is next.
    Starting,
}

/// The one-shot flusher.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Flusher {
    /// Not running; the lock is free.
    Off,
    /// Holds the lock; the next step enumerates.
    Enum,
    /// Holds the lock; forces the snapshot.
    Force(u8),
    /// Holds the lock after a sweep that found nothing; the release is next.
    Empty,
    /// Released the lock after an empty sweep; one more sweep follows.
    Released,
}

/// Whether an unrelated command's dispatch has happened.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Dispatch {
    /// No unrelated command yet.
    None,
    /// An unrelated command waits at L4.
    Requested,
    /// It dispatched.
    Done,
}

/// The model's state.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct State {
    /// The rule under test.
    pub rules: Rules,
    /// The clients.
    pub clients: [Client; CLIENTS],
    /// The receipts.
    pub receipts: [Receipt; CLIENTS],
    /// `lane/sealed` exists.
    pub seal: bool,
    /// The service.
    pub service: Service,
    /// The flusher.
    pub flusher: Flusher,
    /// Flushers beyond the one the lock allows (only the no-lock control creates them).
    pub extra_flushers: u8,
    /// A client rang the service since its last sweep (a wake-up only).
    pub bell: bool,
    /// Forcing is paused by the disk (an outside condition): no forcing step completes.
    pub stalled: bool,
    /// A next owner has taken over and swept: the outside causes that excused an unforced receipt before it no longer do.
    pub replaced: bool,
    /// The service has crashed once.
    pub crashed: bool,
    /// A forcing error happened once.
    pub errored: bool,
    /// The unrelated command.
    pub dispatch: Dispatch,
}

/// The start state: a running service, no receipts.
pub fn initial(rules: Rules) -> State {
    return State {
        rules,
        clients: [Client::Idle; CLIENTS],
        receipts: [Receipt::Absent; CLIENTS],
        seal: false,
        service: Service::Running,
        flusher: Flusher::Off,
        extra_flushers: 0,
        bell: false,
        stalled: false,
        replaced: false,
        crashed: false,
        errored: false,
        dispatch: Dispatch::Done,
    };
}

/// The start state with an unrelated command that may reach L4 at any point (the dispatch property).
pub fn initial_with_unrelated_command(rules: Rules) -> State {
    let mut state: State = initial(rules);
    state.dispatch = Dispatch::None;
    return state;
}

fn seals(rules: Rules) -> bool {
    return rules != Rules::Seventeen;
}

/// The bit set of receipts that are V.
fn visible(receipts: &[Receipt; CLIENTS]) -> u8 {
    let mut mask: u8 = 0;
    for (at, receipt) in receipts.iter().enumerate() {
        if *receipt == Receipt::V {
            mask |= 1 << at;
        }
    }
    return mask;
}

/// Forces the receipts in `mask` that are still V.
fn force(receipts: &mut [Receipt; CLIENTS], mask: u8) {
    for (at, receipt) in receipts.iter_mut().enumerate() {
        if mask & (1 << at) != 0 && *receipt == Receipt::V {
            *receipt = Receipt::D;
        }
    }
}

/// Starts a flusher for a client's launch: with the lock it is one-shot per lane.
fn launch(state: &mut State) {
    if state.rules == Rules::EighteenNoFlusherLock {
        if state.flusher == Flusher::Off {
            state.flusher = Flusher::Enum;
        } else {
            state.extra_flushers += 1;
        }
        return;
    }
    // The lock is held in Enum and Force: the holder's recheck covers the receipt this client created first. Released has not
    // enumerated again yet, so it covers it too.
    if state.flusher == Flusher::Off {
        state.flusher = Flusher::Enum;
    }
}

/// Every step enabled in `state`. Labels `client:` and `owner:` are protocol steps; `environment:` are outside events.
pub fn steps(state: &State) -> Vec<(String, State)> {
    let mut found: Vec<(String, State)> = Vec::new();
    for c in 0..CLIENTS {
        match state.clients[c] {
            Client::Idle => {
                let mut after: State = state.clone();
                after.receipts[c] = Receipt::V;
                after.clients[c] = Client::Created;
                push_step(&mut found, &format!("client {}: creates its receipt", c), after);
            }
            Client::Created => {
                let mut after: State = state.clone();
                // The lane is held until the service has exited: the final sweep runs under the lane lock.
                let held: bool = state.service != Service::Exited;
                let ring: bool = if seals(state.rules) { !state.seal && held } else { held };
                after.clients[c] = Client::Read(ring);
                push_step(&mut found, &format!("client {}: reads the seal and whether a service holds the lane", c), after);
            }
            Client::Read(true) => {
                let mut after: State = state.clone();
                after.clients[c] = Client::Done;
                after.bell = after.service == Service::Running || after.service == Service::Starting;
                push_step(&mut found, &format!("client {}: rings the service", c), after);
            }
            Client::Read(false) => {
                let mut started: State = state.clone();
                launch(&mut started);
                started.clients[c] = Client::Done;
                push_step(&mut found, &format!("client {}: starts a one-shot flusher", c), started);
                let mut refused: State = state.clone();
                refused.clients[c] = Client::NotYet;
                push_step(&mut found, &format!("environment: a cap set by someone else refuses client {}'s flusher", c), refused);
            }
            Client::Done | Client::NotYet => {}
        }
    }
    service_steps(state, &mut found);
    flusher_steps(state, &mut found);
    if !state.stalled {
        let mut paused: State = state.clone();
        paused.stalled = true;
        push_step(&mut found, "environment: the disk stops answering forcing calls", paused);
    }
    if state.dispatch == Dispatch::None {
        let mut asked: State = state.clone();
        asked.dispatch = Dispatch::Requested;
        push_step(&mut found, "environment: an unrelated command reaches L4", asked);
    }
    if state.dispatch == Dispatch::Requested && state.service == Service::Running {
        let mut ready: bool = true;
        if state.rules == Rules::EighteenForegroundForcing || state.rules == Rules::Seventeen {
            // The foreground forcing of outstanding receipts before dispatch: it waits for each V receipt to be forced.
            ready = visible(&state.receipts) == 0;
        }
        if ready {
            let mut after: State = state.clone();
            after.dispatch = Dispatch::Done;
            push_step(&mut found, "owner: L4 queues the outstanding receipts and dispatches", after);
        }
    }
    if state.service == Service::Running && state.dispatch == Dispatch::Requested && !ready_dispatch(state) && !state.stalled {
        let mut after: State = state.clone();
        force(&mut after.receipts, visible(&state.receipts));
        push_step(&mut found, "owner: L4 forces the outstanding receipts in the foreground", after);
    }
    return found;
}

/// Whether dispatch is not held by receipts under the rule.
fn ready_dispatch(state: &State) -> bool {
    return !(state.rules == Rules::EighteenForegroundForcing || state.rules == Rules::Seventeen) || visible(&state.receipts) == 0;
}

fn service_steps(state: &State, found: &mut Vec<(String, State)>) {
    match state.service {
        Service::Running => {
            if state.bell && !state.stalled {
                let mut after: State = state.clone();
                after.bell = false;
                force(&mut after.receipts, visible(&state.receipts));
                push_step(found, "owner: the syncer wakes, sweeps lane/receipt/ and forces what it finds", after);
            }
            let mut begin: State = state.clone();
            if seals(state.rules) && state.rules != Rules::EighteenSealAfterSweep {
                begin.seal = true;
                begin.service = Service::Sealed;
                push_step(found, "environment: a planned shutdown creates lane/sealed", begin);
            } else {
                begin.service = Service::Sealed;
                push_step(found, "environment: a planned shutdown begins its final sweep", begin);
            }
            if !state.crashed {
                let mut crash: State = state.clone();
                crash.service = Service::Exited;
                crash.crashed = true;
                crash.replaced = false;
                crash.flusher = if state.rules == Rules::EighteenFlusherInOwnerScope { Flusher::Off } else { state.flusher };
                push_step(found, "environment: the service crashes", crash);
            }
        }
        Service::Sealed => {
            let mut after: State = state.clone();
            after.service = Service::Enumerated(visible(&state.receipts));
            push_step(found, "owner: the final sweep enumerates lane/receipt/", after);
        }
        Service::Enumerated(mask) => {
            if !state.stalled {
                let mut after: State = state.clone();
                force(&mut after.receipts, mask);
                after.service = Service::Forced;
                push_step(found, "owner: the final sweep forces what it enumerated", after);
            }
        }
        Service::Forced => {
            let mut after: State = state.clone();
            after.service = Service::Exited;
            if state.rules == Rules::EighteenSealAfterSweep {
                after.seal = true;
            }
            if state.rules == Rules::EighteenFlusherInOwnerScope {
                after.flusher = Flusher::Off;
            }
            push_step(found, "owner: the service exits", after);
        }
        Service::Exited => {
            let mut after: State = state.clone();
            after.seal = false;
            after.service = Service::Starting;
            // An unrelated command waiting at L4 is itself the later command that starts the next owner.
            let label: &str = if state.dispatch == Dispatch::Requested {
                "owner: the unrelated command starts a next owner, which removes lane/sealed (L3)"
            } else {
                "environment: a later wrapper command starts a next owner, which removes lane/sealed (L3)"
            };
            push_step(found, label, after);
        }
        Service::Starting => {
            let mut after: State = state.clone();
            if state.rules != Rules::EighteenNoStartupSweep && !state.stalled {
                for receipt in after.receipts.iter_mut() {
                    if matches!(*receipt, Receipt::V | Receipt::Err) {
                        *receipt = Receipt::D;
                    }
                }
            }
            after.service = Service::Running;
            after.bell = false;
            after.replaced = true;
            for client in after.clients.iter_mut() {
                // A receipt whose flusher could not start is the next owner's to sweep; from here on no outside cause excuses it.
                if *client == Client::NotYet {
                    *client = Client::Done;
                }
            }
            push_step(found, "owner: the next owner's syncer sweeps lane/receipt/ at startup", after);
        }
    }
}

fn flusher_steps(state: &State, found: &mut Vec<(String, State)>) {
    match state.flusher {
        Flusher::Off => {}
        Flusher::Enum => {
            let mut after: State = state.clone();
            let mask: u8 = visible(&state.receipts);
            if mask != 0 {
                after.flusher = Flusher::Force(mask);
            } else {
                after.flusher = Flusher::Empty;
            }
            push_step(found, "owner: the flusher enumerates lane/receipt/", after);
        }
        Flusher::Force(mask) => {
            if !state.stalled {
                let mut after: State = state.clone();
                force(&mut after.receipts, mask);
                after.flusher = Flusher::Enum;
                push_step(found, "owner: the flusher forces its snapshot", after);
                if !state.errored {
                    let mut errored: State = state.clone();
                    for (at, receipt) in errored.receipts.iter_mut().enumerate() {
                        if mask & (1 << at) != 0 && *receipt == Receipt::V {
                            *receipt = Receipt::Err;
                        }
                    }
                    errored.errored = true;
                    errored.replaced = false;
                    errored.flusher = Flusher::Off;
                    push_step(found, "environment: a forcing error is recorded in lane/receipt-errors and the flusher exits", errored);
                }
            }
        }
        Flusher::Empty => {
            let mut after: State = state.clone();
            after.flusher = if state.rules == Rules::EighteenNoRecheck { Flusher::Off } else { Flusher::Released };
            push_step(found, "owner: the flusher releases the lock after its last empty sweep", after);
        }
        Flusher::Released => {
            let mut after: State = state.clone();
            let mask: u8 = visible(&state.receipts);
            after.flusher = if mask == 0 { Flusher::Off } else { Flusher::Enum };
            push_step(found, "owner: the flusher sweeps once more without the lock and takes it again if it finds work", after);
        }
    }
}

/// Safety: at most one flusher per lane.
pub fn invariant(state: &State) -> Option<String> {
    if state.extra_flushers > 0 {
        return Some("more than one flusher runs for the lane".to_string());
    }
    return None;
}

/// A quiescent state (only the environment can move) is acceptable only if no receipt is V with nothing responsible for it, and no
/// unrelated command is still waiting at L4. The stated outside causes excuse a stalled disk, a crash of the service or a forcing error (the next owner sweeps
/// `lane/receipt/` at startup and retries what was recorded) and a launch refused by someone else's cap.
pub fn terminal(state: &State) -> Option<String> {
    // A command that arrives while the service is in its shutdown sweep waits for the lane, and that wait is the register's ACQUIRE entry,
    // bounded by its own deadline; only a command that reached L4 on a running service is the property here.
    let lane_busy_with_shutdown: bool = matches!(state.service, Service::Sealed | Service::Enumerated(_) | Service::Forced);
    if state.dispatch == Dispatch::Requested && !lane_busy_with_shutdown {
        return Some("an unrelated command's dispatch is still waiting for receipts".to_string());
    }
    let excused_before_replacement: bool = state.crashed || state.errored;
    if !state.stalled && (state.replaced || !excused_before_replacement) {
        for c in 0..CLIENTS {
            if state.receipts[c] == Receipt::V && state.clients[c] != Client::NotYet {
                return Some(format!("client {}'s receipt is V and no executor is responsible for it", c));
            }
        }
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 400_000, max_depth: 80 };
}

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, terminal, limits());
}

/// Explores every schedule under `rules` with an unrelated command that may reach L4 at any point.
pub fn check_dispatch(rules: Rules) -> Report {
    return explore(vec![initial_with_unrelated_command(rules)], steps, invariant, terminal, limits());
}
