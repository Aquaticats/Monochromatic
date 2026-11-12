//! What:
//!  The Cargo repair:
//! for each Cargo workspace of a new worktree,
//!  `cargo clean --package <member>` for every member so only the worktree's own crates fully rebuild,
//! and the hand-over of the filling of the worktree's build directory to the commit service.
//! Why:
//!  Cargo bakes absolute paths into what it builds (`CARGO_MANIFEST_DIR`,
//! build script output),
//! so build state copied from
//!  another location is wrong for the worktree's own crates,
//! but registry and git dependencies hold nothing path-specific
//!  and can be reused.
//! The owner chose this on 2026-11-03:
//! "Reuse deps,
//! rebuild own",
//! done with "Run cargo clean -p".
//!  Where `build-dir` is outside the worktree (the person's `~/.cargo/config.toml` keys it by workspace path) a copy of
//!  ignored files does not carry it,
//! so it was cloned.
//! The clone is the heaviest step of the repair:
//! a traced add spent 111 s of its 160 s repair phase inside one workspace's clone,
//! so it is no longer the add's work.
//! The add writes what the clone needs (the source's build directory,
//! the worktree's,
//! and the members) and hands it to the commit service,
//! which clones into a staging directory,
//! cleans the copy,
//! and publishes it only where the worktree still has no build directory of its own
//! (`package_repair_prewarm.rs`).
//!  Under Cargo's default layout the build state is inside `target/`,
//! which is already
//!  copied,
//! so only the clean runs.
//! `cargo metadata` names both directories,
//! so no configuration is read by the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const findings = await repairCargo(request); // one per workspace that failed, one for the rest that worked
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::diagnostic_log::debug;
use crate::lazy_runtime::run_on_runtime;
use crate::package_repair_request::RepairRequest;
use crate::package_repair_cargo_concurrent::{ItemLabel, map_concurrent_awaiting};
use crate::package_repair_cargo_lock::{read_lock, restore_lock};
use crate::package_repair_cargo_workspaces::{MANIFEST, Workspace, distinct_workspaces, manifest_directories_awaiting, read_workspace_awaiting};
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolRun, run_tool_awaiting};
use crate::package_repair_prewarm::{PrewarmEntry, common_directory_awaiting, hand_over_in_a_job};
/// Whether a directory a clean deletes from is still inside the worktree it serves.
use crate::package_repair_touches_only_its_own_worktree::is_inside_the_worktree;
/// The jobs of one workspace and what their runs take,
/// kept in a file of their own to stay under the file-size limit.
#[path = "package_repair_cargo_jobs.rs"]
mod jobs;
use self::jobs::{CargoTool, look_for_the_program, ReadWorkspaceJob, RepairWorkspaceJob};
use crate::package_repair_prewarm_job::stage_the_worktrees_own_state;
use crate::package_repair_report::{Finding, Outcome, Tool, missing_tool};
use crate::shell_quote::shell_quote;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairCargo";

/// The program the person's environment resolves.
const PROGRAM: &str = "cargo";

/// What:
///  What a workspace's build state needs in the new worktree.
/// Why:
///   The decision is pure so each layout is tested without Cargo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BuildStatePlan = { kind: 'nothing'; reason: string } | { kind: 'clean-only' } | { kind: 'clone-then-clean'; from: string; to: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuildStatePlan {
    /// Nothing is cloned or cleaned;
    /// the text says why,
    /// for the log.
    Nothing(String),
    /// The worktree holds build state of its own, copied with `target/`:
    /// the add moves it aside in one rename and the service cleans it there,
    /// so no build can trust the copy's artifacts and the add asks Cargo nothing.
    MovedAside {
        /// The worktree's build directory,
        /// which the add moves to the filling's staging name.
        to: PathBuf,
    },
    /// The source's external build directory is filled into the worktree's by the commit service,
    /// and the members are cleaned there,
    /// so the add does not wait for the clone.
    FillFromTheSource {
        /// The source workspace's build directory.
        from: PathBuf,
        /// The worktree's build directory,
        /// which does not exist yet.
        to: PathBuf,
    },
}

