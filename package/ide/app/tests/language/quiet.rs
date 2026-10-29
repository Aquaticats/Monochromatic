//! Quiet lifetime:
//!  what a clean open,
//!  one request,
//!  close,
//!  and shutdown leave in the application
//! log and in the process table;
//!  and what a server that crashes or fails to start leaves there,
//! with the last lines it wrote to standard error.
//!
//! The log is captured through the application's own pipeline (`ide_app::logging::install`):
//!  the
//! same subscriber and the same bridge that re-labels helix-lsp's `log` records.
//!  Only the writer is
//! replaced,
//!  and the level is set here,
//!  because these tests read records below the default.

use crate::support::{self, PRODUCT_TIMEOUT, Probe, SERVER};
use ide_app::language::{
    reply::{RequestKind, RequestOutcome},
    status::{DocumentState, ServerState},
};
use ide_app::logging;
use std::{
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex},
};
use tracing_subscriber::{EnvFilter, fmt::MakeWriter};

/// A log writer that keeps every byte in memory,
///  shared with the test that reads it.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("log buffer").extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        return Ok(());
    }
}

impl<'writer> MakeWriter<'writer> for Capture {
    type Writer = Capture;

    fn make_writer(&'writer self) -> Self::Writer {
        return self.clone();
    }
}

impl Capture {
    fn text(&self) -> String {
        return String::from_utf8(self.0.lock().expect("log buffer").clone()).expect("log text");
    }
}

/// Install the application's log pipeline with every worker record and every helix-lsp record
/// visible,
///  writing into a capture.
///  Once per process,
///  so each test runs in its own child.
fn capture_the_log() -> Capture {
    let capture = Capture::default();
    let filter = EnvFilter::builder()
        .parse_lossy(logging::directives("ide_app=debug,helix_lsp=debug", None));
    logging::install(filter, capture.clone(), Some(false)).expect("log pipeline");
    return capture;
}

/// The records of one level,
///  whole lines.
///  A record starts with its time,
///  then its level.
fn records_at(text: &str, level: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        if line.split_whitespace().nth(1) == Some(level) {
            found.push(line.to_string());
        }
    }
    return found;
}

/// The field of the worker's records that lists a server's last standard-error lines.
const TAIL_FIELD: &str = "stderr_tail=";

/// The worker's record of a server process that ended without being asked to.
const ENDED: &str = "language server process ended";

/// The whole lines of the log that contain `text`.
fn records_with(log: &str, text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in log.lines() {
        if line.contains(text) {
            found.push(line.to_string());
        }
    }
    return found;
}

/// Assert that a record with this level and this ending is in the log.
fn assert_record(text: &str, level: &str, ending: &str) {
    let mut ended = false;
    for line in records_at(text, level).iter() {
        if line.ends_with(ending) {
            ended = true;
        }
    }
    assert!(
        ended,
        "no {level} record ending with {ending:?} in the log:\n{text}"
    );
}

/// One clean lifetime in which the server produces all three record shapes helix-lsp logs at ERROR
/// for a healthy server:
///  a line on standard error at shutdown,
///  as TypeScript 7's server writes
/// `context canceled`;
///  a `-32801` answer that the worker asks again for;
///  and the end of standard
/// error when the process ends.
///  The server also writes a line at its start,
///  so the worker holds
/// kept lines that a clean shutdown must not report.
///  Returns the whole log.
fn log_of_a_clean_lifetime(root: &Path) -> String {
    let capture = capture_the_log();
    let variables = [
        ("STDERR_AT_START", "scripted server starting"),
        ("STDERR", "context canceled"),
        ("HOVER", "modified-once"),
    ];
    let mut probe = Probe::new(root, support::scripted(root, &variables, PRODUCT_TIMEOUT));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.until_ready();
    assert_eq!(support::children().len(), 1);
    let number = probe.request(RequestKind::Hover, 1);
    assert!(
        matches!(probe.answers(number)[0].outcome, RequestOutcome::Hover(_)),
        "the one request of the lifetime was not answered after the worker asked again"
    );
    probe.worker.close().expect("close");
    probe.wait_for("the closed state");
    while !(probe.status.document == DocumentState::Closed) {
        probe.step();
    }
    drop(probe);
    support::children_until_none();
    let received = support::received(&support::report(root));
    assert!(
        received.contains(&"shutdown".to_string()) && received.contains(&"exit".to_string()),
        "the server was not asked to shut down: {received:?}"
    );
    let text = capture.text();
    // Positive controls: the capture saw an application record and the shutdown itself.
    assert!(
        text.contains("language server is ready"),
        "the log capture saw no application record:\n{text}"
    );
    assert!(
        text.contains("language servers were asked to stop still_running=0"),
        "the server did not end by itself within the grace period:\n{text}"
    );
    return text;
}

