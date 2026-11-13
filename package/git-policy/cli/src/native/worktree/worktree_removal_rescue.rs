//! What:
//!  What happens to the unshown push-failure notices and the unfinished backup pushes of a linked worktree when Git removes
//!  that worktree:
//! before `git worktree remove` or `git worktree prune` runs,
//! the wrapper copies those records into a snapshot
//!  under the common Git directory,
//! and a later wrapper command settles each snapshot,
//! turning it into notices that any
//!  worktree shows when the worktree is gone and discarding it when the removal did not happen.
//! Why:
//!  A worktree keeps its records in its own Git directory,
//! which Git deletes with the worktree.
//! Without this a failed push
//!  nobody had seen yet,
//! or a push the service had not run yet,
//! would vanish with no word,
//! and the person would never learn that a
//!  branch is not on the remote.
//! The wrapper hands the process over to Git for the removal,
//! so it cannot look afterwards;
//! the
//!  snapshot is taken for every linked worktree that has records and settled later.
//! A Git directory that is gone proves the
//!  removal.
//! A Git directory that is still there proves a refusal only once the command that took the snapshot has ended
//!  (the process keeps its identity when it becomes Git,
//! so the snapshot names it) and the worktree's checkout is intact;
//!  while that command lives,
//! or a checkout is missing from a Git directory that is still registered,
//! the removal may be
//!  under way or cut short,
//! and the snapshot is kept.
//! A snapshot that cannot be fully promoted,
//! or fully taken,
//! is kept as far as
//!  it goes and reported on standard error,
//! because a line in the diagnostic log is not a word to the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const problems = await snapshotLinkedWorktrees(commonDir); // before `git worktree remove`
//! const later = await settleRemovals(commonDir); // on the next command
//! ```

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::diagnostic_log::debug;
use crate::plain_file::{SMALL_FILE_LIMIT, read_plain_file};
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::process_identity::current_birth_identity;
use crate::path_bytes_in_records as path_bytes;
use crate::path_bytes_in_records::GITDIR_HEX_PREFIX;
use crate::push_failure_notice::notice_directory;
use crate::service_instance::instance_of;
use crate::work_item::{CLAIMED, PENDING, RECOVERING, WORK_DIRECTORY, WorkKind, find_claim, id_of_entry};
use crate::worktree_removal_snapshot_promotion::{NOTICES, WORK, files_of, name_of, promote_snapshot};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalRescue";

/// Directory under the common Git directory that holds the snapshots.
pub const SNAPSHOT_DIRECTORY: &str = "monochromatic-git-policy-cli/removal-snapshots";

/// File in a snapshot that names the Git directory it was taken from;
/// written last,
/// so a snapshot that has it is whole.
const GIT_DIR_FILE: &str = "gitdir";

/// Directory in a snapshot that holds one file per command that took or retook it,
/// named by the command's process number
///  and holding the process's birth identity.
const ATTEMPTS: &str = "attempts";

/// How many rounds of looks a push record gets:
/// `pending`,
/// then the claim.
/// A record moves from the first to the second when the
///  service claims it and back when a claim is given up;
/// the second round catches a record that moved away and back during
///  the first.
const LOOK_ROUNDS: usize = 2;

/// What a snapshot's Git directory,
/// its taker and its checkout say about a removal.
#[path = "worktree_removal_rescue_removal_state.rs"]
mod removal_state;
use removal_state::{Removal, named_git_dir, removal_of};

/// Writes a file into a snapshot unless a file of that name is already there,
/// which an earlier snapshot of the same worktree
///  left and which holds the same record.
fn save_once(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    return match write_private_file_unsynced(path, bytes) {
        Err(error) if error.kind() != std::io::ErrorKind::AlreadyExists => Err(error),
        Ok(()) | Err(_) => Ok(()),
    };
}

