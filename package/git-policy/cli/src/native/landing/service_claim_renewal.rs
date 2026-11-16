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
//!  No lock in the process:
//! the held claims have one owner at a time,
//! and the owner hands the whole set to the step that uses it and takes it back
//! (`service_work_storage_awaited.rs`),
//! so the list travels by value rather than being shared behind a lock.
//! The renewal task is the one owner of the claims' current names (the drain never
//!  renames a claim except to settle it);
//! the drain tells the task of a claim it takes over a channel,
//! and the two share only
//!  atomic flags per claim.
//! A settlement finds the claim's file by its id and generation,
//! which a renewal never changes,
//! and
//!  asks again when a renewal renamed the file between the look and the rename,
//! so neither side ever waits for the other.
//! The drain itself is a task of the runtime whose storage steps are jobs of the blocking pool
//!  (`service_work_storage_awaited.rs`),
//! so the list of held claims would be touched by whichever pool thread runs a step;
//! it is
//!  therefore never shared:
//! the task moves the set into the job and the job hands it back,
//! and a job the system refuses a
//!  thread for ends the round rather than the set.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { held, renewer } = heldClaims(workDir); const renewal = startRenewal(renewer, lease / 3); try { await work() } finally { renewal.stop() }
//! ```

use crate::claim_time::ClaimTime;
use crate::work_item::{CLAIMED, Claimed, id_of_entry, names_claim, parse_claim_name, renew_claim, settle_named, the_names_in};
use crate::work_record_where_it_went::the_claim_directory_could_not_be_read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::RefCell;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use std::sync::Arc;
use std::time::Duration;

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

/// What:
///  The claims one drain holds right now,
/// kept by the drain's task,
/// which alone settles them.
/// Why:
///   The drain must refuse to run an item whose claim was taken over,
/// and settle a claim under the name it has now.
/// The list is
///  the drain's own,
/// and the drain's storage steps run as jobs of the blocking pool:
/// the set is moved into the job
///  that reads or renames it and comes back with the answer,
/// so one owner holds it at a time and nothing locks it.
/// What the renewal task learns of a claim reaches the drain as the claim's
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
    /// The listing of `claimed` the last look took,
    /// shared by every claim of the round,
    /// and nothing before the first look.
    listed: RefCell<Option<ClaimNames>>,
    /// The way to tell the renewal of a claim just taken.
    holds: UnboundedSender<Taken>,
}

/// What:
///  The claim files `claimed` held when it was listed,
/// by the id each name carries.
/// Why:
///   A drain holds many claims and settles them one after another,
/// and a listing of the whole directory for
///  each claim costs a multiple of the directory per round (`RG2`).
/// One listing serves every name the round needs;
/// a name the listing does not hold is looked for in a fresh listing,
/// because a renewal renames the claim of a
///  live holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ClaimNames { byId: Map<string, string[]>; nameOf(claim) }
/// ```
#[derive(Debug)]
struct ClaimNames {
    /// The names of `claimed`,
    /// by the id each one is the claim of.
    by_id: std::collections::HashMap<String, Vec<String>>,
}

impl ClaimNames {
    /// What:
    ///  Lists `claimed` once,
    ///  or answers the error that kept it from being listed.
    /// Why:
    ///   The names are kept whole,
    /// so a claim is found by its id and generation alone,
    /// and the listing is the one
    ///  directory read the round makes.
    /// A directory that could not be listed is an error and never an empty listing:
    /// a settlement that read it as empty would say a claim is gone that nobody moved.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new ClaimNames(Object.groupBy(await readdirOrTheError(claimed), idOfEntry))
    /// ```
    fn listed(work_dir: &Path) -> std::io::Result<ClaimNames> {
        let reached = the_names_in(work_dir, CLAIMED);
        if let Some(error) = reached.stopped_by {
            return Err(error);
        }
        let mut by_id: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for (name, _modified) in reached.names {
            by_id.entry(id_of_entry(name.as_str()).to_owned()).or_default().push(name);
        }
        return Ok(ClaimNames { by_id });
    }

    /// What:
    ///  The name that is the file of `claim` in this listing,
    /// or nothing when this listing holds no such file.
    /// Why:
    ///   A claim is found by its id and the generation of its acquisition,
    /// so another instance's claim of the same item is
    ///  not taken for this one's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// this.byId.get(claim.id)?.find((name) => namesClaim(name, claim))
    /// ```
    fn name_of(&self, claim: &Claim) -> Option<&str> {
        for name in self.by_id.get(claim.id.as_str())? {
            if names_claim(name.as_str(), claim.id.as_str(), claim.token.as_str()) {
                return Some(name.as_str());
            }
        }
        return None;
    }
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
    holds: UnboundedReceiver<Taken>,
    /// The claims being kept fresh,
    /// each with the file name this renewer last wrote.
    names: Vec<(Arc<Claim>, String)>,
}

