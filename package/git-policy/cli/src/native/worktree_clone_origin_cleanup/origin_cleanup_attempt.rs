//! What:
//!  One attempt at one origin cleanup record:
//!  prove the origin dataset and the origin snapshot by GUID,
//!  read the counts and the mark,
//!  submit one single-name destroy,
//!  and prove the absence.
//! Why:
//!  The kernel's dependency refusal is the dependency check,
//!  not a scan of children or a name test,
//!  and a destroy that returned success proves only that the name is not there now,
//!  so the record ends on an absence proof and never on an answer
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "What proves the origin's identity and lineage later" and "What ends it").
//!  The request names exactly one snapshot and carries no `defer` key,
//!  because a deferred name would mark a foreign origin for later reaping.
//!  A promote moves the recorded snapshot under another dataset,
//!  so a dataset whose origin is at or above the recorded group proves nothing and blocks as
//!  `unproved`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = attemptTheCleanup({ record, reads, device, attemptedAt });
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_record::{
    CleanupBlock,
    OriginCleanupRecord,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    OriginMark,
    mark_of_reply,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DeviceError,
    NvList,
};

/// What:
///  What one attempt did:
///  finish the cleanup,
///  or record why it could not.
/// Why:
///  Completion is the record's absence,
///  so the caller removes the file only on `Finished`,
///  and a block carries the rewritten record so the reason travels with the counts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { kind: 'finished'; destroyed?: string } | { kind: 'blocked'; record; detail };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CleanupOutcome {
    /// The recorded snapshot's GUID was absent under a dataset that passed the identity proof.
    Finished {
        /// The snapshot this attempt destroyed,
        /// or nothing when it was already gone.
        destroyed: Option<String>,
    },
    /// The attempt could not finish,
    /// and its reason is one of the four codes.
    Blocked {
        /// The record with this attempt's state lines.
        record: Box<OriginCleanupRecord>,
        /// The detail a person acts on and a printed line reports.
        detail: String,
    },
}

/// What:
///  The one attempt's inputs,
///  kept together so no caller can hand a record and a destroy to different pools.
/// Why:
///  The record names its dataset and its snapshot,
///  and both the identity proof and the request read that one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Attempt = { record; reads; device; attemptedAt };
/// ```
pub struct CleanupAttempt<'a> {
    /// The record to attempt,
    /// as its reader read it.
    pub record: &'a OriginCleanupRecord,
    /// The pool reads the identity proof makes.
    pub reads: &'a dyn RemovalPoolReads,
    /// The device the one destroy request is sent to.
    pub device: &'a dyn SnapshotDestroyDevice,
    /// When this attempt ran,
    /// as ISO 8601 with offset.
    pub attempted_at: &'a str,
}

/// What:
///  One attempt at one record.
/// Why:
///  The seven steps are one function so the order cannot be rearranged by a caller:
///  the destroy sits between two identity proofs,
///  and every early answer is a block before any request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = attemptTheCleanup(attempt);
/// ```
pub fn attempt_the_cleanup(attempt: &CleanupAttempt<'_>) -> CleanupOutcome {
    let record: &OriginCleanupRecord = attempt.record;
    let pool: String = match record.pool() {
        Ok(named) => named,
        Err(refusal) => {
            return blocked(
                record,
                attempt.attempted_at,
                BlockReason::Unproved,
                refusal.said,
            );
        }
    };
    match the_live_snapshot(attempt) {
        LiveAnswer::Unproved(said) => {
            return blocked(record, attempt.attempted_at, BlockReason::Unproved, said);
        }
        LiveAnswer::Absent => {
            return CleanupOutcome::Finished {
                destroyed: None,
            };
        }
        LiveAnswer::Live {
            full_name,
            identity,
        } => {
            if let Some((next, detail)) =
                the_counts(record, attempt.attempted_at, identity.as_ref())
            {
                return CleanupOutcome::Blocked {
                    record: Box::new(next),
                    detail,
                };
            }
            let answer: Result<Option<NvList>, DeviceError> = attempt
                .device
                .destroy_snaps(pool.as_str(), &[full_name.as_str()]);
            match answer {
                Ok(_) => {}
                Err(error) => {
                    let errno: u32 = error
                        .errno()
                        .and_then(|held| return u32::try_from(held).ok())
                        .unwrap_or(0);
                    return blocked_with_errno(
                        record,
                        attempt.attempted_at,
                        BlockReason::Refused,
                        format!("the pool refused the destroy with errno {errno} ({error})"),
                        errno,
                    );
                }
            }
            return match the_live_snapshot(attempt) {
                LiveAnswer::Absent => CleanupOutcome::Finished {
                    destroyed: Some(full_name),
                },
                LiveAnswer::Live {
                    ..
                } => blocked(
                    record,
                    attempt.attempted_at,
                    BlockReason::Refused,
                    String::from("the destroy answered success and the snapshot is still listed"),
                ),
                LiveAnswer::Unproved(said) => {
                    blocked(record, attempt.attempted_at, BlockReason::Unproved, said)
                }
            };
        }
    }
}

