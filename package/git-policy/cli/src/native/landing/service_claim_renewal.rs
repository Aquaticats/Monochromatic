//! What:
//!  Keeps the lease of the claims one service instance holds fresh while it works,
//! so a claim is taken over only when its holder
//!  stopped,
//! never because its holder is slow,
//! and tells the holder when its claim was taken over anyway.
//! Why:
//!  A claim's lease starts at the clock reading in its file name and the other instances run a claim again once the lease ran
//!  out.
//! A push can take longer than any fixed lease,
//! and an item can wait behind other claimed work;
//! a claim that is never
//!  renewed would make both look abandoned.
//! A holder that dies stops renewing,
//! so its claims still expire,
//! and no lock file is
//!  involved:
//! the claim's file name is the whole protocol.
//! A renewal renames the claim from the name the renewal last wrote,
//! so
//!  it can only reach its own claim,
//! and a claim that was taken over is found gone instead of being renewed or settled on
//!  another instance's behalf.
//!  Claim kept:
//! the lease stays because a killed holder's claims are recognised as abandoned only by their reading (no process
//!  identity or lock file is involved),
//! and because the linked worktrees of a bare repository drain one common directory.
//!  No lock is held in the process either.
//! The renewal thread is the one owner of the claims' current names (the drain never
//!  renames a claim except to settle it);
//! the drain tells the thread of a claim it takes over a channel,
//! and the two share only
//!  atomic flags per claim.
//! A settlement finds the claim's file by its id and generation,
//! which a renewal never changes,
//! and
//!  asks again when a renewal renamed the file between the look and the rename,
//! so neither side ever waits for the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { held, renewer } = heldClaims(workDir); const renewal = startRenewal(renewer, lease / 3); try { await work() } finally { renewal.stop() }
//! ```

use crate::claim_time::ClaimTime;
use crate::work_item::{CLAIMED, Claimed, id_of_entry, parse_claim_name, renew_claim, settle_named, state_directory};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::blocking_job::run_blocking_job;
use crate::lazy_runtime::spawn_on_runtime;
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::Instant;

/// How many times inside one lease a holder renews,
/// so a renewal that is late by up to two intervals still beats the lease.
pub const RENEWALS_PER_LEASE: u32 = 3;

/// How many times a look for a claim's file is repeated before the claim is called gone.
/// A rename inside a directory during a listing can make the listing miss the entry once,
/// and a renewal renames the claim at most
/// once per interval,
/// so a claim that is still there is found within a few looks.
const LOOKS: usize = 3;

/// How often a holder renews its claims for a lease of `lease`:
/// a third of it,
/// so that a renewal that is late by up to two
/// intervals still beats the lease.
/// The production lease is 120 seconds,
/// so a holder renews every 40 seconds and loses its
/// claim only when a renewal is late by more than 80.
pub fn renewal_interval(lease: Duration) -> Duration {
    return lease / RENEWALS_PER_LEASE;
}

/// What:
///  One claim this drain holds:
/// its id,
/// its generation,
/// and the two flags the drain and the renewal share.
/// Why:
///   The id and the generation never change over the claim's life,
/// so they are plain data both sides read;
/// only the file name
///  changes (with each renewal),
/// and the renewal alone knows it.
/// The flags are atomic,
/// so neither side waits for the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Claim = { id: string; token: string; lost: AtomicBool; settled: AtomicBool };
/// ```
#[derive(Debug)]
struct Claim {
    /// The item's id.
    id: String,
    /// The claim's generation,
    /// the part of its file name that a renewal keeps.
    token: String,
    /// Set by the renewal when the file was not there under the name it last wrote:
    /// another instance took the claim over.
    lost: AtomicBool,
    /// Set by the drain when it settled or released the claim,
    /// so the renewal stops renewing it.
    settled: AtomicBool,
}

/// A claim the drain took,
/// with the file name it was taken under:
/// what the drain tells the renewal of a new claim.
type Taken = (Arc<Claim>, String);

/// What a controller tells the renewal task.
enum Control {
    /// Renew every claim held now at this reading,
    /// and say so on the channel when done,
    /// if one is given.
    Renew(ClaimTime, Option<SyncSender<()>>),
    /// Stop the task.
    Stop,
}

