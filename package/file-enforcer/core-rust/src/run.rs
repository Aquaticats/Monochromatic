//! What:
//!  Execute one command (`write`,
//! `check` or `explain`) against a worktree and report through an event sink.
//! Why:
//!  The server calls this under its write lock and the in-process path calls it directly,
//! so both behave the same;
//!  the exit code is decided here:
//! 0 clean,
//! 1 drift or rule failure,
//! 2 bad config.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function run(request: RunRequest, registry: RuleRegistry, sink: EventSink): number
//! ```

/// Imports `ConfigScope`,
/// `decode_config` from `crate::config_decode`.
use crate::config_decode::{ConfigScope, decode_config};
/// Imports `load_config_file` from `crate::config_load`.
use crate::config_load::load_config_file;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `Drift`,
/// `compute_drift` from `crate::drift`.
use crate::drift::{Drift, compute_drift};
/// Imports `EventSink` from `crate::event`.
use crate::event::EventSink;
/// Imports `explain`,
/// `normalize_target` from `crate::explain`.
use crate::explain::{explain, normalize_target};
/// Imports `PlanReport`,
/// `plan_run` from `crate::plan`.
use crate::plan::{PlanReport, plan_run};
/// Imports `RuleRegistry` from `crate::registry`.
use crate::registry::RuleRegistry;
/// Imports `Rule` from `crate::rule`.
use crate::rule::Rule;
/// Imports `finish_check`,
/// `finish_write` from `crate::run_finish`.
use crate::run_finish::{finish_check, finish_write};
/// Imports `Path`,
/// `PathBuf` from `std::path`.
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
    /// Report drift;
    /// optionally write the paired trees.
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
///  Everything a run needs:
/// the worktree root,
/// the config file,
/// its scope and the mode.
/// Why:
///  Each server request carries its canonical config path and options;
/// the config is re-read every run.
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
pub fn emit_errors(sink: &mut dyn EventSink, errors: &[Diagnostic]) {
    for error in errors {
        sink.emit(error.to_event("error"));
    }
}

/// What:
///  Load,
/// decode and build the rules of a config;
/// `Err` carries the exit code and the diagnostics already emitted.
/// Why:
///  Any failure here is a bad config:
/// exit code 2.
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
///  A kind that is not wired is a warning,
/// never drift.
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
///  Plan the run and compute its drift,
/// replanning while a file the plan read changes under it.
/// Why:
///  Snapshot revalidation:
/// a plan built on files that changed since is discarded and redone,
/// so `write` never
///  publishes from stale reads.
/// The race with writers that do not use the server is documented,
/// not denied.
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
    let mut paths = WatchPaths::default();
    return run_detailed(request, registry, sink, &mut paths);
}

/// What:
///  The repo-relative paths a run depended on:
/// files read or planned,
/// and directories listed.
/// Why:
///  `watch` watches exactly these,
/// so its set cannot be narrower than what the run read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WatchPaths = { files: string[]; directories: string[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WatchPaths {
    /// Files read,
    /// planned or observed (absent ones included).
    pub files: Vec<String>,
    /// Directories listed (absent ones included).
    pub directories: Vec<String>,
}

/// What:
///  Run one request,
/// also reporting the paths the run depended on.
/// Why:
///  The server's `watch` calls this to learn what to watch after each cycle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runDetailed(request: RunRequest, registry: RuleRegistry, sink: EventSink, paths: WatchPaths): number
/// ```
pub fn run_detailed(request: &RunRequest, registry: &RuleRegistry, sink: &mut dyn EventSink, paths: &mut WatchPaths) -> i32 {
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
    let (read_files, read_directories) = report.view.read_paths();
    paths.files = read_files;
    paths.directories = read_directories;
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
