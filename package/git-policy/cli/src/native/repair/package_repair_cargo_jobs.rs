//! What:
//!  The jobs of the Cargo repair:
//! the program and the bounds one run takes,
//! the lookup of the program beside the worktree's real path,
//! and the two jobs the runner starts for one workspace.
//! Why:
//!  `package_repair_cargo.rs` would otherwise pass the file-size limit,
//! and the jobs need nothing of its decisions but the two
//!  timed reads and the result of one workspace,
//! so they live in a file of their own that the repair imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export class ReadWorkspaceJob { ... }
//! ```

use crate::package_repair_cargo_concurrent::ConcurrentAwaitingWork;
use crate::package_repair_cargo_workspaces::Workspace;
use crate::package_repair_command::{ToolLimits, find_program};
use crate::add_timing::{begin_work, end_work};
use super::{Done, PROGRAM, repair_workspace_awaiting, timed_read_workspace_awaiting};
use std::ffi::OsString;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Instant;

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

/// What:
///  The job of reading the Cargo workspace of one directory,
/// timed as the step `cargoMetadataWorktree`,
/// as an item of the awaited runner.
/// Why:
///   The runner owns the work it starts,
/// so the job owns the program and the bounds,
/// and `cargo metadata` is a child the
///  task awaits rather than a thread it blocks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReadWorkspaceJob implements ConcurrentAwaitingWork<string, Workspace> { constructor(readonly tool: CargoTool) {} async run(directory) { return timedReadWorkspaceAwaiting('cargoMetadataWorktree', this.tool, directory) } }
/// ```
pub(super) struct ReadWorkspaceJob {
    /// The Cargo program and the bounds of its runs.
    pub(super) tool: CargoTool,
}

impl ConcurrentAwaitingWork<PathBuf, Result<Workspace, String>> for ReadWorkspaceJob {
    fn run<'a>(&'a self, directory: &'a PathBuf) -> Pin<Box<dyn Future<Output = Result<Workspace, String>> + Send + 'a>> {
        return Box::pin(async move {
            return timed_read_workspace_awaiting("cargoMetadataWorktree", &self.tool, directory.as_path()).await;
        });
    }
}

/// What:
///  The job of repairing one Cargo workspace,
/// timed as the step `cargoRepairWorkspace`.
/// Why:
///   Same as `ReadWorkspaceJob`:
/// what the repair reads is owned,
/// so the task may outlive the caller's borrows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RepairWorkspaceJob implements ConcurrentAwaitingWork<Workspace, Done> {}
/// ```
pub(super) struct RepairWorkspaceJob {
    /// The Cargo program and the bounds of its runs.
    pub(super) tool: CargoTool,
    /// The real path of the new worktree.
    pub(super) destination_root: PathBuf,
    /// The checkout the build state is carried from.
    pub(super) source: PathBuf,
    /// The Git directory the lock bytes a clean displaces are kept in,
    /// or nothing when no Git directory is known,
    /// in which case a lock the clean changed is left as it is found.
    pub(super) kept_in: Option<PathBuf>,
}

impl ConcurrentAwaitingWork<Workspace, Done> for RepairWorkspaceJob {
    fn run<'a>(&'a self, workspace: &'a Workspace) -> Pin<Box<dyn Future<Output = Done> + Send + 'a>> {
        return Box::pin(async move {
            let began: Instant = begin_work();
            let done: Done = repair_workspace_awaiting(
                &self.tool,
                workspace,
                self.destination_root.as_path(),
                self.source.as_path(),
                self.kept_in.as_deref(),
            )
            .await;
            end_work("cargoRepairWorkspace", began);
            return done;
        });
    }
}

/// What the preparation of a Cargo repair reads:
/// the search path and the worktree.
/// Why:
///   The lookup and the resolve are one job of the pool,
/// so they travel as one state and may be taken again where no
///  thread can start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ThePreparation = [string | undefined, string];
/// ```
type ThePreparation = (Option<OsString>, PathBuf);

/// What:
///  The Cargo program the search path resolves,
/// and the worktree's real path.
/// Why:
///   Both faces of the repair need the program and the real path,
/// and reading the search path and resolving a link are storage work,
///  so they run as a job of the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lookForTheProgram = ([searchPath, destination]) => [findProgram('cargo', searchPath), canonicalize(destination)];
/// ```
pub(super) fn look_for_the_program((search_path, destination): ThePreparation) -> (Option<PathBuf>, Result<PathBuf, std::io::Error>) {
    return (find_program(PROGRAM, search_path.as_deref()), std::fs::canonicalize(destination.as_path()));
}
