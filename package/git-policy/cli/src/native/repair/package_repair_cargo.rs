//! What:
//!  The Cargo repair:
//! for each Cargo workspace of a new worktree,
//! clone the source workspace's build state when Cargo keeps
//!  it outside the worktree,
//! then `cargo clean --package <member>` for every member so only the worktree's own crates
//!  rebuild.
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
//! so it is cloned;
//! under Cargo's default layout it is inside `target/`,
//! which is already
//!  copied.
//! `cargo metadata` names both directories,
//! so no configuration is read by the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const findings = await repairCargo(request); // one per workspace that failed, one for the rest that worked
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::diagnostic_log::debug;
use crate::lazy_runtime::run_on_runtime;
use crate::package_repair_request::RepairRequest;
use crate::package_repair_cargo_concurrent::{ItemLabel, map_concurrent_awaiting};
use crate::package_repair_cargo_lock::{read_lock, restore_lock};
use crate::package_repair_cargo_workspaces::{MANIFEST, Workspace, distinct_workspaces, manifest_directories_awaiting, read_workspace_awaiting};
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolRun, run_tool_awaiting};
/// The jobs of one workspace and what their runs take,
/// kept in a file of their own to stay under the file-size limit.
#[path = "package_repair_cargo_jobs.rs"]
mod jobs;
use self::jobs::{CargoTool, look_for_the_program, ReadWorkspaceJob, RepairWorkspaceJob};
use crate::package_repair_report::{Finding, Outcome, Tool, missing_tool};
use crate::package_repair_tree_clone::clone_tree_awaiting;
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
    /// The state is already in the worktree (copied with `target/`):
    /// only the members are cleaned.
    CleanOnly,
    /// The source's external build directory is cloned to the worktree's,
    /// then the members are cleaned.
    CloneThenClean {
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
/// type StateFacts = { sourceBuild?: string; sourceBuildExists: boolean; destinationBuildExists: boolean; destinationTargetExists: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateFacts {
    /// The build directory of the source's counterpart workspace;
    /// none when the source has no such workspace.
    pub source_build: Option<PathBuf>,
    /// Whether that directory exists.
    pub source_build_exists: bool,
    /// Whether the worktree's build directory exists.
    pub destination_build_exists: bool,
    /// Whether the worktree's target directory exists.
    pub destination_target_exists: bool,
}

/// What:
///  Decide what the worktree's build state needs.
/// Why:
///   Reusing state that is shared with the source,
/// or cleaning where there is none,
/// would do harm or nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planBuildState(destination: Workspace, destinationRoot: string, facts: StateFacts): BuildStatePlan;
/// ```
pub fn plan_build_state(destination: &Workspace, destination_root: &Path, facts: &StateFacts) -> BuildStatePlan {
    let Some(source_build) = facts.source_build.as_ref() else {
        return BuildStatePlan::Nothing(String::from("the source has no such workspace, so no state was carried"));
    };
    if *source_build == destination.build_directory {
        return BuildStatePlan::Nothing(String::from("the source and the worktree share one build directory, which a clean would damage"));
    }
    if destination.build_directory.starts_with(destination_root) {
        return if facts.destination_build_exists || facts.destination_target_exists {
            BuildStatePlan::CleanOnly
        } else {
            BuildStatePlan::Nothing(String::from("the build directory is inside the worktree and was not carried"))
        };
    }
    if facts.destination_build_exists {
        return BuildStatePlan::Nothing(String::from("the worktree's build directory already holds state of its own"));
    }
    if facts.source_build_exists {
        return BuildStatePlan::CloneThenClean { from: source_build.clone(), to: destination.build_directory.clone() };
    }
    return if facts.destination_target_exists { BuildStatePlan::CleanOnly } else { BuildStatePlan::Nothing(String::from("neither side holds build state")) };
}

