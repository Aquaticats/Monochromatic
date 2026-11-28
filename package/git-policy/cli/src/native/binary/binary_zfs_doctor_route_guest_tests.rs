//! What:
//!  Plan step 18's `doctor` checks for steps 2 to 9,
//! run against a real
//!  file-backed pool in a throwaway guest:
//!  the three groups pass on the design's layout,
//!  and each broken fact answers the finding that names it.
//! Why:
//!  The device,
//!  the ABI gate,
//!  a dataset's properties,
//!  its encryption key and the kernel's delegation reply exist only on a
//!  machine with OpenZFS,
//!  and a mount row that hangs from the tree the checkout replaced exists only
//!  where a mount can be made,
//!  so no host control may show them
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "VM measurements";
//!  `package/git-policy/cli/e2e/doctor-zfs-route-guest.ts`).
//!  The arm prints `SKIP` where `CLI_GIT_ZFS_DOCTOR_GUEST` does not name the
//!  guest,
//!  and it creates everything it reads:
//!  the pool,
//!  the design's datasets,
//!  the store,
//!  the installed paths,
//!  the records and the staging bind,
//!  so a run leaves the guest as it found it.
//!  Every claim is a marker line the driver requires,
//!  so a section that silently stopped running fails the run instead of
//!  printing less.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (process.env.CLI_GIT_ZFS_DOCTOR_GUEST !== guest) return print('SKIP');
//! installLayout(); expect(routeChecks(gather()).every(pass)).toBe(true);
//! ```