/// What:
///  Facts about the disk that decide a plan.
/// Why:
///   They are read once,
/// so the planner stays pure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StateFacts = { sourceHasTheWorkspace: boolean; sourceBuild?: string; sourceBuildExists: boolean; destinationBuildExists: boolean; destinationBuildIsInside: boolean; destinationTargetIsInside: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateFacts {
    /// Whether the source checkout has a counterpart workspace at all;
    /// one walk of the tracked manifests answers it,
    /// without asking Cargo anything.
    pub source_has_the_workspace: bool,
    /// The build directory of the source's counterpart workspace;
    /// read only where the worktree's own build directory is absent,
    /// because only a fill uses it.
    pub source_build: Option<PathBuf>,
    /// Whether that directory exists.
    pub source_build_exists: bool,
    /// Whether the worktree's build directory exists.
    pub destination_build_exists: bool,
    /// Whether the worktree's build directory is still inside the worktree once its links are followed.
    pub destination_build_is_inside: bool,
    /// Whether the worktree's target directory is still inside the worktree once its links are followed.
    pub destination_target_is_inside: bool,
}

/// What:
///  Decide what the worktree's build state needs.
/// Why:
///   Reusing state that is shared with the source,
/// or cleaning where there is none,
/// would do harm or nothing.
/// A clean runs only where the worktree's own build directory holds state a build could reuse:
/// Cargo keeps the fingerprints that make artifacts fresh inside that directory,
/// so a worktree whose build directory is absent rebuilds its own crates by itself,
/// which was measured on 2026-11-12 with a copy of a built workspace whose `target/` held the
///  source's binary (the copy's first build rebuilt it and named the copy's own path).
/// A copied `target/` alone is therefore no reason to clean,
/// and on the host whose Cargo keeps the build directory outside the worktree it made every one of
///  an add's 34 cleans a full workspace resolution that deleted nothing
/// (`doc/planning/cli-git-worktree-add-package-repair-off-the-waiting-path.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planBuildState(destination: Workspace, destinationRoot: string, facts: StateFacts): BuildStatePlan;
/// ```
pub fn plan_build_state(destination: &Workspace, destination_root: &Path, facts: &StateFacts) -> BuildStatePlan {
    if destination.build_directory.starts_with(destination_root) {
        // A copied `target/` can be a link to a directory outside the worktree, and a clean through it would delete the
        //  files of the checkout that link names, which this repair does not serve.
        if !facts.destination_build_is_inside || !facts.destination_target_is_inside {
            return BuildStatePlan::Nothing(String::from("the build state is reached through a link that leaves the worktree, and a clean there would delete another directory's files"));
        }
        // What a clean here would remove is the copy's state for the worktree's own crates, so nothing was carried when the
        //  source has no counterpart workspace.
        if !facts.source_has_the_workspace {
            return BuildStatePlan::Nothing(String::from("the source has no such workspace, so no state was carried"));
        }
        return if facts.destination_build_exists {
            BuildStatePlan::MovedAside { to: destination.build_directory.clone() }
        } else {
            BuildStatePlan::Nothing(String::from("the build directory is inside the worktree and was not carried"))
        };
    }
    if facts.destination_build_exists {
        return BuildStatePlan::Nothing(String::from("the worktree's build directory already holds state of its own"));
    }
    let Some(source_build) = facts.source_build.as_ref() else {
        return BuildStatePlan::Nothing(String::from("the source holds no build directory this repair could read, so there is nothing to fill"));
    };
    if *source_build == destination.build_directory {
        return BuildStatePlan::Nothing(String::from("the source and the worktree share one build directory, which a clean would damage"));
    }
    if facts.source_build_exists {
        return BuildStatePlan::FillFromTheSource { from: source_build.clone(), to: destination.build_directory.clone() };
    }
    return BuildStatePlan::Nothing(String::from(
        "the worktree's build directory does not exist and the source holds none for it, so a clean would reach nothing a build could reuse",
    ));
}

