//! Watches for the language servers:
//!  every source folder of the project,
//!  after the tree and the displayed
//! file,
//!  sharing the one inotify instance and its limit.
//!
//! A folder can be wanted by the tree and by the servers at once,
//!  and inotify keeps one watch per folder,
//! so neither side removes a watch the other still holds.
//!  When the watch limit refuses a watch the tree
//! wants,
//!  a watch held only for the servers is given up to make room:
//!  what the user sees comes first.

/// The kernel calls,
///  why an add failed,
///  and the limit backoff,
///  shared with the tree's watches.
use super::{limit::LimitBackoff, reconcile::Kernel, watch_ops::WatchFailure};
/// What:
///  ordered sets of owned paths;
///  `Path` borrows a path;
///  `Instant` is a monotonic time point.
/// Why:
///  Watches are keyed by folder,
///  and the backoff runs on the monotonic clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const active = new Set<string>();
/// ```
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::Instant,
};

/// The servers' watch bookkeeping,
///  owned by the watch thread alone.
#[derive(Debug, Default)]
pub struct ServerWatches {
    /// Folders the latest scans found:
    ///  source folders and empty folders watched provisionally.
    pub desired: BTreeSet<PathBuf>,
    /// Empty folders among `desired`;
    ///  their first change asks to classify them.
    pub provisional: BTreeSet<PathBuf>,
    /// Folders the servers hold a watch on,
    ///  alone or together with the tree.
    pub active: BTreeSet<PathBuf>,
    /// Desired folders without a watch because the inotify watch limit is reached or the tree needed it.
    pub limited: BTreeSet<PathBuf>,
    /// Desired folders whose watch failed for another reason (vanished,
    ///  refused);
    ///  not retried until rescanned.
    pub failed: BTreeSet<PathBuf>,
    /// What:
    ///  `Option<LimitBackoff>` is the backoff while some desired folder waits on the limit,
    ///  or `None`.
    /// Why:
    ///  The servers' limit is its own state:
    ///  logged once on entry and once on exit,
    ///  retried with backoff.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// limit?: LimitBackoff;
    /// ```
    pub limit: Option<LimitBackoff>,
}

/// What a pass may do,
///  taken from the wake.
#[derive(Clone, Copy, Debug, Default)]
pub struct ServerRequest {
    /// A scan changed the desired folders:
    ///  limited folders are tried at once.
    pub desired_changed: bool,
    /// Some folder the tree or the displayed file wants waits on the limit:
    ///  the servers add nothing.
    pub tree_limited: bool,
}

/// Forget everything below `base` (the base included) in a set of folders.
pub(super) fn forget_below(set: &mut BTreeSet<PathBuf>, base: &Path) {
    // What: only the entries outside `base` are put back; `starts_with` compares whole components.
    // Why: A removed or rescanned folder takes its subfolders with it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const path of set) if (isInside(path, base)) set.delete(path);
    // ```
    let held = std::mem::take(set);
    for path in held {
        if !path.starts_with(base) {
            set.insert(path);
        }
    }
}

/// Remove the watches the servers no longer want,
///  then add the desired ones this pass allows.
/// `tree_active` names the folders the tree holds;
///  their kernel watch exists already and must stay.
/// Returns the folders that got a watch for the servers.
pub fn reconcile_servers(
    servers: &mut ServerWatches,
    tree_active: &BTreeSet<PathBuf>,
    request: ServerRequest,
    now: Instant,
    kernel: &mut dyn Kernel,
) -> BTreeSet<PathBuf> {
    // Collect first: the set cannot change while it is being iterated.
    let unwanted: Vec<PathBuf> = servers
        .active
        .difference(&servers.desired)
        .cloned()
        .collect();
    for path in &unwanted {
        servers.active.remove(path);
        if !tree_active.contains(path) {
            kernel.remove(path);
        }
    }
    let desired = &servers.desired;
    let limited = std::mem::take(&mut servers.limited);
    for path in limited {
        if desired.contains(&path) {
            servers.limited.insert(path);
        }
    }
    let failed = std::mem::take(&mut servers.failed);
    for path in failed {
        if desired.contains(&path) {
            servers.failed.insert(path);
        }
    }
    // What: `match` asks the optional backoff whether its wait is over.
    // Why: A folder waiting on the limit is retried after the backoff, or at once after a new scan.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const retryLimited = desiredChanged || (limit !== undefined && limit.mayRetry(now));
    // ```
    let backoff_retry = match servers.limit {
        Some(backoff) => backoff.may_retry(now),
        None => false,
    };
    let retry_limited = request.desired_changed || backoff_retry;
    let mut established = BTreeSet::new();
    let mut limit_hit = false;
    let missing: Vec<PathBuf> = servers
        .desired
        .difference(&servers.active)
        .cloned()
        .collect();
    let yielded = request.tree_limited && !missing.is_empty();
    for path in missing {
        if servers.failed.contains(&path) {
            continue;
        }
        // The tree and the displayed file come first: while they wait on the limit, the servers wait too.
        if request.tree_limited || limit_hit {
            servers.limited.insert(path);
            continue;
        }
        if servers.limited.contains(&path) && !retry_limited {
            continue;
        }
        // A folder the tree already watches needs no second watch, and no call.
        if tree_active.contains(&path) {
            servers.limited.remove(&path);
            servers.active.insert(path.clone());
            established.insert(path);
            continue;
        }
        match kernel.add(&path) {
            Ok(()) => {
                servers.limited.remove(&path);
                servers.active.insert(path.clone());
                established.insert(path);
            }
            Err(WatchFailure::Limit) => {
                servers.limited.insert(path);
                limit_hit = true;
            }
            Err(WatchFailure::Other(message)) => {
                tracing::debug!(%message, "a folder for the language servers cannot be watched");
                servers.limited.remove(&path);
                servers.failed.insert(path);
            }
        }
    }
    // Waiting for the tree counts as a retry in vain too; otherwise the wait would stay over and the watch
    // thread would wake at its shortest sleep for as long as the tree waits on the limit.
    update_limit(servers, backoff_retry && (limit_hit || yielded), now);
    if !established.is_empty() {
        tracing::debug!(
            folders = established.len(),
            "watching project folders for the language servers"
        );
    }
    return established;
}

