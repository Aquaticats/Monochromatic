//! External-reload synchronization:
//!  the server's copy of the text must equal the document text
//! for every negotiated synchronization kind and column unit.

use crate::support::{self, PRODUCT_TIMEOUT, Probe, SERVER};
use ide_app::language::{
    reply::{RequestKind, RequestOutcome},
    status::ServerState,
};
use serde_json::Value;
use std::path::Path;

/// The first required correspondence example of the accepted scope,
///  before and after.
const CAT_BEFORE: &str = "I am a big cat.";
const CAT_AFTER: &str = "I was a big cat.";

/// The second required correspondence example,
///  before and after.
const HUMAN_BEFORE: &str = "I am a big cat";
const HUMAN_AFTER: &str = "I was a big cat, but now I am a human!";

/// Texts that insert a line and rewrite text after an accented letter and an astral character.
const ACCENT_BEFORE: &str = "caf\u{e9} \u{1F600} am here\nlast line\r\nend";
const ACCENT_AFTER: &str = "caf\u{e9} \u{1F600} was here\nnew line\nlast line\r\nthe end";

/// Messages of one method the server received.
fn of_method<'a>(lines: &'a [Value], method: &str) -> Vec<&'a Value> {
    return support::all_received(lines, method);
}

/// Open the first example,
///  apply every reload,
///  and require the server's copy to follow each one.
fn follow_reloads(root: &Path, probe: &mut Probe, name: &str) -> Vec<Value> {
    let file = root.join(name);
    probe.open(&file, CAT_BEFORE);
    probe.until_ready();
    support::server_text_until(root, CAT_BEFORE);
    let mut lines = Vec::new();
    // Each reload raises the protocol version by one, starting from the `didOpen` version zero.
    for (version, text) in (1_i64..).zip([
        CAT_AFTER,
        HUMAN_BEFORE,
        HUMAN_AFTER,
        ACCENT_BEFORE,
        ACCENT_AFTER,
    ]) {
        probe.reload(text);
        lines = {
            let mut wait = support::ReportWait::new(root, "the reloaded text");
            while !(support::server_text(wait.lines()) == Some((version, text.to_string()))) {
                wait.again();
            }
            wait.into_lines()
        };
    }
    return lines;
}

/// Incremental synchronization in the three column units a server can negotiate.
#[test]
fn incremental_reload_reaches_the_server_in_each_column_unit() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "sync::incremental_reload_reaches_the_server_in_each_column_unit",
            support::standard,
        );
        return;
    };
    for encoding in ["utf-8", "utf-16", "utf-32", ""] {
        std::fs::remove_file(support::report_path(&root)).unwrap_or_default();
        let definitions = support::scripted(
            &root,
            &[("SYNC", "incremental"), ("ENCODING", encoding)],
            PRODUCT_TIMEOUT,
        );
        let mut probe = Probe::new(&root, definitions);
        let lines = follow_reloads(&root, &mut probe, "incremental.scripted");
        let changes = of_method(&lines, "textDocument/didChange");
        assert_eq!(changes.len(), 5, "one change notification per reload");
        for change in &changes {
            let events = change["params"]["contentChanges"]
                .as_array()
                .expect("content changes");
            let mut all_ranged = true;
            for event in events.iter() {
                if event["range"].is_null() {
                    all_ranged = false;
                }
            }
            assert!(
                all_ranged,
                "an incremental server was sent a whole-text change in {encoding:?}: {change}"
            );
        }
        assert_eq!(changes[0]["params"]["textDocument"]["version"], 1);
        assert_eq!(of_method(&lines, "textDocument/didOpen").len(), 1);
        assert!(
            of_method(&lines, "textDocument/didSave").is_empty(),
            "didSave was sent to a server that did not ask for it"
        );
        assert_eq!(probe.state(SERVER), Some(&ServerState::Ready));
    }
}

/// Full synchronization sends the whole new text.
#[test]
fn full_reload_sends_the_whole_text() {
    let Some(root) = support::child_root() else {
        support::run_child("sync::full_reload_sends_the_whole_text", support::standard);
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("SYNC", "full")], PRODUCT_TIMEOUT),
    );
    let lines = follow_reloads(&root, &mut probe, "full.scripted");
    let changes = of_method(&lines, "textDocument/didChange");
    assert_eq!(changes.len(), 5);
    let last = &changes[4]["params"]["contentChanges"];
    assert_eq!(last.as_array().map(Vec::len), Some(1));
    assert!(last[0]["range"].is_null());
    assert_eq!(last[0]["text"], ACCENT_AFTER);
}

/// A server that takes no change notifications is closed and reopened with the new text.
#[test]
fn server_without_change_notifications_is_reopened() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "sync::server_without_change_notifications_is_reopened",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("SYNC", "none")], PRODUCT_TIMEOUT),
    );
    let lines = follow_reloads(&root, &mut probe, "none.scripted");
    assert!(
        of_method(&lines, "textDocument/didChange").is_empty(),
        "a change notification was sent to a server that takes none"
    );
    assert_eq!(of_method(&lines, "textDocument/didClose").len(), 5);
    let opens = of_method(&lines, "textDocument/didOpen");
    assert_eq!(opens.len(), 6);
    assert_eq!(opens[5]["params"]["textDocument"]["version"], 5);
    assert_eq!(opens[5]["params"]["textDocument"]["text"], ACCENT_AFTER);
    assert_eq!(probe.state(SERVER), Some(&ServerState::Ready));
    let number = probe.request(RequestKind::Hover, 0);
    assert!(matches!(
        probe.answers(number)[0].outcome,
        RequestOutcome::Hover(_)
    ));
}

