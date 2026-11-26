//! What:
//!  The controls of one cleanup attempt:
//!  a free snapshot finishes the record,
//!  a clone or a hold blocks as a dependency,
//!  a mark blocks as deferred,
//!  a promoted dataset blocks as unproved,
//!  a refused request keeps its errno,
//!  and an absence proof that still finds the snapshot is a refusal.
//! Why:
//!  The kernel's dependency rule is the proof and an error code is never a conclusion,
//!  so each control plants one state and reads the record's own reason back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = attemptTheCleanup({ record, reads, device, attemptedAt });
//! expect(outcome.kind).toBe('finished');
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_origin_cleanup::origin_cleanup_attempt::{
    CleanupAttempt,
    CleanupOutcome,
    attempt_the_cleanup,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_fixtures::{
    ORIGIN_DATASET,
    ORIGIN_DATASET_GUID,
    ORIGIN_SNAPSHOT_GUID,
    ORIGIN_TXG,
    OWN,
    an_owed_record,
    the_full_name,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use crate::worktree_removal_deferred_origin_check::fixtures::origin_properties;
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    MARK_OFF,
    MARK_ON,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DeviceError,
    Error,
    NvList,
    Operation,
};
use std::cell::{
    Cell,
    RefCell,
};
use std::rc::Rc;

/// The moment every attempt records.
const NOW: &str = "2026-11-25T09:15:01-05:00";

/// What:
///  A pool whose origin dataset and snapshot list a control scripts and a destroy empties.
/// Why:
///  The attempt's last step is an absence proof over the same list the destroy just changed,
///  so a fixed list could never show a completion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = MutablePool.of({ originTxg, clones, mark, present: true });
/// ```
struct MutablePool {
    /// The origin dataset's creation group.
    origin_txg: u64,
    /// How many clones the snapshot reports.
    clones: u64,
    /// The mark the snapshot reports.
    mark: u64,
    /// Whether the snapshot is still listed,
    /// which the device shares with this pool.
    present: Rc<Cell<bool>>,
    /// The dataset GUID the origin dataset answers,
    /// when a control plants another one.
    dataset_guid: u64,
    /// The snapshot the dataset answers as its origin,
    /// when a control plants a promotion.
    dataset_origin: String,
    /// The creation group of that origin snapshot,
    /// which a promotion moves to the recorded snapshot's group or later.
    origin_group: u64,
}

impl MutablePool {
    /// A free snapshot under its own dataset.
    fn free() -> MutablePool {
        return MutablePool {
            origin_txg: 2,
            clones: 0,
            mark: MARK_OFF,
            present: Rc::new(Cell::new(true)),
            dataset_guid: ORIGIN_DATASET_GUID,
            dataset_origin: String::new(),
            origin_group: 0,
        };
    }
}

impl RemovalPoolReads for MutablePool {
    fn identity(
        &self,
        name: &str,
    ) -> Result<DatasetIdentity, String> {
        if name == ORIGIN_DATASET {
            return Ok(DatasetIdentity {
                stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(
                    self.dataset_guid,
                    false,
                    self.origin_txg,
                    1,
                    self.dataset_origin.as_str(),
                ),
                properties: NvList::new(),
            });
        }
        if name == self.dataset_origin && !self.dataset_origin.is_empty() {
            return Ok(DatasetIdentity {
                stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(
                    9_999,
                    true,
                    self.origin_group,
                    1,
                    "",
                ),
                properties: NvList::new(),
            });
        }
        if name == the_full_name().as_str() {
            if !self.present.get() {
                return Err(String::from("the snapshot is gone"));
            }
            return Ok(DatasetIdentity {
                stats: crate::worktree_clone_removal_proof_and_plan::removal_fixtures::the_stats(
                    ORIGIN_SNAPSHOT_GUID,
                    true,
                    ORIGIN_TXG,
                    self.clones,
                    "",
                ),
                properties: origin_properties(Some(self.mark), Some(0)),
            });
        }
        return Err(format!("`{name}` does not answer this scripted pool"));
    }

    fn snapshots_of(
        &self,
        dataset: &str,
        _dataset_guid: DatasetGuid,
    ) -> Result<Vec<LiveObject>, String> {
        if dataset != ORIGIN_DATASET || !self.present.get() {
            return Ok(Vec::new());
        }
        return Ok(vec![LiveObject {
            kind: EntryKind::Snapshot,
            name: String::from(OWN),
            guid: DatasetGuid::parse(ORIGIN_SNAPSHOT_GUID.to_string().as_str()).expect("a GUID"),
            createtxg: ORIGIN_TXG,
            owning_dataset: String::from(ORIGIN_DATASET),
            owning_dataset_guid: DatasetGuid::parse(ORIGIN_DATASET_GUID.to_string().as_str())
                .expect("a GUID"),
        }]);
    }

    fn bookmarks_of(
        &self,
        _dataset: &str,
        _dataset_guid: DatasetGuid,
    ) -> Result<Vec<LiveObject>, String> {
        return Ok(Vec::new());
    }
}

/// What:
///  The device one cleanup request goes to:
///  it answers success and clears the pool's snapshot by default,
///  or the errno a control plants.
/// Why:
///  The absence proof is what settles the record,
///  so a device that answered success and left the list whole is the "success,
/// still listed" arm.
struct ScriptedOne {
    /// The pool's own listed flag,
    /// which a successful destroy clears.
    present: Rc<Cell<bool>>,
    /// The errno the one request answers,
    /// when a control plants a refusal.
    refuses: Option<i32>,
    /// How many requests were sent.
    sent: RefCell<Vec<String>>,
}

impl SnapshotDestroyDevice for ScriptedOne {
    fn destroy_snaps(
        &self,
        _pool: &str,
        names: &[&str],
    ) -> Result<Option<NvList>, DeviceError> {
        self.sent
            .borrow_mut()
            .extend(names.iter().map(|name| return String::from(*name)));
        if let Some(errno) = self.refuses {
            return Err(DeviceError::Request {
                source: Error::Call {
                    operation: Operation::DestroySnaps,
                    target: String::from("tank"),
                    errno,
                },
            });
        }
        self.present.set(false);
        return Ok(None);
    }
}

/// One attempt over one pool and device.
fn attempt(
    pool: &MutablePool,
    device: &ScriptedOne,
) -> CleanupOutcome {
    let record: OriginCleanupRecord = an_owed_record();
    let the_attempt: CleanupAttempt<'_> = CleanupAttempt {
        record: &record,
        reads: pool,
        device,
        attempted_at: NOW,
    };
    return attempt_the_cleanup(&the_attempt);
}

