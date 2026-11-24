//! What:
//!  The one pool script every control of this folder drives, and the healthy tuple they break
//!  one field at a time.
//! Why:
//!  No control on a person's machine may open `/dev/zfs`,
//!  so the check reads a scripted pool through the same `DatasetReader` seam the sweep's controls
//!  use (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  assumption A3 and "The tests a build must have").
//!  One healthy tuple is what proves each refusal came from the field its control changed rather
//!  than from another comparison,
//!  and the names are the design's own example names so a message reads as the design's does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pool = new Pool([healthy().clone, healthy().originDataset, healthy().originSnapshot]);
//! const verdict = checkPlannedOrigin(pool, planned());
//! ```

use super::origin_mark::{DEFER_DESTROY_PROPERTY, MARK_OFF, MARK_ON, USER_HOLDS_PROPERTY};
use super::origin_verdict::Verdict;
use super::planned_origin::PlannedOrigin;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::DMU_OBJECT_SET_FILESYSTEM;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, ReadFailure};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, DmuObjsetStats, NvList};
use std::cell::RefCell;
use std::path::PathBuf;

/// The slot the removal was asked for.
pub(crate) const SLOT: &str = "fix-flaky-test";

/// The clone's full dataset name.
pub(crate) const CLONE: &str = "zpcachyos/nosnap/clones/fix-flaky-test";

/// The checkout dataset the add route snapshots.
pub(crate) const CHECKOUT: &str = "zpcachyos/nosnap/checkout";

/// The origin snapshot's full name,
/// in the form the add route gives it.
pub(crate) const ORIGIN_SNAPSHOT: &str = "zpcachyos/nosnap/checkout@git-worktree-clone-origin-0011223344556677";

/// The GUID this removal recorded for the clone.
pub(crate) const CLONE_GUID: u64 = 12_345_678_901_234_567_890;

/// The GUID it recorded for the origin dataset.
pub(crate) const ORIGIN_DATASET_GUID: u64 = 12_345_678_901_234_567_891;

/// The GUID it recorded for the origin snapshot.
pub(crate) const ORIGIN_SNAPSHOT_GUID: u64 = 12_345_678_901_234_567_892;

/// The transaction group it recorded for the origin snapshot.
pub(crate) const ORIGIN_CREATETXG: u64 = 4_242;

/// The plan file the controls name.
pub(crate) const PLAN_HOME: &str = "zpcachyos/nosnap/clones/.git/plans/12345678901234567890";

/// The largest origin name the kernel's field holds.
const NAME_FIELD_BYTES: usize = 256;

/// What:
///  One dataset or snapshot as a scripted pool answers for it.
/// Why:
///  The reply and the failure are the two shapes a read gives,
///  and a control sets exactly one of them per name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { kind: 'identity'; stats; properties } | { kind: 'failed'; said; errno };
/// ```
#[derive(Clone)]
pub(crate) enum Answer {
    /// The read answered with these statistics and properties.
    Identity(Box<DmuObjsetStats>, NvList),
    /// The read answered with this failure.
    Failed(String, Option<i32>),
}

impl Answer {
    /// One identity answer,
    /// boxed so the script's enum stays small.
    pub(crate) fn of(stats: DmuObjsetStats, properties: NvList) -> Answer {
        return Answer::Identity(Box::new(stats), properties);
    }

    /// The identity this answer holds,
    /// or the failure in its place.
    fn identity(&self) -> Result<DatasetIdentity, ReadFailure> {
        return match self {
            Answer::Identity(stats, properties) => Ok(DatasetIdentity { stats: **stats, properties: properties.clone() }),
            Answer::Failed(said, errno) => Err(ReadFailure { said: said.clone(), errno: *errno }),
        };
    }
}

/// What:
///  A pool that answers the names a script gives and records the reads it made.
/// Why:
///  The order of the three reads and the reads that must not happen are half of what the check
///  promises,
///  so the script keeps one list of names in the order they were asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = new Pool([[CLONE, cloneAnswer]]); pool.reads();
/// ```
pub(crate) struct Pool {
    /// The answers by name,
    /// in the order the controls wrote them.
    answers: Vec<(String, Answer)>,
    /// Every name the check read,
    /// in the order it read them.
    reads: RefCell<Vec<String>>,
}