use git_policy_cli::doctor_zfs_route::{
    AbiFacts,
    CheckOutcome,
    CheckReport,
    DatasetFacts,
    DatasetRole,
    DelegationFacts,
    DeviceFacts,
    DeviceState,
    LiveIdentity,
    MountCheckFacts,
    ParentIdentityFacts,
    PathState,
    PoolFacts,
    ReadinessFacts,
    Reading,
    RecordFacts,
    RecordedIdentity,
    RequestFacts,
    SlotIdentityFacts,
    SlotRecordFacts,
    StagedFolder,
    StagingFolderFacts,
    StagingRemovalRequest,
    StoreListingFacts,
    check_clone_properties,
    check_delegation,
    check_key,
    check_snapdir,
    check_unknown_dot_entries,
    grants_of,
    remove_known_staging_forms,
    route_checks,
    snapdir_setting,
    staging_binds,
};
use git_policy_cli::fs_mount_options::parse_mount_table;
use git_policy_cli::worktree_clone_mount::{
    CloneFolderIdentity,
    CloneSlotRefusal,
    HoldCloneSlotRequest,
    PARENT_RECORD_NAME,
    STORE_HEADER,
    SlotIdentityStore,
    StoreRefusal,
    hold_clone_slot_under_root_control,
    hold_the_worktrees_parent,
};
use git_policy_cli::worktree_clone_mount_handler::CONFIGURATION_PATH;
use git_policy_cli::worktree_clone_mount_handler::sweep_configuration::read_configuration;
use git_policy_cli::worktree_clone_mount_handler::sweep_dataset_facts::facts_of;
use git_policy_cli::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use git_policy_cli::worktree_clone_mount_protocol::{
    CloneMountReceipt,
    DESIRED_STATE_PROPERTY,
    DatasetGuid,
    IntentReading,
    MountOperation,
    READINESS_DIRECTORY,
    RESULTS_DIRECTORY,
    ReadinessFile,
    ReadinessRecord,
    ReadyState,
    ReceiptMountIdentity,
    ReceiptOutcome,
    ReceiptsFolder,
    RequestNonce,
    SweepState,
    read_intent,
};
use git_policy_cli::zfs_compatibility_gate::{
    BeforeRequests,
    LiveGate,
};
use git_policy_cli::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    CompatibilityStep,
    DEVICE_PATH,
    DatasetIdentity,
    ZfsDevice,
};
use std::os::unix::fs::{
    MetadataExt as _,
    PermissionsExt as _,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

/// The variable the guest driver sets.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_DOCTOR_GUEST";

/// The variable the guest driver sets for the pool's name.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_DOCTOR_POOL";

/// Where the arm builds everything it reads.
const FIXTURE: &str = "/home/zfs-doctor-fixture";

/// The checkout the design's dataset mounts at.
const CHECKOUT: &str = "/home/zfs-doctor-fixture/checkout";

/// The account the delegation names,
/// which the guest image creates.
const ACCOUNT: u32 = 1000;

/// The pool's image size.
const POOL_IMAGE_SIZE: &str = "1G";

/// The one intent nonce this arm writes.
const NONCE: &str = "0123456789abcdef0123456789abcdef";

/// The install path the checks name as a placeholder for the one executable.
const EXECUTABLE_PATH: &str = git_policy_cli::doctor_zfs_route::ROOT_MODE_EXECUTABLE_PATH;

/// The ZED event path the checks name as a placeholder.
const ZEDLET_PATH: &str = git_policy_cli::doctor_zfs_route::ZEDLET_EVENT_PATH;

/// The stale name a readiness record from another boot is built with.
const OTHER_BOOT: &str = "00000000-0000-0000-0000-000000000000";

//region Commands

/// Run one program,
/// refusing unless it succeeded,
/// and print what it said.
fn run(
    program: &str,
    arguments: &[&str],
) {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("{program} could not be started: {error}"));
    if !output.status.success() {
        panic!(
            "{program} {} exited {:?}\n{}\n{}",
            arguments.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Run one `zfs` command,
/// which the driver's PATH puts on the guest's.
fn zfs(arguments: &[&str]) {
    run("zfs", arguments);
}

/// The pool of this run.
fn pool() -> String {
    return std::env::var(POOL_VARIABLE)
        .unwrap_or_else(|_error| panic!("{POOL_VARIABLE} is not set"));
}

/// The checkout dataset of the design's layout.
fn checkout_dataset() -> String {
    return format!("{}/home/monochromatic-git-checkout", pool());
}

/// The clone container of the design's layout.
fn container_dataset() -> String {
    return format!("{}/home/monochromatic-git-worktree-clone-datasets", pool());
}

/// The one clone this arm manages.
fn clone_dataset() -> String {
    return format!("{}/one", container_dataset());
}

/// The store this arm writes and reads.
fn store_path() -> &'static str {
    return git_policy_cli::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH;
}

/// The staging folder's name for this arm's clone GUID.
fn stage_name() -> String {
    return git_policy_cli::worktree_clone_unmount::staging_folder_name::staging_folder_name(
        DatasetGuid::parse("1837462836473829").expect("a canonical GUID"),
    );
}

/// One path's state as the checks read it.
fn path_state(path: &Path) -> PathState {
    return match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => PathState::Absent,
        Err(error) => PathState::Unreadable {
            why: error.to_string(),
        },
        Ok(metadata) if metadata.is_dir() => PathState::Directory {
            owner: metadata.uid(),
            mode: metadata.permissions().mode() & 0o7777,
        },
        Ok(metadata) if metadata.is_file() => PathState::File {
            owner: metadata.uid(),
            mode: metadata.permissions().mode() & 0o7777,
        },
        Ok(_) => PathState::WrongKind {
            found: String::from("a link or a device node"),
        },
    };
}

//endregion

//region The fixture's layout

/// What the arm needs in place before it reads anything.
fn install_layout() {
    let pool_name: String = pool();
    clear_what_a_killed_arm_left();
    std::fs::create_dir_all(FIXTURE).expect("the fixture root");
    let image: String = format!("{FIXTURE}/pool.img");
    run("truncate", &["--size", POOL_IMAGE_SIZE, image.as_str()]);
    run(
        "zpool",
        &[
            "create",
            "-f",
            "-o",
            "cachefile=none",
            "-O",
            "mountpoint=none",
            pool_name.as_str(),
            image.as_str(),
        ],
    );
    zfs(&[
        "create",
        "-o",
        "mountpoint=none",
        format!("{pool_name}/home").as_str(),
    ]);
    zfs(&[
        "create",
        "-o",
        format!("mountpoint={CHECKOUT}").as_str(),
        "-o",
        "snapdir=disabled",
        checkout_dataset().as_str(),
    ]);
    zfs(&[
        "create",
        "-o",
        "mountpoint=legacy",
        "-o",
        "canmount=off",
        "-o",
        "setuid=off",
        "-o",
        "devices=off",
        "-o",
        "snapdir=disabled",
        container_dataset().as_str(),
    ]);
    zfs(&[
        "snapshot",
        format!("{}@origin", checkout_dataset()).as_str(),
    ]);
    zfs(&[
        "clone",
        "-o",
        "mountpoint=legacy",
        format!("{}@origin", checkout_dataset()).as_str(),
        clone_dataset().as_str(),
    ]);
    // The account's folders below the checkout, which the store records and the walk holds.
    for folder in [
        format!("{CHECKOUT}/.claude"),
        format!("{CHECKOUT}/.claude/worktrees"),
        format!("{CHECKOUT}/.claude/worktrees/one"),
    ] {
        std::fs::create_dir_all(folder.as_str()).expect("the account's folder");
        run(
            "chown",
            &[format!("{ACCOUNT}:{ACCOUNT}").as_str(), folder.as_str()],
        );
        std::fs::set_permissions(folder.as_str(), std::fs::Permissions::from_mode(0o700))
            .expect("the account's mode");
    }
    // The design's route needs a checkout no other namespace receives, and the
    // boot unit sets it; a mount made under the guest's shared root is shared
    // until this runs.
    run("mount", &["--make-rprivate", CHECKOUT]);
    record_the_store();
    install_the_handlers();
    enable_zed();
    publish_the_records();
    make_the_staging_folder();
    allow_the_decided_grants();
}

/// Destroy every pool and mount a killed arm left in this throwaway guest,
/// so a rerun starts from nothing.
///
/// A `zpool destroy` that refuses leaves the pool's image belonging to a live
/// pool,
/// and the next `zpool create` on that image then fails with "part of
/// active pool":
/// the refused destroy is what an earlier arm's killed run left,
/// and this guest has no dataset worth keeping.
fn clear_what_a_killed_arm_left() {
    // A killed arm leaves its staging bind mounted, and `remove_dir_all` cannot remove a folder a mount holds, so every
    // mount under the fixture root is unmounted first, deepest first.
    let mut under: Vec<PathBuf> = Vec::new();
    for line in std::fs::read_to_string("/proc/self/mountinfo")
        .expect("the mount table")
        .lines()
    {
        let mut fields = line.split(' ');
        let _id = fields.next();
        let _parent = fields.next();
        let _device = fields.next();
        let _root = fields.next();
        if let Some(point) = fields.next() {
            let path: PathBuf = PathBuf::from(point.replace("\\040", " "));
            if path.starts_with(FIXTURE) {
                under.push(path);
            }
        }
    }
    under.sort();
    under.reverse();
    for point in &under {
        let _ = Command::new("umount").arg(point).output();
    }
    // `zpool` takes no long options: `-f` is the only spelling of force it has.
    let listed = Command::new("zpool")
        .args(["list", "-H", "-o", "name"])
        .output()
        .expect("zpool lists");
    for name in String::from_utf8_lossy(listed.stdout.as_slice()).split_whitespace() {
        let _ = Command::new("zfs").args(["unmount", "-a"]).output();
        let destroyed = Command::new("zpool")
            .args(["destroy", "-f", name])
            .output()
            .expect("zpool answers");
        assert!(
            destroyed.status.success(),
            "the pool {name} could not be destroyed: {}{}",
            String::from_utf8_lossy(destroyed.stdout.as_slice()),
            String::from_utf8_lossy(destroyed.stderr.as_slice())
        );
    }
    let _ = std::fs::remove_dir_all(FIXTURE);
}

/// The parent's and the slot's records,
/// captured from the live folders so the identity compare passes.
fn record_the_store() {
    std::fs::create_dir_all(store_path()).expect("the store folder");
    std::fs::set_permissions(store_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the store's mode");
    let store: SlotIdentityStore =
        SlotIdentityStore::open_under_root_control(Path::new(store_path()))
            .expect("the store opens under root control");
    let _ = store;
    let parent: CloneFolderIdentity = capture(Path::new(&format!("{CHECKOUT}/.claude/worktrees")));
    let slot: CloneFolderIdentity =
        capture(Path::new(&format!("{CHECKOUT}/.claude/worktrees/one")));
    std::fs::write(
        Path::new(store_path()).join(PARENT_RECORD_NAME),
        format!("{STORE_HEADER}\nparent - {}\n", parent.encode()),
    )
    .expect("the parent record");
    std::fs::write(
        Path::new(store_path()).join("one"),
        format!("{STORE_HEADER}\nslot one {}\n", slot.encode()),
    )
    .expect("the slot record");
}

/// One folder's identity,
/// read through an open descriptor as the route reads it.
fn capture(path: &Path) -> CloneFolderIdentity {
    let folder: std::fs::File = std::fs::File::open(path).expect("the folder opens");
    return CloneFolderIdentity::capture(&folder).expect("the folder's identity");
}

/// The one executable's placeholder path,
/// the ZED event's placeholder path and the configuration,
/// with root's ownership and the modes the checks fix.
fn install_the_handlers() {
    let built: String = format!(
        "{}/target/release/monochromatic-git-policy-cli",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::create_dir_all(
        Path::new(EXECUTABLE_PATH)
            .parent()
            .expect("the install folder"),
    )
    .expect("the install folder");
    std::fs::copy(built.as_str(), EXECUTABLE_PATH).expect("the one executable");
    std::fs::set_permissions(EXECUTABLE_PATH, std::fs::Permissions::from_mode(0o755))
        .expect("the executable's mode");
    std::fs::create_dir_all(Path::new(ZEDLET_PATH).parent().expect("the zed.d folder"))
        .expect("the zed.d folder");
    let shim: String = format!("#!/bin/sh\nexec {EXECUTABLE_PATH}\n");
    std::fs::write(ZEDLET_PATH, shim).expect("the zedlet");
    std::fs::set_permissions(ZEDLET_PATH, std::fs::Permissions::from_mode(0o755))
        .expect("the zedlet's mode");
    std::fs::create_dir_all(
        Path::new(CONFIGURATION_PATH)
            .parent()
            .expect("the configuration folder"),
    )
    .expect("the configuration folder");
    let document: String = format!(
        "{{\n  \"cloneMounts\": {{\n    \"checkout\": \"{CHECKOUT}\",\n    \"parent\": \"{}\",\n    \"checkoutDataset\": \"{}\"\n  }}\n}}\n",
        container_dataset(),
        checkout_dataset()
    );
    std::fs::write(CONFIGURATION_PATH, document).expect("the configuration");
    std::fs::set_permissions(CONFIGURATION_PATH, std::fs::Permissions::from_mode(0o644))
        .expect("the configuration's mode");
}

/// The ZED unit the guest's OpenZFS install wrote,
/// which a source build puts in the prefix and a package in one of the two
/// system folders,
/// linked into the wants folder so the route's wiring is enabled as an install
/// would enable it.
fn enable_zed() {
    let found: Option<&str> = [
        "/lib/systemd/system/zfs-zed.service",
        "/usr/lib/systemd/system/zfs-zed.service",
        "/opt/openzfs-2.4.4/lib/systemd/system/zfs-zed.service",
    ]
    .into_iter()
    .find(|unit| return Path::new(unit).exists());
    let Some(unit) = found else {
        panic!(
            "the guest holds no zfs-zed.service; the recipe's OpenZFS install should have written one"
        );
    };
    let wants: &str = "/etc/systemd/system/multi-user.target.wants";
    std::fs::create_dir_all(wants).expect("the wants folder");
    let link: String = format!("{wants}/zfs-zed.service");
    if !Path::new(link.as_str()).exists() {
        std::os::unix::fs::symlink(unit, link.as_str()).expect("the wants link");
    }
}

/// This boot's readiness record and the clone's receipt,
/// each written through the release writers so the grammar is theirs.
fn publish_the_records() {
    let boot: String = live_boot();
    let record: ReadinessRecord = ReadinessRecord {
        boot: boot.clone(),
        pool_name: pool(),
        pool_state: ReadyState::Ready,
        checkout_path: String::from(CHECKOUT),
        checkout_state: ReadyState::Ready,
        sweep_nonce: None,
        sweep_state: SweepState::Finished,
        time: String::from("2026-11-23T00:00:00+00:00"),
    };
    ReadinessFile::at(PathBuf::from(READINESS_DIRECTORY), 0)
        .publish(&record)
        .expect("this boot's readiness record");
    let guid: u64 = identity(clone_dataset().as_str()).stats.dds_guid;
    let receipt: CloneMountReceipt = CloneMountReceipt {
        dataset: DatasetGuid::parse(guid.to_string().as_str()).expect("the clone's GUID"),
        operation: MountOperation::Mount,
        nonce: RequestNonce::parse(NONCE).expect("the arm's nonce"),
        outcome: ReceiptOutcome::Done,
        mount: Some(ReceiptMountIdentity {
            mount_id: 1,
            device: 2049,
            inode: 42,
            source: clone_dataset(),
            canonical_path: format!("{CHECKOUT}/.claude/worktrees/one"),
        }),
        failed_step: None,
        errno: None,
        boot,
        time: String::from("2026-11-23T00:00:00+00:00"),
    };
    ReceiptsFolder::at(PathBuf::from(RESULTS_DIRECTORY), 0)
        .publish(&receipt)
        .expect("the clone's receipt");
    zfs(&[
        "set",
        format!("{DESIRED_STATE_PROPERTY}=v1:{NONCE}:mount").as_str(),
        clone_dataset().as_str(),
    ]);
}

/// The cache,
/// the staging folder and its private bind.
fn make_the_staging_folder() {
    let cache: String = format!("{FIXTURE}/cache");
    std::fs::create_dir_all(cache.as_str()).expect("the cache");
    run(
        "chown",
        &[format!("{ACCOUNT}:{ACCOUNT}").as_str(), cache.as_str()],
    );
    std::fs::set_permissions(cache.as_str(), std::fs::Permissions::from_mode(0o700))
        .expect("the cache mode");
    let stage: String = format!("{cache}/{}", stage_name());
    std::fs::create_dir(stage.as_str()).expect("the staging folder");
    run(
        "chown",
        &[format!("{ACCOUNT}:{ACCOUNT}").as_str(), stage.as_str()],
    );
    std::fs::set_permissions(stage.as_str(), std::fs::Permissions::from_mode(0o700))
        .expect("the staging folder's mode");
    run("mount", &["--bind", stage.as_str(), stage.as_str()]);
    run("mount", &["--make-private", stage.as_str()]);
}

/// The nine decided grants,
/// given with the one executable's own `zfs` userspace.
fn allow_the_decided_grants() {
    let account: String = ACCOUNT.to_string();
    let checkout: String = checkout_dataset();
    let container: String = container_dataset();
    for arguments in [
        vec![
            "allow",
            "-l",
            "-u",
            account.as_str(),
            "snapshot",
            checkout.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            account.as_str(),
            "clone",
            checkout.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            account.as_str(),
            "mount",
            checkout.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            account.as_str(),
            "destroy",
            checkout.as_str(),
        ],
        vec![
            "allow",
            "-l",
            "-u",
            account.as_str(),
            "create",
            container.as_str(),
        ],
        vec![
            "allow",
            "-l",
            "-u",
            account.as_str(),
            "mount",
            container.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            account.as_str(),
            "mount",
            container.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            account.as_str(),
            "userprop",
            container.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            account.as_str(),
            "destroy",
            container.as_str(),
        ],
    ] {
        zfs(arguments.as_slice());
    }
}

/// This boot's identifier.
fn live_boot() -> String {
    return std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .expect("the boot id")
        .trim()
        .to_string();
}

//endregion

//region The facts the checks read

/// One dataset's identity,
/// or the read's failure.
fn identity(dataset: &str) -> DatasetIdentity {
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    return device
        .objset_stats(dataset)
        .unwrap_or_else(|error| panic!("{dataset} could not be read: {error}"));
}

/// One dataset's reading,
/// classified by the kernel's own answer.
fn reading(
    dataset: &str
) -> Reading<git_policy_cli::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts> {
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let read: Result<DatasetIdentity, ReadFailure> =
        device.objset_stats(dataset).map_err(|error| {
            return ReadFailure {
                said: error.to_string(),
                errno: error.errno(),
            };
        });
    return match git_policy_cli::doctor_zfs_route::reading_of(read) {
        Reading::Read(held) => Reading::Read(facts_of(dataset, &held)),
        Reading::Missing {
            why,
        } => Reading::Missing {
            why,
        },
        Reading::Unreadable {
            why,
        } => Reading::Unreadable {
            why,
        },
    };
}

/// The device's facts.
fn device_facts() -> DeviceFacts {
    let path: PathBuf = PathBuf::from(DEVICE_PATH);
    let state: DeviceState = match std::fs::metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => DeviceState::Absent,
        Err(error) => DeviceState::Unreadable {
            why: error.to_string(),
        },
        Ok(_) => match ZfsDevice::open() {
            Ok(_) => DeviceState::Opens,
            Err(error) => DeviceState::Denied {
                why: error.to_string(),
            },
        },
    };
    return DeviceFacts {
        path,
        state,
    };
}

/// The ABI gate's answer in the checks' three states.
fn abi_facts() -> AbiFacts {
    return match LiveGate.verdict() {
        git_policy_cli::zfs_compatibility_gate::Verdict::Compatible(facts) => {
            AbiFacts::Compatible {
                module: facts.version.clone(),
                details: vec![
                    format!("base type information {} bytes", facts.base_btf_bytes),
                    format!("module type information {} bytes", facts.module_btf_bytes),
                ],
            }
        }
        git_policy_cli::zfs_compatibility_gate::Verdict::Refused(refusal) => {
            if refusal.step == CompatibilityStep::Device {
                AbiFacts::Unreadable {
                    why: refusal.to_string(),
                }
            } else {
                AbiFacts::Refused {
                    words: refusal.to_string(),
                }
            }
        }
    };
}

/// The pool's facts.
fn pool_facts() -> PoolFacts {
    let name: String = pool();
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let reading: Reading<u64> = match git_policy_cli::doctor_zfs_route::reading_of(
        device.objset_stats(name.as_str()).map_err(|error| {
            return ReadFailure {
                said: error.to_string(),
                errno: error.errno(),
            };
        }),
    ) {
        Reading::Read(held) => Reading::Read(held.stats.dds_guid),
        Reading::Missing {
            why,
        } => Reading::Missing {
            why,
        },
        Reading::Unreadable {
            why,
        } => Reading::Unreadable {
            why,
        },
    };
    return PoolFacts {
        pool: name,
        reading,
    };
}

/// The checkout,
/// the container and every clone below it.
fn dataset_facts() -> Vec<DatasetFacts> {
    let mut datasets: Vec<DatasetFacts> = vec![
        DatasetFacts {
            role: DatasetRole::Checkout {
                mount_point: PathBuf::from(CHECKOUT),
            },
            name: checkout_dataset(),
            reading: reading(checkout_dataset().as_str()),
        },
        DatasetFacts {
            role: DatasetRole::Container,
            name: container_dataset(),
            reading: reading(container_dataset().as_str()),
        },
    ];
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let mut cursor: u64 = 0;
    while let Some(listed) = device
        .dataset_list_next(container_dataset().as_str(), cursor)
        .expect("the container's children")
    {
        cursor = listed.cursor;
        let name: String = listed.name;
        if name.contains('@') {
            continue;
        }
        datasets.push(DatasetFacts {
            role: DatasetRole::Clone,
            name: name.clone(),
            reading: reading(name.as_str()),
        });
    }
    return datasets;
}

/// The caller's mount table.
fn mount_facts() -> MountCheckFacts {
    let bytes: Vec<u8> = std::fs::read("/proc/self/mountinfo").expect("the mount table");
    return MountCheckFacts {
        checkout_path: PathBuf::from(CHECKOUT),
        checkout_dataset: checkout_dataset(),
        rows: parse_mount_table(bytes.as_slice()),
    };
}

/// The recorded and live parent identities.
fn parent_facts() -> ParentIdentityFacts {
    let store: SlotIdentityStore =
        SlotIdentityStore::open_under_root_control(Path::new(store_path()))
            .expect("the store opens under root control");
    let recorded: RecordedIdentity = match store.parent() {
        Ok(held) => RecordedIdentity::Read(Box::new(held)),
        Err(StoreRefusal::NoParentRecord {
            ..
        }) => RecordedIdentity::Absent {
            why: String::from("no parent record is filed"),
        },
        Err(refusal) => RecordedIdentity::Unreadable {
            why: refusal.message(),
        },
    };
    let live: LiveIdentity = match hold_the_worktrees_parent(Path::new(CHECKOUT)) {
        Ok(held) => LiveIdentity::Held(Box::new(held.identity)),
        Err(refusal) => LiveIdentity::Refused(refusal),
    };
    return ParentIdentityFacts {
        recorded,
        live,
    };
}

/// The one slot's recorded identity against its live folder.
fn slot_facts(slot: &str) -> SlotIdentityFacts {
    let request: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest {
        checkout: Path::new(CHECKOUT),
        slot,
        store: Path::new(store_path()),
    };
    let refusal: Option<CloneSlotRefusal> = hold_clone_slot_under_root_control(&request).err();
    return SlotIdentityFacts {
        slot: String::from(slot),
        refusal,
    };
}

/// What one record answered.
fn record_facts(
    name: &str,
    store: &SlotIdentityStore,
) -> RecordFacts {
    let read = if name == PARENT_RECORD_NAME {
        store.parent()
    } else {
        store.slot(name)
    };
    return match read {
        Ok(_) => RecordFacts::Read {
            recorded: String::from(name),
        },
        Err(StoreRefusal::RecordUnderAnotherName {
            recorded,
            ..
        }) => RecordFacts::Misfiled {
            recorded,
        },
        Err(
            StoreRefusal::NoParentRecord {
                ..
            }
            | StoreRefusal::NoSlotRecord {
                ..
            },
        ) => RecordFacts::Absent,
        Err(refusal) => RecordFacts::Unreadable {
            why: refusal.message(),
        },
    };
}

/// The store's listing and every record in it.
fn store_facts() -> StoreListingFacts {
    let store: SlotIdentityStore =
        SlotIdentityStore::open_under_root_control(Path::new(store_path()))
            .expect("the store opens under root control");
    let mut entries: Vec<git_policy_cli::doctor_zfs_route::ListedEntry> = Vec::new();
    let mut slots: Vec<SlotRecordFacts> = Vec::new();
    let mut parent: RecordFacts = RecordFacts::Absent;
    let now: std::time::SystemTime = std::time::SystemTime::now();
    for listed in std::fs::read_dir(store_path()).expect("the store lists") {
        let entry = listed.expect("one entry");
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let age: Option<u64> = entry
            .metadata()
            .ok()
            .and_then(|metadata| return metadata.modified().ok())
            .and_then(|modified| return now.duration_since(modified).ok())
            .map(|elapsed| return elapsed.as_secs());
        entries.push(git_policy_cli::doctor_zfs_route::ListedEntry {
            name: name.clone(),
            modified_seconds_ago: age,
        });
        if name == PARENT_RECORD_NAME {
            parent = record_facts(name.as_str(), &store);
            continue;
        }
        if name.starts_with('.') {
            continue;
        }
        slots.push(SlotRecordFacts {
            name: name.clone(),
            facts: record_facts(name.as_str(), &store),
        });
    }
    return StoreListingFacts {
        entries,
        parent,
        slots,
    };
}

/// The account's effective delegation,
/// from both datasets' replies,
/// which carry their own ancestors' grants.
fn delegation_facts() -> DelegationFacts {
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let mut grants: Vec<git_policy_cli::doctor_zfs_route::Grant> = Vec::new();
    for dataset in [checkout_dataset(), container_dataset()] {
        let reply = device
            .get_fsacl(dataset.as_str())
            .unwrap_or_else(|error| panic!("{dataset}'s delegation: {error}"));
        grants.extend(grants_of(&reply).expect("the kernel's delegation shape"));
    }
    return DelegationFacts {
        account: ACCOUNT,
        groups: vec![ACCOUNT],
        checkout_dataset: checkout_dataset(),
        container_dataset: container_dataset(),
        grants,
    };
}

/// The handler's installation,
/// configuration,
/// folders,
/// wiring and records.
fn handler_facts() -> (
    git_policy_cli::doctor_zfs_route::HandlerInstallationFacts,
    git_policy_cli::doctor_zfs_route::HandlerConfigurationFacts,
    git_policy_cli::doctor_zfs_route::FolderFacts,
    git_policy_cli::doctor_zfs_route::HandlerWiringFacts,
    ReadinessFacts,
) {
    use git_policy_cli::doctor_zfs_route::{
        FolderFacts,
        HandlerConfigurationFacts,
        HandlerInstallationFacts,
        HandlerWiringFacts,
        PathFacts,
    };
    let path = |path: &str| {
        return PathFacts {
            path: PathBuf::from(path),
            state: path_state(Path::new(path)),
        };
    };
    let installation: HandlerInstallationFacts = HandlerInstallationFacts {
        executable: path(EXECUTABLE_PATH),
        zedlet: path(ZEDLET_PATH),
    };
    let configuration: HandlerConfigurationFacts = HandlerConfigurationFacts {
        file: path(CONFIGURATION_PATH),
        parse_error: read_configuration(Path::new(CONFIGURATION_PATH), &LiveMountedDatasets)
            .err()
            .map(|refusal| return refusal.message()),
    };
    let folders: FolderFacts = FolderFacts {
        readiness: path(READINESS_DIRECTORY),
        results: path(RESULTS_DIRECTORY),
    };
    let mut zedlets: Vec<String> = Vec::new();
    for entry in std::fs::read_dir("/etc/zfs/zed.d")
        .map(|entries| return entries.flatten())
        .into_iter()
        .flatten()
    {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if format!("/etc/zfs/zed.d/{name}") != ZEDLET_PATH {
            zedlets.push(name);
        }
    }
    let zed_unit: &str = [
        "/lib/systemd/system/zfs-zed.service",
        "/usr/lib/systemd/system/zfs-zed.service",
        "/opt/openzfs-2.4.4/lib/systemd/system/zfs-zed.service",
    ]
    .into_iter()
    .find(|unit| return Path::new(unit).exists())
    .expect("the guest holds the zfs-zed unit this arm enabled");
    let wiring: HandlerWiringFacts = HandlerWiringFacts {
        zed_unit: path(zed_unit),
        zed_enabled: Path::new("/etc/systemd/system/multi-user.target.wants/zfs-zed.service")
            .exists()
            || Path::new("/etc/systemd/system/zfs-zed.service.wants").exists(),
        other_zedlets: zedlets,
    };
    let readiness: ReadinessFacts =
        match std::fs::read_to_string(format!("{READINESS_DIRECTORY}/result")) {
            Ok(text) => {
                match git_policy_cli::worktree_clone_mount_protocol::parse_readiness(text.as_str())
                {
                    Ok(record) => ReadinessFacts::Read {
                        boot: record.boot,
                        pool: record.pool_state,
                        checkout: record.checkout_state,
                        sweep: record.sweep_state.word().to_string(),
                    },
                    Err(refusal) => ReadinessFacts::Unreadable {
                        why: format!("{refusal:?}"),
                    },
                }
            }
            Err(error) => ReadinessFacts::Absent {
                why: error.to_string(),
            },
        };
    return (installation, configuration, folders, wiring, readiness);
}

/// Each clone's request and the receipt that answers it.
fn request_facts() -> Vec<RequestFacts> {
    let mut requests: Vec<RequestFacts> = Vec::new();
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let mut cursor: u64 = 0;
    while let Some(listed) = device
        .dataset_list_next(container_dataset().as_str(), cursor)
        .expect("the container's children")
    {
        cursor = listed.cursor;
        let name: String = listed.name;
        if name.contains('@') {
            continue;
        }
        let held: DatasetIdentity = identity(name.as_str());
        let intent_nonce: Option<String> = match read_intent(Some(&held.properties), name.as_str())
        {
            IntentReading::Request(intent) => Some(String::from(intent.nonce.as_str())),
            IntentReading::Absent
            | IntentReading::Inherited {
                ..
            }
            | IntentReading::Refused(_) => None,
        };
        let receipt = ReceiptsFolder::live()
            .read(
                DatasetGuid::parse(held.stats.dds_guid.to_string().as_str())
                    .expect("the clone's GUID"),
            )
            .ok()
            .flatten();
        requests.push(RequestFacts {
            clone: name,
            intent_nonce,
            receipt_nonce: receipt
                .as_ref()
                .map(|answered| return String::from(answered.nonce.as_str())),
            receipt_boot: receipt.map(|answered| return answered.boot),
        });
    }
    return requests;
}

/// The cache and the staging folders under it.
fn staging_facts() -> StagingFolderFacts {
    let cache: String = format!("{FIXTURE}/cache");
    let mut folders: Vec<StagedFolder> = Vec::new();
    for entry in std::fs::read_dir(cache.as_str())
        .expect("the cache lists")
        .flatten()
    {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(
            git_policy_cli::worktree_clone_unmount::staging_folder_name::STAGING_FOLDER_PREFIX,
        ) {
            continue;
        }
        let state: PathState = path_state(entry.path().as_path());
        let empty: bool = std::fs::read_dir(entry.path())
            .map(|mut entries| return entries.next().is_none())
            .unwrap_or(false);
        folders.push(StagedFolder {
            name: name.clone(),
            state,
            derived: git_policy_cli::doctor_zfs_route::is_derived_folder_name(name.as_str()),
            empty,
        });
    }
    return StagingFolderFacts {
        cache: git_policy_cli::doctor_zfs_route::PathFacts {
            path: PathBuf::from(cache.as_str()),
            state: path_state(Path::new(cache.as_str())),
        },
        account: ACCOUNT,
        folders,
    };
}

//endregion

//region The arm

/// The three groups pass on the design's layout,
/// the removal-side lines pass on a clone no record names,
/// the cancel command passes and every check whose fact this arm does not
/// read is unknown,
/// and each broken fact answers the finding that names it.
#[test]
fn zfs_doctor_route_guest_three_groups_and_each_finding() {
    let Ok(guest) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP {GUEST_VARIABLE} names no guest");
        return;
    };
    if guest.is_empty() {
        println!("SKIP {GUEST_VARIABLE} is empty");
        return;
    }
    install_layout();
    let boot: String = live_boot();
    let reports: Vec<CheckReport> = route_checks(&route_facts(boot.as_str()));
    assert_pass(&reports, "device");
    assert_pass(&reports, "abi");
    assert_pass(&reports, "pool");
    assert_pass(&reports, "checkout-dataset");
    assert_pass(&reports, "container-dataset");
    assert_pass(&reports, "checkout-snapdir");
    assert_pass(&reports, "container-snapdir");
    assert_pass(&reports, "key");
    assert_pass(&reports, "clone-properties");
    println!("DOCTOR-GUEST group-one-passes");
    assert_pass(&reports, "parent-identity");
    assert_pass(&reports, "store-entries");
    assert_pass(&reports, "staging-forms");
    assert_pass(&reports, "unknown-dot-entry");
    assert_pass(&reports, "slot-identity");
    println!("DOCTOR-GUEST group-two-passes");
    assert_pass(&reports, "checkout-mount");
    assert_pass(&reports, "checkout-propagation");
    assert_pass(&reports, "nested-reachability");
    assert_pass(&reports, "delegation");
    assert_pass(&reports, "handler-installation");
    assert_pass(&reports, "root-modes");
    assert_pass(&reports, "handler-configuration");
    assert_pass(&reports, "handler-folders");
    assert_pass(&reports, "zed-wiring");
    assert_pass(&reports, "readiness");
    assert_pass(&reports, "unresolved-requests");
    assert_pass(&reports, "staging-folder");
    assert_pass(&reports, "staging-bind-propagation");
    println!("DOCTOR-GUEST group-three-passes");
    for name in [
        "cancel-command",
        "boot-protection",
        "service-unit",
        "backup-coverage",
        "sanoid",
    ] {
        assert_unknown(&reports, name);
    }
    for name in [
        "clone-state",
        "registration",
        "binding-record",
        "removal-plan",
        "clone-snapshots",
        "clone-origin",
        "clone-retention",
        "blocked-removal",
        "holders",
    ] {
        assert_pass(&reports, name);
    }
    println!("DOCTOR-GUEST unread-step-facts-are-unknown");
    let cancel: &CheckReport = report_of(&reports, "cancel-command");
    assert!(cancel.passed(), "{}", cancel.line());
    for name in [
        "boot-protection",
        "service-unit",
        "backup-coverage",
        "sanoid",
    ] {
        assert_unknown(&reports, name);
    }
    break_each_fact();
}

/// The whole fact set for one live boot.
fn route_facts(boot: &str) -> git_policy_cli::doctor_zfs_route::RouteFacts {
    let (installation, configuration, folders, wiring, readiness) = handler_facts();
    let mounts: MountCheckFacts = mount_facts();
    let cache: PathBuf = PathBuf::from(format!("{FIXTURE}/cache"));
    return git_policy_cli::doctor_zfs_route::RouteFacts {
        device: device_facts(),
        abi: abi_facts(),
        pool: pool_facts(),
        datasets: dataset_facts(),
        staging_binds: staging_binds(&mounts.rows, cache.as_path()),
        clones: vec![clone_facts(mounts.rows.as_slice())],
        mounts,
        parent_identity: parent_facts(),
        slots: vec![slot_facts("one")],
        store: store_facts(),
        delegation: delegation_facts(),
        installation,
        configuration,
        folders,
        wiring,
        readiness,
        live_boot: String::from(boot),
        requests: request_facts(),
        staging: staging_facts(),
        boot_protection: unread_boot_protection(),
        boot_setup: unread_boot_setup(),
        cancel_command: cancel_command_facts(),
        sanoid: unread_sanoid(),
    };
}

/// What:
///  The covered entry this arm does not read,
///  stated as the refusal a read would have to report.
/// Why:
///  the entry's protection is plan step 19's own arm,
///  and an arm that filled a shape by hand would claim a reading of the guest's
///  own underlay that it never made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entry = { kind: 'refused', what: 'the `statx` read', path: FIXTURE, reason: 'this arm reads no underlay entry' };
/// ```
fn unread_boot_protection() -> git_policy_cli::doctor_zfs_route::BootProtectionFacts {
    return git_policy_cli::doctor_zfs_route::BootProtectionFacts {
        entry: Err(
            git_policy_cli::checkout_underlay_entry::underlay_entry_read::EntryReadRefusal::Call {
                what: "the `statx` read",
                path: PathBuf::from(FIXTURE),
                reason: String::from("this arm reads no underlay entry"),
            },
        ),
        wanted_owner: ACCOUNT,
    };
}

/// What:
///  The boot setup this arm does not read:
///  a gate whose file nobody looked at,
///  and no dataset measured under the checkout.
/// Why:
///  the files and the property are plan step 19's own arm,
///  and a reading nobody gathered has to answer unknown rather than pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const setup = { gate: { path: '/etc/systemd/system/…', plannedText: '', atThePath: { kind: 'unreadable', why: 'this arm reads no boot setup' } }, others: [], keyUnit: undefined, nested: [] };
/// ```
fn unread_boot_setup() -> git_policy_cli::doctor_zfs_route::BootSetupFacts {
    return git_policy_cli::doctor_zfs_route::BootSetupFacts {
        gate: git_policy_cli::doctor_zfs_route::PlannedFileReading {
            path: PathBuf::from(
                git_policy_cli::checkout_boot_units::boot_unit_names::BOOT_GATE_UNIT_PATH,
            ),
            planned_text: String::new(),
            at_the_path: git_policy_cli::doctor_zfs_route::FileAtInstallPath::Unreadable {
                why: String::from("this arm reads no boot setup"),
            },
        },
        others: Vec::new(),
        key_unit: None,
        nested: Vec::new(),
    };
}

/// What:
///  The cancel command's facts,
///  read from the command's own grammar.
/// Why:
///  the line's only fact is the built command,
///  and this arm is another crate,
///  so it reads the grammar where the wiring run does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cancel = { word: CANCEL_COMMAND_WORD, usageLine: usage().split('\n')[0] };
/// ```
fn cancel_command_facts() -> git_policy_cli::doctor_zfs_route::CancelCommandFacts {
    let text: String = git_policy_cli::worktree_clone_removal_cancel::cancel_arguments::usage();
    let first_line: String = String::from(text.lines().next().expect("the usage has a line"));
    return git_policy_cli::doctor_zfs_route::CancelCommandFacts {
        word: String::from(
            git_policy_cli::worktree_clone_removal_cancel::cancel_arguments::CANCEL_COMMAND_WORD,
        ),
        usage_line: first_line,
    };
}

/// What:
///  The Sanoid comparison this arm does not make,
///  answered unknown in the shape the report takes.
/// Why:
///  the coverage comparison reads a configuration and a dataset tree this
///  arm's fixture does not build,
///  so the line says so instead of standing in for a comparison nobody ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sanoid = { outcome: { kind: 'unknown', reason: 'this arm reads no sanoid.conf' }, lines: [] };
/// ```
fn unread_sanoid() -> git_policy_cli::doctor_zfs_route::SanoidReport {
    return git_policy_cli::doctor_zfs_route::SanoidReport::new(
        CheckOutcome::Unknown {
            reason: String::from("this arm reads no `sanoid.conf` and measures no dataset tree"),
        },
        Vec::new(),
    );
}

/// What:
///  The pool this arm asks for one dataset's GUID.
/// Why:
///  The managed lock's proof compares the mount's clone GUID while a mount
///  covers the slot,
///  so the guest's own pool answers,
///  through the arm's own identity read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ThisGuestsPool implements DatasetGuidReads { guidOf(dataset) { return identity(dataset).guid; } }
/// ```
struct ThisGuestsPool;

impl git_policy_cli::worktree_clone_managed_lock::DatasetGuidReads for ThisGuestsPool {
    fn guid_of(
        &self,
        dataset: &str,
    ) -> Result<git_policy_cli::worktree_clone_mount_protocol::DatasetGuid, String> {
        return git_policy_cli::worktree_clone_mount_protocol::DatasetGuid::parse(
            identity(dataset).stats.dds_guid.to_string().as_str(),
        )
        .map_err(|refusal| return format!("{refusal:?}"));
    }
}

/// The one managed clone of this arm's fixture,
/// read through the route's own slot readings and pool replies.
fn clone_facts(
    rows: &[git_policy_cli::fs_mount_options::MountFacts]
) -> git_policy_cli::doctor_zfs_route::CloneFacts {
    use git_policy_cli::doctor_zfs_route::{
        AccountingFacts,
        BlockedRemovalFacts,
        CloneFacts,
        CloneOriginFacts,
        CloneRetentionFacts,
        CloneSnapshotFacts,
        CloneStateFacts,
        HolderFacts,
        LOGICAL_REFERENCED_PROPERTY,
        NamespaceHold,
        PathState,
        PlanFileReading,
        REFERENCED_PROPERTY,
        RemovalPlanFacts,
        USED_PROPERTY,
        WRITTEN_PROPERTY,
        staging_binds,
    };
    use git_policy_cli::worktree_clone_mount_handler::sweep_dataset_facts::value_of;
    use git_policy_cli::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
    use git_policy_cli::worktree_clone_mount_protocol::RequestNonce;
    use git_policy_cli::worktree_clone_removal_proof_and_plan::plan_record::PlanSnapshot;
    use git_policy_cli::worktree_clone_removal_proof_and_plan::removal_inventory::LiveInventory;
    use git_policy_cli::worktree_clone_removal_proof_and_plan::removal_reads::{
        LiveRemovalReads,
        whole_inventory,
    };
    use git_policy_cli::worktree_clone_removal_proof_and_plan::started_moment::now_with_offset;
    use git_policy_cli::worktree_clone_slot_state::slot_state_live_read::{
        GatedCloneIntentReads,
        SlotReadingSources,
        read_slot,
    };
    let common: PathBuf = PathBuf::from(format!("{FIXTURE}/repo/.git"));
    let slot: PathBuf = PathBuf::from(format!("{CHECKOUT}/.claude/worktrees/one"));
    let intents: GatedCloneIntentReads<'_> =
        GatedCloneIntentReads::new(&git_policy_cli::zfs_compatibility_gate::LiveGate);
    let mounts: git_policy_cli::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets =
        git_policy_cli::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
    let readings: git_policy_cli::worktree_clone_slot_state::slot_state_readings::SlotReadings =
        read_slot(
            common.as_path(),
            "one",
            slot.as_path(),
            &SlotReadingSources {
                mounts: &mounts,
                intents: &intents,
            },
        );
    let clone: String = clone_dataset();
    let guid: u64 = identity(clone.as_str()).stats.dds_guid;
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let inventory: LiveInventory = whole_inventory(
        &reads,
        clone.as_str(),
        DatasetGuid::parse(guid.to_string().as_str()).expect("the clone's GUID"),
    )
    .expect("the clone's inventory");
    let device: ZfsDevice = ZfsDevice::open().expect("the device opens in the guest");
    let held: git_policy_cli::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts =
        git_policy_cli::worktree_clone_mount_handler::sweep_dataset_facts::facts_of(
            clone.as_str(),
            &identity(clone.as_str()),
        );
    let accounting = |object: &str| -> AccountingFacts {
        let reply = device
            .objset_stats(object)
            .unwrap_or_else(|error| panic!("{object}'s accounting: {error}"));
        return AccountingFacts {
            object: String::from(object),
            used: value_of(&reply.properties, USED_PROPERTY),
            referenced: value_of(&reply.properties, REFERENCED_PROPERTY),
            logical_referenced: value_of(&reply.properties, LOGICAL_REFERENCED_PROPERTY),
            written: value_of(&reply.properties, WRITTEN_PROPERTY),
        };
    };
    let mut own: Vec<AccountingFacts> = Vec::new();
    for snapshot in inventory.snapshots() {
        own.push(accounting(snapshot.full_name().as_str()));
    }
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().expect("the device opens in the guest");
    let _ = &reader;
    let nonce: RequestNonce = RequestNonce::parse(NONCE).expect("the arm's nonce");
    let own_namespace: Option<String> =
        git_policy_cli::mount_namespace_scan::namespace_target(Path::new("/proc/self/ns/mnt"));
    let scan = git_policy_cli::clone_holder_scan::scan_holders(
        &git_policy_cli::clone_holder_scan::HolderScanRequest {
            proc_root: Path::new("/proc"),
            source: clone.as_str(),
            own_namespace: own_namespace.as_deref(),
            readers_root: None,
            dataset: DatasetGuid::parse(guid.to_string().as_str()).expect("the clone's GUID"),
            nonce: &nonce,
            time: now_with_offset(),
        },
    );
    let mut holders: Vec<NamespaceHold> = Vec::new();
    for holder in &scan.holders {
        let task: u32 = holder
            .task
            .file_name()
            .and_then(|name| return name.to_str())
            .and_then(|text| return text.parse().ok())
            .unwrap_or(0);
        for mount in &holder.mounts {
            holders.push(NamespaceHold {
                namespace: String::from(holder.namespace()),
                representative_pid: task,
                source: clone.clone(),
                mount_point: mount.mount_point.clone(),
            });
        }
    }
    let incomplete: Vec<String> = scan
        .incomplete
        .iter()
        .map(|read| return format!("{read:?}"))
        .collect();
    let holder_facts: HolderFacts = HolderFacts {
        clone: clone.clone(),
        holders,
        incomplete,
    };
    let cache: PathBuf = PathBuf::from(format!("{FIXTURE}/cache"));
    let _ = (
        PlanSnapshot {
            name: String::from("unused"),
            guid: DatasetGuid::parse("1").expect("a GUID"),
            createtxg: 0,
        },
        held,
    );
    return CloneFacts {
        state: CloneStateFacts {
            readings,
            staging_binds: staging_binds(rows, cache.as_path()),
        },
        registration: git_policy_cli::doctor_zfs_route::RegistrationFacts {
            slot: String::from("one"),
            recorded: None,
            at_the_path: PathState::Absent,
            gitdir: None,
            slot_git: slot.join(".git"),
        },
        lock: git_policy_cli::doctor_zfs_route::ManagedLockFacts {
            slot: String::from("one"),
            classification: git_policy_cli::worktree_clone_managed_lock::classify_the_managed_lock(
                common.as_path(),
                "one",
                slot.as_path(),
                &mounts,
                &ThisGuestsPool,
            ),
        },
        plan: RemovalPlanFacts {
            clone: clone.clone(),
            reading: PlanFileReading::NoPlan,
        },
        snapshots: CloneSnapshotFacts {
            clone: clone.clone(),
            clone_createtxg: identity(clone.as_str()).stats.dds_creation_txg,
            live: git_policy_cli::doctor_zfs_route::Reading::Read(inventory),
            plan_stands: false,
            recorded: Vec::new(),
        },
        origin: CloneOriginFacts {
            clone: clone.clone(),
            recorded: None,
            live: None,
        },
        retention: CloneRetentionFacts {
            clone: clone.clone(),
            dataset: git_policy_cli::doctor_zfs_route::Reading::Read(accounting(clone.as_str())),
            snapshots: own,
            inherited: Vec::new(),
            origin_snapshot: None,
        },
        blocked: BlockedRemovalFacts {
            slot: String::from("one"),
            clone: clone.clone(),
            intent: None,
            plan_stands: false,
            recorded: Vec::new(),
            holders: holder_facts.clone(),
            in_the_callers_session: Vec::new(),
            the_forced_arm_is_measured: false,
        },
        holders: holder_facts,
    };
}

/// One check passed.
fn assert_pass(
    reports: &[CheckReport],
    name: &str,
) {
    let report: &CheckReport = report_of(reports, name);
    assert!(report.passed(), "{}", report.line());
}

/// One check answered unknown.
fn assert_unknown(
    reports: &[CheckReport],
    name: &str,
) {
    let report: &CheckReport = report_of(reports, name);
    assert!(report.unknown(), "{}", report.line());
}

/// The report of one named check.
fn report_of<'a>(
    reports: &'a [CheckReport],
    name: &str,
) -> &'a CheckReport {
    return reports
        .iter()
        .find(|report| return report.check.name() == name)
        .unwrap_or_else(|| {
            panic!(
                "no {name} report: {:?}",
                reports
                    .iter()
                    .map(|report| return report.check.name())
                    .collect::<Vec<&str>>()
            )
        });
}

