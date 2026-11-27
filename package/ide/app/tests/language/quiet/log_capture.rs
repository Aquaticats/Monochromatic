//! The log capture the five tests below read:
//!  the application's own pipeline with only its writer replaced,
//!  the records of one level or one text,
//!  and one clean lifetime whose log they assert on.

use crate::support::{
    self,
    PRODUCT_TIMEOUT,
    Probe,
};
use ide_app::language::{
    reply::{
        RequestKind,
        RequestOutcome,
    },
    status::DocumentState,
};
use ide_app::logging;
use std::{
    io::{
        self,
        Write,
    },
    path::Path,
    sync::{
        Arc,
        Mutex,
    },
};
use tracing_subscriber::{
    EnvFilter,
    fmt::MakeWriter,
};

/// A log writer that keeps every byte in memory,
///  shared with the test that reads it.
#[derive(Clone, Default)]
pub struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> io::Result<usize> {
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
    pub fn text(&self) -> String {
        return String::from_utf8(self.0.lock().expect("log buffer").clone()).expect("log text");
    }
}

/// Install the application's log pipeline with every worker record and every helix-lsp record
/// visible,
///  writing into a capture.
///  Once per process,
///  so each test runs in its own child.
pub fn capture_the_log() -> Capture {
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
pub fn records_at(
    text: &str,
    level: &str,
) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        if line.split_whitespace().nth(1) == Some(level) {
            found.push(line.to_string());
        }
    }
    return found;
}

/// The field of the worker's records that lists a server's last standard-error lines.
pub const TAIL_FIELD: &str = "stderr_tail=";

/// The worker's record of a server process that ended without being asked to.
pub const ENDED: &str = "language server process ended";

/// The whole lines of the log that contain `text`.
pub fn records_with(
    log: &str,
    text: &str,
) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in log.lines() {
        if line.contains(text) {
            found.push(line.to_string());
        }
    }
    return found;
}

/// Assert that a record with this level and this ending is in the log.
pub fn assert_record(
    text: &str,
    level: &str,
    ending: &str,
) {
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
pub fn log_of_a_clean_lifetime(root: &Path) -> String {
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
