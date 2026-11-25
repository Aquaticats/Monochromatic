//! What:
//!  Plan step 18's removal-side `doctor` lines against a real file-backed pool,
//!  in a throwaway guest:
//!  the healthy clone passes every line,
//!  and each broken fact moves the line that judges it.
//! Why:
//!  The lines read `/dev/zfs`,
//!  a real pool's property replies,
//!  the staging folder's live mounts and the all-namespace scan,
//!  so no host control may show them
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "VM measurements";
//!  `package/git-policy/cli/e2e/doctor-zfs-route-guest.ts`).
//!  The arm builds the removal side's own fixture:
//!  one pool,
//!  a checkout with its origin snapshot,
//!  a clone with its own snapshot,
//!  a Git registration,
//!  and the binding record an add would have left
//!  (`removal_live_fixtures_tests.rs`,
//!  which steps 14 to 16's arms share).
//!  It prints `SKIPPED` where its own variable names no guest,
//!  so a host run reaches no pool and no mount.
//!  Every claim is a marker line the driver requires,
//!  so a section that silently stopped running fails the run instead of
//!  printing less.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = LivePoolFixture.fromEnvironment('doctor', variables);
//! fixture.build(['worktree-own']); expect(cloneFacts(fixture).every(pass)).toBe(true);
//! ```

use super::blocked_removal_checks::{BlockedRemovalFacts, check_blocked_removal};
use super::check_outcome::CheckOutcome;
use super::clone_origin_checks::{CloneOriginFacts, RecordedOrigin, check_clone_origin};
use super::clone_retention_checks::{AccountingFacts, CloneRetentionFacts, LOGICAL_REFERENCED_PROPERTY, REFERENCED_PROPERTY, USED_PROPERTY, WRITTEN_PROPERTY, check_clone_retention};
use super::clone_snapshot_checks::{CloneSnapshotFacts, check_clone_snapshots};
use super::clone_state_checks::{CloneStateFacts, check_clone_state};
use super::handler_checks::PathState;
use super::holder_checks::{HolderFacts, NamespaceHold, check_holders};
use super::registration_checks::{RegistrationFacts, check_registration};
use super::removal_plan_checks::{PlanFileReading, RemovalPlanFacts, check_removal_plan, recorded_outcomes};
use super::staging_folder_checks::{StagingBind, staging_binds};
use super::zfs_checks::{Reading, reading_of};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::value_of;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_mount_protocol::{DESIRED_STATE_PROPERTY, MountOperation};
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanFacts};
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
use crate::worktree_clone_removal_proof_and_plan::removal_live_fixtures::{LivePoolFixture, PoolVariables, SLOT};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{LiveRemovalReads, whole_inventory};
use crate::worktree_clone_removal_proof_and_plan::started_moment::now_with_offset;
use crate::worktree_clone_slot_state::slot_state_live_read::{GatedCloneIntentReads, SlotReadingSources, read_slot};
use crate::worktree_clone_slot_state::slot_state_readings::{BindingReading, IntentFacts, SlotReadings};
use crate::worktree_removal_deferred_origin_check::origin_mark::{OriginMark, origin_mark};
use crate::zfs_compatibility_gate::LiveGate;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, ZfsDevice};
use std::path::{Path, PathBuf};

/// The variable naming this arm's guest.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_DOCTOR_REMOVAL_GUEST";

/// The variable naming this arm's pool.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_DOCTOR_REMOVAL_POOL";

/// The variable naming this guest's OpenZFS prefix.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_DOCTOR_REMOVAL_PREFIX";

/// The variable naming the folder this arm's fixture lives under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_DOCTOR_REMOVAL_ROOT";

/// The folder the fixture lives in when the driver names none.
const DEFAULT_ROOT: &str = "/home/zfs-doctor-removal-fixture";

/// The pool this arm builds when the driver names none.
const DEFAULT_POOL: &str = "zfsdoctorremoval";

/// The one intent nonce this arm writes.
const NONCE: &str = "0123456789abcdef0123456789abcdef";

//region The arm's own fixture

/// What:
///  The variables this arm's guest and pool are named by.
/// Why:
///  Two guest arms run in one crate run,
///  so each arm names its own pool and no arm destroys another's objects.
fn variables() -> PoolVariables {
    return PoolVariables { guest: GUEST_VARIABLE, pool: POOL_VARIABLE, prefix: PREFIX_VARIABLE, root: ROOT_VARIABLE, default_root: DEFAULT_ROOT, default_pool: DEFAULT_POOL };
}

