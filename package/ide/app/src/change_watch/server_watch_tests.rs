//! Watches for the language servers with a fake kernel:
//!  shared with the tree,
//!  given up for the tree under
//! the limit,
//!  their own limit state logged once,
//!  and retried after the backoff.

/// The decisions under test.
use super::{ServerRequest, ServerWatches, TreeFirst, reconcile_servers};
/// The tree's decisions,
///  run through the sharing kernel.
use crate::change_watch::reconcile::{Kernel, Request, Watches, reconcile};
/// The backoff's first wait,
///  and why an add failed.
use crate::change_watch::{limit::FIRST_LIMIT_RETRY, watch_ops::WatchFailure};
/// What:
///  ordered path sets;
///  `Path`/`PathBuf` are a borrowed and an owned path;
///  `Duration`/`Instant`
///       a time span and a monotonic time point.
/// Why:
///  Every wake gets a time the test chooses,
///  so the backoff is checked without waiting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const later = start + 2000;
/// ```
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// A kernel with `free` watches left before it answers with the limit,
///  holding the live watches.
struct Fake {
    /// Watches that can still be added before the limit answers.
    free: usize,
    /// Folders with a live kernel watch.
    live: BTreeSet<PathBuf>,
    /// Every add attempted,
    ///  in order.
    adds: Vec<PathBuf>,
}

/// Count calls and play the limit.
impl Kernel for Fake {
    /// Succeed while watches are free,
    ///  then answer with the limit.
    fn add(&mut self, path: &Path) -> Result<(), WatchFailure> {
        self.adds.push(path.to_path_buf());
        assert!(
            !self.live.contains(path),
            "a second kernel watch was added for {}",
            path.display()
        );
        if self.free == 0 {
            return Err(WatchFailure::Limit);
        }
        self.free -= 1;
        self.live.insert(path.to_path_buf());
        return Ok(());
    }

    /// A removed watch is free again.
    fn remove(&mut self, path: &Path) {
        assert!(
            self.live.remove(path),
            "removed a watch that did not exist: {}",
            path.display()
        );
        self.free += 1;
    }
}

/// The paths `/p/<name>` for each name,
///  plus `/p`.
fn folders(names: &[&str]) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::from([PathBuf::from("/p")]);
    for name in names {
        paths.insert(PathBuf::from(format!("/p/{name}")));
    }
    return paths;
}

/// A kernel with this many free watches and nothing watched.
fn kernel(free: usize) -> Fake {
    return Fake {
        free,
        live: BTreeSet::new(),
        adds: Vec::new(),
    };
}

/// A folder the tree and the servers both want keeps one kernel watch,
///  and collapsing it in the tree
/// does not remove the servers' watch.
#[test]
fn a_folder_both_want_keeps_one_watch() {
    let now = Instant::now();
    let mut fake = kernel(100);
    let mut servers = ServerWatches {
        desired: folders(&["a", "b"]),
        ..ServerWatches::default()
    };
    let mut tree = Watches::default();
    {
        let mut shared = TreeFirst {
            inner: &mut fake,
            servers: &mut servers,
            tree_held: BTreeSet::new(),
        };
        let shown = Request {
            desired: Some(folders(&["a"])),
            ..Request::default()
        };
        reconcile(&mut tree, shown, now, &mut shared);
    }
    let request = ServerRequest {
        desired_changed: true,
        tree_limited: false,
    };
    let established = reconcile_servers(&mut servers, &tree.active, request, now, &mut fake);
    assert_eq!(established, folders(&["a", "b"]));
    assert_eq!(fake.live, folders(&["a", "b"]), "a folder got two watches");
    {
        let mut shared = TreeFirst {
            inner: &mut fake,
            servers: &mut servers,
            tree_held: tree.active.clone(),
        };
        let collapsed = Request {
            desired: Some(BTreeSet::new()),
            ..Request::default()
        };
        reconcile(&mut tree, collapsed, now, &mut shared);
    }
    assert_eq!(
        fake.live,
        folders(&["a", "b"]),
        "collapsing in the tree removed the servers' watch"
    );
}

/// A folder the servers already watch is shared when the tree shows it:
///  no second kernel watch is added,
/// and collapsing it in the tree leaves the servers' watch in place.
#[test]
fn a_folder_the_servers_watch_is_shared_with_the_tree() {
    let now = Instant::now();
    let mut fake = kernel(100);
    let mut servers = ServerWatches {
        desired: folders(&["a", "b"]),
        ..ServerWatches::default()
    };
    let request = ServerRequest {
        desired_changed: true,
        tree_limited: false,
    };
    reconcile_servers(&mut servers, &BTreeSet::new(), request, now, &mut fake);
    let mut tree = Watches::default();
    {
        let mut shared = TreeFirst {
            inner: &mut fake,
            servers: &mut servers,
            tree_held: BTreeSet::new(),
        };
        let shown = Request {
            desired: Some(folders(&["a"])),
            ..Request::default()
        };
        reconcile(&mut tree, shown, now, &mut shared);
    }
    assert_eq!(tree.active, folders(&["a"]));
    assert_eq!(
        fake.adds.len(),
        3,
        "the tree added watches the servers already held"
    );
    {
        let mut shared = TreeFirst {
            inner: &mut fake,
            servers: &mut servers,
            tree_held: tree.active.clone(),
        };
        let collapsed = Request {
            desired: Some(BTreeSet::new()),
            ..Request::default()
        };
        reconcile(&mut tree, collapsed, now, &mut shared);
    }
    assert_eq!(
        fake.live,
        folders(&["a", "b"]),
        "collapsing in the tree removed the servers' watch"
    );
}

