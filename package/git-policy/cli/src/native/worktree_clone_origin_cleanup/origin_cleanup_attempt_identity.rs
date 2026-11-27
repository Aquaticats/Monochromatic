//! What:
//!  The identity proof one origin cleanup attempt makes:
//!  the dataset's own GUID,
//!  its origin's group,
//!  and the recorded snapshot's GUID,
//!  name and group,
//!  read again after the destroy.
//! Why:
//!  A name alone would let a person's `zfs destroy` and a recreated snapshot
//!  pass for the recorded one,
//!  and the kernel's dependency refusal is the dependency check
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "What proves the origin's identity and lineage later").
//!  A promote moves the recorded snapshot under another dataset,
//!  so a dataset whose origin is at or above the recorded group proves nothing
//!  and blocks as `unproved`.
//!  The proof sits apart from the attempt's order,
//!  so the reads that decide "absent,
//!  live or unproved" are read in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = theLiveSnapshot(record, reads);
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;

/// What:
///  What one identity proof found about the recorded snapshot.
/// Why:
///  The three answers lead three ways:
///  absent completes the record,
///  live goes on to the counts,
///  and unproved destroys nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LiveAnswer = { kind: 'absent' } | { kind: 'live'; fullName; identity } | { kind: 'unproved'; said };
/// ```
pub(super) enum LiveAnswer {
    /// No live snapshot carries the recorded GUID under the recorded dataset.
    Absent,
    /// The recorded snapshot answers with its own name and group.
    Live {
        /// The name a destroy request carries.
        full_name: String,
        /// The snapshot's own reply.
        identity: Box<DatasetIdentity>,
    },
    /// Something the proof needs could not be read or did not match.
    Unproved(String),
}

/// What:
///  Steps 2 and 3 of the design's proof,
///  and again after the destroy.
/// Why:
///  The dataset's own GUID,
///  its origin's group and the snapshot's GUID,
///  name and group are what tie the record to the pool;
///  a name alone would let a person's `zfs destroy` and a recreated snapshot pass for the recorded
///  one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = theLiveSnapshot(record, reads);
/// ```
pub(super) fn the_live_snapshot(
    record: &OriginCleanupRecord,
    reads: &dyn RemovalPoolReads,
) -> LiveAnswer {
    let dataset: DatasetIdentity = match reads.identity(record.origin_dataset.as_str()) {
        Ok(read) => read,
        Err(said) => {
            return LiveAnswer::Unproved(format!(
                "the dataset `{}` did not answer ({said})",
                record.origin_dataset
            ));
        }
    };
    if read_guid(&dataset) != Some(record.origin_dataset_guid.file_name()) {
        return LiveAnswer::Unproved(format!(
            "the dataset `{}` answered GUID `{}` and the record holds `{}`",
            record.origin_dataset,
            read_guid(&dataset).unwrap_or_default(),
            record.origin_dataset_guid.file_name()
        ));
    }
    if dataset.stats.dds_is_snapshot != 0 {
        return LiveAnswer::Unproved(format!(
            "`{}` is a snapshot and the record names a dataset",
            record.origin_dataset
        ));
    }
    let origin: String = dataset.stats.origin();
    if !origin.is_empty() {
        let parent: DatasetIdentity = match reads.identity(origin.as_str()) {
            Ok(read) => read,
            Err(said) => {
                return LiveAnswer::Unproved(format!(
                    "the dataset's origin `{origin}` did not answer ({said})"
                ));
            }
        };
        if parent.stats.dds_creation_txg >= record.origin_snapshot_createtxg {
            return LiveAnswer::Unproved(format!(
                "the dataset `{}` is a clone of `{origin}` created in transaction group {}, and the record's snapshot was created in {}, so a promotion moved the recorded snapshot out of this dataset",
                record.origin_dataset,
                parent.stats.dds_creation_txg,
                record.origin_snapshot_createtxg
            ));
        }
    }
    let listed: Vec<LiveObject> =
        match reads.snapshots_of(record.origin_dataset.as_str(), record.origin_dataset_guid) {
            Ok(objects) => objects,
            Err(said) => {
                return LiveAnswer::Unproved(format!(
                    "the snapshots of `{}` could not be listed ({said})",
                    record.origin_dataset
                ));
            }
        };
    for object in listed {
        if object.guid != record.origin_snapshot_guid {
            continue;
        }
        if object.name != record.origin_snapshot
            || object.createtxg != record.origin_snapshot_createtxg
        {
            return LiveAnswer::Unproved(format!(
                "the snapshot with the record's GUID is named `{}` in transaction group {} and the record holds `{}` in {}",
                object.name,
                object.createtxg,
                record.origin_snapshot,
                record.origin_snapshot_createtxg
            ));
        }
        let full_name: String = object.full_name();
        return match reads.identity(full_name.as_str()) {
            Ok(identity) => LiveAnswer::Live {
                full_name,
                identity: Box::new(identity),
            },
            Err(said) => LiveAnswer::Unproved(format!(
                "the snapshot `{full_name}` did not answer ({said})"
            )),
        };
    }
    return LiveAnswer::Absent;
}

/// The GUID one reply carries,
/// spelled canonically,
/// or nothing when the reply names none.
fn read_guid(identity: &DatasetIdentity) -> Option<String> {
    if identity.stats.dds_guid == 0 {
        return None;
    }
    return Some(identity.stats.dds_guid.to_string());
}