/// Copies one file into `directory` under `name`;
/// `false` when the file is gone,
/// because whoever shows or finishes it was
///  quicker than the copy and there is nothing left to keep.
fn copy_into(directory: &Path, name: &str, source: &Path) -> std::io::Result<bool> {
    // Read as a plain file: a record's name sits in a worktree's own Git directory, and a named pipe planted there would
    // hold the command that copies it for ever. A record that is not a plain file is one that cannot be copied, which the
    // caller reports and which keeps the worktree it belongs to (`plain_file.rs`).
    let bytes: Vec<u8> = match read_plain_file(source, SMALL_FILE_LIMIT) {
        Ok(read) => read,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    create_private_directory_all(directory)?;
    save_once(directory.join(name).as_path(), bytes.as_slice())?;
    return Ok(true);
}

/// The ids of the push records of a worktree,
/// in any state a push is unfinished in,
/// once each and in order.
/// A claim's file name
///  carries its generation and clock reading,
/// so the id is the part before them.
fn push_record_names(work_directory: &Path) -> Vec<String> {
    let prefix: String = format!("{}-", WorkKind::Push.name());
    let mut names: Vec<String> = Vec::new();
    for state in [PENDING, CLAIMED, RECOVERING] {
        for path in files_of(work_directory.join(state).as_path()) {
            let name: String = id_of_entry(name_of(path.as_path()).as_str()).to_owned();
            if name.starts_with(prefix.as_str()) {
                names.push(name);
            }
        }
    }
    names.sort();
    names.dedup();
    return names;
}

/// What:
///  The state a push record is in now and its bytes,
/// or nothing when it is in neither state.
/// Why:
///   The service moves a record from `pending` to `claimed` at any moment,
/// and renames a claim each time it renews it.
/// A record
///  listed in one state may be in the other by the time it is read,
/// so the claim is found by its id each time,
/// and a record in
///  neither has finished,
/// so there is nothing to keep.
/// Only a failure to read a record that is there is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await locatePushRecord(workDirectory, name) // ['claimed', bytes] | undefined
/// ```
fn locate_push_record(work_directory: &Path, name: &str) -> std::io::Result<Option<(&'static str, Vec<u8>)>> {
    for _ in 0..LOOK_ROUNDS {
        for state in [PENDING, CLAIMED] {
            let path: PathBuf = if state == PENDING {
                work_directory.join(PENDING).join(name)
            } else {
                match find_claim(work_directory, name) {
                    Some(found) => found,
                    None => continue,
                }
            };
            // A plain file, so a name that holds a named pipe is a record that cannot be read rather than a wait with no end.
            match read_plain_file(path.as_path(), SMALL_FILE_LIMIT) {
                Ok(bytes) => return Ok(Some((state, bytes))),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
    }
    return Ok(None);
}

/// Records,
/// in the snapshot,
/// that this process took it and so may be about to remove the worktree.
fn register_attempt(snapshot: &Path) -> std::io::Result<()> {
    let identity: String = match current_birth_identity() {
        Ok(found) => found,
        Err(error) => return Err(std::io::Error::other(error.to_string())),
    };
    let directory: PathBuf = snapshot.join(ATTEMPTS);
    create_private_directory_all(directory.as_path())?;
    // The file is named by this process's number, so no other live process owns it: one left by an earlier process of the
    //  same number holds an identity that is not this process's, and is replaced.
    let own: PathBuf = directory.join(std::process::id().to_string());
    match std::fs::remove_file(own.as_path()) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
        Ok(()) | Err(_) => {}
    }
    return write_private_file_unsynced(own.as_path(), identity.as_bytes());
}

///  Copies the unshown notices and the unfinished push records of each linked worktree into a snapshot of that worktree,
/// and
///  says what could not be copied.
/// Why:
///   Taken before a command that may remove worktrees;
/// a worktree with no records has no snapshot.
/// A snapshot taken before is
///  added to and not replaced:
/// the command that took it may still be removing,
/// and its records are what a copy made now,
///  from a half-removed worktree,
/// could lack.
/// A record that vanished while it was being copied was shown or finished and needs
///  no copy;
/// any other failure is reported to the person,
/// never hidden in the diagnostic log,
/// and what was copied is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const problems: string[] = await snapshotLinkedWorktrees(commonDir)
/// ```
pub fn snapshot_linked_worktrees(common_dir: &Path) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();
    let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) else { return problems };
    for entry in listing.flatten() {
        let git_dir: PathBuf = entry.path();
        if !git_dir.is_dir() {
            continue;
        }
        for problem in snapshot_one(common_dir, git_dir.as_path()) {
            problems.push(problem);
        }
    }
    return problems;
}

