//! What:
//!  The facts a control builds for the ZFS route's `doctor` checks:
//!  one healthy dataset of each role,
//!  one mount table with the checkout on it,
//!  one store listing and one delegation that hold the decided shape.
//! Why:
//!  Every check takes its facts as a value,
//!  so one builder of a healthy route lets each control change exactly the
//!  field it judges and read the one answer that moved.
//!  The values are the design's own:
//!  the checkout dataset mounted at the checkout path,
//!  the clone container beside it with `mountpoint=legacy`,
//!  `canmount=off`,
//!  `setuid=off` and `devices=off`,
//!  and a clone below it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dataset and path layout").
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = routeFacts(); facts.datasets[1].reading.value.mountpoint = { kind: 'text', text: '/' };
//! const report = route_checks(facts).find((entry) => entry.check === 'container-dataset');
//! ```

use super::blocked_removal_checks::BlockedRemovalFacts;
use super::clone_origin_checks::{CloneOriginFacts, RecordedOrigin};
use super::clone_retention_checks::{AccountingFacts, CloneRetentionFacts};
use super::clone_snapshot_checks::CloneSnapshotFacts;
use super::clone_state_checks::CloneStateFacts;
use super::delegation_checks::{DelegationFacts, Grant};
use super::handler_checks::{
    FolderFacts, HandlerConfigurationFacts, HandlerInstallationFacts, HandlerWiringFacts, PathFacts, PathState, ReadinessFacts, RequestFacts, RESULTS_DIRECTORY_MODE, READINESS_DIRECTORY_MODE, ROOT_OWNER,
};
use super::holder_checks::HolderFacts;
use super::identity_checks::{LiveIdentity, ParentIdentityFacts, RecordedIdentity};
use super::managed_lock_checks::ManagedLockFacts;
use super::mount_checks::MountCheckFacts;
use super::registration_checks::RegistrationFacts;
use super::removal_plan_checks::{PlanFileReading, RemovalPlanFacts};
use super::route_checks::{CloneFacts, RouteFacts};
use super::staging_folder_checks::{StagingBind, StagingFolderFacts, staging_binds};
use super::store_listing::{ListedEntry, RecordFacts, SlotRecordFacts, StoreListingFacts};
use super::zfs_checks::{AbiFacts, DatasetFacts, DatasetRole, DeviceFacts, DeviceState, PoolFacts, Reading};
use crate::fs_mount_options::parse_mount_table;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_managed_lock::{Classification, ManagedLockState};
use crate::worktree_clone_mount_protocol::RequestNonce;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{CloneObjectFacts, PropertyValue};
use crate::worktree_clone_mount_protocol::ReadyState;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveInventory;
use crate::worktree_clone_slot_state::slot_state_mounts::{MountRows, SlotMount};
use crate::worktree_clone_slot_state::slot_state_readings::{BindingReading, IntentFacts, PlanReading, SlotReadings};
use crate::worktree_clone_slot_state::slot_state_test_records::record_at;
use crate::worktree_removal_deferred_origin_check::origin_mark::OriginMark;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;
use std::path::PathBuf;

/// The checkout the design names.
pub(crate) const CHECKOUT: &str = "/home/user/Monochromatic";

/// The pool the checkout dataset lives in.
pub(crate) const POOL: &str = "zpcachyos";

/// The checkout dataset.
pub(crate) const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The clone container beside it.
pub(crate) const CONTAINER_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets";

/// One managed clone below the container.
pub(crate) const CLONE_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one";

/// The managed account's user id.
pub(crate) const ACCOUNT: u32 = 1000;

/// The managed account's primary group.
pub(crate) const ACCOUNT_GROUP: u32 = 1000;

/// The boot identifier this run is in.
pub(crate) const LIVE_BOOT: &str = "4c4c4544-0042-3510-8051-b4c04f4e4d32";

/// The staging folder prefix,
/// which a control's names carry.
pub(crate) const STAGING_PREFIX: &str = crate::worktree_clone_unmount::staging_folder_name::STAGING_FOLDER_PREFIX;

