//! What:
//!  The toolchain repair: when the launchers copied into a new worktree's `.cache/toolchain-launchers` still name another
//!  checkout's `toolchain.lock.json`, rebuild the worktree's own links and launchers with `mpm-shadow install
//!  --frozen-lockfile`.
//! Why:
//!  mpm-shadow owns `toolchain.plan.jsonc`, `toolchain.lock.json` and the pinned tools. Its exposed links and launcher records
//!  hold absolute paths, so a copy would run the source checkout's tool versions and guard, and a branch that changes the
//!  lock would silently use the source's. It installs nothing here: the repair runs only when `mpm-shadow list` finds every
//!  tool in sync, so `install` only relinks and never reaches the network. A tool that is out of sync is left for the person,
//!  with `mise run toolchain:install` named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairToolchain(destination, environment); // undefined when the launchers already name this worktree
//! ```

use super::diagnostic_log::debug;
use super::package_repair::RepairEnvironment;
use super::package_repair_command::{ToolCommand, ToolEnd, ToolRun, find_program, run_tool};
use super::package_repair_report::{Finding, Outcome, Tool, finding_from, missing_tool};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairToolchain";

/// Where mpm-shadow keeps the launchers and their records, relative to a worktree.
const LAUNCHERS: &str = ".cache/toolchain-launchers";

/// The record that sits beside a launcher, named after it.
const RECORD_EXTENSION: &str = "launch";

/// The lock file a launcher record names, relative to a worktree.
const LOCK_FILE: &str = "toolchain.lock.json";

/// The key of a launcher record that names the lock file.
const LOCK_KEY: &str = "lockFile";

/// The program that owns the toolchain, and where a worktree keeps a link to it.
const PROGRAM: &str = "mpm-shadow";

/// Where the worktree's own link to the program sits when the search path does not provide one.
const OWN_LINK: &str = ".cache/toolchain-bin/mpm-shadow";

/// The variable that names the worktree whose toolchain a command acts on.
const ROOT_VARIABLE: &str = "MONOCHROMATIC_WORKTREE_ROOT";

/// The command a person runs to finish the repair: the repository's own task, which runs the same installer.
pub const MANUAL_COMMAND: &str = "mise run toolchain:install";

/// What:
///  The launcher records of a worktree that name a lock file other than the worktree's own.
/// Why:
///   They are what a copy leaves behind; a record naming this worktree's lock is already right and needs no repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staleLaunchRecords(destination: string): string[];
/// ```
pub fn stale_launch_records(destination: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(destination.join(LAUNCHERS)) else { return Vec::new() };
    let own: PathBuf = destination.join(LOCK_FILE);
    let own_real: Option<PathBuf> = std::fs::canonicalize(destination).ok().map(|real: PathBuf| { return real.join(LOCK_FILE); });
    let mut stale: Vec<PathBuf> = Vec::new();
    for item in listing.flatten() {
        let path: PathBuf = item.path();
        if path.extension().is_none_or(|extension| { return extension != RECORD_EXTENSION; }) {
            continue;
        }
        let named: Option<PathBuf> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text: String| { return serde_json::from_str::<serde_json::Value>(text.as_str()).ok(); })
            .and_then(|record: serde_json::Value| { return record.get(LOCK_KEY).and_then(serde_json::Value::as_str).map(PathBuf::from); });
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
///  The command that starts mpm-shadow with the worktree as the toolchain root.
/// Why:
///   An inherited `MONOCHROMATIC_WORKTREE_ROOT` names the checkout the person works in, which is not the new worktree.
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
///  Rebuild the worktree's toolchain links and launchers, when its launcher records are stale and its tools are in sync.
/// Why:
///   `list` reads and compares; `install --frozen-lockfile` then only relinks. Installing while the tools differ could
///  download, which a worktree add must not do unasked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairToolchain(destination: string, environment: RepairEnvironment): Promise<Finding | undefined>;
/// ```
pub fn repair_toolchain(destination: &Path, environment: &RepairEnvironment) -> Option<Finding> {
    let stale: Vec<PathBuf> = stale_launch_records(destination);
    if stale.is_empty() {
        debug(TAG, format!("{}: every launcher record names this worktree", destination.display()).as_str());
        return None;
    }
    let own_link: PathBuf = destination.join(OWN_LINK);
    let Some(program) = find_program(PROGRAM, environment.path()).or_else(|| { return own_link.is_file().then_some(own_link.clone()); }) else {
        return Some(missing_tool(Tool::Toolchain, PROGRAM, MANUAL_COMMAND));
    };
    let listed: ToolRun = run_tool(program.as_path(), &mpm_command(destination, &["list"]), environment.tool_limit);
    debug(TAG, format!("{}: list ended {:?} after {} ms", destination.display(), listed.end, listed.millis).as_str());
    if listed.end != ToolEnd::Succeeded {
        let reason: String = match &listed.end {
            ToolEnd::Failed(text) => format!("the installed tools differ from this worktree's {LOCK_FILE}, so nothing was installed: {text}"),
            ToolEnd::TimedOut | ToolEnd::Succeeded => String::from("listing the installed tools ran past its time limit and was stopped"),
        };
        return Some(Finding { tool: Tool::Toolchain, outcome: Outcome::Failed { reason, run_instead: String::from(MANUAL_COMMAND) } });
    }
    let installed: ToolRun = run_tool(program.as_path(), &mpm_command(destination, &["install", "--frozen-lockfile"]), environment.tool_limit);
    debug(TAG, format!("{}: install ended {:?} after {} ms", destination.display(), installed.end, installed.millis).as_str());
    let total: ToolRun = ToolRun { end: installed.end.clone(), millis: listed.millis + installed.millis, stdout: Vec::new() };
    let finding: Finding = finding_from(Tool::Toolchain, &total, "mpm-shadow rebuilt the toolchain links", MANUAL_COMMAND);
    let remaining: usize = stale_launch_records(destination).len();
    if matches!(finding.outcome, Outcome::Repaired { .. }) && remaining > 0 {
        return Some(Finding { tool: Tool::Toolchain, outcome: Outcome::Failed { reason: format!("{remaining} launcher records still name another lock file"), run_instead: String::from(MANUAL_COMMAND) } });
    }
    return Some(finding);
}

/// Toolchain repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_toolchain_tests.rs"]
mod tests;
