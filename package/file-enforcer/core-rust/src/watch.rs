//! What:
//!  The server side of `watch`:
//! keep the generated files current and warn when one is edited by hand.
//! Why:
//!  The server watches every path the last run depended on (inputs,
//! outputs,
//! listed directories,
//! the nearest existing
//!  ancestor of an absent one,
//! and the config),
//! re-runs when something changes,
//! and so also notices files and
//!  directories that appear later.
//! Its watch set is rebuilt after every cycle from what the run actually read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function watchConnection(shared: Shared, sink: RpcSink, configPath: string): Promise<void>
//! ```

/// Imports `CollectingSink`,
/// `Event`,
/// `EventSink` from `crate::event`.
use crate::event::{CollectingSink, Event, EventSink};
/// Imports `STAGING_MARKER` from `crate::publish`.
use crate::publish::STAGING_MARKER;
/// Imports `EXIT_CLEAN`,
/// `RunMode`,
/// `RunRequest`,
/// `WatchPaths`,
/// `run_detailed` from `crate::run`.
use crate::run::{EXIT_CLEAN, RunMode, RunRequest, WatchPaths, run_detailed};
/// Imports `Shared` from `crate::server`.
use crate::server::Shared;
/// Imports `RpcSink` from `crate::server_connection`.
use crate::server_connection::RpcSink;
/// Imports `RecommendedWatcher`,
/// `RecursiveMode`,
/// `Watcher` from `notify`.
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `BTreeSet` from `std::collections`.
use std::collections::BTreeSet;
/// Imports `Path`,
/// `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Ordering` from `std::sync::atomic`.
use std::sync::atomic::Ordering;
/// Imports `RecvTimeoutError`,
/// `channel` from `std::sync::mpsc`.
use std::sync::mpsc::{RecvTimeoutError, channel};
/// Imports `Duration`,
/// `Instant` from `std::time`.
use std::time::{Duration, Instant};

/// How long the file system must stay quiet before a cycle runs.
const QUIET_PERIOD: Duration = Duration::from_millis(200);
/// How often the loop pings the client and checks for shutdown.
const TICK: Duration = Duration::from_millis(500);
/// The longest a burst of events may extend the quiet-period wait.
const MAX_DEBOUNCE: Duration = Duration::from_secs(2);