impl Pool {
    /// A pool answering the names given.
    pub(crate) fn new(answers: Vec<(&str, Answer)>) -> Pool {
        return Pool {
            answers: answers.into_iter().map(|(name, answer)| return (String::from(name), answer)).collect(),
            reads: RefCell::new(Vec::new()),
        };
    }

    /// The names this pool was asked for,
    /// in order.
    pub(crate) fn reads(&self) -> Vec<String> {
        return self.reads.borrow().clone();
    }

    /// The same pool with one name's answer replaced,
    /// so a control changes one object of the healthy script.
    pub(crate) fn with(mut self, name: &str, answer: Answer) -> Pool {
        let found: Option<&mut (String, Answer)> = self.answers.iter_mut().find(|(answered, _)| return answered == name);
        match found {
            Some(entry) => entry.1 = answer,
            None => self.answers.push((String::from(name), answer)),
        }
        return self;
    }
}

impl DatasetReader for Pool {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        self.reads.borrow_mut().push(String::from(dataset));
        let found: Option<&(String, Answer)> = self.answers.iter().find(|(name, _)| return name == dataset);
        let Some((_name, answer)) = found else {
            return Err(ReadFailure::at(format!("this scripted pool holds no `{dataset}`")));
        };
        return answer.identity();
    }

    fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

//region The healthy tuple

/// The identities this removal recorded,
/// which every control starts from.
pub(crate) fn planned() -> PlannedOrigin {
    return PlannedOrigin {
        slot: String::from(SLOT),
        clone_name: String::from(CLONE),
        clone_guid: guid(CLONE_GUID),
        origin_dataset_name: String::from(CHECKOUT),
        origin_dataset_guid: guid(ORIGIN_DATASET_GUID),
        origin_snapshot_name: String::from(ORIGIN_SNAPSHOT),
        origin_snapshot_guid: guid(ORIGIN_SNAPSHOT_GUID),
        origin_snapshot_createtxg: ORIGIN_CREATETXG,
    };
}

/// One GUID as a control writes it.
pub(crate) fn guid(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str()).expect("a control writes a canonical GUID");
}

/// The kernel's statistics for one object,
/// with every field the check reads set the way a healthy pool answers.
pub(crate) fn stats(guid_value: u64, is_snapshot: bool, creation_txg: u64, clones: u64, origin: &str) -> DmuObjsetStats {
    let mut origin_bytes: [u8; NAME_FIELD_BYTES] = [0; NAME_FIELD_BYTES];
    origin_bytes[..origin.len()].copy_from_slice(origin.as_bytes());
    return DmuObjsetStats {
        dds_num_clones: clones,
        dds_creation_txg: creation_txg,
        dds_guid: guid_value,
        dds_type: DMU_OBJECT_SET_FILESYSTEM,
        dds_is_snapshot: u8::from(is_snapshot),
        dds_inconsistent: 0,
        dds_redacted: 0,
        dds_origin: origin_bytes,
        dds_flags: 0,
    };
}

/// The clone's healthy statistics,
/// whose reported origin is the planned snapshot.
pub(crate) fn clone_stats() -> DmuObjsetStats {
    return stats(CLONE_GUID, false, 7, 0, ORIGIN_SNAPSHOT);
}

/// The origin dataset's healthy statistics.
pub(crate) fn origin_dataset_stats() -> DmuObjsetStats {
    return stats(ORIGIN_DATASET_GUID, false, 3, 1, "");
}

/// The origin snapshot's healthy statistics,
/// which report the clones and the holds its reply's properties count.
pub(crate) fn origin_snapshot_stats(clones: u64, mark: Option<u64>, holds: Option<u64>) -> (DmuObjsetStats, NvList) {
    return (stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG, clones, ""), origin_properties(mark, holds));
}

/// The origin snapshot's property list,
/// which carries the mark and the hold count when a control asks for them.
pub(crate) fn origin_properties(mark: Option<u64>, holds: Option<u64>) -> NvList {
    let mut reply: NvList = NvList::new();
    if let Some(value) = mark {
        reply.add_nvlist(DEFER_DESTROY_PROPERTY, pair_number(value, None));
    }
    if let Some(count) = holds {
        reply.add_nvlist(USER_HOLDS_PROPERTY, pair_number(count, None));
    }
    return reply;
}

