//! The records of an ended server and a timed-out start,
//!  under a capture subscriber on a runtime
//! like the worker's.
//!  Every test uses its own server name,
//!  because the kept lines are shared by
//! the whole test process.

use super::{AfterEnd, SETTLE, ended_unexpectedly, ended_unexpectedly_then, stopped_during_start};
use crate::logging::stderr_tail;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing_subscriber::fmt::MakeWriter;

/// Stores the lines it is handed where the test can read them.
struct Handover(Arc<Mutex<Option<Vec<String>>>>);

impl AfterEnd for Handover {
    fn after_end(self, lines: Vec<String>) {
        *self.0.lock().expect("sink") = Some(lines);
    }
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("capture").extend_from_slice(bytes);
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

/// Run `body` on a current-thread runtime under a capture subscriber,
///  drop the runtime while the
/// subscriber is still the default,
///  and return what was written.
fn logged<F>(body: F) -> String
where
    F: Future<Output = ()>,
{
    let capture = Capture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_ansi(false)
        .with_writer(capture.clone())
        .finish();
    // The guard keeps the subscriber as this thread's default until it is dropped.
    let default_guard = tracing::subscriber::set_default(subscriber);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("runtime");
    runtime.block_on(body);
    drop(runtime);
    drop(default_guard);
    return String::from_utf8(capture.0.lock().expect("capture").clone()).expect("text");
}

/// The lines of `text` that hold `message`.
fn records_with<'text>(text: &'text str, message: &str) -> Vec<&'text str> {
    let mut found: Vec<&'text str> = Vec::new();
    for line in text.lines() {
        if line.contains(message) {
            found.push(line);
        }
    }
    return found;
}

#[test]
fn an_end_waits_for_lines_that_arrive_after_it() {
    let server = "report-late";
    let text = logged(async {
        ended_unexpectedly(server.to_string(), true);
        tokio::time::sleep(Duration::from_millis(50)).await;
        stderr_tail::remember(server, "panicked at main.rs:1:1\\n");
        stderr_tail::ended(server);
        tokio::time::sleep(Duration::from_millis(200)).await;
    });
    let ended = records_with(&text, "language server process ended");
    assert_eq!(ended.len(), 1, "{text}");
    assert!(ended[0].contains(" WARN "), "{text}");
    assert!(
        ended[0].contains(
            r#"server=report-late was_ready=true stderr_tail=["panicked at main.rs:1:1"]"#
        ),
        "the record did not wait for the last line:\n{text}"
    );
    assert!(
        text.contains("waited for the end of a server's standard error"),
        "{text}"
    );
}

#[test]
fn the_lines_are_handed_over_complete_and_the_record_still_reports_them() {
    let server = "report-handover";
    let handed: Arc<Mutex<Option<Vec<String>>>> = Arc::new(Mutex::new(None));
    let sink = handed.clone();
    let text = logged(async {
        ended_unexpectedly_then(server.to_string(), false, Handover(sink));
        tokio::time::sleep(Duration::from_millis(50)).await;
        stderr_tail::remember(server, "cannot read the configuration\n");
        stderr_tail::ended(server);
        tokio::time::sleep(Duration::from_millis(200)).await;
    });
    assert_eq!(
        handed.lock().expect("sink").clone(),
        Some(vec!["cannot read the configuration".to_string()]),
        "the hand-over was not the complete lines:\n{text}"
    );
    assert!(
        text.contains(
            r#"server=report-handover was_ready=false stderr_tail=["cannot read the configuration"]"#
        ),
        "handing the lines over drained them from the log record:\n{text}"
    );
}

#[test]
fn an_end_whose_stream_already_ended_is_written_without_waiting() {
    let server = "report-closed";
    let text = logged(async {
        stderr_tail::remember(server, "bye\\n");
        stderr_tail::ended(server);
        ended_unexpectedly(server.to_string(), false);
        tokio::task::yield_now().await;
    });
    assert!(
        text.contains(r#"language server process ended server=report-closed was_ready=false stderr_tail=["bye"]"#),
        "{text}"
    );
    assert!(!text.contains("waited for"), "{text}");
}

#[test]
fn an_end_is_written_when_the_runtime_stops_during_the_wait() {
    let server = "report-runtime-gone";
    let text = logged(async {
        stderr_tail::remember(server, "still open\\n");
        ended_unexpectedly(server.to_string(), true);
        tokio::task::yield_now().await;
    });
    assert!(
        text.contains(r#"stderr_tail=["still open"]"#),
        "the record was lost with the runtime:\n{text}"
    );
}

#[test]
fn an_end_gives_up_waiting_after_the_settle_time() {
    let server = "report-never-closed";
    let text = logged(async {
        stderr_tail::remember(server, "held open\\n");
        ended_unexpectedly(server.to_string(), true);
        tokio::time::sleep(SETTLE + Duration::from_millis(300)).await;
        // Written after the record: dropped, because the process's words were reported.
        stderr_tail::remember(server, "after the record\\n");
    });
    assert!(
        text.contains(r#"stderr_tail=["held open"]"#) && text.contains("closed=false"),
        "{text}"
    );
    assert!(stderr_tail::take(server).is_empty());
}

#[test]
fn an_end_without_lines_has_no_tail_field() {
    let server = "report-silent";
    let text = logged(async {
        stderr_tail::ended(server);
        ended_unexpectedly(server.to_string(), true);
        tokio::task::yield_now().await;
    });
    assert!(
        text.contains("language server process ended server=report-silent was_ready=true\n"),
        "{text}"
    );
}

#[test]
fn a_stopped_start_has_the_lines_so_far_and_nothing_without_them() {
    let server = "report-start";
    let text = logged(async {
        stderr_tail::remember(server, "loading project\\n");
        stopped_during_start(server);
        stopped_during_start("report-start-silent");
    });
    assert!(
        text.contains(r#" WARN ide_app::language::attach::report: language server wrote this to standard error before it was stopped server=report-start stderr_tail=["loading project"]"#),
        "{text}"
    );
    assert!(!text.contains("report-start-silent"), "{text}");
}
