//! What:
//!  The repair of one Cargo workspace of a new worktree:
//! plan what its build state needs,
//! hand the filling of its build directory to the commit service,
//! and clean the worktree's own crates here,
//! with every wait awaited on the runtime.
//! Why:
//!  Each workspace's state is in a directory of its own,
//! so one workspace is one item of the repair's ordered run,
//! and the Cargo program and the two bounds of its runs are the values an item owns.
//! A workspace whose build state is only in the source checkout is the service's work,
//! and one whose build directory holds state the copy carried is cleaned here,
//! so a build cannot be overtaken by a later clean.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CargoTool = { program: string; limits: { windowMs: number; ceilingMs: number } };
//! type Done = { kind: 'skipped' } | { kind: 'cleaned'; said?: string } | { kind: 'handed-over'; entry: PrewarmEntry } | { kind: 'failed'; finding: Finding };
//! async function repairWorkspaceAwaiting(tool: CargoTool, workspace: Workspace, destinationRoot: string, source: string, keptIn?: string): Promise<Done>;
//! ```

/// Import the timing record the two tool steps are kept in.
use crate::add_timing::{
    begin_work,
    end_work,
};
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the plan of what a workspace's build state needs and the facts it is made from.
use crate::package_repair_cargo_build_state_plan::{
    BuildStatePlan,
    StateFacts,
    plan_build_state,
};
/// Import the clean's arguments and the command a person runs instead.
use crate::package_repair_cargo_clean_arguments::{
    clean_arguments,
    manual_command,
};
/// Import the reading of `Cargo.lock` and the restore of its checked-out bytes.
use crate::package_repair_cargo_lock::{
    read_lock,
    restore_lock,
};
/// Import a Cargo workspace and the reading of one.
use crate::package_repair_cargo_workspaces::{
    MANIFEST,
    Workspace,
    read_workspace_awaiting,
};
/// Import the command one tool run is given,
/// its bounds and how it ended.
use crate::package_repair_command::{
    ToolCommand,
    ToolEnd,
    ToolLimits,
    ToolRun,
    run_tool_awaiting,
};
/// Import what the commit service is asked to fill,
/// and the entry one workspace hands it.
use crate::package_repair_prewarm::PrewarmEntry;
/// Import the add's own move of the worktree's copied build state.
use crate::package_repair_prewarm_job::stage_the_worktrees_own_state;
/// Import what a repair reports and the words of a ceiling it reached.
use crate::package_repair_report::{
    Finding,
    Outcome,
    Tool,
};
/// Import the check that a directory a repair deletes from is inside the worktree it serves.
use crate::package_repair_touches_only_its_own_worktree::is_inside_the_worktree;
/// Import the borrowed and owned filesystem paths a repair names.
use std::path::{
    Path,
    PathBuf,
};
/// Import the clock that times the two tool steps.
use std::time::Instant;

/// Tag used for diagnostics from this module.
pub(super) const TAG: &str = "packageRepairCargo";
/// What the Cargo program and the bounds every one of its runs takes:
/// found once,
/// and owned by the tasks that start it.
/// Why:
///   The repair's items are tasks of the runtime,
/// which must own what they read;
/// the program is one path and the two bounds are
///  copied values,
/// so each item holds them without a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CargoTool = { program: string; limits: { windowMs: number; ceilingMs: number } };
/// ```
#[derive(Clone)]
pub(super) struct CargoTool {
    /// The Cargo program found on the person's search path.
    pub(super) program: PathBuf,
    /// The quiet window and the ceiling of one run.
    pub(super) limits: ToolLimits,
}

/// What a repair of one workspace did.
pub(super) enum Done {
    /// Nothing was needed;
    /// the text says why.
    Skipped,
    /// The members were cleaned at the add's own location,
    /// and the build directory they were cleaned in was the worktree's;
    /// the line is what the person reads about the lock bytes the clean displaced,
    /// if any.
    Cleaned(Option<String>),
    /// The workspace's build directory is to be filled by the commit service,
    /// which also cleans the worktree's own crates out of the copy,
    /// so nothing was cleaned here.
    HandedOver(PrewarmEntry),
    /// The workspace could not be repaired.
    Failed(Finding),
}

