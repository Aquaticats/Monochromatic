//! Registered file watchers without a server:
//!  the merge rule,
//!  glob matching per the protocol,
//!  kinds,
//! bursts,
//!  and servers that stopped.

/// The registry and the merge rule under test.
use super::{FORWARD_LIMIT, FORWARD_QUIET, WatchHost, WatchedFiles, merge};
/// The changes the change watcher forwards.
use crate::change_watch::{ServerChange, ServerChangeKind};
/// Registrations are written as JSON;
///  the macro is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use crate::json_value;
/// The protocol's data types.
use helix_lsp::lsp;
/// What:
///  `from_value` decodes a JSON value into a typed record.
/// Why:
///  Registrations are written exactly as a server sends them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const options = JSON.parse(text) as RegistrationOptions;
/// ```
use serde_json::from_value;
/// Paths and the chosen times of each step.
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

/// Decode registration options written as JSON.
fn options(watchers: serde_json::Value) -> lsp::DidChangeWatchedFilesRegistrationOptions {
    return from_value(json_value!({ "watchers": watchers })).expect("registration options");
}

/// One change at `path`.
fn change(path: &str, kind: ServerChangeKind) -> ServerChange {
    return ServerChange {
        path: PathBuf::from(path),
        kind,
    };
}

/// What:
///  The answers `take` asks for in these tests:
///  paths are already in the servers' spelling,
///  and either every server runs (`only` is `None`) or just the one named.
struct Servers {
    /// The one server that runs,
    ///  or `None` when all of them do.
    only: Option<u32>,
}

impl WatchHost<u32> for Servers {
    fn respell(&self, path: &Path) -> PathBuf {
        return path.to_path_buf();
    }

    fn live(&self, server: u32) -> bool {
        return match self.only {
            Some(only) => server == only,
            None => true,
        };
    }
}

/// Orders batches by the number of their server.
fn by_server(
    left: &(u32, Vec<lsp::FileEvent>),
    right: &(u32, Vec<lsp::FileEvent>),
) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// The (address,
///  kind number) pairs of one batch,
///  for comparison.
fn pairs(events: &[lsp::FileEvent]) -> Vec<(String, i32)> {
    let mut found: Vec<(String, i32)> = Vec::new();
    for event in events {
        let number = if event.typ == lsp::FileChangeType::CREATED {
            1
        } else if event.typ == lsp::FileChangeType::CHANGED {
            2
        } else {
            3
        };
        found.push((event.uri.as_str().to_string(), number));
    }
    return found;
}

/// The final state on disk decides:
///  a deletion after a creation stays a deletion,
///  and a replaced file changed.
#[test]
fn the_merged_kind_matches_the_final_state() {
    use ServerChangeKind::{Changed, Created, Deleted};
    assert_eq!(merge(None, Created), Created);
    assert_eq!(merge(Some(Created), Changed), Created);
    assert_eq!(merge(Some(Created), Deleted), Deleted);
    assert_eq!(merge(Some(Deleted), Created), Changed);
    assert_eq!(merge(Some(Changed), Created), Changed);
    assert_eq!(merge(Some(Changed), Deleted), Deleted);
    assert_eq!(merge(Some(Deleted), Changed), Changed);
    assert_eq!(merge(None, Changed), Changed);
}

/// Relative and string patterns follow the protocol:
///  `*` stays in one segment,
///  `**` spans segments,
/// `{}` groups,
///  and a relative pattern only matches below its base.
#[test]
fn glob_patterns_follow_the_protocol() {
    let mut watched: WatchedFiles<u32> = WatchedFiles::default();
    watched.register(
        1,
        "relative".to_string(),
        options(json_value!([
            { "globPattern": { "baseUri": "file:///p/src", "pattern": "**/*.{ts,tsx}" } },
            { "globPattern": { "baseUri": { "uri": "file:///p", "name": "p" }, "pattern": "*.json" } },
        ])),
    );
    watched.register(
        2,
        "string".to_string(),
        options(json_value!([{ "globPattern": "**/*.rs" }])),
    );
    let now = Instant::now();
    for path in [
        "/p/src/a.ts",
        "/p/src/deep/b.tsx",
        "/p/other/c.ts",
        "/p/tsconfig.json",
        "/p/src/nested.json",
        "/p/lib.rs",
        "/p/src/x.rs",
    ] {
        watched.record(change(path, ServerChangeKind::Changed), now);
    }
    let mut batches = watched.take(&Servers { only: None }, Path::new("/p"));
    batches.sort_by(by_server);
    assert_eq!(batches.len(), 2);
    assert_eq!(
        pairs(&batches[0].1),
        vec![
            ("file:///p/src/a.ts".to_string(), 2),
            ("file:///p/src/deep/b.tsx".to_string(), 2),
            ("file:///p/tsconfig.json".to_string(), 2),
        ],
        "relative patterns matched the wrong files"
    );
    assert_eq!(
        pairs(&batches[1].1),
        vec![
            ("file:///p/lib.rs".to_string(), 2),
            ("file:///p/src/x.rs".to_string(), 2),
        ]
    );
}

