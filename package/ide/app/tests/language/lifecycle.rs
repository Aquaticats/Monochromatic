//! Server lifetime:
//!  crash and restart,
//!  file switches,
//!  files outside the project,
//!  launch refusal,
//! and shutdown without leftover processes.

use crate::support::{self, PRODUCT_TIMEOUT, Probe, SERVER};
use ide_app::language::{
    config::LanguageSetup,
    launch::{LaunchRequest, ServerLaunch, launch_directly},
    reply::{RequestFailure, RequestKind, RequestOutcome},
    status::{DocumentState, ServerState},
};
use serde_json::Value;

/// Process identifiers of every scripted server that started,
///  in order.
fn started(lines: &[Value]) -> Vec<u64> {
    let mut pids = Vec::new();
    for line in lines {
        if let Some(pid) = line["started"]["pid"].as_u64() {
            pids.push(pid);
        }
    }
    return pids;
}

/// A crash fails the pending request,
///  removes the server,
///  and the next open starts a new one.
#[test]
fn crash_fails_the_pending_request_and_the_next_open_restarts() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::crash_fails_the_pending_request_and_the_next_open_restarts",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("HOVER", "crash")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("first.scripted"), "alpha\n");
    probe.until_ready();
    probe.wait_for("diagnostics from the first process");
    while !(!probe.messages().is_empty()) {
        probe.step();
    }
    let number = probe.request(RequestKind::Hover, 1);
    let answers = probe.answers(number);
    assert_eq!(
        answers[0].outcome,
        RequestOutcome::Failed(RequestFailure::StreamClosed),
        "the request pending at the crash did not fail"
    );
    probe.wait_for("the exited state");
    while !(probe.state(SERVER) == Some(&ServerState::Exited)) {
        probe.step();
    }
    assert!(
        probe.messages().is_empty(),
        "diagnostics of an exited server stayed on display"
    );
    support::children_until_none();
    let after_exit = probe.request(RequestKind::Definition, 1);
    // An explicit request starts the server again; this request is answered by the starting state.
    let restarted = probe.answers(after_exit);
    assert_eq!(
        restarted[0].outcome,
        RequestOutcome::Starting,
        "an exited server could not be started again"
    );
    assert_eq!(support::instance_of(&restarted[0]), Some(2));
    probe.until_ready();
    assert_eq!(probe.status.servers.len(), 1);
    assert_eq!(probe.status.servers[0].server.instance, 2);
    let definition = probe.request(RequestKind::Definition, 1);
    assert!(matches!(
        probe.answers(definition)[0].outcome,
        RequestOutcome::Locations(_)
    ));
    // Crash again, then restart by displaying a file.
    let second_crash = probe.request(RequestKind::Hover, 1);
    probe.answers(second_crash);
    probe.wait_for("the second exit");
    while !(probe.state(SERVER) == Some(&ServerState::Exited)) {
        probe.step();
    }
    probe.open(&root.join("second.scripted"), "beta\n");
    probe.until_ready();
    assert_eq!(probe.status.servers[0].server.instance, 3);
    let pids = started(&support::report(&root));
    assert_eq!(pids.len(), 3, "three processes were expected: {pids:?}");
    assert!(pids[0] != pids[1] && pids[1] != pids[2]);
}

/// Switching files closes the first on the server and reuses the same process for the second.
#[test]
fn file_switch_closes_the_first_file_and_reuses_the_server() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::file_switch_closes_the_first_file_and_reuses_the_server",
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
            "lifecycle::file_outside_the_project_attaches_only_running_servers",
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

/// A launch policy that cannot produce a safe launch.
fn refusing(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    return Err(format!("no confinement for {}", request.server));
}

/// A launch policy that needs a private directory below the project,
///  which the test configures as
/// the state root itself.
fn needing_state(request: &LaunchRequest) -> Result<ServerLaunch, String> {
    let mut launch = launch_directly(request)?;
    launch
        .directories
        .push(request.project_root.join("state-inside-the-project"));
    return Ok(launch);
}

