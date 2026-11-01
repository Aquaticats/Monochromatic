//! The default level,
//!  the `RUST_LOG` override,
//!  and helix-lsp's directive inside it,
//!  decided by the
//! real filter under a scoped subscriber;
//!  the environment itself is never changed.

use super::directives;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
use tracing_subscriber::{EnvFilter, fmt::MakeWriter};

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

/// Emit one record per probe under the filter built from `defaults` and `requested`,
///  and return
/// the names of the probes that were written.
fn written(defaults: &str, requested: Option<&str>) -> Vec<&'static str> {
    let capture = Capture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::builder().parse_lossy(directives(defaults, requested)))
        .with_ansi(false)
        .with_writer(capture.clone())
        .finish();
    // The guard keeps the subscriber as this thread's default until it is dropped.
    let default_guard = tracing::subscriber::set_default(subscriber);
    tracing::warn!(target: "slint", "probe=slint-warn");
    tracing::info!(target: "slint", "probe=slint-info");
    tracing::warn!(target: "ide_app::language", "probe=app-warn");
    tracing::info!(target: "ide_app::language", "probe=app-info");
    tracing::debug!(target: "ide_app::language", "probe=app-debug");
    tracing::warn!(target: "helix_lsp::transport", "probe=helix-warn");
    tracing::info!(target: "helix_lsp::transport", "probe=helix-info");
    tracing::error!(target: "helix_lsp::transport", "probe=helix-error");
    drop(default_guard);
    let text = String::from_utf8(capture.0.lock().expect("capture").clone()).expect("text");
    let probes = [
        "slint-warn",
        "slint-info",
        "app-warn",
        "app-info",
        "app-debug",
        "helix-warn",
        "helix-info",
        "helix-error",
    ];
    let mut found = Vec::new();
    for probe in probes {
        if text.contains(&format!("probe={probe}\n")) {
            found.push(probe);
        }
    }
    return found;
}

#[test]
fn directives_put_the_override_last() {
    assert_eq!(directives("", None), "warn,helix_lsp=warn");
    assert_eq!(
        directives("ide_app=debug", None),
        "warn,helix_lsp=warn,ide_app=debug"
    );
    assert_eq!(
        directives("ide_app=debug", Some("info")),
        "warn,helix_lsp=warn,ide_app=debug,info"
    );
    assert_eq!(directives("", Some("  ")), "warn,helix_lsp=warn");
}

#[test]
fn the_default_is_warnings_and_errors() {
    assert_eq!(
        written("", None),
        vec!["slint-warn", "app-warn", "helix-warn", "helix-error"]
    );
}

#[test]
fn a_global_override_adds_detail_but_keeps_helix_at_warnings() {
    assert_eq!(
        written("", Some("debug")),
        vec![
            "slint-warn",
            "slint-info",
            "app-warn",
            "app-info",
            "app-debug",
            "helix-warn",
            "helix-error"
        ]
    );
}

#[test]
fn a_targeted_override_keeps_warnings_from_everything_else() {
    assert_eq!(
        written("", Some("ide_app=debug")),
        vec![
            "slint-warn",
            "app-warn",
            "app-info",
            "app-debug",
            "helix-warn",
            "helix-error"
        ]
    );
}

#[test]
fn naming_helix_in_the_override_replaces_its_directive() {
    assert_eq!(
        written("", Some("helix_lsp=info")),
        vec![
            "slint-warn",
            "app-warn",
            "helix-warn",
            "helix-info",
            "helix-error"
        ]
    );
    assert_eq!(
        written("", Some("helix_lsp::transport=info")),
        vec![
            "slint-warn",
            "app-warn",
            "helix-warn",
            "helix-info",
            "helix-error"
        ]
    );
}

#[test]
fn an_override_can_also_reduce_the_log() {
    assert_eq!(directives("", Some("error")), "warn,helix_lsp=error,error");
    assert_eq!(written("", Some("error")), vec!["helix-error"]);
    assert_eq!(written("", Some("off")), Vec::<&str>::new());
    assert_eq!(
        written("ide_app=debug", Some("ide_app=warn")),
        vec!["slint-warn", "app-warn", "helix-warn", "helix-error"]
    );
}