/// One block whose errno is zero,
/// because no request was made.
fn blocked(
    record: &OriginCleanupRecord,
    attempted_at: &str,
    reason: BlockReason,
    detail: String,
) -> CleanupOutcome {
    return blocked_with_errno(record, attempted_at, reason, detail, 0);
}

/// One block with the errno a refused request carried,
/// which is what an administrator reads for a permission refusal.
fn blocked_with_errno(
    record: &OriginCleanupRecord,
    attempted_at: &str,
    reason: BlockReason,
    detail: String,
    errno: u32,
) -> CleanupOutcome {
    let block: CleanupBlock = CleanupBlock {
        reason,
        errno,
        clones: 0,
        holds: 0,
    };
    return CleanupOutcome::Blocked {
        record: Box::new(record.blocked(block, attempted_at)),
        detail,
    };
}

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
enum LiveAnswer {
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
/// const answer = theLiveSnapshot(attempt);
/// ```
fn the_live_snapshot(attempt: &CleanupAttempt<'_>) -> LiveAnswer {
    let record: &OriginCleanupRecord = attempt.record;
    let dataset: DatasetIdentity = match attempt.reads.identity(record.origin_dataset.as_str()) {
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
        let parent: DatasetIdentity = match attempt.reads.identity(origin.as_str()) {
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
    let listed: Vec<LiveObject> = match attempt
        .reads
        .snapshots_of(record.origin_dataset.as_str(), record.origin_dataset_guid)
    {
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
        return match attempt.reads.identity(full_name.as_str()) {
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

/// What:
///  Whether the live snapshot is pinned or marked,
///  which is a block before any request.
/// Why:
///  The counts report the block before a request that would be refused anyway,
///  and a mark is another writer's,
///  which the wrapper never clears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const block = theCounts(record, attemptedAt, identity); // undefined when free
/// ```
fn the_counts(
    record: &OriginCleanupRecord,
    attempted_at: &str,
    identity: &DatasetIdentity,
) -> Option<(OriginCleanupRecord, String)> {
    let clones: u64 = identity.stats.dds_num_clones;
    let mark: OriginMark = mark_of_reply(identity);
    let holds: u64 = match &mark {
        OriginMark::Marked {
            holds,
            ..
        } => match holds {
            crate::worktree_removal_deferred_origin_check::origin_mark::UserHolds::Counted(
                count,
            ) => *count,
            crate::worktree_removal_deferred_origin_check::origin_mark::UserHolds::Unreadable {
                said,
            } => {
                let block: CleanupBlock = CleanupBlock {
                    reason: BlockReason::Unproved,
                    errno: 0,
                    clones,
                    holds: 0,
                };
                return Some((
                    record.blocked(block, attempted_at),
                    format!("the snapshot's hold count could not be read ({said})"),
                ));
            }
        },
        _ => 0,
    };
    if clones != 0 || holds != 0 {
        let block: CleanupBlock = CleanupBlock {
            reason: BlockReason::Dependency,
            errno: 0,
            clones,
            holds,
        };
        return Some((
            record.blocked(block, attempted_at),
            format!("{clones} clone(s) and {holds} hold(s) pin the snapshot"),
        ));
    }
    if let OriginMark::Marked {
        ..
    } = mark
    {
        let block: CleanupBlock = CleanupBlock {
            reason: BlockReason::Deferred,
            errno: 0,
            clones,
            holds,
        };
        return Some((
            record.blocked(block, attempted_at),
            String::from("the snapshot is marked for deferred destroy"),
        ));
    }
    if let OriginMark::Unreadable {
        said,
    } = mark
    {
        let block: CleanupBlock = CleanupBlock {
            reason: BlockReason::Unproved,
            errno: 0,
            clones,
            holds,
        };
        return Some((
            record.blocked(block, attempted_at),
            format!("the mark could not be read ({said})"),
        ));
    }
    return None;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_attempt_tests.rs"]
mod tests;
