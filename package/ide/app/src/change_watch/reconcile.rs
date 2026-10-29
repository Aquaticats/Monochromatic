//! Decide which watches to add and remove for one wake of the watch thread.
//! The watch calls are passed in,
//!  so tests drive this with failures they choose,
//!  the watch limit included.

/// The watch-limit backoff,
///  and why an add failed.
use super::{limit::LimitBackoff, watch_ops::WatchFailure};
/// What:
///  ordered sets and maps of owned paths;
///  `Path` borrows a path;
///  `Instant` is a monotonic time point.
/// Why:
///  Watches are keyed by directory,
///  and the limit backoff runs on the monotonic clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const active = new Set<string>(); const failed = new Map<string, string>();
/// ```
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::Instant,
};

/// The kernel calls a wake makes.
///
/// What:
///  a `trait` is an interface:
///  any type with these two methods can be passed where a `Kernel` is expected.
/// Why:
///  The watch thread passes real inotify calls;
///  tests pass fakes that fail on purpose,
///  the limit included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Kernel { add(path: string): WatchFailure | undefined; remove(path: string): void }
/// ```
pub trait Kernel {
    /// Watch one directory non-recursively.
    fn add(&mut self, path: &Path) -> Result<(), WatchFailure>;
    /// Stop watching one directory.
    fn remove(&mut self, path: &Path);
}

/// Mutable bookkeeping owned by the watch thread alone.
#[derive(Debug, Default)]
pub struct Watches {
    /// Directories with a live watch.
    pub active: BTreeSet<PathBuf>,
    /// What:
    ///  `BTreeMap<PathBuf, String>` maps each directory whose last watch attempt failed to that
    ///       failure's text (`BTreeSet`,
    ///  the sibling,
    ///  would hold the directories without their text).
    /// Why:
    ///  Failed directories are retried only on request,
    ///  and a retry that fails with the same text
    ///      is neither logged nor reported again,
    ///  so a retry every sweep does not repeat itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const failed = new Map<string, string>();
    /// ```
    pub failed: BTreeMap<PathBuf, String>,
    /// Directories without a watch because the inotify watch limit is reached.
    pub limited: BTreeSet<PathBuf>,
    /// What:
    ///  `Option<LimitBackoff>` is the backoff while the limit is reached,
    ///  or `None` (`LimitBackoff | undefined`).
    /// Why:
    ///  The limit is one state:
    ///  entering and leaving it are logged once,
    ///  and sweep retries back off.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// limit?: LimitBackoff;
    /// ```
    pub limit: Option<LimitBackoff>,
    /// The UI's latest desired set.
    pub desired: BTreeSet<PathBuf>,
    /// The displayed file as of the previous wake,
    ///  to notice a switch to another file.
    pub file: Option<PathBuf>,
}

/// One wake's requests,
///  taken from the shared state.
#[derive(Debug, Default)]
pub struct Request {
    /// A new desired set:
    ///  the user expanded,
    ///  collapsed,
    ///  or switched files.
    pub desired: Option<BTreeSet<PathBuf>>,
    /// The safety sweep asked to retry watches that failed.
    pub retry: bool,
    /// The user acted on the tree;
    ///  watches waiting on the limit are tried without the backoff.
    pub user_retry: bool,
    /// Live watches whose directory was removed or renamed.
    pub stale: BTreeSet<PathBuf>,
    /// The displayed file now.
    pub file: Option<PathBuf>,
}

/// What one wake changed,
///  for the UI.
#[derive(Debug, Default)]
pub struct Outcome {
    /// Directories that got a watch;
    ///  each is read once more.
    pub established: BTreeSet<PathBuf>,
    /// Reread everything shown once:
    ///  a directory failed anew,
    ///  or the watch limit was just reached.
    pub everything: bool,
    /// The displayed file changed to another one.
    pub switched: bool,
}