/// What:
///  Repair one workspace:
/// plan,
/// hand the filling of its build directory over,
/// clean what is already here,
/// with every wait awaited on the runtime.
/// Why:
///   Each workspace's state is in a directory of its own,
/// so workspaces never touch each other's,
/// and the tool run is a wait a
///  task cannot block on.
/// A workspace whose build state is only in the source checkout is the service's:
/// the clone that fills it is the repair's heaviest step,
/// and the worktree is correct without it
/// (`package_repair_prewarm.rs`).
/// A workspace whose build directory holds state the copy carried is cleaned here and nowhere later:
/// Cargo would treat the copied artifacts of the worktree's own crates as fresh,
/// so a clean that ran after the add returned could be overtaken by a build that then reused them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairWorkspace(tool: CargoTool, workspace: Workspace, destinationRoot: string, source: string): Promise<Done>;
/// ```
pub(super) async fn repair_workspace_awaiting(
    tool: &CargoTool,
    workspace: &Workspace,
    destination_root: &Path,
    source: &Path,
    kept_in: Option<&Path>,
) -> Done {
    let manual: String = manual_command(workspace.root.as_path(), workspace.members.as_slice());
    // The repository's Git directory, whose own exclude file names the staging patterns: the staging sits beside the build
    //  directory it takes, inside the worktree, and without that the person's `git status` would show it.
    let staging_root: Option<PathBuf> = kept_in.map(Path::to_path_buf);
    let Ok(relative) = workspace.root.strip_prefix(destination_root) else {
        debug(
            TAG,
            format!(
                "{} lies outside {} and is left alone",
                workspace.root.display(),
                destination_root.display()
            )
            .as_str(),
        );
        return Done::Skipped;
    };
    let source_workspace: PathBuf = source.join(relative);
    let source_has_the_workspace: bool = source_workspace.join(MANIFEST).is_file();
    // The source's build directory is read only where the worktree's own is absent:
    // a worktree whose build directory holds state needs its own members cleaned and nothing cloned,
    // and reading the source there would be a workspace resolution with no answer the plan uses.
    let destination_build_exists: bool = workspace.build_directory.exists();
    let source_build: Option<PathBuf> = if !destination_build_exists && source_has_the_workspace {
        match timed_read_workspace_awaiting("cargoMetadataSource", tool, source_workspace.as_path())
            .await
        {
            Ok(found) => Some(found.build_directory),
            Err(reason) => {
                debug(
                    TAG,
                    format!(
                        "the source workspace {} cannot be read: {reason}",
                        source_workspace.display()
                    )
                    .as_str(),
                );
                None
            }
        }
    } else {
        None
    };
    let facts: StateFacts = StateFacts {
        source_has_the_workspace,
        source_build_exists: match source_build.as_ref() {
            Some(path) => path.exists(),
            None => false,
        },
        source_build,
        destination_build_exists,
        destination_build_is_inside: is_inside_the_worktree(
            destination_root,
            workspace.build_directory.as_path(),
        ),
        destination_target_is_inside: is_inside_the_worktree(
            destination_root,
            workspace.target_directory.as_path(),
        ),
    };
    let plan: BuildStatePlan = plan_build_state(workspace, destination_root, &facts);
    debug(
        TAG,
        format!("{}: {plan:?}", workspace.root.display()).as_str(),
    );
    match &plan {
        BuildStatePlan::Nothing(_) => return Done::Skipped,
        // The clone that fills this worktree's build directory is the service's work, and the same clean runs on the copy
        //  it publishes, so this workspace is done here.
        BuildStatePlan::FillFromTheSource {
            from,
            to,
        } => {
            return Done::HandedOver(PrewarmEntry {
                root: workspace.root.clone(),
                from: Some(from.clone()),
                to: to.clone(),
                members: workspace.members.clone(),
                staged_in: None,
            });
        }
        // The state the copy carried is moved aside here, before the add answers: one rename, no Cargo run, so no build can
        //  start on artifacts Cargo would call fresh and the add never waits for a clean. The staging name lies beside the
        //  build directory it takes, inside the worktree, because btrfs refuses to rename a directory out of a subvolume, and
        //  the repository's `info/exclude` names it so the person's `git status` and `git add -A` never see it.
        BuildStatePlan::MovedAside {
            to,
        } => {
            let staged: Result<crate::package_repair_prewarm_job::Staged, String> =
                match staging_root.as_ref() {
                    Some(root) => stage_the_worktrees_own_state(to.as_path(), Some(root.as_path())),
                    None => Err(String::from(
                        "the repair was handed no Git directory to stage the copied build state in",
                    )),
                };
            match staged {
                Ok(crate::package_repair_prewarm_job::Staged::Staged(staged_in)) => {
                    return Done::HandedOver(PrewarmEntry {
                        root: workspace.root.clone(),
                        from: None,
                        to: to.clone(),
                        members: workspace.members.clone(),
                        staged_in: Some(staged_in),
                    });
                }
                Ok(crate::package_repair_prewarm_job::Staged::Nothing) => return Done::Skipped,
                // The state could not be moved aside, and a build would reuse it where it stands, so this add cleans it here
                //  as every release before this one did. The wait is then the repair tool's own window, which this path keeps
                //  for the case a rename cannot happen at all, and the finding names the clean to run if it fails.
                Err(reason) => debug(
                    TAG,
                    format!("{reason}; the clean runs here instead").as_str(),
                ),
            }
        }
    }
    let command: ToolCommand = ToolCommand {
        arguments: clean_arguments(workspace.members.as_slice()),
        directory: workspace.root.clone(),
        set: Vec::new(),
    };
    let lock_before: Option<Vec<u8>> = read_lock(workspace.root.as_path());
    let run: ToolRun = timed_clean_awaiting(tool, &command).await;
    // The bytes the clean displaced are kept before the checked-out lock comes back, and the line says where they are
    //  (`package_repair_cargo_lock.rs`).
    let displaced: Option<String> =
        restore_lock(workspace.root.as_path(), lock_before.as_deref(), kept_in);
    return match run.end {
        ToolEnd::Succeeded => Done::Cleaned(displaced),
        ToolEnd::Failed(reason) => workspace_failed(manual.as_str(), reason),
        ToolEnd::TimedOut => workspace_failed(
            manual.as_str(),
            String::from("it showed no sign of work for the quiet window and was stopped"),
        ),
        ToolEnd::CeilingReached(ceiling) => workspace_failed(
            manual.as_str(),
            crate::package_repair_report::ceiling_reason(
                crate::package_repair_report::Tool::Cargo,
                ceiling,
            ),
        ),
    };
}