/// A refused or unpreparable launch is reported and nothing is spawned in its place.
#[test]
fn refused_launch_is_reported_and_nothing_is_spawned() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::refused_launch_is_reported_and_nothing_is_spawned",
            support::standard,
        );
        return;
    };
    let refused = LanguageSetup {
        launch: refusing,
        state_root: None,
        extra_languages: Some(support::scripted(&root, &[], PRODUCT_TIMEOUT)),
        ..LanguageSetup::unconfined()
    };
    let mut probe = Probe::with_setup(&root, refused);
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the refused state");
    while !(matches!(probe.state(SERVER), Some(ServerState::LaunchRefused { .. }))) {
        probe.step();
    }
    assert_eq!(
        probe.state(SERVER),
        Some(&ServerState::LaunchRefused {
            reason: "no confinement for scripted-ls".to_string()
        })
    );
    drop(probe);
    // The project itself as the state root would make all of it writable, so preparation refuses it.
    let unprepared = LanguageSetup {
        launch: needing_state,
        state_root: Some(root.clone()),
        extra_languages: Some(support::scripted(&root, &[], PRODUCT_TIMEOUT)),
        ..LanguageSetup::unconfined()
    };
    let mut second = Probe::with_setup(&root, unprepared);
    second.open(&root.join("file.scripted"), "alpha\n");
    second.wait_for("the refused state");
    while !(matches!(
        second.state(SERVER),
        Some(ServerState::LaunchRefused { .. })
    )) {
        second.step();
    }
    let Some(ServerState::LaunchRefused { reason }) = second.state(SERVER) else {
        panic!("unexpected state {:?}", second.state(SERVER));
    };
    assert!(reason.contains("contains the project"), "{reason}");
    assert!(
        !root.join("state-inside-the-project").exists(),
        "a private directory was created inside the project"
    );
    assert!(
        support::children().is_empty(),
        "a server was spawned although its launch was refused"
    );
    assert!(
        support::report(&root).is_empty(),
        "the server ran although its launch was refused"
    );
}

/// Dropping the handle stops every server,
///  including one that never initialized.
#[test]
fn dropping_the_worker_leaves_no_child_process() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::dropping_the_worker_leaves_no_child_process",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.until_ready();
    assert_eq!(support::children().len(), 1);
    drop(probe);
    support::children_until_none();
    let received = support::received(&support::report(&root));
    assert!(
        received.contains(&"shutdown".to_string()) && received.contains(&"exit".to_string()),
        "the server was not asked to shut down: {received:?}"
    );
    // A server stuck in `initialize` cannot be asked. helix-lsp closes its input instead, and
    // keeps its handle inside the task that awaits `initialize` until the worker's runtime ends.
    let mut hanging = Probe::new(&root, support::scripted(&root, &[("INIT", "hang")], 30));
    hanging.open(&root.join("file.scripted"), "alpha\n");
    hanging.wait_for("the starting state");
    while !(hanging.state(SERVER) == Some(&ServerState::Starting)) {
        hanging.step();
    }
    assert_eq!(support::children().len(), 1);
    drop(hanging);
    support::children_until_none();
}

/// A server that ignores `exit` and the end of its input is killed after the grace,
///  and it is
/// reaped before the drop returns:
///  no child is left,
///  running or as a zombie.
#[test]
fn server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "lifecycle::server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("LINGER", "1")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.until_ready();
    assert_eq!(support::children().len(), 1);
    let before = std::time::Instant::now();
    drop(probe);
    // Checked at once, without waiting: the drop joins the worker thread, and that thread ends
    // only when it has reaped what it killed.
    let left = support::children();
    assert!(
        left.is_empty(),
        "a child process was left when the drop returned: {left:?}"
    );
    assert!(
        before.elapsed() >= std::time::Duration::from_secs(1),
        "the server was not given its grace to exit: {:?}",
        before.elapsed()
    );
    let received = support::received(&support::report(&root));
    assert!(
        received.contains(&"shutdown".to_string()) && received.contains(&"exit".to_string()),
        "the server was not asked to shut down before it was killed: {received:?}"
    );
}