/// What:
///  The fixture one arm runs against,
///  built and given the add's records.
/// Why:
///  A record that names a clone the pool does not hold would make every line
///  unknown,
///  so the fixture's own identities are read from the live objects.
fn fixture() -> Option<(LivePoolFixture, PathBuf)> {
    let built: LivePoolFixture = LivePoolFixture::from_environment("the doctor removal lines", &variables())?;
    built.build(&[OWN_SNAPSHOT]);
    let recorded = built.recorded();
    built.write_the_adds_records(&recorded);
    let cache: PathBuf = built.root.join("cache");
    std::fs::create_dir_all(cache.as_path()).expect("the cache");
    return Some((built, cache));
}

/// The own snapshot the fixture's clone holds.
const OWN_SNAPSHOT: &str = "worktree-own";

/// The staging folder's name for the fixture's clone.
fn staging_folder(cache: &Path, guid: DatasetGuid) -> PathBuf {
    return cache.join(crate::worktree_clone_unmount::staging_folder_name::staging_folder_name(guid));
}

//endregion

//region The facts one clone's lines read

/// One mount table's rows,
/// parsed by the repository's own reader.
fn mount_rows() -> Vec<crate::fs_mount_options::MountFacts> {
    let bytes: Vec<u8> = std::fs::read("/proc/self/mountinfo").expect("the mount table");
    return crate::fs_mount_options::parse_mount_table(bytes.as_slice());
}

/// One dataset's or snapshot's accounting fields,
/// from its own object-stats reply.
fn accounting(device: &ZfsDevice, object: &str) -> AccountingFacts {
    let held: DatasetIdentity = device.objset_stats(object).unwrap_or_else(|error| panic!("{object}'s accounting: {error}"));
    return AccountingFacts {
        object: String::from(object),
        used: value_of(&held.properties, USED_PROPERTY),
        referenced: value_of(&held.properties, REFERENCED_PROPERTY),
        logical_referenced: value_of(&held.properties, LOGICAL_REFERENCED_PROPERTY),
        written: value_of(&held.properties, WRITTEN_PROPERTY),
    };
}

/// What one slot's readings are,
/// from the machine.
fn readings_of(common: &Path, slot_path: &Path) -> SlotReadings {
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    let intents: GatedCloneIntentReads<'_> = GatedCloneIntentReads::new(&LiveGate);
    return read_slot(common, SLOT, slot_path, &SlotReadingSources { mounts: &mounts, intents: &intents });
}

/// The record one slot's readings hold,
/// or the panic that says the fixture's record did not read.
fn record_of(readings: &SlotReadings) -> &crate::worktree_clone_binding_record::binding_record::BindingRecord {
    return match &readings.binding {
        BindingReading::Record(record) => record,
        other => panic!("the fixture's binding record must read, and it answered {other:?}"),
    };
}

/// One slot's registration facts,
/// with the registration's own `gitdir` read.
fn registration_of(readings: &SlotReadings, slot_path: &Path) -> RegistrationFacts {
    let recorded: Option<PathBuf> = record_of(readings).registration_path();
    let at_the_path: PathState = match &recorded {
        None => PathState::Absent,
        Some(path) => path_state(path.as_path()),
    };
    let gitdir: Option<PathBuf> = recorded.as_ref().and_then(|path| return std::fs::read_to_string(path.join("gitdir")).ok()).map(|text| return PathBuf::from(text.trim()));
    return RegistrationFacts { slot: String::from(SLOT), recorded, at_the_path, gitdir, slot_git: slot_path.join(".git") };
}

/// One path's state,
/// in the words the checks print.
fn path_state(path: &Path) -> PathState {
    return match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => PathState::Absent,
        Err(error) => PathState::Unreadable { why: error.to_string() },
        Ok(metadata) if metadata.is_dir() => PathState::Directory { owner: owner_of(&metadata), mode: mode_of(&metadata) },
        Ok(metadata) if metadata.is_file() => PathState::File { owner: owner_of(&metadata), mode: mode_of(&metadata) },
        Ok(_) => PathState::WrongKind { found: String::from("a link or a device node") },
    };
}

/// A file's owner.
fn owner_of(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::MetadataExt as _;
    return metadata.uid();
}

