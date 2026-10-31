//! What:
//! The launcher a tree's exposed link goes through for a repository-published tool:
//! a per-tree hard link (or copy) of the `mpm-shadow` binary named after the tool,
//! plus a `<name>.launch` record beside it.
//! Started under that path it reads the record and the tree's lock,
//! prints a warning on standard error when the tool that will run is newer than the lock pins,
//! then replaces itself with the real binary,
//! passing argv,
//! environment and standard streams through.
//! Why:
//! A bare symlink to the binary has nowhere to check the tree's lock.
//! The kernel resolves a symlink chain to its final file,
//! so a hard link per tree is what tells `/proc/self/exe` which tree's link was used.
//! Replacing the process (instead of waiting for a child) makes exit codes,
//! signals and standard output the real binary's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // launcher.ts
//! export function ensureLauncher(scope: Scope, tool: LockedTool, name: string, real: string): string
//! export function launchRecordOfCurrentExe(): LaunchRecord | undefined
//! export function prepareLaunch(record: LaunchRecord, lockText?: string): Launch
//! export function runLauncher(record: LaunchRecord, args: string[]): number
//! ```

/// Imports `OsString` from `std::ffi`.
use std::ffi::{OsStr, OsString};
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};
/// Imports `Command` from `std::process`.
use std::process::Command;

/// Imports `Value` from `serde_json`.
use serde_json::{Map, Value};

/// Imports the exit codes the shadow already uses.
use crate::constants::{EXEC_FAILED_EXIT_CODE, NOT_FOUND_EXIT_CODE};

/// Imports what this module uses from `super::cargo_installs`.
use super::cargo_installs::{CURRENT_LINK_NAME, is_newer_than_locked, older_lock_message};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

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
    fields.insert("lockFile".to_string(), Value::String(record.lock_file.to_string_lossy().to_string()));
    fields.insert("real".to_string(), Value::String(record.real.to_string_lossy().to_string()));
    return Value::Object(fields).to_string();
}

/// The record in `text`,
/// or `None` when it is not one.
pub fn parse_record(text: &str) -> Option<LaunchRecord> {
    let value: Value = serde_json::from_str(text).ok()?;
    return Some(LaunchRecord {
        tool_id: value.get("toolId")?.as_str()?.to_string(),
        lock_file: PathBuf::from(value.get("lockFile")?.as_str()?),
        real: PathBuf::from(value.get("real")?.as_str()?),
    });
}

/// Writes `bytes` to `target` through a temporary file and a rename,
/// so a reader sees the old file or the new one.
fn write_atomically(target: &Path, bytes: &[u8]) -> Result<(), ToolchainError> {
    let mut temporary_name: OsString = OsString::from(".");
    temporary_name.push(target.file_name().unwrap_or_default());
    temporary_name.push(format!(".{}.tmp", std::process::id()));
    let temporary: PathBuf = target.with_file_name(temporary_name);
    if let Err(error) = fs::write(&temporary, bytes) {
        return Err(ToolchainError::failure(format!("cannot write {}: {error}", temporary.display())));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        return Err(ToolchainError::failure(format!("cannot place {}: {error}", target.display())));
    }
    return Ok(());
}

/// Whether both paths name the same file (the same device and inode),
/// which a rename between two links of one file would silently leave as two names.
#[cfg(unix)]
fn is_same_file(first: &Path, second: &Path) -> bool {
    /// Imports `MetadataExt` from `std::os::unix::fs`.
    use std::os::unix::fs::MetadataExt;
    return match (fs::metadata(first), fs::metadata(second)) {
        (Ok(left), Ok(right)) => left.dev() == right.dev() && left.ino() == right.ino(),
        _ => false,
    };
}

/// Without inodes every placement replaces the target.
#[cfg(not(unix))]
fn is_same_file(_first: &Path, _second: &Path) -> bool {
    return false;
}

/// A hard link of `source` at `target` (a copy when the two are on different file systems),
/// placed with a rename.
fn place_binary(source: &Path, target: &Path) -> Result<(), ToolchainError> {
    if is_same_file(source, target) {
        return Ok(());
    }
    let mut temporary_name: OsString = OsString::from(".");
    temporary_name.push(target.file_name().unwrap_or_default());
    temporary_name.push(format!(".{}.bin", std::process::id()));
    let temporary: PathBuf = target.with_file_name(temporary_name);
    // Best effort: a leftover from a killed run is replaced; absence is the normal case.
    let _ = fs::remove_file(&temporary);
    if fs::hard_link(source, &temporary).is_err()
        && let Err(error) = fs::copy(source, &temporary)
    {
        return Err(ToolchainError::failure(format!("cannot copy {} to {}: {error}", source.display(), temporary.display())));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        return Err(ToolchainError::failure(format!("cannot place {}: {error}", target.display())));
    }
    return Ok(());
}