/// Which retries this wake allows.
struct Allowed {
    /// The UI asked to retry directories that failed for another reason than the limit.
    failed: bool,
    /// Directories waiting on the limit may be tried:
    ///  what is shown changed,
    ///  or the backoff allows a sweep retry.
    limited: bool,
}

/// What a pass over the desired directories found.
#[derive(Default)]
struct Pass {
    /// Directories that got a watch.
    established: BTreeSet<PathBuf>,
    /// A directory failed anew for another reason than the limit.
    lost: bool,
    /// Some add answered with the watch limit.
    limit_hit: bool,
}

/// Desired directories in the order to try them:
///  the displayed file's folder first,
///  then by path,
/// so the root precedes its descendants.
///  With only a few watches left,
///  the source stays watched.
fn attempt_order(desired: &BTreeSet<PathBuf>, file: Option<&Path>) -> Vec<PathBuf> {
    let mut order = Vec::new();
    // What: `and_then(Path::parent)` takes the optional file's folder; `filter` keeps it only when desired.
    // Why: The displayed source is what the user is looking at.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parent = file === undefined ? undefined : dirname(file); if (parent && desired.has(parent)) order.push(parent);
    // ```
    let mut parent = None;
    if let Some(directory) = file.and_then(Path::parent)
        && desired.contains(directory)
    {
        parent = Some(directory);
    }
    if let Some(directory) = parent {
        order.push(directory.to_path_buf());
    }
    for path in desired {
        if Some(path.as_path()) != parent {
            order.push(path.clone());
        }
    }
    return order;
}

/// Try every desired directory that lacks a watch and that this wake allows.
/// After the first limit failure,
///  the rest wait on the limit without another failing call,
/// because the limit counts everything this user runs and the next add would fail the same way.
fn try_watches(watches: &mut Watches, allowed: &Allowed, kernel: &mut dyn Kernel) -> Pass {
    let mut pass = Pass::default();
    for path in attempt_order(&watches.desired, watches.file.as_deref()) {
        if watches.active.contains(&path) {
            continue;
        }
        // A directory that already failed is tried again only when the UI asks for a retry.
        if !allowed.failed && watches.failed.contains_key(&path) {
            continue;
        }
        // A directory waiting on the limit is tried again only when this wake allows it.
        if !allowed.limited && watches.limited.contains(&path) {
            continue;
        }
        if pass.limit_hit {
            watches.limited.insert(path);
            continue;
        }
        match kernel.add(&path) {
            Ok(()) => {
                // What: `remove` returns `Some(message)` when the directory had failed before.
                // Why: A watch that works again after its own failure is a state change worth one log line;
                //      one that only waited on the limit is covered by the limit's own line.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (failed.delete(path)) log.info('restored'); else log.debug('watching');
                // ```
                if watches.failed.remove(&path).is_some() {
                    tracing::info!(path = %path.display(), "directory watch restored after an earlier failure");
                } else {
                    tracing::debug!(path = %path.display(), "watching directory");
                }
                watches.limited.remove(&path);
                watches.active.insert(path.clone());
                pass.established.insert(path);
            }
            Err(WatchFailure::Limit) => {
                watches.failed.remove(&path);
                watches.limited.insert(path);
                pass.limit_hit = true;
            }
            Err(WatchFailure::Other(message)) => {
                watches.limited.remove(&path);
                // The same failure as last time is neither logged nor reported again.
                if watches.failed.get(&path) == Some(&message) {
                    continue;
                }
                tracing::warn!(%message, "directory watch failed; rereading it on a timer and rereading everything shown");
                watches.failed.insert(path, message);
                pass.lost = true;
            }
        }
    }
    return pass;
}