/// What:
///  The arguments of the clean that removes exactly the members' own artifacts,
/// for the add's own clean.
/// Why:
///   `--offline` keeps the clean from reaching the network;
/// one `--package` per member keeps registry and git dependencies.
///  `--locked` is not used here:
/// a committed `Cargo.lock` that no longer matches its manifest (several do here) would make the clean
///  fail,
/// and the person's own clean would not;
/// the file is put back as checked out instead (`restore_lock`),
/// which `SPEC.md` records as the reason this stands where the worktree was just made and nobody
/// has it yet.
/// The filling of a build directory after the add has returned passes `--locked` instead
/// (`package_repair_prewarm_job.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanArguments(members: string[]): string[];
/// ```
pub fn clean_arguments(members: &[String]) -> Vec<OsString> {
    return clean_arguments_of(members, false);
}

/// What:
///  The arguments of the clean the commit service runs on a copy of a worktree's build state:
/// the same clean,
/// with `--locked`.
/// Why:
///   That clean runs after the add returned,
///  when the person may be working in the worktree,
/// and `--locked` keeps Cargo from rewriting the worktree's tracked `Cargo.lock` at all,
/// so the person's own file is never in the way of the clean.
/// A lock that no longer matches its manifest makes such a clean refuse,
/// and its caller asks Cargo to resolve the workspace first (`package_repair_prewarm_job.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockedCleanArguments(members: string[]): string[];
/// ```
pub fn locked_clean_arguments(members: &[String]) -> Vec<OsString> {
    return clean_arguments_of(members, true);
}

/// What:
///  The clean's arguments,
/// with `--locked` when the caller's lock must not be rewritten.
/// Why:
///   One list builds both faces,
/// so the members and the switches each clean names can never drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleanArgumentsOf = (members, locked) => ['clean', '--offline', ...(locked ? ['--locked'] : []), ...members.flatMap((m) => ['--package', m])];
/// ```
fn clean_arguments_of(members: &[String], locked: bool) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = ["clean", "--offline"].iter().map(OsString::from).collect();
    if locked {
        arguments.push(OsString::from("--locked"));
    }
    for member in members {
        arguments.push(OsString::from("--package"));
        arguments.push(OsString::from(member));
    }
    return arguments;
}

/// What:
///  The command a person runs in a workspace to do what the repair could not.
/// Why:
///   A printed line must work as pasted,
/// so the directory change is part of it,
/// and the filling of a build directory reports the same command the add's repair does
/// (`package_repair_prewarm_job.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function manualCommand(root: string, members: string[]): string;
/// ```
pub fn manual_command(root: &Path, members: &[String]) -> String {
    let mut words: Vec<String> = Vec::new();
    for word in clean_arguments(members) {
        words.push(shell_quote(word.as_encoded_bytes()));
    }
    return format!("cd {} && {PROGRAM} {}", shell_quote(root.as_os_str().as_encoded_bytes()), words.join(" "));
}

