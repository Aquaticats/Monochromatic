//! What:
//!  One managed clone's whole fact set for plan step 18's `doctor` report,
//!  gathered from one request and one set of reads.
//! Why:
//!  The report prints one group per managed clone,
//!  and a group's lines must describe one clone at one moment,
//!  so every read of that clone happens once here and every value the lines
//!  judge is built in this one place
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "What it builds";
//!  `package/git-policy/cli/SPEC.md`,
//!  "The ZFS route's checks,
//! built and reached by no command").
//!  A read that fails is carried into the fact's own unknown-carrying shape,
//!  never dropped and never replaced by a plausible zero:
//!  `Reading::Unreadable` for the pool,
//!  `PlanFileReading::Unreadable` for a plan nobody read,
//!  `PathState::Unreadable` for a link this account may not look at,
//!  `PropertyValue::Unreadable` for one accounting field,
//!  and a sentence in the holder scan's own list of incomplete reads.
//!  The one number with no such shape is the clone's creation group,
//!  and it is `0` exactly where the identity read failed,
//!  which is the answer the snapshot line stops at before it reads the number.
//!  Nothing here writes:
//!  no record,
//!  no mount,
//!  no property and no file is created or removed,
//!  and the scan it asks for reads `/proc` alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = gatherCloneFacts({ commonDir, slot, slotPath, cache, callerTask }, reads);
//! routeChecks({ ...machine, clones: [facts] }).forEach((report) => console.log(report.line()));
//! ```

/// The blocked-removal state.
use super::blocked_removal_checks::BlockedRemovalFacts;
/// The namespaces that hold one clone.
use super::clone_facts_gathering_holders::holders_of;
/// The registration one record names,
/// and the origin with its mark.
use super::clone_facts_gathering_records::{
    origin_facts,
    registration_facts,
};
/// What one clone and its own snapshots retain.
use super::clone_facts_gathering_retention::retention_of;
/// Every read one clone's facts come from,
/// and the request they answer.
use super::clone_facts_reads::{
    CloneFactsIdentity,
    CloneFactsReads,
    CloneFactsRequest,
};
/// The origin an add recorded and the mark on it.
use super::clone_origin_checks::CloneOriginFacts;
/// What one clone and its snapshots retain.
use super::clone_retention_checks::CloneRetentionFacts;
/// The clone's own snapshots beside the plan's states.
use super::clone_snapshot_checks::CloneSnapshotFacts;
/// What one managed slot is.
use super::clone_state_checks::CloneStateFacts;
/// The namespaces that hold a clone.
use super::holder_checks::HolderFacts;
/// The sentence one slot's managed Git lock prints.
use super::managed_lock_checks::ManagedLockFacts;
/// The Git registration one slot's record names.
use super::registration_checks::RegistrationFacts;
/// The destruction plan one removal left,
/// and the outcomes its lines record.
use super::removal_plan_checks::{
    PlanFileReading,
    RecordedSnapshot,
    RemovalPlanFacts,
    recorded_outcomes,
};
/// The facts every report is built from.
use super::route_checks::CloneFacts;
/// The staging folder's live binds.
use super::staging_folder_checks::staging_binds;
/// What a read of one object answered.
use super::zfs_checks::Reading;
/// The record one add wrote,
/// and what step 12 read of it.
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
/// The pool's own GUID for one dataset.
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
/// The snapshots and bookmarks one clone holds.
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveInventory;
/// The intent the clone's own property carries,
/// and the record,
/// plan,
/// intent and mounts step 12 read.
use crate::worktree_clone_slot_state::slot_state_readings::{
    BindingReading,
    IntentFacts,
    SlotReadings,
};
/// What:
///  One managed clone's facts,
///  gathered from one request.
/// Why:
///  The report's group for a clone is one value,
///  so the caller reads it once and no line can be built from another
///  clone's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = gatherCloneFacts(request, reads); // CloneFacts
/// ```
pub fn gather_clone_facts(
    request: &CloneFactsRequest,
    reads: &dyn CloneFactsReads,
) -> CloneFacts {
    let readings: SlotReadings = reads.slot_readings(request);
    let record: Option<&BindingRecord> = match &readings.binding {
        BindingReading::Record(held) => Some(held.as_ref()),
        BindingReading::Absent
        | BindingReading::Unreadable {
            ..
        } => None,
    };
    let slot: &str = request.slot.as_str();
    let clone: String = match record {
        Some(held) => held.clone_dataset.clone(),
        None => String::from(slot),
    };
    let no_record: String = format!(
        "the binding record filed under the slot `{slot}` names no clone dataset this report can read, so no live object answers for `{clone}`"
    );
    let identity: Reading<CloneFactsIdentity> = match record {
        Some(_) => reads.clone_identity(clone.as_str()),
        None => Reading::Unreadable {
            why: no_record,
        },
    };
    let clone_guid: Option<DatasetGuid> = record.and_then(|held| return held.clone_guid);
    let inventory: Reading<LiveInventory> = match clone_guid
        .or(the_guid_the_pool_answered(&identity))
    {
        Some(guid) => reads.inventory(clone.as_str(), guid),
        None => Reading::Unreadable {
            why: format!(
                "the record filed under the slot `{slot}` names no clone GUID and the pool answered none for `{clone}`, so no snapshot inventory can be named for it"
            ),
        },
    };
    let plan: RemovalPlanFacts = plan_facts(request, reads, clone.as_str(), clone_guid);
    let recorded: Vec<RecordedSnapshot> = recorded_outcomes_of(&plan.reading);
    let plan_stands: bool = matches!(
        plan.reading,
        PlanFileReading::Standing { .. } | PlanFileReading::Unreadable { .. }
    );
    let holders: HolderFacts = holders_of(reads, clone.as_str(), clone_guid, request.caller_task);
    let snapshots: CloneSnapshotFacts = CloneSnapshotFacts {
        clone: clone.clone(),
        clone_createtxg: the_group_the_pool_answered(&identity),
        live: inventory.clone(),
        plan_stands,
        recorded: recorded.clone(),
    };
    let held_tasks: Vec<u32> = holders
        .holders
        .iter()
        .map(|holder| return holder.representative_pid)
        .collect();
    let retention: CloneRetentionFacts =
        retention_of(reads, clone.as_str(), record, &snapshots, &inventory);
    let origin: CloneOriginFacts = origin_facts(reads, clone.as_str(), record);
    let registration: RegistrationFacts = registration_facts(request, reads, record);
    // The lock's proof reads the record and the mount table itself, so it is
    // asked once here and the state's own readings are moved into the facts
    // after every read that borrows them.
    let lock: ManagedLockFacts = ManagedLockFacts {
        slot: String::from(slot),
        classification: reads.lock_classification(request),
    };
    let blocked: BlockedRemovalFacts = BlockedRemovalFacts {
        slot: String::from(slot),
        clone: clone.clone(),
        intent: the_intent_of(&readings),
        plan_stands,
        recorded,
        holders: holders.clone(),
        in_the_callers_session: reads.in_the_callers_session(held_tasks.as_slice()),
        the_forced_arm_is_measured: reads.the_forced_arm_is_measured(),
    };
    let state: CloneStateFacts = CloneStateFacts {
        staging_binds: staging_binds(reads.mount_rows().as_slice(), request.cache.as_path()),
        readings,
    };
    return CloneFacts {
        state,
        registration,
        lock,
        plan,
        retention,
        origin,
        snapshots,
        blocked,
        holders,
    };
}

