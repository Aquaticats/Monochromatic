//! What:
//!  Controls of the whole pass in its order:
//!  the readiness of this boot goes before any mount is touched,
//!  an idle clone's mount is unmounted and its desired state is left exactly
//!  as it was,
//!  a boot whose record proves nothing unmounts nothing,
//!  a configuration the boot pass could not use stops the pass,
//!  and the recorded folder's refusal stops it before any claim.
//! Why:
//!  The order is the design's and the whole point of the round:
//!  "Withdraw readiness and close admission to new ZED attaches before
//!  draining in-flight slot operations",
//!  and "Unmount the verified managed mounts only;
//!  do not change durable intent"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  A control reads a mount table it states, a walk and an unmount it states,
//!  a readiness record in a fixture folder and a configuration in a fixture
//!  file,
//!  so nothing here reads `/run`,
//!  `/etc` or a mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = runThePass(request);
//! expect(outcome.unmounted()).toBe(1);
//! ```

use crate::fs_mount_options::MountFacts;
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_mount::slot_identity_store::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_answer::{
    MountEnding,
    PassOutcome,
};
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_pass_test_doubles::{
    CLONE,
    MOUNT_ID,
    PARENT_MOUNT_ID,
    StatedCalls,
    StatedGuids,
    StatedHolders,
    StatedLookups,
    StatedMounts,
    StatedWalk,
    StatedWorktrees,
    clone_guid,
    fresh_lookup_of,
    identity_of,
    mount_row,
    scan_finding_one_holder,
};
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_places::ShutdownUnmountPlaces;
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_request::ShutdownUnmountRequest;
use crate::worktree_clone_shutdown_unmount_pass::shutdown_unmount_turn::run_the_pass;
use crate::worktree_clone_shutdown_unmount_pass::unmountable_mounts::slot_path_of;
use crate::zfs_mounted_dataset::mounted_dataset_instance::MountedDatasetInstance;
use std::collections::BTreeMap;
use std::path::{
    Path,
    PathBuf,
};

/// The boot every control runs in.
const BOOT: &str = "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b";

/// The container every control's configuration names.
const CONTAINER: &str = "zpcachyos/nosnap/git-worktree-clones";

/// The slot every control reads.
const SLOT: &str = "one";

/// This account's own number.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// A configuration file the boot unit could have written.
///
/// Why:
///  The pass reads the same file its `ExecStart` read,
///  so a control writes that file into its own fixture.
fn write_configuration(
    path: &Path,
    checkout: &Path,
) {
    let document: String = format!(
        "{{ \"cloneMounts\": {{ \"parent\": \"{CONTAINER}\", \"checkout\": \"{}\", \"checkoutDataset\": \"zpcachyos/nosnap/git-worktree-checkout\" }} }}\n",
        checkout.display()
    );
    std::fs::write(path, document).expect("the fixture configuration is written");
}

/// A ready record for this boot.
fn record(checkout: &Path) -> ReadinessRecord {
    return ReadinessRecord {
        boot: String::from(BOOT),
        pool_name: String::from("zpcachyos"),
        pool_state: ReadyState::Ready,
        checkout_path: checkout.display().to_string(),
        checkout_state: ReadyState::Ready,
        sweep_nonce: RequestNonce::parse("3".repeat(32).as_str()).ok(),
        sweep_state: SweepState::Finished,
        time: String::from("2026-11-27T20:00:00-05:00"),
    };
}

/// One control's world:
/// its places,
/// the checkout it names,
/// the slot inside it,
/// and the calls the pass makes.
struct World {
    /// The places the pass reads and writes.
    places: ShutdownUnmountPlaces,
    /// The checkout the configuration names.
    checkout: PathBuf,
    /// The identity the route recorded for the folder the slots hang in.
    recorded: CloneFolderIdentity,
    /// The slot's folder.
    slot: std::fs::File,
    /// The mount table the pass reads.
    mounts: StatedMounts,
    /// The staging route's answers.
    calls: StatedCalls,
    /// The holder scan's answer.
    holders: StatedHolders,
    /// The clone's GUID.
    guids: StatedGuids,
    /// The folder identity the store answers.
    worktrees: StatedWorktrees,
    /// The mount number a fresh walk reaches.
    walk_mount_id: u64,
}

