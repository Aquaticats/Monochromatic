//! What:
//!  One workspace resolved and cleaned inside a mirror of it,
//!  so the person's `Cargo.lock` is read and never written.
//! Why:
//!   A `--locked` clean fails on a lock that no longer matches its manifest,
//! and the resolution that makes it match is a write to `Cargo.lock`,
//!  which is the person's file:
//! Cargo has no option that names another lock (`cargo clean --help`,
//! and no `-Z lockfile-path` in this toolchain),
//! so the write is taken somewhere the person's file is not (`package_repair_workspace_mirror.rs`).
//! The resolution and the clean both run in the mirror,
//! and the clean deletes from the directories this repair names,
//! which are the same artifacts the workspace's own clean deletes
//! (measured on this host on 2026-11-14 in the mirror module's note).
//! Nothing of the person's moves: no placement, no restore, no copy left under the Git directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const run = await cleanInAMirror({ tool, worktree, workspace, members, buildDirectory, targetDirectory, locked, keptIn, job, step });
//! ```

use crate::package_repair_cargo::{CargoTool, clean_arguments, locked_clean_arguments, time_the_clean_awaiting};
use crate::package_repair_command::{ToolCommand, ToolRun};
use crate::package_repair_prewarm::{BUILD_DIRECTORY_VARIABLE, TARGET_DIRECTORY_VARIABLE};
use crate::package_repair_workspace_mirror::{mirror_the_workspace, remove_the_mirror, the_mirror_root};
use std::ffi::OsString;
use std::path::PathBuf;

/// What:
///  One call of the mirror clean:
///  what to resolve and clean,
///  which directories the clean deletes from,
///  and where the mirror may live.
/// Why:
///   The filling and the add's own clean ask the same two Cargo runs of the same mirror,
/// and they differ only in those facts,
/// so the fields are the whole of the difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MirrorClean = { tool: CargoTool; worktree: PathBuf; workspace: PathBuf; members: Vec<String>; buildDirectory: PathBuf; targetDirectory: PathBuf; locked: bool; keptIn: Option<PathBuf>; job: String; step: Option<&'static str> };
/// ```
pub(crate) struct MirrorClean {
    /// The Cargo that resolves and cleans, and the bounds it runs under.
    pub(crate) tool: CargoTool,
    /// The worktree the workspace sits in, which is the root the mirror is laid out below.
    pub(crate) worktree: PathBuf,
    /// The workspace whose manifest and lock the mirror copies.
    pub(crate) workspace: PathBuf,
    /// The package names the clean removes.
    pub(crate) members: Vec<String>,
    /// The build directory the clean deletes from,
    /// which is the repair's own copy in the filling and the worktree's own in the add.
    pub(crate) build_directory: PathBuf,
    /// The target directory the clean deletes from,
    /// chosen for the same reason.
    pub(crate) target_directory: PathBuf,
    /// Whether the clean is given `--locked`,
    /// which the filling wants because the mirror was resolved first.
    pub(crate) locked: bool,
    /// The Git directory the mirror may live under,
    /// or nothing when none is known,
    /// in which case this step cannot run at all.
    pub(crate) kept_in: Option<PathBuf>,
    /// The name of this job, so two jobs of one workspace never share a mirror.
    pub(crate) job: String,
}

/// What:
///  Resolve one workspace in a mirror of it,
///  clean the named directories from there,
///  and answer the clean's run.
/// Why:
///   Both Cargo runs see the person's manifests and the person's lock bytes,
/// and neither can write the person's file:
/// the mirror's own `Cargo.lock` is what the resolution writes and the clean reads.
/// A mirror that cannot be made,
/// a resolution that fails,
/// and no Git directory to keep the mirror under all answer a reason,
/// and the caller gives that workspace up rather than run Cargo in the worktree.
///
/// # Errors
///
/// No Git directory to keep a mirror under,
/// a workspace that cannot be mirrored,
/// and a resolution Cargo refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = await cleanInAMirror(clean); // mirror, metadata, clean --offline [--locked] -p ..., remove the mirror
/// ```
pub(crate) async fn clean_in_a_mirror(clean: MirrorClean) -> Result<ToolRun, String> {
    let Some(kept_in) = clean.kept_in.as_deref() else {
        return Err(format!(
            "the workspace of {} would have to be resolved, and no Git directory was named to resolve it outside the worktree",
            clean.workspace.display()
        ));
    };
    let root: PathBuf = the_mirror_root(kept_in, clean.workspace.as_path(), clean.job.as_str());
    // A mirror a killed job left is this job's own, and it is rebuilt rather than trusted.
    remove_the_mirror(root.as_path());
    let mirrored: PathBuf = match mirror_the_workspace(clean.worktree.as_path(), clean.workspace.as_path(), root.as_path()) {
        Ok(mirrored) => mirrored,
        Err(reason) => {
            remove_the_mirror(root.as_path());
            return Err(reason);
        }
    };
    let resolving: Vec<OsString> = ["metadata", "--offline", "--format-version", "1"].iter().map(OsString::from).collect();
    let resolution: ToolCommand = ToolCommand { arguments: resolving, directory: mirrored.clone(), set: Vec::new() };
    let resolved: ToolRun = crate::package_repair_command::run_tool_awaiting(clean.tool.program.as_path(), &resolution, clean.tool.limits).await;
    if !matches!(resolved.end, crate::package_repair_command::ToolEnd::Succeeded) {
        remove_the_mirror(root.as_path());
        return Err(format!(
            "the workspace of {} could not be resolved outside the worktree ({})",
            clean.workspace.display(),
            crate::package_repair_prewarm_job::end_reason(&resolved.end)
        ));
    }
    let arguments: Vec<OsString> = if clean.locked { locked_clean_arguments(clean.members.as_slice()) } else { clean_arguments(clean.members.as_slice()) };
    let command: ToolCommand = ToolCommand {
        arguments,
        directory: mirrored.clone(),
        set: vec![
            (OsString::from(BUILD_DIRECTORY_VARIABLE), clean.build_directory.as_os_str().to_os_string()),
            (OsString::from(TARGET_DIRECTORY_VARIABLE), clean.target_directory.as_os_str().to_os_string()),
        ],
    };
    let run: ToolRun = time_the_clean_awaiting(&clean.tool, &command).await;
    remove_the_mirror(root.as_path());
    return Ok(run);
}

#[cfg(test)]
#[path = "package_repair_cargo_clean_in_mirror_tests.rs"]
mod tests;
