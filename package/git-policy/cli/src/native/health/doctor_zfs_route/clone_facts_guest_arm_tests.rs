//! What:
//!  The gatherer's guest arm:
//!  one repository holding six managed clones,
//!  each gathered from the live machine with [`LiveCloneFactsReads`],
//!  and each asserted as its own group of lines.
//! Why:
//!  The facts read `/dev/zfs`,
//!  a real pool's property replies,
//!  the plan folder,
//!  the cache's staging mounts and the all-namespace scan,
//!  so no host control may show them
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "VM measurements").
//!  The six clones are the states the plan names:
//!  one mounted at its slot,
//!  one unmounted,
//!  one a removal staged under the cache,
//!  one whose removal stopped after its first write,
//!  one whose origin snapshot carries the deferred-destroy mark,
//!  and one a task in another mount namespace holds,
//!  which the holders line names by namespace and representative task.
//!  The stopped removal's plan is published by the removal's own writer
//!  ([`super::super::worktree_clone_removal_proof_and_plan::plan_publication`]),
//!  as the guide requires,
//!  and the intent beside it is the property the
//!  removal writes,
//!  so the arm reads a state the route makes and not one typed by hand.
//!  The arm also proves the gatherer writes nothing:
//!  every record and the plan's bytes are read before and after a gather and
//!  asserted equal.
//!  It prints `SKIPPED` where its own variable names no guest,
//!  and every claim is a marker line its driver requires,
//!  so a section that silently stopped running fails the run.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = LivePoolFixture.fromEnvironment('the gatherer', variables);
//! installLayout(fixture);
//! for (const slot of SLOTS) console.log(`CLONE-FACTS-GUEST ${slot}-${stateOf(slot)}`);
//! ```

/// The checks the arm judges.
use super::blocked_removal_checks::check_blocked_removal;
use super::check_outcome::CheckOutcome;
/// The gatherer,
/// the seam it reads through and the live answers it is given.
use super::clone_facts_gathering::gather_clone_facts;
use super::clone_facts_live_reads::LiveCloneFactsReads;
use super::clone_facts_reads::CloneFactsRequest;
use super::clone_origin_checks::check_clone_origin;
use super::clone_retention_checks::check_clone_retention;
use super::clone_snapshot_checks::check_clone_snapshots;
use super::clone_state_checks::check_clone_state;
use super::holder_checks::check_holders;
use super::managed_lock_checks::check_managed_lock;
use super::registration_checks::check_registration;
use super::removal_plan_checks::PlanFileReading;
/// The facts one gather answers.
use super::route_checks::CloneFacts;
/// One record's bytes,
/// the plan writer and the property a removal's intent is written in.
use crate::worktree_clone_binding_record::binding_encoding::{
    FolderIdentity,
    bytes_as_hexadecimal,
    path_as_hexadecimal,
};
use crate::worktree_clone_binding_record::binding_inventory::{
    EntryKind,
    InventoryEntry,
    InventoryKind,
    InventoryRecord,
};
/// The header and the home every record is filed under.
use crate::worktree_clone_binding_record::binding_words::BINDING_HEADER;
use crate::worktree_clone_binding_record::registration_home::{
    binding_name,
    home_folder,
    origin_inventory_name,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_mount_protocol::{
    DESIRED_STATE_PROPERTY,
    MountOperation,
};
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanFacts,
};
/// The live pool and repository the removal's own arms build,
/// for this arm's six clones.
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_live_fixtures::{
    LivePoolFixture,
    PoolVariables,
};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::whole_inventory;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{
    LiveRemovalReads,
    RemovalPoolReads,
};
use crate::worktree_clone_removal_proof_and_plan::started_moment::now_with_offset;
use crate::worktree_clone_unmount::staging_folder_name::staging_folder_name;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
};

/// The variable naming the guest this arm runs in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_CLONE_FACTS_GUEST";

/// The variable naming the pool this arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_CLONE_FACTS_POOL";