/// What:
///  The nearest ancestor of a path that exists,
/// or the path itself when it does.
/// Why:
///  An absent fan-out root is watched through its nearest existing ancestor,
/// so its creation is noticed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nearestExisting(path: string): string
/// ```
fn nearest_existing(path: &Path) -> PathBuf {
    let mut current = path.to_path_buf();
    while !current.exists() {
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
    return current;
}

/// What:
///  The directories to watch for a run's paths and the config file.
///  `BTreeSet` is a sorted set (TS:
/// `Set`,
/// iterated in order).
/// Why:
///  Files are watched through their parent directory so creation,
/// replacement and deletion all show up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchTargets(root: string, config: string, paths: WatchPaths): Set<string>
/// ```
fn watch_targets(root: &Path, config: &Path, paths: &WatchPaths) -> BTreeSet<PathBuf> {
    let mut targets: BTreeSet<PathBuf> = BTreeSet::new();
    if let Some(parent) = config.parent() {
        targets.insert(nearest_existing(parent));
    }
    for file in &paths.files {
        let parent = Path::new(file).parent().map(Path::to_path_buf).unwrap_or_default();
        targets.insert(nearest_existing(&root.join(parent)));
    }
    for directory in &paths.directories {
        targets.insert(nearest_existing(&root.join(directory)));
    }
    return targets;
}

/// What:
///  Bring the watcher's registrations in line with the wanted set.
/// Why:
///  Newly discovered directories are added,
/// directories no longer needed are dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncWatches(watcher: Watcher, current: Set<string>, wanted: Set<string>): void
/// ```
fn sync_watches(watcher: &mut RecommendedWatcher, current: &mut BTreeSet<PathBuf>, wanted: &BTreeSet<PathBuf>) {
    for stale in current.difference(wanted).cloned().collect::<Vec<PathBuf>>() {
        let _ = watcher.unwatch(&stale);
        current.remove(&stale);
    }
    for added in wanted.difference(&current.clone()) {
        if watcher.watch(added, RecursiveMode::NonRecursive).is_ok() {
            current.insert(added.clone());
        }
    }
}

/// What:
///  Run one request under the shared run lock.
/// Why:
///  A watch cycle is a run like any other and must not overlap one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runLocked(shared: Shared, request: RunRequest, sink: EventSink, paths: WatchPaths): number
/// ```
fn run_locked(shared: &Shared, request: &RunRequest, sink: &mut dyn EventSink, paths: &mut WatchPaths) -> i32 {
    let _guard = shared.run_lock.lock();
    return run_detailed(request, &shared.registry, sink, paths);
}

/// What:
///  Run one watch cycle:
/// check,
/// warn about hand edits,
/// then write.
/// Why:
///  A drifted file that was itself among the changed paths was edited by hand;
/// one that drifted because an input
///  changed was not,
/// and gets no warning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cycle(shared: Shared, sink: RpcSink, request: RunRequest, changed: Set<string>, paths: WatchPaths): void
/// ```
fn cycle(shared: &Shared, sink: &mut RpcSink, base: &RunRequest, changed: &BTreeSet<String>, paths: &mut WatchPaths) {
    let mut collected = CollectingSink::default();
    let check = RunRequest { mode: RunMode::Check { expected_dir: None }, ..base.clone() };
    let mut seen = WatchPaths::default();
    let code = run_locked(shared, &check, &mut collected, &mut seen);
    if !seen.files.is_empty() || !seen.directories.is_empty() {
        *paths = seen;
    }
    let drifts: Vec<&Event> = collected.events.iter().filter(|event| return event.event_type == "drift").collect();
    if code == EXIT_CLEAN && drifts.is_empty() {
        return;
    }
    if code != EXIT_CLEAN && drifts.is_empty() {
        for event in &collected.events {
            sink.emit(event.clone());
        }
        return;
    }
    for drift in drifts {
        let path = drift.fields.get("path").and_then(Value::as_str).unwrap_or_default();
        if changed.contains(path) {
            let message = format!("{path} is generated and was edited; it is being regenerated, edit its source instead");
            sink.emit(Event::new("warning", "generated-file-edited", &message));
        }
    }
    let write = RunRequest { mode: RunMode::Write, ..base.clone() };
    let mut after = WatchPaths::default();
    run_locked(shared, &write, sink, &mut after);
    if !after.files.is_empty() || !after.directories.is_empty() {
        *paths = after;
    }
}

/// What:
///  Serve one `watch` subscription until the client leaves or the server shuts down.
/// Why:
///  The first cycle brings every file current;
/// later cycles run when watched paths change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function watchConnection(shared: Shared, sink: RpcSink, configPath: string): Promise<void>
/// ```
pub fn watch_connection(shared: &Shared, mut sink: RpcSink, config_path: &Path) {
    let root = shared.options.root.clone();
    let base = RunRequest {
        root: root.clone(),
        config_path: config_path.to_path_buf(),
        scope: shared.options.scope,
        mode: RunMode::Write,
    };
    let (sender, receiver) = channel();
    let Ok(mut watcher) = notify::recommended_watcher(sender) else {
        sink.emit(Event::new("error", "watch-unavailable", "the operating system's file watcher could not start"));
        return;
    };
    let mut current: BTreeSet<PathBuf> = BTreeSet::new();
    let mut paths = WatchPaths::default();
    sink.emit(Event::new("watching", "watch-started", "watching; the first run brings every generated file current"));
    let mut changed: BTreeSet<String> = BTreeSet::new();
    cycle(shared, &mut sink, &base, &changed, &mut paths);
    let mut last_ping = Instant::now();
    loop {
        let wanted = watch_targets(&root, config_path, &paths);
        sync_watches(&mut watcher, &mut current, &wanted);
        if shared.shutdown.load(Ordering::SeqCst) {
            return;
        }
        // A closed client is noticed by a failing write, so ping on a clock, not only when the file system is quiet.
        if last_ping.elapsed() >= TICK {
            if !sink.ping() {
                return;
            }
            last_ping = Instant::now();
        }
        changed.clear();
        match receiver.recv_timeout(TICK) {
            Ok(first) => {
                collect_changed(&root, first, &mut changed);
                let debounce_started = Instant::now();
                while debounce_started.elapsed() < MAX_DEBOUNCE {
                    let Ok(next) = receiver.recv_timeout(QUIET_PERIOD) else { break };
                    collect_changed(&root, next, &mut changed);
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        }
        if changed.is_empty() {
            continue;
        }
        cycle(shared, &mut sink, &base, &changed, &mut paths);
    }
}

/// What:
///  Add the repo-relative paths of one notification to the changed set,
/// skipping staging files.
/// Why:
///  The tool's own atomic writes create and rename hidden staging files that must not trigger a cycle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectChanged(root: string, event: Event, changed: Set<string>): void
/// ```
fn collect_changed(root: &Path, received: Result<notify::Event, notify::Error>, changed: &mut BTreeSet<String>) {
    let Ok(event) = received else { return };
    // Reading a file is an access event; the run itself reads files, so counting accesses would loop forever.
    if matches!(event.kind, notify::EventKind::Access(_)) {
        return;
    }
    for path in event.paths {
        let name = path.file_name().map(|name| return name.to_string_lossy().to_string()).unwrap_or_default();
        if name.contains(STAGING_MARKER) {
            continue;
        }
        if let Ok(relative) = path.strip_prefix(root) {
            changed.insert(relative.display().to_string());
        }
    }
}
