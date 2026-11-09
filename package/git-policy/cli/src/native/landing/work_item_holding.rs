//! What:
//!  The three steps of a work item whose producer has not approved it yet:
//! hold it where nothing drains it,
//! promote it when
//!  the producer approves it,
//! and forget it when the producer refuses it.
//! Why:
//!   A landed commit's push must be durable before the command that owes it can be killed,
//! and it must not run
//!  before that command's post-commit gate has decided.
//! A held record is both:
//! no round lists the `held` state,
//! so
//!  only an explicit promotion makes the item runnable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await hold(workDir, item); promote(workDir, id); forgetHeld(workDir, id);
//! ```

use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::work_item::{CLAIMED, DONE, FAILED, HELD, PENDING, RECOVERING, WorkItem, state_directory};
use std::path::{Path, PathBuf};

/// What:
///  Holds one item where nothing drains it,
/// until its producer promotes it.
/// Why:
///   A landed commit's push must be durable before the command that owes it can be killed,
/// and it must not run
///  before that command's post-commit gate has decided:
/// a held record is both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await hold(workDir, item)
/// ```
///
/// # Errors
///
/// The work queue's own words when the record cannot be written.
pub fn hold(work_dir: &Path, item: &WorkItem) -> Result<(), String> {
    // A gate that blocked this commit keeps it local: no later hold, recovery included, may publish it.
    if is_blocked(work_dir, item.id().as_str()) {
        return Ok(());
    }
    // The owner is published before the record, so no reader ever meets a held record that says no one holds it.
    claim(work_dir, item.id().as_str());
    if let Err(error) = create_private_directory_all(state_directory(work_dir, HELD).as_path()) {
        return Err(format!("creating the {HELD} directory failed: {error}"));
    }
    let path: PathBuf = state_directory(work_dir, HELD).join(item.id());
    return match write_private_file_unsynced(path.as_path(), item.encode().as_bytes()) {
        Ok(()) => Ok(()),
        // The same commit's push is already held; the gate has not decided either way.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(format!("writing the held record failed: {error}")),
    };
}

/// What:
///  Makes a held item runnable,
/// whole by one rename,
/// and answers whether anything was held.
/// Why:
///   The promotion is the producer's approval,
/// so a record that is not there is no approval and the caller says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// promote(workDir, id) // rename held/<id> to pending/<id>
/// ```
pub fn promote(work_dir: &Path, id: &str) -> bool {
    if !create_states(work_dir) {
        return false;
    }
    if std::fs::rename(state_directory(work_dir, HELD).join(id), state_directory(work_dir, PENDING).join(id)).is_err() {
        return false;
    }
    // The decision is made: the owner's name over it is no longer needed.
    let _gone: std::io::Result<()> = std::fs::remove_file(owner_path(work_dir, id));
    return true;
}

/// The extension of the file that names the process which holds an item until its gate decides.
const OWNER_EXTENSION: &str = "owner";


/// The state directory of the blocked verdicts.
pub const BLOCKED: &str = "blocked";

/// The path of one held item's owner record.
fn owner_path(work_dir: &Path, id: &str) -> PathBuf {
    return state_directory(work_dir, HELD).join(format!("{id}.{OWNER_EXTENSION}"));
}

/// What:
///  Names this process as the one whose gate decides a held item.
/// Why:
///   A held item is a push whose command has not approved it yet.
/// Any other command that meets it may promote it only when this
///  process is proven gone,
/// so a living gate is never overtaken and a dead one never strands a push.
/// The record
///  is this process's number and its birth identity,
/// judged by `process_birth_identity`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claimHeld(workDir, id) // write `<pid> <birth identity>`
/// ```
pub fn claim(work_dir: &Path, id: &str) {
    // The identity is the one another process's own is judged by, so the two are compared as one value.
    let own: i64 = i64::from(std::process::id());
    let identity: String = match crate::process_identity::process_birth_identity(own) {
        Ok(Some(identity)) => identity,
        Ok(None) => {
            crate::service_warn!("the held push record {id} cannot be claimed: this process has no birth identity, so another command may promote it before its gate");
            return;
        }
        Err(error) => {
            crate::service_warn!("the held push record {id} cannot be claimed ({error:?}), so another command may promote it before its gate");
            return;
        }
    };
    let text: String = format!("{}\n{identity}\n", std::process::id());
    let _written: std::io::Result<()> = write_private_file_unsynced(owner_path(work_dir, id).as_path(), text.as_bytes());
}

/// The owner one owner-shaped file names.
fn owner_of_file(path: &Path) -> Option<(i64, String)> {
    let text: String = std::fs::read_to_string(path).ok()?;
    let mut lines = text.lines();
    let pid: i64 = lines.next()?.trim().parse().ok()?;
    let identity: String = lines.next()?.trim().to_string();
    return Some((pid, identity));
}

