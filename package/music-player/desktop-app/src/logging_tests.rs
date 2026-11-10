// What:     Unit tests for `logging.rs`, pulled in by `#[cfg(test)] #[path = "logging_tests.rs"] mod tests;`.
// Why:      Issue 1010: the file dialog library reports a chooser failure through `log`, and ICU's CJK-segmentation
//           warnings arrive through the same facade. The bridge must carry the first and not the second.

use super::{DEFAULT_LEVEL, SILENT_TARGETS, directives, install_with};
use crate::logging::log_capture::{Capture, capture_records};

/// What:
///     Emits one probe per target a filter has to decide about,
/// under the filter built from `rust_log`.
/// Why:
///      A tracing macro's `target` must be a literal,
/// so each probe is written out;
/// a test then names only the
///           filter and the records it expects.
///
/// The records are:
/// a failure and a fallback warning from the chooser library,
/// two ICU warnings,
/// and the player's own
/// records at two levels.
fn written(rust_log: Option<&str>) -> String {
    return capture_records(rust_log, || {
        tracing::error!(target: "rfd::backend::xdg_desktop_portal", "probe=chooser-error");
        tracing::warn!(target: "rfd::backend::xdg_desktop_portal", "probe=chooser-warn");
        tracing::warn!(target: "icu_provider::error", "probe=icu-warn");
        tracing::warn!(target: "icu_segmenter::complex", "probe=icu-segmentation-warn");
        tracing::info!(target: "music_player::ui_wire", "probe=app-info");
        tracing::debug!(target: "music_player::ui_wire", "probe=app-debug");
    });
}

#[test]
fn a_chooser_failure_from_the_log_facade_reaches_the_log() {
    let records = written(None);
    assert!(
        records.contains("probe=chooser-error"),
        "the dialog library's error record must reach the sink; got {records:?}",
    );
    assert!(
        records.contains("probe=chooser-warn"),
        "the warning about falling back to a chooser program must reach the sink; got {records:?}",
    );
}

#[test]
fn icu_segmentation_warnings_stay_silent() {
    let records = written(None);
    assert!(!records.contains("probe=icu-warn"), "ICU warnings must stay out of the log; got {records:?}");
    assert!(
        !records.contains("probe=icu-segmentation-warn"),
        "ICU segmentation warnings must stay out of the log; got {records:?}",
    );
}

#[test]
fn the_default_level_is_info() {
    let records = written(None);
    assert!(records.contains("probe=app-info"), "info is the default; got {records:?}");
    assert!(!records.contains("probe=app-debug"), "debug needs RUST_LOG; got {records:?}");
}

#[test]
fn naming_a_silenced_target_in_rust_log_turns_it_back_on() {
    let records = written(Some("icu_provider=warn"));
    assert!(
        records.contains("probe=icu-warn"),
        "an operator who names the target must hear it; got {records:?}",
    );
    assert!(
        !records.contains("probe=icu-segmentation-warn"),
        "a target the operator did not name stays silent; got {records:?}",
    );
}

#[test]
fn a_rust_log_override_needs_no_second_directive_to_keep_the_silence() {
    let records = written(Some("music_player=debug"));
    assert!(records.contains("probe=app-debug"), "the override applies; got {records:?}");
    assert!(!records.contains("probe=icu-warn"), "the silence survives an override; got {records:?}");
}

#[test]
fn an_invalid_rust_log_falls_back_to_the_default_level() {
    let records = written(Some("music_player=nonsense"));
    assert!(records.contains("probe=app-info"), "a typo must not silence the player; got {records:?}");
    assert!(!records.contains("probe=icu-warn"), "the silence survives a typo; got {records:?}");
}

#[test]
fn directives_put_the_operator_last_so_the_override_wins() {
    assert_eq!(directives(None), format!("{SILENT_TARGETS},{DEFAULT_LEVEL}"));
    assert_eq!(directives(Some("   ")), format!("{SILENT_TARGETS},{DEFAULT_LEVEL}"));
    assert_eq!(directives(Some("  rfd=debug  ")), format!("{SILENT_TARGETS},rfd=debug"));
    assert!(
        directives(Some("icu_provider=debug")).ends_with(",icu_provider=debug"),
        "the operator's directive must come after the silence it overrides",
    );
}

#[test]
fn the_silence_names_both_icu_crates() {
    assert!(SILENT_TARGETS.contains("icu_provider=off"), "got {SILENT_TARGETS:?}");
    assert!(SILENT_TARGETS.contains("icu_segmenter=off"), "got {SILENT_TARGETS:?}");
}

#[test]
fn a_record_from_the_log_facade_reaches_the_installed_pipeline() {
    // What:     the pipeline `install` installs, over a buffer: `try_init` sets the global default and the `log` bridge
    //           from tracing-subscriber's default features.
    // Why:      A `log` record is only ever visible through the bridge, so this is the test that fails when the bridge is
    //           not installed. icu_provider re-exports the facade it logs through (its `logging` feature is on).
    let buffer: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::default();
    let sink = Capture {
        buffer: std::sync::Arc::clone(&buffer),
    };
    assert!(install_with(sink, None), "the test binary installs the pipeline first");
    icu_provider::log::warn!("probe=facade-warn");
    icu_provider::log::warn!(target: "icu_provider", "probe=icu-warn");
    let records: String = String::from_utf8_lossy(&buffer.lock().expect("the capture buffer is readable")).into_owned();
    assert!(
        records.contains("probe=facade-warn"),
        "a `log` record must reach the sink through the bridge; got {records:?}",
    );
    assert!(
        !records.contains("probe=icu-warn"),
        "the silenced target stays out even through the bridge; got {records:?}",
    );
}
