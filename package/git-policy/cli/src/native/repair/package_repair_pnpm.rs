//! What:
//!  The pnpm repair: `pnpm install --offline --frozen-lockfile` in a new worktree whose `node_modules` was copied.
//! Why:
//!  pnpm's links reach its store by a path counted in directory levels from the project, so a copy at another depth holds
//!  links that are dead or, worse, resolve through a parent directory's `node_modules` to the source's packages. pnpm
//!  rewrites its own layout for the new location in about the time of a no-op install.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairPnpm(destination, environment); // undefined when pnpm has nothing to relink
//! ```

use crate::package_repair::RepairEnvironment;
use crate::package_repair_command::{ToolCommand, ToolRun, find_program, run_tool};
use crate::package_repair_report::{Finding, Tool, finding_from, missing_tool};
use crate::shell_quote::shell_quote;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The program the person's environment resolves; in this repository a shadow that checks manifests, then runs pnpm.
const PROGRAM: &str = "pnpm";

/// The file that names the packages the layout is built from.
const LOCKFILE: &str = "pnpm-lock.yaml";

/// The directory pnpm links packages into.
const MODULES: &str = "node_modules";

/// The arguments of the relink: no network, and no change to the lockfile.
const ARGUMENTS: [&str; 3] = ["install", "--offline", "--frozen-lockfile"];

/// What:
///  The command a person runs to do what the repair could not, from any directory.
/// Why:
///   `--dir` names the worktree, so the printed line works wherever it is pasted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function manualCommand(destination: string): string;
/// ```
pub fn manual_command(destination: &Path) -> String {
    return format!("{PROGRAM} --dir {} {}", shell_quote(destination.as_os_str().as_encoded_bytes()), ARGUMENTS.join(" "));
}

/// What:
///  Relink the copied `node_modules` of a worktree with pnpm, when it has a lockfile and a `node_modules`.
/// Why:
///   Without a lockfile or a copied `node_modules` there is no layout to repair, and nothing is started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairPnpm(destination: string, environment: RepairEnvironment): Promise<Finding | undefined>;
/// ```
pub fn repair_pnpm(destination: &Path, environment: &RepairEnvironment) -> Option<Finding> {
    if !destination.join(LOCKFILE).is_file() || !destination.join(MODULES).is_dir() {
        crate::diagnostic_log::debug("packageRepairPnpm", format!("{}: no lockfile or no node_modules, nothing to relink", destination.display()).as_str());
        return None;
    }
    let manual: String = manual_command(destination);
    let Some(program) = find_program(PROGRAM, environment.path()) else {
        return Some(missing_tool(Tool::Pnpm, PROGRAM, manual.as_str()));
    };
    let command: ToolCommand = ToolCommand { arguments: ARGUMENTS.iter().map(OsString::from).collect(), directory: PathBuf::from(destination), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, environment.quiet_window);
    crate::diagnostic_log::debug("packageRepairPnpm", format!("{}: {} ended {:?} after {} ms", destination.display(), program.display(), run.end, run.millis).as_str());
    return Some(finding_from(Tool::Pnpm, &run, "pnpm relinked node_modules", manual.as_str()));
}

/// pnpm repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_pnpm_tests.rs"]
mod tests;
