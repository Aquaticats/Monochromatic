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
    /// Control: a client whose enqueue a running service rejects treats the receipt as handed over and starts no flusher.
    EighteenIgnoresRejection,
    /// Control: a flusher that meets a forcing error lists and records the receipts of its snapshot only, not the receipts that joined it,
    /// and does not list the lane again after it releases the lock.
    EighteenErrorIgnoresJoiners,
    /// Control: the error handler lists the lane when the error is met, records that listing, releases the lock and exits without
    /// listing again, so a receipt that joins between its listing and its release has no executor and no record (review 9b item 1).
    EighteenErrorNoRelist,
    /// Control: the flusher lives in the incarnation scope that the next owner's L2 reaps, and the next owner does not sweep at startup.
    EighteenReapedFlusherNoStartupSweep,
    /// Review 11's proposal: the admission token is kept until the flusher exits (no release, no recheck), a client that finds it held
    /// starts nothing and relies on the continuing retry of the running owner's syncer, which also forces receipts that are still V.
    EighteenTokenToExitWithSyncerRetry,
    /// The proposal with the retry owner started by the client itself: a client that finds the token held, and no service running, starts
    /// the next owner (L3) before it returns, so the continuing retry exists.
    EighteenTokenToExitClientStartsOwner,
    /// Control: the running owner's syncer never retries a recorded error; only a next owner's start does, so a record written while an
    /// owner runs can wait without end (the coordinator pick of round 15 makes the retry the rule).
    EighteenSyncerNeverRetries,
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
    /// Met a forcing error and listed the receipts it has not forced (the bit set is its listing); holds the lock; recording is next.
    ErrListed(u8),
    /// Recorded its listing; holds the lock; the release is next.
    ErrRecorded,
    /// Released the lock on the error path; the listing of the lane once more is next.
    ErrReleased,
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
    /// The flushers: two slots, so that a launch in the window where the holder has released the lock is representable.
    pub flushers: [Flusher; FLUSHER_SLOTS],
    /// A client rang the service since its last sweep (a wake-up only).
    pub bell: bool,
    /// Forcing is paused by the disk (an outside condition): no forcing step completes.
    pub stalled: bool,
    /// A next owner has taken over and swept: the outside causes that excused an unforced receipt before it no longer do.
    pub replaced: bool,
    /// The service has crashed once.
    pub crashed: bool,
    /// A flusher was killed from outside once.
    pub killed: bool,
    /// A forcing error happened once.
    pub errored: bool,
    /// The unrelated command.
    pub dispatch: Dispatch,
}

/// How many flushers the model can hold at once: one per client is the most a launch can start.
pub const FLUSHER_SLOTS: usize = CLIENTS;