/// A file's permission bits.
fn mode_of(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::MetadataExt as _;
    return metadata.mode() & 0o7777;
}

/// What stands at the plan's file name for one clone.
fn plan_of(common: &Path, readings: &SlotReadings) -> RemovalPlanFacts {
    let clone: String = record_of(readings).clone_dataset.clone();
    let Some(found) = record_of(readings).clone_guid else {
        return RemovalPlanFacts { clone, reading: PlanFileReading::NoPlan };
    };
    let path: PathBuf = crate::worktree_clone_binding_record::registration_home::plans_folder(common).join(found.file_name());
    return RemovalPlanFacts {
        clone,
        reading: match read_an_existing_plan(common, found) {
            Ok(None) => PlanFileReading::NoPlan,
            Ok(Some(plan)) => PlanFileReading::Standing { path, plan: Box::new(plan) },
            Err(refusal) => PlanFileReading::Unreadable { path, reason: format!("{refusal:?}") },
        },
    };
}

/// The clone's own snapshots beside the plan's states.
fn snapshots_of(clone: &str, clone_createtxg: u64, plan: &RemovalPlanFacts) -> CloneSnapshotFacts {
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let guid: DatasetGuid = DatasetGuid::parse(device.objset_stats(clone).expect("the clone answers").stats.dds_guid.to_string().as_str()).expect("the GUID is canonical");
    let live: Reading<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveInventory> = reading_of(whole_inventory(&reads, clone, guid).map_err(|said| return ReadFailure { said, errno: None }));
    let recorded_states = match &plan.reading {
        PlanFileReading::Standing { plan: standing, .. } => recorded_outcomes(standing),
        PlanFileReading::NoPlan | PlanFileReading::Unreadable { .. } => Vec::new(),
    };
    return CloneSnapshotFacts { clone: String::from(clone), clone_createtxg, live, recorded: recorded_states };
}

/// The origin the record names and the mark its snapshot carries.
fn origin_of(readings: &SlotReadings) -> CloneOriginFacts {
    let record = record_of(readings);
    let Some(dataset) = record.origin_dataset.clone() else {
        return CloneOriginFacts { clone: record.clone_dataset.clone(), recorded: None, live: None };
    };
    let snapshot: String = record.origin_snapshot.clone();
    let reader: crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader = crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader::open().expect("the device opens in the guest");
    let mark: OriginMark = origin_mark(&reader, snapshot.as_str());
    return CloneOriginFacts { clone: record.clone_dataset.clone(), recorded: Some(RecordedOrigin { dataset, snapshot }), live: Some(Reading::Read(mark)) };
}

/// What one clone and its snapshots retain.
fn retention_of(clone: &str, readings: &SlotReadings, snapshots: &CloneSnapshotFacts) -> CloneRetentionFacts {
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let dataset: Reading<AccountingFacts> = reading_of(Ok(accounting(&device, clone)));
    let mut own: Vec<AccountingFacts> = Vec::new();
    let mut inherited: Vec<String> = Vec::new();
    if let Reading::Read(inventory) = &snapshots.live {
        for object in inventory.snapshots() {
            if object.createtxg > snapshots.clone_createtxg {
                own.push(accounting(&device, object.full_name().as_str()));
            } else {
                inherited.push(object.full_name());
            }
        }
    }
    return CloneRetentionFacts { clone: String::from(clone), dataset, snapshots: own, inherited, origin_snapshot: Some(record_of(readings).origin_snapshot.clone()) };
}

