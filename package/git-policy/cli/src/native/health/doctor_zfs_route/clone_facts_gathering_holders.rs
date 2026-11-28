//! What:
//!  The namespaces that hold one clone,
//!  as one clone's fact set carries them:
//!  every namespace other than the reader's own that holds it,
//!  the mounts of the reader's own namespace,
//!  and every read the scan could not complete.
//! Why:
//!  A removal is blocked by every namespace that holds the clone and by the
//!  caller's own mount,
//!  and the caller's own is the one a reader must see,
//!  so the scan's two halves travel into one fact
//!  (`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`;
//!  plan step 18,
//!  "the holders of a clone by namespace ID and representative PID").
//!  A task number nobody can read still names the namespace that holds the
//!  clone,
//!  because a holder whose task is unreadable still stops a removal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holders = holdersOf(reads, clone, cloneGuid, callerTask); // HolderFacts
//! ```

/// Every read one clone's facts come from.
use super::clone_facts_reads::CloneFactsReads;
/// The namespaces that hold a clone.
use super::holder_checks::{
    HolderFacts,
    NamespaceHold,
};
/// The pool's own GUID for one dataset.
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
/// The holders one all-namespace scan found,
/// as the removal folder reports them.
use crate::worktree_clone_removal_proof_and_plan::holder_report::TheHoldersOfOneClone;

/// The words a namespace is named by when the reader's own could not be read.
pub(crate) const THE_READERS_OWN_NAMESPACE: &str =
    "the reader's own namespace, which could not be read";

/// What:
///  The namespaces that hold one clone,
///  from one all-namespace scan,
///  the reader's own namespace's mounts among them.
/// Why:
///  A removal is blocked by every namespace that holds the clone and by the
///  caller's own mount,
///  and the caller's own is the one a reader must see,
///  so the scan's two halves travel into one fact.
pub(crate) fn holders_of(
    reads: &dyn CloneFactsReads,
    clone: &str,
    clone_guid: Option<DatasetGuid>,
    caller_task: u32,
) -> HolderFacts {
    let Some(guid) = clone_guid else {
        return HolderFacts {
            clone: String::from(clone),
            holders: Vec::new(),
            incomplete: Vec::from([format!(
                "the record filed under this slot names no clone GUID, so no all-namespace scan could be made for `{clone}`"
            )]),
        };
    };
    let scan: TheHoldersOfOneClone = match reads.holder_scan(clone, guid) {
        Ok(found) => found,
        Err(reason) => {
            return HolderFacts {
                clone: String::from(clone),
                holders: Vec::new(),
                incomplete: Vec::from([reason]),
            };
        }
    };
    let mut holders: Vec<NamespaceHold> = Vec::new();
    for held in &scan.holders {
        holders.push(NamespaceHold {
            namespace: held.namespace.clone(),
            representative_pid: held.task,
            source: String::from(clone),
            mount_point: held.mount_point.clone(),
        });
    }
    if !scan.own_mounts.is_empty() {
        let own: &str = scan
            .own_namespace
            .as_deref()
            .unwrap_or(THE_READERS_OWN_NAMESPACE);
        for mount_point in &scan.own_mounts {
            holders.push(NamespaceHold {
                namespace: String::from(own),
                representative_pid: caller_task,
                source: String::from(clone),
                mount_point: mount_point.clone(),
            });
        }
    }
    return HolderFacts {
        clone: String::from(clone),
        holders,
        incomplete: scan.incomplete.clone(),
    };
}