/// What:
///  The claims one drain holds right now,
/// kept by the drain thread,
/// which alone settles them.
/// Why:
///   The drain must refuse to run an item whose claim was taken over,
/// and settle a claim under the name it has now.
/// The list is
///  the drain thread's own,
/// so nothing locks it;
/// what the renewal thread learns of a claim reaches the drain as the claim's
///  flags,
/// and what the drain learns of a new claim reaches the renewal as a command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldClaims = { held: Claim[]; renewal: Channel<Command> };
/// ```
#[derive(Debug)]
pub struct HeldClaims {
    /// The claims held,
    /// in the order they were taken.
    claims: RefCell<Vec<Arc<Claim>>>,
    /// The way to tell the renewal of a claim just taken.
    holds: Sender<Taken>,
}

/// What:
///  The renewal's half of a drain's claims:
/// the names it last wrote,
/// and the way the drain tells it of new ones.
/// Why:
///   Only the renewal renames a held claim,
/// so only it needs the current name;
/// the drain finds the file by id and generation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Renewer = { workDir: string; inbox: Channel<Command>; names: Map<Claim, string> };
/// ```
pub struct Renewer {
    /// The work directory the claims are in.
    work_dir: PathBuf,
    /// The claims the drain took.
    holds: Receiver<Taken>,
    /// The claims being kept fresh,
    /// each with the file name this renewer last wrote.
    names: Vec<(Arc<Claim>, String)>,
}

/// The two halves of the claims of a drain of `work_dir`:
/// the drain's list and the renewer that keeps them fresh.
pub fn held_claims(work_dir: &Path) -> (HeldClaims, Renewer) {
    let (holds, taken) = channel::<Taken>();
    return (
        HeldClaims { claims: RefCell::new(Vec::new()), holds },
        Renewer { work_dir: work_dir.to_path_buf(), holds: taken, names: Vec::new() },
    );
}

/// What:
///  Whether a held claim is still unsettled.
/// Why:
///  The renewal drops the claims the drain settled,
/// and `Vec::retain` hands each entry to a function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isUnsettled = ([claim]: [Claim, string]) => !claim.settled;
/// ```
fn is_unsettled(entry: &(Arc<Claim>, String)) -> bool {
    return !entry.0.settled.load(Ordering::SeqCst);
}

/// Whether `name` is the file of the claim `claim` (the id and the generation match).
fn names_claim(name: &str, claim: &Claim) -> bool {
    if id_of_entry(name) != claim.id {
        return false;
    }
    if claim.token.is_empty() {
        return true;
    }
    return match parse_claim_name(name) {
        Some((token, _stamp)) => token == claim.token,
        None => false,
    };
}

/// What:
///  The file name that holds `claim` in `claimed` now,
/// or nothing when no file does.
/// Why:
///   A renewal changes the reading in the name and nothing else,
/// so the id and the generation find the claim whatever its
///  reading is,
/// and a claim that was taken over and claimed again by another instance has another generation and is not found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(join(workDir, 'claimed'))).find((name) => nameClaims(name, claim))
/// ```
fn current_name(work_dir: &Path, claim: &Claim) -> Option<String> {
    for _ in 0..LOOKS {
        // A directory that cannot be listed finds nothing on this look; the next look tries again.
        let Ok(listing) = std::fs::read_dir(state_directory(work_dir, CLAIMED)) else {
            continue;
        };
        for entry in listing.flatten() {
            let name: String = entry.file_name().to_string_lossy().into_owned();
            if names_claim(name.as_str(), claim) {
                return Some(name);
            }
        }
    }
    return None;
}

/// Methods over the held claims.
impl HeldClaims {
    /// Add a claim the drain took,
    /// and tell the renewal thread to keep it fresh.
    pub fn hold(&self, claimed: &Claimed) {
        // A claim by an earlier release has no generation, so its token is empty.
        let token: String = match parse_claim_name(claimed.name.as_str()) {
            Some((found, _stamp)) => found.to_owned(),
            None => String::new(),
        };
        let claim: Arc<Claim> = Arc::new(Claim { id: claimed.id.clone(), token, lost: AtomicBool::new(false), settled: AtomicBool::new(false) });
        self.claims.borrow_mut().push(Arc::clone(&claim));
        // A renewal thread that is gone (no thread could be made, or it ended) renews nothing: the claim then expires with its lease.
        let _sent = self.holds.send((claim, claimed.name.clone()));
    }

