//! What:
//!  The toolchain repair:
//! when the launchers copied into a new worktree's `.cache/toolchain-launchers` still name another
//!  checkout's `toolchain.lock.json`,
//! rebuild the worktree's own links and launchers with `mpm-shadow install
//!  --frozen-lockfile`.
//! Why:
//!  mpm-shadow owns `toolchain.plan.jsonc`,
//! `toolchain.lock.json` and the pinned tools.
//! Its exposed links and launcher records
//!  hold absolute paths,
//! so a copy would run the source checkout's tool versions and guard,
//! and a branch that changes the
//!  lock would silently use the source's.
//! The install is lock-constrained:
//! it relinks what is installed and,
//! when this
//!  worktree's lock names tools that are not installed,
//! downloads them.
//! The owner chose that on 2026-11-03 ("Install online
//!  during the add"),
//! knowing an add can then download and can retire tool copies another checkout still links.
//! The summary
//!  line mpm-shadow prints says what was installed.
//! A failed install never fails the add:
//! it prints the reason and
//!  `mise run toolchain:install`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairToolchain(destination, environment); // undefined when the launchers already name this worktree
//! ```

use crate::diagnostic_log::debug;
use crate::package_repair_request::RepairEnvironment;
use crate::package_repair_command::{ToolCommand, ToolLimits, ToolRun, find_program, run_refused, run_tool_awaiting};
use crate::package_repair_report::{Finding, Outcome, Tool, finding_from, missing_tool};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairToolchain";

/// Where mpm-shadow keeps the launchers and their records,
/// relative to a worktree.
const LAUNCHERS: &str = ".cache/toolchain-launchers";

/// The record that sits beside a launcher,
/// named after it.
const RECORD_EXTENSION: &str = "launch";

/// The lock file a launcher record names,
/// relative to a worktree.
const LOCK_FILE: &str = "toolchain.lock.json";

/// The key of a launcher record that names the lock file.
const LOCK_KEY: &str = "lockFile";

/// The program that owns the toolchain,
/// and where a worktree keeps a link to it.
const PROGRAM: &str = "mpm-shadow";

/// Where the worktree's own link to the program sits when the search path does not provide one.
const OWN_LINK: &str = ".cache/toolchain-bin/mpm-shadow";

/// The variable that names the worktree whose toolchain a command acts on.
const ROOT_VARIABLE: &str = "MONOCHROMATIC_WORKTREE_ROOT";

/// The command a person runs to finish the repair:
/// the repository's own task,
/// which runs the same installer.
pub const MANUAL_COMMAND: &str = "mise run toolchain:install";

/// What:
///  The launcher records of a worktree that name a lock file other than the worktree's own.
/// Why:
///   They are what a copy leaves behind;
/// a record naming this worktree's lock is already right and needs no repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staleLaunchRecords(destination: string): string[];
/// ```
pub fn stale_launch_records(destination: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(destination.join(LAUNCHERS)) else { return Vec::new() };
    let own: PathBuf = destination.join(LOCK_FILE);
    let own_real: Option<PathBuf> = match std::fs::canonicalize(destination) {
        Ok(real) => Some(real.join(LOCK_FILE)),
        Err(_) => None,
    };
    let mut stale: Vec<PathBuf> = Vec::new();
    for item in listing.flatten() {
        let path: PathBuf = item.path();
        let is_record: bool = match path.extension() {
            Some(extension) => extension == RECORD_EXTENSION,
            None => false,
        };
        if !is_record {
            continue;
        }
        let named: Option<PathBuf> = lock_named_by_record(&path);
        let Some(lock) = named else {
            debug(TAG, format!("{} names no lock file and is left alone", path.display()).as_str());
            continue;
        };
        if lock != own && Some(&lock) != own_real.as_ref() {
            stale.push(path);
        }
    }
    stale.sort();
    return stale;
}

