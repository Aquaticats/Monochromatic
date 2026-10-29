//! The watch limit with a fake kernel:
//!  entered once,
//!  no further adds in the same wake,
//!  sweep retries
//! only after the backoff,
//!  an immediate retry when what is shown changes,
//!  and one line when it is left.

/// The decisions under test and the interface the fake implements.
use super::{Kernel, Request, Watches, reconcile};
/// The backoff's first and longest waits,
///  and why an add failed.
use crate::change_watch::{
    limit::{FIRST_LIMIT_RETRY, LONGEST_LIMIT_RETRY},
    watch_ops::WatchFailure,
};
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
///  recording every call.
struct Fake {
    /// Watches that can still be added before the limit answers.
    free: usize,
    /// Every directory an add was attempted for,
    ///  in order.
    adds: Vec<PathBuf>,
}

/// Count calls and play the limit.
impl Kernel for Fake {
    /// Succeed while watches are free,
    ///  then answer with the limit.
    fn add(&mut self, path: &Path) -> Result<(), WatchFailure> {
        self.adds.push(path.to_path_buf());
        if self.free == 0 {
            // What: `Err(WatchFailure::Limit)` is the failure the real kernel returns for `ENOSPC`.
            // Why: The watch thread must treat it as one shared state, not one failure per directory.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { kind: 'limit' };
            // ```
            return Err(WatchFailure::Limit);
        }
        self.free -= 1;
        return Ok(());
    }

    /// A removed watch is free again.
    fn remove(&mut self, _path: &Path) {
        self.free += 1;
    }
}

/// The paths `/p` and `/p/<name>` for each name.
fn shown(names: &[&str]) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::new();
    paths.insert(PathBuf::from("/p"));
    for name in names {
        paths.insert(PathBuf::from(format!("/p/{name}")));
    }
    return paths;
}

/// A wake that brings a new desired set,
///  as an expansion does.
fn expanded(names: &[&str]) -> Request {
    return Request {
        desired: Some(shown(names)),
        ..Request::default()
    };
}

/// A wake that only asks for a retry,
///  as the safety sweep does while something lacks a watch.
fn sweep() -> Request {
    return Request {
        retry: true,
        ..Request::default()
    };
}

/// The first limit failure enters the state once and stops adding in that wake;
///  sweeps then retry only
/// after a doubling wait,
///  while a change of what is shown retries at once and does not lengthen the wait.
#[test]
fn the_watch_limit_is_one_state_with_backoff() {
    let start = Instant::now();
    let mut watches = Watches::default();
    let mut kernel = Fake {
        free: 2,
        adds: Vec::new(),
    };
    let entered = reconcile(
        &mut watches,
        expanded(&["a", "b", "c", "d"]),
        start,
        &mut kernel,
    );
    assert!(
        entered.everything,
        "reaching the limit did not reread everything once"
    );
    assert_eq!(
        kernel.adds.len(),
        3,
        "adds continued after the limit answered: {:?}",
        kernel.adds
    );
    assert_eq!(
        watches.limited.len(),
        3,
        "the directories without a watch were not kept for a retry"
    );
    let early = reconcile(
        &mut watches,
        sweep(),
        start + FIRST_LIMIT_RETRY - Duration::from_millis(1),
        &mut kernel,
    );
    assert_eq!(
        kernel.adds.len(),
        3,
        "a sweep retried before the backoff allowed it"
    );
    assert!(
        !early.everything,
        "a sweep under the limit reread everything"
    );
    let first = reconcile(
        &mut watches,
        sweep(),
        start + FIRST_LIMIT_RETRY,
        &mut kernel,
    );
    assert_eq!(
        kernel.adds.len(),
        4,
        "the sweep after the backoff did not try exactly once"
    );
    assert!(
        !first.everything,
        "a retry that hit the limit again reread everything"
    );
    let shown_at = start + FIRST_LIMIT_RETRY + Duration::from_millis(1);
    let changed = reconcile(
        &mut watches,
        expanded(&["a", "b", "c", "d", "e"]),
        shown_at,
        &mut kernel,
    );
    assert_eq!(
        kernel.adds.len(),
        5,
        "an expansion did not try at once, or tried more than once"
    );
    assert!(
        !changed.everything,
        "an expansion under the limit reread everything"
    );
    // The failed sweep retry doubled the wait to twice the first one.
    let doubled = start + FIRST_LIMIT_RETRY + FIRST_LIMIT_RETRY * 2;
    reconcile(
        &mut watches,
        sweep(),
        doubled - Duration::from_millis(1),
        &mut kernel,
    );
    assert_eq!(
        kernel.adds.len(),
        5,
        "the wait did not double after a failed retry"
    );
    reconcile(&mut watches, sweep(), doubled, &mut kernel);
    assert_eq!(kernel.adds.len(), 6, "the doubled wait was not honored");
}