/// The two halves of the claims of a drain of `work_dir`:
/// the drain's list and the renewer that keeps them fresh.
pub fn held_claims(work_dir: &Path) -> (HeldClaims, Renewer) {
    let (holds, taken) = unbounded_channel::<Taken>();
    return (
        HeldClaims { claims: RefCell::new(Vec::new()), listed: RefCell::new(None), holds },
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
    ///  The file name that holds `claim` in `claimed` now,
    /// or nothing when no file does.
    /// Why:
    ///   A renewal changes the reading in the name and nothing else,
    /// so the id and the generation find the claim whatever its
    ///  reading is,
    /// and a claim that was taken over and claimed again by another instance has another generation and is not found.
    /// The listing is taken once and serves every claim of the round;
    /// a name it does not hold is looked for in a fresh listing,
    /// because a renewal may have renamed the file since.
    ///
    /// A fresh listing that fails is the third answer and not a miss:
    /// the error goes to the caller, which says that the look failed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (let look = 0; look < LOOKS; look += 1) { const name = this.listed.nameOf(claim); if (name) return name; this.listed = await ClaimNames.listed(workDir) }
    /// ```
    fn current_name(&self, work_dir: &Path, claim: &Claim) -> std::io::Result<Option<String>> {
        for _ in 0..LOOKS {
            let found: Option<String> = match self.listed.borrow().as_ref() {
                Some(names) => names.name_of(claim).map(str::to_owned),
                None => None,
            };
            if let Some(name) = found {
                return Ok(Some(name));
            }
            // Nothing in the listing: a renewal may have renamed the file since it was taken, so `claimed` is listed again.
            let fresh: ClaimNames = ClaimNames::listed(work_dir)?;
            *self.listed.borrow_mut() = Some(fresh);
        }
        return Ok(None);
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
    ///  A directory that could not be listed is not a claim that is gone:
    /// this face answers what was found out,
    /// and the drain says so instead of naming a takeover.
    /// `is_held_on_disk` keeps the plain answer for a caller that cannot say a look failed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// held.has(id) && !held.lost(id) && currentName(workDir, claim).ok()?.value !== undefined
    /// ```
    pub fn is_held_on_disk_or_the_error(&self, work_dir: &Path, id: &str) -> Result<bool, String> {
        // The claim is cloned out of the list, so the look at the directory happens with no lock held.
        let found: Option<Arc<Claim>> = {
            let claims = self.claims.borrow();
            let mut found: Option<Arc<Claim>> = None;
            for candidate in claims.iter() {
                if candidate.id == id {
                    found = Some(Arc::clone(candidate));
                    break;
                }
            }
            found
        };
        let Some(claim) = found else { return Ok(false) };
        if claim.lost.load(Ordering::SeqCst) {
            return Ok(false);
        }
        return match self.current_name(work_dir, claim.as_ref()) {
            Ok(Some(_name)) => Ok(true),
            Ok(None) => {
                claim.lost.store(true, Ordering::SeqCst);
                Ok(false)
            }
            // The look failed, so nothing is known about the claim: it is neither taken over nor let go.
            Err(error) => Err(the_claim_directory_could_not_be_read(&crate::work_item::state_directory(work_dir, CLAIMED), id, &error)),
        };
    }

    /// The same question with two answers,
    /// for a caller that has no way to say that the look failed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// held.has(id) && !held.lost(id) && currentName(workDir, claim) !== undefined
    /// ```
    pub fn is_held_on_disk(&self, work_dir: &Path, id: &str) -> bool {
        return self.is_held_on_disk_or_the_error(work_dir, id).unwrap_or(false);
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
    ///  A move that finds nothing says which order left the record elsewhere,
    /// read from the record's own directories (`work_record_where_it_went.rs`):
    /// another instance's claim,
    /// the item queued again,
    /// a take in flight and the two endings it can have,
    /// a record given up on,
    /// or a work directory that went with a repository removed under the service.
    /// The claim's absence alone says none of those,
    /// so it is never read as a lease that ran out.
    ///  A listing that failed says only that nothing could be found out about the claim,
    /// and never that the claim was absent.
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
        let mut unread: Option<std::io::Error> = None;
        for _ in 0..LOOKS {
            let found: Option<String> = match self.current_name(work_dir, &claim) {
                Ok(found) => found,
                Err(error) => {
                    unread = Some(error);
                    break;
                }
            };
            let Some(name) = found else { break };
            match settle_named(work_dir, name.as_str(), state) {
                Ok(()) => return Ok(()),
                Err(reason) => {
                    last = reason;
                    // A renewal renamed the file between the look and the rename: the listing the name came from is old.
                    *self.listed.borrow_mut() = None;
                }
            }
        }
        // A move that failed and then a listing that failed say nothing about the claim being absent,
        // so the move is named without that clause and the words below say what was found out.
        let reason: String = match (last.is_empty(), unread.is_some()) {
            (true, true) => format!("moving {id} to {state} failed"),
            (true, false) => format!("moving {id} to {state} failed: the claim is not in {CLAIMED}"),
            (false, _) => last,
        };
        let where_it_went: String = crate::work_record_where_it_went::words_for_a_move_that_found_nothing(work_dir, id, claim.token.as_str());
        return Err(format!("{reason}; {where_it_went}"));
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

/// Renewal controls stay out of the release executable.
#[cfg(test)]
#[path = "service_claim_renewal_tests.rs"]
mod tests;