/// The pool every refusal control starts from:
/// a healthy clone,
/// its healthy origin dataset,
/// and an origin snapshot whose mark and hold count come from the control.
pub(crate) fn healthy(mark: Option<u64>, holds: Option<u64>) -> Pool {
    return pool_over(1, mark, holds);
}

/// The same pool with the number of clones of the origin snapshot a control names.
pub(crate) fn pool_over(clones: u64, mark: Option<u64>, holds: Option<u64>) -> Pool {
    let (snapshot_stats, snapshot_properties) = origin_snapshot_stats(clones, mark, holds);
    return Pool::new(vec![
        (CLONE, Answer::of(clone_stats(), NvList::new())),
        (CHECKOUT, Answer::of(origin_dataset_stats(), NvList::new())),
        (ORIGIN_SNAPSHOT, Answer::of(snapshot_stats, snapshot_properties)),
    ]);
}

/// The pool whose two origin-side answers are healthy and whose mark reads off.
pub(crate) fn unmarked() -> Pool {
    return healthy(Some(MARK_OFF), Some(0));
}

/// The pool whose origin snapshot is marked,
/// with one clone and no user hold,
/// which is the state a removal would reap.
pub(crate) fn marked() -> Pool {
    return healthy(Some(MARK_ON), Some(0));
}

//endregion

//region Replies as OpenZFS writes them

/// One numeric property,
/// with the source only when a control asks for one.
pub(crate) fn pair_number(value: u64, source: Option<&str>) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_u64("value", value);
    if let Some(from) = source {
        pair.add_str("source", from);
    }
    return pair;
}

/// One text property.
pub(crate) fn pair_text(value: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    return pair;
}

//endregion

//region Answers a control plants

/// The answer for a name the kernel says is not there.
pub(crate) fn gone(name: &str) -> Answer {
    return Answer::Failed(format!("`{name}` does not exist"), Some(libc::ENOENT));
}

/// The answer for a read that failed for another reason.
pub(crate) fn refused(said: &str) -> Answer {
    return Answer::Failed(String::from(said), Some(libc::EACCES));
}

/// What a verdict refused with,
/// for a control to compare against.
pub(crate) fn refusal(verdict: Verdict) -> super::origin_refusal::OriginRefusal {
    return match verdict {
        Verdict::Proceed => panic!("the check proceeded over the state this control planted"),
        Verdict::Refused(cause) => cause,
    };
}

/// A plan's fields with one of them missing.
pub(crate) fn fields() -> super::planned_origin::PlannedOriginFields {
    let recorded: PlannedOrigin = planned();
    return super::planned_origin::PlannedOriginFields {
        plan_home: PathBuf::from(PLAN_HOME),
        clone_name: Some(recorded.clone_name),
        clone_guid: Some(recorded.clone_guid),
        origin_dataset_name: Some(recorded.origin_dataset_name),
        origin_dataset_guid: Some(recorded.origin_dataset_guid),
        origin_snapshot_name: Some(recorded.origin_snapshot_name),
        origin_snapshot_guid: Some(recorded.origin_snapshot_guid),
        origin_snapshot_createtxg: Some(recorded.origin_snapshot_createtxg),
    };
}

//endregion

//region One cause of every kind

