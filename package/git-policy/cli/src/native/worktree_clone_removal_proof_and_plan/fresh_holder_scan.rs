//! What:
//!  One fresh all-namespace holder scan, made immediately before a destroy request,
//!  as a seam the removal calls and a control answers.
//! Why:
//!  The design requires fresh root all-namespace verification before the first snapshot delete
//!  and before the head destroy
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//!  steps 3 and 5;
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 9's notes),
//!  and the wrapper names the holders before each destroy
//!  (`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`).
//!  The scan reads `/proc` and changes nothing,
//!  and every scan answers under a nonce of its own,
//!  never the clone's request nonce,
//!  because two scans of one removal would otherwise share one.
//!  An incomplete read is reported and does not refuse by itself:
//!  the destroy-anyway decision accepts that residual,
//!  and a scan that saw no holder never proves there is none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scan = holders.holdersOf(clone, cloneGuid);
//! if (scan.holders.length) refuse(scan);
//! ```
use crate::clone_holder_scan::holder_scan::{HolderScan, HolderScanRequest, scan_holders};
use crate::foundation::iso_time::now_iso;
use crate::mount_namespace_scan::mount_namespace_tasks::RootIdentity;
use crate::mount_namespace_scan::namespace_target;
use crate::worktree_clone_mount_handler::holder_scan_mode::{PROC_ROOT, SELF_TASK};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::Path;

/// What:
///  The one question a removal asks the mount namespaces.
/// Why:
///  A control answers it from a value and the release answers it from `/proc`,
///  and neither the caller nor the refusal reads more than the scan's own result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface FreshHolderScan { holdersOf(dataset, guid): HolderScan }
/// ```
pub trait FreshHolderScan {
    /// A fresh scan of every mount namespace for one dataset.
    ///
    /// # Throws
    ///
    /// No nonce could be made,
    /// which would bind the answer to nothing.
    fn holders_of(&self, dataset: &str, dataset_guid: DatasetGuid) -> Result<HolderScan, RemovalStop>;
}

/// What:
///  The release answer:
///  this machine's `/proc`,
///  this process's own namespace and root,
///  and a nonce of the scan's own.
/// Why:
///  The scan's own namespace and root say which rows are the reader's own,
///  and the nonce keeps two scans of one removal apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scan = TheLiveHolderScan.holdersOf(clone, guid);
/// ```
pub struct TheLiveHolderScan;

impl FreshHolderScan for TheLiveHolderScan {
    fn holders_of(&self, dataset: &str, dataset_guid: DatasetGuid) -> Result<HolderScan, RemovalStop> {
        let nonce: RequestNonce = RequestNonce::fresh().map_err(|refusal| return RemovalStop::NoRunNonce { reason: refusal.message() })?;
        let self_task: &Path = Path::new(SELF_TASK);
        let own_namespace: Option<String> = namespace_target(self_task.join("ns").join("mnt").as_path());
        let readers_root: Option<RootIdentity> = RootIdentity::of(self_task.join("root").as_path());
        return Ok(scan_holders(&HolderScanRequest {
            proc_root: Path::new(PROC_ROOT),
            source: dataset,
            own_namespace: own_namespace.as_deref(),
            readers_root,
            dataset: dataset_guid,
            nonce: &nonce,
            time: now_iso(),
        }));
    }
}

/// What:
///  The names one scan found,
///  so a refusal and a report print the same list.
/// Why:
///  The result's holders are namespaces with mounts,
///  and a person needs the namespace and each mount it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = theHolderLines(scan);
/// ```
pub fn the_holder_lines(scan: &HolderScan) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for mount in &scan.own_mounts {
        lines.push(format!("this process's own namespace holds it at {}", mount.mount_point.display()));
    }
    for holder in &scan.holders {
        for mount in &holder.mounts {
            lines.push(format!("namespace {} holds it at {}", holder.namespace, mount.mount_point.display()));
        }
    }
    return lines;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "fresh_holder_scan_tests.rs"]
mod tests;
