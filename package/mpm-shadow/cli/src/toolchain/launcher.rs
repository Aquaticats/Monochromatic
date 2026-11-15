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
//!
//! The record's write is one critical section under a lock of that folder's own,
//! and the lock on disk is asked in it which tool declares this bin name:
//! two installs can load different locks for one name,
//! and the older one can finish after the newer one wrote its record,
//! so a writer that finds the lock on disk naming another tool leaves the newer record in place
//! instead of writing its own over it.
//! Leaving it changes nothing that a person can see:
//! the launcher is a copy of the same `mpm-shadow` program either way,
//! and the record that stands names the tool the lock on disk declares.
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
use std::ffi::OsString;
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
use super::cargo_installs::{is_newer_than_locked, is_repository_published, older_lock_message_for};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports the check that follows a launcher chain and names a loop or a broken record.
use super::launcher_chain::chain_problem;
/// Imports what this module uses from `super::lock`.
use super::lock::{Lock, current_lock};
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Postinstall};
/// Imports what this module uses from `super::launcher_current_link`.
use super::launcher_current_link::{MissingProgram, locked_version, missing_program_advice, program_to_run, version_that_runs};
/// Imports what this module uses from `super::launcher_source`.
use super::launcher_source::{launcher_source, running_program};
/// Imports what this module uses from `super::repository_link_cause`.
use super::repository_link_cause::{CHECK_DEADLINE, LinkVerdict, repository_link_verdict};
/// Imports what this module uses from `super::scope`.
use super::scope::{Scope, ScopeKind};
/// Imports the one atomic write the launcher and the lock share.
use super::util::write_atomically;
/// Imports the one way this tool reads a file it did not just create.
use super::ordinary_file::{create_lock_file, read_ordinary_text, read_ordinary_text_through_links};

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
    fields.insert("lockFile".to_string(), Value::String(record.lock_file.to_string_lossy().to_string()));
    fields.insert("real".to_string(), Value::String(record.real.to_string_lossy().to_string()));
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

/// What:
/// A hard link of `source` at `target` (a copy when the two are on different file systems),
/// placed with a rename.
/// Why:
/// `source` may be a symbolic link:
/// a release archive's install leaves a link beside the folder it unpacked into,
/// and the search for the newest installed `mpm-shadow` finds the program through `current`.
/// A hard link of a symbolic link is another symbolic link,
/// and its relative target then resolves beside the launcher instead of beside the version directory,
/// so the launcher dangled and the tool it was made for stopped starting.
/// The file the link resolves to is what a launcher must be.
fn place_binary(source: &Path, target: &Path) -> Result<(), ToolchainError> {
    let mut temporary_name: OsString = OsString::from(".");
    temporary_name.push(target.file_name().unwrap_or_default());
    temporary_name.push(format!(".{}.bin", std::process::id()));
    let temporary: PathBuf = target.with_file_name(temporary_name);
    // A link on the way to the program is followed: the launcher must be the program's own file.
    let file: PathBuf = match fs::canonicalize(source) {
        Ok(resolved) => resolved,
        Err(_) => source.to_path_buf(),
    };
    if is_same_file(&file, target) {
        return Ok(());
    }
    // Best effort: a leftover from a killed run is replaced; absence is the normal case.
    let _ = fs::remove_file(&temporary);
    if fs::hard_link(&file, &temporary).is_err()
        && let Err(error) = fs::copy(&file, &temporary)
    {
        return Err(ToolchainError::failure(format!("cannot copy {} to {}: {error}", source.display(), temporary.display())));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        return Err(ToolchainError::failure(format!("cannot place {}: {error}", target.display())));
    }
    return Ok(());
}

