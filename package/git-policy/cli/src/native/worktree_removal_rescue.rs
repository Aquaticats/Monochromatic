//! What:
//!  What happens to the unshown push-failure notices and the unfinished backup pushes of a linked worktree when Git removes
//!  that worktree: before `git worktree remove` or `git worktree prune` runs, the wrapper copies those records into a snapshot
//!  under the common Git directory, and a later wrapper command settles each snapshot, turning it into notices that any
//!  worktree shows when the worktree is gone and discarding it when the worktree is still there.
//! Why:
//!  A worktree keeps its records in its own Git directory, which Git deletes with the worktree. Without this a failed push
//!  nobody had seen yet, or a push the service had not run yet, would vanish with no word, and the person would never learn that a
//!  branch is not on the remote. The wrapper hands the process over to Git for the removal, so it cannot look afterwards; the
//!  snapshot is taken for every linked worktree that has records and settled by whether its Git directory still exists, so a
//!  removal that Git refuses, a dry run and a prune that spares a worktree need no special handling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await snapshotLinkedWorktrees(commonDir); // before `git worktree remove`
//! await settleRemovals(commonDir); // on the next command
//! ```

use super::diagnostic_log::warn;
use super::push_failure_notice::{ORPHAN_NOTICE_DIRECTORY, notice_directory, record_lost_push};
use super::private_storage::{create_private_directory_all, write_private_file_unsynced};
use super::service_instance::instance_of;
use super::work_item::{CLAIMED, PENDING, WORK_DIRECTORY, WorkItem, WorkKind};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalRescue";

/// Directory under the common Git directory that holds the snapshots.
pub const SNAPSHOT_DIRECTORY: &str = "cli-git/removal-snapshots";

/// File in a snapshot that names the Git directory it was taken from.
const GIT_DIR_FILE: &str = "gitdir";

/// Directory in a snapshot that holds copies of the notices.
const NOTICES: &str = "notices";

/// Directory in a snapshot that holds copies of the unfinished push records, named `<state>-<record>`.
const WORK: &str = "work";

/// The files of a directory, or none when it cannot be read.
fn files_of(directory: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(directory) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = listing.flatten().map(|entry| { return entry.path(); }).filter(|path| { return path.is_file(); }).collect();
    paths.sort();
    return paths;
}

/// The name of a file as text.
fn name_of(path: &Path) -> String {
    return path.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default();
}

/// Copies one file into `directory` under `name`.
fn copy_into(directory: &Path, name: &str, source: &Path) -> std::io::Result<()> {
    create_private_directory_all(directory)?;
    return write_private_file_unsynced(directory.join(name).as_path(), std::fs::read(source)?.as_slice());
}

/// What:
///  Copies the unshown notices and the unfinished push records of each linked worktree into a snapshot of that worktree.
/// Why:
///   Taken before a command that may remove worktrees; a worktree with no records has no snapshot, and one that was
///  snapshotted before has its snapshot replaced by the current records. Failing to copy is said in the diagnostic log and
///  never stops the command the person asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await snapshotLinkedWorktrees(commonDir)
/// ```
pub fn snapshot_linked_worktrees(common_dir: &Path) {
    let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) else { return };
    for entry in listing.flatten() {
        let git_dir: PathBuf = entry.path();
        if !git_dir.is_dir() {
            continue;
        }
        let notices: Vec<PathBuf> = files_of(notice_directory(git_dir.as_path()).as_path());
        let work_directory: PathBuf = git_dir.join(WORK_DIRECTORY);
        let pushes: Vec<(&str, PathBuf)> = [PENDING, CLAIMED]
            .into_iter()
            .flat_map(|state| { return files_of(work_directory.join(state).as_path()).into_iter().map(move |path| { return (state, path); }); })
            .filter(|(_, path)| { return name_of(path).starts_with(format!("{}-", WorkKind::Push.name()).as_str()); })
            .collect();
        if notices.is_empty() && pushes.is_empty() {
            continue;
        }
        let snapshot: PathBuf = common_dir.join(SNAPSHOT_DIRECTORY).join(instance_of(git_dir.as_path()));
        let _replaced = std::fs::remove_dir_all(snapshot.as_path());
        let copied: std::io::Result<()> = (|| {
            create_private_directory_all(snapshot.as_path())?;
            write_private_file_unsynced(snapshot.join(GIT_DIR_FILE).as_path(), git_dir.to_string_lossy().as_bytes())?;
            for path in &notices {
                copy_into(snapshot.join(NOTICES).as_path(), name_of(path).as_str(), path.as_path())?;
            }
            for (state, path) in &pushes {
                copy_into(snapshot.join(WORK).as_path(), format!("{state}-{}", name_of(path)).as_str(), path.as_path())?;
            }
            return Ok(());
        })();
        if let Err(error) = copied {
            warn(TAG, format!("snapshotting {} failed: {error}", git_dir.display()).as_str());
            let _removed = std::fs::remove_dir_all(snapshot.as_path());
        }
    }
}

/// What:
///  Settles every snapshot: one whose worktree is still registered is discarded; one whose worktree is gone turns its notices
///  into notices any worktree shows, and each unfinished push into a record that says the push may not have finished.
/// Why:
///   Whether the removal happened is known only now, from whether the worktree's Git directory exists. When nothing was
///  snapshotted this is one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleRemovals(commonDir)
/// ```
pub fn settle_removals(common_dir: &Path) {
    let Ok(listing) = std::fs::read_dir(common_dir.join(SNAPSHOT_DIRECTORY)) else { return };
    for entry in listing.flatten() {
        let snapshot: PathBuf = entry.path();
        let Ok(named) = std::fs::read_to_string(snapshot.join(GIT_DIR_FILE)) else {
            // A snapshot being written has no name yet; one that never got it is not worth keeping once it is old, but a
            //  concurrent writer may be at it, so it is left to the next settle.
            continue;
        };
        if Path::new(named.as_str()).exists() {
            let _discarded = std::fs::remove_dir_all(snapshot.as_path());
            continue;
        }
        let orphans: PathBuf = common_dir.join(ORPHAN_NOTICE_DIRECTORY);
        for path in files_of(snapshot.join(NOTICES).as_path()) {
            let moved: std::io::Result<()> = create_private_directory_all(orphans.as_path()).and_then(|()| { return std::fs::rename(path.as_path(), orphans.join(name_of(path.as_path()))); });
            if let Err(error) = moved {
                warn(TAG, format!("keeping the notice {} of a removed worktree failed: {error}", path.display()).as_str());
            }
        }
        for path in files_of(snapshot.join(WORK).as_path()) {
            let item: Result<WorkItem, String> = std::fs::read_to_string(path.as_path()).map_err(|error| { return error.to_string(); }).and_then(|text| { return WorkItem::decode(text.as_str()); });
            match item {
                Ok(found) => {
                    if let Err(reason) = record_lost_push(common_dir, found.branch_ref.as_str(), found.oid.as_str()) {
                        warn(TAG, format!("recording the lost push of {} failed: {reason}", found.oid).as_str());
                    }
                }
                Err(reason) => warn(TAG, format!("a push record of a removed worktree is unreadable: {reason}").as_str()),
            }
        }
        let _settled = std::fs::remove_dir_all(snapshot.as_path());
    }
}

/// Snapshot controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_rescue_tests.rs"]
mod tests;
