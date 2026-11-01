//! Start states:
//!  nothing to start,
//!  missing program,
//!  on-demand start,
//!  and the three failed starts.

use crate::support::{self, PRODUCT_TIMEOUT, Probe, SERVER};
use ide_app::language::{
    reply::{RequestKind, RequestOutcome},
    status::{DocumentState, ServerState},
};

/// A file without a recognized language,
///  and a language without a server,
///  start nothing.
#[test]
fn files_without_language_or_server_start_nothing() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "start::files_without_language_or_server_start_nothing",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.open(&root.join("notes.unrecognized-extension"), "plain text\n");
    probe.wait_for("the no-language state");
    while !(probe.status.document == DocumentState::NoLanguage) {
        probe.step();
    }
    assert!(probe.status.servers.is_empty());
    let number = probe.request(RequestKind::Hover, 0);
    let answers = probe.answers(number);
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].outcome, RequestOutcome::NoServer);
    assert_eq!(answers[0].server, None);
    // Helix's built-in SQL definition names no language server.
    probe.open(&root.join("query.sql"), "select 1;\n");
    probe.wait_for("the no-server state");
    while !(probe.status.document == DocumentState::NoServerConfigured) {
        probe.step();
    }
    assert_eq!(probe.status.language.as_deref(), Some("sql"));
    assert!(
        support::children().is_empty(),
        "a process was started for a file without a server"
    );
}

/// A server whose program is not installed is reported and never started.
#[test]
fn missing_program_is_reported_without_starting_anything() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "start::missing_program_is_reported_without_starting_anything",
            support::standard,
        );
        return;
    };
    let definitions = support::scripted(&root, &[], PRODUCT_TIMEOUT).replace(
        env!("CARGO_BIN_EXE_ide-scripted-lsp"),
        "definitely-not-installed-language-server",
    );
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("file.scripted"), "one\n");
    probe.wait_for("the missing-executable state");
    while !(matches!(
        probe.state(SERVER),
        Some(ServerState::MissingExecutable { .. })
    )) {
        probe.step();
    }
    assert_eq!(
        probe.state(SERVER),
        Some(&ServerState::MissingExecutable {
            reason: "the program 'definitely-not-installed-language-server' for scripted-ls was not found on PATH".to_string()
        })
    );
    assert_eq!(probe.status.servers[0].server.instance, 0);
    assert!(support::children().is_empty());
    let number = probe.request(RequestKind::Definition, 0);
    assert_eq!(probe.answers(number)[0].outcome, RequestOutcome::NoServer);
}

/// A server starts when its first file is displayed,
///  and receives `didOpen` once,
///  after `initialized`.
#[test]
fn server_starts_on_demand_and_is_opened_once_after_initialized() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "start::server_starts_on_demand_and_is_opened_once_after_initialized",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[], PRODUCT_TIMEOUT));
    probe.poll();
    assert_eq!(probe.status.document, DocumentState::Closed);
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(
        support::children().is_empty(),
        "a server was started before any file was displayed"
    );
    let file = root.join("file.scripted");
    probe.open(&file, "alpha\nbeta\n");
    probe.until_ready();
    assert_eq!(probe.status.document, DocumentState::Attached);
    assert_eq!(probe.status.language.as_deref(), Some("scripted"));
    assert_eq!(probe.status.servers[0].server.instance, 1);
    let features = probe.status.servers[0]
        .features
        .expect("features of a ready server");
    assert!(features.definition && features.references && features.hover && features.inlay_hints);
    assert!(!features.pull_diagnostics);
    let lines = support::server_text_until(&root, "alpha\nbeta\n");
    let received = support::received(&lines);
    assert_eq!(received[0], "initialize", "{received:?}");
    assert_eq!(received[1], "initialized", "{received:?}");
    assert_eq!(
        support::count_received(&lines, "textDocument/didOpen"),
        1,
        "the server did not receive exactly one didOpen: {received:?}"
    );
    assert!(received.contains(&"workspace/didChangeConfiguration".to_string()));
    let initialize = support::first_received(&lines, "initialize").expect("initialize");
    let root_uri = format!("file://{}", root.display());
    assert_eq!(initialize["params"]["rootUri"], root_uri.as_str());
    assert_eq!(
        initialize["params"]["workspaceFolders"][0]["uri"],
        root_uri.as_str()
    );
    assert_eq!(
        initialize["params"]["initializationOptions"]["scripted"]["nested"]["value"],
        42
    );
    let started = support::first_started(&lines).expect("start record");
    assert_eq!(
        started["started"]["cwd"],
        root.display().to_string().as_str()
    );
    let opened = support::first_received(&lines, "textDocument/didOpen").expect("didOpen");
    assert_eq!(opened["params"]["textDocument"]["version"], 0);
    assert_eq!(opened["params"]["textDocument"]["languageId"], "scripted");
    assert_eq!(
        opened["params"]["textDocument"]["uri"],
        format!("file://{}", file.display()).as_str()
    );
    assert_eq!(
        support::server_text(&lines),
        Some((0, "alpha\nbeta\n".to_string()))
    );
}

