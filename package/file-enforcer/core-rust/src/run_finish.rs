//! What:
//!  The last step of `check` and `write`:
//! report drift or publish,
//! and build the summary event.
//! Why:
//!  Kept apart from `run.rs` so each file stays under the length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function finishCheck(...): number; function finishWrite(...): number
//! ```

/// Imports `Drift` from `crate::drift`.
use crate::drift::Drift;
/// Imports `Event`,
/// `EventSink` from `crate::event`.
use crate::event::{Event, EventSink};
/// Imports `validate_expected_dir`,
/// `write_expected_trees` from `crate::expected_dir`.
use crate::expected_dir::{validate_expected_dir, write_expected_trees};
/// Imports `PlanReport` from `crate::plan`.
use crate::plan::PlanReport;
/// Imports `publish` from `crate::publish`.
use crate::publish::publish;
/// Imports `EXIT_CLEAN`,
/// `EXIT_DRIFT`,
/// `EXIT_USAGE`,
/// `RunRequest`,
/// `emit_errors` from `crate::run`.
use crate::run::{EXIT_CLEAN, EXIT_DRIFT, EXIT_USAGE, RunRequest, emit_errors};
/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// What:
///  Build the final `result` event with counts.
/// Why:
///  Every command ends with one summary line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resultEvent(command: string, clean: boolean, counts: Record<string, number>): Event
/// ```
pub fn result_event(command: &str, clean: bool, counts: &[(&str, usize)]) -> Event {
    let (code, message) = if command == "write" {
        ("write-done".to_string(), "write: every generated file is now up to date".to_string())
    } else if clean {
        (format!("{command}-clean"), format!("{command}: every generated file is up to date"))
    } else {
        (format!("{command}-drift"), format!("{command}: generated files differ from their rules"))
    };
    let mut event = Event::new("result", &code, &message);
    for (key, count) in counts {
        event.set_field(key, Value::from(*count as u64));
    }
    return event;
}

/// What:
///  Report drift for `check`,
/// writing the paired trees when asked.
/// Why:
///  Exit code 1 when anything drifted,
/// 0 otherwise;
/// a bad `--expected-dir` is a usage error (2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishCheck(...): number
/// ```
pub fn finish_check(request: &RunRequest, sink: &mut dyn EventSink, drifts: &[Drift], expected_dir: &Option<PathBuf>) -> i32 {
    let mut written: Vec<String> = Vec::new();
    if let Some(dir) = expected_dir {
        let checked = match validate_expected_dir(dir, &request.root) {
            Ok(canonical) => write_expected_trees(&canonical, drifts),
            Err(error) => Err(error),
        };
        match checked {
            Ok(paths) => written = paths,
            Err(error) => {
                emit_errors(sink, &[error]);
                return EXIT_USAGE;
            }
        }
    }
    for drift in drifts {
        let mut event = drift.to_event();
        if written.contains(&drift.path) {
            event.set_field("trees", Value::Bool(true));
        }
        sink.emit(event);
    }
    sink.emit(result_event("check", drifts.is_empty(), &[("drifted", drifts.len())]));
    if drifts.is_empty() {
        return EXIT_CLEAN;
    }
    return EXIT_DRIFT;
}

/// What:
///  Publish the drift for `write` and report what changed.
/// Why:
///  Exit code 0 after a successful publish,
/// 1 when publishing failed part way (the next run reconverges).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishWrite(...): number
/// ```
pub fn finish_write(request: &RunRequest, sink: &mut dyn EventSink, report: &PlanReport, drifts: &[Drift]) -> i32 {
    match publish(&request.root, report, drifts) {
        Ok(outcome) => {
            for path in &outcome.written {
                let mut event = Event::new("wrote", "file-written", &format!("wrote {path}"));
                event.set_field("path", Value::String(path.clone()));
                sink.emit(event);
            }
            let counts = [("written", outcome.written.len()), ("modes", outcome.modes.len()), ("removed", outcome.removed.len())];
            sink.emit(result_event("write", true, &counts));
            return EXIT_CLEAN;
        }
        Err(error) => {
            let mut failure = error.to_event("error");
            failure.hint = Some("the files published before this one stay; run `file-enforcer write` again to reconverge".to_string());
            sink.emit(failure);
            return EXIT_DRIFT;
        }
    }
}
