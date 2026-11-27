//! What:
//!  The toolchain repair itself:
//!  what a worktree's launcher records ask of it,
//!  the `mpm-shadow install --frozen-lockfile` that answers them,
//!  the count of records that still name another lock after it,
//!  and the two faces that wait for it,
//!  one for a caller on its own thread and one for a caller on the runtime.
//! Why:
//!  `package_repair_toolchain.rs` would otherwise pass the file-size limit,
//!  and the decision,
//!  the run and the report of it are one role,
//!  which reads the records and their removal rather than writing them,
//!  so it lives in a file of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairToolchainAwaiting(destination, environment);
//! ```

/// Import the removal of the copies no install rewrites.
use super::copy_removal::remove_what_no_install_rewrites;
/// Import the one reading of the launcher directory both faces decide from.
use super::launcher_records::{
    Launchers,
    launchers_of,
};
/// Import the log tag.
use super::names_and_bounds::TAG;
/// Import the program and the variables it runs with.
use super::names_and_bounds::{
    OWN_LINK,
    PROGRAM,
    ROOT_VARIABLE,
};
/// Import the refusal a place of the blocking pool gives.
use crate::blocking_job::BlockingJobError;
/// Import the run of a blocking job within the process's budget,
/// or in turn.
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the command and the bounds of one package-manager program.
use crate::package_repair_command::{
    ToolCommand,
    ToolLimits,
};
/// Import the run of one package-manager program.
use crate::package_repair_command::{
    ToolRun,
    find_program,
    run_refused,
    run_tool_awaiting,
};
/// Import the finding and the outcome a repair reports.
use crate::package_repair_report::{
    Finding,
    Outcome,
    Tool,
};
/// Import the constructors of a finding for a run that ended.
use crate::package_repair_report::{
    finding_from,
    missing_tool,
};
/// Import what the process offers a repair.
use crate::package_repair_request::RepairEnvironment;
/// Import the command a person runs.
use crate::the_commands_a_repair_prints::MANUAL_COMMAND;
/// Import the owned argument a command is started with.
use std::ffi::OsString;
/// Import native paths.
use std::path::{
    Path,
    PathBuf,
};
/// Import the moment a run began.
use std::time::Instant;
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
fn mpm_command(
    destination: &Path,
    arguments: &[&str],
    environment: &RepairEnvironment,
) -> ToolCommand {
    let mut set: Vec<(OsString, OsString)> = vec![(
        OsString::from(ROOT_VARIABLE),
        destination.as_os_str().to_os_string(),
    )];
    set.extend(crate::package_repair_request::with_the_persons_path(
        environment,
    ));
    return ToolCommand {
        arguments: arguments.iter().map(OsString::from).collect(),
        directory: PathBuf::from(destination),
        set,
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
pub(super) struct Install {
    /// The mpm-shadow the search path or the worktree's own link resolved.
    pub(super) program: PathBuf,
    /// What it is started with and where.
    pub(super) command: ToolCommand,
    /// The worktree whose launchers the install rewrites.
    pub(super) destination: PathBuf,
    /// The two bounds the tool runs under.
    pub(super) limits: ToolLimits,
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
pub(super) enum ToolchainRepair {
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
///  `prepare_install` for the state a job of the blocking pool owns.
/// Why:
///  A job reads only what it was handed,
/// so the worktree and the environment travel together as owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareInstallInAJob([destination, environment]: [string, RepairEnvironment]): ToolchainRepair;
/// ```
fn prepare_install_in_a_job(
    (destination, environment): (PathBuf, RepairEnvironment)
) -> ToolchainRepair {
    return prepare_install_on_a_caller_thread(destination.as_path(), &environment);
}

/// What:
///  `prepare_install` as a job of the process's blocking pool.
/// Why:
///  The launcher records are storage work,
/// and the task that polls the three repairs must never do any of it:
/// the toolchain wait shares that task with the pnpm ceiling and the Cargo waits,
///  so a read that waited there would stop
/// their deadlines with it.
/// A refusal from the system runs the same read in turn,
/// under a place of the process's bound on blocking jobs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decided = await prepareInstallWithinTheBudget(destination, environment);
/// ```
pub(super) async fn prepare_install_within_the_budget(
    destination: &Path,
    environment: &RepairEnvironment,
) -> Result<ToolchainRepair, BlockingJobError> {
    let state: (PathBuf, RepairEnvironment) = (destination.to_path_buf(), environment.clone());
    return run_a_job_or_in_turn(state, prepare_install_in_a_job).await;
}

/// What:
///  How many launcher records the worktree's lock declares still name another lock file,
/// as a job of the process's blocking pool.
/// Why:
///  The records are read again after the install,
/// on the same task the install was awaited from,
/// so the same rule holds:
/// no storage read of the launcher directory runs on the task that polls the repairs.
///  The count leaves out the copies no install rewrites,
///  because an install that did everything the lock asks of it is not a failed relink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await countStaleWithinTheBudget(destination);
/// ```
fn count_stale_in_a_job(destination: PathBuf) -> usize {
    return launchers_of(destination.as_path()).stale.len();
}

/// What:
///  `count_stale_in_a_job` awaited on the runtime.
/// Why:
///  The awaiting face has no read of the launcher directory of its own,
/// so a worktree whose records changed under the install is counted without stopping the task's other waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await countStaleWithinTheBudget(destination);
/// ```
pub(super) async fn count_stale_within_the_budget(
    destination: &Path
) -> Result<usize, BlockingJobError> {
    return run_a_job_or_in_turn(destination.to_path_buf(), count_stale_in_a_job).await;
}

/// What:
///  What the toolchain repair of `destination` is,
/// decided from the launcher records and the search path alone,
/// for a caller that is not a task of the runtime.
/// Why:
///  Both faces decide it the same way before they wait,
/// so neither can start a tool the other would not.
///  A launcher record whose name this worktree's lock declares nowhere is removed here,
///  so every face that asks what the worktree needs also leaves it holding nothing another checkout wrote.
/// This face runs on the caller's own thread,
/// which polls no wait of the repair,
/// so its reads need no place of the pool;
/// the awaiting face reads them as a job (`prepare_install_within_the_budget`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareInstall(destination: string, environment: RepairEnvironment): ToolchainRepair;
/// ```
fn prepare_install_on_a_caller_thread(
    destination: &Path,
    environment: &RepairEnvironment,
) -> ToolchainRepair {
    let launchers: Launchers = launchers_of(destination);
    // A copy of an earlier release's layout is removed rather than reported: no install rewrites it, and it names another
    //  checkout's lock until it goes. The removal proves the bytes it deletes again, because a concurrent update and
    //  install can write a correct record and launcher for a name the lock declares between the two.
    remove_what_no_install_rewrites(destination, launchers.unrewritable.as_slice());
    if launchers.stale.is_empty() {
        debug(
            TAG,
            format!(
                "{}: every launcher record this worktree's lock declares names this worktree",
                destination.display()
            )
            .as_str(),
        );
        return ToolchainRepair::NothingToDo;
    }
    let own_link: PathBuf = destination.join(OWN_LINK);
    // The link in the worktree is looked at only when the search path gave no program, as `.or_else(...)` ran it.
    let found_program: Option<PathBuf> =
        match find_program(PROGRAM, environment.path(), &environment.start_directory) {
            Some(program) => Some(program),
            None => {
                if own_link.is_file() {
                    Some(own_link.clone())
                } else {
                    None
                }
            }
        };
    let Some(program) = found_program else {
        return ToolchainRepair::NoProgram(missing_tool(Tool::Toolchain, PROGRAM, MANUAL_COMMAND));
    };
    return ToolchainRepair::Install(Install {
        program,
        command: mpm_command(destination, &["install", "--frozen-lockfile"], environment),
        destination: PathBuf::from(destination),
        limits: environment.limits(),
    });
}

/// What:
///  What the worktree needs of the toolchain repair,
/// decided from the launcher records and the search path alone.
/// Why:
///   The add no longer runs the install itself,
/// so it asks this before it writes the record the service runs:
/// a worktree whose launchers already name it leaves no work,
/// and one whose mpm-shadow is missing gets the line that names the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolchainNeed = 'nothing' | 'needed' | { missing: Finding };
/// ```
pub(crate) enum ToolchainNeed {
    /// Every launcher record names this worktree,
    /// so no install is needed.
    Nothing,
    /// The install is needed and a program was found;
    /// running it is the commit service's work.
    Needed,
    /// The install is needed and no mpm-shadow was found;
    /// the finding names the command to run.
    NoProgram(Finding),
}

/// What:
///  What the toolchain repair is,
/// as one of the three answers the add asks about.
/// Why:
///  The install carries the program and the command it built,
/// and the add needs only which of the three states the worktree is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needOfPrepared = (prepared) => prepared.kind === 'nothing' ? 'nothing' : prepared.kind === 'install' ? 'needed' : prepared.finding;
/// ```
fn need_of_prepared(prepared: ToolchainRepair) -> ToolchainNeed {
    return match prepared {
        ToolchainRepair::NothingToDo => ToolchainNeed::Nothing,
        ToolchainRepair::NoProgram(finding) => ToolchainNeed::NoProgram(finding),
        ToolchainRepair::Install(_work) => ToolchainNeed::Needed,
    };
}

/// What:
///  `need_of_prepared` on the records of the worktree,
/// read as a job of the process's blocking pool.
/// Why:
///   The add asks this on the task that also runs the Cargo repair,
/// so the read of the launcher directory must never run there;
/// the pool's refusal is handed back so the add can say what it could not decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needWithinTheBudget = async (destination, environment) => needOfPrepared(await prepareInstallWithinTheBudget(destination, environment));
/// ```
pub(crate) async fn need_within_the_budget(
    destination: &Path,
    environment: &RepairEnvironment,
) -> Result<ToolchainNeed, BlockingJobError> {
    return match prepare_install_within_the_budget(destination, environment).await {
        Ok(prepared) => Ok(need_of_prepared(prepared)),
        Err(refusal) => Err(refusal),
    };
}

/// The install,
/// awaited on the runtime the caller runs on.
pub(super) async fn install_awaiting(work: &Install) -> ToolRun {
    return run_tool_awaiting(work.program.as_path(), &work.command, work.limits).await;
}

/// What the install reports:
/// the line the person reads,
/// and the records that still name another lock file.
/// Why:
///  Both faces report the same run the same way,
/// whatever waited for it,
/// and the read of the records is the caller's:
/// the synchronous face reads them on its caller's thread,
/// and the awaiting face hands over a job of the pool.
/// A success whose records could not be read again is a failure that says so,
/// never a success the wrapper did not check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const installFinding = (work: Install, run: ToolRun, remaining: number | Refusal) => Finding | undefined;
/// ```
pub(super) fn install_finding(
    work: &Install,
    run: &ToolRun,
    remaining: Result<usize, BlockingJobError>,
) -> Option<Finding> {
    debug(
        TAG,
        format!(
            "{}: install ended {:?} after {} ms",
            work.destination.display(),
            run.end,
            run.millis
        )
        .as_str(),
    );
    let detail: String = match install_summary(run.stdout.as_slice()) {
        Some(summary) => format!("mpm-shadow rebuilt the toolchain links ({summary})"),
        None => String::from("mpm-shadow rebuilt the toolchain links"),
    };
    let finding: Finding = finding_from(Tool::Toolchain, run, detail.as_str(), MANUAL_COMMAND);
    if !matches!(finding.outcome, Outcome::Repaired { .. }) {
        return Some(finding);
    }
    let left: Option<String> = match remaining {
        Ok(0) => None,
        Ok(count) => Some(format!(
            "{count} launcher records still name another lock file"
        )),
        Err(refusal) => Some(format!(
            "the launcher records could not be read again ({refusal})"
        )),
    };
    return match left {
        None => Some(finding),
        Some(said) => Some(Finding {
            tool: Tool::Toolchain,
            outcome: Outcome::Failed {
                reason: said,
                run_instead: String::from(MANUAL_COMMAND),
            },
        }),
    };
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
pub fn repair_toolchain(
    destination: &Path,
    environment: &RepairEnvironment,
) -> Option<Finding> {
    return match prepare_install_on_a_caller_thread(destination, environment) {
        ToolchainRepair::NothingToDo => None,
        ToolchainRepair::NoProgram(finding) => Some(finding),
        ToolchainRepair::Install(work) => {
            let started: Instant = Instant::now();
            let run: ToolRun = match crate::lazy_runtime::run_on_runtime(install_awaiting(&work)) {
                Ok(run) => run,
                // No install was made, and the runtime's own reason is what the person is told.
                Err(refusal) => run_refused(work.program.as_path(), started, &refusal),
            };
            install_finding(
                &work,
                &run,
                Ok(launchers_of(work.destination.as_path()).stale.len()),
            )
        }
    };
}

/// What:
///  `repair_toolchain` on the runtime:
/// the install is awaited,
/// never blocked on,
/// and every read of the launcher records is a job of the pool.
/// Why:
///  A caller that is itself a task of the runtime cannot enter it
/// (`RuntimeError::Nested`),
/// so it awaits this face;
/// the finding is the synchronous face's.
/// Nothing of this face's work runs on the task that polls it,
/// so the pnpm and Cargo repairs beside it keep their deadlines
/// whatever the worktree holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairToolchainAwaiting(destination: string, environment: RepairEnvironment): Promise<Finding | undefined>;
/// ```
pub async fn repair_toolchain_awaiting(
    destination: &Path,
    environment: &RepairEnvironment,
) -> Option<Finding> {
    return match prepare_install_within_the_budget(destination, environment).await {
        Ok(ToolchainRepair::NothingToDo) => None,
        Ok(ToolchainRepair::NoProgram(finding)) => Some(finding),
        Ok(ToolchainRepair::Install(work)) => {
            let run: ToolRun = install_awaiting(&work).await;
            install_finding(
                &work,
                &run,
                count_stale_within_the_budget(work.destination.as_path()).await,
            )
        }
        // The records were not read, so nothing about the launchers is known; the person is told the reason.
        Err(refusal) => Some(Finding {
            tool: Tool::Toolchain,
            outcome: Outcome::Failed {
                reason: format!(
                    "the launcher records of the worktree could not be read ({refusal})"
                ),
                run_instead: String::from(MANUAL_COMMAND),
            },
        }),
    };
}