/// The variable naming this guest's OpenZFS prefix.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_CLONE_FACTS_PREFIX";

/// The variable naming the folder this arm's fixture lives under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_CLONE_FACTS_ROOT";

/// The folder the fixture lives in when the driver names none.
const DEFAULT_ROOT: &str = "/home/zfs-clone-facts-fixture";

/// The pool this arm builds when the driver names none.
const DEFAULT_POOL: &str = "zfsclonefacts";

/// The one intent and plan nonce this arm writes.
const NONCE: &str = "0123456789abcdef0123456789abcdef";

/// The six managed clones,
/// one per state the plan's report names.
const THE_MOUNTED: &str = "mounted";
const THE_UNMOUNTED: &str = "unmounted";
const THE_STAGED: &str = "staged";
const THE_STOPPED: &str = "stopped";
const THE_MARKED: &str = "marked";
const THE_HELD: &str = "held";

/// Every slot this arm manages,
/// in the order its markers print.
const SLOTS: &[&str] = &[
    THE_MOUNTED,
    THE_UNMOUNTED,
    THE_STAGED,
    THE_STOPPED,
    THE_MARKED,
    THE_HELD,
];

/// The clone's own snapshot each slot holds,
/// which the removal's plan records.
const OWN_SNAPSHOT: &str = "worktree-own";

/// How long the task in the other mount namespace holds the clone.
/// Why:
///  The scan runs while the holder stands,
///  and the arm ends the holder whatever else it does,
///  so the length is longer than one gather and no longer than the guest.
const HOLD_SECONDS: &str = "900";

//region The fixture

/// The variables this arm's guest and pool are named by.
fn variables() -> PoolVariables {
    return PoolVariables {
        guest: GUEST_VARIABLE,
        pool: POOL_VARIABLE,
        prefix: PREFIX_VARIABLE,
        root: ROOT_VARIABLE,
        default_root: DEFAULT_ROOT,
        default_pool: DEFAULT_POOL,
    };
}

/// The cache the staged clone is mounted under.
fn cache(fixture: &LivePoolFixture) -> PathBuf {
    return fixture.root.join("cache");
}

/// One slot's full path inside the repository.
fn slot_path(
    fixture: &LivePoolFixture,
    slot: &str,
) -> PathBuf {
    return fixture
        .root
        .join("repo")
        .join(".claude")
        .join("worktrees")
        .join(slot);
}

/// One slot's registration directory.
fn registration(
    fixture: &LivePoolFixture,
    slot: &str,
) -> PathBuf {
    return fixture.common().join("worktrees").join(slot);
}

/// One slot's clone dataset.
fn clone_of(
    fixture: &LivePoolFixture,
    slot: &str,
) -> String {
    return format!("{}/{slot}", fixture.named("clones"));
}

/// The snapshot the marked slot was cloned from,
/// which carries the deferred-destroy mark.
fn marked_origin(fixture: &LivePoolFixture) -> String {
    return format!("{}@marked-{THE_MARKED}", fixture.checkout());
}

/// What:
///  The six clones,
///  their folders,
///  their records and the states a later gather reads.
fn install_the_layout(fixture: &LivePoolFixture) {
    fixture.build(&[]);
    std::fs::create_dir_all(cache(fixture).as_path()).expect("the cache");
    for slot in SLOTS {
        std::fs::create_dir_all(slot_path(fixture, slot).as_path()).expect("the slot folder");
        std::fs::create_dir_all(registration(fixture, slot).as_path()).expect("the registration");
    }
    // The marked slot is cloned from a snapshot of its own, so the mark is on
    // its origin alone and no other clone's line moves with it.
    fixture.run("zfs", &["snapshot", marked_origin(fixture).as_str()]);
    for slot in SLOTS {
        let origin: String = match *slot {
            THE_MARKED => marked_origin(fixture),
            _ => fixture.origin(),
        };
        let clone: String = clone_of(fixture, slot);
        // The shared fixture's own clone stands at its own name, so it is
        // destroyed and rebuilt under this arm's slot names.
        if fixture.exists(clone.as_str()) {
            fixture.run("zfs", &["destroy", clone.as_str()]);
        }
        fixture.run(
            "zfs",
            &[
                "clone",
                "-o",
                "mountpoint=legacy",
                origin.as_str(),
                clone.as_str(),
            ],
        );
        fixture.run(
            "zfs",
            &["snapshot", format!("{clone}@{OWN_SNAPSHOT}").as_str()],
        );
        write_the_record(fixture, slot, origin.as_str());
    }
    mount_the_states(fixture);
}

