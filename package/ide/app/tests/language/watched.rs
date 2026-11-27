//! File changes made outside the IDE reach the servers that registered file watchers,
//!  through the IDE's
//! own folder watching:
//!  glob patterns and kinds decide,
//!  bursts arrive gathered,
//!  ignored and pruned folders
//! are skipped,
//!  new folders are followed,
//!  and a server that registered nothing hears nothing.

/// The worker,
/// the change watcher and the waits,
///  the fixture project,
///  and the questions asked of a report.
#[path = "watched/watcher_session.rs"]
mod watcher_session;

use crate::support;
use serde_json::Value;
use std::fs;
use watcher_session::{
    Changes,
    SETTLE,
    Watching,
    burst_complete,
    project,
    pulled_after_notification,
};

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