/// The namespaces that hold one clone,
/// from step 9's all-namespace scan,
/// the reader's own namespace included.
fn holders_of(clone: &str) -> HolderFacts {
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let guid: DatasetGuid = DatasetGuid::parse(device.objset_stats(clone).expect("the clone answers").stats.dds_guid.to_string().as_str()).expect("the GUID is canonical");
    let nonce: RequestNonce = RequestNonce::parse(NONCE).expect("the arm's nonce");
    let own_namespace: Option<String> = crate::mount_namespace_scan::namespace_target(Path::new("/proc/self/ns/mnt"));
    let scan = crate::clone_holder_scan::scan_holders(&crate::clone_holder_scan::HolderScanRequest {
        proc_root: Path::new("/proc"),
        source: clone,
        own_namespace: own_namespace.as_deref(),
        readers_root: None,
        dataset: guid,
        nonce: &nonce,
        time: now_with_offset(),
    });
    let mut holders: Vec<NamespaceHold> = Vec::new();
    for holder in &scan.holders {
        let pid: u32 = holder.task.file_name().and_then(|name| return name.to_str()).and_then(|text| return text.parse().ok()).unwrap_or(0);
        for mount in &holder.mounts {
            holders.push(NamespaceHold { namespace: String::from(holder.namespace()), representative_pid: pid, source: String::from(clone), mount_point: mount.mount_point.clone() });
        }
    }
    // The reader's own mounts are holders of a clone too: a removal blocked by
    // the caller's own mount is the state the caller must see, and the scan
    // keeps that table apart because a local unmount is the first thing it asks.
    if !scan.own_mounts.is_empty() {
        let own_pid: u32 = std::process::id();
        for mount in &scan.own_mounts {
            holders.push(NamespaceHold { namespace: own_namespace.clone().unwrap_or_else(|| return String::from("mnt:[self]")), representative_pid: own_pid, source: String::from(clone), mount_point: mount.mount_point.clone() });
        }
    }
    let incomplete: Vec<String> = scan.incomplete.iter().map(|read| return format!("{read:?}")).collect();
    return HolderFacts { clone: String::from(clone), holders, incomplete };
}

/// The blocked-removal state,
/// with the scan's holders and whether one of them is the caller's own task.
fn blocked_of(readings: &SlotReadings, plan: &RemovalPlanFacts, holders: HolderFacts) -> BlockedRemovalFacts {
    let recorded_states = match &plan.reading {
        PlanFileReading::Standing { plan: standing, .. } => recorded_outcomes(standing),
        PlanFileReading::NoPlan | PlanFileReading::Unreadable { .. } => Vec::new(),
    };
    let intent: Option<String> = match &readings.intent {
        IntentFacts::Unmounting { nonce } => Some(nonce.clone()),
        _ => None,
    };
    let own_pid: u32 = std::process::id();
    let mine: Vec<u32> = holders.holders.iter().filter(|holder| return holder.representative_pid == own_pid).map(|holder| return holder.representative_pid).collect();
    let plan_stands: bool = matches!(plan.reading, PlanFileReading::Standing { .. });
    return BlockedRemovalFacts { slot: String::from(SLOT), clone: record_of(readings).clone_dataset.clone(), intent, plan_stands, recorded: recorded_states, holders, in_the_callers_session: mine, the_forced_arm_is_measured: false };
}

/// One clone's whole group of facts,
/// read from the machine.
fn clone_facts(common: &Path, slot_path: &Path, cache: &Path) -> super::route_checks::CloneFacts {
    let readings: SlotReadings = readings_of(common, slot_path);
    let clone: String = record_of(&readings).clone_dataset.clone();
    let clone_createtxg: u64 = record_of(&readings).clone_createtxg.unwrap_or(0);
    let plan: RemovalPlanFacts = plan_of(common, &readings);
    let snapshots: CloneSnapshotFacts = snapshots_of(clone.as_str(), clone_createtxg, &plan);
    let holders: HolderFacts = holders_of(clone.as_str());
    let state: CloneStateFacts = CloneStateFacts { readings: readings.clone(), staging_binds: staging_rows(cache) };
    return super::route_checks::CloneFacts {
        state,
        registration: registration_of(&readings, slot_path),
        plan: plan.clone(),
        origin: origin_of(&readings),
        retention: retention_of(clone.as_str(), &readings, &snapshots),
        blocked: blocked_of(&readings, &plan, holders.clone()),
        snapshots,
        holders,
    };
}

/// Every live mount under the cache whose name a removal derived.
fn staging_rows(cache: &Path) -> Vec<StagingBind> {
    return staging_binds(mount_rows().as_slice(), cache);
}

/// One line passed,
/// or the panic that prints what it answered instead.
fn passed(outcome: &CheckOutcome, what: &str) {
    assert!(matches!(outcome, CheckOutcome::Pass { .. }), "{what} passes: {outcome:?}");
}

//endregion

//region The arm

