//! What:
//!  The wrapper's side of after-commit work: leave the push and upkeep records for a commit the service landed and wake
//!  the service.
//! Why:
//!  The commit returns as soon as it is saved. The push (which can take as long as the network does) and the upkeep run
//!  in the service afterwards; a push that fails is reported by the person's next wrapper command (`push_failure_notice.rs`)
//!  (`doc/planning/cli-git-resident-service.md`, "After-commit work"). The wrapper starts no program and waits for no
//!  one: it writes whole files and sends one message.
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
use crate::work_item::{WorkItem, WorkKind, enqueue};
use std::io::Write;
use std::path::PathBuf;

/// What the wrapper knows about a commit the service landed.
#[derive(Clone, Debug)]
pub struct ServiceHandoff {
    /// How to reach the service.
    pub config: ClientConfig,
    /// The worktree root.
    pub repository: PathBuf,
    /// The worktree's own Git directory, which holds the work records and names the service instance.
    pub git_dir: PathBuf,
    /// The branch the commit went to.
    pub reference: String,
    /// The branch value the commit was made on.
    pub parent: String,
}

/// The kinds of work a landed commit leaves for the service.
pub const HANDED_OVER: [WorkKind; 2] = [WorkKind::Push, WorkKind::Maintenance];

/// What:
///  Writes the work records for a landed commit and wakes the service.
/// Why:
///   The records are written before the message is sent, so a service that wakes always finds them; records that were
///  written but not announced stay pending and are run by the next wake for the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOver(handoff, landed)
/// ```
pub fn hand_over(handoff: &ServiceHandoff, landed: &str, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let key: Option<ReviewKey> = key_from_environment(&|name| { return std::env::var(name).ok(); });
    return hand_over_with_key(handoff, landed, key.as_ref(), notices);
}

/// What:
///  `hand_over` with the standards-review credential given. The records written to disk never hold it; the wake message
///  carries it to the service, which keeps it in memory for the review.
/// Why:
///   A systemd user service does not inherit the wrapper's environment, and the credential must not be stored in a work
///  item, a unit file or the manager's environment. The wrapper therefore sends it over the user-only socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverWithKey(handoff, landed, key)
/// ```
pub fn hand_over_with_key(handoff: &ServiceHandoff, landed: &str, review_key: Option<&ReviewKey>, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let work_dir: PathBuf = work_directory(handoff.git_dir.as_path());
    // A review is left only where the repository keeps a shadow log; the file is the repository's own switch.
    let review: &[WorkKind] = if has_shadow_log(handoff.repository.as_path()) { &[WorkKind::Review] } else { &[] };
    for kind in HANDED_OVER.iter().chain(review.iter()).copied() {
        let item: WorkItem = WorkItem {
            kind,
            repository: handoff.repository.display().to_string(),
            git_dir: handoff.git_dir.display().to_string(),
            branch_ref: handoff.reference.clone(),
            oid: landed.to_string(),
            parent: handoff.parent.clone(),
            created: now_iso(),
            tries: 0,
        };
        enqueue(work_dir.as_path(), &item).map_err(ClientFailure::WorkNotHandedOver)?;
    }
    return wake(&handoff.config, handoff.git_dir.as_path(), review_key, notices);
}

/// What:
///  Writes the push record for a branch tip that a command other than a commit moved, and wakes the service.
/// Why:
///   A merge, a cherry-pick, a revert, an applied patch or a pull's merge adds commits that the next commit would otherwise
///  be the first to push. Only the push is left: the review and the upkeep belong to a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverPush(handoff, tip)
/// ```
pub fn hand_over_push(handoff: &ServiceHandoff, tip: &str, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let item: WorkItem = WorkItem {
        kind: WorkKind::Push,
        repository: handoff.repository.display().to_string(),
        git_dir: handoff.git_dir.display().to_string(),
        branch_ref: handoff.reference.clone(),
        oid: tip.to_string(),
        parent: handoff.parent.clone(),
        created: now_iso(),
        tries: 0,
    };
    enqueue(work_directory(handoff.git_dir.as_path()).as_path(), &item).map_err(ClientFailure::WorkNotHandedOver)?;
    return wake(&handoff.config, handoff.git_dir.as_path(), None, notices);
}

/// Handoff controls stay out of the release executable.
#[cfg(test)]
#[path = "service_handoff_tests.rs"]
mod tests;