/// Every broken fact this arm can make,
/// each in its own function so no local outlives the answer it read.
fn break_each_fact() {
    breaks_container_canmount();
    breaks_clone_setuid();
    breaks_unloaded_key();
    breaks_nested_row();
    breaks_misfiled_record();
    breaks_staging_forms();
    breaks_rename_grant();
    breaks_readiness_boot();
    breaks_missing_slot();
    breaks_snapdir_visible();
    breaks_snapdir_hidden();
    breaks_snapdir_default();
    snapdir_follows_the_container();
}

/// A container that could mount itself is a finding naming `canmount`.
fn breaks_container_canmount() {
    zfs(&["set", "canmount=on", container_dataset().as_str()]);
    let datasets: Vec<DatasetFacts> = dataset_facts();
    let container: &DatasetFacts = datasets
        .iter()
        .find(|dataset| return dataset.name == container_dataset())
        .expect("the container is read");
    let answer: CheckOutcome = git_policy_cli::doctor_zfs_route::check_dataset(container);
    let CheckOutcome::Finding {
        field,
        ..
    } = answer
    else {
        panic!("a container that can mount is a finding: {answer:?}");
    };
    assert_eq!(field, "canmount");
    println!("DOCTOR-GUEST container-canmount-is-a-finding");
    zfs(&["set", "canmount=off", container_dataset().as_str()]);
}