/// The start state: a running service, no receipts.
pub fn initial(rules: Rules) -> State {
    return State {
        rules,
        clients: [Client::Idle; CLIENTS],
        receipts: [Receipt::Absent; CLIENTS],
        seal: false,
        service: Service::Running,
        flushers: [Flusher::Off; FLUSHER_SLOTS],
        bell: false,
        stalled: false,
        replaced: false,
        crashed: false,
        killed: false,
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

/// Whether a flusher in `phase` holds `lane/flusher.lock`.
fn holds_lock(phase: Flusher) -> bool {
    return matches!(phase, Flusher::Enum | Flusher::Force(_) | Flusher::Empty | Flusher::ErrListed(_) | Flusher::ErrRecorded);
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

/// The bit set of receipts that are V or carry a recorded error: every file of `lane/receipt/` that is not forced.
fn unforced(receipts: &[Receipt; CLIENTS]) -> u8 {
    let mut mask: u8 = 0;
    for (at, receipt) in receipts.iter().enumerate() {
        if matches!(*receipt, Receipt::V | Receipt::Err) {
            mask |= 1 << at;
        }
    }
    return mask;
}

/// Forces the receipts in `mask` that are still V or carry a recorded error (a recorded error is still a file in the directory).
fn force(receipts: &mut [Receipt; CLIENTS], mask: u8) {
    for (at, receipt) in receipts.iter_mut().enumerate() {
        if mask & (1 << at) != 0 && matches!(*receipt, Receipt::V | Receipt::Err) {
            *receipt = Receipt::D;
        }
    }
}

/// Marks the receipts in `mask` that are still V as carrying a recorded forcing error.
fn record_errors(state: &mut State, mask: u8) {
    for (at, receipt) in state.receipts.iter_mut().enumerate() {
        if mask & (1 << at) != 0 && *receipt == Receipt::V {
            *receipt = Receipt::Err;
        }
    }
}

/// Whether the error path lists the lane once more after it releases the lock.
fn relists(rules: Rules) -> bool {
    return !matches!(rules, Rules::EighteenErrorIgnoresJoiners | Rules::EighteenErrorNoRelist);
}

/// Starts a flusher for a client's launch. Under the lock the holder's recheck sweep covers a receipt created before this client read the
/// lock; with the lock free a new flusher takes it. Without the lock (the control) every launch starts another flusher.
fn launch(state: &mut State) {
    let lock_held: bool = state.flushers.iter().any(|phase| { return holds_lock(*phase); });
    if lock_held && state.rules != Rules::EighteenNoFlusherLock {
        return;
    }
    if let Some(slot) = state.flushers.iter_mut().find(|phase| { return **phase == Flusher::Off; }) {
        *slot = Flusher::Enum;
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
                if state.service == Service::Running {
                    // A running service can refuse the enqueue (draining queue, full queue): the receipt is not handed over.
                    let mut rejected: State = state.clone();
                    rejected.clients[c] = if state.rules == Rules::EighteenIgnoresRejection { Client::Done } else { Client::Read(false) };
                    push_step(&mut found, &format!("environment: the running service rejects client {}'s enqueue", c), rejected);
                }
            }
            Client::Read(false) => {
                let starts_owner_instead: bool = state.rules == Rules::EighteenTokenToExitClientStartsOwner && state.service == Service::Exited;
                if !starts_owner_instead {
                    let mut started: State = state.clone();
                    launch(&mut started);
                    started.clients[c] = Client::Done;
                    push_step(&mut found, &format!("client {}: starts a one-shot flusher", c), started);
                }
                if state.rules == Rules::EighteenTokenToExitClientStartsOwner && state.service == Service::Exited {
                    let mut owner: State = state.clone();
                    owner.seal = false;
                    owner.service = Service::Starting;
                    owner.clients[c] = Client::Done;
                    push_step(&mut found, &format!("client {}: finds no service and starts the next owner before it returns", c), owner);
                }
                let mut refused: State = state.clone();
                refused.clients[c] = Client::NotYet;
                push_step(&mut found, &format!("environment: a cap set by someone else refuses client {}'s flusher", c), refused);
            }
            Client::Done | Client::NotYet => {}
        }
    }
    service_steps(state, &mut found);
    for slot in 0..FLUSHER_SLOTS {
        flusher_steps(state, slot, &mut found);
    }
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
        let foreground: bool = state.rules == Rules::EighteenForegroundForcing || state.rules == Rules::Seventeen;
        if !foreground || visible(&state.receipts) == 0 {
            let mut after: State = state.clone();
            after.dispatch = Dispatch::Done;
            push_step(&mut found, "owner: L4 queues the outstanding receipts and dispatches", after);
        } else if !state.stalled {
            // The foreground forcing of outstanding receipts before dispatch: it waits for each V receipt to be forced.
            let mut after: State = state.clone();
            force(&mut after.receipts, visible(&state.receipts));
            push_step(&mut found, "owner: L4 forces the outstanding receipts in the foreground", after);
        }
    }
    return found;
}