/// A clean lifetime leaves no child process and no ERROR record,
///  and each healthy-server record
/// is still in the log,
///  whole,
///  at the level that says what it is.
#[test]
fn clean_lifetime_logs_no_error_level_record() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "quiet::clean_lifetime_logs_no_error_level_record",
            support::standard,
        );
        return;
    };
    let text = log_of_a_clean_lifetime(&root);
    let errors = records_at(&text, "ERROR");
    assert!(
        errors.is_empty(),
        "a clean lifetime logged at ERROR: {errors:#?}"
    );
    assert_record(
        &text,
        "INFO",
        r#"helix_lsp::transport: scripted-ls err <- "context canceled\n""#,
    );
    assert_record(
        &text,
        "DEBUG",
        "helix_lsp::transport: scripted-ls <- ServerError(-32801): content modified",
    );
    assert_record(
        &text,
        "DEBUG",
        "helix_lsp::transport: scripted-ls err: <- StreamClosed",
    );
    // A clean shutdown logs nothing extra: no ended-server record and no kept lines.
    assert_record(
        &text,
        "INFO",
        r#"helix_lsp::transport: scripted-ls err <- "scripted server starting\n""#,
    );
    let extra = records_with(&text, TAIL_FIELD);
    assert!(
        extra.is_empty() && records_with(&text, ENDED).is_empty(),
        "a clean shutdown logged the server's last lines or an ended-server record: {extra:#?}"
    );
}

/// helix-lsp records the bridge does not know keep ERROR:
///  here a server that refuses `initialize`,
/// which helix-lsp logs as an error answer with code `-32603` and as a failed initialization.
#[test]
fn unknown_helix_error_records_keep_their_level() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "quiet::unknown_helix_error_records_keep_their_level",
            support::standard,
        );
        return;
    };
    let capture = capture_the_log();
    let mut probe = Probe::new(&root, support::scripted(&root, &[("INIT", "error")], 1));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the failed-start state");
    while !(matches!(probe.state(SERVER), Some(ServerState::FailedToStart { .. }))) {
        probe.step();
    }
    drop(probe);
    support::children_until_none();
    let text = capture.text();
    assert_record(
        &text,
        "ERROR",
        "helix_lsp::transport: scripted-ls <- InternalError: scripted initialize failure",
    );
    assert_record(
        &text,
        "ERROR",
        "helix_lsp: failed to initialize language server: protocol error: InternalError: scripted initialize failure",
    );
}

/// A server that writes a line at its start and another right before it exits with status 7,
///  in
/// the middle of a hover request:
///  one warning names the server and holds both lines,
///  oldest first.
/// The last line arrives just before the end,
///  so the record waits for it.
#[test]
fn a_crashed_server_is_logged_with_its_last_stderr_lines() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "quiet::a_crashed_server_is_logged_with_its_last_stderr_lines",
            support::standard,
        );
        return;
    };
    let capture = capture_the_log();
    let variables = [
        ("STDERR_AT_START", "scripted server starting"),
        ("STDERR", "panicked at the hover request"),
        ("HOVER", "crash"),
    ];
    let mut probe = Probe::new(&root, support::scripted(&root, &variables, PRODUCT_TIMEOUT));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.until_ready();
    let _crashing = probe.request(RequestKind::Hover, 1);
    probe.wait_for("the exited state");
    while !(probe.state(SERVER) == Some(&ServerState::Exited)) {
        probe.step();
    }
    drop(probe);
    support::children_until_none();
    let text = capture.text();
    let ended = records_with(&text, ENDED);
    assert_eq!(ended.len(), 1, "not one ended-server record:\n{text}");
    assert!(
        ended[0].contains(" WARN ")
            && ended[0].ends_with(
                r#"server=scripted-ls was_ready=true stderr_tail=["scripted server starting", "panicked at the hover request"]"#
            ),
        "the ended-server record lacks the server's last lines: {}",
        ended[0]
    );
}