/// What one slot's add recorded,
/// read from the live objects as the add would have read them.
fn recorded_of(
    fixture: &LivePoolFixture,
    slot: &str,
    origin: &str,
) -> RecordedBinding {
    let clone: String = clone_of(fixture, slot);
    let (clone_guid, clone_createtxg) = fixture.identity(clone.as_str());
    let (origin_dataset_guid, _) = fixture.identity(fixture.checkout().as_str());
    let (origin_snapshot_guid, origin_snapshot_createtxg) = fixture.identity(origin);
    let (pool_guid, _) = fixture.identity(fixture.pool.as_str());
    return RecordedBinding {
        slot: String::from(slot),
        slot_path: slot_path(fixture, slot),
        pool: fixture.pool.clone(),
        pool_guid: a_guid(pool_guid),
        clone_dataset: clone,
        clone_guid: a_guid(clone_guid),
        clone_createtxg,
        origin_dataset: fixture.checkout(),
        origin_dataset_guid: a_guid(origin_dataset_guid),
        origin_snapshot: String::from(origin),
        origin_snapshot_guid: a_guid(origin_snapshot_guid),
        origin_snapshot_createtxg,
        registration_id: Vec::from(slot.as_bytes()),
        // The record's registration identity is written from the folder once
        // it stands, so a placeholder here keeps the live read in one place.
        registration_identity: FolderIdentity::decode("1:1:1").expect("a decodable identity"),
    };
}

/// One live number as this route names a GUID.
fn a_guid(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str())
        .expect("the kernel reports a canonical GUID");
}