/// A clone carrying setuid is a finding naming `setuid`.
fn breaks_clone_setuid() {
    zfs(&["set", "setuid=on", clone_dataset().as_str()]);
    let clones: Vec<DatasetFacts> = dataset_facts()
        .into_iter()
        .filter(|dataset| return matches!(dataset.role, DatasetRole::Clone))
        .collect();
    let answer: CheckOutcome = check_clone_properties(&clones);
    let CheckOutcome::Finding {
        field,
        ..
    } = answer
    else {
        panic!("a clone carrying setuid is a finding: {answer:?}");
    };
    assert_eq!(field, "setuid");
    zfs(&["set", "setuid=off", clone_dataset().as_str()]);
}

/// A clone whose key is not loaded is a finding naming `keystatus`.
fn breaks_unloaded_key() {
    let key_file: String = format!("{FIXTURE}/key");
    std::fs::write(key_file.as_str(), "correct horse battery staple").expect("the key file");
    let encrypted: String = format!("{}/home/encrypted", pool());
    zfs(&[
        "create",
        "-o",
        "encryption=on",
        "-o",
        "keyformat=passphrase",
        "-o",
        format!("keylocation=file://{key_file}").as_str(),
        "-o",
        "mountpoint=legacy",
        "-o",
        "canmount=off",
        encrypted.as_str(),
    ]);
    zfs(&["snapshot", format!("{encrypted}@ready").as_str()]);
    let encrypted_clone: String = format!("{}/encrypted-clone", container_dataset());
    zfs(&[
        "clone",
        "-o",
        "mountpoint=legacy",
        format!("{encrypted}@ready").as_str(),
        encrypted_clone.as_str(),
    ]);
    zfs(&["unload-key", encrypted.as_str()]);
    let unloaded: Vec<DatasetFacts> = vec![DatasetFacts {
        role: DatasetRole::Clone,
        name: encrypted_clone.clone(),
        reading: reading(encrypted_clone.as_str()),
    }];
    let answer: CheckOutcome = check_key(&unloaded);
    let CheckOutcome::Finding {
        field,
        ..
    } = answer
    else {
        panic!("an unloaded key is a finding: {answer:?}");
    };
    assert_eq!(field, "keystatus");
    println!("DOCTOR-GUEST unloaded-key-is-a-finding");
    zfs(&["load-key", encrypted.as_str()]);
    zfs(&["destroy", "-r", "-f", encrypted_clone.as_str()]);
    zfs(&["destroy", "-r", "-f", encrypted.as_str()]);
}

