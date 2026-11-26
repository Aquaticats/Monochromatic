//! What:
//!  One managed clone at the design's healthy state:
//!  the slot it stands at,
//!  its record's readings,
//!  its Git registration,
//!  the sentence of its managed Git lock,
//!  the removal plan it has none of,
//!  the snapshots it has none of,
//!  the origin it names,
//!  what it and its snapshots retain,
//!  and the holders that keep it.
//! Why:
//!  A healthy slot is the control every other state's control is read
//!  against,
//!  so its whole shape is built once here,
//!  and a control takes one field out of it to read the one answer that moved.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clone = healthyClone(); clone.plan = { reading: 'no-plan' };
//! ```

use super::blocked_removal_checks::BlockedRemovalFacts;
use super::clone_origin_checks::{
    CloneOriginFacts,
    RecordedOrigin,
};
use super::clone_retention_checks::{
    AccountingFacts,
    CloneRetentionFacts,
};
use super::clone_snapshot_checks::CloneSnapshotFacts;
use super::clone_state_checks::CloneStateFacts;
use super::handler_checks::PathState;
use super::holder_checks::HolderFacts;
use super::managed_lock_checks::ManagedLockFacts;
use super::registration_checks::RegistrationFacts;
use super::removal_plan_checks::{
    PlanFileReading,
    RemovalPlanFacts,
};
use super::route_checks::CloneFacts;
use super::route_test_facts_names::{
    ACCOUNT,
    CHECKOUT,
    CHECKOUT_DATASET,
    CLONE_DATASET,
    LOCK_NONCE,
    ORIGIN_SNAPSHOT,
};
use super::zfs_checks::Reading;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_managed_lock::{
    Classification,
    ManagedLockState,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;
use crate::worktree_clone_mount_protocol::RequestNonce;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveInventory;
use crate::worktree_clone_slot_state::slot_state_mounts::{
    MountRows,
    SlotMount,
};
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    IntentFacts,
    PlanReading,
    SlotReadings,
};
use crate::worktree_clone_slot_state::slot_state_test_records::record_at;
use crate::worktree_removal_deferred_origin_check::origin_mark::OriginMark;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use std::path::{
    Path,
    PathBuf,
};

/// The slot the healthy clone stands at,
/// which is the store's own name for it.
pub(crate) const CLONE_SLOT: &str = "one";

/// The clone's whole slot path.
pub(crate) fn clone_slot_path() -> PathBuf {
    return PathBuf::from(format!("{CHECKOUT}/.claude/worktrees/{}", CLONE_SLOT));
}

/// The one accounting value a healthy object carries,
/// with the four named fields a control changes.
/// `referenced` is zero because a freshly cloned dataset references nothing of
/// its own.
pub(crate) fn accounting(
    object: &str,
    used: u64,
) -> AccountingFacts {
    return AccountingFacts {
        object: String::from(object),
        used: PropertyValue::Number(used),
        referenced: PropertyValue::Number(0),
        logical_referenced: PropertyValue::Number(used),
        written: PropertyValue::Number(used),
    };
}

/// The managed Git lock a healthy slot holds:
/// the sentence the add wrote for its nonce,
/// with the slot's pointer beside it.
/// Why:
/// a healthy slot is the control every other state's control is read against.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const classification = healthyClassification(registration); // { state: { kind: 'managed', nonce }, pointer: { kind: 'bytes' } }
/// ```
fn healthy_classification(registration: &Path) -> Classification {
    return Classification {
        state: ManagedLockState::Managed {
            nonce: RequestNonce::parse(LOCK_NONCE).expect("the fixture nonce reads"),
            registration: registration.to_path_buf(),
        },
        pointer: crate::worktree_clone_managed_lock::Reading::Bytes(Vec::from(
            format!("gitdir: {}\n", registration.display()).into_bytes(),
        )),
    };
}

/// One managed clone at the design's healthy state:
/// mounted at its slot,
/// its Git registration complete,
/// no plan,
/// no snapshot of its own,
/// an unmarked origin,
/// no holder and no blocked removal.
pub(crate) fn healthy_clone() -> CloneFacts {
    let slot_path: PathBuf = clone_slot_path();
    let mut record = record_at(Phase::Filled, slot_path.as_path());
    record.clone_dataset = String::from(CLONE_DATASET);
    record.origin_snapshot = String::from(ORIGIN_SNAPSHOT);
    record.origin_dataset = Some(String::from(CHECKOUT_DATASET));
    let registration: PathBuf = PathBuf::from("/home/user/Monochromatic/.git/worktrees/one");
    record.registration = Some(registration.as_os_str().as_encoded_bytes().to_vec());
    let binding: BindingReading = BindingReading::Record(Box::new(record));
    let readings: SlotReadings = SlotReadings {
        slot: String::from(CLONE_SLOT),
        slot_path: slot_path.clone(),
        binding,
        plan: PlanReading::NoPlan,
        intent: IntentFacts::Absent,
        mounts: MountRows::Rows(vec![MountedDatasetInstance {
            dataset: String::from(CLONE_DATASET),
            mount_point: slot_path.clone(),
            root: PathBuf::from("/"),
        }]),
        slot_mount: SlotMount::Something {
            described: String::from("a zfs mount of the clone dataset"),
        },
        registration_stands: Some(true),
    };
    let holders: HolderFacts = HolderFacts {
        clone: String::from(CLONE_DATASET),
        holders: Vec::new(),
        incomplete: Vec::new(),
    };
    let lock: ManagedLockFacts = ManagedLockFacts {
        slot: String::from(CLONE_SLOT),
        classification: healthy_classification(&registration),
    };
    return CloneFacts {
        state: CloneStateFacts {
            readings,
            staging_binds: Vec::new(),
        },
        registration: RegistrationFacts {
            slot: String::from(CLONE_SLOT),
            recorded: Some(registration),
            at_the_path: PathState::Directory {
                owner: ACCOUNT,
                mode: 0o755,
            },
            gitdir: Some(slot_path.join(".git")),
            slot_git: slot_path.join(".git"),
        },
        lock,
        plan: RemovalPlanFacts {
            clone: String::from(CLONE_DATASET),
            reading: PlanFileReading::NoPlan,
        },
        snapshots: CloneSnapshotFacts {
            clone: String::from(CLONE_DATASET),
            clone_createtxg: 121_294,
            live: Reading::Read(LiveInventory {
                dataset: String::from(CLONE_DATASET),
                dataset_guid: DatasetGuid::parse("7").expect("the fixture GUID reads"),
                objects: Vec::new(),
            }),
            plan_stands: false,
            recorded: Vec::new(),
        },
        origin: CloneOriginFacts {
            clone: String::from(CLONE_DATASET),
            recorded: Some(RecordedOrigin {
                dataset: String::from(CHECKOUT_DATASET),
                snapshot: String::from(ORIGIN_SNAPSHOT),
            }),
            live: Some(Reading::Read(OriginMark::Unmarked)),
        },
        retention: CloneRetentionFacts {
            clone: String::from(CLONE_DATASET),
            dataset: Reading::Read(accounting(CLONE_DATASET, 4_096)),
            snapshots: Vec::new(),
            inherited: Vec::new(),
            origin_snapshot: Some(String::from(ORIGIN_SNAPSHOT)),
        },
        blocked: BlockedRemovalFacts {
            slot: String::from(CLONE_SLOT),
            clone: String::from(CLONE_DATASET),
            intent: None,
            plan_stands: false,
            recorded: Vec::new(),
            holders: holders.clone(),
            in_the_callers_session: Vec::new(),
            the_forced_arm_is_measured: false,
        },
        holders,
    };
}