/// What:
///  The records an add would have left for one slot:
///  its binding record and its origin-side inventory.
/// Why:
///  The gatherer reads the clone,
///  its origin and its registration from these bytes,
///  so a slot without them answers unknown instead of the state the arm means
///  to show.
fn write_the_record(
    fixture: &LivePoolFixture,
    slot: &str,
    origin: &str,
) {
    let recorded: RecordedBinding = recorded_of(fixture, slot, origin);
    let registration_identity: FolderIdentity =
        FolderIdentity::of_folder(registration(fixture, slot).as_path())
            .expect("the registration's identity");
    let slot_identity: FolderIdentity =
        FolderIdentity::of_folder(slot_path(fixture, slot).as_path()).expect("the slot's identity");
    let home: PathBuf = home_folder(fixture.common().as_path());
    std::fs::create_dir_all(home.as_path()).expect("the binding home");
    let lines: Vec<String> = Vec::from([
        String::from(BINDING_HEADER),
        String::from("claim=1"),
        format!("nonce={NONCE}"),
        format!("request=branch {}", bytes_as_hexadecimal(b"main")),
        String::from("phase=filled"),
        format!("checkoutDataset={}", recorded.origin_dataset),
        format!(
            "checkoutDatasetGuid={}",
            recorded.origin_dataset_guid.file_name()
        ),
        format!(
            "repository={}",
            path_as_hexadecimal(fixture.common().as_path())
        ),
        format!(
            "repositoryIdentity={}",
            FolderIdentity::of_folder(fixture.common().as_path())
                .expect("the common directory's identity")
                .encode()
        ),
        format!(
            "slot={}",
            path_as_hexadecimal(slot_path(fixture, slot).as_path())
        ),
        format!("slotIdentity={}", slot_identity.encode()),
        format!(
            "registration={}",
            path_as_hexadecimal(registration(fixture, slot).as_path())
        ),
        format!("registrationIdentity={}", registration_identity.encode()),
        format!("pool={}", recorded.pool),
        format!("poolGuid={}", recorded.pool_guid.file_name()),
        format!("originSnapshot={}", recorded.origin_snapshot),
        format!(
            "originSnapshotGuid={}",
            recorded.origin_snapshot_guid.file_name()
        ),
        format!(
            "originSnapshotCreatetxg={}",
            recorded.origin_snapshot_createtxg
        ),
        format!("originDataset={}", recorded.origin_dataset),
        format!(
            "originDatasetGuid={}",
            recorded.origin_dataset_guid.file_name()
        ),
        format!("cloneDataset={}", recorded.clone_dataset),
        format!("cloneGuid={}", recorded.clone_guid.file_name()),
        format!("cloneCreatetxg={}", recorded.clone_createtxg),
        format!(
            "pointer={}",
            bytes_as_hexadecimal(
                format!("gitdir: {}\n", registration(fixture, slot).display()).as_bytes()
            )
        ),
    ]);
    std::fs::write(
        home.join(binding_name(slot)),
        format!("{}\n", lines.join("\n")),
    )
    .expect("the binding record");
    let inventory: InventoryRecord = the_origin_inventory(fixture, &recorded);
    std::fs::write(home.join(origin_inventory_name(slot)), inventory.encode())
        .expect("the origin inventory");
    write_a_complete_registration(fixture, slot);
    // The managed Git lock the add's own writer installs beside the
    // registration, so a healthy slot is the state the route leaves and not
    // the one a killed add leaves.
    let answer: crate::worktree_clone_managed_lock::InstallAnswer =
        crate::worktree_clone_managed_lock::write_the_lost_lock(
            registration(fixture, slot).as_path(),
            slot,
            &RequestNonce::parse(NONCE).expect("the arm's nonce"),
        );
    assert!(
        matches!(
            answer,
            crate::worktree_clone_managed_lock::InstallAnswer::Wrote { .. }
        ),
        "the arm's own lock is written for `{slot}`: {answer:?}"
    );
}

/// The origin-side inventory the add recorded for one slot,
/// from the checkout's own snapshots and bookmarks.
fn the_origin_inventory(
    fixture: &LivePoolFixture,
    recorded: &RecordedBinding,
) -> InventoryRecord {
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let mut entries: Vec<InventoryEntry> = Vec::new();
    for snapshot in reads
        .snapshots_of(fixture.checkout().as_str(), recorded.origin_dataset_guid)
        .expect("the checkout's snapshots")
    {
        if snapshot.createtxg <= recorded.origin_snapshot_createtxg {
            entries.push(InventoryEntry {
                kind: EntryKind::Snapshot,
                guid: snapshot.guid,
                createtxg: snapshot.createtxg,
                name: snapshot.name,
            });
        }
    }
    for bookmark in reads
        .bookmarks_of(fixture.checkout().as_str(), recorded.origin_dataset_guid)
        .expect("the checkout's bookmarks")
    {
        entries.push(InventoryEntry {
            kind: EntryKind::Bookmark,
            guid: bookmark.guid,
            createtxg: bookmark.createtxg,
            name: bookmark.name,
        });
    }
    return InventoryRecord {
        kind: InventoryKind::Origin,
        pool_guid: recorded.pool_guid,
        dataset: recorded.origin_dataset.clone(),
        dataset_guid: recorded.origin_dataset_guid,
        bounds: recorded.origin_snapshot.clone(),
        bounds_guid: recorded.origin_snapshot_guid,
        bounds_createtxg: recorded.origin_snapshot_createtxg,
        entries,
    };
}