/// A nested dataset whose row hangs from the tree the checkout replaced is a
/// finding naming the path's reachability.
fn breaks_nested_row() {
    zfs(&["unmount", checkout_dataset().as_str()]);
    std::fs::create_dir_all(CHECKOUT).expect("the underlay path");
    let output: String = format!("{}/home/vm-builder-output", pool());
    zfs(&[
        "create",
        "-o",
        format!("mountpoint={CHECKOUT}/output").as_str(),
        output.as_str(),
    ]);
    zfs(&["mount", checkout_dataset().as_str()]);
    let mounts: MountCheckFacts = mount_facts();
    let answer: CheckOutcome = git_policy_cli::doctor_zfs_route::check_nested_reachability(&mounts);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a row hanging from the replaced tree is a finding: {answer:?}");
    };
    assert_eq!(field, "reachability");
    assert!(detail.contains("vm-builder-output"), "{detail}");
    println!("DOCTOR-GUEST nested-row-is-a-finding");
    zfs(&["unmount", checkout_dataset().as_str()]);
    zfs(&["unmount", output.as_str()]);
    zfs(&["destroy", "-r", "-f", output.as_str()]);
    zfs(&["mount", checkout_dataset().as_str()]);
}

/// A slot record filed under another name is a finding naming both.
fn breaks_misfiled_record() {
    let misfiled: String = format!("{}/two", store_path());
    std::fs::copy(format!("{}/one", store_path()), misfiled.as_str()).expect("the misfiled copy");
    let store: StoreListingFacts = store_facts();
    let answer: CheckOutcome = git_policy_cli::doctor_zfs_route::check_store_entries(&store);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a misfiled record is a finding: {answer:?}");
    };
    assert_eq!(field, "record");
    assert!(detail.contains("two"), "{detail}");
    println!("DOCTOR-GUEST misfiled-record-is-a-finding");
    std::fs::remove_file(misfiled.as_str()).expect("the misfiled copy goes");
}

