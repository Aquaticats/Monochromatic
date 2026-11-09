//! What:
//!  The wrapper's side of after-commit work:
//! leave the push and upkeep records for a commit the service landed and wake
//!  the service.
//! Why:
//!  The commit returns as soon as it is saved.
//! The push (which can take as long as the network does) and the upkeep run
//!  in the service afterwards;
//! a push that fails is reported by the person's next wrapper command (`push_failure_notice.rs`)
//!  (`doc/planning/cli-git-resident-service.md`,
//! "After-commit work").
//! The wrapper starts no program and waits for no
//!  one:
//! it writes whole files and sends one message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await enqueue(workDir, push); await enqueue(workDir, maintenance); await wake(commonDir);
//! ```

use crate::iso_time::now_iso;
use crate::service_client::{ClientConfig, wake};
use crate::service_messages::ClientFailure;
use crate::service_work::work_directory;
use crate::service_review_key::{ReviewKey, key_from_environment};
use crate::standards_review_input::has_shadow_log;
use crate::work_item::{WorkItem, WorkKind, enqueue, enqueue_all, state_directory};
use std::io::Write;
use std::path::PathBuf;

/// What the wrapper knows about a commit the service landed.
#[derive(Clone, Debug)]
pub struct ServiceHandoff {
    /// How to reach the service.
    pub config: ClientConfig,
    /// The worktree root.
    pub repository: PathBuf,
    /// The worktree's own Git directory,
    /// which holds the work records and names the service instance.
    pub git_dir: PathBuf,
    /// The branch the commit went to.
    pub reference: String,
    /// The branch value the commit was made on.
    pub parent: String,
}

/// The kinds of work a landed commit leaves for the service.
pub const HANDED_OVER: [WorkKind; 1] = [WorkKind::Maintenance];

/// What:
///  The value of an environment variable,
/// nothing when it is unset or not text.
/// Why:
///  `hand_over` reads the review key from the process environment through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const environmentVariable = (name: string) => process.env[name];
/// ```
fn environment_variable(name: &str) -> Option<String> {
    return std::env::var(name).ok();
}

/// What:
///  Writes the work records for a landed commit and wakes the service.
/// Why:
///   The records are written before the message is sent,
/// so a service that wakes always finds them;
/// records that were
///  written but not announced stay pending and are run by the next wake for the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOver(handoff, landed)
/// ```
pub fn hand_over(handoff: &ServiceHandoff, landed: &str, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let key: Option<ReviewKey> = key_from_environment(&environment_variable);
    return hand_over_with_key(handoff, landed, key.as_ref(), notices);
}

/// What:
///  `hand_over` with the standards-review credential given.
/// The records written to disk never hold it;
/// the wake message
///  carries it to the service,
/// which keeps it in memory for the review.
/// Why:
///   A systemd user service does not inherit the wrapper's environment,
/// and the credential must not be stored in a work
///  item,
/// a unit file or the manager's environment.
/// The wrapper therefore sends it over the user-only socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverWithKey(handoff, landed, key)
/// ```
pub fn hand_over_with_key(handoff: &ServiceHandoff, landed: &str, review_key: Option<&ReviewKey>, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let work_dir: PathBuf = work_directory(handoff.git_dir.as_path());
    // A review is left only where the repository keeps a shadow log; the file is the repository's own switch.
    let review: &[WorkKind] = if has_shadow_log(handoff.repository.as_path()) { &[WorkKind::Review] } else { &[] };
    // One round for the commit's records: each state directory the queue asks about is listed once, however many kinds the
    //  commit leaves.
    let mut items: Vec<WorkItem> = Vec::new();
    for kind in HANDED_OVER.iter().chain(review.iter()).copied() {
        items.push(WorkItem {
            kind,
            repository: handoff.repository.clone(),
            git_dir: handoff.git_dir.clone(),
            branch_ref: handoff.reference.clone(),
            oid: landed.to_string(),
            parent: handoff.parent.clone(),
            created: now_iso(),
            tries: 0,
        });
    }
    enqueue_all(work_dir.as_path(), items.as_slice()).map_err(ClientFailure::WorkNotHandedOver)?;
    return wake(&handoff.config, handoff.git_dir.as_path(), review_key, notices);
}