/// What:
/// The file to make a launcher from:
/// the running executable's path,
/// or `/proc/self/exe` when that path no longer names a file.
/// Why:
/// An `mpm-shadow` that installs a newer `mpm-shadow` deletes its own version directory (older repository tools are never
/// kept) before it rebuilds the exposure directory,
/// and the kernel then reports the running executable as `<path> (deleted)`;
/// copying through `/proc/self/exe` still reads the open file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runningBinary(): string
/// ```
fn running_binary() -> Result<PathBuf, ToolchainError> {
    let path: PathBuf = match std::env::current_exe() {
        Ok(found) => found,
        Err(error) => return Err(ToolchainError::failure(format!("cannot find the running mpm-shadow to make a launcher: {error}"))),
    };
    return Ok(usable_source(path));
}

/// `path` when it names a file,
/// otherwise the running process's own executable through `/proc`.
pub fn usable_source(path: PathBuf) -> PathBuf {
    if path.exists() {
        return path;
    }
    return PathBuf::from("/proc/self/exe");
}

/// What:
/// Makes the launcher for binary `name` of a repository-published tool in this tree and returns its path.
/// The record is written first,
/// so a launcher never exists without one.
/// Why:
/// The tree's exposed links and postinstall links point here instead of at the real binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ensureLauncher(scope: Scope, tool: LockedTool, name: string, real: string): string
/// ```
pub fn ensure_launcher(scope: &Scope, tool_id: &str, name: &str, real: &Path) -> Result<PathBuf, ToolchainError> {
    if let Err(error) = fs::create_dir_all(&scope.launcher_dir) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", scope.launcher_dir.display())));
    }
    let launcher: PathBuf = scope.launcher_dir.join(name);
    let record: LaunchRecord = LaunchRecord { tool_id: tool_id.to_string(), lock_file: scope.lock_file.clone(), real: real.to_path_buf() };
    write_atomically(&record_path(&launcher), render_record(&record).as_bytes())?;
    let source: PathBuf = running_binary()?;
    place_binary(&source, &launcher)?;
    return Ok(launcher);
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

/// The version the lock pins for the tool,
/// read from the lock's text.
fn locked_version(lock_text: &str, tool_id: &str) -> Option<String> {
    let lock: Value = serde_json::from_str(lock_text).ok()?;
    return lock.get("tools")?.get(tool_id)?.get("version")?.as_str().map(str::to_string);
}

/// Whether a path is named `current`,
/// the link of a repository-published tool.
fn is_current_link(candidate: &&Path) -> bool {
    return candidate.file_name() == Some(OsStr::new(CURRENT_LINK_NAME));
}

/// The version the tool's `current` link names,
/// found from the real binary's path:
/// `<crate directory>/current/bin/<name>` for a crate,
/// `<tool directory>/current/<archive folder>/<name>` for a release archive,
/// so the link is the nearest ancestor of the binary that is named `current`.
fn version_that_runs(real: &Path) -> Option<String> {
    let link: &Path = real.ancestors().skip(1).find(is_current_link)?;
    let target: PathBuf = fs::read_link(link).ok()?;
    let name: String = file_name_text(&target);
    if name.is_empty() {
        return None;
    }
    return Some(name);
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
pub fn prepare_launch(record: &LaunchRecord, lock_text: Option<&str>) -> Launch {
    let mut warning: Option<String> = None;
    if let Some(used) = version_that_runs(&record.real)
        && let Some(text) = lock_text
        && let Some(locked) = locked_version(text, &record.tool_id)
        && is_newer_than_locked(&used, &locked)
    {
        warning = Some(older_lock_message(&record.tool_id, &locked, &used));
    }
    return Launch { warning, program: record.real.clone() };
}

/// Replaces this process with `program`,
/// keeping `arguments` (argv[0] included) as they were;
/// returns only when that failed.
#[cfg(unix)]
fn exec_program(program: &Path, arguments: &[OsString]) -> std::io::Error {
    /// Imports `CommandExt` from `std::os::unix::process`.
    use std::os::unix::process::CommandExt;
    let mut command: Command = Command::new(program);
    if let Some(argv0) = arguments.first() {
        command.arg0(argv0);
    }
    command.args(arguments.iter().skip(1));
    return command.exec();
}

/// Runs `program` and waits (a platform without `exec`),
/// exiting the way it did.
#[cfg(not(unix))]
fn exec_program(program: &Path, arguments: &[OsString]) -> std::io::Error {
    return match Command::new(program).args(arguments.iter().skip(1)).status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => error,
    };
}

/// What:
/// Runs a launcher:
/// warns when the lock pins an older version than the one that runs,
/// then becomes the real binary.
/// Why:
/// Standard output is never written here,
/// since the wrapped tool's own output (the git wrapper prints JSON lines) must reach the caller unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runLauncher(record: LaunchRecord, args: string[]): number
/// ```
pub fn run_launcher(record: &LaunchRecord, arguments: &[OsString]) -> i32 {
    let lock_text: Option<String> = fs::read_to_string(&record.lock_file).ok();
    let launch: Launch = prepare_launch(record, lock_text.as_deref());
    if let Some(warning) = &launch.warning {
        eprintln!("mpm-shadow: warning: {warning}");
    }
    let error: std::io::Error = exec_program(&launch.program, arguments);
    eprintln!("mpm-shadow: could not start {}: {error}", launch.program.display());
    if error.kind() == std::io::ErrorKind::NotFound {
        return NOT_FOUND_EXIT_CODE;
    }
    return EXEC_FAILED_EXIT_CODE;
}
