//! The harness the four tests of this file share:
//!  the worker and the project's change watcher relayed by one application tick,
//!  the waits that read what the server received,
//!  the fixture project the writes land in,
//!  and the two questions asked of a report.

use crate::support::{
    self,
    Probe,
};
use ide_app::{
    change_watch::ChangeWatcher,
    workspace::Workspace,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{
        Duration,
        Instant,
    },
};

/// Longest wait for an expected change.
const PATIENCE: Duration = Duration::from_secs(20);

/// How long a change that must not arrive is waited for,
///  after a later change already arrived.
pub const SETTLE: Duration = Duration::from_millis(800);

/// The worker,
///  the change watcher,
///  and the relay between them that the application's tick performs.
pub struct Watching {
    /// The language worker and what it published.
    pub probe: Probe,
    /// The project's change watcher.
    pub watcher: ChangeWatcher,
    /// What the current wait is for,
    ///  named in the failure when the wait times out.
    waiting_for: String,
    /// When the current wait began.
    wait_started: Instant,
    /// The changes the server had received when the wait last looked.
    pub changes: Changes,
}

impl Watching {
    /// Start both for `root`,
    ///  with the scripted server configured by `variables` (`WATCHERS` is the JSON it
    /// registers after `initialized`).
    pub fn new(
        root: &Path,
        variables: &[(&str, &str)],
    ) -> Self {
        let probe = Probe::new(root, support::scripted(root, variables, 20));
        let watcher =
            ChangeWatcher::new(Workspace::new(root).expect("workspace")).expect("change watcher");
        return Self {
            probe,
            watcher,
            waiting_for: String::new(),
            wait_started: Instant::now(),
            changes: Changes {
                notifications: Vec::new(),
            },
        };
    }

    /// One application tick:
    ///  poll the worker and hand its feed to the watcher while it wants changes.
    fn relay(&mut self) {
        self.probe.poll();
        let mut feed = None;
        if self.probe.worker.wants_file_changes() {
            feed = Some(self.probe.worker.file_change_sender());
        }
        self.watcher.feed_servers(feed);
        // The tree's invalidations are not under test; taking them keeps them from piling up.
        let _tree = self.watcher.take();
    }

    /// Relay until at least `folders` folders are watched for the servers.
    pub fn until_watching(
        &mut self,
        folders: usize,
    ) {
        let start = Instant::now();
        while self.watcher.server_folders() < folders {
            self.relay();
            assert!(
                start.elapsed() < PATIENCE,
                "only {} folders were watched for the servers",
                self.watcher.server_folders()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Begin a wait for `what`,
    ///  relay once,
    ///  and read the changes the server received.
    /// The caller loops `while !condition { session.step_changes(root); }`.
    pub fn wait_for_changes(
        &mut self,
        root: &Path,
        what: &str,
    ) {
        self.waiting_for = what.to_string();
        self.wait_started = Instant::now();
        self.relay();
        self.changes = Changes::read(root);
    }

    /// One more round of the wait begun by `wait_for_changes`:
    ///  fail with the notifications seen when the patience ran out,
    ///  else pause,
    ///  relay,
    ///  and read again.
    pub fn step_changes(
        &mut self,
        root: &Path,
    ) {
        assert!(
            self.wait_started.elapsed() < PATIENCE,
            "timed out waiting for {}; notifications: {:?}",
            self.waiting_for,
            self.changes.notifications
        );
        std::thread::sleep(Duration::from_millis(10));
        self.relay();
        self.changes = Changes::read(root);
    }

    /// Take the changes the last wait saw.
    pub fn take_changes(&mut self) -> Changes {
        return std::mem::replace(
            &mut self.changes,
            Changes {
                notifications: Vec::new(),
            },
        );
    }

    /// Relay for `span`,
    ///  so changes that must not arrive have the chance to.
    pub fn settle(
        &mut self,
        span: Duration,
    ) {
        let start = Instant::now();
        while start.elapsed() < span {
            self.relay();
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// True when a diagnostics request follows the first watched-file notification in the report.
pub fn pulled_after_notification(lines: &[Value]) -> bool {
    let methods = support::received(lines);
    let Some(first) = support::index_of(&methods, "workspace/didChangeWatchedFiles") else {
        return false;
    };
    for method in &methods[first..] {
        if method == "textDocument/diagnostic" {
            return true;
        }
    }
    return false;
}

/// True when every path of the burst of `count` files was reported with its final kind.
pub fn burst_complete(
    changes: &Changes,
    count: usize,
) -> bool {
    let last = changes.last();
    for index in 0..count {
        let kind = last.get(&format!("src/f{index:03}.scripted")).copied();
        // A burst split by the sending limit may report a later rewrite as a change.
        let settled = if index % 2 == 0 {
            kind == Some(3)
        } else {
            matches!(kind, Some(1 | 2))
        };
        if !settled {
            return false;
        }
    }
    return true;
}

/// Every `workspace/didChangeWatchedFiles` the server received,
///  as (path below the root,
///  kind number) lists.
pub struct Changes {
    /// One list per notification,
    ///  in arrival order.
    pub notifications: Vec<Vec<(String, i64)>>,
}

impl Changes {
    /// Read the scripted server's report.
    pub fn read(root: &Path) -> Self {
        let prefix = format!("file://{}/", root.display());
        let mut notifications = Vec::new();
        for line in support::report(root) {
            if line["received"] != "workspace/didChangeWatchedFiles" {
                continue;
            }
            let mut events = Vec::new();
            for change in line["params"]["changes"].as_array().into_iter().flatten() {
                let uri = change["uri"].as_str().unwrap_or("");
                let path = uri.strip_prefix(&prefix).unwrap_or(uri).to_string();
                events.push((path, change["type"].as_i64().unwrap_or(0)));
            }
            notifications.push(events);
        }
        return Self {
            notifications,
        };
    }

    /// The latest kind each path was reported with.
    fn last(&self) -> BTreeMap<String, i64> {
        let mut latest = BTreeMap::new();
        for events in &self.notifications {
            for (path, kind) in events {
                latest.insert(path.clone(), *kind);
            }
        }
        return latest;
    }

    /// True when some notification reported `path` with `kind`.
    pub fn has(
        &self,
        path: &str,
        kind: i64,
    ) -> bool {
        for events in &self.notifications {
            for (found, found_kind) in events {
                if found == path && *found_kind == kind {
                    return true;
                }
            }
        }
        return false;
    }

    /// True when some notification mentions a path starting with `prefix`.
    pub fn mentions(
        &self,
        prefix: &str,
    ) -> bool {
        for events in &self.notifications {
            for (found, _) in events {
                if found.starts_with(prefix) {
                    return true;
                }
            }
        }
        return false;
    }
}

/// A git work tree with an ignored `dist`,
///  a `node_modules`,
///  and a source folder;
///  ripgrep honours
/// `.gitignore` once a `.git` folder exists.
pub fn project(root: &Path) {
    for folder in [".git", "src", "dist", "node_modules/pkg"] {
        fs::create_dir_all(root.join(folder)).expect("fixture folder");
    }
    fs::write(root.join(".gitignore"), "dist/\nbuild/\n").expect("ignore file");
    fs::write(root.join("dist/old.scripted"), "x").expect("ignored file");
    fs::write(root.join("node_modules/pkg/old.scripted"), "x").expect("dependency file");
}