/// Both staging forms and a stranger's dot name:
/// the forms are reported and removed,
/// and every other byte stays.
fn breaks_staging_forms() {
    let parent_before: Vec<u8> =
        std::fs::read(format!("{}/{PARENT_RECORD_NAME}", store_path())).expect("the parent bytes");
    let slot_before: Vec<u8> =
        std::fs::read(format!("{}/one", store_path())).expect("the slot bytes");
    for planted in [
        ".0123456789abcdef0123456789abcdef",
        "..parent.0123456789abcdef.partial",
        ".partial-write",
    ] {
        std::fs::write(
            format!("{}/{planted}", store_path()),
            b"left by a killed writer",
        )
        .expect("a planted entry");
    }
    let store: StoreListingFacts = store_facts();
    let answer: CheckOutcome = git_policy_cli::doctor_zfs_route::check_staging_forms(&store);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("two staging forms are a finding: {answer:?}");
    };
    assert_eq!(field, "staging");
    assert!(detail.contains('2'), "{detail}");
    // The dot entry that matches neither form is reported and left in place,
    // and the two known forms are not reported as unknown names.
    let unknown: CheckOutcome = check_unknown_dot_entries(&store);
    let CheckOutcome::Finding {
        field: unknown_field,
        detail: unknown_detail,
    } = unknown
    else {
        panic!("a stranger's dot entry is a finding: {unknown:?}");
    };
    assert_eq!(unknown_field, "name");
    assert!(
        unknown_detail.contains(".partial-write"),
        "{unknown_detail}"
    );
    assert!(
        !unknown_detail.contains(".0123456789abcdef0123456789abcdef"),
        "{unknown_detail}"
    );
    assert!(!unknown_detail.contains("..parent."), "{unknown_detail}");
    let stored: SlotIdentityStore =
        SlotIdentityStore::open_under_root_control(Path::new(store_path()))
            .expect("the store opens under root control");
    let removal = remove_known_staging_forms(StagingRemovalRequest {
        store: &stored,
    })
    .expect("the removal runs");
    assert_eq!(removal.removed.len(), 2, "{removal:?}");
    assert_eq!(
        std::fs::read(format!("{}/{PARENT_RECORD_NAME}", store_path())).expect("the parent bytes"),
        parent_before
    );
    assert_eq!(
        std::fs::read(format!("{}/one", store_path())).expect("the slot bytes"),
        slot_before
    );
    assert!(
        Path::new(format!("{}/.partial-write", store_path()).as_str()).exists(),
        "the stranger's name stays"
    );
    println!("DOCTOR-GUEST staging-forms-are-removed-and-nothing-else-is");
    std::fs::remove_file(format!("{}/.partial-write", store_path()))
        .expect("the planted stranger goes");
}