/// A request sent while the server is still starting is answered "starting";
///  nothing reaches the server.
#[test]
fn request_before_initialize_is_answered_starting_and_nothing_is_sent() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "start::request_before_initialize_is_answered_starting_and_nothing_is_sent",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[("INIT", "hang")], 1));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the starting state");
    while !(probe.state(SERVER) == Some(&ServerState::Starting)) {
        probe.step();
    }
    assert_eq!(probe.status.servers[0].features, None);
    for kind in [
        RequestKind::Hover,
        RequestKind::Definition,
        RequestKind::References,
    ] {
        let number = probe.request(kind, 1);
        let answers = probe.answers(number);
        assert_eq!(
            answers[0].outcome,
            RequestOutcome::Starting,
            "a request before initialize was not answered as starting"
        );
        assert_eq!(support::instance_of(&answers[0]), Some(1));
    }
    probe.reload("alpha changed\n");
    assert!(
        probe
            .worker
            .request_hints(
                probe.stamp(),
                ide_app::language::hints::HintWindow {
                    first_line: 0,
                    visible_lines: 10
                }
            )
            .expect("a request before initialize stopped the language worker")
    );
    // The start deadline is the start allowance, three request timeouts (3 s), plus the margin.
    probe.wait_for("the failed-start state");
    while !(matches!(probe.state(SERVER), Some(ServerState::FailedToStart { .. }))) {
        probe.step();
    }
    assert_eq!(
        probe.state(SERVER),
        Some(&ServerState::FailedToStart {
            reason: "scripted-ls did not answer initialize within 3 seconds".to_string()
        })
    );
    let received = support::received(&support::report(&root));
    assert_eq!(
        received,
        ["initialize"],
        "something was sent to a server that had not initialized"
    );
    support::children_until_none();
}

/// A server that exits instead of answering `initialize` is a failed start.
#[test]
fn exit_during_initialize_is_a_failed_start() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "start::exit_during_initialize_is_a_failed_start",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(
        &root,
        support::scripted(&root, &[("INIT", "exit")], PRODUCT_TIMEOUT),
    );
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the failed-start state");
    while !(matches!(probe.state(SERVER), Some(ServerState::FailedToStart { .. }))) {
        probe.step();
    }
    assert_eq!(
        probe.state(SERVER),
        Some(&ServerState::FailedToStart {
            reason: "the server process ended before it finished starting".to_string()
        })
    );
    let number = probe.request(RequestKind::Hover, 0);
    assert_eq!(probe.answers(number)[0].outcome, RequestOutcome::NoServer);
    support::children_until_none();
}

/// A server that answers `initialize` with an error stays alive;
///  only the deadline reveals the failure.
#[test]
fn error_reply_to_initialize_is_a_failed_start_by_deadline() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "start::error_reply_to_initialize_is_a_failed_start_by_deadline",
            support::standard,
        );
        return;
    };
    let mut probe = Probe::new(&root, support::scripted(&root, &[("INIT", "error")], 1));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the starting state");
    while !(probe.state(SERVER) == Some(&ServerState::Starting)) {
        probe.step();
    }
    assert_eq!(support::children().len(), 1);
    probe.wait_for("the failed-start state");
    while !(matches!(probe.state(SERVER), Some(ServerState::FailedToStart { .. }))) {
        probe.step();
    }
    support::children_until_none();
    // The next explicit open tries again with a new process.
    probe.open(&root.join("other.scripted"), "beta\n");
    probe.wait_for("a second start");
    while !(probe.state(SERVER) == Some(&ServerState::Starting)
        && probe.status.servers[0].server.instance == 2)
    {
        probe.step();
    }
}

/// A server that answers `initialize` later than its request timeout,
///  but within its start
/// allowance of three request timeouts,
///  is ready;
///  the late answer is not discarded.
#[test]
fn initialize_answered_within_the_start_allowance_is_accepted() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "start::initialize_answered_within_the_start_allowance_is_accepted",
            support::standard,
        );
        return;
    };
    // A request timeout of 4 s gives a start allowance of 12 s; the answer comes after 5 s.
    // The answer can never beat the request timeout, so without the allowance the start fails
    // every time; with it, a stall of up to 7 s still passes.
    let definitions = support::scripted(&root, &[("INIT_DELAY_MS", "5000")], 4);
    let mut probe = Probe::new(&root, definitions);
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the server to be ready or to fail");
    while !matches!(
        probe.state(SERVER),
        Some(ServerState::Ready | ServerState::FailedToStart { .. })
    ) {
        probe.step();
    }
    assert_eq!(
        probe.state(SERVER),
        Some(&ServerState::Ready),
        "a server that answered initialize within its start allowance was not accepted"
    );
    assert_eq!(probe.status.servers[0].server.instance, 1);
    let mut started = 0;
    for line in support::report(&root).iter() {
        if !line["started"].is_null() {
            started += 1;
        }
    }
    assert_eq!(started, 1, "the server was started more than once");
}