/// One control's world at one fixture,
/// with the checkout,
/// the slot and every agreeing answer.
fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let checkout: PathBuf = root.join("checkout");
    let slot_path: PathBuf = slot_path_of(checkout.as_path(), SLOT);
    std::fs::create_dir_all(slot_path.as_path()).expect("the slot folder is made");
    let slot: std::fs::File =
        std::fs::File::open(slot_path.as_path()).expect("the slot folder opens");
    let recorded: CloneFolderIdentity = identity_of(checkout.as_path());
    let places: ShutdownUnmountPlaces = ShutdownUnmountPlaces {
        readiness: ReadinessFile::at(root.join("readiness"), this_uid()),
        configuration: root.join("configuration.jsonc"),
        claims: SlotClaimDirectory::at(root.join("claims"), this_uid()),
        store: PathBuf::from(SLOT_IDENTITY_STORE_PATH),
        boot_identity: root.join("boot_id"),
        proc_root: root.join("proc"),
        self_task: root.join("proc").join("self"),
    };
    std::fs::write(places.boot_identity.as_path(), format!("{BOOT}\n"))
        .expect("the boot identifier is written");
    write_configuration(places.configuration.as_path(), checkout.as_path());
    let mut rows: BTreeMap<u64, MountFacts> = BTreeMap::new();
    rows.insert(
        MOUNT_ID,
        mount_row(
            CLONE,
            slot_path.as_path(),
            Path::new("/"),
            "zfs",
            PARENT_MOUNT_ID,
        ),
    );
    return World {
        places,
        checkout,
        recorded: recorded.clone(),
        slot,
        mounts: StatedMounts {
            rows: vec![MountedDatasetInstance {
                dataset: String::from(CLONE),
                mount_point: slot_path.clone(),
                root: PathBuf::from("/"),
            }],
        },
        calls: StatedCalls {
            mount_id: MOUNT_ID,
            observed_path: slot_path.clone(),
            rows,
            unmount: Ok(()),
        },
        holders: StatedHolders {
            answer: Ok(Box::new(scan_finding_one_holder())),
        },
        guids: StatedGuids {
            answer: Ok(clone_guid()),
        },
        worktrees: StatedWorktrees {
            answer: Ok(recorded),
        },
        walk_mount_id: MOUNT_ID,
    };
}

/// One whole pass over this world,
/// with the lookup the world's own slot answers.
fn pass(world: &World) -> Result<PassOutcome, String> {
    let lookup: StatedLookups = StatedLookups {
        answer: Ok(Box::new(fresh_lookup_of(&world.slot, &world.recorded))),
    };
    let walk: StatedWalk = StatedWalk {
        slot: world
            .slot
            .try_clone()
            .expect("the fixture descriptor is copied"),
        look_again: world.walk_mount_id,
    };
    let request: ShutdownUnmountRequest<'_> = ShutdownUnmountRequest {
        places: &world.places,
        mounts: &world.mounts,
        calls: &world.calls,
        walk: &walk,
        lookups: &lookup,
        worktrees: &world.worktrees,
        guids: &world.guids,
        holders: &world.holders,
    };
    return run_the_pass(&request);
}