/// Enter,
///  keep,
///  or leave the servers' limit state;
///  each change of state is logged once.
/// `retried_in_vain` says a backoff retry met the limit again,
///  which lengthens the wait.
fn update_limit(servers: &mut ServerWatches, retried_in_vain: bool, now: Instant) {
    if servers.limited.is_empty() {
        if servers.limit.take().is_some() {
            tracing::info!(
                "every project folder is watched for the language servers again; they hear about changes made outside the IDE"
            );
        }
        return;
    }
    // What: `if let Some(backoff) = &mut ...` lends the running backoff for changing.
    // Why: Only entering the state is logged; later refusals lengthen the wait.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (limit !== undefined) { if (retriedInVain) limit.failedAgain(now); return; }
    // ```
    if let Some(backoff) = &mut servers.limit {
        if retried_in_vain {
            backoff.failed_again(now);
            tracing::debug!(retry_after = ?backoff.wait(), folders = servers.limited.len(), "project folders for the language servers still wait on the inotify watch limit");
        }
        return;
    }
    let backoff = LimitBackoff::reached(now);
    tracing::warn!(
        folders = servers.limited.len(),
        retry_after = ?backoff.wait(),
        "the inotify watch limit (fs.inotify.max_user_watches) is reached, so some project folders have no watch for the language servers, which do not hear about changes made there outside the IDE; shown folders and the displayed file are watched first, and these folders are retried after a growing wait; the limit counts the watches of every program this user runs"
    );
    servers.limit = Some(backoff);
}

/// The tree's kernel while the servers hold watches too:
///  a folder both want keeps its one watch,
///  and a
/// watch held only for the servers is given up when the limit refuses one the tree wants.
pub struct TreeFirst<'a> {
    /// What:
    ///  `&'a mut dyn Kernel` lends the real kernel calls for as long as `'a`.
    /// Why:
    ///  The watch thread owns the watcher;
    ///  a wake only borrows it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// inner: Kernel;
    /// ```
    pub inner: &'a mut dyn Kernel,
    /// The servers' bookkeeping,
    ///  changed when one of their watches is given up.
    pub servers: &'a mut ServerWatches,
    /// Folders the tree held before this wake or got a watch for during it;
    ///  never given up.
    pub tree_held: BTreeSet<PathBuf>,
}

/// Share watches with the servers,
///  and take one of theirs when the limit refuses the tree.
impl Kernel for TreeFirst<'_> {
    /// Watch a folder for the tree,
    ///  unless the servers' watch already covers it.
    fn add(&mut self, path: &Path) -> Result<(), WatchFailure> {
        if self.servers.active.contains(path) {
            self.tree_held.insert(path.to_path_buf());
            return Ok(());
        }
        loop {
            match self.inner.add(path) {
                Err(WatchFailure::Limit) => {}
                other => {
                    // What: `if other.is_ok()` records success before handing the result back unchanged.
                    // Why: A folder the tree got must never be given up for the servers later in this wake.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (!(other instanceof Error)) treeHeld.add(path); return other;
                    // ```
                    if other.is_ok() {
                        self.tree_held.insert(path.to_path_buf());
                    }
                    return other;
                }
            }
            // What: `find` returns the first watch held only for the servers, or `None`; `cloned` copies it.
            // Why: Giving one up frees one watch for the tree; with none left, the tree waits on the limit.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const victim = [...servers.active].find(path => !treeHeld.has(path));
            // ```
            let mut candidate = None;
            for held in self.servers.active.iter() {
                if !self.tree_held.contains(held) {
                    candidate = Some(held.clone());
                    break;
                }
            }
            let Some(victim) = candidate else {
                return Err(WatchFailure::Limit);
            };
            tracing::debug!(tree = %path.display(), servers = %victim.display(), "gave up a language-server watch so a shown folder can be watched");
            self.inner.remove(&victim);
            self.servers.active.remove(&victim);
            self.servers.limited.insert(victim);
        }
    }

    /// Stop the tree's watch,
    ///  unless the servers still hold the folder.
    fn remove(&mut self, path: &Path) {
        self.tree_held.remove(path);
        if self.servers.active.contains(path) {
            return;
        }
        self.inner.remove(path);
    }
}

/// Sharing,
///  yielding,
///  and the servers' limit state with fake kernel calls.
#[cfg(test)]
#[path = "server_watch_tests.rs"]
mod tests;
