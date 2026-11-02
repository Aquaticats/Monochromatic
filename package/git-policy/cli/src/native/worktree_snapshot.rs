//! What:
//!  The mechanics of rung 1 of the cheap-worktree ladder: a snapshot of the source subvolume taken at a stage name beside
//!  the destination, made into a worktree there (its own `.git` pointer, no stubs of nested worktrees, the source's index
//!  with refreshed stat fields), and swapped with the destination in one atomic step.
//! Why:
//!  The snapshot shares every data block with the source and carries its ignored state at no cost
//!  (`doc/planning/cli-git-btrfs-worktree.md`). It is made at a name Git never hands out, so the destination is
//!  complete or untouched at every instant: `RENAME_EXCHANGE` puts the finished subvolume at the destination and the
//!  empty directory real Git made at the stage name, which is then removed. A source that changed while the snapshot was
//!  taken is caught by comparing the snapshot's own files with the index entries copied from the source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await takeSnapshot({ source, plan, stage, admin }); // throws a reason; the caller destroys the stage
//! await putInPlace({ stage, destination, mode });
//! ```

use super::fs_btrfs_ioctl::snapshot_create_at;
use super::fs_clone_exchange::exchange_paths;
use super::subvolume_index_files::{IndexLock, RefreshOptions, refresh_index_file_with};
use super::subvolume_remove::remove_tree;
use super::worktree_add_plan::AddPlan;
use super::worktree_clean::{StatEntry, stored_stats, tree_differs_from_index};
use super::worktree_snapshot_source::SnapshotSource;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// What:
///  Remove the stub that a nested worktree leaves in the snapshot, and the destination's own directory when it lies
///  inside the source.
/// Why:
///   A nested worktree that is a subvolume is an empty directory in the snapshot; the destination was made by Git
///  before the snapshot, so a source that contains it holds a directory with Git's `.git` file there. Neither belongs in
///  the new worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeStubs({ stage, sourceRoot, nested, destination });
/// ```
fn remove_stubs(stage: &Path, source: &SnapshotSource, plan: &AddPlan) -> Result<(), String> {
    let mut paths = source.nested.iter().map(|path: &std::path::PathBuf| { return path.as_path(); }).chain(std::iter::once(plan.destination.as_path()));
    paths.try_for_each(|path: &Path| {
        let Ok(relative) = path.strip_prefix(source.root.as_path()) else { return Ok(()) };
        let inside: std::path::PathBuf = stage.join(relative);
        match std::fs::remove_file(inside.join(".git")) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{}/.git cannot be removed from the snapshot ({error})", inside.display())),
        }
        return match std::fs::remove_dir(inside.as_path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("{} cannot be removed from the snapshot ({error})", inside.display())),
        };
    })?;
    return Ok(());
}

/// What:
///  Put the new worktree's `.git` pointer into the snapshot in place of the source's `.git` (a directory for a main
///  checkout, a pointer file for a linked worktree).
/// Why:
///   The bytes are the ones real Git wrote for the new worktree, so the pointer is exactly Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await replaceGitPointer(stage, gitFileOfNewWorktree);
/// ```
fn replace_git_pointer(stage: &Path, pointer: &[u8]) -> Result<(), String> {
    let path = stage.join(".git");
    remove_tree(path.as_path()).map_err(|error| { return format!("the source's .git cannot be removed from the snapshot ({error})"); })?;
    return std::fs::write(path.as_path(), pointer).map_err(|error| { return format!("the .git pointer cannot be written ({error})"); });
}

/// What:
///  Check the snapshot against the source's index, then install that index in the new worktree's administrative
///  directory with stat fields refreshed and the accelerator extensions of the source dropped.
/// Why:
///   Every tracked file of the snapshot must equal its entry, which proves the snapshot and the index describe the same
///  state whatever the source did meanwhile. The extensions (`UNTR`, `FSMN`) name the source's worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await installIndex({ source, stage, admin });
/// ```
fn install_index(source: &SnapshotSource, stage: &Path, admin: &Path) -> Result<(), String> {
    let bytes: Vec<u8> = std::fs::read(source.git_dir.join("index")).map_err(|error| { return format!("the source's index is unreadable ({error})"); })?;
    let entries: Vec<StatEntry> = stored_stats(bytes.as_slice(), source.clean.hash_length).ok_or_else(|| { return String::from("the source's index changed shape"); })?;
    if let Some(reason) = tree_differs_from_index(stage, entries.as_slice(), source.clean.index_seconds) {
        return Err(format!("the source changed while it was copied ({reason})"));
    }
    let target = admin.join("index");
    std::fs::write(target.as_path(), bytes.as_slice()).map_err(|error| { return format!("the index cannot be written ({error})"); })?;
    let report: String = refresh_index_file_with(target.as_path(), stage, &RefreshOptions { lock: IndexLock::Acquire, references: &[], hash_length: Some(source.clean.hash_length) });
    return if report.starts_with("skipped") { Err(format!("the index could not be refreshed ({report})")) } else { Ok(()) };
}

/// What:
///  Take the snapshot at `stage` and make it the new worktree there; the caller destroys `stage` on an error.
/// Why:
///   Each step is one the later steps depend on; the first failure ends the rung with its reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await takeSnapshot({ source, plan, stage, admin, pointer });
/// ```
pub fn take_snapshot(source: &SnapshotSource, plan: &AddPlan, stage: &Path, admin: &Path, pointer: &[u8]) -> Result<(), String> {
    snapshot_create_at(source.root.as_path(), stage).map_err(|error| { return format!("the snapshot was refused ({error})"); })?;
    std::fs::set_permissions(stage, std::fs::Permissions::from_mode(0o700)).map_err(|error| { return format!("the stage cannot be made private ({error})"); })?;
    remove_stubs(stage, source, plan)?;
    replace_git_pointer(stage, pointer)?;
    return install_index(source, stage, admin);
}

/// What:
///  Swap the finished stage with the destination, after giving the stage the mode of the source's root.
/// Why:
///   `RENAME_EXCHANGE` leaves no instant at which the destination is missing or incomplete; the old destination, an
///  empty directory with Git's `.git` file, ends up at the stage name for `remove_leftover`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await putInPlace({ stage, destination, sourceMode });
/// ```
pub fn put_in_place(stage: &Path, destination: &Path, source_mode: u32) -> Result<(), String> {
    std::fs::set_permissions(stage, std::fs::Permissions::from_mode(source_mode)).map_err(|error| { return format!("the stage cannot take the source's mode ({error})"); })?;
    return exchange_paths(stage, destination).map_err(|error| { return format!("the destination cannot be swapped with the stage ({error})"); });
}

/// What:
///  Remove the directory real Git made, now at the stage name.
/// Why:
///   The worktree is complete once swapped; a leftover is the only thing a failure here costs, so the caller says so
///  without failing the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeLeftover(stage);
/// ```
pub fn remove_leftover(stage: &Path) -> Result<(), String> {
    return remove_tree(stage).map_err(|error| { return format!("the directory Git made at {} was left behind ({error})", stage.display()); });
}

/// Snapshot mechanics controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_tests.rs"]
mod tests;