fn service_steps(state: &State, found: &mut Vec<(String, State)>) {
    match state.service {
        Service::Running => {
            if matches!(state.rules, Rules::EighteenTokenToExitWithSyncerRetry | Rules::EighteenTokenToExitClientStartsOwner) && !state.stalled && visible(&state.receipts) != 0 {
                let mut after: State = state.clone();
                force(&mut after.receipts, unforced(&state.receipts));
                push_step(found, "owner: the running syncer retries the receipts that a client left to the continuing retry", after);
            }
            if state.rules != Rules::EighteenSyncerNeverRetries && !state.stalled && state.receipts.iter().any(|receipt| { return *receipt == Receipt::Err; }) {
                let mut after: State = state.clone();
                force(&mut after.receipts, unforced(&state.receipts));
                push_step(found, "owner: the running syncer retries the receipts that carry a recorded error", after);
            }
            if state.bell && !state.stalled {
                let mut after: State = state.clone();
                after.bell = false;
                force(&mut after.receipts, unforced(&state.receipts));
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
                if state.rules == Rules::EighteenFlusherInOwnerScope {
                    crash.flushers = [Flusher::Off; FLUSHER_SLOTS];
                }
                push_step(found, "environment: the service crashes", crash);
            }
        }
        Service::Sealed => {
            let mut after: State = state.clone();
            after.service = Service::Enumerated(unforced(&state.receipts));
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
                after.flushers = [Flusher::Off; FLUSHER_SLOTS];
            }
            push_step(found, "owner: the service exits", after);
        }
        Service::Exited => {
            let mut after: State = state.clone();
            after.seal = false;
            after.service = Service::Starting;
            // The next owner's L2 reaps the scope of the previous incarnation: a flusher that lives in it ends mid-sync.
            if state.rules == Rules::EighteenReapedFlusherNoStartupSweep {
                after.flushers = [Flusher::Off; FLUSHER_SLOTS];
            }
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
            let sweeps: bool = !matches!(state.rules, Rules::EighteenNoStartupSweep | Rules::EighteenReapedFlusherNoStartupSweep);
            if sweeps && !state.stalled {
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

fn flusher_steps(state: &State, slot: usize, found: &mut Vec<(String, State)>) {
    let phase: Flusher = state.flushers[slot];
    let with = |next: Flusher| -> State {
        let mut after: State = state.clone();
        after.flushers[slot] = next;
        return after;
    };
    match phase {
        Flusher::Off => {}
        Flusher::Enum => {
            let mask: u8 = visible(&state.receipts);
            let next: Flusher = if mask != 0 { Flusher::Force(mask) } else { Flusher::Empty };
            push_step(found, &format!("owner: flusher {} enumerates lane/receipt/", slot), with(next));
        }
        Flusher::Force(mask) => {
            if !state.stalled {
                let mut forced: State = with(Flusher::Enum);
                force(&mut forced.receipts, mask);
                push_step(found, &format!("owner: flusher {} forces its snapshot", slot), forced);
                if !state.errored {
                    // A forcing error, in observable phases: the handler lists the receipts it has not forced (the directory enumerates
                    // them), records that listing, releases the lock and lists the lane once more. A receipt that joins between the
                    // listing and the release found the lock held and started no flusher; only the second listing covers it.
                    let listed: u8 = if state.rules == Rules::EighteenErrorIgnoresJoiners { mask } else { visible(&state.receipts) };
                    let mut errored: State = with(Flusher::ErrListed(listed));
                    errored.errored = true;
                    errored.replaced = false;
                    push_step(found, &format!("environment: flusher {} meets a forcing error and lists the receipts it has not forced", slot), errored);
                }
            }
        }
        Flusher::ErrListed(listed) => {
            let mut recorded: State = with(Flusher::ErrRecorded);
            record_errors(&mut recorded, listed);
            push_step(found, &format!("owner: flusher {} records its listing in lane/receipt-errors", slot), recorded);
        }
        Flusher::ErrRecorded => {
            let next: Flusher = if relists(state.rules) { Flusher::ErrReleased } else { Flusher::Off };
            push_step(found, &format!("owner: flusher {} releases the lock on its error path", slot), with(next));
        }
        Flusher::ErrReleased => {
            let mut relisted: State = with(Flusher::Off);
            record_errors(&mut relisted, visible(&state.receipts));
            push_step(found, &format!("owner: flusher {} lists the lane once more and records the receipts that joined", slot), relisted);
        }
        Flusher::Empty => {
            let next: Flusher = if matches!(state.rules, Rules::EighteenNoRecheck | Rules::EighteenTokenToExitWithSyncerRetry | Rules::EighteenTokenToExitClientStartsOwner) { Flusher::Off } else { Flusher::Released };
            push_step(found, &format!("owner: flusher {} releases the lock after its last empty sweep", slot), with(next));
        }
        Flusher::Released => {
            // The one more sweep, without the lock; it takes the lock again when it finds work and nobody else holds it (a holder's own
            // recheck covers the receipt otherwise).
            let work: bool = visible(&state.receipts) != 0;
            let other_holder: bool = state.flushers.iter().enumerate().any(|(other, held)| { return other != slot && holds_lock(*held); });
            let next: Flusher = if work && !other_holder { Flusher::Enum } else { Flusher::Off };
            push_step(found, &format!("owner: flusher {} sweeps once more without the lock", slot), with(next));
        }
    }
    if phase != Flusher::Off && !state.killed {
        let mut killed: State = with(Flusher::Off);
        killed.killed = true;
        killed.replaced = false;
        push_step(found, &format!("environment: flusher {} is killed from outside", slot), killed);
    }
}

/// Safety: at most one flusher holds the lock.
pub fn invariant(state: &State) -> Option<String> {
    if state.flushers.iter().filter(|phase| { return holds_lock(**phase); }).count() > 1 {
        return Some("more than one flusher runs for the lane".to_string());
    }
    return None;
}

/// A quiescent state (only the environment can move) is acceptable only if no receipt is V with nothing responsible for it, no recorded
/// error was left unretried after a next owner took over, and no unrelated command is still waiting at L4. The stated outside causes
/// excuse a stalled disk, a crash of the service, a flusher killed from outside, and a launch refused by someone else's cap, each only until
/// a next owner has swept. A forcing error does not excuse a receipt: the error is recorded per receipt.
pub fn terminal(state: &State) -> Option<String> {
    // A command that arrives while the service is in its shutdown sweep waits for the lane, and that wait is the register's ACQUIRE entry,
    // bounded by its own deadline; only a command that reached L4 on a running service is the property here.
    let lane_busy_with_shutdown: bool = matches!(state.service, Service::Sealed | Service::Enumerated(_) | Service::Forced);
    if state.dispatch == Dispatch::Requested && !lane_busy_with_shutdown {
        return Some("an unrelated command's dispatch is still waiting for receipts".to_string());
    }
    if state.stalled {
        return None;
    }
    let excused: bool = (state.crashed || state.killed) && !state.replaced;
    for c in 0..CLIENTS {
        if state.receipts[c] == Receipt::V && !excused && state.clients[c] != Client::NotYet {
            return Some(format!("client {}'s receipt is V and no executor is responsible for it", c));
        }
        if state.receipts[c] == Receipt::Err && state.replaced && state.service == Service::Running {
            return Some(format!("client {}'s recorded forcing error was not retried after the next owner took over", c));
        }
    }
    return None;
}

/// The caps.
pub fn limits() -> Limits {
    return Limits { max_states: 1_500_000, max_depth: 100 };
}

/// Explores every schedule under `rules`.
pub fn check(rules: Rules) -> Report {
    return explore(vec![initial(rules)], steps, invariant, terminal, limits());
}

/// Explores every schedule under `rules` with an unrelated command that may reach L4 at any point.
pub fn check_dispatch(rules: Rules) -> Report {
    return explore(vec![initial_with_unrelated_command(rules)], steps, invariant, terminal, limits());
}
