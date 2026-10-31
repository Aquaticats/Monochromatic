//! What:
//!  Execute one command (`write`, `check` or `explain`) against a worktree and report through an event sink.
//! Why:
//!  The server calls this under its write lock and the in-process path calls it directly, so both behave the same;
//!  the exit code is decided here: 0 clean, 1 drift or rule failure, 2 bad config.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function run(request: RunRequest, registry: RuleRegistry, sink: EventSink): number
//! ```

use crate::config_decode::{ConfigScope, decode_config};
use crate::config_load::load_config_file;
use crate::diagnostic::Diagnostic;
use crate::drift::{Drift, compute_drift};
use crate::event::{Event, EventSink};
use crate::expected_dir::{validate_expected_dir, write_expected_trees};
use crate::explain::{explain, normalize_target};
use crate::plan::{PlanReport, plan_run};
use crate::publish::publish;
use crate::registry::RuleRegistry;
use crate::rule::Rule;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Exit code for a clean run.
pub const EXIT_CLEAN: i32 = 0;
/// Exit code for drift or a rule failure.
pub const EXIT_DRIFT: i32 = 1;
/// Exit code for a bad command line or config.
pub const EXIT_USAGE: i32 = 2;

/// How many times a write replans after a read file changed before it gives up.
const REPLAN_LIMIT: usize = 3;

/// What:
///  Which command to run.
/// Why:
///  One request type serves the server protocol and the in-process path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunMode = { kind: 'write' } | { kind: 'check'; expectedDir?: string } | { kind: 'explain'; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunMode {
    /// Regenerate every drifted file.
    Write,
    /// Report drift; optionally write the paired trees.
    Check {
        /// Directory for `actual/` and `expected/` trees.
        expected_dir: Option<PathBuf>,
    },
    /// Say which rules read or write a path.
    Explain {
        /// The path as typed.
        path: String,
    },
}

/// What:
///  Everything a run needs: the worktree root, the config file, its scope and the mode.
/// Why:
///  Each server request carries its canonical config path and options; the config is re-read every run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunRequest = { root: string; configPath: string; scope: ConfigScope; mode: RunMode };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRequest {
    /// Worktree root (or the home directory for per-user runs).
    pub root: PathBuf,
    /// Config file path.
    pub config_path: PathBuf,
    /// Repository or per-user config.
    pub scope: ConfigScope,
    /// The command.
    pub mode: RunMode,
}

/// What:
///  Emit a list of diagnostics as `error` events.
/// Why:
///  Config and rule problems share one shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emitErrors(sink: EventSink, errors: Diagnostic[]): void
/// ```
fn emit_errors(sink: &mut dyn EventSink, errors: &[Diagnostic]) {
    for error in errors {
        sink.emit(error.to_event("error"));
    }
}

/// What:
///  Load, decode and build the rules of a config; `Err` carries the exit code and the diagnostics already emitted.
/// Why:
///  Any failure here is a bad config: exit code 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadRules(request: RunRequest, registry: RuleRegistry, sink: EventSink): Rule[] | number
/// ```
pub fn load_rules(request: &RunRequest, registry: &RuleRegistry, sink: &mut dyn EventSink) -> Result<Vec<Box<dyn Rule>>, i32> {
    let value = match load_config_file(&request.config_path) {
        Ok(value) => value,
        Err(error) => {
            emit_errors(sink, &[error]);
            return Err(EXIT_USAGE);
        }
    };
    let display = request.config_path.display().to_string();
    let config = match decode_config(&value, request.scope, &display) {
        Ok(config) => config,
        Err(errors) => {
            emit_errors(sink, &errors);
            return Err(EXIT_USAGE);
        }
    };
    match registry.build(&config) {
        Ok(rules) => return Ok(rules),
        Err(errors) => {
            emit_errors(sink, &errors);
            return Err(EXIT_USAGE);
        }
    }
}