/// What:
///  Makes a held item runnable only when the command that held it is proven gone.
/// Why:
///   This is the promotion another command performs,
/// and it never overtakes a living gate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// promoteProven(workDir, id)
/// ```
pub fn promote_proven(work_dir: &Path, id: &str) -> bool {
    if is_blocked(work_dir, id) {
        return false;
    }
    // The right to promote this record is taken by one exclusive name, and the owner is judged from the file taken with it: a claim
    //  that lands after that point recreates the owner file, and its presence stops this promotion.
    let claiming: PathBuf = state_directory(work_dir, HELD).join(format!("{id}.promoting"));
    if create_private_directory_all(state_directory(work_dir, HELD).as_path()).is_err()
        || write_private_file_unsynced(claiming.as_path(), b"promoting\n").is_err()
    {
        return false;
    }
    let judging: PathBuf = state_directory(work_dir, HELD).join(format!("{id}.judging"));
    let taken: bool = std::fs::rename(owner_path(work_dir, id), judging.as_path()).is_ok();
    let gone: bool = match taken {
        true => {
            let judged: bool = match owner_in(judging.as_path()) {
                Some((pid, recorded)) => match crate::process_identity::process_birth_identity(pid) {
                    Ok(None) => true,
                    Ok(Some(now)) => now != recorded,
                    Err(_) => false,
                },
                None => false,
            };
            let _gone_file: std::io::Result<()> = std::fs::remove_file(judging.as_path());
            judged
        }
        false => false,
    };
    // A claim written after the owner was taken means a command owns it now.
    let reclaimed: bool = owner_path(work_dir, id).exists();
    let promoted: bool = gone && !reclaimed && promote(work_dir, id);
    let _released: std::io::Result<()> = std::fs::remove_file(claiming.as_path());
    return promoted;
}

/// What:
///  Says durably that the gate of this commit kept it local,
/// and gives the held record up.
/// Why:
///   `forget_the_push` alone leaves no trace of the verdict,
/// so a later recovery of the same batch would hold the push again
///  and publish a commit the gate refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// block(workDir, id) // write `blocked/<id>`, hold nothing
/// ```
pub fn block(work_dir: &Path, id: &str) {
    if create_private_directory_all(state_directory(work_dir, BLOCKED).as_path()).is_err() {
        crate::service_error!("the blocked verdict of the push record {id} cannot be kept");
        return;
    }
    let _written: std::io::Result<()> = write_private_file_unsynced(state_directory(work_dir, BLOCKED).join(id).as_path(), b"blocked\n");
    forget_held(work_dir, id);
}

/// Whether the gate of this commit kept it local.
fn is_blocked(work_dir: &Path, id: &str) -> bool {
    return state_directory(work_dir, BLOCKED).join(id).exists();
}

/// The owner one file names,
/// as its process and its birth identity.
fn owner_in(path: &Path) -> Option<(i64, String)> {
    return owner_of_file(path);
}

/// What:
///  Creates every state directory a claimed item passes through.
/// Why:
///   A claim moves an item into `recovering` and then into `claimed`,
/// and its settlement into `done` or `failed`;
/// a
///  worktree whose first record was held has none of them,
/// so a promotion that made only `pending` would leave an approved
///  push that no consumer can ever claim.
/// The set is the one `enqueue_all` makes,
/// because both are the queue's own directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createStates(workDir) // pending, claimed, done, failed, recovering
/// ```
fn create_states(work_dir: &Path) -> bool {
    for state in [PENDING, HELD, BLOCKED, CLAIMED, DONE, FAILED, RECOVERING] {
        if create_private_directory_all(state_directory(work_dir, state).as_path()).is_err() {
            return false;
        }
    }
    return true;
}

/// What:
///  Makes every held record of a worktree runnable,
/// and gives how many there were.
/// Why:
///   A held record is a push whose command died before its post-commit gate decided,
/// and nothing drains it.
/// A
///  service start and the next command in the worktree are the moments when no gate of a killed command is
///  still deciding,
/// so an approved or abandoned push is never stranded.
/// A record whose gate is still running in a living
///  command is rare and is overtaken here,
/// which is the price of never stranding one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// promoteHeld(workDir) // the count promoted
/// ```
pub fn promote_held(work_dir: &Path) -> usize {
    let Ok(listing) = std::fs::read_dir(state_directory(work_dir, HELD)) else {
        return 0;
    };
    let mut promoted: usize = 0;
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(OWNER_EXTENSION) || name.ends_with("promoting") || name.ends_with("judging") {
            continue;
        }
        if promote_proven(work_dir, name.as_str()) {
            promoted += 1;
        }
    }
    return promoted;
}

/// Forgets a held item whose producer refused it;
/// one that is already promoted is not reached.
pub fn forget_held(work_dir: &Path, id: &str) {
    let _removed: std::io::Result<()> = std::fs::remove_file(state_directory(work_dir, HELD).join(id));
}
