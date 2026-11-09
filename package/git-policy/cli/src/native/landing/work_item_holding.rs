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

/// The owner a held item names:
/// its process and its birth identity.
fn owner_of(work_dir: &Path, id: &str) -> Option<(i64, String)> {
    let text: String = std::fs::read_to_string(owner_path(work_dir, id)).ok()?;
    let mut lines = text.lines();
    let pid: i64 = lines.next()?.trim().parse().ok()?;
    let identity: String = lines.next()?.trim().to_string();
    return Some((pid, identity));
}

/// What:
///  Whether the process that holds an item is proven gone,
/// so that another command may promote it.
/// Why:
///   A record with no owner is an item whose command died before it could claim it.
/// A record whose process
///  runs under another birth identity is one whose number was used again.
/// A process that cannot be judged is
///  not gone:
/// its gate may still decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownerIsGone(workDir, id) // true when absent, or its process is gone or was born again
/// ```
fn owner_is_gone(work_dir: &Path, id: &str) -> bool {
    let Some((pid, recorded)) = owner_of(work_dir, id) else { return true };
    return match crate::process_identity::process_birth_identity(pid) {
        Ok(None) => true,
        Ok(Some(now)) => now != recorded,
        // A process in another namespace, or a `/proc` that refuses: nothing is decided.
        Err(_) => false,
    };
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
    if !owner_is_gone(work_dir, id) {
        return false;
    }
    return promote(work_dir, id);
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
    for state in [PENDING, HELD, CLAIMED, DONE, FAILED, RECOVERING] {
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
        if name.ends_with(OWNER_EXTENSION) {
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
