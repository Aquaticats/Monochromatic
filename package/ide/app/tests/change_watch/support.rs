//! Shared waits and kernel-state probes for the change-watch integration tests.

/// The watcher under test,
///  its invalidation record,
///  and the canonical project boundary.
use ide_app::{
    change_watch::{ChangeWatcher, Changes},
    workspace::Workspace,
};
/// What:
///  `MetadataExt::ino` reads a file's inode number,
///  which `/proc/self/fdinfo` prints for each inotify watch.
/// Why:
///  A watch removed from the kernel is visible there even when the watcher filters its events anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inode = statSync(path).ino;
/// ```
use std::os::unix::fs::MetadataExt;
/// What:
///  `Duration`/`Instant` bound each wait;
///  `Path` borrows a fixture path and `PathBuf` owns one.
/// Why:
///  Notifications arrive on another thread,
///  so tests poll with a deadline instead of sleeping blindly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadline = performance.now() + 5000;
/// ```
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// Longest wait for a notification that must arrive.
const ARRIVAL: Duration = Duration::from_secs(5);

/// How long silence is observed before concluding that nothing was reported.
const SILENCE: Duration = Duration::from_millis(400);

/// Fold one `take` result into an accumulated record;
///  the latest source change and watched set win.
fn merge(into: &mut Changes, from: Changes) {
    into.directories.extend(from.directories);
    if from.source.is_some() {
        into.source = from.source;
    }
    into.everything = into.everything || from.everything;
    if from.watched.is_some() {
        into.watched = from.watched;
    }
}

/// What:
///  A wait for changes that must arrive.
///  The caller loops `while !condition(&arrival.record) { arrival.again(watcher); }`
///  and takes the record with `into_record`.
/// Why:
///  Notifications arrive on another thread,
///  so the wait accumulates what the watcher publishes and fails the test after a deadline.
pub struct Arrival {
    /// What the wait is for,
    ///  named in the failure when it times out.
    what: String,
    /// When the wait began.
    started: Instant,
    /// Everything reported since the wait began.
    pub record: Changes,
}

impl Arrival {
    /// Begin a wait for `what` and take what the watcher already published.
    pub fn begin(watcher: &mut ChangeWatcher, what: &str) -> Self {
        let mut record = Changes::default();
        merge(&mut record, watcher.take());
        return Self {
            what: what.to_string(),
            started: Instant::now(),
            record,
        };
    }

    /// Fail the test naming what was awaited when five seconds passed,
    ///  else pause and take what the watcher published since.
    pub fn again(&mut self, watcher: &mut ChangeWatcher) {
        assert!(
            self.started.elapsed() < ARRIVAL,
            "{} was not reported: {:?}",
            self.what,
            self.record
        );
        std::thread::sleep(Duration::from_millis(5));
        merge(&mut self.record, watcher.take());
    }

    /// The record the wait accumulated.
    pub fn into_record(self) -> Changes {
        return self.record;
    }
}

/// Everything reported during 400 ms;
///  used to drain earlier notifications and to observe silence.
pub fn quiet(watcher: &mut ChangeWatcher) -> Changes {
    let start = Instant::now();
    let mut record = Changes::default();
    loop {
        merge(&mut record, watcher.take());
        if start.elapsed() >= SILENCE {
            return record;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// True when the record's latest watched set contains `path`.
pub fn watching(record: &Changes, path: &Path) -> bool {
    return match record.watched.as_ref() {
        Some(watched) => watched.contains(path),
        None => false,
    };
}

/// True when the record carries a watched set that lacks `path`.
pub fn not_watching(record: &Changes, path: &Path) -> bool {
    return match record.watched.as_ref() {
        Some(watched) => !watched.contains(path),
        None => false,
    };
}

/// True when the record's latest watched set contains every one of `paths`.
fn watching_all(record: &Changes, paths: &[&Path]) -> bool {
    for path in paths {
        if !watching(record, path) {
            return false;
        }
    }
    return true;
}

/// Wait until every one of `paths` has a live watch,
///  then drain the follow-up invalidations.
pub fn settle(watcher: &mut ChangeWatcher, paths: &[&Path]) {
    let mut arrival = Arrival::begin(watcher, "the watches for the shown folders");
    while !watching_all(&arrival.record, paths) {
        arrival.again(watcher);
    }
    quiet(watcher);
}

/// True when any inotify descriptor of this process holds a watch on `path`'s inode.
pub fn kernel_watches(path: &Path) -> bool {
    let inode = fs::metadata(path).expect("watched fixture metadata").ino();
    let needle = format!(" ino:{inode:x} ");
    for listed in fs::read_dir("/proc/self/fd").expect("descriptor list") {
        let entry = listed.expect("descriptor entry");
        // Descriptors can close between listing and reading; such an entry is simply skipped.
        let Ok(target) = fs::read_link(entry.path()) else {
            continue;
        };
        if target != Path::new("anon_inode:inotify") {
            continue;
        }
        let info_path = Path::new("/proc/self/fdinfo").join(entry.file_name());
        let Ok(info) = fs::read_to_string(info_path) else {
            continue;
        };
        for line in info.lines() {
            if line.starts_with("inotify wd:") && line.contains(&needle) {
                return true;
            }
        }
    }
    return false;
}

/// Start a watcher over a canonical disposable root.
pub fn start(root: &Path) -> (ChangeWatcher, Workspace) {
    let workspace = Workspace::new(root).expect("disposable workspace");
    // `clone` gives the watcher its own copy of the read-only boundary.
    let watcher = ChangeWatcher::new(workspace.clone()).expect("change watcher");
    return (watcher, workspace);
}

/// Owned set of the given directories,
///  as the native tree passes them.
pub fn set(paths: &[&Path]) -> BTreeSet<PathBuf> {
    let mut owned = BTreeSet::new();
    for path in paths {
        owned.insert(path.to_path_buf());
    }
    return owned;
}