/// The add nonce the healthy clone's managed Git lock names,
/// which is step 13's own fixture nonce.
pub(crate) const LOCK_NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The origin snapshot every fixture clone names.
pub(crate) const ORIGIN_SNAPSHOT: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// One healthy dataset of the route's layout,
/// with the fields a control changes.
pub(crate) struct DatasetBuilder {
    /// The dataset's name.
    name: String,
    /// The `mountpoint` property's value.
    mountpoint: String,
    /// The `canmount` index value,
    /// where the role reads it.
    canmount: Option<u64>,
    /// The `setuid` index value.
    setuid: u64,
    /// The `devices` index value.
    devices: u64,
    /// The `encryption` value.
    encryption: String,
    /// The `keystatus` value.
    key_status: u64,
    /// The `snapdir` index value,
    /// where the role reads it.
    snapdir: Option<u64>,
    /// The object-set type.
    object_type: u32,
    /// Whether the kernel called it a snapshot.
    is_snapshot: bool,
}

impl DatasetBuilder {
    /// The design's healthy shape for one dataset.
    pub(crate) fn healthy(name: &str, mountpoint: &str) -> Self {
        return DatasetBuilder {
            name: String::from(name),
            mountpoint: String::from(mountpoint),
            canmount: None,
            setuid: 0,
            devices: 0,
            encryption: String::from("off"),
            key_status: 0,
            snapdir: Some(crate::health::doctor_zfs_route::snapshot_directory_checks::SNAPDIR_DISABLED),
            object_type: super::zfs_checks::FILESYSTEM_OBJECT_SET,
            is_snapshot: false,
        };
    }

    /// A container at the design's shape.
    pub(crate) fn container() -> Self {
        let mut builder: DatasetBuilder = DatasetBuilder::healthy(CONTAINER_DATASET, "legacy");
        builder.canmount = Some(super::zfs_checks::CANMOUNT_OFF);
        return builder;
    }

    /// The checkout at the design's shape.
    pub(crate) fn checkout() -> Self {
        return DatasetBuilder::healthy(CHECKOUT_DATASET, CHECKOUT);
    }

    /// A clone at the design's shape.
    pub(crate) fn clone() -> Self {
        return DatasetBuilder::healthy(CLONE_DATASET, "legacy");
    }

    /// The `mountpoint` this dataset reports.
    pub(crate) fn mountpoint(mut self, mountpoint: &str) -> Self {
        self.mountpoint = String::from(mountpoint);
        return self;
    }

    /// The `setuid` index value.
    pub(crate) fn setuid(mut self, setuid: u64) -> Self {
        self.setuid = setuid;
        return self;
    }

    /// The `devices` index value.
    pub(crate) fn devices(mut self, devices: u64) -> Self {
        self.devices = devices;
        return self;
    }

    /// The `canmount` index value.
    pub(crate) fn canmount(mut self, canmount: u64) -> Self {
        self.canmount = Some(canmount);
        return self;
    }

    /// The `snapdir` index value,
    /// or nothing where the reply leaves the default out.
    pub(crate) fn snapdir(mut self, snapdir: Option<u64>) -> Self {
        self.snapdir = snapdir;
        return self;
    }

    /// An encrypted dataset whose key is loaded,
    /// or not where the caller says so.
    pub(crate) fn encrypted(mut self, key_status: u64) -> Self {
        self.encryption = String::from("aes-256-gcm");
        self.key_status = key_status;
        return self;
    }

    /// The object-set type,
    /// and whether the kernel called it a snapshot.
    pub(crate) fn object_type(mut self, object_type: u32, is_snapshot: bool) -> Self {
        self.object_type = object_type;
        self.is_snapshot = is_snapshot;
        return self;
    }

    /// The kernel's facts,
    /// with the properties the checks read beside the fields.
    pub(crate) fn build(self) -> CloneObjectFacts {
        let mut properties: NvList = NvList::new();
        add_property(&mut properties, "mountpoint", &self.mountpoint);
        if let Some(canmount) = self.canmount {
            add_number_property(&mut properties, "canmount", canmount);
        }
        if let Some(snapdir) = self.snapdir {
            add_number_property(&mut properties, crate::health::doctor_zfs_route::snapshot_directory_checks::SNAPDIR_PROPERTY, snapdir);
        }
        return CloneObjectFacts {
            name: self.name,
            guid: 7,
            object_type: self.object_type,
            is_snapshot: self.is_snapshot,
            origin: String::from("zpcachyos/ROOT/cos/home/monochromatic-git-checkout@git-worktree-clone-origin-1"),
            mountpoint: PropertyValue::Text(self.mountpoint),
            setuid: PropertyValue::Number(self.setuid),
            devices: PropertyValue::Number(self.devices),
            encryption: PropertyValue::Text(self.encryption),
            encryption_root: PropertyValue::Text(String::from("zpcachyos/ROOT/cos/home/monochromatic-git-checkout")),
            key_status: PropertyValue::Number(self.key_status),
            properties,
        };
    }
}