#[test]
fn a_free_snapshot_is_destroyed_and_the_record_finishes() {
    let pool: MutablePool = MutablePool::free();
    let device: ScriptedOne = ScriptedOne {
        present: Rc::clone(&pool.present),
        refuses: None,
        sent: RefCell::new(Vec::new()),
    };
    let outcome: CleanupOutcome = attempt(&pool, &device);
    assert_eq!(
        device.sent.borrow().clone(),
        vec![the_full_name()],
        "one request naming exactly the recorded snapshot"
    );
    match outcome {
        CleanupOutcome::Finished {
            destroyed,
        } => assert_eq!(destroyed.as_deref(), Some(the_full_name().as_str())),
        other => panic!("a free snapshot finishes the record, not {other:?}"),
    }
}

#[test]
fn a_clone_or_a_hold_blocks_as_a_dependency_with_its_counts() {
    let mut pool: MutablePool = MutablePool::free();
    pool.clones = 1;
    let device: ScriptedOne = ScriptedOne {
        present: Rc::clone(&pool.present),
        refuses: None,
        sent: RefCell::new(Vec::new()),
    };
    let outcome: CleanupOutcome = attempt(&pool, &device);
    assert!(
        device.sent.borrow().is_empty(),
        "no request is sent to a pinned snapshot"
    );
    match outcome {
        CleanupOutcome::Blocked {
            record,
            detail,
        } => {
            assert_eq!(
                record.block.map(|held| return held.reason),
                Some(BlockReason::Dependency)
            );
            assert_eq!(record.block.map(|held| return held.clones), Some(1));
            assert!(detail.contains("1 clone(s)"));
        }
        other => panic!("a clone blocks, not {other:?}"),
    }
}

#[test]
fn a_mark_blocks_as_deferred() {
    let mut pool: MutablePool = MutablePool::free();
    pool.mark = MARK_ON;
    let device: ScriptedOne = ScriptedOne {
        present: Rc::clone(&pool.present),
        refuses: None,
        sent: RefCell::new(Vec::new()),
    };
    let outcome: CleanupOutcome = attempt(&pool, &device);
    match outcome {
        CleanupOutcome::Blocked {
            record,
            ..
        } => assert_eq!(
            record.block.map(|held| return held.reason),
            Some(BlockReason::Deferred)
        ),
        other => panic!("a mark blocks, not {other:?}"),
    }
}