/// What:
///  The arguments of the clean that removes exactly the members' own artifacts.
/// Why:
///   `--offline` keeps the clean from reaching the network;
/// one `--package` per member keeps registry and git dependencies.
///  `--locked` is not used:
/// a committed `Cargo.lock` that no longer matches its manifest (several do here) would make the clean
///  fail,
/// and the person's own clean would not;
/// the file is put back as checked out instead (`restore_lock`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanArguments(members: string[]): string[];
/// ```
pub fn clean_arguments(members: &[String]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = ["clean", "--offline"].iter().map(OsString::from).collect();
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
/// so the directory change is part of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function manualCommand(root: string, members: string[]): string;
/// ```
fn manual_command(root: &Path, members: &[String]) -> String {
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
    /// The members were cleaned,
    /// and the build directory was cloned first when `cloned` is set.
    Cleaned {
        /// Whether a build directory was cloned.
        cloned: bool,
        /// How many entries of the source's build directory could not be read and were left out of the clone.
        left_out: usize,
    },
    /// The workspace could not be repaired.
    Failed(Finding),
}

/// What:
///  Repair one workspace:
/// plan,
/// clone,
/// clean,
/// with every wait awaited on the runtime.
/// Why:
///   Each workspace's state is in a directory of its own,
/// so workspaces never touch each other's,
/// and the clone and the two tool runs are waits a
///  task cannot block on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairWorkspace(tool: CargoTool, workspace: Workspace, destinationRoot: string, source: string, cloneWorkers: number): Promise<Done>;
/// ```
async fn repair_workspace_awaiting(tool: &CargoTool, workspace: &Workspace, destination_root: &Path, source: &Path, clone_workers: usize) -> Done {
    let manual: String = manual_command(workspace.root.as_path(), workspace.members.as_slice());
    let Ok(relative) = workspace.root.strip_prefix(destination_root) else {
        debug(TAG, format!("{} lies outside {} and is left alone", workspace.root.display(), destination_root.display()).as_str());
        return Done::Skipped;
    };
    let source_workspace: PathBuf = source.join(relative);
    let source_build: Option<PathBuf> = if source_workspace.join(MANIFEST).is_file() {
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
        source_build_exists: match source_build.as_ref() {
            Some(path) => path.exists(),
            None => false,
        },
        source_build,
        destination_build_exists: workspace.build_directory.exists(),
        destination_target_exists: workspace.target_directory.exists(),
    };
    let plan: BuildStatePlan = plan_build_state(workspace, destination_root, &facts);
    debug(TAG, format!("{}: {plan:?}", workspace.root.display()).as_str());
    let (cloned, left_out): (bool, usize) = match &plan {
        BuildStatePlan::Nothing(_) => return Done::Skipped,
        BuildStatePlan::CleanOnly => (false, 0),
        BuildStatePlan::CloneThenClean { from, to } => {
            match timed_clone_tree_awaiting(from.as_path(), to.as_path(), clone_workers).await {
                Ok(made) => {
                    debug(TAG, format!("cloned {} entries from {} to {}", made.made, from.display(), to.display()).as_str());
                    for path in &made.left_out {
                        debug(TAG, format!("{} could not be read and is not in the clone", from.join(path).display()).as_str());
                    }
                    (true, made.left_out.len())
                }
                Err(reason) => return workspace_failed(manual.as_str(), format!("the build state could not be cloned: {reason}")),
            }
        }
    };
    let command: ToolCommand = ToolCommand { arguments: clean_arguments(workspace.members.as_slice()), directory: workspace.root.clone(), set: Vec::new() };
    let lock_before: Option<Vec<u8>> = read_lock(workspace.root.as_path());
    let run: ToolRun = timed_clean_awaiting(tool, &command).await;
    restore_lock(workspace.root.as_path(), lock_before.as_deref());
    return match run.end {
        ToolEnd::Succeeded => Done::Cleaned { cloned, left_out },
        ToolEnd::Failed(reason) => workspace_failed(manual.as_str(), reason),
        ToolEnd::TimedOut => workspace_failed(manual.as_str(), String::from("it showed no sign of work for the quiet window and was stopped")),
        ToolEnd::CeilingReached(ceiling) => workspace_failed(manual.as_str(), crate::package_repair_report::ceiling_reason(crate::package_repair_report::Tool::Cargo, ceiling)),
    };
}

/// What:
///  The words that say how many entries of the cloned build state could not be read,
/// or nothing when all could.
/// Why:
///   The person is told what the clone left out:
/// the entries are the source's,
/// unreadable to everyone but a chmod the repair
///  never performs,
/// and Cargo rebuilds whatever of them it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leftOutWords(2) // '; 2 unreadable entries were left out'
/// ```
fn left_out_words(left_out: usize) -> String {
    return match left_out {
        0 => String::new(),
        1 => String::from("; 1 unreadable entry was left out"),
        many => format!("; {many} unreadable entries were left out"),
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
///  Clone a build directory with the timing record's step `cargoCloneTree`.
/// Why:
///   Same as `timed_read_workspace_awaiting`:
/// the call runs in place between `begin_work` and `end_work`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = beginWork();
/// const made = await cloneTreeAwaiting(from, to, workers);
/// endWork('cargoCloneTree', began);
/// ```
async fn timed_clone_tree_awaiting(from: &Path, to: &Path, clone_workers: usize) -> Result<crate::package_repair_tree_clone::Cloned, String> {
    let began: Instant = begin_work();
    let made: Result<crate::package_repair_tree_clone::Cloned, String> = clone_tree_awaiting(from, to, clone_workers).await;
    end_work("cargoCloneTree", began);
    return made;
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
    let clone_workers: usize = if workspaces.len() > 1 { 1 } else { workers };
    let repairer: RepairWorkspaceJob =
        RepairWorkspaceJob { tool, destination_root, source: request.source.to_path_buf(), clone_workers };
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
    let mut cloned: usize = 0;
    let mut left_out: usize = 0;
    for outcome in done {
        match outcome {
            Done::Skipped => {}
            Done::Cleaned { cloned: was_cloned, left_out: unreadable } => {
                cleaned += 1;
                cloned += usize::from(was_cloned);
                left_out += unreadable;
            }
            Done::Failed(finding) => findings.push(finding),
        }
    }
    if cleaned > 0 {
        let workspace_noun: &str = if cleaned == 1 { "workspace" } else { "workspaces" };
        let directory_noun: &str = if cloned == 1 { "build directory" } else { "build directories" };
        findings.push(Finding { tool: Tool::Cargo, outcome: Outcome::Repaired { detail: format!("Cargo kept dependency builds and cleaned the worktree's own crates in {cleaned} {workspace_noun} ({cloned} {directory_noun} cloned{})", left_out_words(left_out)), millis: started.elapsed().as_millis() } });
    }
    return findings;
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