/// One property in the shape OpenZFS returns it.
fn add_property(properties: &mut NvList, name: &str, text: &str) {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", text);
    pair.add_str("source", "local");
    properties.add_nvlist(name, pair);
}

/// One number property in the shape OpenZFS returns it.
fn add_number_property(properties: &mut NvList, name: &str, number: u64) {
    let mut pair: NvList = NvList::new();
    pair.add_u64("value", number);
    pair.add_str("source", "local");
    properties.add_nvlist(name, pair);
}

/// One dataset of the route with its reading.
pub(crate) fn dataset(role: DatasetRole, facts: CloneObjectFacts) -> DatasetFacts {
    return DatasetFacts { role, name: facts.name.clone(), reading: Reading::Read(facts) };
}

/// The three datasets of the design's healthy route.
pub(crate) fn healthy_datasets() -> Vec<DatasetFacts> {
    return vec![
        dataset(DatasetRole::Checkout { mount_point: PathBuf::from(CHECKOUT) }, DatasetBuilder::checkout().build()),
        dataset(DatasetRole::Container, DatasetBuilder::container().build()),
        dataset(DatasetRole::Clone, DatasetBuilder::clone().build()),
    ];
}

/// One mountinfo row in the kernel's grammar.
pub(crate) fn mount_row(id: u32, parent: u32, root: &str, mount_point: &str, filesystem: &str, source: &str, optional: &str) -> String {
    let optional_fields: String = if optional.is_empty() { String::new() } else { format!("{optional} ") };
    return format!("{id} {parent} 0:42 {root} {mount_point} rw,relatime {optional_fields}- {filesystem} {source} rw,xattr,posixacl");
}

/// A mount table with the checkout mounted at its path and the container
/// unmounted.
pub(crate) fn healthy_mount_text() -> String {
    return [
        mount_row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root", ""),
        mount_row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET, ""),
        mount_row(41, 1, "/", "/run", "tmpfs", "tmpfs", ""),
        mount_row(42, 1, "/", "/home/user/.cache", "zfs", "zpcachyos/nosnap/user-cache", "shared:7"),
    ]
    .join("\n");
}

/// The mount facts of the healthy table.
pub(crate) fn healthy_mounts() -> MountCheckFacts {
    return MountCheckFacts { checkout_path: PathBuf::from(CHECKOUT), checkout_dataset: String::from(CHECKOUT_DATASET), rows: parse_mount_table(healthy_mount_text().as_bytes()) };
}

/// A store folder holding `.parent` and one slot record,
/// and nothing else.
pub(crate) fn healthy_store() -> StoreListingFacts {
    return StoreListingFacts {
        entries: vec![ListedEntry { name: String::from(".parent"), modified_seconds_ago: Some(90) }, ListedEntry { name: String::from("one"), modified_seconds_ago: Some(60) }],
        parent: RecordFacts::Read { recorded: String::from(".parent") },
        slots: vec![SlotRecordFacts { name: String::from("one"), facts: RecordFacts::Read { recorded: String::from("one") } }],
    };
}

/// The parent identity a store records and the live folder answers.
pub(crate) fn healthy_parent_identity() -> ParentIdentityFacts {
    let identity: crate::worktree_clone_mount::CloneFolderIdentity = folder_identity(7);
    return ParentIdentityFacts { recorded: RecordedIdentity::Read(Box::new(identity.clone())), live: LiveIdentity::Held(Box::new(identity)) };
}

/// One folder identity,
/// with a device and inode a control can tell apart.
pub(crate) fn folder_identity(inode: u64) -> crate::worktree_clone_mount::CloneFolderIdentity {
    return crate::worktree_clone_mount::CloneFolderIdentity { device: 2049, inode, owner: ACCOUNT, mode: 0o700, access_acl: None, default_acl: None };
}

/// One grant in the kernel's model.
pub(crate) fn grant(carried_by: &str, permission: &str, who: super::delegation_checks::Who, inheritance: super::delegation_checks::Inheritance) -> Grant {
    return Grant { carried_by: String::from(carried_by), permission: String::from(permission), who, inheritance };
}

