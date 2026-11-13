//! What:
//!  Finishing the removal a record names once its destroy happened,
//!  and saying what could not be finished,
//!  in the words a
//!  person acts on.
//! Why:
//!   A command killed between the kernel's destroy and the removal of the registration leaves a record and a
//!  registration,
//! and the files are gone:
//! an ordinary `git worktree prune` would forget the registration and take the
//!  index,
//! the reflog and the per-worktree refs with it.
//! The state is decidable because the record says
//!  whether the destroy was intended or returned,
//! and because the place a handed-over subvolume takes is written before the rename
//!  (`worktree_removal_interrupted_destroy.rs`,
//! `record_leaving`).
//!  The record is written and read by that module;
//! what decides an unfinished removal is here,
//! so each file stays
//!  one subject.
//!  Nothing here removes a name it was given without proving what stands there:
//! the registration is opened below
//!  the held `worktrees` directory,
//! its identity is compared with the one the record wrote,
//! the tree below it is
//!  refused when it holds a volume this program did not make,
//! and the snapshot the rescue copied is promoted only
//!  after the registration is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const problems: string[] = await finishInterruptedDestroys(commonDir); // what a person still has to act on
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::{ObjectId, handle_path, identity_of, open_directory_at, open_root, remove_at};
use crate::plain_file::read_plain_file;
use crate::process_identity::process_birth_identity;
use crate::subvolume_remove_walk::{empty_directory, verify_below};
use crate::worktree_removal_clears_the_ahead_pool::clear_the_pools_ahead_of_a_worktree;
use crate::worktree_removal_interrupted_destroy::{DestroyRecord, RECORD_FOLDER, RECORD_LIMIT, forget_destroy, read_record};
use crate::worktree_removal_rescue::{SNAPSHOT_DIRECTORY, snapshot_one};
use crate::worktree_removal_snapshot_promotion::promote_snapshot;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalInterruptedDestroy";

/// What:
///  Whether the command that wrote a record may still be running,
/// which a record without an owner,
/// or one whose process
///  cannot be inspected,
/// counts as.
/// Why:
///   A removal writes its record,
/// destroys the tree and marks the record;
/// a second command that reads the record in that
///  time and forgets it takes away the only place the first can mark its success,
/// so the next command has nothing to
///  finish.
/// A process number alone can be reused,
/// so the birth identity written beside it decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const running = recordOwnerMayBeRunning(record.owner); // true when nothing proves the owner left
/// ```
fn record_owner_may_be_running(owner: Option<&String>) -> bool {
    let Some(spelled) = owner else { return true };
    let Some((spelled_pid, identity)) = spelled.split_once(':') else { return true };
    let Ok(pid) = spelled_pid.parse::<i64>() else { return true };
    return match process_birth_identity(pid) {
        Ok(Some(now)) => now == identity,
        Ok(None) => false,
        Err(error) => {
            warn(TAG, format!("the process {pid} of a recorded destroy cannot be inspected: {error}").as_str());
            true
        }
    };
}

/// What:
///  The report for a registration whose `gitdir` names a checkout other than the path the record holds,
///  or nothing when it names
///  that path.
/// Why:
///   The record says which path the destroy was aimed at.
/// A registration a repair pointed at a restored checkout is
///  live elsewhere,
/// and its `.git` file names it:
/// emptying it would take that checkout's index and reflog.
/// A `gitdir` that
///  cannot be read is treated the same way,
/// because nothing proves the registration is this record's any more;
/// a `gitdir` that is not there is not,
/// because a registration no checkout points at is one this command's own removal already emptied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = namedCheckout(admin, target); // undefined when the registration names target
/// ```
fn named_checkout(admin: &Path, target: &Path) -> Option<String> {
    let named: Vec<u8> = match read_plain_file(&admin.join("gitdir"), RECORD_LIMIT) {
        Ok(read) => read,
        // No back-pointer at all is the state the removal being finished leaves when it empties a registration and then
        // cannot remove its directory: no checkout points at it, so finishing it discards nothing.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        // An answer that could not be read proves nothing about which checkout uses the registration, and a registration
        // whose directory is writable can be emptied through it: the checkout that may use it keeps its index and logs.
        Err(error) => {
            return Some(format!(
                "The registration {} cannot be read ({error}), so nothing proves which checkout it belongs to now, and the record of its interrupted removal names {}. Nothing was removed, so a checkout that uses that registration keeps its index and its logs; the record of the interrupted removal is dropped, and `git worktree prune` forgets a registration nothing uses.",
                admin.display(),
                target.display()
            ));
        }
    };
    let recorded: &[u8] = named.strip_suffix(b"\n".as_slice()).unwrap_or(named.as_slice());
    let spelled: PathBuf = PathBuf::from(OsStr::from_bytes(recorded));
    let resolved: PathBuf = if spelled.is_absolute() { spelled } else { admin.join(spelled) };
    let checkout: Option<&Path> = resolved.parent();
    if checkout == Some(target) {
        return None;
    }
    let named_checkout: String = match checkout {
        Some(shown) => shown.display().to_string(),
        None => String::from("nothing"),
    };
    return Some(format!(
        "The registration {} names the checkout {named_checkout} now, and the record of its interrupted removal names {}. Nothing was removed, so that checkout keeps its index and its logs; the record of the interrupted removal is dropped.",
        admin.display(),
        target.display()
    ));
}