/// The GUID the pool answered for the clone,
/// where the record names none.
fn the_guid_the_pool_answered(identity: &Reading<CloneFactsIdentity>) -> Option<DatasetGuid> {
    return match identity {
        Reading::Read(held) => Some(held.guid),
        Reading::Missing {
            ..
        }
        | Reading::Unreadable {
            ..
        } => None,
    };
}

/// The clone's creation group,
/// which is `0` only where the identity read failed and the snapshot line
/// answers unknown before it reads this number.
fn the_group_the_pool_answered(identity: &Reading<CloneFactsIdentity>) -> u64 {
    return match identity {
        Reading::Read(held) => held.createtxg,
        Reading::Missing {
            ..
        }
        | Reading::Unreadable {
            ..
        } => 0,
    };
}

/// What stands at the destruction plan's own file name.
fn plan_facts(
    request: &CloneFactsRequest,
    reads: &dyn CloneFactsReads,
    clone: &str,
    clone_guid: Option<DatasetGuid>,
) -> RemovalPlanFacts {
    let reading: PlanFileReading = match clone_guid {
        Some(guid) => reads.plan_reading(request, guid),
        // A plan is filed under the clone's GUID, so a record that names none
        // proves no plan can stand for this clone.
        None => PlanFileReading::NoPlan,
    };
    return RemovalPlanFacts {
        clone: String::from(clone),
        reading,
    };
}

/// Every name a standing plan holds,
/// with the outcome its history ends on.
///
/// A plan nobody read holds no name this report can state,
/// so an unreadable plan stands without lines rather than with invented ones.
fn recorded_outcomes_of(reading: &PlanFileReading) -> Vec<RecordedSnapshot> {
    return match reading {
        PlanFileReading::Standing {
            plan,
            ..
        } => recorded_outcomes(plan),
        PlanFileReading::NoPlan
        | PlanFileReading::Unreadable {
            ..
        } => Vec::new(),
    };
}

/// The `unmount` intent standing on the clone,
/// and nothing when no intent of the clone's own stands.
fn the_intent_of(readings: &SlotReadings) -> Option<String> {
    return match &readings.intent {
        IntentFacts::Unmounting {
            nonce,
        } => Some(nonce.clone()),
        IntentFacts::Unasked
        | IntentFacts::Absent
        | IntentFacts::Mounting {
            ..
        }
        | IntentFacts::Inherited {
            ..
        }
        | IntentFacts::Unreadable {
            ..
        } => None,
    };
}
