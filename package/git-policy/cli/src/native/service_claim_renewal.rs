//! What:
//!  Keeps the lease of the claims one service instance holds fresh while it works, so a claim in a work directory that several
//!  instances drain is taken over only when its holder stopped, never because its holder is slow.
//! Why:
//!  A claim's lease starts at its stamp and the other instances run a claim again once the lease ran out. A push can take
//!  longer than any fixed lease, and an item can wait behind other claimed work; a stamp that is never renewed would make
//!  both look abandoned. A holder that dies stops renewing, so its claims still expire, and no lock file is involved: the
//!  claim file's modification time is the whole protocol.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const renewal = startRenewal(workDir, held, lease / 3); try { await work() } finally { renewal.stop() }
//! ```

use super::work_item::renew_claim;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

/// How many times inside one lease a holder renews, so a renewal that is late by up to two intervals still beats the lease.
pub const RENEWALS_PER_LEASE: u32 = 3;

/// What:
///  The ids of the claims one drain holds right now.
/// Why:
///   The renewal must refresh only what this instance holds: refreshing every file in the directory would keep the dead
///  holder's claims alive for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldClaims = { ids: Set<string> };
/// ```
#[derive(Clone, Debug, Default)]
pub struct HeldClaims {
    /// Shared with the renewal thread.
    ids: Arc<Mutex<Vec<String>>>,
}

/// Methods over the held claims.
impl HeldClaims {
    /// Add a claim the drain took.
    pub fn hold(&self, id: &str) {
        self.ids.lock().unwrap_or_else(PoisonError::into_inner).push(id.to_owned());
    }

    /// Forget a claim the drain settled, so it is no longer renewed.
    pub fn release(&self, id: &str) {
        self.ids.lock().unwrap_or_else(PoisonError::into_inner).retain(|held| { return held != id; });
    }

    /// The ids held at this moment.
    pub fn snapshot(&self) -> Vec<String> {
        return self.ids.lock().unwrap_or_else(PoisonError::into_inner).clone();
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
                for id in held.snapshot() {
                    // A claim that was settled or taken over since the snapshot is gone from the directory; renewing it finds nothing.
                    let _renewed = renew_claim(work_dir.as_path(), id.as_str(), SystemTime::now());
                }
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
