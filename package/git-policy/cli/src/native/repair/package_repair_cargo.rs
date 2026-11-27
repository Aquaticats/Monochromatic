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
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_cargo_build_state_plan.rs`:
//!   what one workspace's build state needs in the new worktree,
//!   decided from what the disk holds.
//! - `package_repair_cargo_clean_arguments.rs`:
//!   the arguments of the clean,
//!   and the command a person can run instead.
//! - `package_repair_cargo_one_workspace.rs`:
//!   the repair of one workspace:
//!   plan,
//!   hand the filling over,
//!   clean what is already here.

/// Import the runner that does a blocking job on the pool or in this thread.
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the process's runtime,
/// which the synchronous face enters.
use crate::lazy_runtime::run_on_runtime;
/// Import the label of an item and the runner that does one job for every item.
use crate::package_repair_cargo_concurrent::{
    ItemLabel,
    map_concurrent_awaiting,
};
/// Import the Cargo workspaces of a worktree and the listing of their manifests.
use crate::package_repair_cargo_workspaces::{
    Workspace,
    distinct_workspaces,
    manifest_directories_awaiting,
};
/// Import what the commit service is asked to fill,
/// and the directory one Git answers with.
use crate::package_repair_prewarm::{
    PrewarmEntry,
    common_directory_awaiting,
};
/// Import what a repair reports and the line of a tool that is not there.
use crate::package_repair_report::{
    Finding,
    Outcome,
    Tool,
    missing_tool,
};
/// Import what a repair reads and is asked.
use crate::package_repair_request::RepairRequest;
/// Import the jobs of one workspace and what their runs take,
/// kept in a file of their own to stay under the file-size limit.
#[path = "package_repair_cargo_jobs.rs"]
mod jobs;
use self::jobs::{
    ReadWorkspaceJob,
    RepairWorkspaceJob,
    look_for_the_program,
};
/// Import the hand-over of the filling to the commit service and the line the add answers for it,
/// kept in a file of their own to stay under the file-size limit.
#[path = "package_repair_cargo_hand_over.rs"]
mod hand_over;
use self::hand_over::hand_the_filling_over;
/// Import the plan of what a workspace's build state needs.
pub use crate::package_repair_cargo_build_state_plan::{
    BuildStatePlan,
    StateFacts,
    plan_build_state,
};
/// Import the program the person's environment resolves.
use crate::package_repair_cargo_clean_arguments::PROGRAM;
/// Import the clean's arguments and the command a person runs instead,
/// so the paths their callers already name are unchanged.
pub use crate::package_repair_cargo_clean_arguments::{
    clean_arguments,
    locked_clean_arguments,
    manual_command,
};
/// Import the tag every line of this repair's diagnostic log carries.
use crate::package_repair_cargo_one_workspace::TAG;
/// Import the repair of one workspace and what it came to,
/// so the paths the jobs,
/// the controls and this file already name are unchanged.
pub(super) use crate::package_repair_cargo_one_workspace::{
    CargoTool,
    Done,
    repair_workspace_awaiting,
    timed_read_workspace_awaiting,
};
/// Import the owned,
/// platform-native string a search path is.
use std::ffi::OsString;
/// Import the owned filesystem path of the worktree a repair is asked about.
use std::path::PathBuf;
/// Import the shared list of workspaces the runner reads from several items.
use std::sync::Arc;
/// Import the clock that times the repair.
use std::time::Instant;
/// The label of a metadata read whose item panicked:
/// the directory it was reading.
struct ReadingLabel;

impl ItemLabel<PathBuf> for ReadingLabel {
    fn label(
        &self,
        directory: &PathBuf,
    ) -> String {
        return format!("reading the Cargo workspace at {}", directory.display());
    }
}

/// The label of a workspace repair whose item panicked:
/// the workspace it was repairing.
struct RepairLabel;

impl ItemLabel<Workspace> for RepairLabel {
    fn label(
        &self,
        workspace: &Workspace,
    ) -> String {
        return format!(
            "the repair of the Cargo workspace at {}",
            workspace.root.display()
        );
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
    let directories: Vec<PathBuf> =
        match manifest_directories_awaiting(request.git, request.destination).await {
            Ok(found) => found,
            Err(reason) => {
                debug(
                    TAG,
                    format!("{}: {reason}", request.destination.display()).as_str(),
                );
                return Vec::new();
            }
        };
    if directories.is_empty() {
        return Vec::new();
    }
    let search_path: Option<OsString> = request.environment.search_path.clone();
    let looked = run_a_job_or_in_turn(
        (
            search_path,
            request.destination.to_path_buf(),
            request.environment.start_directory.clone(),
        ),
        look_for_the_program,
    )
    .await;
    let (found_program, real_destination): (Option<PathBuf>, Result<PathBuf, std::io::Error>) =
        match looked {
            Ok(found) => found,
            // Which workspaces need what was not learnt, so the repair says what it could not do and names the command.
            Err(refused) => {
                return vec![Finding {
                    tool: Tool::Cargo,
                    outcome: Outcome::Failed {
                        reason: format!("the Cargo repair could not be prepared ({refused})"),
                        run_instead: manual_command(
                            request.destination,
                            &[String::from("<member>")],
                        ),
                    },
                }];
            }
        };
    let Some(cargo) = found_program else {
        return vec![missing_tool(
            Tool::Cargo,
            PROGRAM,
            "cargo clean --offline --package <member> in each Cargo workspace of the worktree",
        )];
    };
    // The repository's common Git directory, asked once for the whole lane: a moved-aside build state waits under it, out of the
    //  worktree, and the lock bytes a clean displaces are kept under it too.
    let common_dir: Option<PathBuf> = common_directory_awaiting(request.git, request.destination)
        .await
        .ok();
    let workers: usize = request.environment.workers;
    let tool: CargoTool = CargoTool {
        program: cargo,
        limits: request.environment.limits(),
    };
    let listed: Arc<[PathBuf]> = Arc::from(directories);
    let mut read: Vec<Result<Workspace, String>> = Vec::new();
    for outcome in map_concurrent_awaiting(
        Arc::clone(&listed),
        workers,
        Arc::new(ReadingLabel),
        Arc::new(ReadWorkspaceJob {
            tool: tool.clone(),
        }),
    )
    .await
    {
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
            Err(reason) => findings.push(Finding {
                tool: Tool::Cargo,
                outcome: Outcome::Failed {
                    reason,
                    run_instead: manual_command(directory.as_path(), &[String::from("<member>")]),
                },
            }),
        }
    }
    let workspaces: Arc<[Workspace]> = Arc::from(distinct_workspaces(found));
    // Real paths: Cargo names the workspace by its real path, and the worktree may be reached through a link.
    let destination_root: PathBuf = match &real_destination {
        Ok(real) => real.clone(),
        // A worktree whose real path cannot be read is used as the caller named it.
        Err(_cannot_be_resolved) => request.destination.to_path_buf(),
    };
    let repairer: RepairWorkspaceJob = RepairWorkspaceJob {
        tool,
        destination_root,
        source: request.source.to_path_buf(),
        kept_in: common_dir.clone(),
    };
    let repaired: Vec<Result<Done, String>> = map_concurrent_awaiting(
        Arc::clone(&workspaces),
        workers,
        Arc::new(RepairLabel),
        Arc::new(repairer),
    )
    .await;
    let mut done: Vec<Done> = Vec::new();
    for (outcome, workspace) in repaired.into_iter().zip(workspaces.iter()) {
        done.push(match outcome {
            Ok(finished) => finished,
            Err(reason) => Done::Failed(Finding {
                tool: Tool::Cargo,
                outcome: Outcome::Failed {
                    reason,
                    run_instead: manual_command(
                        workspace.root.as_path(),
                        &[String::from("<member>")],
                    ),
                },
            }),
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
        let workspace_noun: &str = if cleaned == 1 {
            "workspace"
        } else {
            "workspaces"
        };
        // A clean that displaced a lock keeps those bytes, and the line says where they are: a worktree that is clean again may
        //  still hold a command's newer lock under its Git directory (`package_repair_cargo_lock.rs`).
        let kept: String = if displaced.is_empty() {
            String::new()
        } else {
            format!("; {}", displaced.join("; "))
        };
        findings.push(Finding { tool: Tool::Cargo, outcome: Outcome::Repaired { detail: format!("Cargo kept dependency builds and cleaned the worktree's own crates in {cleaned} {workspace_noun}{kept}"), millis: started.elapsed().as_millis() } });
    }
    if !handed_over.is_empty() {
        findings.push(
            hand_the_filling_over(request, handed_over.as_slice(), common_dir.as_deref()).await,
        );
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
            debug(
                TAG,
                format!(
                    "{}: git could not list the tracked manifests ({refusal})",
                    request.destination.display()
                )
                .as_str(),
            );
            Vec::new()
        }
    };
}

/// Cargo repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_cargo_tests.rs"]
mod tests;
