//! What:
//! The launcher a tree's exposed link goes through for a repository-published tool:
//! a per-tree hard link (or copy) of the `mpm-shadow` binary named after the tool,
//! plus a `<name>.launch` record beside it.
//! Started under that path it reads the record and the tree's lock,
//! prints a warning on standard error when the tool that will run is newer than the lock pins,
//! then replaces itself with the real binary
//! (the one its record names,
//! or the one under the tool's `current` link when that file was deleted by a newer install),
//! passing argv,
//! environment and standard streams through.
//! Why:
//! A bare symlink to the binary has nowhere to check the tree's lock.
//! The kernel resolves a symlink chain to its final file,
//! so a hard link per tree is what tells `/proc/self/exe` which tree's link was used.
//! Replacing the process (instead of waiting for a child) makes exit codes,
//! signals and standard output the real binary's own.
//! Making the launcher file and starting it are files of their own,
//! named for what they hold:
//! the hard link and its record (`making_the_launcher`)
//! and the run (`starting_it`).
//! This file keeps the record's shape,
//! the decision what to run and the sentence that decision carries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // launcher.ts
//! export function recordPath(launcher: string): string
//! export function renderRecord(record: LaunchRecord): string
//! export function parseRecord(text: string): LaunchRecord | undefined
//! export function launchRecordOfCurrentExe(): LaunchRecord | undefined
//! export function prepareLaunch(record: LaunchRecord, lockText?: string): Launch
//! export * from './launcher/making_the_launcher.ts';
//! export * from './launcher/starting_it.ts';
//! ```

/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `Value` from `serde_json`.
use serde_json::{
    Map,
    Value,
};

/// Imports what this module uses from `super::cargo_installs`.
use super::cargo_installs::{
    is_newer_than_locked,
    older_lock_message_for,
};
/// Imports what this module uses from `super::launcher_current_link`.
use super::launcher_current_link::{
    locked_version,
    program_to_run,
    version_that_runs,
};
/// Imports `ScopeKind` from `super::scope`.
use super::scope::ScopeKind;

/// The launcher file:
/// the hard link or copy of the newest installed `mpm-shadow`,
/// and its record.
mod making_the_launcher;
/// Running a launcher:
/// the program check,
/// the warning,
/// the chain refusal and the exec.
mod starting_it;

/// Re-exports making the launcher for a binary of a repository-published tool.
pub use self::making_the_launcher::ensure_launcher;
/// Re-exports running a launcher and the sentence that says why a program was not found.
pub use self::starting_it::run_launcher;

/// The suffix of the record beside a launcher.
pub const RECORD_SUFFIX: &str = ".launch";

/// What a launcher needs to know,
/// written at `install` time:
/// the tool's id in the lock,
/// the lock file to compare against,
/// and the real binary (through the crate's `current` link).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchRecord {
    /// The tool's id in the lock.
    pub tool_id: String,
    /// The tree's `toolchain.lock.json`.
    pub lock_file: PathBuf,
    /// The binary to run:
    /// `<crate directory>/current/bin/<name>`.
    pub real: PathBuf,
    /// Which scope made the launcher,
    /// so the warning names the upgrade command of that scope's lock.
    pub scope: ScopeKind,
}

/// The record file beside a launcher.
pub fn record_path(launcher: &Path) -> PathBuf {
    let mut name: OsString = launcher.as_os_str().to_os_string();
    name.push(RECORD_SUFFIX);
    return PathBuf::from(name);
}

/// The record as JSON text.
pub fn render_record(record: &LaunchRecord) -> String {
    // Built field by field: the `json!` macro unwraps a conversion that clippy.toml disallows.
    let mut fields: Map<String, Value> = Map::new();
    fields.insert("toolId".to_string(), Value::String(record.tool_id.clone()));
    fields.insert(
        "lockFile".to_string(),
        Value::String(record.lock_file.to_string_lossy().to_string()),
    );
    fields.insert(
        "real".to_string(),
        Value::String(record.real.to_string_lossy().to_string()),
    );
    let scope: &str = match record.scope {
        ScopeKind::Repo => "repo",
        ScopeKind::Global => "global",
    };
    fields.insert("scope".to_string(), Value::String(scope.to_string()));
    return Value::Object(fields).to_string();
}

/// The record in `text`,
/// or `None` when it is not one.
pub fn parse_record(text: &str) -> Option<LaunchRecord> {
    let value: Value = serde_json::from_str(text).ok()?;
    // A record written before the field existed belongs to a tree.
    let scope: ScopeKind = match value.get("scope").and_then(Value::as_str) {
        Some("global") => ScopeKind::Global,
        _ => ScopeKind::Repo,
    };
    return Some(LaunchRecord {
        tool_id: value.get("toolId")?.as_str()?.to_string(),
        lock_file: PathBuf::from(value.get("lockFile")?.as_str()?),
        real: PathBuf::from(value.get("real")?.as_str()?),
        scope,
    });
}

/// The record beside the running executable,
/// or `None` when this process was not started as a launcher.
pub fn launch_record_of_current_exe() -> Option<LaunchRecord> {
    let executable: PathBuf = std::env::current_exe().ok()?;
    let text: String = fs::read_to_string(record_path(&executable)).ok()?;
    return parse_record(&text);
}

/// What a launcher decided:
/// the warning to print (if any) and the program to run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Launch {
    /// The sentence to print on standard error before running,
    /// when the tool that runs is newer than the lock pins.
    pub warning: Option<String>,
    /// The binary to run.
    pub program: PathBuf,
}

/// What:
/// Compares the version that runs (the target of the crate's `current` link) with the lock's and says what to print.
/// Why:
/// Pure over the record and the lock's text,
/// so tests cover each case without starting a program;
/// an unreadable lock or link means no warning,
/// never a refusal to run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function prepareLaunch(record: LaunchRecord, lockText?: string): Launch
/// ```
pub fn prepare_launch(
    record: &LaunchRecord,
    lock_text: Option<&str>,
) -> Launch {
    let mut warning: Option<String> = None;
    if let Some(used) = version_that_runs(&record.real)
        && let Some(text) = lock_text
        && let Some(locked) = locked_version(text, &record.tool_id)
        && is_newer_than_locked(&used, &locked)
    {
        warning = Some(older_lock_message_for(
            record.scope,
            &record.tool_id,
            &locked,
            &used,
            &record.lock_file,
        ));
    }
    let program: PathBuf = program_to_run(&record.real);
    if let Some(message) = &warning {
        warning = Some(warning_naming_program(message, &program));
    }
    return Launch {
        warning,
        program,
    };
}

/// What:
/// The older-lock sentence followed by the program the launcher is about to start.
/// Why:
/// The sentence names the version that runs,
/// and a person who finds the program by path must be able to tell that this is the file started,
/// not the path a stale record once named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function warningNamingProgram(message: string, program: string): string
/// ```
pub fn warning_naming_program(
    message: &str,
    program: &Path,
) -> String {
    return format!("{message}; starting {}", program.display());
}