/// The healthy clone passes every line,
/// and each broken fact moves the one that judges it.
#[test]
fn zfs_doctor_removal_lines_guest() {
    let Some((built, cache)) = fixture() else {
        return;
    };
    let common: PathBuf = built.common();
    let slot_path: PathBuf = built.slot_path();
    let clone: String = built.clone();
    write_a_complete_registration(&slot_path, &built.registration());
    let healthy: super::route_checks::CloneFacts = clone_facts(common.as_path(), slot_path.as_path(), cache.as_path());
    assert_passes(&healthy);
    println!("DOCTOR-GUEST-REMOVAL healthy-lines-pass");
    let interrupt: super::route_checks::CloneFacts = publishes_a_plan(&built, &cache);
    assert_interrupted(&interrupt);
    println!("DOCTOR-GUEST-REMOVAL interrupt-state-is-a-finding");
    let held: super::route_checks::CloneFacts = mounts_the_clone(&built, &cache, &slot_path);
    assert_held(&held);
    println!("DOCTOR-GUEST-REMOVAL blocked-removal-names-the-holder");
    unstages(&built, &clone, cache.as_path());
    let lost: super::route_checks::CloneFacts = destroys_the_recorded_snapshot(&built, common.as_path(), slot_path.as_path(), cache.as_path());
    assert_lost(&lost);
    println!("DOCTOR-GUEST-REMOVAL lost-snapshot-is-a-finding");
    let marked: super::route_checks::CloneFacts = marks_the_origin(&built, common.as_path(), slot_path.as_path(), cache.as_path());
    assert_marked(&marked);
    println!("DOCTOR-GUEST-REMOVAL origin-mark-is-a-finding");
    let pruned: super::route_checks::CloneFacts = prunes_the_registration(&built, common.as_path(), slot_path.as_path(), cache.as_path());
    assert_pruned(&pruned);
    println!("DOCTOR-GUEST-REMOVAL pruned-registration-is-a-finding");
    let retained: super::route_checks::CloneFacts = clone_facts(common.as_path(), slot_path.as_path(), cache.as_path());
    assert_retained(&retained);
    println!("DOCTOR-GUEST-REMOVAL retention-names-its-fields");
    built.teardown();
}

/// The healthy clone passes the seven lines,
/// and the scan finds no holder.
fn assert_passes(facts: &super::route_checks::CloneFacts) {
    let state: CheckOutcome = check_clone_state(&facts.state);
    let CheckOutcome::Pass { detail } = state else {
        panic!("the healthy clone's state line passes: {state:?}");
    };
    assert!(detail.contains("known unmounted clone"), "{detail}");
    passed(&check_registration(&facts.registration), "the registration line");
    passed(&check_removal_plan(&facts.plan), "the plan line with no plan");
    passed(&check_clone_snapshots(&facts.snapshots), "the snapshot line");
    let origin: CheckOutcome = check_clone_origin(&facts.origin);
    let CheckOutcome::Pass { detail: origin_detail } = origin else {
        panic!("an unmarked origin passes: {origin:?}");
    };
    assert!(origin_detail.contains("reads 0"), "{origin_detail}");
    passed(&check_clone_retention(&facts.retention), "the retention line");
    passed(&check_blocked_removal(&facts.blocked), "the blocked line with nothing pending");
    passed(&check_holders(&facts.holders), "the holders line with nothing mounted");
}

/// A published plan and a written `unmount` intent leave the slot a removal in
/// flight,
/// which the state line names.
fn publishes_a_plan(built: &LivePoolFixture, cache: &Path) -> super::route_checks::CloneFacts {
    let common: PathBuf = built.common();
    let slot_path: PathBuf = built.slot_path();
    let record = built.recorded();
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let clone_guid: DatasetGuid = DatasetGuid::parse(built.identity(built.clone().as_str()).0.to_string().as_str()).expect("the clone's GUID");
    let inventory = whole_inventory(&reads, built.clone().as_str(), clone_guid).expect("the clone's inventory");
    let own: Vec<crate::worktree_clone_removal_proof_and_plan::plan_record::PlanSnapshot> = inventory.own_snapshots(record.clone_createtxg);
    assert!(!own.is_empty(), "the fixture gives the clone one snapshot of its own");
    let facts = PlanFacts {
        run: RequestNonce::parse(NONCE).expect("the arm's nonce"),
        owner: String::from("1:linux:1"),
        started: now_with_offset(),
        clone_dataset: record.clone_dataset.clone(),
        clone_guid: record.clone_guid.clone(),
        clone_createtxg: record.clone_createtxg,
        origin_dataset: record.origin_dataset.clone(),
        origin_dataset_guid: record.origin_dataset_guid.clone(),
        origin_snapshot: record.origin_snapshot.clone(),
        origin_snapshot_guid: record.origin_snapshot_guid.clone(),
        origin_snapshot_createtxg: record.origin_snapshot_createtxg,
        registration_id: Vec::from(SLOT.as_bytes()),
        admin: record.registration_identity.encode(),
        target: slot_path.clone(),
        unmount_nonce: RequestNonce::parse(NONCE).expect("the arm's nonce"),
        unmount_mount_id: None,
    };
    let folder: PlanFolder = PlanFolder::open_or_create(common.as_path()).expect("the plan folder");
    let published: PathBuf = folder.publish(&DestructionPlan::first_write(facts, own.as_slice())).expect("the plan is published");
    assert!(published.exists(), "the plan's file stands");
    built.run("zfs", &["set", format!("{DESIRED_STATE_PROPERTY}=v1:{NONCE}:{}", MountOperation::Unmount.word()).as_str(), built.clone().as_str()]);
    return clone_facts(common.as_path(), slot_path.as_path(), cache);
}