/// What:
///  The lock file a launcher record names,
/// when the record can be read and holds the key.
///  `Option<PathBuf>` is "the path,
/// or no answer".
/// Why:
///   A record that cannot be read,
/// is not JSON or names no lock is left alone,
/// and each step of reading it may fail the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockNamedByRecord(path: string): string | undefined;
/// ```
fn lock_named_by_record(path: &Path) -> Option<PathBuf> {
    let text: String = std::fs::read_to_string(path).ok()?;
    let record: serde_json::Value = serde_json::from_str::<serde_json::Value>(text.as_str()).ok()?;
    let lock: &str = record.get(LOCK_KEY).and_then(serde_json::Value::as_str)?;
    return Some(PathBuf::from(lock));
}

/// What:
///  The command that starts mpm-shadow with the worktree as the toolchain root.
/// Why:
///   An inherited `MONOCHROMATIC_WORKTREE_ROOT` names the checkout the person works in,
/// which is not the new worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mpmCommand(destination: string, verb: string[]): ToolCommand;
/// ```
fn mpm_command(destination: &Path, arguments: &[&str]) -> ToolCommand {
    return ToolCommand {
        arguments: arguments.iter().map(OsString::from).collect(),
        directory: PathBuf::from(destination),
        set: vec![(OsString::from(ROOT_VARIABLE), destination.as_os_str().to_os_string())],
    };
}

/// What:
///  The part of mpm-shadow's last summary line that says what was installed,
/// such as `3 installed, 62 unchanged, 0 failed`.
/// Why:
///   The person is told what an add installed,
/// in mpm-shadow's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installSummary(stdout: string): string | undefined;
/// ```
fn install_summary(stdout: &[u8]) -> Option<String> {
    const PREFIX: &str = "toolchain install: ";
    let text: String = String::from_utf8_lossy(stdout).into_owned();
    for line in text.lines().rev() {
        if let Some(rest) = line.strip_prefix(PREFIX)
            && rest.contains("installed")
        {
            return Some(rest.to_owned());
        }
    }
    return None;
}

/// What:
///  One install of the worktree's toolchain,
/// with everything its start and its report need.
/// Why:
///  The launcher records are looked at once,
/// and the two faces of the repair
/// (`repair_toolchain` and `repair_toolchain_awaiting`) then differ only in how
///  they wait for the tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Install = { program: string; command: ToolCommand; destination: string; limits: ToolLimits };
/// ```
struct Install {
    /// The mpm-shadow the search path or the worktree's own link resolved.
    program: PathBuf,
    /// What it is started with and where.
    command: ToolCommand,
    /// The worktree whose launchers the install rewrites.
    destination: PathBuf,
    /// The two bounds the tool runs under.
    limits: ToolLimits,
}

/// What a toolchain repair is,
/// once the launcher records have been read.
/// Why:
///  The three answers differ in what they start and report,
/// and the records are read twice
/// (before and after the install),
/// so the
///  first reading decides what the repair is before either face waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolchainRepair = { kind: 'nothing' } | { kind: 'noProgram'; finding: Finding } | { kind: 'install'; work: Install };
/// ```
enum ToolchainRepair {
    /// Every launcher record names this worktree,
    /// so no tool runs and nothing is reported.
    NothingToDo,
    /// The worktree needs the install and the environment provides no mpm-shadow;
    /// the finding names the command to run.
    NoProgram(Finding),
    /// The install to start.
    Install(Install),
}

/// What:
///  What the toolchain repair of `destination` is,
/// decided from the launcher records and the search path alone.
/// Why:
///  Both faces decide it the same way before they wait,
/// so neither can start a tool the other would not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareInstall(destination: string, environment: RepairEnvironment): ToolchainRepair;
/// ```
fn prepare_install(destination: &Path, environment: &RepairEnvironment) -> ToolchainRepair {
    let stale: Vec<PathBuf> = stale_launch_records(destination);
    if stale.is_empty() {
        debug(TAG, format!("{}: every launcher record names this worktree", destination.display()).as_str());
        return ToolchainRepair::NothingToDo;
    }
    let own_link: PathBuf = destination.join(OWN_LINK);
    // The link in the worktree is looked at only when the search path gave no program, as `.or_else(...)` ran it.
    let found_program: Option<PathBuf> = match find_program(PROGRAM, environment.path()) {
        Some(program) => Some(program),
        None => {
            if own_link.is_file() { Some(own_link.clone()) } else { None }
        }
    };
    let Some(program) = found_program else {
        return ToolchainRepair::NoProgram(missing_tool(Tool::Toolchain, PROGRAM, MANUAL_COMMAND));
    };
    return ToolchainRepair::Install(Install { program, command: mpm_command(destination, &["install", "--frozen-lockfile"]), destination: PathBuf::from(destination), limits: environment.limits() });
}

