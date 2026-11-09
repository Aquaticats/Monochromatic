//! What:
//!  Finish the removal of a worktree whose files a destroy already took and whose registration it did not,
//!  so that no removal is left half done by a kill or a failed step.
//! Why:
//!  The served removal destroys the subvolume first and removes the registration after it,
//! and the registration is emptied and
//!  removed through its own held descriptor and proof,
//! which can fail where Git's unlinks would not.
//! A command killed in that window,
//!  or one whose step failed,
//! leaves a worktree Git reports as missing but registered,
//! and `git worktree add` at that
//!  path then refuses until someone runs `git worktree prune`.
//! The destroy is therefore recorded before the kernel call,
//! in a
//!  folder of the common Git directory,
//! and every wrapped command finishes what such a record names:
//! the registration is removed
//!  once its records are copied into the removal snapshot the rescue route settles,
//! so an unshown push failure in it is
//!  shown rather than destroyed with it.
//! A record whose worktree is still there is dropped:
//! the destroy did not happen,
//!  so nothing is removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordDestroy(commonDir, id, identity, target);       // before the ioctl
//! await forgetDestroy(commonDir, id);                          // once the registration is gone
//! const problems = await finishInterruptedDestroys(commonDir); // on the next command
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::{ObjectId, identity_of, open_directory_at, open_root, remove_at};
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::subvolume_remove_walk::{empty_directory, verify_below};
use crate::worktree_removal_rescue::{SNAPSHOT_DIRECTORY, snapshot_one};
use crate::worktree_removal_snapshot_promotion::promote_snapshot;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalInterruptedDestroy";

/// The folder of the common Git directory that holds one record per destroy the wrapper started,
/// named by the registration's id.
pub const RECORD_FOLDER: &str = "monochromatic-git-policy-cli/recorded-destroys";

/// The path of the record of one registration.
fn record_path(common_dir: &Path, id: &OsStr) -> PathBuf {
    return common_dir.join(RECORD_FOLDER).join(id);
}

/// What:
///  Record that this command is about to destroy the files of the worktree whose registration is `id`,
///  so that a later command can finish what this one does not.
/// Why:
///   The record is written before the kernel call,
/// so a kill at any point after it leaves one of two decidable states:
/// the worktree
///  directory is still there (nothing was destroyed,
/// and the record is dropped),
/// or it is gone (the destroy happened and only
///  the registration is left).
/// A record that cannot be written costs the automatic finish and nothing else,
/// so the removal goes
///  on and says so in the diagnostic log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordDestroy(commonDir, id, identity, target);
/// ```
pub fn record_destroy(common_dir: &Path, id: &OsStr, admin: &ObjectId, target: &Path) -> Result<(), String> {
    let path: PathBuf = record_path(common_dir, id);
    if let Some(parent) = path.parent()
        && let Err(error) = create_private_directory_all(parent)
    {
        return Err(format!("{} cannot be made ({error})", parent.display()));
    }
    // The identity first, so the registration a later command empties is the one this destroy was proven against, then the
    // target's own bytes, which need not be text.
    let mut bytes: Vec<u8> = admin.encode().into_bytes();
    bytes.push(b'\n');
    bytes.extend_from_slice(target.as_os_str().as_bytes());
    return match write_private_file_unsynced(path.as_path(), bytes.as_slice()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(format!("{} cannot be written ({error})", path.display())),
    };
}

/// What:
///  Drop the record of one registration,
/// which had nothing left to finish.
/// Why:
///   The caller drops it only after the registration is gone or the destroy is known not to have happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forgetDestroy(commonDir, id);
/// ```
pub fn forget_destroy(common_dir: &Path, id: &OsStr) {
    let path: PathBuf = record_path(common_dir, id);
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            warn(TAG, format!("the record {} cannot be removed ({error}); a later command tries the same thing again", path.display()).as_str());
            return;
        }
    }
    // The folder goes when the last record does; one that still holds another record stays, which its own failure says.
    let folder: PathBuf = common_dir.join(RECORD_FOLDER);
    let _kept = std::fs::remove_dir(folder.as_path());
}

/// What a record holds:
/// the identity of the registration it was written for and the worktree it names.
/// Why:
///   The identity says whether the registration is still the one the destroy was proven against,
/// and the target says
///  whether the destroy happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DestroyRecord = { admin: ObjectId; target: PathBuf };
/// ```
struct DestroyRecord {
    /// The identity of the registration when the destroy was proven.
    admin: ObjectId,
    /// The worktree directory the destroy was aimed at.
    target: PathBuf,
}

/// Whether one byte of a record is the newline that ends its identity line.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// The fields of a record,
/// or the reason it cannot be used.
fn read_record(path: &Path) -> Result<DestroyRecord, String> {
    let bytes: Vec<u8> = match std::fs::read(path) {
        Ok(read) => read,
        Err(error) => return Err(format!("it cannot be read ({error})")),
    };
    let Some(split_at) = bytes.iter().position(is_newline) else {
        return Err(String::from("it has no identity line"));
    };
    let (identity_bytes, rest): (&[u8], &[u8]) = bytes.split_at(split_at);
    let Ok(identity_text) = std::str::from_utf8(identity_bytes) else {
        return Err(String::from("its identity line is not text"));
    };
    let Some(admin) = ObjectId::decode(identity_text) else {
        return Err(format!("its identity line {identity_text} cannot be read"));
    };
    let target: &[u8] = rest.strip_prefix(b"\n").unwrap_or(rest);
    return Ok(DestroyRecord { admin, target: PathBuf::from(OsStr::from_bytes(target)) });
}