/// The plan's line names the head and every name,
/// and the state line names the removal in flight.
fn assert_interrupted(facts: &super::route_checks::CloneFacts) {
    let state: CheckOutcome = check_clone_state(&facts.state);
    let CheckOutcome::Finding { field, detail } = state else {
        panic!("a removal in flight is a state finding: {state:?}");
    };
    assert_eq!(field, "plan");
    assert!(detail.contains("removal in flight"), "{detail}");
    let plan: CheckOutcome = check_removal_plan(&facts.plan);
    let CheckOutcome::Finding { field: plan_field, detail: plan_detail } = plan else {
        panic!("a standing plan is a finding: {plan:?}");
    };
    assert_eq!(plan_field, "plan");
    assert!(plan_detail.contains(OWN_SNAPSHOT), "{plan_detail}");
    assert!(plan_detail.contains("head=present"), "{plan_detail}");
    let blocked: CheckOutcome = check_blocked_removal(&facts.blocked);
    passed(&blocked, "the blocked line with no holder");
}

/// The clone mounted in this namespace is a holder,
/// and the blocked line names it and the caller's session it belongs to.
fn mounts_the_clone(built: &LivePoolFixture, cache: &Path, slot_path: &Path) -> super::route_checks::CloneFacts {
    let clone: String = built.clone();
    let guid: DatasetGuid = DatasetGuid::parse(built.identity(clone.as_str()).0.to_string().as_str()).expect("the clone's GUID");
    let folder: PathBuf = staging_folder(cache, guid);
    std::fs::create_dir_all(folder.as_path()).expect("the staging folder");
    built.run("zfs", &["set", format!("mountpoint={}", folder.display()).as_str(), clone.as_str()]);
    built.run("zfs", &["mount", clone.as_str()]);
    let _ = slot_path;
    return clone_facts(built.common().as_path(), built.slot_path().as_path(), cache);
}

/// The holder is named by its namespace and task,
/// the blocked line names the loss and the caller's own session,
/// and the state line names the staged clone.
fn assert_held(facts: &super::route_checks::CloneFacts) {
    let holders: CheckOutcome = check_holders(&facts.holders);
    let CheckOutcome::Finding { field, detail } = holders else {
        panic!("a mount in this namespace is a holder: {holders:?}");
    };
    assert_eq!(field, "holder");
    assert!(detail.contains(std::process::id().to_string().as_str()), "{detail}");
    let blocked: CheckOutcome = check_blocked_removal(&facts.blocked);
    let CheckOutcome::Finding { field: blocked_field, detail: blocked_detail } = blocked else {
        panic!("a holder blocks the removal: {blocked:?}");
    };
    assert_eq!(blocked_field, "removal");
    assert!(blocked_detail.contains("cancels itself"), "{blocked_detail}");
    assert!(blocked_detail.contains("never ends the caller or its session"), "{blocked_detail}");
    let state: CheckOutcome = check_clone_state(&facts.state);
    let CheckOutcome::Finding { field: state_field, detail: state_detail } = state else {
        panic!("a staged clone is a state finding: {state:?}");
    };
    assert_eq!(state_field, "staging");
    assert!(state_detail.contains("staging folder"), "{state_detail}");
}