/// The nine decided grants,
/// each where the design puts it.
pub(crate) fn healthy_grants() -> Vec<Grant> {
    use super::delegation_checks::{Inheritance, Who};
    let user: Who = Who::User(ACCOUNT);
    return vec![
        grant(CHECKOUT_DATASET, "snapshot", user.clone(), Inheritance::Local),
        grant(CHECKOUT_DATASET, "clone", user.clone(), Inheritance::Descendants),
        grant(CHECKOUT_DATASET, "mount", user.clone(), Inheritance::Descendants),
        grant(CHECKOUT_DATASET, "destroy", user.clone(), Inheritance::Descendants),
        grant(CONTAINER_DATASET, "create", user.clone(), Inheritance::Local),
        grant(CONTAINER_DATASET, "mount", user.clone(), Inheritance::Local),
        grant(CONTAINER_DATASET, "mount", user.clone(), Inheritance::Descendants),
        grant(CONTAINER_DATASET, "userprop", user.clone(), Inheritance::Descendants),
        grant(CONTAINER_DATASET, "destroy", user, Inheritance::Descendants),
    ];
}

/// The delegation facts of the healthy route.
pub(crate) fn healthy_delegation() -> DelegationFacts {
    return DelegationFacts { account: ACCOUNT, groups: vec![ACCOUNT_GROUP], checkout_dataset: String::from(CHECKOUT_DATASET), container_dataset: String::from(CONTAINER_DATASET), grants: healthy_grants() };
}

/// One path the route installs or reads,
/// at the state a healthy route shows.
pub(crate) fn path_facts(path: &str, state: PathState) -> PathFacts {
    return PathFacts { path: PathBuf::from(path), state };
}

/// A root-owned executable file.
pub(crate) fn root_file(path: &str) -> PathFacts {
    return path_facts(path, PathState::File { owner: ROOT_OWNER, mode: 0o755 });
}

/// A root-owned directory at the given mode.
pub(crate) fn root_directory(path: &str, mode: u32) -> PathFacts {
    return path_facts(path, PathState::Directory { owner: ROOT_OWNER, mode });
}

/// The handler's two installed halves at the build's own paths.
pub(crate) fn healthy_installation() -> HandlerInstallationFacts {
    return HandlerInstallationFacts { executable: root_file(super::handler_checks::ROOT_MODE_EXECUTABLE_PATH), zedlet: root_file(super::handler_checks::ZEDLET_EVENT_PATH) };
}

/// The handler's configuration at the build's own path.
pub(crate) fn healthy_configuration() -> HandlerConfigurationFacts {
    return HandlerConfigurationFacts { file: path_facts(crate::worktree_clone_mount_handler::CONFIGURATION_PATH, PathState::File { owner: ROOT_OWNER, mode: 0o644 }), parse_error: None };
}

/// The readiness and results folders at the design's modes.
pub(crate) fn healthy_folders() -> FolderFacts {
    return FolderFacts {
        readiness: root_directory(crate::worktree_clone_mount_protocol::READINESS_DIRECTORY, READINESS_DIRECTORY_MODE),
        results: root_directory(crate::worktree_clone_mount_protocol::RESULTS_DIRECTORY, RESULTS_DIRECTORY_MODE),
    };
}

/// ZED installed and enabled,
/// with its own other zedlet beside the route's.
pub(crate) fn healthy_wiring() -> HandlerWiringFacts {
    return HandlerWiringFacts {
        zed_unit: root_file("/usr/lib/systemd/system/zfs-zed.service"),
        zed_enabled: true,
        other_zedlets: vec![String::from("history_event-zfs-list-cacher.sh")],
    };
}

/// This boot's readiness record with every prerequisite ready.
pub(crate) fn healthy_readiness() -> ReadinessFacts {
    return ReadinessFacts::Read { boot: String::from(LIVE_BOOT), pool: ReadyState::Ready, checkout: ReadyState::Ready, sweep: String::from("finished") };
}

/// One clone's request answered by a receipt of this boot.
pub(crate) fn healthy_request() -> RequestFacts {
    return RequestFacts { clone: String::from(CLONE_DATASET), intent_nonce: Some(String::from("0123456789abcdef0123456789abcdef")), receipt_nonce: Some(String::from("0123456789abcdef0123456789abcdef")), receipt_boot: Some(String::from(LIVE_BOOT)) };
}