/// A registration Git lists the worktree through,
/// as the add's Git step writes the pair.
fn write_a_complete_registration(
    fixture: &LivePoolFixture,
    slot: &str,
) {
    let registration: PathBuf = registration(fixture, slot);
    let slot_git: PathBuf = slot_path(fixture, slot).join(".git");
    std::fs::write(
        registration.join("gitdir"),
        format!("{}\n", slot_git.display()),
    )
    .expect("the registration's gitdir");
    std::fs::write(slot_git, format!("gitdir: {}\n", registration.display()))
        .expect("the slot's .git");
}

/// What:
///  The six states the plan names,
///  each made by the command that makes it on a person's machine.
/// Why:
///  A state typed into a record would prove the arm's own text and not the
///  route,
///  so every one of them is a mount,
///  a property,
///  a plan the removal's writer publishes,
///  or a mark the pool's own request sets.
fn mount_the_states(fixture: &LivePoolFixture) {
    let mounted: String = clone_of(fixture, THE_MOUNTED);
    fixture.run(
        "zfs",
        &[
            "set",
            "-u",
            format!("mountpoint={}", slot_path(fixture, THE_MOUNTED).display()).as_str(),
            mounted.as_str(),
        ],
    );
    fixture.run("zfs", &["mount", mounted.as_str()]);
    stage_the_clone(fixture, THE_STAGED);
    stop_the_removal(fixture, THE_STOPPED);
    fixture.run("zfs", &["destroy", "-d", marked_origin(fixture).as_str()]);
}

/// One clone mounted under the cache,
/// which is the state a removal's staging leaves.
fn stage_the_clone(
    fixture: &LivePoolFixture,
    slot: &str,
) {
    let clone: String = clone_of(fixture, slot);
    let (guid, _) = fixture.identity(clone.as_str());
    let folder: PathBuf = cache(fixture).join(staging_folder_name(a_guid(guid)));
    std::fs::create_dir_all(folder.as_path()).expect("the staging folder");
    fixture.run(
        "zfs",
        &[
            "set",
            "-u",
            format!("mountpoint={}", folder.display()).as_str(),
            clone.as_str(),
        ],
    );
    fixture.run("zfs", &["mount", clone.as_str()]);
}

/// One clone whose removal stopped after its first write:
/// a published plan and the `unmount` intent the removal sets beside it.
fn stop_the_removal(
    fixture: &LivePoolFixture,
    slot: &str,
) {
    let recorded: RecordedBinding = recorded_of(fixture, slot, fixture.origin().as_str());
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
    let inventory = whole_inventory(&reads, recorded.clone_dataset.as_str(), recorded.clone_guid)
        .expect("the clone's inventory");
    let own: Vec<crate::worktree_clone_removal_proof_and_plan::plan_record::PlanSnapshot> =
        inventory.own_snapshots(recorded.clone_createtxg);
    assert!(
        !own.is_empty(),
        "the arm gives the stopped slot one snapshot of its own"
    );
    let facts: PlanFacts = PlanFacts {
        run: RequestNonce::parse(NONCE).expect("the arm's nonce"),
        owner: String::from("1:linux:1"),
        started: now_with_offset(),
        clone_dataset: recorded.clone_dataset.clone(),
        clone_guid: recorded.clone_guid,
        clone_createtxg: recorded.clone_createtxg,
        origin_dataset: recorded.origin_dataset.clone(),
        origin_dataset_guid: recorded.origin_dataset_guid,
        origin_snapshot: recorded.origin_snapshot.clone(),
        origin_snapshot_guid: recorded.origin_snapshot_guid,
        origin_snapshot_createtxg: recorded.origin_snapshot_createtxg,
        registration_id: Vec::from(slot.as_bytes()),
        admin: crate::fs_beneath::identity_of(
            &std::fs::File::open(registration(fixture, slot)).expect("the registration opens"),
        )
        .expect("the registration's identity")
        .encode(),
        target: slot_path(fixture, slot),
        unmount_nonce: RequestNonce::parse(NONCE).expect("the arm's nonce"),
        unmount_mount_id: None,
    };
    let folder: PlanFolder =
        PlanFolder::open_or_create(fixture.common().as_path()).expect("the plan folder");
    let published: PathBuf = folder
        .publish(&DestructionPlan::first_write(facts, own.as_slice()))
        .expect("the plan is published");
    assert!(published.exists(), "the plan's file stands");
    fixture.run(
        "zfs",
        &[
            "set",
            format!(
                "{DESIRED_STATE_PROPERTY}=v1:{NONCE}:{}",
                MountOperation::Unmount.word()
            )
            .as_str(),
            recorded.clone_dataset.as_str(),
        ],
    );
}

