//! What:
//!  The pnpm repair:
//! `pnpm install --offline --frozen-lockfile` in a new worktree whose `node_modules` was copied.
//! Why:
//!  pnpm's links reach its store by a path counted in directory levels from the project,
//! so a copy at another depth holds
//!  links that are dead or,
//! worse,
//! resolve through a parent directory's `node_modules` to the source's packages.
//! pnpm
//!  rewrites its own layout for the new location in about the time of a no-op install.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairPnpm(destination, environment); // undefined when pnpm has nothing to relink
//! ```

use crate::package_repair_request::RepairEnvironment;
use crate::package_repair_command::{ToolCommand, ToolLimits, ToolRun, find_program, run_refused, run_tool_awaiting};
use crate::package_repair_report::{Finding, Tool, finding_from, missing_tool};
use crate::shell_quote::shell_quote;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The program the person's environment resolves;
/// in this repository a shadow that checks manifests,
/// then runs pnpm.
const PROGRAM: &str = "pnpm";

/// The file that names the packages the layout is built from.
const LOCKFILE: &str = "pnpm-lock.yaml";

/// The directory pnpm links packages into.
const MODULES: &str = "node_modules";

/// The arguments of the relink:
/// no network,
/// and no change to the lockfile.
const ARGUMENTS: [&str; 3] = ["install", "--offline", "--frozen-lockfile"];

/// What:
///  The command a person runs to do what the repair could not,
/// from any directory.
/// Why:
///   `--dir` names the worktree,
/// so the printed line works wherever it is pasted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function manualCommand(destination: string): string;
/// ```
pub fn manual_command(destination: &Path) -> String {
    return format!("{PROGRAM} --dir {} {}", shell_quote(destination.as_os_str().as_encoded_bytes()), ARGUMENTS.join(" "));
}

/// What:
///  One relink of a worktree's `node_modules`,
/// with everything its start and its report need.
/// Why:
///  The worktree is looked at once,
/// and the two faces of the repair
/// (`repair_pnpm` and `repair_pnpm_awaiting`) then differ only in how
///  they wait for the tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Relink = { program: string; command: ToolCommand; manual: string; limits: ToolLimits };
/// ```
struct Relink {
    /// The pnpm the person's search path resolved.
    program: PathBuf,
    /// What it is started with and where.
    command: ToolCommand,
    /// The command a person runs to do what the repair could not.
    manual: String,
    /// The two bounds the tool runs under.
    limits: ToolLimits,
}

/// What a pnpm repair is,
/// once the worktree has been looked at.
/// Why:
///  The three answers differ in what they start and report,
/// and the lookup that decides between them reads nothing
///  that waits,
/// so both faces make it before they wait for anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PnpmRepair = { kind: 'nothing' } | { kind: 'noPnpm'; finding: Finding } | { kind: 'relink'; work: Relink };
/// ```
enum PnpmRepair {
    /// There is no lockfile or no `node_modules`,
    /// so no tool runs and nothing is reported.
    NothingToDo,
    /// The worktree needs a relink and the environment provides no pnpm;
    /// the finding names the command to run.
    NoPnpm(Finding),
    /// The relink to start.
    Relink(Relink),
}

/// What:
///  What the pnpm repair of `destination` is,
/// decided from the worktree and the search path alone.
/// Why:
///  Both faces decide it the same way before they wait,
/// so neither can start a tool the other would not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareRelink(destination: string, environment: RepairEnvironment): PnpmRepair;
/// ```
fn prepare_relink(destination: &Path, environment: &RepairEnvironment) -> PnpmRepair {
    if !destination.join(LOCKFILE).is_file() || !destination.join(MODULES).is_dir() {
        crate::diagnostic_log::debug("packageRepairPnpm", format!("{}: no lockfile or no node_modules, nothing to relink", destination.display()).as_str());
        return PnpmRepair::NothingToDo;
    }
    let manual: String = manual_command(destination);
    let Some(program) = find_program(PROGRAM, environment.path()) else {
        return PnpmRepair::NoPnpm(missing_tool(Tool::Pnpm, PROGRAM, manual.as_str()));
    };
    let command: ToolCommand = ToolCommand { arguments: ARGUMENTS.iter().map(OsString::from).collect(), directory: PathBuf::from(destination), set: Vec::new() };
    return PnpmRepair::Relink(Relink { program, command, manual, limits: environment.limits() });
}