/// When watches become free,
///  the next allowed retry watches everything and leaves the state.
#[test]
fn freed_watches_end_the_limit_state() {
    let start = Instant::now();
    let mut watches = Watches::default();
    let mut kernel = Fake {
        free: 1,
        adds: Vec::new(),
    };
    reconcile(&mut watches, expanded(&["a", "b"]), start, &mut kernel);
    assert!(watches.limit.is_some(), "the limit state was not entered");
    kernel.free = 10;
    let restored = reconcile(
        &mut watches,
        sweep(),
        start + FIRST_LIMIT_RETRY,
        &mut kernel,
    );
    assert!(
        watches.limit.is_none(),
        "the limit state outlived free watches"
    );
    assert!(
        watches.limited.is_empty(),
        "directories still wait on the limit"
    );
    assert_eq!(
        restored.established.len(),
        2,
        "the waiting directories were not watched"
    );
    assert!(!restored.everything, "leaving the limit reread everything");
}

/// The displayed file's folder is tried first,
///  so with one watch left the source stays watched.
#[test]
fn the_displayed_files_folder_is_watched_first() {
    let start = Instant::now();
    let mut watches = Watches::default();
    let mut kernel = Fake {
        free: 1,
        adds: Vec::new(),
    };
    let mut request = expanded(&["a", "z"]);
    request.file = Some(PathBuf::from("/p/z/view.txt"));
    reconcile(&mut watches, request, start, &mut kernel);
    assert_eq!(
        kernel.adds.first().map(PathBuf::as_path),
        Some(Path::new("/p/z")),
        "the displayed file's folder was not tried first"
    );
    assert!(
        watches.active.contains(Path::new("/p/z")),
        "the displayed file's folder has no watch"
    );
}

/// The wait doubles on every failed sweep retry but never exceeds the longest wait.
#[test]
fn the_backoff_is_capped() {
    let start = Instant::now();
    let mut backoff = crate::change_watch::LimitBackoff::reached(start);
    let mut now = start;
    for _ in 0..20 {
        now += backoff.wait();
        backoff.failed_again(now);
    }
    assert_eq!(
        backoff.wait(),
        LONGEST_LIMIT_RETRY,
        "the wait grew past its cap"
    );
    assert!(!backoff.may_retry(now + LONGEST_LIMIT_RETRY - Duration::from_millis(1)));
    assert!(backoff.may_retry(now + LONGEST_LIMIT_RETRY));
}

/// Scrolling the tree retries a watch waiting on the limit at once,
///  without waiting for the backoff,
/// and a failed user retry does not lengthen the backoff.
#[test]
fn a_user_retry_skips_the_backoff() {
    let start = Instant::now();
    let mut watches = Watches::default();
    let mut kernel = Fake {
        free: 1,
        adds: Vec::new(),
    };
    reconcile(&mut watches, expanded(&["a", "b"]), start, &mut kernel);
    let before = kernel.adds.len();
    let scrolled = Request {
        user_retry: true,
        ..Request::default()
    };
    let soon = start + Duration::from_millis(100);
    let outcome = reconcile(&mut watches, scrolled, soon, &mut kernel);
    assert_eq!(
        kernel.adds.len(),
        before + 1,
        "a scroll did not retry at once"
    );
    assert!(
        !outcome.everything,
        "a user retry under the limit reread everything"
    );
    let may_retry = match watches.limit {
        Some(backoff) => backoff.may_retry(start + FIRST_LIMIT_RETRY),
        None => false,
    };
    assert!(may_retry, "a failed user retry lengthened the backoff");
}
