//! File changes made outside the IDE reach the servers that registered file watchers,
//!  through the IDE's
//! own folder watching:
//!  glob patterns and kinds decide,
//!  bursts arrive gathered,
//!  ignored and pruned folders
//! are skipped,
//!  new folders are followed,
//!  and a server that registered nothing hears nothing.

use crate::support::{self, Probe};
use ide_app::{change_watch::ChangeWatcher, workspace::Workspace};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, Instant},
};

/// Longest wait for an expected change.
const PATIENCE: Duration = Duration::from_secs(20);

/// How long a change that must not arrive is waited for,
///  after a later change already arrived.
const SETTLE: Duration = Duration::from_millis(800);

/// The worker,
///  the change watcher,
///  and the relay between them that the application's tick performs.
struct Watching {
    /// The language worker and what it published.
    probe: Probe,
    /// The project's change watcher.
    watcher: ChangeWatcher,
    /// What the current wait is for,
    ///  named in the failure when the wait times out.
    waiting_for: String,
    /// When the current wait began.
    wait_started: Instant,
    /// The changes the server had received when the wait last looked.
    changes: Changes,
}

impl Watching {
    /// Start both for `root`,
    ///  with the scripted server configured by `variables` (`WATCHERS` is the JSON it
    /// registers after `initialized`).
    fn new(root: &Path, variables: &[(&str, &str)]) -> Self {
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
    fn until_watching(&mut self, folders: usize) {
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
    fn wait_for_changes(&mut self, root: &Path, what: &str) {
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
    fn step_changes(&mut self, root: &Path) {
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
    fn take_changes(&mut self) -> Changes {
        return std::mem::replace(
            &mut self.changes,
            Changes {
                notifications: Vec::new(),
            },
        );
    }

    /// Relay for `span`,
    ///  so changes that must not arrive have the chance to.
    fn settle(&mut self, span: Duration) {
        let start = Instant::now();
        while start.elapsed() < span {
            self.relay();
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// True when a diagnostics request follows the first watched-file notification in the report.
fn pulled_after_notification(lines: &[Value]) -> bool {
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
fn burst_complete(changes: &Changes, count: usize) -> bool {
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
struct Changes {
    /// One list per notification,
    ///  in arrival order.
    notifications: Vec<Vec<(String, i64)>>,
}

impl Changes {
    /// Read the scripted server's report.
    fn read(root: &Path) -> Self {
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
        return Self { notifications };
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
    fn has(&self, path: &str, kind: i64) -> bool {
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
    fn mentions(&self, prefix: &str) -> bool {
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
fn project(root: &Path) {
    for folder in [".git", "src", "dist", "node_modules/pkg"] {
        fs::create_dir_all(root.join(folder)).expect("fixture folder");
    }
    fs::write(root.join(".gitignore"), "dist/\nbuild/\n").expect("ignore file");
    fs::write(root.join("dist/old.scripted"), "x").expect("ignored file");
    fs::write(root.join("node_modules/pkg/old.scripted"), "x").expect("dependency file");
}

/// Created,
///  changed,
///  and deleted files reach the server when a watcher's glob and kind match;
///  files in
/// ignored and dependency folders,
///  and files no glob matches,
///  do not.
#[test]
fn changes_reach_the_server_by_glob_and_kind() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "watched::changes_reach_the_server_by_glob_and_kind",
            support::standard,
        );
        return;
    };
    project(&root);
    let watchers = format!(
        r#"[{{"globPattern":"**/*.scripted"}},{{"globPattern":{{"baseUri":"file://{}/src","pattern":"**/*.txt"}},"kind":1}}]"#,
        root.display()
    );
    let mut session = Watching::new(&root, &[("WATCHERS", &watchers), ("PULL", "1")]);
    session
        .probe
        .open(&root.join("src/main.scripted"), "watched\n");
    session.probe.until_ready();
    // The root and `src`; `dist` and `node_modules` are not watched.
    session.until_watching(2);
    for (file, text) in [
        ("dist/new.scripted", "ignored"),
        ("node_modules/pkg/new.scripted", "dependency"),
        ("src/skip.rs", "no glob matches"),
        ("src/notes.txt", "created"),
        ("src/other.scripted", "created"),
    ] {
        fs::write(root.join(file), text).expect("external write");
    }
    session.wait_for_changes(&root, "the creations");
    while !(session.changes.has("src/other.scripted", 1) && session.changes.has("src/notes.txt", 1))
    {
        session.step_changes(&root);
    }
    // Another file may change the displayed file's diagnostics, so they are pulled again afterwards.
    let mut pull_wait =
        support::ReportWait::new(&root, "a diagnostics pull after the file changes");
    while !pulled_after_notification(pull_wait.lines()) {
        pull_wait.again();
    }
    fs::write(root.join("src/other.scripted"), "changed").expect("external change");
    fs::remove_file(root.join("src/notes.txt")).expect("external deletion");
    session.wait_for_changes(&root, "the change");
    while !(session.changes.has("src/other.scripted", 2)) {
        session.step_changes(&root);
    }
    fs::remove_file(root.join("src/other.scripted")).expect("external deletion");
    session.wait_for_changes(&root, "the deletion");
    while !(session.changes.has("src/other.scripted", 3)) {
        session.step_changes(&root);
    }
    session.settle(SETTLE);
    let all = Changes::read(&root);
    assert!(
        !all.mentions("dist/"),
        "an ignored folder's change was sent: {:?}",
        all.notifications
    );
    assert!(
        !all.mentions("node_modules/"),
        "a dependency folder's change was sent: {:?}",
        all.notifications
    );
    assert!(
        !all.mentions("src/skip.rs"),
        "a file no glob matches was sent: {:?}",
        all.notifications
    );
    assert!(
        !all.has("src/notes.txt", 3),
        "a deletion reached a watcher that asked only for creations: {:?}",
        all.notifications
    );
}

/// A burst arrives in a few notifications,
///  each path once in each,
///  with the kind of its final state.
#[test]
fn a_burst_arrives_gathered_with_the_final_kinds() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "watched::a_burst_arrives_gathered_with_the_final_kinds",
            support::standard,
        );
        return;
    };
    project(&root);
    let mut session = Watching::new(
        &root,
        &[("WATCHERS", r#"[{"globPattern":"**/*.scripted"}]"#)],
    );
    session
        .probe
        .open(&root.join("src/main.scripted"), "watched\n");
    session.probe.until_ready();
    session.until_watching(2);
    let count = 200;
    for index in 0..count {
        fs::write(root.join(format!("src/f{index:03}.scripted")), "one").expect("burst write");
        fs::write(root.join(format!("src/f{index:03}.scripted")), "two").expect("burst rewrite");
    }
    for index in (0..count).step_by(2) {
        fs::remove_file(root.join(format!("src/f{index:03}.scripted"))).expect("burst deletion");
    }
    session.wait_for_changes(&root, "every path of the burst");
    while !burst_complete(&session.changes, count) {
        session.step_changes(&root);
    }
    let changes = session.take_changes();
    assert!(
        changes.notifications.len() <= 10,
        "a burst of {} changes arrived in {} notifications",
        count * 3,
        changes.notifications.len()
    );
    for events in &changes.notifications {
        let mut seen = std::collections::BTreeSet::new();
        for (path, _) in events {
            assert!(
                seen.insert(path.clone()),
                "{path} appeared twice in one notification"
            );
        }
    }
}

/// A new folder is followed,
///  its files included,
///  while a new ignored folder's files are never sent.
#[test]
fn new_folders_are_followed_unless_ignored() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "watched::new_folders_are_followed_unless_ignored",
            support::standard,
        );
        return;
    };
    project(&root);
    let mut session = Watching::new(
        &root,
        &[("WATCHERS", r#"[{"globPattern":"**/*.scripted"}]"#)],
    );
    session
        .probe
        .open(&root.join("src/main.scripted"), "watched\n");
    session.probe.until_ready();
    session.until_watching(2);
    fs::create_dir(root.join("build")).expect("ignored folder");
    fs::create_dir(root.join("src/module")).expect("new folder");
    // Wait until the empty new folder is watched, so the next write lands in a watched folder.
    session.until_watching(3);
    fs::write(root.join("build/out.scripted"), "ignored").expect("ignored write");
    fs::write(root.join("src/module/a.scripted"), "new").expect("write in a new folder");
    fs::create_dir_all(root.join("src/module/deeper")).expect("nested new folder");
    fs::write(root.join("src/module/deeper/b.scripted"), "new")
        .expect("write in a nested new folder");
    session.wait_for_changes(&root, "the new folders' files");
    while !(session.changes.has("src/module/a.scripted", 1)
        && session.changes.has("src/module/deeper/b.scripted", 1))
    {
        session.step_changes(&root);
    }
    session.settle(SETTLE);
    let all = Changes::read(&root);
    assert!(
        !all.mentions("build/out"),
        "a new ignored folder's file was sent: {:?}",
        all.notifications
    );
}

/// A server that registered no watcher hears nothing,
///  and no folder is watched for it.
#[test]
fn a_server_without_watchers_hears_nothing() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "watched::a_server_without_watchers_hears_nothing",
            support::standard,
        );
        return;
    };
    project(&root);
    let mut session = Watching::new(&root, &[]);
    session
        .probe
        .open(&root.join("src/main.scripted"), "watched\n");
    session.probe.until_ready();
    session.settle(SETTLE);
    fs::write(root.join("src/other.scripted"), "created").expect("external write");
    session.settle(SETTLE);
    assert_eq!(
        session.watcher.server_folders(),
        0,
        "folders were watched for a server that registered nothing"
    );
    let lines: Vec<Value> = support::report(&root);
    assert!(
        !support::received(&lines).contains(&"workspace/didChangeWatchedFiles".to_string()),
        "a server that registered nothing was sent file changes"
    );
}