/// The install,
/// awaited on the runtime the caller runs on.
async fn install_awaiting(work: &Install) -> ToolRun {
    return run_tool_awaiting(work.program.as_path(), &work.command, work.limits).await;
}

/// What the install reports:
/// the line the person reads,
/// and the records that still name another lock file.
/// Why:
///  Both faces report the same run the same way,
/// whatever waited for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const installFinding = (work: Install, run: ToolRun) => { ... };
/// ```
fn install_finding(work: &Install, run: &ToolRun) -> Option<Finding> {
    debug(TAG, format!("{}: install ended {:?} after {} ms", work.destination.display(), run.end, run.millis).as_str());
    let detail: String = match install_summary(run.stdout.as_slice()) {
        Some(summary) => format!("mpm-shadow rebuilt the toolchain links ({summary})"),
        None => String::from("mpm-shadow rebuilt the toolchain links"),
    };
    let finding: Finding = finding_from(Tool::Toolchain, run, detail.as_str(), MANUAL_COMMAND);
    let remaining: usize = stale_launch_records(work.destination.as_path()).len();
    if matches!(finding.outcome, Outcome::Repaired { .. }) && remaining > 0 {
        return Some(Finding { tool: Tool::Toolchain, outcome: Outcome::Failed { reason: format!("{remaining} launcher records still name another lock file"), run_instead: String::from(MANUAL_COMMAND) } });
    }
    return Some(finding);
}

/// What:
///  Rebuild the worktree's toolchain links and launchers,
/// when its launcher records are stale.
/// Why:
///   See the module comment:
/// one frozen install,
/// with the network when the lock names tools that are not installed yet.
/// The callers of this face are synchronous (`package_repair.rs` inside the repair the add runs),
///  so it enters the process's runtime once and waits there;
/// from inside the runtime it refuses,
/// and `repair_toolchain_awaiting` is
///  the face such a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repairToolchain(destination: string, environment: RepairEnvironment): Finding | undefined;
/// ```
pub fn repair_toolchain(destination: &Path, environment: &RepairEnvironment) -> Option<Finding> {
    return match prepare_install(destination, environment) {
        ToolchainRepair::NothingToDo => None,
        ToolchainRepair::NoProgram(finding) => Some(finding),
        ToolchainRepair::Install(work) => {
            let started: Instant = Instant::now();
            let run: ToolRun = match crate::lazy_runtime::run_on_runtime(install_awaiting(&work)) {
                Ok(run) => run,
                // No install was made, and the runtime's own reason is what the person is told.
                Err(refusal) => run_refused(work.program.as_path(), started, &refusal),
            };
            install_finding(&work, &run)
        }
    };
}

/// What:
///  `repair_toolchain` on the runtime:
/// the install is awaited,
/// never blocked on.
/// Why:
///  A caller that is itself a task of the runtime cannot enter it
/// (`RuntimeError::Nested`),
/// so it awaits this face;
/// the finding is the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairToolchainAwaiting(destination: string, environment: RepairEnvironment): Promise<Finding | undefined>;
/// ```
pub async fn repair_toolchain_awaiting(destination: &Path, environment: &RepairEnvironment) -> Option<Finding> {
    return match prepare_install(destination, environment) {
        ToolchainRepair::NothingToDo => None,
        ToolchainRepair::NoProgram(finding) => Some(finding),
        ToolchainRepair::Install(work) => {
            let run: ToolRun = install_awaiting(&work).await;
            install_finding(&work, &run)
        }
    };
}

/// Toolchain repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_toolchain_tests.rs"]
mod tests;
