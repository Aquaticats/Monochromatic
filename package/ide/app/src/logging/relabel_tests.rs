//! Record shapes from helix-lsp's format strings at the pinned revision,
//!  near misses that keep
//! their level,
//!  and the re-labelled record as the application's subscriber writes it.

use super::{Matched, RELABEL, Shape, matched, shape};
use crate::logging::stderr_tail;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
use tracing_log::log::{self, Log};
use tracing_subscriber::fmt::MakeWriter;

const TRANSPORT: &str = "helix_lsp::transport";

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

/// Send one `log` record with this target,
///  level,
///  and text through the bridge,
///  under a subscriber
/// with `filter`,
///  and return what the subscriber wrote.
fn written(filter: &str, target: &str, level: log::Level, text: &str) -> String {
    let capture = Capture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(capture.clone())
        .finish();
    // The guard keeps the subscriber as this thread's default until it is dropped.
    let default_guard = tracing::subscriber::set_default(subscriber);
    RELABEL.log(
        &log::Record::builder()
            .args(format_args!("{text}"))
            .level(level)
            .target(target)
            .module_path(Some(target))
            .file(Some("helix-lsp/src/transport.rs"))
            .line(Some(168))
            .build(),
    );
    drop(default_guard);
    return String::from_utf8(capture.0.lock().expect("capture").clone()).expect("text");
}

#[test]
fn a_server_stderr_line_is_recognized() {
    assert_eq!(
        shape(
            TRANSPORT,
            log::Level::Error,
            r#"typescript-native err <- "context canceled\n""#
        ),
        Some(Shape::ServerStderrLine)
    );
    assert_eq!(
        shape(
            TRANSPORT,
            log::Level::Error,
            r#"rust-analyzer err <- "2026-10-05T17:08:48  WARN notify error: No path was found.\n""#
        ),
        Some(Shape::ServerStderrLine)
    );
}

#[test]
fn the_end_of_server_stderr_is_recognized() {
    assert_eq!(
        shape(
            TRANSPORT,
            log::Level::Error,
            "rust-analyzer err: <- StreamClosed"
        ),
        Some(Shape::EndOfServerStderr)
    );
}

#[test]
fn moot_error_answers_are_recognized() {
    assert_eq!(
        shape(
            TRANSPORT,
            log::Level::Error,
            "rust-analyzer <- ServerError(-32801): content modified"
        ),
        Some(Shape::MootAnswer)
    );
    assert_eq!(
        shape(
            TRANSPORT,
            log::Level::Error,
            "scripted-ls <- ServerError(-32800): request cancelled"
        ),
        Some(Shape::MootAnswer)
    );
}

#[test]
fn other_error_records_keep_their_level() {
    let kept = [
        // An error answer that is a real request failure.
        (
            TRANSPORT,
            "scripted-ls <- InternalError: scripted initialize failure",
        ),
        (
            TRANSPORT,
            "scripted-ls <- ServerError(-32803): request failed",
        ),
        // A failure to read or write a server's streams.
        (
            TRANSPORT,
            "rust-analyzer err: <- IO(Os { code: 32, kind: BrokenPipe, message: \"Broken pipe\" })",
        ),
        (
            TRANSPORT,
            "Exiting rust-analyzer after unexpected error: Parse(\"bad header\")",
        ),
        (
            TRANSPORT,
            "Could not close request on a closed channel (id=Num(3))",
        ),
        // The same text without a server name in front.
        (TRANSPORT, " err: <- StreamClosed"),
        (TRANSPORT, " err <- \"line\""),
        (TRANSPORT, " <- ServerError(-32801): content modified"),
        // A quoted line that is cut off is not the stderr shape.
        (TRANSPORT, "scripted-ls err <- \"unterminated"),
        // Another helix-lsp module.
        (
            "helix_lsp",
            "failed to initialize language server: scripted initialize failure",
        ),
        ("helix_lsp", "scripted-ls err <- \"line\""),
    ];
    for (target, text) in kept {
        assert_eq!(
            shape(target, log::Level::Error, text),
            None,
            "{target}: {text}"
        );
    }
    // A record that is not at ERROR is never touched, whatever its text.
    assert_eq!(
        shape(
            TRANSPORT,
            log::Level::Warn,
            "scripted-ls err: <- StreamClosed"
        ),
        None
    );
}