    /// Forget a claim the drain gave up,
    /// so it is no longer renewed.
    pub fn release(&self, id: &str) {
        // The claims of other ids are kept in order; the ones of this id are marked settled and dropped.
        let mut claims = self.claims.borrow_mut();
        let mut kept: Vec<Arc<Claim>> = Vec::with_capacity(claims.len());
        for claim in claims.drain(..) {
            if claim.id == id {
                claim.settled.store(true, Ordering::SeqCst);
            } else {
                kept.push(claim);
            }
        }
        *claims = kept;
    }

    /// The ids held at this moment,
    /// including claims found taken over.
    pub fn snapshot(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for claim in self.claims.borrow().iter() {
            ids.push(claim.id.clone());
        }
        return ids;
    }

    /// Whether the claim of `id` is held and was not found taken over.
    pub fn is_held(&self, id: &str) -> bool {
        for claim in self.claims.borrow().iter() {
            if claim.id == id && !claim.lost.load(Ordering::SeqCst) {
                return true;
            }
        }
        return false;
    }

    /// What:
    ///  Whether the claim of `id` is held and its file is in `claimed` now,
    /// found by its id and generation.
    /// Why:
    ///   A renewal that has not run since another instance took the claim over has not found it gone,
    /// so the flag of `is_held` still says held.
    /// The holder looks at the directory itself just before it acts on the claim,
    /// so a push is not started from a claim that is already another instance's,
    /// however late the renewal is.
    /// The look and the push are two steps,
    /// so a takeover between them is still possible;
    /// what the look removes is the long window of a renewal that is late.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// held.has(id) && !held.lost(id) && currentName(workDir, claim) !== undefined
    /// ```
    pub fn is_held_on_disk(&self, work_dir: &Path, id: &str) -> bool {
        let claims = self.claims.borrow();
        let mut found: Option<&Arc<Claim>> = None;
        for candidate in claims.iter() {
            if candidate.id == id {
                found = Some(candidate);
                break;
            }
        }
        let Some(claim) = found else { return false };
        if claim.lost.load(Ordering::SeqCst) {
            return false;
        }
        if current_name(work_dir, claim).is_none() {
            claim.lost.store(true, Ordering::SeqCst);
            return false;
        }
        return true;
    }

    /// What:
    ///  Move the claim of `id` to `state` and stop holding it;
    /// an error says so when another instance took the claim over.
    /// Why:
    ///   The rename names the claim's current file,
    /// found by its generation,
    /// so a claim that was taken over,
    /// or taken over and
    ///  claimed again by another instance,
    /// is not moved:
    /// that instance's work is its own to settle.
    /// A renewal that renames the
    ///  file between the look and the rename makes the rename find nothing,
    /// and the look is made again.
    pub fn settle(&self, work_dir: &Path, id: &str, state: &str) -> Result<(), String> {
        let claim: Arc<Claim> = {
            let mut claims = self.claims.borrow_mut();
            let mut found_at: Option<usize> = None;
            for (at, candidate) in claims.iter().enumerate() {
                if candidate.id == id {
                    found_at = Some(at);
                    break;
                }
            }
            let Some(position) = found_at else {
                return Err(format!("{id} is not held by this instance"));
            };
            claims.remove(position)
        };
        claim.settled.store(true, Ordering::SeqCst);
        if claim.lost.load(Ordering::SeqCst) {
            return Err(format!("{id} was taken over by another instance"));
        }
        let mut last: String = String::new();
        for _ in 0..LOOKS {
            let Some(name) = current_name(work_dir, &claim) else { break };
            match settle_named(work_dir, name.as_str(), state) {
                Ok(()) => return Ok(()),
                Err(reason) => last = reason,
            }
        }
        let reason: String = if last.is_empty() { format!("moving {id} to {state} failed: the claim is not in {CLAIMED}") } else { last };
        return Err(format!("{reason}; its lease probably ran out and another instance took the work over"));
    }
}

/// Methods over the renewer.
impl Renewer {
    /// Take in the claims the drain has told of since the last look.
    fn take_in(&mut self) {
        while let Ok(taken) = self.holds.try_recv() {
            self.names.push(taken);
        }
    }