/// What:
///  Remove the registration `id` through descriptors below the held `worktrees` directory,
///  only while it is the object the record names and while nothing below it is a nested subvolume or a mount point.
/// Why:
///   The registration is emptied and then removed by name from the held directory,
/// which succeeds only for an empty
///  directory,
/// so a registration that another process replaced is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeRecordedRegistration(worktrees, id, admin, recorded);
/// ```
fn remove_recorded_registration(worktrees: &File, id: &OsStr, admin: &Path, recorded: &ObjectId) -> Result<(), String> {
    let held: File = match open_directory_at(worktrees, id) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{} cannot be opened ({error})", admin.display())),
    };
    let found: ObjectId = step_reason(identity_of(&held), admin, "its identity cannot be read")?;
    if found != *recorded {
        return Err(format!("{} is not the registration the destroy was recorded for, so it is left alone", admin.display()));
    }
    step_reason(verify_below(&held, admin), admin, "it holds something this command does not delete")?;
    step_reason(empty_directory(&held, admin), admin, "its content cannot be removed")?;
    drop(held);
    return step_reason(remove_at(worktrees, id, true), admin, "it cannot be removed from its parent");
}

/// What:
///  The outcome of one step on `shown`,
/// or the sentence for the person when it failed.
/// Why:
///   Every failure of the finish reads the same way,
/// and a named function keeps that shape without a closure per step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stepReason(outcome, shown, 'its content cannot be removed')
/// ```
fn step_reason<T>(outcome: std::io::Result<T>, shown: &Path, what: &str) -> Result<T, String> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("{what} ({error}): {}", shown.display())),
    };
}

/// What:
///  Finish every removal whose record says a destroy happened,
/// and drop every record whose destroy did not happen.
///  Gives what could not be finished,
/// for the person.
/// Why:
///   See the module description.
/// The records are one `read_dir` and absent when no removal was interrupted,
/// so an ordinary command pays
///  one failed open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const problems: string[] = await finishInterruptedDestroys(commonDir);
/// ```
pub fn finish_interrupted_destroys(common_dir: &Path) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();
    let folder: PathBuf = common_dir.join(RECORD_FOLDER);
    let Ok(listing) = std::fs::read_dir(&folder) else { return problems };
    for entry in listing.flatten() {
        let record: PathBuf = entry.path();
        let id: OsString = entry.file_name();
        finish_one(common_dir, record.as_path(), id.as_os_str(), &mut problems);
    }
    return problems;
}

/// Finish one record,
/// or say why it stays.
fn finish_one(common_dir: &Path, record: &Path, id: &OsStr, problems: &mut Vec<String>) {
    let read: DestroyRecord = match read_record(record) {
        Ok(found) => found,
        Err(reason) => {
            // A record that cannot be read cannot be used later either: it is said once and dropped.
            problems.push(format!("The record of an interrupted worktree removal {record:?} cannot be used ({reason}), so that removal is not finished automatically; run `git worktree prune` for the repository."));
            forget_destroy(common_dir, id);
            return;
        }
    };
    let worktrees_path: PathBuf = common_dir.join("worktrees");
    let admin: PathBuf = worktrees_path.join(id);
    if !admin.exists() {
        // The registration went after all: nothing is left to finish.
        forget_destroy(common_dir, id);
        return;
    }
    if read.target.exists() {
        debug(TAG, format!("the worktree {} of the recorded destroy is still there, so nothing was destroyed", read.target.display()).as_str());
        forget_destroy(common_dir, id);
        return;
    }
    // The destroy happened. What the registration holds is copied before it can go: a notice or an unfinished push dies
    // with it otherwise.
    let before: usize = problems.len();
    for problem in snapshot_one(common_dir, admin.as_path()) {
        problems.push(problem);
    }
    if problems.len() > before {
        return;
    }
    let worktrees: File = match open_root(worktrees_path.as_path()) {
        Ok(opened) => opened,
        Err(error) => {
            problems.push(format!("The interrupted removal of {} cannot be finished: {} cannot be opened ({error}). Run `git worktree prune` for the repository.", read.target.display(), worktrees_path.display()));
            return;
        }
    };
    match remove_recorded_registration(&worktrees, id, admin.as_path(), &read.admin) {
        Ok(()) => {}
        Err(reason) => {
            problems.push(format!("The interrupted removal of {} cannot be finished: {reason}. Run `git worktree prune` for the repository.", read.target.display()));
            return;
        }
    }
    // Git removes the `worktrees` directory when the last registration goes; a directory that still holds others stays.
    let _kept = std::fs::remove_dir(worktrees_path.as_path());
    forget_destroy(common_dir, id);
    debug(TAG, format!("finished the removal of {}: its registration {} is gone", read.target.display(), admin.display()).as_str());
    // What the snapshot kept is promoted now; a failure leaves it for the settle of the next command, which sees the Git
    // directory gone.
    let snapshot: PathBuf = common_dir.join(SNAPSHOT_DIRECTORY).join(crate::service_instance::instance_of(admin.as_path()));
    if snapshot.is_dir() {
        promote_snapshot(common_dir, snapshot.as_path(), problems);
    }
}