/// What:
///  One task in a mount namespace of its own holding one clone,
///  which is the holder a removal elsewhere must see.
/// Why:
///  A mount a second namespace holds is the state the design's holder scan
///  names by namespace and representative task,
///  and the arm owns the child that holds it so it can end it on every path.
fn hold_the_clone(
    fixture: &LivePoolFixture,
    slot: &str,
) -> Child {
    let clone: String = clone_of(fixture, slot);
    let folder: PathBuf = fixture.root.join("held");
    std::fs::create_dir_all(folder.as_path()).expect("the holder's folder");
    let script: String = format!(
        "mount -t zfs {clone} {} && exec sleep {HOLD_SECONDS}",
        folder.display()
    );
    let mut command: Command = Command::new("unshare");
    command
        .args(["--mount", "sh", "-c", script.as_str()])
        .env("PATH", fixture.search_path());
    // The guest has no service manager and this helper is the arm's own, so it
    // is started by the one shape every helper of this crate is started with.
    return crate::test_support::spawn_owned(&mut command).expect("the guest starts `unshare`");
}

//endregion

//region The arm

/// The gatherer's six groups,
/// each read from the live machine and each asserted.
#[test]
fn zfs_doctor_clone_facts_guest() {
    let Some(fixture) = LivePoolFixture::from_environment("the clone facts", &variables()) else {
        return;
    };
    install_the_layout(&fixture);
    let reads: LiveCloneFactsReads = LiveCloneFactsReads::open();
    let records: Vec<Vec<u8>> = the_records(&fixture);
    for slot in SLOTS {
        let request: CloneFactsRequest = CloneFactsRequest {
            common_dir: fixture.common(),
            slot: String::from(*slot),
            slot_path: slot_path(&fixture, slot),
            cache: cache(&fixture),
            caller_task: std::process::id(),
        };
        let facts: CloneFacts = gather_clone_facts(&request, &reads);
        assert_eq!(
            facts.state.readings.slot, *slot,
            "the group is the slot it was asked for"
        );
        assert_eq!(
            facts.plan.clone,
            clone_of(&fixture, slot),
            "the group names the slot's own clone"
        );
        match *slot {
            THE_MOUNTED => the_mounted(&fixture, &facts),
            THE_UNMOUNTED => the_unmounted(&facts),
            THE_STAGED => the_staged(&fixture, &facts),
            THE_STOPPED => the_stopped(&facts),
            THE_MARKED => the_marked(&facts),
            THE_HELD => the_held(&fixture),
            _ => panic!("the arm has no state for `{slot}`"),
        }
        println!("CLONE-FACTS-GUEST {slot}-group-reads");
    }
    assert_eq!(the_records(&fixture), records, "a gather wrote no record");
    println!("CLONE-FACTS-GUEST gather-writes-nothing");
    fixture.teardown();
}