/// A watcher's kind limits what it hears;
///  with no kind it hears all three.
#[test]
fn kinds_limit_what_a_watcher_hears() {
    let mut watched: WatchedFiles<u32> = WatchedFiles::default();
    watched.register(
        1,
        "creations".to_string(),
        options(json_value!([{ "globPattern": "**/*.txt", "kind": 1 }])),
    );
    let now = Instant::now();
    watched.record(change("/p/new.txt", ServerChangeKind::Created), now);
    watched.record(change("/p/old.txt", ServerChangeKind::Changed), now);
    watched.record(change("/p/gone.txt", ServerChangeKind::Deleted), now);
    let batches = watched.take(&Servers { only: None }, Path::new("/p"));
    assert_eq!(batches.len(), 1);
    assert_eq!(
        pairs(&batches[0].1),
        vec![("file:///p/new.txt".to_string(), 1)]
    );
}

/// A burst is sent once quiet,
///  or at the latest after the limit,
///  with each path once.
#[test]
fn a_burst_is_sent_once_with_each_path_once() {
    let mut watched: WatchedFiles<u32> = WatchedFiles::default();
    watched.register(
        1,
        "all".to_string(),
        options(json_value!([{ "globPattern": "**/*" }])),
    );
    let start = Instant::now();
    assert!(watched.record(change("/p/a", ServerChangeKind::Created), start));
    let mut now = start;
    for _ in 0..100 {
        now += Duration::from_millis(4);
        assert!(!watched.record(change("/p/a", ServerChangeKind::Changed), now));
        watched.record(change("/p/b", ServerChangeKind::Changed), now);
    }
    assert!(
        watched.wait(now).expect("pending changes") > Duration::ZERO,
        "a burst still writing was due"
    );
    assert_eq!(
        watched.wait(start + FORWARD_LIMIT),
        Some(Duration::ZERO),
        "a burst that never pauses was not due at the limit"
    );
    assert_eq!(watched.wait(now + FORWARD_QUIET), Some(Duration::ZERO));
    let batches = watched.take(&Servers { only: None }, Path::new("/p"));
    assert_eq!(
        pairs(&batches[0].1),
        vec![
            ("file:///p/a".to_string(), 1),
            ("file:///p/b".to_string(), 2)
        ]
    );
    assert_eq!(
        watched.wait(now),
        None,
        "the burst stayed pending after it was taken"
    );
}

/// Servers that registered nothing get nothing,
///  and a stopped server's watchers are forgotten.
#[test]
fn only_running_servers_with_watchers_hear_anything() {
    let mut watched: WatchedFiles<u32> = WatchedFiles::default();
    assert!(!watched.wanted());
    let now = Instant::now();
    assert!(
        !watched.record(change("/p/a.rs", ServerChangeKind::Changed), now),
        "a change was gathered with no watcher registered"
    );
    watched.register(
        1,
        "rust".to_string(),
        options(json_value!([{ "globPattern": "**/*.rs" }])),
    );
    watched.register(
        2,
        "rust".to_string(),
        options(json_value!([{ "globPattern": "**/*.rs" }])),
    );
    assert!(watched.wanted());
    watched.record(change("/p/a.rs", ServerChangeKind::Changed), now);
    let batches = watched.take(&Servers { only: Some(1) }, Path::new("/p"));
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].0, 1);
    watched.unregister(1, "rust");
    assert!(
        !watched.wanted(),
        "watchers outlived their server or their registration"
    );
}