/// A lineage-changing grant is a finding naming the permission.
fn breaks_rename_grant() {
    zfs(&[
        "allow",
        "-d",
        "-u",
        ACCOUNT.to_string().as_str(),
        "rename",
        container_dataset().as_str(),
    ]);
    let delegation: DelegationFacts = delegation_facts();
    let answer: CheckOutcome = check_delegation(&delegation);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a rename grant is a finding: {answer:?}");
    };
    assert_eq!(field, "permission");
    assert!(detail.contains("rename"), "{detail}");
    println!("DOCTOR-GUEST rename-grant-is-a-finding");
    zfs(&[
        "unallow",
        "-d",
        "-u",
        ACCOUNT.to_string().as_str(),
        "rename",
        container_dataset().as_str(),
    ]);
}

/// A readiness record judged against another boot is a finding naming the boot.
fn breaks_readiness_boot() {
    let (_, _, _, _, readiness) = handler_facts();
    let answer: CheckOutcome =
        git_policy_cli::doctor_zfs_route::check_readiness(&readiness, OTHER_BOOT);
    let CheckOutcome::Finding {
        field,
        ..
    } = answer
    else {
        panic!("another boot's record is a finding: {answer:?}");
    };
    assert_eq!(field, "boot");
    println!("DOCTOR-GUEST readiness-from-another-boot-is-a-finding");
}

