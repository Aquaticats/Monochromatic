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
//!  The identity proof and the counts each sit in the file named for them,
//!  and this file holds the order the seven steps run in,
//!  so a reader can hold one decision at a time.
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
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DeviceError,
    NvList,
};

/// Whether the live snapshot is pinned or marked,
/// which blocks before any request.
#[path = "origin_cleanup_attempt_counts.rs"]
mod origin_cleanup_attempt_counts;
/// What the recorded snapshot's identity proof found,
/// and the reads it makes.
#[path = "origin_cleanup_attempt_identity.rs"]
mod origin_cleanup_attempt_identity;

/// The block the snapshot's own counts and mark decide.
use self::origin_cleanup_attempt_counts::the_counts;
/// The three answers one identity proof leads to.
use self::origin_cleanup_attempt_identity::{
    LiveAnswer,
    the_live_snapshot,
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
    match the_live_snapshot(record, attempt.reads) {
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
            return match the_live_snapshot(record, attempt.reads) {
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_attempt_tests.rs"]
mod tests;
