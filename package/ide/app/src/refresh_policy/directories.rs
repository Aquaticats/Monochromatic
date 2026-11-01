//! Decide which shown directory the single directory reader lists next.

/// Intervals shared with the source schedule.
use super::{REREAD_GAP, SAFETY_SWEEP, UNWATCHED_DIRECTORY_POLL};
/// What:
///  `BTreeSet<PathBuf>` is an ordered set of owned paths and `BTreeMap<PathBuf, Instant>` an ordered
///       map from path to time (`HashMap` is the unordered sibling);
///  `Instant` is a monotonic time point.
/// Why:
///  Notified directories collapse into one pending entry each,
///  however many events arrived.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = new Set<string>(); const started = new Map<string, number>();
/// ```
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::Instant,
};

/// Directory reread schedule;
///  `shown` is always the root plus visible expanded folders,
///  in visible order.
#[derive(Clone, Debug, Default)]
pub struct DirectoryRefresh {
    /// Notified directories;
    ///  read before anything else except first listings.
    changed: BTreeSet<PathBuf>,
    /// Remaining directories of the current safety sweep;
    ///  read only when nothing else waits.
    sweep: BTreeSet<PathBuf>,
    /// When this schedule last started a read of each directory;
    ///  bounds rereads of a busy folder.
    started: BTreeMap<PathBuf, Instant>,
    /// When the last sweep started;
    ///  `None` until the first schedule call starts the clock.
    last_sweep: Option<Instant>,
    /// When the last unwatched directory was read.
    last_unwatched: Option<Instant>,
    /// What:
    ///  `usize` is the unsigned index type (siblings `u32`,
    ///  `u64`);
    ///  collections index with it.
    /// Why:
    ///  Round-robin position among unwatched shown directories,
    ///  as the old polling kept.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// unwatchedIndex = 0;
    /// ```
    unwatched_index: usize,
}

/// Remove and return the first path of `shown` (visible order) that is pending and not read too recently.
fn first_ready(
    pending: &mut BTreeSet<PathBuf>,
    shown: &[PathBuf],
    started: &BTreeMap<PathBuf, Instant>,
    now: Instant,
) -> Option<PathBuf> {
    if pending.is_empty() {
        return None;
    }
    for path in shown {
        // What: `get` returns `Option<&Instant>`; a missing entry or an old enough one is rested.
        // Why: A folder changing continuously is reread at most once per `REREAD_GAP`; others are not delayed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rested = !started.has(path) || now - started.get(path) >= REREAD_GAP;
        // ```
        let rested = match started.get(path) {
            Some(at) => now.saturating_duration_since(*at) >= REREAD_GAP,
            None => true,
        };
        if rested && pending.remove(path) {
            // `clone` returns an owned copy; the caller keeps its shown list intact.
            return Some(path.clone());
        }
    }
    // Entries that are no longer shown are dropped; a later expansion reads them as a first listing
    // and again when their watch starts. Shown entries inside their gap stay pending.
    let mut doomed: Vec<PathBuf> = Vec::new();
    for path in pending.iter() {
        if !shown.contains(path) {
            doomed.push(path.clone());
        }
    }
    for path in doomed {
        pending.remove(&path);
    }
    return None;
}

/// Pure scheduling:
///  callers pass `shown`,
///  the live-watch set,
///  and the current time.
impl DirectoryRefresh {
    /// A notification or a newly established watch made this directory's listing stale.
    pub fn changed(&mut self, path: PathBuf) {
        self.changed.insert(path);
    }

    /// Overflow,
    ///  an error,
    ///  or a lost watch:
    ///  every shown directory is stale.
    pub fn changed_all(&mut self, shown: &[PathBuf]) {
        for path in shown {
            self.changed.insert(path.clone());
        }
    }

    /// Start a safety sweep when one is due;
    ///  true tells the caller to retry failed watches too.
    pub fn start_sweep_if_due(&mut self, shown: &[PathBuf], now: Instant) -> bool {
        // What: `let ... else` binds the last start time or starts the clock on the first call.
        // Why: Startup already lists everything shown, so the first sweep waits one full interval.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.lastSweep === undefined) { this.lastSweep = now; return false; }
        // ```
        let Some(last) = self.last_sweep else {
            self.last_sweep = Some(now);
            return false;
        };
        if now.saturating_duration_since(last) < SAFETY_SWEEP {
            return false;
        }
        // What: `is_empty` is true once the current pass has read, or dropped as hidden, every directory.
        // Why: One read starts per native tick, so a pass over many directories outlasts the interval.
        //      Refilling the set then would restart at the top of the visible order each interval and
        //      never reach the directories late in it; the next pass waits for this one instead.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.sweep.size > 0) return false;
        // ```
        if !self.sweep.is_empty() {
            return false;
        }
        self.last_sweep = Some(now);
        for path in shown {
            self.sweep.insert(path.clone());
        }
        // Forget read times of folders that are no longer shown, so the map stays as small as the tree.
        let mut doomed: Vec<PathBuf> = Vec::new();
        for path in self.started.keys() {
            if !shown.contains(path) {
                doomed.push(path.clone());
            }
        }
        for path in doomed {
            self.started.remove(&path);
        }
        tracing::debug!(
            directories = shown.len(),
            "started the safety reread of shown directories"
        );
        return true;
    }

    /// Choose without recording:
    ///  notified folders,
    ///  then unwatched ones on their timer,
    ///  then the sweep.
    fn pick(
        &mut self,
        shown: &[PathBuf],
        watched: &BTreeSet<PathBuf>,
        now: Instant,
    ) -> Option<PathBuf> {
        if let Some(path) = first_ready(&mut self.changed, shown, &self.started, now) {
            return Some(path);
        }
        // What: the poll is due for `None` or when the contained time is old enough.
        // Why: Unwatched directories keep the old 500 ms round robin, never faster.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.lastUnwatched === undefined || now - this.lastUnwatched >= 500) { ... }
        // ```
        let unwatched_due = match self.last_unwatched {
            Some(at) => now.saturating_duration_since(at) >= UNWATCHED_DIRECTORY_POLL,
            None => true,
        };
        if unwatched_due {
            let mut unwatched = Vec::new();
            for path in shown {
                if !watched.contains(path) {
                    unwatched.push(path);
                }
            }
            if !unwatched.is_empty() {
                self.unwatched_index %= unwatched.len();
                let path = unwatched[self.unwatched_index].clone();
                self.unwatched_index += 1;
                self.last_unwatched = Some(now);
                return Some(path);
            }
        }
        return first_ready(&mut self.sweep, shown, &self.started, now);
    }

    /// The next directory to list,
    ///  or `None` when nothing is due;
    ///  the caller lists it right away.
    pub fn next(
        &mut self,
        shown: &[PathBuf],
        watched: &BTreeSet<PathBuf>,
        now: Instant,
    ) -> Option<PathBuf> {
        let picked = self.pick(shown, watched, now);
        // What: `if let Some(path) = &picked` borrows the chosen path without taking it out of `picked`.
        // Why: The read starts now; a notification arriving during it waits out the gap from this moment.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (picked !== undefined) this.started.set(picked, now);
        // ```
        if let Some(path) = &picked {
            self.started.insert(path.clone(), now);
        }
        return picked;
    }
}