/// The cache with no staging folder under it.
pub(crate) fn healthy_staging() -> StagingFolderFacts {
    return StagingFolderFacts { cache: root_directory("/home/user/.cache", 0o700), account: ACCOUNT, folders: Vec::new() };
}

/// The healthy route's whole fact set,
/// which every control starts from.
pub(crate) fn healthy_route_facts() -> RouteFacts {
    let mounts: MountCheckFacts = healthy_mounts();
    let staging_binds_value: Vec<StagingBind> = staging_binds(&mounts.rows, mounts.checkout_path.as_path());
    let cache: String = String::from("/home/user/.cache");
    return RouteFacts {
        device: DeviceFacts { path: PathBuf::from(monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH), state: DeviceState::Opens },
        abi: AbiFacts::Compatible { module: Some(String::from("2.4.4-1")), details: vec![String::from("base type information 1234 bytes")] },
        pool: PoolFacts { pool: String::from(POOL), reading: Reading::Read(9) },
        datasets: healthy_datasets(),
        mounts,
        parent_identity: healthy_parent_identity(),
        slots: Vec::new(),
        store: healthy_store(),
        delegation: healthy_delegation(),
        installation: healthy_installation(),
        configuration: healthy_configuration(),
        folders: healthy_folders(),
        wiring: healthy_wiring(),
        readiness: healthy_readiness(),
        live_boot: String::from(LIVE_BOOT),
        requests: vec![healthy_request()],
        staging: StagingFolderFacts { cache: path_facts(cache.as_str(), PathState::Directory { owner: ACCOUNT, mode: 0o700 }), account: ACCOUNT, folders: Vec::new() },
        staging_binds: staging_binds_value,
        clones: vec![healthy_clone()],
    };
}

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
pub(crate) fn accounting(object: &str, used: u64) -> AccountingFacts {
    return AccountingFacts { object: String::from(object), used: PropertyValue::Number(used), referenced: PropertyValue::Number(0), logical_referenced: PropertyValue::Number(used), written: PropertyValue::Number(used) };
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
fn healthy_classification(registration: &PathBuf) -> Classification {
    return Classification {
        state: ManagedLockState::Managed { nonce: RequestNonce::parse(LOCK_NONCE).expect("the fixture nonce reads"), registration: registration.clone() },
        pointer: crate::worktree_clone_managed_lock::Reading::Bytes(Vec::from(format!("gitdir: {}\n", registration.display()).into_bytes())),
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
        mounts: MountRows::Rows(vec![MountedDatasetInstance { dataset: String::from(CLONE_DATASET), mount_point: slot_path.clone(), root: PathBuf::from("/") }]),
        slot_mount: SlotMount::Something { described: String::from("a zfs mount of the clone dataset") },
        registration_stands: Some(true),
    };
    let holders: HolderFacts = HolderFacts { clone: String::from(CLONE_DATASET), holders: Vec::new(), incomplete: Vec::new() };
    let lock: ManagedLockFacts = ManagedLockFacts { slot: String::from(CLONE_SLOT), classification: healthy_classification(&registration) };
    return CloneFacts {
        state: CloneStateFacts { readings, staging_binds: Vec::new() },
        registration: RegistrationFacts {
            slot: String::from(CLONE_SLOT),
            recorded: Some(registration),
            at_the_path: PathState::Directory { owner: ACCOUNT, mode: 0o755 },
            gitdir: Some(slot_path.join(".git")),
            slot_git: slot_path.join(".git"),
        },
        lock,
        plan: RemovalPlanFacts { clone: String::from(CLONE_DATASET), reading: PlanFileReading::NoPlan },
        snapshots: CloneSnapshotFacts {
            clone: String::from(CLONE_DATASET),
            clone_createtxg: 121_294,
            live: Reading::Read(LiveInventory { dataset: String::from(CLONE_DATASET), dataset_guid: DatasetGuid::parse("7").expect("the fixture GUID reads"), objects: Vec::new() }),
            plan_stands: false,
            recorded: Vec::new(),
        },
        origin: CloneOriginFacts {
            clone: String::from(CLONE_DATASET),
            recorded: Some(RecordedOrigin { dataset: String::from(CHECKOUT_DATASET), snapshot: String::from(ORIGIN_SNAPSHOT) }),
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
