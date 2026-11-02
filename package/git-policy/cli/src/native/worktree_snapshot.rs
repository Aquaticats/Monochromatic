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

use super::fs_beneath::{open_directory, open_directory_at, open_directory_beneath, plain_components, remove_at};
use super::fs_btrfs_ioctl::snapshot_create_at;
use super::fs_clone_exchange::exchange_paths;
use super::subvolume_index_files::{IndexLock, RefreshOptions, install_index_file};
use super::subvolume_remove::remove_tree;
use super::worktree_add_plan::AddPlan;
use super::worktree_clean::{StatEntry, stored_stats, tree_differs_from_index_beneath};
use super::worktree_snapshot_source::SnapshotSource;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// What:
///  Remove one stub from the snapshot below the held handle of its root: the directory `path` names (a path inside the
///  source), after its `.git` pointer when `pointer` is set.
/// Why:
///   Every directory on the way is opened without following a link and on the snapshot's own device, so a registered
///  path that reaches the snapshot through a link (a link left where a directory was moved) is an error and never an
///  unlink in the tree the link points at. A nested subvolume's stub is removed only as an empty directory; only the
///  destination's own directory, which Git made, holds a pointer file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeStub(root, sourceRoot, path, pointer);
/// ```
fn remove_stub(root: &File, source_root: &Path, path: &Path, pointer: bool) -> Result<(), String> {
    let Ok(relative) = path.strip_prefix(source_root) else { return Ok(()) };
    let shown: String = path.display().to_string();
    let names: Vec<OsString> = plain_components(relative).map_err(|error| { return format!("{shown} is not a plain path inside the source ({error})"); })?;
    let Some((name, parents)) = names.split_last() else { return Ok(()) };
    let gone = |error: &std::io::Error| { return error.kind() == std::io::ErrorKind::NotFound; };
    let parent: File = match open_directory_beneath(root, parents) {
        Ok(found) => found,
        Err(error) if gone(&error) => return Ok(()),
        Err(error) => return Err(format!("{shown} is reached through a link or another mount in the snapshot ({error})")),
    };
    if pointer {
        match open_directory_at(&parent, name.as_os_str()) {
            Ok(stub) => match remove_at(&stub, OsStr::new(".git"), false) {
                Ok(()) => {}
                Err(error) if gone(&error) => {}
                Err(error) => return Err(format!("{shown}/.git cannot be removed from the snapshot ({error})")),
            },
            Err(error) if gone(&error) => return Ok(()),
            Err(error) => return Err(format!("{shown} cannot be entered in the snapshot ({error})")),
        }
    }
    return match remove_at(&parent, name.as_os_str(), true) {
        Ok(()) => Ok(()),
        Err(error) if gone(&error) => Ok(()),
        Err(error) => Err(format!("{shown} cannot be removed from the snapshot as an empty directory ({error})")),
    };
}

/// What:
///  Remove the stub that each nested worktree leaves in the snapshot, and the destination's own directory when it lies
///  inside the source.
/// Why:
///   A nested worktree that is a subvolume is an empty directory in the snapshot; the destination was made by Git before
///  the snapshot, so a source that contains it holds a directory with Git's `.git` file there. Neither belongs in the new
///  worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeStubs({ stage, sourceRoot, nested, destination });
/// ```
fn remove_stubs(stage: &Path, source: &SnapshotSource, plan: &AddPlan) -> Result<(), String> {
    let root: File = open_directory(stage).map_err(|error| { return format!("the stage cannot be opened ({error})"); })?;
    for nested in &source.nested {
        remove_stub(&root, source.root.as_path(), nested.as_path(), false)?;
    }
    return remove_stub(&root, source.root.as_path(), plan.destination.as_path(), true);
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
    let bytes: &[u8] = source.clean.index_bytes.as_slice();
    let entries: Vec<StatEntry> = stored_stats(bytes, source.clean.hash_length).ok_or_else(|| { return String::from("the source's index changed shape"); })?;
    let root: File = open_directory(stage).map_err(|error| { return format!("the stage cannot be opened ({error})"); })?;
    if let Some(reason) = tree_differs_from_index_beneath(&root, entries.as_slice(), source.clean.index_seconds) {
        return Err(format!("the source changed while it was copied ({reason})"));
    }
    let tree: PathBuf = Path::new("/proc/self/fd").join(root.as_raw_fd().to_string());
    let report: String = install_index_file(admin.join("index").as_path(), tree.as_path(), bytes, &RefreshOptions { lock: IndexLock::Acquire, references: &[], hash_length: Some(source.clean.hash_length) });
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

/// The device and inode of a directory.
pub type DirectoryIdentity = (u64, u64);

/// What:
///  Swap the finished stage with the destination, after giving the stage the mode of the source's root.
/// Why:
///   `RENAME_EXCHANGE` leaves no instant at which the destination is missing or incomplete; the old destination, an
///  empty directory with Git's `.git` file, ends up at the stage name for `remove_leftover`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await putInPlace({ stage, destination, sourceMode, expected }); // expected: the (device, inode) of the directory Git made
/// ```
pub fn put_in_place(stage: &Path, destination: &Path, source_mode: u32, expected: DirectoryIdentity) -> Result<(), String> {
    let found = |path: &Path| { return std::fs::symlink_metadata(path).ok().filter(|metadata: &std::fs::Metadata| { return metadata.is_dir(); }).map(|metadata: std::fs::Metadata| { return (metadata.dev(), metadata.ino()); }); };
    if found(destination) != Some(expected) {
        return Err(format!("{} is not the directory Git made", destination.display()));
    }
    std::fs::set_permissions(stage, std::fs::Permissions::from_mode(source_mode)).map_err(|error| { return format!("the stage cannot take the source's mode ({error})"); })?;
    exchange_paths(stage, destination).map_err(|error| { return format!("the destination cannot be swapped with the stage ({error})"); })?;
    // A path swapped between the look and the exchange puts something else at the stage name: put it back.
    if found(stage) != Some(expected) {
        exchange_paths(stage, destination).map_err(|error| { return format!("the destination changed during the swap and could not be put back ({error})"); })?;
        return Err(format!("{} changed while it was swapped", destination.display()));
    }
    return Ok(());
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