/// What:
///  What the worktree needs of the pnpm repair,
/// decided from the worktree and the search path alone.
/// Why:
///   The add no longer runs the relink itself,
/// so it asks this before it writes the record the service runs:
/// a worktree with no layout to relink leaves no work,
/// and one whose pnpm is missing gets the line that names the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PnpmNeed = 'nothing' | 'needed' | { missing: Finding };
/// ```
pub(crate) enum PnpmNeed {
    /// There is no lockfile or no `node_modules`,
    /// so no relink is needed.
    Nothing,
    /// The relink is needed and the search path provides pnpm;
    /// running it is the commit service's work.
    Needed,
    /// The relink is needed and the search path provides no pnpm;
    /// the finding names the command to run.
    NoProgram(Finding),
}

/// What:
///  What this tool's part of a worktree's repair is,
/// before anything is started.
/// Why:
///  Both faces of the repair decide it the same way,
/// so a worktree the add hands over is one the repair would have relinked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needOf = (destination, environment) => decide(prepareRelink(destination, environment));
/// ```
pub(crate) fn need_of(destination: &Path, environment: &RepairEnvironment) -> PnpmNeed {
    return match prepare_relink(destination, environment) {
        PnpmRepair::NothingToDo => PnpmNeed::Nothing,
        PnpmRepair::NoPnpm(finding) => PnpmNeed::NoProgram(finding),
        PnpmRepair::Relink(_work) => PnpmNeed::Needed,
    };
}

/// The relink,
/// awaited on the runtime the caller runs on.
async fn relink_awaiting(work: &Relink) -> ToolRun {
    return run_tool_awaiting(work.program.as_path(), &work.command, work.limits).await;
}

/// What the relink reports:
/// the line the person reads,
/// and the diagnostic that says how the tool ended.
/// Why:
///  Both faces report the same run the same way,
/// whatever waited for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const relinkFinding = (work: Relink, destination: string, run: ToolRun) => findingFrom('pnpm', run, 'pnpm relinked node_modules', work.manual);
/// ```
fn relink_finding(work: &Relink, destination: &Path, run: &ToolRun) -> Finding {
    crate::diagnostic_log::debug("packageRepairPnpm", format!("{}: {} ended {:?} after {} ms", destination.display(), work.program.display(), run.end, run.millis).as_str());
    return finding_from(Tool::Pnpm, run, "pnpm relinked node_modules", work.manual.as_str());
}

/// What:
///  Relink the copied `node_modules` of a worktree with pnpm,
/// when it has a lockfile and a `node_modules`.
/// Why:
///   Without a lockfile or a copied `node_modules` there is no layout to repair,
/// and nothing is started.
/// The callers of this face are synchronous (`package_repair.rs` inside the repair the add runs),
///  so it enters the process's runtime once and waits there;
/// from inside the runtime it refuses,
/// and `repair_pnpm_awaiting` is the
///  face such a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repairPnpm(destination: string, environment: RepairEnvironment): Finding | undefined;
/// ```
pub fn repair_pnpm(destination: &Path, environment: &RepairEnvironment) -> Option<Finding> {
    return match prepare_relink(destination, environment) {
        PnpmRepair::NothingToDo => None,
        PnpmRepair::NoPnpm(finding) => Some(finding),
        PnpmRepair::Relink(work) => {
            let started: Instant = Instant::now();
            let run: ToolRun = match crate::lazy_runtime::run_on_runtime(relink_awaiting(&work)) {
                Ok(run) => run,
                // No relink was made, and the runtime's own reason is what the person is told.
                Err(refusal) => run_refused(work.program.as_path(), started, &refusal),
            };
            Some(relink_finding(&work, destination, &run))
        }
    };
}

/// What:
///  `repair_pnpm` on the runtime:
/// the relink is awaited,
/// never blocked on.
/// Why:
///  A caller that is itself a task of the runtime cannot enter it
/// (`RuntimeError::Nested`),
/// so it awaits this face;
/// the finding is the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairPnpmAwaiting(destination: string, environment: RepairEnvironment): Promise<Finding | undefined>;
/// ```
pub async fn repair_pnpm_awaiting(destination: &Path, environment: &RepairEnvironment) -> Option<Finding> {
    return match prepare_relink(destination, environment) {
        PnpmRepair::NothingToDo => None,
        PnpmRepair::NoPnpm(finding) => Some(finding),
        PnpmRepair::Relink(work) => {
            let run: ToolRun = relink_awaiting(&work).await;
            Some(relink_finding(&work, destination, &run))
        }
    };
}

/// pnpm repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_pnpm_tests.rs"]
mod tests;