    /// Renew every claim held at the reading `now`;
    /// a claim whose file is not there under its name is marked taken over,
    /// and a
    /// claim the drain settled is dropped.
    pub fn renew(&mut self, now: &ClaimTime) {
        self.take_in();
        self.names.retain(is_unsettled);
        for (claim, name) in &mut self.names {
            if claim.lost.load(Ordering::SeqCst) {
                continue;
            }
            match renew_claim(self.work_dir.as_path(), name.as_str(), now) {
                Some(renewed) => *name = renewed,
                None => claim.lost.store(true, Ordering::SeqCst),
            }
        }
    }

    /// The file names of the claims being kept fresh,
    /// as this renewer last wrote them.
    pub fn names(&mut self) -> Vec<String> {
        self.take_in();
        let mut names: Vec<String> = Vec::new();
        for (_claim, name) in &self.names {
            names.push(name.clone());
        }
        return names;
    }
}

/// How the claims of one drain are kept fresh.
#[derive(Clone, Copy, Debug)]
pub enum Renewal<'a> {
    /// A thread of the drain renews every third of the lease by the real clock.
    Timed,
    /// Nothing renews until the driver is told to,
    /// with the reading it is given;
    /// a control uses this to be independent of how
    /// the host schedules threads,
    /// and to let any number of leases pass without waiting.
    Driven(&'a DrivenRenewal),
}

/// What:
///  The way a control renews the claims of a drain it started,
/// at the reading it chooses.
/// Why:
///   The renewal thread's timing is real time,
/// and a control of what a renewal protects cannot be told apart from a slow host by waiting.
/// A driven renewal does the same renaming at moments the control chooses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const driver = new DrivenRenewal(); drain(..., { driven: driver }); driver.renew(clockAfterTwoLeases);
/// ```
#[derive(Debug, Default)]
pub struct DrivenRenewal {
    /// The way to the renewal thread of the drain that registered,
    /// once it has.
    /// A driver serves one drain.
    ticks: OnceLock<UnboundedSender<Control>>,
}

/// Methods over a driver.
impl DrivenRenewal {
    /// Register the renewal thread of a drain;
    /// the first drain to register is the one the driver serves.
    fn attach(&self, control: UnboundedSender<Control>) {
        let _first: Result<(), UnboundedSender<Control>> = self.ticks.set(control);
    }

    /// Renew every claim the registered drain holds at the reading `now`,
    /// and return once that is done;
    /// false when no drain has registered or its thread is gone.
    pub fn renew(&self, now: &ClaimTime) -> bool {
        let Some(control) = self.ticks.get() else { return false };
        let (done, finished) = sync_channel::<()>(1);
        return control.send(Control::Renew(now.clone(), Some(done))).is_ok() && finished.recv().is_ok();
    }
}

/// What:
///  A running renewal:
/// a thread that stamps every held claim again each interval until this value is dropped.
/// Why:
///   Dropping stops and joins the thread,
/// so a drain that returns or unwinds leaves no thread behind,
/// and a drain that ended
///  renews nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const renewal = ClaimRenewal.start(renewer, every); renewal.stop();
/// ```
pub struct ClaimRenewal {
    /// The way to tell the task to stop.
    control: UnboundedSender<Control>,
    /// What the task sends when it has ended,
    /// taken when it is waited for;
    /// nothing when no task could be started.
    ended: Option<Receiver<()>>,
}

/// Does what a controller asked of the renewal task;
/// the renewer after it and whether the task goes on.
/// The renaming is files of the work directory,
/// which can wait on storage,
/// so it runs on the blocking pool and the renewer goes there and back.
async fn obey(renewer: Renewer, control: Control) -> (Option<Renewer>, bool) {
    return match control {
        Control::Renew(now, done) => {
            let renewed: Option<Renewer> = renew_off_the_workers(renewer, now).await;
            if let Some(finished) = done {
                let _told = finished.send(());
            }
            (renewed, true)
        }
        Control::Stop => (Some(renewer), false),
    };
}