/// A slot the walk cannot reach is a finding naming the component.
fn breaks_missing_slot() {
    let missing: SlotIdentityFacts = slot_facts("no-such-slot");
    let answer: CheckOutcome = git_policy_cli::doctor_zfs_route::check_slot_identity(&missing);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a missing slot is a finding: {answer:?}");
    };
    assert!(field.contains("no-such-slot"), "{field}");
    assert!(detail.contains("not there"), "{detail}");
    println!("DOCTOR-GUEST missing-slot-is-a-finding");
}

/// What:
///  A checkout dataset that lists its snapshot directory is a finding naming
///  `visible`,
/// the command that sets `disabled`,
/// and the module floor.
/// Why:
///  Only `disabled` keeps the kernel from mounting a snapshot an account enters
///  by name,
///  so a person must learn the value,
/// the command and that an older module has
///  no such value,
///  and the value is read through this route's own dataset-stats reply rather
///  than through `zfs get`
///  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
///  "What hiding comes to,
/// answered 2026-11-23").
fn breaks_snapdir_visible() {
    zfs(&["set", "snapdir=visible", checkout_dataset().as_str()]);
    let datasets: Vec<DatasetFacts> = dataset_facts();
    let checkout: &DatasetFacts = datasets
        .iter()
        .find(|dataset| return dataset.name == checkout_dataset())
        .expect("the checkout is read");
    let answer: CheckOutcome = check_snapdir(checkout);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a checkout showing its snapshot directory is a finding: {answer:?}");
    };
    assert_eq!(field, "snapdir");
    assert!(detail.contains("`visible`"), "{detail}");
    assert!(
        detail.contains(format!("zfs set snapdir=disabled {}", checkout_dataset()).as_str()),
        "{detail}"
    );
    assert!(detail.contains("OpenZFS 2.3.0"), "{detail}");
    println!("DOCTOR-GUEST snapdir-visible-is-a-finding");
    zfs(&["set", "snapdir=disabled", checkout_dataset().as_str()]);
}

/// What:
///  A container set to `hidden` is a finding naming that word,
///  which is how this arm measures what the kernel reports for the value the
///  reply otherwise leaves out.
/// Why:
///  The default and an explicitly set `hidden` are the same state to a person,
///  so a build that could not name one of them would report a value nobody can
///  act on.
fn breaks_snapdir_hidden() {
    zfs(&["set", "snapdir=hidden", container_dataset().as_str()]);
    let datasets: Vec<DatasetFacts> = dataset_facts();
    let container: &DatasetFacts = datasets
        .iter()
        .find(|dataset| return dataset.name == container_dataset())
        .expect("the container is read");
    let answer: CheckOutcome = check_snapdir(container);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("a container set to hidden is a finding: {answer:?}");
    };
    assert_eq!(field, "snapdir");
    assert!(detail.contains("`hidden`"), "{detail}");
    assert!(detail.contains("every clone below it inherits"), "{detail}");
    println!("DOCTOR-GUEST snapdir-hidden-is-a-finding");
    zfs(&["set", "snapdir=disabled", container_dataset().as_str()]);
}

/// What:
///  A container that inherits an unset value is a finding naming the default
///  `hidden`,
///  which is the one state the kernel's reply leaves out.
/// Why:
///  An absent property must be reported as the default and not as an unread
///  value,
///  because a machine that never ran the setup has to learn what its snapshot
///  directories still do.
fn breaks_snapdir_default() {
    zfs(&["inherit", "snapdir", container_dataset().as_str()]);
    let datasets: Vec<DatasetFacts> = dataset_facts();
    let container: &DatasetFacts = datasets
        .iter()
        .find(|dataset| return dataset.name == container_dataset())
        .expect("the container is read");
    let answer: CheckOutcome = check_snapdir(container);
    let CheckOutcome::Finding {
        field,
        detail,
    } = answer
    else {
        panic!("an unset container is a finding: {answer:?}");
    };
    assert_eq!(field, "snapdir");
    assert!(detail.contains("unset"), "{detail}");
    assert!(detail.contains("default `hidden`"), "{detail}");
    println!("DOCTOR-GUEST snapdir-default-is-a-finding");
    zfs(&["set", "snapdir=disabled", container_dataset().as_str()]);
}

/// One dataset's `snapdir` as the route's own dataset-stats reply reports it.
fn snapdir_of(dataset: &str) -> git_policy_cli::doctor_zfs_route::SnapdirSetting {
    let held: DatasetIdentity = identity(dataset);
    return snapdir_setting(&held.properties);
}

/// What:
///  A dataset created plainly below the container follows the container's
///  `snapdir` rather than keeping a copy,
///  which is the fact the container's line states about every clone.
/// Why:
///  The decision's words are that the setup sets the value on the clone
///  container and the clones inherit it,
///  so a child that did not follow its parent would leave every clone showing
///  its own snapshot directory;
///  the child carries no origin,
/// so the read is inheritance rather than a value
///  a clone copied from its origin snapshot.
fn snapdir_follows_the_container() {
    let probe: String = format!("{}/snapdir-probe", container_dataset());
    zfs(&["create", "-o", "mountpoint=legacy", probe.as_str()]);
    let inherited: git_policy_cli::doctor_zfs_route::SnapdirSetting = snapdir_of(probe.as_str());
    assert_eq!(
        inherited,
        git_policy_cli::doctor_zfs_route::SnapdirSetting::Word("disabled"),
        "a child below a disabled container inherits it: {inherited:?}"
    );
    zfs(&["set", "snapdir=visible", container_dataset().as_str()]);
    let followed: git_policy_cli::doctor_zfs_route::SnapdirSetting = snapdir_of(probe.as_str());
    assert_eq!(
        followed,
        git_policy_cli::doctor_zfs_route::SnapdirSetting::Word("visible"),
        "a child follows its parent's value: {followed:?}"
    );
    println!("DOCTOR-GUEST snapdir-follows-the-container");
    zfs(&["destroy", "-r", "-f", probe.as_str()]);
    zfs(&["set", "snapdir=disabled", container_dataset().as_str()]);
}

//endregion