/// Under the limit,
///  the tree gets the watches:
///  one held only for the servers is given up,
///  the servers
/// enter their own limit state once,
///  and they retry only after the backoff.
#[test]
fn the_tree_comes_first_under_the_limit() {
    let start = Instant::now();
    let mut fake = kernel(3);
    let mut servers = ServerWatches {
        desired: folders(&["s1", "s2"]),
        ..ServerWatches::default()
    };
    let request = ServerRequest {
        desired_changed: true,
        tree_limited: false,
    };
    reconcile_servers(&mut servers, &BTreeSet::new(), request, start, &mut fake);
    assert_eq!(
        servers.active.len(),
        3,
        "the servers did not use the free watches"
    );
    let mut tree = Watches::default();
    {
        let mut shared = TreeFirst {
            inner: &mut fake,
            servers: &mut servers,
            tree_held: BTreeSet::new(),
        };
        let shown = Request {
            desired: Some(BTreeSet::from([PathBuf::from("/shown")])),
            ..Request::default()
        };
        let outcome = reconcile(&mut tree, shown, start, &mut shared);
        assert!(
            !outcome.everything,
            "the tree reached the limit although a server watch could be given up"
        );
    }
    assert!(tree.active.contains(Path::new("/shown")));
    assert_eq!(servers.active.len(), 2, "no server watch was given up");
    assert_eq!(servers.limited.len(), 1);
    // The next pass enters the servers' limit state; the backoff holds further attempts.
    let quiet = ServerRequest::default();
    reconcile_servers(&mut servers, &tree.active, quiet, start, &mut fake);
    assert!(
        servers.limit.is_some(),
        "the servers' limit state was not entered"
    );
    let attempts = fake.adds.len();
    reconcile_servers(
        &mut servers,
        &tree.active,
        quiet,
        start + Duration::from_millis(500),
        &mut fake,
    );
    assert_eq!(
        fake.adds.len(),
        attempts,
        "the servers retried before the backoff allowed it"
    );
    reconcile_servers(
        &mut servers,
        &tree.active,
        quiet,
        start + FIRST_LIMIT_RETRY,
        &mut fake,
    );
    assert_eq!(
        fake.adds.len(),
        attempts + 1,
        "the backoff retry did not happen"
    );
    // A freed watch ends the state.
    fake.free = 1;
    reconcile_servers(
        &mut servers,
        &tree.active,
        quiet,
        start + FIRST_LIMIT_RETRY * 4,
        &mut fake,
    );
    assert!(servers.limited.is_empty());
    assert!(
        servers.limit.is_none(),
        "the servers' limit state outlived the free watch"
    );
}

/// While the tree waits on the limit,
///  the servers add nothing at all.
#[test]
fn the_servers_wait_while_the_tree_waits() {
    let now = Instant::now();
    let mut fake = kernel(100);
    let mut servers = ServerWatches {
        desired: folders(&["a"]),
        ..ServerWatches::default()
    };
    let request = ServerRequest {
        desired_changed: true,
        tree_limited: true,
    };
    let established = reconcile_servers(&mut servers, &BTreeSet::new(), request, now, &mut fake);
    assert!(
        established.is_empty(),
        "the servers added a watch while the tree waited"
    );
    assert!(
        fake.adds.is_empty(),
        "the servers added a watch while the tree waited"
    );
    assert!(servers.limit.is_some());
    // A backoff retry that still finds the tree waiting lengthens the wait instead of leaving it over.
    let later = now + FIRST_LIMIT_RETRY;
    let still = ServerRequest {
        desired_changed: false,
        tree_limited: true,
    };
    reconcile_servers(&mut servers, &BTreeSet::new(), still, later, &mut fake);
    let waiting = match servers.limit {
        Some(backoff) => !backoff.may_retry(later),
        None => false,
    };
    assert!(
        waiting,
        "the servers' wait did not grow while they waited for the tree, so the watch thread would wake every 50 ms"
    );
}

/// Folders no scan wants any more lose their watch,
///  unless the tree holds them.
#[test]
fn unwanted_folders_lose_only_the_servers_watch() {
    let now = Instant::now();
    let mut fake = kernel(100);
    let mut servers = ServerWatches {
        desired: folders(&["a", "b"]),
        ..ServerWatches::default()
    };
    let changed = ServerRequest {
        desired_changed: true,
        tree_limited: false,
    };
    reconcile_servers(&mut servers, &BTreeSet::new(), changed, now, &mut fake);
    servers.desired = BTreeSet::new();
    let tree_active = BTreeSet::from([PathBuf::from("/p/a")]);
    reconcile_servers(&mut servers, &tree_active, changed, now, &mut fake);
    assert!(servers.active.is_empty());
    assert_eq!(
        fake.live,
        BTreeSet::from([PathBuf::from("/p/a")]),
        "the tree's folder lost its watch, or another kept one"
    );
}