/// What:
/// Makes the launcher for binary `name` of a repository-published tool in this tree and returns its path.
/// The record is written after the launcher,
/// so a visible record always has the launcher it describes.
/// Why:
/// The retirement asks that one question of a record: whose launcher is absent,
/// and may it go? An install that is still placing its program would answer it wrongly if the record came first,
/// and an install beside it must not be able to delete that fresh record
/// (a killed placement leaves the launcher without its record, which the next install of that name rewrites).
/// The tree's exposed links and postinstall links point here instead of at the real binary,
/// and the launcher is a copy of the newest installed `mpm-shadow`
/// (`launcher_source`),
/// never of whichever copy happened to run `install`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ensureLauncher(scope: Scope, tool: LockedTool, name: string, real: string): string
/// ```
pub fn ensure_launcher(scope: &Scope, tool_id: &str, name: &str, real: &Path) -> Result<PathBuf, ToolchainError> {
    // A launcher folder under a repository whose link to the 860 drive does not resolve is never made:
    // the plain message of the repository link check comes before any write.
    if let LinkVerdict::NotReady { message } = repository_link_verdict(&scope.home, &scope.launcher_dir, CHECK_DEADLINE) {
        return Err(ToolchainError::failure(message));
    }
    if let Err(error) = fs::create_dir_all(&scope.launcher_dir) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", scope.launcher_dir.display())));
    }
    let launcher: PathBuf = scope.launcher_dir.join(name);
    let record: LaunchRecord =
        LaunchRecord { tool_id: tool_id.to_string(), lock_file: scope.lock_file.clone(), real: real.to_path_buf(), scope: scope.kind };
    let source: PathBuf = launcher_source(scope, running_program()?, cfg!(target_os = "linux"))?;
    place_binary(&source, &launcher)?;
    // The seam a control uses to let a second install finish its launcher and its record here, while this one stands
    // between its program and its record. Compiled in tests only.
    #[cfg(test)]
    seam::after_the_place();
    // After the program, never before: the record is the proof a retirement may remove a launcher, so it must not name a
    // program that is not there yet.
    write_the_record_the_lock_asks_for(scope, name, tool_id, &launcher, &record)?;
    return Ok(launcher);
}

/// The file name of the lock that makes one folder's record writes one critical section.
/// One lock covers one folder's records:
/// that is the smallest unit a single lock file can name,
/// and a record's write is a few hundred bytes,
/// so an install beside this one is never made to wait for the launcher or link passes.
const RECORD_LOCK_NAME: &str = ".launch-records.lock";

/// How long a record writer waits for that lock before it refuses the write and says so.
/// The section is a read, a comparison and a write of a few hundred bytes,
/// so this bound has a hundredfold slack;
/// refusing is the safe side because the record that stands keeps the launcher provable.
const RECORD_LOCK_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// How long the wait sleeps between two tries.
const RECORD_LOCK_POLL: std::time::Duration = std::time::Duration::from_millis(10);

/// What:
/// Writes the `<name>.launch` record beside `launcher` when the lock on disk asks for it,
/// and returns without writing anything when the record already says what this run would write.
/// Why:
/// Two installs can hold different locks that assign one bin name to different tools,
/// and the older install can finish after the newer one wrote its record
/// (the seventh review reproduced exactly that, and the launcher then started the older tool).
/// So a write is made only while the folder's record lock is held,
/// and the lock on disk is asked inside it, at that instant, whether it assigns this bin name to a tool other than this
/// one: if it does, the newer record that stands is what the tree's own lock asks for, and this run leaves it alone.
/// The launcher is a copy of the same `mpm-shadow` program either way, so nothing a person sees is wrong.
/// A lock that cannot be read says nothing about another tool, and then this run writes what its own lock says.
/// A record that stands there naming another tool is left too, unless the lock on disk gives this name to this tool:
/// that record is what the last install made of the name.
/// A record that already says what this run would write is not written again, and no lock is taken for it:
/// an install with nothing to do replaces no file, which is what keeps it as cheap as it was.
/// A file at the record's name that is not a launcher record is left and named, never replaced.
fn write_the_record_the_lock_asks_for(
    scope: &Scope,
    name: &str,
    tool_id: &str,
    launcher: &Path,
    record: &LaunchRecord,
) -> Result<(), ToolchainError> {
    let path: PathBuf = record_path(launcher);
    let text: String = render_record(record);
    // The common case of an install with nothing to do: the record already says what this run would write, so nothing is
    // replaced and no lock is taken. A reader needs no lock; only a writer does.
    if the_record_at(&path)?.as_deref() == Some(text.as_str()) {
        return Ok(());
    }
    let lock_path: PathBuf = scope.launcher_dir.join(RECORD_LOCK_NAME);
    let _guard: fs::File = take_the_records_lock(&lock_path)?;
    // Under the lock the record is read again, because another install may have written it between the read above and
    // this instant, and the lock on disk is asked which tool it gives this name now.
    let standing: Option<String> = the_record_at(&path)?;
    let on_disk: Option<Lock> = current_lock(scope);
    if on_disk.as_ref().is_some_and(|lock| return another_tool_declares(lock, name, tool_id)) {
        return Ok(());
    }
    if standing.as_deref() == Some(text.as_str()) {
        return Ok(());
    }
    // A record naming another tool stands, and the lock on disk does not give this name to this tool: that record is what
    // the last install made of the name, and this one leaves it. A name the lock declares nowhere is retired by the next
    // pass with whatever record stands, so which of two installs of a dropped name wrote first changes nothing a person
    // can see.
    if let Some(standing_text) = &standing
        && parse_record(standing_text).is_some_and(|standing_record| return standing_record.tool_id != tool_id)
        && !on_disk.as_ref().is_some_and(|lock| return declares_launcher_for(lock, name, tool_id))
    {
        return Ok(());
    }
    if let Some(standing_text) = &standing
        && parse_record(standing_text).is_none()
    {
        return Err(ToolchainError::failure(format!(
            "{} stands where this launcher's record belongs and it is not a launcher record mpm-shadow wrote, so it was left \
             where it is and no record was written",
            path.display()
        )));
    }
    return write_atomically(&path, text.as_bytes());
}

