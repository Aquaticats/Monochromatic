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
    if let Err(error) = create_private_directory_all(state_directory(work_dir, HELD).as_path()) {
        return Err(format!("creating the {HELD} directory failed: {error}"));
    }
    let path: PathBuf = state_directory(work_dir, HELD).join(item.id());
    return match write_private_file_unsynced(path.as_path(), item.encode().as_bytes()) {
        Ok(()) => {
            say_held(item);
            Ok(())
        }
        // The same commit's push is already held; the gate has not decided either way.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(format!("writing the held record failed: {error}")),
    };
}

/// What:
///  Writes the worktree's notice about a push that is held,
/// where it is held.
/// Why:
///   The notice exists from the moment the push is held,
/// so the next command shows it whether or not any notice
///  directory existed before,
/// and takes it once;
/// releasing the push or its gate's verdict clears it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayHeld(item) // record a push-skip notice for the held commit
/// ```
fn say_held(item: &WorkItem) {
    let said: String = crate::service_handoff::held_push_line(item.branch_ref.as_str(), item.oid.as_str());
    if let Err(reason) = crate::push_failure_notice::record_push_skip(item.git_dir.as_path(), item.branch_ref.as_str(), item.oid.as_str(), said.as_str()) {
        crate::service_warn!("the held push of {} cannot be said ({reason})", item.oid);
    }
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
    return true;
}

/// The state directory of the blocked verdicts.
pub const BLOCKED: &str = "blocked";

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

/// Forgets a held item whose producer refused it;
/// one that is already promoted is not reached.
pub fn forget_held(work_dir: &Path, id: &str) {
    let _removed: std::io::Result<()> = std::fs::remove_file(state_directory(work_dir, HELD).join(id));
}