/// Renews every claim the renewer holds at the reading `now` on the blocking pool;
/// nothing back when the pool could not run it,
/// which ends the renewal:
/// the claims then expire with their lease, as for a service that died.
async fn renew_off_the_workers(mut renewer: Renewer, now: ClaimTime) -> Option<Renewer> {
    return match run_blocking_job(move || {
        renewer.renew(&now);
        return renewer;
    })
    .await
    {
        Ok(renewed) => Some(renewed),
        Err(error) => {
            crate::service_error!("claims could not be renewed, so they expire with their lease: {error}");
            None
        }
    };
}

/// What:
///  The task of one renewal:
/// obeys what the controller sends,
/// and,
/// for a timed renewal,
/// renews by the real clock every `every`.
/// Why:
///  The task waits on the channel and on the timer with `.await`,
/// so a renewal holds no thread while it waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const request = await Promise.race([requests.next(), timer(every)]); if (request === STOP) return; renew(request ?? clockNow()); }
/// ```
async fn renew_until_stopped(mut renewer: Renewer, mut requests: UnboundedReceiver<Control>, every: Option<Duration>) {
    let mut due: Option<Instant> = every.map(|interval| return Instant::now() + interval);
    loop {
        let waited: Waited = match due {
            Some(at) => tokio::select! {
                request = requests.recv() => Waited::Asked(request),
                () = tokio::time::sleep_until(at) => Waited::Due,
            },
            None => Waited::Asked(requests.recv().await),
        };
        let renewed: Option<Renewer> = match waited {
            // Every controller is gone: nothing asks for anything any more.
            Waited::Asked(None) => return,
            Waited::Asked(Some(request)) => {
                let (after, goes_on) = obey(renewer, request).await;
                if !goes_on {
                    return;
                }
                after
            }
            // A claim that was settled is no longer held; one that was taken over is found gone and marked.
            Waited::Due => {
                let after: Option<Renewer> = renew_off_the_workers(renewer, ClaimTime::read()).await;
                due = every.map(|interval| return Instant::now() + interval);
                after
            }
        };
        let Some(kept) = renewed else { return };
        renewer = kept;
    }
}

/// What a renewal task woke for.
enum Waited {
    /// A controller asked,
    /// or every controller is gone.
    Asked(Option<Control>),
    /// The interval of a timed renewal passed.
    Due,
}

/// Starts the task of a renewal on the process's runtime;
/// nothing back when the runtime has no worker to run it,
/// where the claims expire with their lease, as for a service that died.
fn start_task(renewer: Renewer, requests: UnboundedReceiver<Control>, every: Option<Duration>) -> Option<Receiver<()>> {
    let (ended_sender, ended) = channel::<()>();
    let started = spawn_on_runtime(async move {
        renew_until_stopped(renewer, requests, every).await;
        let _told = ended_sender.send(());
    });
    if started.is_none() {
        crate::service_error!("no worker to renew claims on, so they expire with their lease");
    }
    return started.map(|_handle| return ended);
}

/// Methods over a renewal.
impl ClaimRenewal {
    /// Start renewing the claims the drain tells `renewer` of,
    /// every `every`,
    /// by the real clock.
    pub fn start(renewer: Renewer, every: Duration) -> ClaimRenewal {
        let (control, requests) = unbounded_channel::<Control>();
        let ended: Option<Receiver<()>> = start_task(renewer, requests, Some(every));
        return ClaimRenewal { control, ended };
    }

    /// Start the task of a driven renewal:
    /// it renews only when `driver` is told to,
    /// and registers with it.
    pub fn driven(renewer: Renewer, driver: &DrivenRenewal) -> ClaimRenewal {
        let (control, requests) = unbounded_channel::<Control>();
        driver.attach(control.clone());
        let ended: Option<Receiver<()>> = start_task(renewer, requests, None);
        return ClaimRenewal { control, ended };
    }
}

/// Stops the task when the renewal goes out of scope,
/// and waits until it has ended:
/// a renewal in the middle of its renames finishes them first, as the thread it replaces was joined.
impl Drop for ClaimRenewal {
    fn drop(&mut self) {
        let _told = self.control.send(Control::Stop);
        if let Some(ended) = self.ended.take() {
            // A task that panicked has nothing left to report (its sender is dropped, which ends this wait): the claims it held
            //  expire with the lease.
            let _waited = ended.recv();
        }
    }
}

/// Renewal controls stay out of the release executable.
#[cfg(test)]
#[path = "service_claim_renewal_tests.rs"]
mod tests;
