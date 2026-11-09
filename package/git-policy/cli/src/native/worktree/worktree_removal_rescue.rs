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
//! so the snapshot names it) and the worktree's checkout is intact.
//! A Git directory that is still registered once its checkout is gone is a removal that will never finish,
//! whether the
//!  destroy or the person's own deletion took the checkout:
//! a worktree without a directory can run no command,
//! so the records reach the person only if a settle promotes them,
//! and the registration is left to the person.
//! While the command that took the snapshot lives,
//! the removal may be under way,
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
use crate::diagnostic_log::{debug, warn};
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::process_identity::{current_birth_identity, process_birth_identity};
use crate::push_failure_notice::{ORPHAN_NOTICE_DIRECTORY, notice_directory, record_lost_push};
use crate::service_instance::instance_of;
use crate::work_item::{CLAIMED, PENDING, RECOVERING, WORK_DIRECTORY, WorkItem, WorkKind, find_claim, id_of_entry};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalRescue";

/// Directory under the common Git directory that holds the snapshots.
pub const SNAPSHOT_DIRECTORY: &str = "monochromatic-git-policy-cli/removal-snapshots";

/// File in a snapshot that names the Git directory it was taken from;
/// written last,
/// so a snapshot that has it is whole.
const GIT_DIR_FILE: &str = "gitdir";

/// Directory in a snapshot that holds copies of the notices.
const NOTICES: &str = "notices";

/// Directory in a snapshot that holds copies of the unfinished push records,
/// named `<state>-<record>`.
const WORK: &str = "work";

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

/// What a Git directory,
/// the process that took a snapshot,
/// and the checkout say about a removal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Removal {
    /// The Git directory is gone,
    /// or it is still registered and its checkout is gone:
    /// the worktree is removed,
    /// or never will be finished.
    Done,
    /// Every command that took the snapshot has ended and the checkout is intact:
    /// Git refused,
    /// or it was a dry run.
    Refused,
    /// A command that took the snapshot may still be removing,
    /// or the evidence cannot be read:
    /// not decided yet.
    Undecided,
}

/// The files of a directory,
/// or none when it cannot be read.
fn files_of(directory: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(directory) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();
    return paths;
}