/// What:
/// The text standing at the record's path,
/// `None` when nothing stands there,
/// and the sentence when something that is not an ordinary file does.
/// Why:
/// The record is a file this tool writes, so a read of it never waits and takes nothing but an ordinary file;
/// a file a person put there is left and named rather than replaced.
fn the_record_at(path: &Path) -> Result<Option<String>, ToolchainError> {
    return match read_ordinary_text(path) {
        Ok(text) => Ok(Some(text)),
        Err(_reason) if fs::symlink_metadata(path).is_err_and(|error| return error.kind() == std::io::ErrorKind::NotFound) => Ok(None),
        Err(reason) => Err(ToolchainError::failure(format!(
            "{reason}, so no launcher record was written and it was left where it is"
        ))),
    };
}

/// What:
/// The folder's record lock, taken with a bound,
/// or the sentence that says why the record was not written.
/// Why:
/// A writer that cannot take it must not write a record at all:
/// a record written without the lock could replace the newer record another install just wrote,
/// which is the defect this lock exists for.
fn take_the_records_lock(lock_path: &Path) -> Result<fs::File, ToolchainError> {
    let handle: fs::File = create_lock_file(lock_path).map_err(ToolchainError::failure)?;
    let deadline: std::time::Instant = std::time::Instant::now() + RECORD_LOCK_WAIT;
    loop {
        match handle.try_lock() {
            Ok(()) => return Ok(handle),
            Err(std::fs::TryLockError::WouldBlock) => {
                if std::time::Instant::now() >= deadline {
                    return Err(ToolchainError::failure(format!(
                        "no launcher record was written: another mpm-shadow holds {} for longer than {} seconds, and a record \
                         written without it could replace the record a newer install just wrote",
                        lock_path.display(),
                        RECORD_LOCK_WAIT.as_secs()
                    )));
                }
                std::thread::sleep(RECORD_LOCK_POLL);
            }
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(ToolchainError::failure(format!("cannot test {}: {error}", lock_path.display())));
            }
        }
    }
}

/// What:
/// The bin names a tool gets a launcher for:
/// its declared bins,
/// and the source bin of the postinstall link it declares.
/// Why:
/// `rebuild_exposure` asks for a launcher for each of them,
/// and the retirement counts the same names as the lock's own,
/// so both ask this one place what a launcher's name is.
pub(crate) fn launcher_names_of(tool: &LockedTool) -> Vec<String> {
    let mut names: Vec<String> = tool.declaration.bins.clone();
    if let Some(Postinstall::Link(link)) = &tool.declaration.options.postinstall {
        names.push(link.source_bin.clone());
    }
    return names;
}

