//! What:
//!  Keeps the lease of the claims one service instance holds fresh while it works, so a claim is taken over only when its holder
//!  stopped, never because its holder is slow, and tells the holder when its claim was taken over anyway.
//! Why:
//!  A claim's lease starts at the clock reading in its file name and the other instances run a claim again once the lease ran
//!  out. A push can take longer than any fixed lease, and an item can wait behind other claimed work; a claim that is never
//!  renewed would make both look abandoned. A holder that dies stops renewing, so its claims still expire, and no lock file is
//!  involved: the claim's file name is the whole protocol. A renewal renames the claim from the name the holder last wrote, so
//!  it can only reach its own claim, and a claim that was taken over is found gone instead of being renewed or settled on
//!  another instance's behalf.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const renewal = startRenewal(workDir, held, lease / 3); try { await work() } finally { renewal.stop() }
//! ```

use super::claim_time::ClaimTime;
use super::work_item::{Claimed, renew_claim, settle_named};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

/// How many times inside one lease a holder renews, so a renewal that is late by up to two intervals still beats the lease.
pub const RENEWALS_PER_LEASE: u32 = 3;

/// One claim this drain holds: the id, the name of its file as of the last renewal, and whether a renewal found it gone.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Held {
    /// The item's id.
    id: String,
    /// The claim's file name in `claimed` that this holder last wrote.
    name: String,
    /// Set when the file was not there under that name: another instance took the claim over.
    lost: bool,
}

/// What:
///  The claims one drain holds right now, each with the file name its holder last wrote.
/// Why:
///   The renewal must refresh only what this instance holds: refreshing every file in the directory would keep the dead
///  holder's claims alive for good. The drain and the renewal thread both rename a claim, so the current name is kept here
///  and every rename happens with the list locked; a settlement can then never name a file that a renewal just renamed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldClaims = { held: { id: string; name: string; lost: boolean }[] };
/// ```
#[derive(Clone, Debug, Default)]
pub struct HeldClaims {
    /// Shared with the renewal thread.
    held: Arc<Mutex<Vec<Held>>>,
}

/// Methods over the held claims.
impl HeldClaims {
    /// Add a claim the drain took.
    pub fn hold(&self, claimed: &Claimed) {
        self.held.lock().unwrap_or_else(PoisonError::into_inner).push(Held { id: claimed.id.clone(), name: claimed.name.clone(), lost: false });
    }

    /// Forget a claim the drain settled, so it is no longer renewed.
    pub fn release(&self, id: &str) {
        self.held.lock().unwrap_or_else(PoisonError::into_inner).retain(|held| { return held.id != id; });
    }

    /// The ids held at this moment, including claims found taken over.
    pub fn snapshot(&self) -> Vec<String> {
        return self.held.lock().unwrap_or_else(PoisonError::into_inner).iter().map(|held| { return held.id.clone(); }).collect();
    }

    /// Whether the claim of `id` is held and was not found taken over.
    pub fn is_held(&self, id: &str) -> bool {
        return self.held.lock().unwrap_or_else(PoisonError::into_inner).iter().any(|held| { return held.id == id && !held.lost; });
    }

    /// The file name the claim of `id` has now, or nothing when it is not held.
    pub fn name_of(&self, id: &str) -> Option<String> {
        return self.held.lock().unwrap_or_else(PoisonError::into_inner).iter().find(|held| { return held.id == id; }).map(|held| { return held.name.clone(); });
    }

    /// Renew every claim held at the reading `now`; a claim whose file is not there under its name is marked taken over.
    pub fn renew(&self, work_dir: &Path, now: &ClaimTime) {
        for held in self.held.lock().unwrap_or_else(PoisonError::into_inner).iter_mut().filter(|held| { return !held.lost; }) {
            match renew_claim(work_dir, held.name.as_str(), now) {
                Some(renewed) => held.name = renewed,
                None => held.lost = true,
            }
        }
    }

    /// What:
    ///  Move the claim of `id` to `state` and stop holding it; an error says so when another instance took the claim over.
    /// Why:
    ///   The rename names the claim's current file, so a claim that was taken over, or taken over and claimed again by
    ///  another instance, is not moved: that instance's work is its own to settle.
    pub fn settle(&self, work_dir: &Path, id: &str, state: &str) -> Result<(), String> {
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(position) = held.iter().position(|candidate| { return candidate.id == id; }) else {
            return Err(format!("{id} is not held by this instance"));
        };
        let claim: Held = held.remove(position);
        if claim.lost {
            return Err(format!("{id} was taken over by another instance"));
        }
        return settle_named(work_dir, claim.name.as_str(), state)
            .map_err(|reason| { return format!("{reason}; its lease probably ran out and another instance took the work over"); });
    }
}

/// What:
///  A running renewal: a thread that stamps every held claim again each interval until this value is dropped.
/// Why:
///   Dropping stops and joins the thread, so a drain that returns or unwinds leaves no thread behind, and a drain that ended
///  renews nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const renewal = ClaimRenewal.start(workDir, held, every); renewal.stop();
/// ```
pub struct ClaimRenewal {
    /// Set to `true` and signalled to stop the thread.
    stop: Arc<(Mutex<bool>, Condvar)>,
    /// The thread, taken when it is joined.
    worker: Option<JoinHandle<()>>,
}

/// Methods over a renewal.
impl ClaimRenewal {
    /// Start renewing the claims `held` names in `work_dir` every `every`.
    pub fn start(work_dir: PathBuf, held: HeldClaims, every: Duration) -> ClaimRenewal {
        let stop: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));
        let signal: Arc<(Mutex<bool>, Condvar)> = Arc::clone(&stop);
        let worker: JoinHandle<()> = std::thread::spawn(move || {
            let (flag, wake) = &*signal;
            let mut stopped = flag.lock().unwrap_or_else(PoisonError::into_inner);
            loop {
                let (guard, _timeout) = wake.wait_timeout_while(stopped, every, |done| { return !*done; }).unwrap_or_else(PoisonError::into_inner);
                stopped = guard;
                if *stopped {
                    return;
                }
                // A claim that was settled is no longer held; one that was taken over is found gone and marked.
                held.renew(work_dir.as_path(), &ClaimTime::read());
            }
        });
        return ClaimRenewal { stop, worker: Some(worker) };
    }
}

/// Stops the thread when the renewal goes out of scope.
impl Drop for ClaimRenewal {
    fn drop(&mut self) {
        let (flag, wake) = &*self.stop;
        *flag.lock().unwrap_or_else(PoisonError::into_inner) = true;
        wake.notify_all();
        if let Some(worker) = self.worker.take() {
            // A renewal thread that panicked has nothing left to report: the claims it held expire with the lease.
            let _joined = worker.join();
        }
    }
}

/// Renewal controls stay out of the release executable.
#[cfg(test)]
#[path = "service_claim_renewal_tests.rs"]
mod tests;