#[test]
fn each_shape_has_its_level() {
    assert_eq!(Shape::ServerStderrLine.level(), log::Level::Info);
    assert_eq!(Shape::EndOfServerStderr.level(), log::Level::Debug);
    assert_eq!(Shape::MootAnswer.level(), log::Level::Debug);
}

#[test]
fn a_relabelled_record_keeps_its_target_and_text() {
    let text = r#"typescript-native err <- "context canceled\n""#;
    let stderr_line = written("helix_lsp=debug", TRANSPORT, log::Level::Error, text);
    assert!(
        stderr_line.ends_with(&format!(" INFO helix_lsp::transport: {text}\n")),
        "{stderr_line}"
    );
    let end_line = written(
        "helix_lsp=debug",
        TRANSPORT,
        log::Level::Error,
        "scripted-ls err: <- StreamClosed",
    );
    assert!(
        end_line.ends_with(" DEBUG helix_lsp::transport: scripted-ls err: <- StreamClosed\n"),
        "{end_line}"
    );
}

#[test]
fn a_relabelled_record_is_filtered_at_its_new_level() {
    let text = "scripted-ls err: <- StreamClosed";
    assert_eq!(written("warn", TRANSPORT, log::Level::Error, text), "");
    assert_eq!(
        written("helix_lsp=info", TRANSPORT, log::Level::Error, text),
        ""
    );
    assert!(!written("helix_lsp=debug", TRANSPORT, log::Level::Error, text).is_empty());
}

#[test]
fn an_unknown_error_record_stays_error() {
    let text =
        "rust-analyzer err: <- IO(Os { code: 32, kind: BrokenPipe, message: \"Broken pipe\" })";
    let transport_line = written("warn", TRANSPORT, log::Level::Error, text);
    assert!(
        transport_line.ends_with(&format!(" ERROR helix_lsp::transport: {text}\n")),
        "{transport_line}"
    );
    let helix_line = written(
        "warn",
        "helix_lsp",
        log::Level::Error,
        "failed to initialize language server: refused",
    );
    assert!(
        helix_line.ends_with(" ERROR helix_lsp: failed to initialize language server: refused\n"),
        "{helix_line}"
    );
}

#[test]
fn records_at_other_levels_pass_through_unchanged() {
    let line = written(
        "helix_lsp=info",
        TRANSPORT,
        log::Level::Info,
        "scripted-ls -> {\"jsonrpc\":\"2.0\"}",
    );
    assert!(
        line.ends_with(" INFO helix_lsp::transport: scripted-ls -> {\"jsonrpc\":\"2.0\"}\n"),
        "{line}"
    );
}

#[test]
fn a_match_names_the_server_and_the_quoted_line() {
    assert_eq!(
        matched(
            TRANSPORT,
            log::Level::Error,
            r#"scripted-ls err <- "say \"hi\"\n""#
        ),
        Some(Matched {
            shape: Shape::ServerStderrLine,
            server: "scripted-ls",
            line: r#"say \"hi\"\n"#,
        })
    );
    assert_eq!(
        matched(
            TRANSPORT,
            log::Level::Error,
            "scripted-ls err: <- StreamClosed"
        ),
        Some(Matched {
            shape: Shape::EndOfServerStderr,
            server: "scripted-ls",
            line: "",
        })
    );
}

#[test]
fn the_bridge_keeps_stderr_lines_the_filter_hides() {
    let server = "relabel-tail";
    let line = format!(r#"{server} err <- "last words\n""#);
    assert_eq!(written("warn", TRANSPORT, log::Level::Error, &line), "");
    assert!(!stderr_tail::is_closed(server));
    let end = format!("{server} err: <- StreamClosed");
    assert_eq!(written("warn", TRANSPORT, log::Level::Error, &end), "");
    assert!(stderr_tail::is_closed(server));
    assert_eq!(stderr_tail::take(server), vec!["last words".to_string()]);
}