/// A server without any synchronization capability cannot follow a reload and is no longer asked.
#[test]
fn server_without_synchronization_becomes_unsynchronized() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "sync::server_without_synchronization_becomes_unsynchronized",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("SYNC", "absent")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("absent.scripted"), CAT_BEFORE);
    probe.until_ready();
    support::server_text_until(&root, CAT_BEFORE);
    let number = probe.request(RequestKind::Hover, 0);
    assert!(matches!(
        probe.answers(number)[0].outcome,
        RequestOutcome::Hover(_)
    ));
    probe.reload(CAT_AFTER);
    probe.wait_for("the unsynchronized state");
    while !(probe.state(SERVER) == Some(&ServerState::Unsynchronized)) {
        probe.step();
    }
    let stale = probe.request(RequestKind::Hover, 0);
    assert_eq!(
        probe.answers(stale)[0].outcome,
        RequestOutcome::Unsynchronized,
        "a position request was routed to a server holding the old text"
    );
    let lines = support::report(&root);
    assert!(of_method(&lines, "textDocument/didChange").is_empty());
    assert!(of_method(&lines, "textDocument/didClose").is_empty());
    assert_eq!(of_method(&lines, "textDocument/didOpen").len(), 1);
    assert_eq!(of_method(&lines, "textDocument/hover").len(), 1);
    assert_eq!(
        support::server_text(&lines),
        Some((0, CAT_BEFORE.to_string())),
        "the test must show the server still holds the old text"
    );
    // Another file starts synchronized again.
    probe.open(&root.join("second.scripted"), HUMAN_BEFORE);
    probe.until_ready();
    support::server_text_until(&root, HUMAN_BEFORE);
}

/// After an external reload the text on disk equals the new text,
///  so `didSave` is sent when asked for.
#[test]
fn did_save_follows_a_reload_when_the_server_asks_for_it() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "sync::did_save_follows_a_reload_when_the_server_asks_for_it",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("SAVE", "1")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("saved.scripted"), CAT_BEFORE);
    probe.until_ready();
    probe.reload(CAT_AFTER);
    let lines = {
        let mut wait = support::ReportWait::new(&root, "didSave");
        while !(!of_method(wait.lines(), "textDocument/didSave").is_empty()) {
            wait.again();
        }
        wait.into_lines()
    };
    let received = support::received(&lines);
    let change = support::index_of(&received, "textDocument/didChange").expect("didChange");
    let save = support::index_of(&received, "textDocument/didSave").expect("didSave");
    assert!(change < save, "didSave must follow didChange: {received:?}");
    assert_eq!(of_method(&lines, "textDocument/didSave").len(), 1);
}

/// A reload that arrives while the server is still starting reaches it through its one `didOpen`.
#[test]
fn reload_during_start_is_delivered_by_the_eventual_did_open() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "sync::reload_during_start_is_delivered_by_the_eventual_did_open",
            support::standard,
        );
        return;
    };
    // The server answers `initialize` only once the gate file exists, which the test creates
    // after the reload was sent; the worker takes the reload before the server can be ready.
    let gate = support::scratch(&root).join("initialize-gate");
    let gate_text = gate.display().to_string();
    let definitions =
        support::scripted(&root, &[("INIT_GATE", gate_text.as_str())], PRODUCT_TIMEOUT);
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("slow.scripted"), CAT_BEFORE);
    probe.wait_for("the starting state");
    while !(probe.state(SERVER) == Some(&ServerState::Starting)) {
        probe.step();
    }
    probe.reload(CAT_AFTER);
    std::fs::write(&gate, "").expect("initialize gate");
    probe.until_ready();
    let lines = support::server_text_until(&root, CAT_AFTER);
    let opens = of_method(&lines, "textDocument/didOpen");
    assert_eq!(opens.len(), 1);
    assert_eq!(opens[0]["params"]["textDocument"]["version"], 1);
    assert_eq!(opens[0]["params"]["textDocument"]["text"], CAT_AFTER);
    assert!(of_method(&lines, "textDocument/didChange").is_empty());
}

/// A reload that does not continue from the text servers hold is diffed against that text.
#[test]
fn missed_reload_is_recomputed_from_the_synchronized_text() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "sync::missed_reload_is_recomputed_from_the_synchronized_text",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("missed.scripted"), CAT_BEFORE);
    probe.until_ready();
    support::server_text_until(&root, CAT_BEFORE);
    // The application accepts a reload that never reaches the worker.
    let missed = probe.document.prepare_reload(HUMAN_BEFORE);
    assert!(probe.document.apply_reload(missed));
    probe.reload(HUMAN_AFTER);
    let lines = support::server_text_until(&root, HUMAN_AFTER);
    assert_eq!(of_method(&lines, "textDocument/didChange").len(), 1);
    assert_eq!(
        support::server_text(&lines),
        Some((1, HUMAN_AFTER.to_string()))
    );
}
