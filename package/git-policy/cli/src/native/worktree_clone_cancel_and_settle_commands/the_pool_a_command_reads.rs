//! What:
//!  The pool the two commands read,
//!  opened when a read first needs it and never before.
//! Why:
//!  Both commands refuse a caller's mistake before they read the pool:
//!  the cancellation proves the slot's records first,
//!  and the settling command reads the claim,
//!  the binding and the two confirming values first,
//!  so a mistyped slot must not need `/dev/zfs` at all.
//!  The compatibility gate runs before the device is opened for the whole
//!  removal
//!  (`worktree_clone_removal_proof_and_plan::removal_reads`,
//!  which holds that order),
//!  so the absence reader here opens after the pool's own open answered,
//!  and one asked-for read runs the gate and then opens.
//!  A command that answered a typo with "the device is absent" would send a
//!  person to the kernel for a mistake they made,
//!  and a control on this machine must never open this machine's pool
//!  (`AGENTS.md`,
//!  "no test or probe on the host that opens `/dev/zfs`"),
//!  which is why no control of this file reads the pool on this machine:
//!  the guest arm of the grammar is where a read is opened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pool = new ThePoolACommandReadsWhenAsked();
//! pool.snapshotsOf(clone, guid); // the gate runs, then the device opens, once
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
    ZfsDatasetReader,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{
    LiveRemovalReads,
    RemovalPoolReads,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::cell::OnceCell;

/// What:
///  The pool one command reads:
///  the removal's reads and the absence proof's reader over one device,
///  opened at the first read that needs it.
/// Why:
///  The cancellation takes both seams and the settling command takes the
///  reader,
///  so one value with both answers is what the grammar can build before it knows
///  whether the command will read the pool at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ThePool = { pool: OnceCell<Result<LiveRemovalReads, string>>; children: OnceCell<Result<ZfsDatasetReader, ReadFailure>> };
/// ```
pub(crate) struct ThePoolACommandReadsWhenAsked {
    /// The removal's reads,
    /// opened by the first pool request,
    /// which runs the compatibility gate first.
    pool: OnceCell<Result<LiveRemovalReads, String>>,
    /// The absence proof's reader,
    /// opened after the pool's own open answered,
    /// so the gate speaks before the device does.
    children: OnceCell<Result<ZfsDatasetReader, ReadFailure>>,
}

impl ThePoolACommandReadsWhenAsked {
    /// The pool nothing has opened yet.
    pub(crate) fn not_opened() -> ThePoolACommandReadsWhenAsked {
        return ThePoolACommandReadsWhenAsked {
            pool: OnceCell::new(),
            children: OnceCell::new(),
        };
    }

    /// The removal's reads,
    /// opened once,
    /// with the gate's own refusal kept as it answered.
    fn the_removal_s_reads(&self) -> Result<&LiveRemovalReads, String> {
        return self
            .pool
            .get_or_init(LiveRemovalReads::open)
            .as_ref()
            .map_err(|said| return said.clone());
    }

    /// The absence proof's reader,
    /// whose open waits for the pool's own open so the gate speaks first.
    fn the_absence_reader(&self) -> Result<&ZfsDatasetReader, ReadFailure> {
        return self
            .children
            .get_or_init(|| {
                self.the_removal_s_reads().map_err(ReadFailure::at)?;
                return ZfsDatasetReader::open()
                    .map_err(|error| return ReadFailure::at(error.to_string()));
            })
            .as_ref()
            .map_err(|failure| return failure.clone());
    }
}

impl RemovalPoolReads for ThePoolACommandReadsWhenAsked {
    fn identity(
        &self,
        name: &str,
    ) -> Result<DatasetIdentity, String> {
        return self.the_removal_s_reads()?.identity(name);
    }

    fn snapshots_of(
        &self,
        dataset: &str,
        dataset_guid: DatasetGuid,
    ) -> Result<Vec<LiveObject>, String> {
        return self
            .the_removal_s_reads()?
            .snapshots_of(dataset, dataset_guid);
    }

    fn bookmarks_of(
        &self,
        dataset: &str,
        dataset_guid: DatasetGuid,
    ) -> Result<Vec<LiveObject>, String> {
        return self
            .the_removal_s_reads()?
            .bookmarks_of(dataset, dataset_guid);
    }
}

impl DatasetReader for ThePoolACommandReadsWhenAsked {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        return self.the_absence_reader()?.identity(dataset);
    }

    fn next_child(
        &self,
        parent: &str,
        cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return self
            .the_absence_reader()
            .map_err(|failure| return failure.said)?
            .next_child(parent, cursor);
    }
}