/// Every record's bytes,
/// and every plan's,
/// as the arm read them before the gathers.
fn the_records(fixture: &LivePoolFixture) -> Vec<Vec<u8>> {
    let home: PathBuf = home_folder(fixture.common().as_path());
    let plans: PathBuf = crate::worktree_clone_binding_record::registration_home::plans_folder(
        fixture.common().as_path(),
    );
    let mut found: Vec<Vec<u8>> = Vec::new();
    for folder in [home, plans] {
        let Ok(entries) = std::fs::read_dir(folder.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            if let Ok(bytes) = std::fs::read(entry.path()) {
                found.push(bytes);
            }
        }
    }
    return found;
}

/// The clone mounted at its slot passes its state and registration lines.
fn the_mounted(
    fixture: &LivePoolFixture,
    facts: &CloneFacts,
) {
    let state: CheckOutcome = check_clone_state(&facts.state);
    passed(&state, "a clone mounted at its slot");
    assert!(
        matches!(
            facts.state.readings.slot_mount,
            crate::worktree_clone_slot_state::slot_state_mounts::SlotMount::Something { .. }
        ),
        "the slot's own mount is read: {:?}",
        facts.state.readings.slot_mount
    );
    let _ = fixture;
    passed(&check_registration(&facts.registration), "its registration");
    passed(&check_managed_lock(&facts.lock), "its managed lock");
    println!("CLONE-FACTS-GUEST mounted-clone-passes");
}

/// The clone nothing has mounted passes and answers no holder.
fn the_unmounted(facts: &CloneFacts) {
    let state: CheckOutcome = check_clone_state(&facts.state);
    let CheckOutcome::Pass {
        detail,
    } = &state
    else {
        panic!("a known unmounted clone passes: {state:?}");
    };
    assert!(detail.contains("unmounted"), "{detail}");
    assert!(
        facts.holders.holders.is_empty() && facts.holders.incomplete.is_empty(),
        "nothing holds it: {:?}",
        facts.holders
    );
    passed(&check_holders(&facts.holders), "its holders line");
    println!("CLONE-FACTS-GUEST unmounted-clone-passes");
}

/// The clone a removal staged under the cache is a staging finding,
/// and its mount is the caller's own,
/// so the blocked line names it.
fn the_staged(
    fixture: &LivePoolFixture,
    facts: &CloneFacts,
) {
    let state: CheckOutcome = check_clone_state(&facts.state);
    let CheckOutcome::Finding {
        field,
        detail,
    } = &state
    else {
        panic!("a staged clone is a state finding: {state:?}");
    };
    assert_eq!(field, "staging");
    assert!(detail.contains("staging folder"), "{detail}");
    assert_eq!(facts.state.staging_binds.len(), 1, "one staging bind");
    let (guid, _) = fixture.identity(clone_of(fixture, THE_STAGED).as_str());
    assert_eq!(
        facts.state.staging_binds[0].mount_point,
        cache(fixture).join(staging_folder_name(a_guid(guid)))
    );
    let holders: CheckOutcome = check_holders(&facts.holders);
    let CheckOutcome::Finding {
        detail: held,
        ..
    } = &holders
    else {
        panic!("this process holds the staged clone: {holders:?}");
    };
    assert!(
        held.contains(std::process::id().to_string().as_str()),
        "{held}"
    );
    println!("CLONE-FACTS-GUEST staged-clone-is-a-staging-finding");
}

/// The clone whose removal stopped after its first write stands with a plan,
/// and its line names the snapshot the plan recorded.
fn the_stopped(facts: &CloneFacts) {
    assert!(
        facts.snapshots.plan_stands && facts.blocked.plan_stands,
        "the plan stands"
    );
    let plan: CheckOutcome = super::removal_plan_checks::check_removal_plan(&facts.plan);
    let CheckOutcome::Finding {
        field,
        detail,
    } = &plan
    else {
        panic!("a standing plan is a finding: {plan:?}");
    };
    assert_eq!(field, "plan");
    assert!(detail.contains(OWN_SNAPSHOT), "{detail}");
    assert!(detail.contains("head="), "{detail}");
    assert!(matches!(
        facts.plan.reading,
        PlanFileReading::Standing { .. }
    ));
    println!("CLONE-FACTS-GUEST stopped-removal-stands-with-its-plan");
}

