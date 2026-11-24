//! What:
//!  The one function a removal calls before the first snapshot request and again before the head
//!  destroy,
//!  and the judgement the look inside the lineage proof makes.
//! Why:
//!  Destroying the last clone of a snapshot somebody marked for deferred destroy destroys that
//!  snapshot in the same transaction,
//!  so a removal checks that the clone and its origin are the objects it planned and that the
//!  origin is not marked,
//!  and refuses otherwise
//!  (`doc/decision/cli-git-zfs-worktree-design.md`, "What was decided",
//!  and the proposal, "Clone removal identity protection").
//!  The check reads by the planned names and compares field by field in one order,
//!  stopping at the first disagreement,
//!  so the mark is only ever judged on the snapshot whose own reply proved its identity:
//!  a mark read from a replacement proves nothing about the planned snapshot
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "What the check reads, in order").
//!  The check sends no request of its own kind and writes nothing;
//!  three object-stats reads are its whole cost
//!  (the same design, "Reading the mark" and veto V5).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = checkPlannedOrigin(reader, planned); // { kind: 'proceed' } | { kind: 'refused'; cause }
//! ```

use super::origin_mark::{OriginMark, mark_of_reply};
use super::origin_refusal::OriginRefusal;
use super::origin_refusals::{JudgedObject, RefusalPoint};
use super::planned_origin::PlannedOrigin;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::DMU_OBJECT_SET_FILESYSTEM;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ReadFailure};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;

//region The verdict

/// What:
///  What a refusal point answers:
///  the removal goes on,
///  or it stops with one typed cause.
/// Why:
///  One word per state is what a call site reads,
///  and the cause carries the fields so the message and a receipt need no parsing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = { kind: 'proceed' } | { kind: 'refused'; cause: OriginRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// Every identity the plan recorded answered,
    /// and the origin is not marked.
    Proceed,
    /// The removal stops and prints the cause.
    Refused(OriginRefusal),
}

//endregion

//region The check

/// What:
///  Whether a removal may send the request it is about to send:
///  the clone and its origin are the planned ones and the origin is not marked for deferred
///  destroy.
/// Why:
///  A removal calls this before every snapshot request, each chunk of a split included, and
///  again before the head destroy,
///  so no request of this route is sent over an origin somebody marked or over an object the
///  removal did not plan
///  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
///  "One check, called at every snapshot request and before the head destroy").
///  A first call site is skipped only when the run sends no snapshot request at all,
///  and the head check then keeps the guarantee for the one delete that is left
///  (the same design, veto V1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkPlannedOrigin(reader, planned);
/// ```
#[must_use]
pub fn check_planned_origin(reader: &dyn DatasetReader, planned: &PlannedOrigin) -> Verdict {
    let clone: DatasetIdentity = match read(reader, &planned.clone_name) {
        Ok(found) => found,
        Err(answer) => return Verdict::Refused(refused_read(planned, JudgedObject::Clone, &planned.clone_name, answer)),
    };
    if clone.stats.dds_type != DMU_OBJECT_SET_FILESYSTEM || clone.stats.dds_is_snapshot != 0 {
        return Verdict::Refused(OriginRefusal::CloneIsNotAFilesystem { planned: planned.clone(), found: clone.stats.dds_type, snapshot: clone.stats.dds_is_snapshot != 0 });
    }
    if clone.stats.dds_guid != planned.clone_guid.value() {
        return Verdict::Refused(OriginRefusal::CloneIsNotThePlannedOne { planned: planned.clone(), live: clone.stats.dds_guid });
    }
    let live_origin: String = clone.stats.origin();
    if live_origin.is_empty() {
        return Verdict::Refused(OriginRefusal::CloneHasNoOrigin { planned: planned.clone() });
    }
    if live_origin != planned.origin_snapshot_name {
        return Verdict::Refused(OriginRefusal::CloneOriginDiffers { planned: planned.clone(), live: live_origin });
    }
    let origin_dataset: DatasetIdentity = match read(reader, &planned.origin_dataset_name) {
        Ok(found) => found,
        Err(answer) => return Verdict::Refused(refused_read(planned, JudgedObject::OriginDataset, &planned.origin_dataset_name, answer)),
    };
    if origin_dataset.stats.dds_type != DMU_OBJECT_SET_FILESYSTEM || origin_dataset.stats.dds_is_snapshot != 0 {
        return Verdict::Refused(OriginRefusal::OriginDatasetIsNotAFilesystem {
            planned: planned.clone(),
            found: origin_dataset.stats.dds_type,
            snapshot: origin_dataset.stats.dds_is_snapshot != 0,
        });
    }
    if origin_dataset.stats.dds_guid != planned.origin_dataset_guid.value() {
        return Verdict::Refused(OriginRefusal::OriginDatasetIsNotThePlannedOne { planned: planned.clone(), live: origin_dataset.stats.dds_guid });
    }
    let origin_snapshot: DatasetIdentity = match read(reader, &planned.origin_snapshot_name) {
        Ok(found) => found,
        Err(answer) => return Verdict::Refused(refused_read(planned, JudgedObject::OriginSnapshot, &planned.origin_snapshot_name, answer)),
    };
    if origin_snapshot.stats.dds_is_snapshot == 0 {
        return Verdict::Refused(OriginRefusal::OriginIsNotASnapshot { planned: planned.clone() });
    }
    if origin_snapshot.stats.dds_guid != planned.origin_snapshot_guid.value() {
        return Verdict::Refused(OriginRefusal::OriginSnapshotIsNotThePlannedOne { planned: planned.clone(), live: origin_snapshot.stats.dds_guid });
    }
    if origin_snapshot.stats.dds_creation_txg != planned.origin_snapshot_createtxg {
        return Verdict::Refused(OriginRefusal::OriginSnapshotCreatetxgDiffers { planned: planned.clone(), live: origin_snapshot.stats.dds_creation_txg });
    }
    return match mark_of_reply(&origin_snapshot) {
        OriginMark::Unmarked => Verdict::Proceed,
        OriginMark::Marked { clones, holds } => Verdict::Refused(OriginRefusal::OriginMarkedForDeferredDestroy { planned: planned.clone(), clones, holds }),
        OriginMark::Unreadable { said } => Verdict::Refused(OriginRefusal::MarkUnreadable {
            clone_name: planned.clone_name.clone(),
            slot: planned.slot.clone(),
            origin_snapshot: planned.origin_snapshot_name.clone(),
            point: RefusalPoint::AtACheck,
            said,
        }),
    };
}