/// What:
///  Emit a plan's warnings as `warning` events.
/// Why:
///  A kind that is not wired is a warning, never drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emitWarnings(sink: EventSink, report: PlanReport): void
/// ```
fn emit_warnings(sink: &mut dyn EventSink, report: &PlanReport) {
    for warning in &report.warnings {
        sink.emit(warning.to_event("warning"));
    }
}

/// What:
///  Build the final `result` event with counts.
/// Why:
///  Every command ends with one summary line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resultEvent(command: string, clean: boolean, counts: Record<string, number>): Event
/// ```
fn result_event(command: &str, clean: bool, counts: &[(&str, usize)]) -> Event {
    let (code, message) = if clean {
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
///  Plan the run and compute its drift, replanning while a file the plan read changes under it.
/// Why:
///  Snapshot revalidation: a plan built on files that changed since is discarded and redone, so `write` never
///  publishes from stale reads. The race with writers that do not use the server is documented, not denied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planAndDiff(root: string, rules: Rule[], revalidate: boolean): [PlanReport, Drift[]]
/// ```
fn plan_and_diff(root: &Path, rules: &[Box<dyn Rule>], revalidate: bool) -> Result<(PlanReport, Vec<Drift>), Diagnostic> {
    let mut attempts = 0;
    loop {
        let mut report = plan_run(root, rules);
        attempts += 1;
        if !report.errors.is_empty() {
            return Ok((report, Vec::new()));
        }
        let drifts = compute_drift(&mut report)?;
        if !revalidate {
            return Ok((report, drifts));
        }
        let changed = report.view.changed_paths()?;
        if changed.is_empty() {
            return Ok((report, drifts));
        }
        if attempts >= REPLAN_LIMIT {
            let message = format!("{} changed while the plan was being built, {attempts} times in a row", changed.join(", "));
            return Err(Diagnostic::new("snapshot-changed", &message).with_hint("stop the other writer and run again"));
        }
    }
}

/// What:
///  Run one request and return the exit code.
/// Why:
///  The single entry point under both the server and the in-process path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(request: RunRequest, registry: RuleRegistry, sink: EventSink): number
/// ```
pub fn run(request: &RunRequest, registry: &RuleRegistry, sink: &mut dyn EventSink) -> i32 {
    let rules = match load_rules(request, registry, sink) {
        Ok(rules) => rules,
        Err(code) => return code,
    };
    let is_write = request.mode == RunMode::Write;
    let (report, drifts) = match plan_and_diff(&request.root, &rules, is_write) {
        Ok(pair) => pair,
        Err(error) => {
            emit_errors(sink, &[error]);
            return EXIT_DRIFT;
        }
    };
    emit_warnings(sink, &report);
    if !report.errors.is_empty() {
        emit_errors(sink, &report.errors);
        return EXIT_DRIFT;
    }
    if let RunMode::Explain { path } = &request.mode {
        for event in explain(&report, &normalize_target(&request.root, path)) {
            sink.emit(event);
        }
        return EXIT_CLEAN;
    }
    match &request.mode {
        RunMode::Check { expected_dir } => return finish_check(request, sink, &drifts, expected_dir),
        _ => return finish_write(request, sink, &report, &drifts),
    }
}

/// What:
///  Report drift for `check`, writing the paired trees when asked.
/// Why:
///  Exit code 1 when anything drifted, 0 otherwise; a bad `--expected-dir` is a usage error (2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishCheck(...): number
/// ```
fn finish_check(request: &RunRequest, sink: &mut dyn EventSink, drifts: &[Drift], expected_dir: &Option<PathBuf>) -> i32 {
    let mut written: Vec<String> = Vec::new();
    if let Some(dir) = expected_dir {
        let checked = validate_expected_dir(dir, &request.root).and_then(|canonical| return write_expected_trees(&canonical, drifts));
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
///  Exit code 0 after a successful publish, 1 when publishing failed part way (the next run reconverges).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishWrite(...): number
/// ```
fn finish_write(request: &RunRequest, sink: &mut dyn EventSink, report: &PlanReport, drifts: &[Drift]) -> i32 {
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
