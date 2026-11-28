//! What:
//!  The two shapes the gatherer's pool reads take of the one device the
//!  compatibility gate admitted:
//!  a `DatasetReader` for step 15's mark read,
//!  and a `DatasetGuidReads` for step 13's lock proof.
//! Why:
//!  The gate admits one open of `/dev/zfs`,
//!  and a second open beside it would issue its requests before the gate
//!  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`,
//!  "A missing device ... disables raw requests,
//!  reads included"),
//!  so both seams read through the pool the gatherer already holds.
//!  The mark wants one dataset's identity and lists no children,
//!  and a caller that asks it for one is told so rather than given an empty
//!  listing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reader = new ThePoolBehindTheGate(pool);
//! const guids = new ThePoolsGuids(pool);
//! ```

/// The managed Git lock's proof,
/// and the pool read its mounted arm compares a record against.
use crate::worktree_clone_managed_lock::DatasetGuidReads;
/// One dataset's identity and property reply,
/// and the listing step step 15's mark never asks for.
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
/// The pool's own GUID for one dataset.
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
/// The pool behind the gate,
/// and the read that answers one dataset's identity.
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{
    LiveRemovalReads,
    RemovalPoolReads,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;

/// What:
///  The pool read step 15's mark wants,
///  from the gated pool this reader already opened.
/// Why:
///  The mark is one property of the snapshot's own reply,
///  and the reader that answers it must be a `DatasetReader`,
///  so the gated pool is wrapped rather than a second device opened beside it:
///  a device opened here would issue its requests before the gate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = { identity: (name) => pool.identity(name), nextChild: () => { throw new Error('the mark reads one identity and no listing'); } };
/// ```
pub(crate) struct ThePoolBehindTheGate<'a> {
    /// The pool the gate admitted.
    pub(crate) pool: &'a LiveRemovalReads,
}

impl DatasetReader for ThePoolBehindTheGate<'_> {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        return self.pool.identity(dataset).map_err(|said| {
            return ReadFailure {
                said,
                errno: None,
            };
        });
    }

    fn next_child(
        &self,
        parent: &str,
        _cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return Err(format!(
            "this reader answers one dataset's identity for the deferred-destroy mark and lists no children, so it has no child of `{parent}` to name"
        ));
    }
}

/// What:
///  The GUID the managed-lock proof compares a record's name against,
///  read through the gated pool.
/// Why:
///  The proof's mounted arm reads the GUID of the dataset a mount's source
///  names,
///  and this route reads it through the one device the gate admitted rather
///  than through a second open of `/dev/zfs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guids = { guidOf: (dataset) => pool.identity(dataset).stats.dds_guid };
/// ```
pub(crate) struct ThePoolsGuids<'a> {
    /// The pool,
    /// or the gate's own refusal,
    /// which the proof reads as a GUID it could not name.
    pub(crate) pool: &'a Result<LiveRemovalReads, String>,
}

impl DatasetGuidReads for ThePoolsGuids<'_> {
    fn guid_of(
        &self,
        dataset: &str,
    ) -> Result<DatasetGuid, String> {
        let pool: &LiveRemovalReads = match self.pool {
            Ok(read) => read,
            Err(why) => return Err(why.clone()),
        };
        let held: DatasetIdentity = pool.identity(dataset)?;
        return DatasetGuid::parse(held.stats.dds_guid.to_string().as_str())
            .map_err(|refusal| return format!("{refusal:?}"));
    }
}