/// What:
///  The same snapshot for one worktree's Git directory,
/// which a caller that knows exactly which worktree it means uses.
/// Why:
///   The finish of an interrupted destroy copies the records of that one registration before it removes it,
/// without
///  reading every other worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const problems: string[] = snapshotOne(commonDir, gitDir)
/// ```
pub(crate) fn snapshot_one(common_dir: &Path, git_dir: &Path) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();
    {
        let notices: Vec<PathBuf> = files_of(notice_directory(git_dir).as_path());
        let work_directory: PathBuf = path_for_reading(git_dir, WORK_DIRECTORY);
        let pushes: Vec<String> = push_record_names(work_directory.as_path());
        if notices.is_empty() && pushes.is_empty() {
            return problems;
        }
        let snapshot: PathBuf = common_dir.join(SNAPSHOT_DIRECTORY).join(instance_of(git_dir));
        let mut failures: Vec<String> = Vec::new();
        match create_private_directory_all(snapshot.as_path()) {
            Ok(()) => {
                if let Err(error) = register_attempt(snapshot.as_path()) {
                    failures.push(format!("the command could not be recorded as the one removing it ({error})"));
                }
            }
            Err(error) => failures.push(format!("{} could not be created ({error})", snapshot.display())),
        }
        if failures.is_empty() {
            for path in &notices {
                if let Err(error) = copy_into(snapshot.join(NOTICES).as_path(), name_of(path.as_path()).as_str(), path.as_path()) {
                    failures.push(format!("the notice {} could not be copied ({error})", name_of(path.as_path())));
                }
            }
            for name in &pushes {
                match locate_push_record(work_directory.as_path(), name.as_str()) {
                    Ok(Some((state, bytes))) => {
                        let copied: std::io::Result<()> = match create_private_directory_all(snapshot.join(WORK).as_path()) {
                            Ok(()) => save_once(snapshot.join(WORK).join(format!("{state}-{name}")).as_path(), bytes.as_slice()),
                            Err(error) => Err(error),
                        };
                        if let Err(error) = copied {
                            failures.push(format!("the push record {name} could not be copied ({error})"));
                        }
                    }
                    Ok(None) => {}
                    Err(error) => failures.push(format!("the push record {name} could not be read ({error})")),
                }
            }
            // Last, so that a snapshot that names its worktree is a whole one.
            // The snapshot names its worktree by the bytes of its Git directory, so a path that is not text is still found.
            let named: String = format!("{GITDIR_HEX_PREFIX}{}\n", path_bytes::hex_of_path(git_dir));
            if let Err(error) = save_once(snapshot.join(GIT_DIR_FILE).as_path(), named.as_bytes()) {
                failures.push(format!("the snapshot could not be completed ({error})"));
            }
        }
        if !failures.is_empty() {
            let said: String = format!(
                "Before this removal, the unshown push failures and unfinished pushes of the worktree {} could not all be saved: {}. \
                 The command goes on, so whatever was not saved is lost if Git removes that worktree; what was saved is kept in {}.",
                git_dir.display(),
                failures.join("; "),
                snapshot.display()
            );
            debug(TAG, said.as_str());
            problems.push(said);
        }
    }
    return problems;
}

/// What:
///  Settles every snapshot whose removal is decided:
/// one whose worktree is still registered and whose taker has ended is
///  discarded;
/// one whose worktree is gone turns its notices into notices any worktree shows,
/// and each unfinished push into a
///  record that says the push may not have finished.
/// Gives what could not be settled,
/// for the person.
/// Why:
///   Whether the removal happened is known only now (`removal_of`).
/// A snapshot whose removal may be under way is left alone,
///  and one that could not be fully promoted is kept for the next try.
/// When nothing was snapshotted this is one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const problems: string[] = await settleRemovals(commonDir)
/// ```
pub fn settle_removals(common_dir: &Path) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();
    let Ok(listing) = std::fs::read_dir(common_dir.join(SNAPSHOT_DIRECTORY)) else { return problems };
    for entry in listing.flatten() {
        let snapshot: PathBuf = entry.path();
        // A plain file, so a name that holds a named pipe is a snapshot left to the next settle rather than a wait with no end.
        let Ok(recorded) = read_plain_file(snapshot.join(GIT_DIR_FILE).as_path(), SMALL_FILE_LIMIT) else {
            // A snapshot being written has no name yet; one that never got it is not worth keeping once it is old, but a
            //  concurrent writer may be at it, so it is left to the next settle.
            continue;
        };
        let Ok(named) = String::from_utf8(recorded) else { continue };
        let Some(git_dir) = named_git_dir(named.as_str()) else { continue };
        match removal_of(git_dir.as_path(), snapshot.as_path()) {
            Removal::Undecided => {}
            Removal::Refused => {
                let _discarded = std::fs::remove_dir_all(snapshot.as_path());
            }
            Removal::Done => promote_snapshot(common_dir, snapshot.as_path(), &mut problems),
        }
    }
    return problems;
}

/// Snapshot controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_rescue_tests.rs"]
mod tests;