//endregion

//region The look before the unmount

/// What:
///  Whether a first removal may go on past its lineage proof:
///  the mark on the origin snapshot the proof resolved,
///  judged where the proof read it.
/// Why:
///  A mark present when a person starts a removal is refused before anything is taken down,
///  with the worktree still mounted and nothing written
///  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
///  "A look before the unmount").
///  The proof owns the identity comparisons,
///  so this judgement has no identity causes of its own:
///  it judges a mark only on the snapshot whose own reply proved it is the recorded origin
///  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
///  "The look before the unmount").
///  It runs once per removal, on a first run only,
///  and a retry over a kept plan is refused by the check instead,
///  with the plan's loss in its message (the same design, veto V11).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// judgeTheLook('fix-flaky-test', 'pool/clones/fix-flaky-test', mark); // { kind: 'proceed' } | { kind: 'refused'; cause }
/// ```
#[must_use]
pub fn judge_the_look(clone_name: &str, slot: &str, origin_snapshot: &str, mark: &OriginMark) -> Verdict {
    return match mark {
        OriginMark::Unmarked => Verdict::Proceed,
        OriginMark::Marked { clones, holds } => Verdict::Refused(OriginRefusal::OriginMarkedBeforeTheUnmount {
            clone_name: String::from(clone_name),
            slot: String::from(slot),
            origin_snapshot: String::from(origin_snapshot),
            clones: *clones,
            holds: holds.clone(),
        }),
        OriginMark::Unreadable { said } => Verdict::Refused(OriginRefusal::MarkUnreadable {
            clone_name: String::from(clone_name),
            slot: String::from(slot),
            origin_snapshot: String::from(origin_snapshot),
            point: RefusalPoint::BeforeTheUnmount,
            said: said.clone(),
        }),
    };
}

//endregion

//region Reading one of the three objects

/// What:
///  What a read that answered nothing useful said:
///  the object is not there,
///  or the read failed for another reason.
/// Why:
///  "Not there" is the object's own gone cause and every other failure says what the kernel
///  answered,
///  and the answer stays small so the three reads keep one shape without a large error type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadAnswer = { kind: 'gone' } | { kind: 'failed'; said; errno };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
enum ReadAnswer {
    /// The kernel answered that no such dataset exists.
    Gone,
    /// The read failed for another reason.
    Failed(ReadFailure),
}

/// One of the three reads,
/// or what a read that gave nothing useful answered.
fn read(reader: &dyn DatasetReader, name: &str) -> Result<DatasetIdentity, ReadAnswer> {
    let answer: Result<DatasetIdentity, ReadFailure> = reader.identity(name);
    let failure: ReadFailure = match answer {
        Ok(identity) => return Ok(identity),
        Err(broken) => broken,
    };
    if failure.errno == Some(libc::ENOENT) {
        return Err(ReadAnswer::Gone);
    }
    return Err(ReadAnswer::Failed(failure));
}

/// The refusal one failed read gives,
/// naming the object and the plan's own tuple.
fn refused_read(planned: &PlannedOrigin, object: JudgedObject, name: &str, answer: ReadAnswer) -> OriginRefusal {
    return match answer {
        ReadAnswer::Gone => match object {
            JudgedObject::Clone => OriginRefusal::CloneGone { planned: planned.clone() },
            JudgedObject::OriginDataset => OriginRefusal::OriginDatasetGone { planned: planned.clone() },
            JudgedObject::OriginSnapshot => OriginRefusal::OriginSnapshotGone { planned: planned.clone() },
        },
        ReadAnswer::Failed(failure) => OriginRefusal::ReadFailed { planned: planned.clone(), object, name: String::from(name), said: failure.message() },
    };
}

//endregion

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_verdict_tests.rs"]
mod tests;

/// The live arm that runs only inside the throwaway guest.
#[cfg(test)]
#[path = "live_origin_check_tests.rs"]
mod live_tests;
