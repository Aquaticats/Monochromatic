//! What:
//!  The wrapper's side of after-commit work: leave the push and upkeep records for a commit the service landed, wake
//!  the service, and show the notices that earlier work left.
//! Why:
//!  The commit returns as soon as it is saved. The push (which can take as long as the network does) and the upkeep run
//!  in the service afterwards; a push that fails is reported at the person's next commit through the service
//!  (`doc/planning/cli-git-resident-service.md`, "After-commit work"). The wrapper starts no program and waits for no
//!  one: it writes whole files and sends one message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await enqueue(workDir, push); await enqueue(workDir, maintenance); await wake(commonDir);
//! ```

use super::iso_time::now_iso;
use super::service_client::{ClientConfig, wake};
use super::service_messages::ClientFailure;
use super::service_work::{take_notices, work_directory};
use super::standards_review_input::has_shadow_log;
use super::work_item::{WorkItem, WorkKind, enqueue};
use std::io::Write;
use std::path::PathBuf;

/// What the wrapper knows about a commit the service landed.
#[derive(Clone, Debug)]
pub struct ServiceHandoff {
    /// How to reach the service.
    pub config: ClientConfig,
    /// The worktree root.
    pub repository: PathBuf,
    /// The common Git directory.
    pub common_dir: PathBuf,
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
    let work_dir: PathBuf = work_directory(handoff.common_dir.as_path());
    // A review is left only where the repository keeps a shadow log; the file is the repository's own switch.
    let review: &[WorkKind] = if has_shadow_log(handoff.repository.as_path()) { &[WorkKind::Review] } else { &[] };
    for kind in HANDED_OVER.iter().chain(review.iter()).copied() {
        let item: WorkItem = WorkItem {
            kind,
            repository: handoff.repository.display().to_string(),
            git_dir: handoff.common_dir.display().to_string(),
            branch_ref: handoff.reference.clone(),
            oid: landed.to_string(),
            parent: handoff.parent.clone(),
            created: now_iso(),
            tries: 0,
        };
        enqueue(work_dir.as_path(), &item).map_err(ClientFailure::WorkNotHandedOver)?;
    }
    return wake(&handoff.config, handoff.common_dir.as_path(), notices);
}

/// Shows and removes the notices that earlier work left for this repository.
pub fn show_notices(handoff: &ServiceHandoff, out: &mut dyn Write) {
    for text in take_notices(handoff.common_dir.as_path()) {
        // A closed terminal is not a reason to fail a commit that already landed.
        let _written = writeln!(out, "{text}");
    }
}

/// Handoff controls stay out of the release executable.
#[cfg(test)]
#[path = "service_handoff_tests.rs"]
mod tests;