/// A server that writes a line and exits with status 3 when `initialize` arrives:
///  the warning
/// says it never finished starting and holds the line.
#[test]
fn a_server_that_ends_during_its_start_is_logged_with_its_stderr_lines() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "quiet::a_server_that_ends_during_its_start_is_logged_with_its_stderr_lines",
            support::standard,
        );
        return;
    };
    let capture = capture_the_log();
    let variables = [
        ("STDERR_AT_START", "cannot read the configuration"),
        ("INIT", "exit"),
    ];
    let mut probe = Probe::new(&root, support::scripted(&root, &variables, PRODUCT_TIMEOUT));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the failed-start state");
    while !(matches!(probe.state(SERVER), Some(ServerState::FailedToStart { .. }))) {
        probe.step();
    }
    // The server's last lines reach the reason once its standard error ended, after the first state.
    probe.wait_for("the cause in the reason");
    while !(matches!(
        probe.state(SERVER),
        Some(ServerState::FailedToStart { reason }) if reason.contains("cannot read the configuration")
    )) {
        probe.step();
    }
    assert_eq!(
        probe.state(SERVER),
        Some(&ServerState::FailedToStart {
            reason: "the server process ended before it finished starting; the server wrote this to its standard error:\ncannot read the configuration".to_string()
        })
    );
    // The same words are on screen as the entry that says why nothing works.
    probe.wait_for("the cause in the entry");
    while !(probe.notice_contains("cannot read the configuration")) {
        probe.step();
    }
    drop(probe);
    support::children_until_none();
    let text = capture.text();
    let ended = records_with(&text, ENDED);
    assert_eq!(ended.len(), 1, "not one ended-server record:\n{text}");
    assert!(
        ended[0].ends_with(
            r#"server=scripted-ls was_ready=false stderr_tail=["cannot read the configuration"]"#
        ),
        "the ended-server record lacks the server's last lines: {}",
        ended[0]
    );
}

/// A server that writes a line and never answers `initialize`:
///  the start-deadline error is followed
/// by a warning that holds the line;
///  the worker stopped the server itself,
///  so there is no
/// ended-server record and nothing it writes while it is stopped is reported.
#[test]
fn a_timed_out_start_is_logged_with_its_stderr_lines() {
    let Some(root) = support::child_root() else {
        support::run_child(
            "quiet::a_timed_out_start_is_logged_with_its_stderr_lines",
            support::standard,
        );
        return;
    };
    let capture = capture_the_log();
    let variables = [
        ("STDERR_AT_START", "indexing the workspace"),
        ("STDERR", "asked to stop"),
        ("INIT", "hang"),
    ];
    // A request timeout of 1 s gives a start allowance of three request timeouts, 3 s.
    let mut probe = Probe::new(&root, support::scripted(&root, &variables, 1));
    probe.open(&root.join("file.scripted"), "alpha\n");
    probe.wait_for("the failed-start state");
    while !(matches!(probe.state(SERVER), Some(ServerState::FailedToStart { .. }))) {
        probe.step();
    }
    assert_eq!(
        probe.state(SERVER),
        Some(&ServerState::FailedToStart {
            reason: "scripted-ls did not answer initialize within 3 seconds; the server wrote this to its standard error:\nindexing the workspace".to_string()
        })
    );
    drop(probe);
    support::children_until_none();
    let text = capture.text();
    assert_record(
        &text,
        "ERROR",
        "language server did not answer initialize in time and is stopped server=scripted-ls seconds=3",
    );
    assert_record(
        &text,
        "WARN",
        r#"language server wrote this to standard error before it was stopped server=scripted-ls stderr_tail=["indexing the workspace"]"#,
    );
    assert_eq!(
        records_with(&text, TAIL_FIELD).len(),
        1,
        "the stopped server's lines were reported more than once:\n{text}"
    );
    assert!(
        records_with(&text, ENDED).is_empty(),
        "a server the worker stopped was logged as ended:\n{text}"
    );
}