/// The clone goes back to its legacy mountpoint,
/// so the snapshot arms read a clone nothing holds.
fn unstages(built: &LivePoolFixture, clone: &str, cache: &Path) {
    built.run("zfs", &["unmount", clone]);
    built.run("zfs", &["set", "mountpoint=legacy", clone]);
    let guid: DatasetGuid = DatasetGuid::parse(built.identity(clone).0.to_string().as_str()).expect("the clone's GUID");
    let folder: PathBuf = staging_folder(cache, guid);
    if folder.exists() {
        std::fs::remove_dir_all(folder.as_path()).expect("the staging folder goes");
    }
}

/// A snapshot the plan recorded as `planned` that the pool lost is a finding.
fn destroys_the_recorded_snapshot(built: &LivePoolFixture, common: &Path, slot_path: &Path, cache: &Path) -> super::route_checks::CloneFacts {
    built.run("zfs", &["destroy", format!("{}@{OWN_SNAPSHOT}", built.clone()).as_str()]);
    return clone_facts(common, slot_path, cache);
}

/// The snapshot line names the lost name,
/// and its GUID.
fn assert_lost(facts: &super::route_checks::CloneFacts) {
    let answer: CheckOutcome = check_clone_snapshots(&facts.snapshots);
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a lost planned snapshot is a finding: {answer:?}");
    };
    assert_eq!(field, "snapshots");
    assert!(detail.contains(OWN_SNAPSHOT), "{detail}");
    assert!(detail.contains("lists no such snapshot"), "{detail}");
}

/// The origin snapshot marked for deferred destroy is a finding naming the mark.
fn marks_the_origin(built: &LivePoolFixture, common: &Path, slot_path: &Path, cache: &Path) -> super::route_checks::CloneFacts {
    built.run("zfs", &["destroy", "-d", built.origin().as_str()]);
    return clone_facts(common, slot_path, cache);
}

/// The mark reads on,
/// the two counts are named,
/// and the line says what a person can do.
fn assert_marked(facts: &super::route_checks::CloneFacts) {
    let answer: CheckOutcome = check_clone_origin(&facts.origin);
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a marked origin is a finding: {answer:?}");
    };
    assert_eq!(field, "defer_destroy");
    assert!(detail.contains("cannot be cleared"), "{detail}");
    assert!(detail.contains("clone(s)"), "{detail}");
    assert!(detail.contains("user hold(s)"), "{detail}");
}

/// A registration directory that is gone is a finding naming the prune.
fn prunes_the_registration(built: &LivePoolFixture, common: &Path, slot_path: &Path, cache: &Path) -> super::route_checks::CloneFacts {
    std::fs::remove_dir_all(built.registration().as_path()).expect("the registration goes");
    return clone_facts(common, slot_path, cache);
}

/// The registration line names the path and the prune,
/// and never passes.
fn assert_pruned(facts: &super::route_checks::CloneFacts) {
    let answer: CheckOutcome = check_registration(&facts.registration);
    let CheckOutcome::Finding { field, detail } = answer else {
        panic!("a pruned registration is a finding: {answer:?}");
    };
    assert_eq!(field, "registration");
    assert!(detail.contains("prune"), "{detail}");
    assert!(detail.contains("cannot rebuild"), "{detail}");
}

/// The retention line prints the named fields the pool reports,
/// one object's own at a time.
fn assert_retained(facts: &super::route_checks::CloneFacts) {
    let answer: CheckOutcome = check_clone_retention(&facts.retention);
    let CheckOutcome::Pass { detail } = answer else {
        panic!("retention is a report: {answer:?}");
    };
    for field in [USED_PROPERTY, REFERENCED_PROPERTY, LOGICAL_REFERENCED_PROPERTY] {
        assert!(detail.contains(&format!("`{field}`")), "{field} is missing from {detail}");
    }
    assert!(detail.contains("sums none of them"), "{detail}");
}

/// A complete registration on the fixture's slot:
/// the registration's `gitdir` names the slot's `.git`,
/// and the slot's `.git` names the registration,
/// as Git writes the pair.
fn write_a_complete_registration(slot_path: &Path, registration: &Path) {
    std::fs::create_dir_all(registration).expect("the registration");
    std::fs::write(registration.join("gitdir"), format!("{}\n", slot_path.join(".git").display())).expect("the registration's gitdir");
    let pointer: Vec<u8> = format!("gitdir: {}\n", registration.display()).into_bytes();
    std::fs::write(slot_path.join(".git"), pointer.as_slice()).expect("the slot's .git");
}

//endregion