/// An idle clone's mount is unmounted,
/// and the readiness and the checkout are left as the pass found them.
#[test]
fn an_idle_clones_mount_is_unmounted_and_nothing_durable_changed() {
    let world: World = world("turn-idle");
    world
        .places
        .readiness
        .publish(&record(world.checkout.as_path()))
        .expect("the record is published");
    let before: Vec<u8> =
        std::fs::read(world.places.configuration.as_path()).expect("the configuration is read");
    let outcome: PassOutcome = pass(&world).expect("the pass runs");
    let summary: String = outcome.lines()[0].clone();
    assert!(
        summary.contains(&format!(
            "readiness for boot {BOOT} was withdrawn before any mount was touched"
        )),
        "the summary says the withdrawal came first, and this one says {summary}"
    );
    assert!(outcome.verified_nothing.is_none());
    assert_eq!(outcome.reached.len(), 1);
    assert_eq!(outcome.unmounted(), 1);
    assert_eq!(
        outcome.reached[0].ending,
        MountEnding::Unmounted {
            mount_id: MOUNT_ID
        }
    );
    assert_eq!(
        std::fs::read(world.places.configuration.as_path())
            .expect("the configuration is read again"),
        before,
        "the pass writes no configuration"
    );
    assert_eq!(
        world.places.readiness.read().expect("the directory reads"),
        None,
        "the pass withdrew the record before it unmounted"
    );
}

/// A boot whose record proves nothing unmounts nothing and says why.
#[test]
fn a_boot_that_proves_nothing_unmounts_nothing() {
    let world: World = world("turn-nothing");
    let outcome: PassOutcome = pass(&world).expect("a boot with no record is a pass");
    assert!(
        outcome.reached.is_empty(),
        "no mount is read on a boot that proved nothing"
    );
    let said: &str = outcome
        .verified_nothing
        .as_deref()
        .expect("the pass says what was missing");
    assert!(
        said.contains("published no readiness record"),
        "the sentence says what was missing, and this one says {said}"
    );
}

/// A configuration the boot pass could not use stops the pass,
/// with the readiness already withdrawn.
#[test]
fn an_unusable_configuration_stops_the_pass_after_the_withdrawal() {
    let world: World = world("turn-bad-configuration");
    world
        .places
        .readiness
        .publish(&record(world.checkout.as_path()))
        .expect("the record is published");
    std::fs::write(world.places.configuration.as_path(), "{ \"cloneMounts\": ")
        .expect("the broken document is written");
    let said: String = pass(&world).expect_err("a broken document stops the pass");
    assert!(
        said.contains("no mount was unmounted and no durable intent was changed"),
        "the refusal says what it cost, and this one says {said}"
    );
    assert_eq!(
        world.places.readiness.read().expect("the directory reads"),
        None,
        "readiness was withdrawn before the configuration was read"
    );
}

/// A record and a configuration naming different places stop the pass.
#[test]
fn a_record_naming_another_checkout_stops_the_pass() {
    let world: World = world("turn-checkout-mismatch");
    let mut published: ReadinessRecord = record(world.checkout.as_path());
    published.checkout_path = String::from("/home/user/elsewhere");
    world
        .places
        .readiness
        .publish(&published)
        .expect("the record is published");
    let said: String = pass(&world).expect_err("two checkouts stop the pass");
    assert!(
        said.contains("names checkout /home/user/elsewhere"),
        "the refusal names the record's checkout, and this one says {said}"
    );
}

/// A store whose recorded folder cannot be read stops the pass before a claim.
#[test]
fn an_unreadable_recorded_folder_stops_the_pass() {
    let mut world: World = world("turn-store-refused");
    world
        .places
        .readiness
        .publish(&record(world.checkout.as_path()))
        .expect("the record is published");
    world.worktrees = StatedWorktrees {
        answer: Err(String::from(
            "the store holds no `.parent` record, so no slot hangs in a folder this route recorded",
        )),
    };
    let said: String = pass(&world).expect_err("an unreadable store stops the pass");
    assert!(
        said.contains("no `.parent` record"),
        "the store's own sentence travels back, and this one says {said}"
    );
    assert!(
        world.places.claims.claim(SLOT).is_ok(),
        "no claim was taken, so the slot was free the whole time"
    );
}