/// The name of a file as text.
fn name_of(path: &Path) -> String {
    return match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::new(),
    };
}

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
    let bytes: Vec<u8> = match std::fs::read(source) {
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
            match std::fs::read(path) {
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

/// What:
///  Whether a command that took the snapshot may still be running,
/// which a process that cannot be told apart from it counts as.
/// Why:
///   A process number alone can be reused,
/// so the birth identity recorded with it decides.
/// A record that cannot be read,
/// or a
///  process that cannot be inspected,
/// is an owner that may be alive:
/// discarding the snapshot on a guess would lose the records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await attemptsMayBeRunning(snapshot)
/// ```
fn attempt_may_be_running(snapshot: &Path) -> bool {
    // The loop stops at the first attempt that may be running, as `any` did.
    for path in files_of(snapshot.join(ATTEMPTS).as_path()) {
        if attempt_file_may_be_running(path.as_path()) {
            return true;
        }
    }
    return false;
}

/// Whether the command that wrote one attempt file may still be running;
/// a file that cannot be understood counts as running.
fn attempt_file_may_be_running(path: &Path) -> bool {
    let Ok(pid) = name_of(path).parse::<i64>() else { return false };
    let Ok(recorded) = std::fs::read_to_string(path) else { return true };
    if recorded.is_empty() {
        // Being written.
        return true;
    }
    return match process_birth_identity(pid) {
        Ok(Some(now)) => now == recorded,
        Ok(None) => false,
        Err(error) => {
            warn(TAG, format!("the process {pid} of a worktree removal cannot be inspected: {error}").as_str());
            true
        }
    };
}

/// Whether the checkout a registered worktree's Git directory names is still there;
/// nothing when the Git directory does not say.
fn checkout_present(git_dir: &Path) -> Option<bool> {
    let text: String = std::fs::read_to_string(git_dir.join("gitdir")).ok()?;
    let marker: &Path = Path::new(text.trim_end());
    if marker.as_os_str().is_empty() {
        return None;
    }
    return Some(git_dir.join(marker).exists());
}

/// What:
///  What a snapshot's Git directory,
/// its taker and its checkout show of the removal it was taken for.
/// Why:
///   Git deletes the checkout first and the Git directory last,
/// and refuses before it deletes anything.
/// So a missing Git
///  directory is a removal;
/// an intact checkout whose taker has ended is a refusal;
/// a checkout that is gone
///  while its taker has ended cannot come back,
/// so the removal is over however it ended and the records must reach the person;
/// while the taker lives the removal may be under way,
/// and a checkout whose state cannot be read decides nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removalOf(gitDir, snapshot) // 'done' | 'refused' | 'undecided'
/// ```
fn removal_of(git_dir: &Path, snapshot: &Path) -> Removal {
    match std::fs::symlink_metadata(git_dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Removal::Done,
        Err(_) => return Removal::Undecided,
        Ok(_) => {}
    }
    if attempt_may_be_running(snapshot) {
        return Removal::Undecided;
    }
    return match checkout_present(git_dir) {
        Some(true) => Removal::Refused,
        Some(false) => Removal::Done,
        None => Removal::Undecided,
    };
}

/// What:
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
            if let Err(error) = save_once(snapshot.join(GIT_DIR_FILE).as_path(), git_dir.to_string_lossy().as_bytes()) {
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

/// Moves one saved notice to the notices any worktree shows;
/// a notice another command already moved is not a failure.
fn promote_notice(orphans: &Path, path: &Path) -> std::io::Result<()> {
    create_private_directory_all(orphans)?;
    return match std::fs::rename(path, orphans.join(name_of(path))) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
        Ok(()) | Err(_) => Ok(()),
    };
}

/// What:
///  Turns what a snapshot of a removed worktree kept into notices any worktree shows,
/// and removes what it promoted;
/// whether
///  everything was promoted.
/// Why:
///   Each item is removed from the snapshot as soon as it is promoted,
/// so a snapshot kept after a failure promotes only what is
///  left on the next try and a notice already shown is not made again.
/// A failure is reported to the person and the snapshot
///  stays,
/// because the snapshot is then the last copy of that record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const complete = await promote(commonDir, snapshot, problems)
/// ```
fn promote(common_dir: &Path, snapshot: &Path, problems: &mut Vec<String>) -> bool {
    let orphans: PathBuf = common_dir.join(ORPHAN_NOTICE_DIRECTORY);
    let mut complete: bool = true;
    for path in files_of(snapshot.join(NOTICES).as_path()) {
        if let Err(error) = promote_notice(orphans.as_path(), path.as_path()) {
            complete = false;
            let said: String = format!(
                "The push failure notice {} of a removed worktree could not be kept for you ({error}). It stays in {}; once that is fixed, any git command shows it.",
                name_of(path.as_path()),
                snapshot.display()
            );
            debug(TAG, said.as_str());
            problems.push(said);
        }
    }
    for path in files_of(snapshot.join(WORK).as_path()) {
        let text: String = match std::fs::read_to_string(path.as_path()) {
            Ok(read) => read,
            // Another command promoted it first.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                complete = false;
                let said: String = format!("The saved record {} of a removed worktree's unfinished push could not be read ({error}); it stays in {}.", name_of(path.as_path()), snapshot.display());
                debug(TAG, said.as_str());
                problems.push(said);
                continue;
            }
        };
        match WorkItem::decode(text.as_str()) {
            Ok(found) => match record_lost_push(common_dir, found.branch_ref.as_str(), found.oid.as_str()) {
                Ok(()) => {
                    // Removed now, so that a try after a failure elsewhere in this snapshot does not record the push again.
                    if let Err(error) = std::fs::remove_file(path.as_path()) {
                        warn(TAG, format!("removing the promoted record {} failed: {error}", path.display()).as_str());
                    }
                }
                Err(reason) => {
                    complete = false;
                    let said: String = format!(
                        "The backup push of {} for commit {} had not finished when its worktree was removed, and noting that failed ({reason}). \
                         The record stays in {}; once that is fixed, any git command shows it.",
                        found.branch_ref,
                        found.oid,
                        snapshot.display()
                    );
                    debug(TAG, said.as_str());
                    problems.push(said);
                }
            },
            Err(reason) => {
                // A record that cannot be read now cannot be read later: it is said once and dropped.
                let said: String = format!("A saved record of an unfinished push of a removed worktree is unreadable ({reason}), so that push is not reported: {}", path.display());
                debug(TAG, said.as_str());
                problems.push(said);
                let _dropped = std::fs::remove_file(path.as_path());
            }
        }
    }
    return complete;
}

/// What:
///  Promote what one snapshot kept and remove it when everything went through.
/// Why:
///   Two callers settle a snapshot:
/// the settle of every snapshot,
/// and the finish of an interrupted destroy,
/// whose
///  snapshot is the one it just took.
/// A failure keeps the snapshot,
/// which is then the last copy of that record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// promoteSnapshot(commonDir, snapshot, problems)
/// ```
pub(crate) fn promote_snapshot(common_dir: &Path, snapshot: &Path, problems: &mut Vec<String>) {
    if promote(common_dir, snapshot, problems) {
        let _settled = std::fs::remove_dir_all(snapshot);
    }
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
        let Ok(named) = std::fs::read_to_string(snapshot.join(GIT_DIR_FILE)) else {
            // A snapshot being written has no name yet; one that never got it is not worth keeping once it is old, but a
            //  concurrent writer may be at it, so it is left to the next settle.
            continue;
        };
        match removal_of(Path::new(named.as_str()), snapshot.as_path()) {
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