/// What:
///  Remove the registration `id` of the repository whose common directory is `common_dir`,
/// through descriptors below the held `worktrees` directory,
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
fn remove_recorded_registration(common_dir: &Path, worktrees: &File, id: &OsStr, admin: &Path, recorded: &ObjectId) -> Result<(), String> {
    let held: File = match open_directory_at(worktrees, id) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{} cannot be opened ({error})", admin.display())),
    };
    let found: ObjectId = step_reason(identity_of(&held), admin, "its identity cannot be read")?;
    if found != *recorded {
        return Err(format!("{} is not the registration the destroy was recorded for, so it is left alone", admin.display()));
    }
    // The snapshots ahead of an add are this program's own and stand in the registration, and the walk below refuses every
    // nested subvolume, so they go first: a removal this program began on an earlier build left them here, and nothing a
    // person put in the registration is touched by this clear.
    if let Err(reason) = clear_the_pools_ahead_of_a_worktree(common_dir, handle_path(&held).as_path()) {
        return Err(format!("its snapshots ahead of an add cannot be removed ({reason}): {}", admin.display()));
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
    // What a directory entry is, before what it holds: a named pipe, a device or a link is not this command's record, and
    // opening one would wait for a writer that never comes, on every wrapped command. `read_plain_file` opens without
    // following a link and without waiting, and refuses anything that is not a plain file.
    let bytes: Vec<u8> = match read_plain_file(record, RECORD_LIMIT) {
        Ok(read) => read,
        Err(error) => {
            debug(TAG, format!("{} is not a plain record and is left alone ({error})", record.display()).as_str());
            return;
        }
    };
    let read: DestroyRecord = match read_record(bytes.as_slice()) {
        Ok(found) => found,
        Err(reason) => {
            // A record that cannot be read cannot be used later either: it is said once and dropped.
            problems.push(format!("The record of an interrupted worktree removal {record:?} cannot be used ({reason}), so that removal is not finished automatically; run `git worktree prune` for the repository."));
            forget_destroy(common_dir, id);
            return;
        }
    };
    if !read.destroyed && record_owner_may_be_running(read.owner.as_ref()) {
        // Another command is removing that worktree now, and its record is what it marks when the kernel call returns, so
        // nothing here reads, decides or forgets anything yet. A marked record is finished even while its owner lives: the
        // call it names has returned, and the owner's own removal of the registration is what finishing completes.
        debug(TAG, format!("the record {} was written by a command that may still be running, so it is left alone", record.display()).as_str());
        return;
    }
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
    // What the registration names now, before the record's path is trusted: a repair can point it at a restored checkout,
    // whose index, reflog and per-worktree refs are live, and emptying it would leave that checkout's `.git` dangling.
    if let Some(named) = named_checkout(admin.as_path(), read.target.as_path()) {
        problems.push(named);
        forget_destroy(common_dir, id);
        return;
    }
    // A removal that handed its subvolume to the service writes the place it renamed that subvolume to before the rename,
    // so a path with nothing at it and a place named is a removal killed between the rename and its mark: the delete of
    // that place was recorded before the rename, and only the registration is left to finish (a review finding, F4).
    if !read.destroyed && read.leaving.is_some() {
        debug(TAG, format!("the record {} names where its files stand and was not marked, so its registration is finished as a marked record's is", record.display()).as_str());
    }
    if !read.destroyed && read.leaving.is_none() {
        // Nothing is at the recorded name, and the record says only that a destroy was intended: the worktree may have been
        // moved, and its registration is the only thing that can still name its index and logs. The person is told once.
        problems.push(format!(
            "The removal of the worktree {} did not finish before its files were destroyed: nothing is at that path, and {} records only that the destroy was intended. Nothing was removed; if the worktree was moved, `git worktree repair <new path>` puts it right, and if it is gone, `git worktree prune` forgets the registration {}. The record of the interrupted removal is dropped.",
            read.target.display(),
            record.display(),
            admin.display()
        ));
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
    match remove_recorded_registration(common_dir, &worktrees, id, admin.as_path(), &read.admin) {
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