/// The clone whose origin snapshot is marked is an origin finding,
/// and the mark reads on the clone that names it.
fn the_marked(facts: &CloneFacts) {
    let origin: CheckOutcome = check_clone_origin(&facts.origin);
    let CheckOutcome::Finding {
        field,
        detail,
    } = &origin
    else {
        panic!("a marked origin is a finding: {origin:?}");
    };
    assert_eq!(field, "defer_destroy");
    assert!(detail.contains("cannot be cleared"), "{detail}");
    passed(&check_clone_snapshots(&facts.snapshots), "its snapshots");
    passed(&check_clone_retention(&facts.retention), "its retention");
    println!("CLONE-FACTS-GUEST marked-origin-is-a-finding");
}

/// The clone a task in another mount namespace holds is named there,
/// by the namespace the kernel gives and the task that read its table.
fn the_held(fixture: &LivePoolFixture) {
    let mut holder: Child = hold_the_clone(fixture, THE_HELD);
    // The scan reads `/proc`, so the holder's mount is waited for rather than
    // slept past: the arm asks the same question the report asks until the
    // namespace answers with it.
    let request: CloneFactsRequest = CloneFactsRequest {
        common_dir: fixture.common(),
        slot: String::from(THE_HELD),
        slot_path: slot_path(fixture, THE_HELD),
        cache: cache(fixture),
        caller_task: std::process::id(),
    };
    let reads: LiveCloneFactsReads = LiveCloneFactsReads::open();
    let mut waited: u32 = 0;
    let mut last: CloneFacts = gather_clone_facts(&request, &reads);
    while waited < 600 {
        let facts: CloneFacts = gather_clone_facts(&request, &reads);
        if let Some(held) = facts
            .holders
            .holders
            .iter()
            .find(|held| return held.namespace != a_namespace_of_this_process())
        {
            assert!(
                held.representative_pid > 0,
                "the holder names a live task: {held:?}"
            );
            assert!(
                held.mount_point.starts_with(fixture.root.as_path()),
                "{held:?}"
            );
            let holders: CheckOutcome = check_holders(&facts.holders);
            let CheckOutcome::Finding {
                detail,
                ..
            } = &holders
            else {
                panic!("a holder is a finding: {holders:?}");
            };
            assert!(detail.contains(held.namespace.as_str()), "{detail}");
            // The holder is this arm's own child, so the kernel names it in
            // the caller's session: the line a removal spares must carry it.
            assert_eq!(
                facts.blocked.in_the_callers_session,
                Vec::from([held.representative_pid]),
                "the holder is in this arm's own session"
            );
            end_the_holder(&mut holder);
            println!("CLONE-FACTS-GUEST held-clone-names-its-namespace-and-task");
            return;
        }
        passed(&check_blocked_removal(&facts.blocked), "nothing is blocked");
        last = facts;
        waited += 1;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    end_the_holder(&mut holder);
    panic!(
        "no namespace held the held slot's clone within 60 s: {:?}",
        last.holders
    );
}

/// This process's own mount namespace,
/// as the kernel prints it,
/// so the arm tells its own mounts from another namespace's.
fn a_namespace_of_this_process() -> String {
    return crate::mount_namespace_scan::namespace_target(Path::new("/proc/self/ns/mnt"))
        .unwrap_or_else(|| return String::from("mnt:[self]"));
}

/// End the task that held the clone,
/// and wait for it,
/// so the guest holds no mount when the next state is read.
fn end_the_holder(holder: &mut Child) {
    let _ = holder.kill();
    let _ = holder.wait();
}

/// One line passed,
/// or the panic that prints what it answered instead.
fn passed(
    outcome: &CheckOutcome,
    what: &str,
) {
    assert!(
        matches!(outcome, CheckOutcome::Pass { .. }),
        "{what}: {outcome:?}"
    );
}

//endregion