/// What:
///  One refusal of every cause this folder can report,
///  each with the label a control names it by.
/// Why:
///  The step word,
///  the form and the message of every cause are compared from one list,
///  so a cause added without its words fails here rather than at a call site.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [label, cause] of everyCause()) expect(cause.message(history()).length).toBeGreaterThan(0);
/// ```
pub(crate) fn every_cause() -> Vec<(&'static str, super::origin_refusal::OriginRefusal)> {
    use super::origin_mark::UserHolds;
    use super::origin_refusal::OriginRefusal;
    use super::origin_refusals::{JudgedObject, PlanField, RefusalPoint};
    let recorded: PlannedOrigin = planned();
    let marked: UserHolds = UserHolds::Counted(0);
    return vec![
        (
            "plan-incomplete",
            OriginRefusal::PlanIncomplete { slot: String::from(SLOT), field: PlanField::CloneGuid, plan_home: PathBuf::from(PLAN_HOME) },
        ),
        (
            "marked-before-the-unmount",
            OriginRefusal::OriginMarkedBeforeTheUnmount {
                clone_name: String::from(CLONE),
                slot: String::from(SLOT),
                origin_snapshot: String::from(ORIGIN_SNAPSHOT),
                clones: 1,
                holds: marked.clone(),
            },
        ),
        ("marked-at-a-check", OriginRefusal::OriginMarkedForDeferredDestroy { planned: recorded.clone(), clones: 1, holds: marked }),
        (
            "unreadable-mark-before-the-unmount",
            OriginRefusal::MarkUnreadable {
                clone_name: String::from(CLONE),
                slot: String::from(SLOT),
                origin_snapshot: String::from(ORIGIN_SNAPSHOT),
                point: RefusalPoint::BeforeTheUnmount,
                said: String::from("`defer_destroy` is not in the reply"),
            },
        ),
        (
            "unreadable-mark-at-a-check",
            OriginRefusal::MarkUnreadable {
                clone_name: String::from(CLONE),
                slot: String::from(SLOT),
                origin_snapshot: String::from(ORIGIN_SNAPSHOT),
                point: RefusalPoint::AtACheck,
                said: String::from("`defer_destroy` is not in the reply"),
            },
        ),
        ("clone-gone", OriginRefusal::CloneGone { planned: recorded.clone() }),
        ("clone-not-a-filesystem", OriginRefusal::CloneIsNotAFilesystem { planned: recorded.clone(), found: 3, snapshot: false }),
        ("clone-is-a-snapshot", OriginRefusal::CloneIsNotAFilesystem { planned: recorded.clone(), found: 2, snapshot: true }),
        ("clone-not-the-planned-one", OriginRefusal::CloneIsNotThePlannedOne { planned: recorded.clone(), live: CLONE_GUID + 1 }),
        ("clone-has-no-origin", OriginRefusal::CloneHasNoOrigin { planned: recorded.clone() }),
        (
            "clone-origin-differs",
            OriginRefusal::CloneOriginDiffers { planned: recorded.clone(), live: String::from("zpcachyos/nosnap/checkout@git-worktree-clone-origin-ffeeddccbbaa9988") },
        ),
        ("origin-dataset-gone", OriginRefusal::OriginDatasetGone { planned: recorded.clone() }),
        ("origin-dataset-not-a-filesystem", OriginRefusal::OriginDatasetIsNotAFilesystem { planned: recorded.clone(), found: 3, snapshot: false }),
        ("origin-dataset-not-the-planned-one", OriginRefusal::OriginDatasetIsNotThePlannedOne { planned: recorded.clone(), live: ORIGIN_DATASET_GUID + 1 }),
        ("origin-snapshot-gone", OriginRefusal::OriginSnapshotGone { planned: recorded.clone() }),
        ("origin-is-not-a-snapshot", OriginRefusal::OriginIsNotASnapshot { planned: recorded.clone() }),
        ("origin-snapshot-not-the-planned-one", OriginRefusal::OriginSnapshotIsNotThePlannedOne { planned: recorded.clone(), live: ORIGIN_SNAPSHOT_GUID + 1 }),
        ("origin-snapshot-createtxg-differs", OriginRefusal::OriginSnapshotCreatetxgDiffers { planned: recorded.clone(), live: ORIGIN_CREATETXG + 1 }),
        (
            "read-failed-on-the-clone",
            OriginRefusal::ReadFailed { planned: recorded.clone(), object: JudgedObject::Clone, name: String::from(CLONE), said: String::from("the device answered EACCES") },
        ),
        (
            "read-failed-on-the-origin-dataset",
            OriginRefusal::ReadFailed { planned: recorded.clone(), object: JudgedObject::OriginDataset, name: String::from(CHECKOUT), said: String::from("the pool is not imported") },
        ),
        (
            "read-failed-on-the-origin-snapshot",
            OriginRefusal::ReadFailed { planned: recorded, object: JudgedObject::OriginSnapshot, name: String::from(ORIGIN_SNAPSHOT), said: String::from("the pool is not imported") },
        ),
    ];
}

//endregion