/// What:
///  The finding of a workspace whose repair failed:
/// the reason,
/// and the command the person can run instead.
/// Why:
///   Every failing step of `repair_workspace` ends the same way,
/// and a named function stands where a closure that
///  built this result stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const workspaceFailed = (manual: string, reason: string) => ({ kind: 'Failed', reason, runInstead: manual });
/// ```
fn workspace_failed(
    manual: &str,
    reason: String,
) -> Done {
    return Done::Failed(Finding {
        tool: Tool::Cargo,
        outcome: Outcome::Failed {
            reason,
            run_instead: manual.to_owned(),
        },
    });
}

/// What:
///  Read one workspace with `cargo metadata`,
/// timed as the step `name`.
/// Why:
///   The add timing record sums the work of the tasks by name;
/// the call runs in place between `begin_work` and `end_work`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork();
/// const workspace = await readWorkspaceAwaiting(cargo, directory, limits);
/// endWork(name, began);
/// ```
pub(super) async fn timed_read_workspace_awaiting(
    name: &str,
    tool: &CargoTool,
    directory: &Path,
) -> Result<Workspace, String> {
    let began: Instant = begin_work();
    let read: Result<Workspace, String> =
        read_workspace_awaiting(tool.program.as_path(), directory, tool.limits).await;
    end_work(name, began);
    return read;
}

/// What:
///  Run `cargo clean` with the timing record's step `cargoClean`.
/// Why:
///   Same as `timed_read_workspace_awaiting`:
/// the call runs in place between `begin_work` and `end_work`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork();
/// const run = await runToolAwaiting(cargo, command, limits);
/// endWork('cargoClean', began);
/// ```
async fn timed_clean_awaiting(
    tool: &CargoTool,
    command: &ToolCommand,
) -> ToolRun {
    let began: Instant = begin_work();
    let run: ToolRun = run_tool_awaiting(tool.program.as_path(), command, tool.limits).await;
    end_work("cargoClean", began);
    return run;
}