#[test]
fn a_promoted_dataset_blocks_as_unproved() {
    let mut pool: MutablePool = MutablePool::free();
    pool.dataset_origin = String::from("tank/wt/later-clone@later");
    pool.origin_group = ORIGIN_TXG;
    let device: ScriptedOne = ScriptedOne {
        present: Rc::clone(&pool.present),
        refuses: None,
        sent: RefCell::new(Vec::new()),
    };
    let outcome: CleanupOutcome = attempt(&pool, &device);
    match outcome {
        CleanupOutcome::Blocked {
            record,
            detail,
        } => {
            assert_eq!(
                record.block.map(|held| return held.reason),
                Some(BlockReason::Unproved)
            );
            assert!(detail.contains("promotion"), "{detail}");
        }
        other => panic!("a promotion proves nothing, not {other:?}"),
    }
}

#[test]
fn another_dataset_guid_blocks_as_unproved() {
    let mut pool: MutablePool = MutablePool::free();
    pool.dataset_guid = ORIGIN_DATASET_GUID + 1;
    let device: ScriptedOne = ScriptedOne {
        present: Rc::clone(&pool.present),
        refuses: None,
        sent: RefCell::new(Vec::new()),
    };
    let outcome: CleanupOutcome = attempt(&pool, &device);
    match outcome {
        CleanupOutcome::Blocked {
            record,
            ..
        } => assert_eq!(
            record.block.map(|held| return held.reason),
            Some(BlockReason::Unproved)
        ),
        other => panic!("a replaced dataset proves nothing, not {other:?}"),
    }
}

#[test]
fn a_refused_request_keeps_its_errno() {
    let pool: MutablePool = MutablePool::free();
    let device: ScriptedOne = ScriptedOne {
        present: Rc::clone(&pool.present),
        refuses: Some(libc::EPERM),
        sent: RefCell::new(Vec::new()),
    };
    let outcome: CleanupOutcome = attempt(&pool, &device);
    match outcome {
        CleanupOutcome::Blocked {
            record,
            ..
        } => {
            assert_eq!(
                record.block.map(|held| return held.reason),
                Some(BlockReason::Refused)
            );
            assert_eq!(
                record.block.map(|held| return held.errno),
                Some(u32::try_from(libc::EPERM).expect("a positive errno"))
            );
        }
        other => panic!("a refused request blocks, not {other:?}"),
    }
}

#[test]
fn a_success_whose_absence_proof_still_finds_the_snapshot_is_a_refusal() {
    let pool: MutablePool = MutablePool::free();
    let stubborn: StubbornDevice = StubbornDevice;
    let standing: OriginCleanupRecord = an_owed_record();
    let outcome: CleanupOutcome = attempt_the_cleanup(&CleanupAttempt {
        record: &standing,
        reads: &pool,
        device: &stubborn,
        attempted_at: NOW,
    });
    match outcome {
        CleanupOutcome::Blocked {
            record,
            detail,
        } => {
            assert_eq!(
                record.block.map(|held| return held.reason),
                Some(BlockReason::Refused)
            );
            assert!(detail.contains("still listed"), "{detail}");
        }
        other => panic!("an unproved absence is a refusal, not {other:?}"),
    }
}

#[test]
fn an_already_absent_snapshot_finishes_without_a_request() {
    let pool: MutablePool = MutablePool::free();
    pool.present.set(false);
    let device: ScriptedOne = ScriptedOne {
        present: Rc::clone(&pool.present),
        refuses: None,
        sent: RefCell::new(Vec::new()),
    };
    let outcome: CleanupOutcome = attempt(&pool, &device);
    assert!(
        device.sent.borrow().is_empty(),
        "a destroy of an absent name is a no-op this record does not send"
    );
    assert_eq!(
        outcome,
        CleanupOutcome::Finished {
            destroyed: None
        }
    );
}

/// A device that answers success and destroys nothing,
/// which is the "success,
/// still listed" arm.
struct StubbornDevice;

impl SnapshotDestroyDevice for StubbornDevice {
    fn destroy_snaps(
        &self,
        _pool: &str,
        _names: &[&str],
    ) -> Result<Option<NvList>, DeviceError> {
        return Ok(None);
    }
}