/// What:
/// Whether the lock gives `name` a launcher for `tool_id`.
/// Why:
/// This and `another_tool_declares` are the two answers a record's write needs from the lock on disk:
/// the name is another tool's, so the record that stands stays;
/// or it is this tool's, so this run writes what the lock asks for.
pub(crate) fn declares_launcher_for(lock: &Lock, name: &str, tool_id: &str) -> bool {
    return lock.tools.iter().any(|tool| {
        return tool.declaration.id == tool_id && is_repository_published(tool) && launcher_names_of(tool).iter().any(|bin| return bin == name);
    });
}

/// What:
/// Whether the lock declares `name` a launcher for a tool other than `tool_id`.
/// Why:
/// This is the question a record's write asks the lock on disk at that instant:
/// a name the lock gives another tool belongs to that tool's record, whatever lock this run loaded.
pub(crate) fn another_tool_declares(lock: &Lock, name: &str, tool_id: &str) -> bool {
    return lock
        .tools
        .iter()
        .any(|tool| return tool.declaration.id != tool_id && is_repository_published(tool) && launcher_names_of(tool).iter().any(|bin| return bin == name));
}

/// The record beside the running executable,
/// or `None` when this process was not started as a launcher.
pub fn launch_record_of_current_exe() -> Option<LaunchRecord> {
    let executable: PathBuf = std::env::current_exe().ok()?;
    let text: String = read_ordinary_text(&record_path(&executable)).ok()?;
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
pub fn prepare_launch(record: &LaunchRecord, lock_text: Option<&str>) -> Launch {
    let mut warning: Option<String> = None;
    if let Some(used) = version_that_runs(&record.real)
        && let Some(text) = lock_text
        && let Some(locked) = locked_version(text, &record.tool_id)
        && is_newer_than_locked(&used, &locked)
    {
        warning = Some(older_lock_message_for(record.scope, &record.tool_id, &locked, &used, &record.lock_file));
    }
    let program: PathBuf = program_to_run(&record.real);
    if let Some(message) = &warning {
        warning = Some(warning_naming_program(message, &program));
    }
    return Launch { warning, program };
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
pub fn warning_naming_program(message: &str, program: &Path) -> String {
    return format!("{message}; starting {}", program.display());
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
/// Rewrites this launcher's record so that it names `program`,
/// when the record names another path and `program` exists.
/// Why:
/// The shadows a launcher starts (`cargo-shadow`,
/// `pnpm-shadow`,
/// `search-shadow`,
/// `mpm-shadow` in its shim mode) skip a `PATH` candidate
/// that is a launcher whose record leads back to themselves,
/// and they read the path the record names.
/// A record that names a deleted release folder leads nowhere they can resolve,
/// so the shadow took the launcher for the real tool,
/// started it,
/// and its recursion guard stopped the run.
/// Naming the program that ran keeps the record true after every version move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshStaleRecord(record: LaunchRecord, program: string): void  // throws
/// ```
fn refresh_stale_record(record: &LaunchRecord, program: &Path) -> Result<(), ToolchainError> {
    if program == record.real || !program.exists() {
        return Ok(());
    }
    let executable: PathBuf = match std::env::current_exe() {
        Ok(found) => found,
        Err(error) => return Err(ToolchainError::failure(format!("cannot find this launcher's own path: {error}"))),
    };
    let refreshed: LaunchRecord = LaunchRecord { real: program.to_path_buf(), ..record.clone() };
    return write_atomically(&record_path(&executable), render_record(&refreshed).as_bytes());
}

/// What:
/// Says on standard error why an unreadable lock file is unreadable when the cause is the repository's link to the 860 drive.
/// Why:
/// The launcher never blocks a run for an unreadable lock,
/// so without this the version warning is dropped in silence.
/// Nothing is asked unless the identity file exists and the lock file is under the repository it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayWhyTheLockIsUnreadable(lockFile)
/// ```
fn say_why_the_lock_is_unreadable(lock_file: &Path) {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    match repository_link_verdict(Path::new(&home), lock_file, CHECK_DEADLINE) {
        LinkVerdict::NotReady { message } => {
            eprintln!("mpm-shadow: warning: {} cannot be read, so the version check was skipped and the tool runs anyway: {message}", lock_file.display());
        }
        LinkVerdict::CheckUnavailable { why } => {
            eprintln!("mpm-shadow: warning: {} cannot be read, so the version check was skipped and the tool runs anyway; the cause is unknown: {why}", lock_file.display());
        }
        LinkVerdict::NotConfigured | LinkVerdict::Ready => {}
    }
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
    let lock_text: Option<String> = read_ordinary_text_through_links(&record.lock_file).ok();
    if lock_text.is_none() {
        say_why_the_lock_is_unreadable(&record.lock_file);
    }
    let launch: Launch = prepare_launch(record, lock_text.as_deref());
    // A launcher starts only a path it has just found to be a file: neither the recorded path nor the one under `current` is
    // tried blind, and no warning says "starting" for a program that does not start.
    if !launch.program.is_file() {
        return report_program_not_found(record, &launch.program, lock_text.as_deref());
    }
    if let Some(warning) = &launch.warning {
        eprintln!("mpm-shadow: warning: {warning}");
    }
    // A launcher never starts itself: a chain of records that leads back to this launcher, or in a circle, is refused before anything is started.
    if let Ok(own) = std::env::current_exe()
        && let Some(problem) = chain_problem(&own, &launch.program)
    {
        eprintln!("mpm-shadow: {problem}");
        return EXEC_FAILED_EXIT_CODE;
    }
    if let Err(error) = refresh_stale_record(record, &launch.program) {
        eprintln!("mpm-shadow: warning: could not update the launcher record, so it keeps naming a program that no longer exists: {error}");
    }
    let error: std::io::Error = exec_program(&launch.program, arguments);
    eprintln!("mpm-shadow: could not start {}: {error}", launch.program.display());
    if error.kind() == std::io::ErrorKind::NotFound {
        // The file vanished between the check and the start (another install deleted it): the same advice applies.
        eprintln!("mpm-shadow: {}", advice_for(record, lock_text.as_deref()));
        return NOT_FOUND_EXIT_CODE;
    }
    return EXEC_FAILED_EXIT_CODE;
}

/// The sentence that says no program was found for `record`,
/// with the tool,
/// both versions and the command that makes the links again.
fn advice_for(record: &LaunchRecord, lock_text: Option<&str>) -> String {
    return missing_program_advice(MissingProgram {
        tool_id: &record.tool_id,
        scope: record.scope,
        lock_file: &record.lock_file,
        real: &record.real,
        lock_text,
    });
}

/// What:
/// Says on standard error that `program` was not started because no file exists there,
/// followed by the advice,
/// and returns the exit status for a missing program.
/// Why:
/// The check comes before any start,
/// so the message names the path that was checked,
/// and the start attempt's operating-system error is not the only evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportProgramNotFound(record: LaunchRecord, program: string, lockText?: string): number
/// ```
fn report_program_not_found(record: &LaunchRecord, program: &Path, lock_text: Option<&str>) -> i32 {
    eprintln!("mpm-shadow: not starting {}: no file exists there", program.display());
    eprintln!("mpm-shadow: {}", advice_for(record, lock_text));
    return NOT_FOUND_EXIT_CODE;
}

/// The seam a control uses to let a second install finish while this one stands between its program and its record.
/// Compiled in tests only, so no shipped path can be moved by anything but the file system.
#[cfg(test)]
pub(crate) mod seam {
    /// Imports `Mutex` from `std`.
    use std::sync::Mutex;

    /// The hook a control sets to run after the program is placed and before the record is written.
    static AFTER_THE_PLACE: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);

    /// Sets (or with `None` clears) the hook run after the program is placed.
    pub(crate) fn set_after_the_place(hook: Option<Box<dyn Fn() + Send>>) {
        *AFTER_THE_PLACE.lock().expect("the seam is not poisoned") = hook;
    }

    /// Runs the hook once, when there is one.
    pub(crate) fn after_the_place() {
        let waiting: Option<Box<dyn Fn() + Send>> = AFTER_THE_PLACE.lock().expect("the seam is not poisoned").take();
        if let Some(hook) = waiting {
            hook();
        }
    }
}