/// What:
///  Promotes the push record the service held for a commit it landed,
/// once the post-commit gate approves the backup.
/// Why:
///   The record must outlive a wrapper killed between the answer and the gate,
/// and it must not run before the gate:
///  the service writes no record of its own,
/// so nothing pushes a commit whose gate blocked it,
/// and a record the gate
///  then allows is the one the hand-off wakes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await leaveThePush(handoff, landed)
/// ```
///
/// # Errors
///
/// The work queue's own words when the record cannot be written.
pub fn promote_the_push(handoff: &ServiceHandoff, landed: &str) -> Result<(), ClientFailure> {
    let item: WorkItem = push_item(handoff, landed);
    // The push is released, so the line that says it is held must not be shown.
    crate::push_failure_notice::clear_push_failures(handoff.git_dir.as_path(), handoff.reference.as_str());
    if crate::work_item::promote(work_directory(handoff.git_dir.as_path()).as_path(), item.id().as_str()) {
        return Ok(());
    }
    // Nothing was held for this commit: the record is written here instead, so the push is not lost.
    return match enqueue(work_directory(handoff.git_dir.as_path()).as_path(), &item) {
        Ok(_enqueued) => Ok(()),
        Err(reason) => Err(ClientFailure::WorkNotHandedOver(reason)),
    };
}

/// What:
///  Takes the push record back when the post-commit gate blocks the backup.
/// Why:
///   A blocked gate keeps the commit local,
/// and a record left behind would push it at the next wake.
/// A record another round already
///  claimed is not reached here:
/// the gate is what says whether the push may run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetThePush(handoff, landed) // unlink the pending record
/// ```
pub fn forget_the_push(handoff: &ServiceHandoff, landed: &str) {
    crate::push_failure_notice::clear_push_failures(handoff.git_dir.as_path(), handoff.reference.as_str());
    let work_dir: std::path::PathBuf = work_directory(handoff.git_dir.as_path());
    let id: String = push_item(handoff, landed).id();
    // The verdict is durable: a later recovery of the same batch must not hold the push again.
    crate::work_item::block(work_dir.as_path(), id.as_str());
    let _removed: std::io::Result<()> = std::fs::remove_file(state_directory(work_dir.as_path(), crate::work_item::PENDING).join(id));
}

/// The push record of one landed commit of a worktree.
fn push_item(handoff: &ServiceHandoff, landed: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: handoff.repository.clone(),
        git_dir: handoff.git_dir.clone(),
        branch_ref: handoff.reference.clone(),
        oid: landed.to_string(),
        parent: handoff.parent.clone(),
        created: now_iso(),
        tries: 0,
    };
}

/// What:
///  Writes the push record for a branch tip that a command other than a commit moved,
/// before anything asks whether a service can be reached.
/// Why:
///   A merge,
/// a cherry-pick,
/// a revert,
/// an applied patch or a pull's merge adds commits that the next commit would otherwise
///  be the first to push.
/// Only the push is left:
/// the review and the upkeep belong to a commit.
///   The record is a directory of this worktree's own Git directory,
/// so writing it needs no service,
/// no socket and no session bus,
/// and a context with no route to the service still leaves the push on disk for the next wake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePushRecord({ repository, gitDir, reference, tip, parent })
/// ```
pub fn write_push_record(repository: &std::path::Path, git_dir: &std::path::Path, reference: &str, tip: &str, parent: &str) -> Result<(), ClientFailure> {
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: repository.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: reference.to_string(),
        oid: tip.to_string(),
        parent: parent.to_string(),
        created: now_iso(),
        tries: 0,
    };
    enqueue(work_directory(git_dir).as_path(), &item).map_err(ClientFailure::WorkNotHandedOver)?;
    return Ok(());
}

/// What:
///  `write_push_record` for a handoff,
/// and then the wake of the service.
/// Why:
///   The caller that already knows how to reach the service leaves both in one call;
/// a caller that may find no route writes the record with `write_push_record` and wakes only when it has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverPush(handoff, tip)
/// ```
pub fn hand_over_push(handoff: &ServiceHandoff, tip: &str, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    write_push_record(handoff.repository.as_path(), handoff.git_dir.as_path(), handoff.reference.as_str(), tip, handoff.parent.as_str())?;
    return wake(&handoff.config, handoff.git_dir.as_path(), None, notices);
}

/// Handoff controls stay out of the release executable.
#[cfg(test)]
#[path = "service_handoff_tests.rs"]
mod tests;

/// What:
///  The one sentence the worktree says about a held push.
/// Why:
///   What is known is that the push is held and that the next push of its branch carries the commit.
/// Whether the
///  command that held it has stopped cannot be told,
/// so the sentence does not claim it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heldPushLine(branchRef, commit)
/// ```
pub fn held_push_line(reference: &str, commit: &str) -> String {
    return format!("the commit {commit} is held: its push has not been released yet, and it goes out with the next push from {reference}");
}