/// What a repair of one workspace did.
enum Done {
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
async fn repair_workspace_awaiting(tool: &CargoTool, workspace: &Workspace, destination_root: &Path, source: &Path, kept_in: Option<&Path>) -> Done {
    let manual: String = manual_command(workspace.root.as_path(), workspace.members.as_slice());
    // The repository's Git directory, whose own exclude file names the staging patterns: the staging sits beside the build
    //  directory it takes, inside the worktree, and without that the person's `git status` would show it.
    let staging_root: Option<PathBuf> = kept_in.map(Path::to_path_buf);
    let Ok(relative) = workspace.root.strip_prefix(destination_root) else {
        debug(TAG, format!("{} lies outside {} and is left alone", workspace.root.display(), destination_root.display()).as_str());
        return Done::Skipped;
    };
    let source_workspace: PathBuf = source.join(relative);
    let source_has_the_workspace: bool = source_workspace.join(MANIFEST).is_file();
    // The source's build directory is read only where the worktree's own is absent:
    // a worktree whose build directory holds state needs its own members cleaned and nothing cloned,
    // and reading the source there would be a workspace resolution with no answer the plan uses.
    let destination_build_exists: bool = workspace.build_directory.exists();
    let source_build: Option<PathBuf> = if !destination_build_exists && source_has_the_workspace {
        match timed_read_workspace_awaiting("cargoMetadataSource", tool, source_workspace.as_path()).await {
            Ok(found) => Some(found.build_directory),
            Err(reason) => {
                debug(TAG, format!("the source workspace {} cannot be read: {reason}", source_workspace.display()).as_str());
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
        destination_build_is_inside: is_inside_the_worktree(destination_root, workspace.build_directory.as_path()),
        destination_target_is_inside: is_inside_the_worktree(destination_root, workspace.target_directory.as_path()),
    };
    let plan: BuildStatePlan = plan_build_state(workspace, destination_root, &facts);
    debug(TAG, format!("{}: {plan:?}", workspace.root.display()).as_str());
    match &plan {
        BuildStatePlan::Nothing(_) => return Done::Skipped,
        // The clone that fills this worktree's build directory is the service's work, and the same clean runs on the copy
        //  it publishes, so this workspace is done here.
        BuildStatePlan::FillFromTheSource { from, to } => {
            return Done::HandedOver(PrewarmEntry { root: workspace.root.clone(), from: Some(from.clone()), to: to.clone(), members: workspace.members.clone(), staged_in: None });
        }
        // The state the copy carried is moved aside here, before the add answers: one rename, no Cargo run, so no build can
        //  start on artifacts Cargo would call fresh and the add never waits for a clean. The staging name lies under the
        //  repository's Git directory, never in the worktree, which the person reads with `git status` the moment it exists.
        BuildStatePlan::MovedAside { to } => {
            let staged: Result<crate::package_repair_prewarm_job::Staged, String> = match staging_root.as_ref() {
                Some(root) => stage_the_worktrees_own_state(to.as_path(), Some(root.as_path())),
                None => Err(String::from("the repair was handed no Git directory to stage the copied build state in")),
            };
            match staged {
                Ok(crate::package_repair_prewarm_job::Staged::Staged(staged_in)) => {
                    return Done::HandedOver(PrewarmEntry { root: workspace.root.clone(), from: None, to: to.clone(), members: workspace.members.clone(), staged_in: Some(staged_in) });
                }
                Ok(crate::package_repair_prewarm_job::Staged::Nothing) => return Done::Skipped,
                // The state could not be moved aside, and a build would reuse it where it stands, so this add cleans it here
                //  as every release before this one did. The wait is then the repair tool's own window, which this path keeps
                //  for the case a rename cannot happen at all, and the finding names the clean to run if it fails.
                Err(reason) => {
                    eprintln!("PROBE staging refused: {reason}");
                    debug(TAG, format!("{reason}; the clean runs here instead").as_str())
                }
            }
        }
    }
    let command: ToolCommand = ToolCommand { arguments: clean_arguments(workspace.members.as_slice()), directory: workspace.root.clone(), set: Vec::new() };
    let lock_before: Option<Vec<u8>> = read_lock(workspace.root.as_path());
    let run: ToolRun = timed_clean_awaiting(tool, &command).await;
    // The bytes the clean displaced are kept before the checked-out lock comes back, and the line says where they are
    //  (`package_repair_cargo_lock.rs`).
    let displaced: Option<String> = restore_lock(workspace.root.as_path(), lock_before.as_deref(), kept_in);
    return match run.end {
        ToolEnd::Succeeded => Done::Cleaned(displaced),
        ToolEnd::Failed(reason) => workspace_failed(manual.as_str(), reason),
        ToolEnd::TimedOut => workspace_failed(manual.as_str(), String::from("it showed no sign of work for the quiet window and was stopped")),
        ToolEnd::CeilingReached(ceiling) => workspace_failed(manual.as_str(), crate::package_repair_report::ceiling_reason(crate::package_repair_report::Tool::Cargo, ceiling)),
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
fn workspace_failed(manual: &str, reason: String) -> Done {
    return Done::Failed(Finding { tool: Tool::Cargo, outcome: Outcome::Failed { reason, run_instead: manual.to_owned() } });
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
async fn timed_read_workspace_awaiting(name: &str, tool: &CargoTool, directory: &Path) -> Result<Workspace, String> {
    let began: Instant = begin_work();
    let read: Result<Workspace, String> = read_workspace_awaiting(tool.program.as_path(), directory, tool.limits).await;
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
async fn timed_clean_awaiting(tool: &CargoTool, command: &ToolCommand) -> ToolRun {
    let began: Instant = begin_work();
    let run: ToolRun = run_tool_awaiting(tool.program.as_path(), command, tool.limits).await;
    end_work("cargoClean", began);
    return run;
}

/// The label of a metadata read whose item panicked:
/// the directory it was reading.
struct ReadingLabel;

impl ItemLabel<PathBuf> for ReadingLabel {
    fn label(&self, directory: &PathBuf) -> String {
        return format!("reading the Cargo workspace at {}", directory.display());
    }
}

/// The label of a workspace repair whose item panicked:
/// the workspace it was repairing.
struct RepairLabel;

impl ItemLabel<Workspace> for RepairLabel {
    fn label(&self, workspace: &Workspace) -> String {
        return format!("the repair of the Cargo workspace at {}", workspace.root.display());
    }
}

/// What:
///  Repair every Cargo workspace of the new worktree on the runtime.
/// Why:
///   Workspaces are independent (each has its own build directory),
/// so each is a lane of one awaited ordered run,
/// and the
///  tracked manifests are listed by a Git child awaited on the same runtime.
/// The program lookup and the worktree's real path
///  are one job of the blocking pool,
/// because they read the search path and resolve a link,
/// which are storage waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairCargoAwaiting(request: RepairRequest): Promise<Finding[]>;
/// ```
pub async fn repair_cargo_awaiting(request: &RepairRequest<'_>) -> Vec<Finding> {
    let started: Instant = Instant::now();
    let directories: Vec<PathBuf> = match manifest_directories_awaiting(request.git, request.destination).await {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("{}: {reason}", request.destination.display()).as_str());
            return Vec::new();
        }
    };
    if directories.is_empty() {
        return Vec::new();
    }
    let search_path: Option<OsString> = request.environment.search_path.clone();
    let looked = run_a_job_or_in_turn((search_path, request.destination.to_path_buf()), look_for_the_program).await;
    let (found_program, real_destination): (Option<PathBuf>, Result<PathBuf, std::io::Error>) = match looked {
        Ok(found) => found,
        // Which workspaces need what was not learnt, so the repair says what it could not do and names the command.
        Err(refused) => {
            return vec![Finding { tool: Tool::Cargo, outcome: Outcome::Failed { reason: format!("the Cargo repair could not be prepared ({refused})"), run_instead: manual_command(request.destination, &[String::from("<member>")]) } }];
        }
    };
    let Some(cargo) = found_program else {
        return vec![missing_tool(Tool::Cargo, PROGRAM, "cargo clean --offline --package <member> in each Cargo workspace of the worktree")];
    };
    // The repository's common Git directory, asked once for the whole lane: a moved-aside build state waits under it, out of the
    //  worktree, and the lock bytes a clean displaces are kept under it too.
    let common_dir: Option<PathBuf> = common_directory_awaiting(request.git, request.destination).await.ok();
    let workers: usize = request.environment.workers;
    let tool: CargoTool = CargoTool { program: cargo, limits: request.environment.limits() };
    let listed: Arc<[PathBuf]> = Arc::from(directories);
    let mut read: Vec<Result<Workspace, String>> = Vec::new();
    for outcome in map_concurrent_awaiting(Arc::clone(&listed), workers, Arc::new(ReadingLabel), Arc::new(ReadWorkspaceJob { tool: tool.clone() })).await {
        // A panic in the read and a failed read are both a failed read.
        read.push(match outcome {
            Ok(inner) => inner,
            Err(reason) => Err(reason),
        });
    }
    let mut findings: Vec<Finding> = Vec::new();
    let mut found: Vec<Workspace> = Vec::new();
    for (directory, outcome) in listed.iter().zip(read) {
        // A panic in the read and a failed read are both a failed read.
        match outcome {
            Ok(workspace) => found.push(workspace),
            Err(reason) => findings.push(Finding { tool: Tool::Cargo, outcome: Outcome::Failed { reason, run_instead: manual_command(directory.as_path(), &[String::from("<member>")]) } }),
        }
    }
    let workspaces: Arc<[Workspace]> = Arc::from(distinct_workspaces(found));
    // Real paths: Cargo names the workspace by its real path, and the worktree may be reached through a link.
    let destination_root: PathBuf = match &real_destination {
        Ok(real) => real.clone(),
        // A worktree whose real path cannot be read is used as the caller named it.
        Err(_cannot_be_resolved) => request.destination.to_path_buf(),
    };
    let repairer: RepairWorkspaceJob = RepairWorkspaceJob { tool, destination_root, source: request.source.to_path_buf(), kept_in: common_dir.clone() };
    let repaired: Vec<Result<Done, String>> =
        map_concurrent_awaiting(Arc::clone(&workspaces), workers, Arc::new(RepairLabel), Arc::new(repairer)).await;
    let mut done: Vec<Done> = Vec::new();
    for (outcome, workspace) in repaired.into_iter().zip(workspaces.iter()) {
        done.push(match outcome {
            Ok(finished) => finished,
            Err(reason) => Done::Failed(Finding { tool: Tool::Cargo, outcome: Outcome::Failed { reason, run_instead: manual_command(workspace.root.as_path(), &[String::from("<member>")]) } }),
        });
    }
    let mut cleaned: usize = 0;
    let mut displaced: Vec<String> = Vec::new();
    let mut handed_over: Vec<PrewarmEntry> = Vec::new();
    for outcome in done {
        match outcome {
            Done::Skipped => {}
            Done::Cleaned(said) => {
                cleaned += 1;
                if let Some(line) = said {
                    displaced.push(line);
                }
            }
            Done::HandedOver(entry) => handed_over.push(entry),
            Done::Failed(finding) => findings.push(finding),
        }
    }
    if cleaned > 0 {
        let workspace_noun: &str = if cleaned == 1 { "workspace" } else { "workspaces" };
        // A clean that displaced a lock keeps those bytes, and the line says where they are: a worktree that is clean again may
        //  still hold a command's newer lock under its Git directory (`package_repair_cargo_lock.rs`).
        let kept: String = if displaced.is_empty() { String::new() } else { format!("; {}", displaced.join("; ")) };
        findings.push(Finding { tool: Tool::Cargo, outcome: Outcome::Repaired { detail: format!("Cargo kept dependency builds and cleaned the worktree's own crates in {cleaned} {workspace_noun}{kept}"), millis: started.elapsed().as_millis() } });
    }
    if !handed_over.is_empty() {
        findings.push(hand_the_filling_over(request, handed_over.as_slice(), common_dir.as_deref()).await);
    }
    return findings;
}

/// What:
///  Hand the filling of the worktree's build directories to the commit service,
/// and answer the line the person reads.
/// Why:
///   The record is written before the service is reached,
/// so a wake that cannot reach it leaves the work named rather than lost,
/// and the add never fails for it:
/// the worktree is correct without the filling and only its first build is slower.
/// The writing of a record and the wake that may wait for a service to start are storage and socket
///  work,
/// so they run as a job of the blocking pool and never on the task that polls the three repairs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handedOver = async (request, entries) => { const told = await handOverWithinTheBudget(request.git, await commonDirectory(request.destination), request.destination, entries); return repaired(told); };
/// ```
async fn hand_the_filling_over(request: &RepairRequest<'_>, entries: &[PrewarmEntry], common_dir: Option<&Path>) -> Finding {
    let began: Instant = begin_work();
    // Where a person watches what became of the work this add leaves behind: the queue's own states, and the service's journal.
    let the_queues_states: Option<String> = common_dir.map(|common| crate::service_work::work_directory(common).display().to_string());
    let told: Result<Result<(), String>, BlockingJobError> = match common_dir {
        Some(common) => {
            let state: (crate::transaction_git::GitContext, PathBuf, PathBuf, Vec<PrewarmEntry>) =
                (request.git.clone(), common.to_path_buf(), request.destination.to_path_buf(), entries.to_vec());
            run_a_job_or_in_turn(state, hand_over_in_a_job).await
        }
        None => Ok(Err(String::from("git could not name the repository's common directory"))),
    };
    end_work("cargoPrewarmHandOver", began);
    let staged: usize = entries.iter().filter(|entry| entry.staged_in.is_some()).count();
    let workspace_noun: &str = if entries.len() == 1 { "build directory" } else { "build directories" };
    let them: &str = if entries.len() == 1 { "it" } else { "them" };
    let queue: String = match the_queues_states {
        Some(directory) => format!("`{directory}/{{pending,claimed,done,failed}}` and the service's journal"),
        // No queue directory could be named, so the line says only what happens; a command naming a path that is not there would
        //  be worse than none.
        None => String::from("the commit service's journal"),
    };
    // A workspace whose own copied state the add moved aside is named apart: what waits for the service there is a clean of
    //  state the worktree no longer holds, and a build started at once compiles it itself.
    let moved: String = if staged == 0 {
        String::new()
    } else {
        format!(
            ", and the cleaning of the {staged} {} the copy carried, moved aside before this command answered so that no build trusts {}",
            if staged == 1 { "build state" } else { "build states" },
            if staged == 1 { "it" } else { "them" }
        )
    };
    let detail: String = match told {
        Ok(Ok(())) => format!(
            "Cargo kept dependency builds and handed the filling of {} {workspace_noun} to the commit service{moved}, whose progress shows in {queue}, and which fills or cleans {them} after this command returns",
            entries.len()
        ),
        // The record stays and the service fills it when it next runs for this repository, so this is a line of the report and
        //  never the add's failure: the worktree is whole either way and its first build only compiles more.
        Ok(Err(reason)) => format!(
            "Cargo kept dependency builds and wrote the record that fills {} {workspace_noun}{moved}, which it could not hand to the commit service ({reason}); the worktree's first build compiles what it needs itself, and the record waits in {queue} until a service runs for this repository",
            entries.len()
        ),
        Err(refusal) => format!(
            "Cargo kept dependency builds and could not write the record that fills {} {workspace_noun}{moved} ({refusal}); the worktree's first build compiles what it needs itself, and nothing was left for a service to pick up",
            entries.len()
        ),
    };
    return Finding { tool: Tool::Cargo, outcome: Outcome::Repaired { detail, millis: began.elapsed().as_millis() } };
}

/// What:
///  Repair every Cargo workspace of the new worktree.
/// Why:
///   Workspaces are independent (each has its own build directory),
/// so they are repaired side by side.
/// The callers of this face are synchronous (the repair the add runs),
/// so it enters the process's runtime once and waits there;
/// from inside the runtime it refuses,
/// and the lanes of the worktree repair await `repair_cargo_awaiting`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repairCargo(request: RepairRequest): Finding[];
/// ```
pub fn repair_cargo(request: &RepairRequest<'_>) -> Vec<Finding> {
    return match run_on_runtime(repair_cargo_awaiting(request)) {
        Ok(findings) => findings,
        // The tracked manifests cannot be listed without the runtime, and the repair says so in the log alone, as it did.
        Err(refusal) => {
            debug(TAG, format!("{}: git could not list the tracked manifests ({refusal})", request.destination.display()).as_str());
            Vec::new()
        }
    };
}

/// Cargo repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_cargo_tests.rs"]
mod tests;