/// Enter,
///  keep,
///  or leave the watch-limit state after a pass;
///  true when it was just entered.
/// `sweep_retry` says the pass retried limited directories because the backoff allowed it.
fn update_limit(watches: &mut Watches, pass: &Pass, sweep_retry: bool, now: Instant) -> bool {
    if pass.limit_hit {
        // What: `if let Some(backoff) = &mut ...` lends the running backoff for changing.
        // Why: Only entering the state is logged and rereads everything; later failures lengthen the wait.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (limit !== undefined) { if (sweepRetry) limit.failedAgain(now); return false; }
        // ```
        if let Some(backoff) = &mut watches.limit {
            if sweep_retry {
                backoff.failed_again(now);
                tracing::debug!(retry_after = ?backoff.wait(), "inotify watch limit still reached");
            }
            return false;
        }
        let backoff = LimitBackoff::reached(now);
        tracing::warn!(
            directories = watches.limited.len(),
            retry_after = ?backoff.wait(),
            "the inotify watch limit (fs.inotify.max_user_watches) is reached, so some shown directories have no watch; they are reread on timers and the safety sweep, and watching is retried when what is shown changes and after a growing wait; the limit counts the watches of every program this user runs"
        );
        watches.limit = Some(backoff);
        return true;
    }
    if watches.limit.is_some() && watches.limited.is_empty() {
        tracing::info!("inotify watches are available again; every shown directory is watched");
        watches.limit = None;
    }
    return false;
}

/// Apply one wake:
///  forget stale and hidden watches first,
///  which frees watches under the limit,
/// then add what is shown and allowed.
///  `&mut dyn Kernel` lends any value implementing `Kernel`.
pub fn reconcile(
    watches: &mut Watches,
    request: Request,
    now: Instant,
    kernel: &mut dyn Kernel,
) -> Outcome {
    let shown_changed = request.desired.is_some();
    if let Some(next) = request.desired {
        watches.desired = next;
    }
    // The handler reports only the file named in the shared state, so a change made to a newly
    // displayed file before the switch reached that state went unrecorded.
    let switched = request.file.is_some() && request.file != watches.file;
    watches.file = request.file;
    for path in &request.stale {
        if watches.active.remove(path) {
            kernel.remove(path);
        }
    }
    // Collect first: the set cannot change while it is being iterated.
    let mut hidden = Vec::new();
    for path in watches.active.difference(&watches.desired) {
        hidden.push(path.clone());
    }
    for path in &hidden {
        watches.active.remove(path);
        kernel.remove(path);
        tracing::debug!(path = %path.display(), "stopped watching a directory that is no longer shown");
    }
    // What: the entries are taken out and only the desired ones are put back, in their order.
    // Why: A directory that is no longer shown must not be retried later, and its failure is forgotten.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const path of failed.keys()) if (!desired.has(path)) failed.delete(path);
    // ```
    let desired = &watches.desired;
    let failed = std::mem::take(&mut watches.failed);
    for (path, message) in failed {
        if desired.contains(&path) {
            watches.failed.insert(path, message);
        }
    }
    let limited = std::mem::take(&mut watches.limited);
    for path in limited {
        if desired.contains(&path) {
            watches.limited.insert(path);
        }
    }
    // What: `match` asks the optional backoff whether a sweep may retry now.
    // Why: Under the limit a sweep retries only when the backoff allows; what is shown changing retries at once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const sweepRetry = retry && limit !== undefined && limit.mayRetry(now);
    // ```
    let backoff_allows = match watches.limit {
        Some(backoff) => backoff.may_retry(now),
        None => false,
    };
    let sweep_retry = request.retry && backoff_allows;
    let allowed = Allowed {
        failed: request.retry,
        limited: shown_changed || sweep_retry || request.user_retry,
    };
    let pass = try_watches(watches, &allowed, kernel);
    let entered = update_limit(watches, &pass, sweep_retry, now);
    return Outcome {
        everything: pass.lost || entered,
        established: pass.established,
        switched,
    };
}

/// Fake watch calls drive the limit,
///  retries,
///  and failures with chosen times.
#[cfg(test)]
#[path = "reconcile_tests.rs"]
mod tests;
