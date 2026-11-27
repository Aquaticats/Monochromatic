//! Displaying another file:
//!  the first file is closed on the server and the same process serves the second,
//!  and a file outside the project starts nothing and is served only by a server that already runs.

/// The process identifiers the report names.
use super::started;
/// The scripted-server reader and the reports it reads.
use crate::support::{
    self,
    PRODUCT_TIMEOUT,
    Probe,
    SERVER,
};
/// What each request asked for and what became of it.
use ide_app::language::reply::{
    RequestKind,
    RequestOutcome,
};
/// What the displayed document is to the project.
use ide_app::language::status::DocumentState;
/// The state a displayed server is in.
use ide_app::language::status::ServerState;
/// The report lines the notifications are read from.
use serde_json::Value;

/// Switching files closes the first on the server and reuses the same process for the second.
#[test]
fn file_switch_closes_the_first_file_and_reuses_the_server() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::file_switching::file_switch_closes_the_first_file_and_reuses_the_server",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    let first = root.join("first.scripted");
    let second = root.join("second.scripted");
    probe.open(&first, "alpha\n");
    probe.until_ready();
    probe.wait_for("diagnostics for the first file");
    while !(probe.messages() == ["TEXT:alpha\n"]) {
        probe.step();
    }
    probe.open(&second, "beta\n");
    let stamp = probe.stamp();
    probe.wait_for("diagnostics for the second file");
    while !(probe.messages() == ["TEXT:beta\n"] && probe.diagnostics_stamped(stamp)) {
        probe.step();
    }
    assert_eq!(probe.status.servers[0].server.instance, 1);
    let lines = support::report(&root);
    assert_eq!(started(&lines).len(), 1);
    let closed = support::first_received(&lines, "textDocument/didClose").expect("didClose");
    assert_eq!(
        closed["params"]["textDocument"]["uri"],
        format!("file://{}", first.display()).as_str()
    );
    assert_eq!(support::count_received(&lines, "textDocument/didOpen"), 2);
    probe.worker.close().expect("close");
    probe.wait_for("the closed state");
    while !(probe.status.document == DocumentState::Closed) {
        probe.step();
    }
    let mut close_wait = support::ReportWait::new(&root, "the second didClose");
    while support::count_received(close_wait.lines(), "textDocument/didClose") != 2 {
        close_wait.again();
    }
    let after_close = close_wait.into_lines();
    assert_eq!(started(&after_close).len(), 1);
    assert_eq!(
        support::children().len(),
        1,
        "the server stays for the next file"
    );
}

/// A file outside the project starts nothing and is served only by servers that already run.
#[test]
fn file_outside_the_project_attaches_only_running_servers() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::file_switching::file_outside_the_project_attaches_only_running_servers",
            support::standard,
        );
        return;
    };
    let outside = support::scratch(&root).join("dependency.scripted");
    std::fs::write(&outside, "dependency text\n").expect("outside file");
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.display(&outside, "dependency text\n");
    probe.wait_for("the outside-project state");
    while !(probe.status.document == DocumentState::OutsideProject) {
        probe.step();
    }
    assert!(probe.status.servers.is_empty());
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(
        support::children().is_empty(),
        "a server was started for a file outside the project"
    );
    let unanswerable = probe.request(RequestKind::Hover, 1);
    assert_eq!(
        probe.answers(unanswerable)[0].outcome,
        RequestOutcome::NoServer
    );
    probe.open(&root.join("inside.scripted"), "inside\n");
    probe.until_ready();
    probe.display(&outside, "dependency text\n");
    probe.wait_for("the running server to serve the outside file");
    while !(probe.status.document == DocumentState::OutsideProject
        && probe.state(SERVER) == Some(&ServerState::Ready))
    {
        probe.step();
    }
    let number = probe.request(RequestKind::Hover, 1);
    let answers = probe.answers(number);
    let RequestOutcome::Hover(hover) = &answers[0].outcome else {
        panic!("hover produced {:?}", answers[0].outcome);
    };
    assert_eq!(hover.text, "line=dependency text char=e");
    let lines = support::report(&root);
    assert_eq!(started(&lines).len(), 1);
    let opened: Vec<&Value> = support::all_received(&lines, "textDocument/didOpen");
    assert_eq!(
        opened[1]["params"]["textDocument"]["uri"],
        format!("file://{}", outside.display()).as_str()
    );
}
