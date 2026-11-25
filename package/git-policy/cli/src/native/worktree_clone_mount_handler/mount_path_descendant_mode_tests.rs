//! What:
//!  Controls of the `mount-path-descendant` mode's proofs:
//!  the record it reads,
//!  the kernel's facts about the checkout and the dataset,
//!  the mount table's answers,
//!  every refusal that stands before a mount,
//!  and the two answers a proved state gives.
//! Why:
//!  No control on this host may mount a ZFS dataset,
//!  so everything the mode decides before the walk and the mount is held here
//!  against scripted reads and a fixture record,
//!  and the walk and the mount are reached only with a plan these controls
//!  prove.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(planWith(places, { dataset: 'zpcachyos/nosnap/output' }, true, 0)).toEqual({ kind: 'already-mounted', ... });
//! ```

use super::{
    ATIME_PROPERTY, CANMOUNT_PROPERTY, ENCRYPTION_PROPERTY, INCONSISTENT_PROPERTY, KEY_STATUS_PROPERTY, MOUNTPOINT_PROPERTY, OVERLAY_PROPERTY, READONLY_PROPERTY, RECEIVE_RESUME_TOKEN_PROPERTY, REDACTED_PROPERTY, RELATIME_PROPERTY, SETUID_PROPERTY, ZONED_PROPERTY, Answer, MountPathDescendantFailure, MountPlaces, plan_with,
};
use crate::worktree_clone_mount_handler::handler_mode_arguments::MountPathDescendantRequest;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild, ReadFailure};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{MountHoldingAPath, MountedDatasetInstance, MountedDatasets};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, DmuObjsetStats, NvList};
use std::path::{Path, PathBuf};

/// The checkout's own dataset.
const CHECKOUT_DATASET: &str = "zpcachyos/nosnap/monochromatic-git-checkout";

/// The container beside it.
const CONTAINER: &str = "zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets";

/// The container's own mountpoint.
const CONTAINER_MOUNTPOINT: &str = "/var/lib/monochromatic-git-policy-cli/clones";

/// The nested dataset the mode places.
const NESTED: &str = "zpcachyos/nosnap/vm-builder-output";

/// The checkout's GUID,
/// which the record holds.
const CHECKOUT_GUID: u64 = 1_234_567_890_123_456_789;

/// One property as OpenZFS writes it.
fn pair_text(value: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    pair.add_str("source", "local");
    return pair;
}

/// One number property in the same shape.
fn pair_number(value: u64) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_u64("value", value);
    pair.add_str("source", "local");
    return pair;
}

/// The statistics of one dataset.
fn stats(guid: u64) -> DmuObjsetStats {
    return DmuObjsetStats { dds_num_clones: 0, dds_creation_txg: 9, dds_guid: guid, dds_type: 2, dds_is_snapshot: 0, dds_inconsistent: 0, dds_redacted: 0, dds_origin: [0; 256], dds_flags: 0 };
}

/// What one dataset's reply holds,
/// which each case edits to make its refusal.
#[derive(Clone)]
struct Reply {
    /// The dataset's name.
    name: String,
    /// Its GUID.
    guid: u64,
    /// Its own mountpoint.
    mountpoint: String,
    /// Its `canmount`.
    canmount: String,
    /// Its `zoned`.
    zoned: String,
    /// Its `redacted` count.
    redacted: u64,
    /// Its `inconsistent` flag.
    inconsistent: u64,
    /// Its `receive_resume_token`.
    receive_resume_token: String,
    /// Its `encryption`.
    encryption: String,
    /// Its `keystatus`.
    keystatus: String,
    /// Its `atime`.
    atime: String,
    /// Its `relatime`.
    relatime: String,
    /// Its `readonly`.
    readonly: String,
    /// Its `setuid`.
    setuid: String,
    /// Its `overlay`.
    overlay: String,
}

impl Reply {
    /// A dataset that every proof accepts.
    fn of(name: &str, guid: u64, mountpoint: &str) -> Reply {
        return Reply {
            name: String::from(name),
            guid,
            mountpoint: String::from(mountpoint),
            canmount: String::from("on"),
            zoned: String::from("off"),
            redacted: 0,
            inconsistent: 0,
            receive_resume_token: String::new(),
            encryption: String::from("off"),
            keystatus: String::from("-"),
            atime: String::from("off"),
            relatime: String::from("off"),
            readonly: String::from("off"),
            setuid: String::from("on"),
            overlay: String::from("off"),
        };
    }

    /// The dataset's reply as the kernel writes it.
    fn properties(&self) -> NvList {
        let mut properties: NvList = NvList::new();
        properties.add_nvlist(MOUNTPOINT_PROPERTY, pair_text(self.mountpoint.as_str()));
        properties.add_nvlist(CANMOUNT_PROPERTY, pair_text(self.canmount.as_str()));
        properties.add_nvlist(ZONED_PROPERTY, pair_text(self.zoned.as_str()));
        properties.add_nvlist(REDACTED_PROPERTY, pair_number(self.redacted));
        properties.add_nvlist(INCONSISTENT_PROPERTY, pair_number(self.inconsistent));
        properties.add_nvlist(RECEIVE_RESUME_TOKEN_PROPERTY, pair_text(self.receive_resume_token.as_str()));
        properties.add_nvlist(ENCRYPTION_PROPERTY, pair_text(self.encryption.as_str()));
        properties.add_nvlist(KEY_STATUS_PROPERTY, pair_text(self.keystatus.as_str()));
        properties.add_nvlist(ATIME_PROPERTY, pair_text(self.atime.as_str()));
        properties.add_nvlist(RELATIME_PROPERTY, pair_text(self.relatime.as_str()));
        properties.add_nvlist(READONLY_PROPERTY, pair_text(self.readonly.as_str()));
        properties.add_nvlist(SETUID_PROPERTY, pair_text(self.setuid.as_str()));
        properties.add_nvlist(OVERLAY_PROPERTY, pair_text(self.overlay.as_str()));
        return properties;
    }
}

/// The nested dataset's own mountpoint,
/// which descends from the checkout.
fn nested_mountpoint(checkout: &Path) -> PathBuf {
    return checkout.join("package").join("vm-builder").join("cli").join("output");
}

/// A pool holding the replies the control scripts.
struct ScriptedPool {
    /// One reply per dataset.
    replies: Vec<Reply>,
}

impl ScriptedPool {
    /// The three datasets a case starts from,
    /// with the checkout mounted at the fixture's own checkout folder.
    fn beside(checkout: &Path) -> ScriptedPool {
        return ScriptedPool {
            replies: vec![
                Reply::of(CHECKOUT_DATASET, CHECKOUT_GUID, checkout.display().to_string().as_str()),
                Reply::of(CONTAINER, 77, CONTAINER_MOUNTPOINT),
                Reply::of(NESTED, 88, nested_mountpoint(checkout).display().to_string().as_str()),
                Reply::of("zpcachyos", 5, "/"),
            ],
        };
    }

    /// One dataset's reply,
    /// which a case edits.
    fn edit(&mut self, name: &str) -> &mut Reply {
        return self.replies.iter_mut().find(|reply| return reply.name == name).expect("the scripted pool holds the dataset");
    }
}

impl DatasetReader for ScriptedPool {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, ReadFailure> {
        let Some(reply) = self.replies.iter().find(|reply| return reply.name == dataset) else {
            return Err(ReadFailure::at(format!("this scripted pool holds no `{dataset}`")));
        };
        return Ok(DatasetIdentity { stats: stats(reply.guid), properties: reply.properties() });
    }

    fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// The mount table rows the control scripts.
struct ScriptedTable {
    /// The rows.
    rows: Vec<MountedDatasetInstance>,
}

impl ScriptedTable {
    /// The table of a boot whose checkout is mounted.
    fn checkout_mounted(checkout: &Path) -> ScriptedTable {
        return ScriptedTable { rows: vec![MountedDatasetInstance { dataset: String::from(CHECKOUT_DATASET), mount_point: checkout.to_path_buf(), root: PathBuf::from("/") }] };
    }
}

impl MountedDatasets for ScriptedTable {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        return Ok(self.rows.clone());
    }

    fn mount_holding(&self, _path: &Path) -> Result<Option<MountHoldingAPath>, String> {
        return Ok(None);
    }
}

/// A fixture folder holding the record the mode reads.
struct Fixture {
    /// The folder.
    folder: PathBuf,
}

impl Fixture {
    /// A fixture with a record the kernel agrees with,
    /// whose checkout is a folder inside it.
    fn new(name: &str) -> Fixture {
        let folder: PathBuf = crate::test_support::fixture(format!("mount-descendant-{name}").as_str());
        let _checkout: std::io::Result<()> = std::fs::create_dir_all(folder.join("checkout"));
        let record: String = format!(
            "{{\n  \"version\": 1,\n  \"checkout\": \"{}\",\n  \"dataset\": \"{CHECKOUT_DATASET}\",\n  \"guid\": \"{CHECKOUT_GUID}\",\n  \"container\": \"{CONTAINER}\",\n  \"pool\": \"zpcachyos\"\n}}\n",
            folder.join("checkout").display()
        );
        std::fs::write(folder.join("git-worktree-clone-checkout.json"), record).expect("the fixture record");
        return Fixture { folder };
    }

    /// The recorded checkout.
    fn checkout(&self) -> PathBuf {
        return self.folder.join("checkout");
    }

    /// The record's path.
    fn record(&self) -> PathBuf {
        return self.folder.join("git-worktree-clone-checkout.json");
    }
}

/// The request every case makes.
fn request() -> MountPathDescendantRequest {
    return MountPathDescendantRequest { dataset: String::from(NESTED) };
}

/// One plan over the fixture.
fn plan(fixture: &Fixture, pool: &ScriptedPool, table: &ScriptedTable, running_as_root: bool) -> Result<Answer, MountPathDescendantFailure> {
    let places: MountPlaces<'_> = MountPlaces { record: fixture.record(), configuration: fixture.folder.join("configuration.jsonc"), datasets: pool, mounted: table };
    return plan_with(&places, &request(), running_as_root, 1_000);
}

/// A caller that is not root is refused before the record is read.
#[test]
fn a_caller_that_is_not_root_is_refused_first() {
    let fixture: Fixture = Fixture::new("not-root");
    let pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, false).expect_err("an account may not mount inside a served checkout");
    assert_eq!(failure, MountPathDescendantFailure::NotRoot { effective_user: 1_000 });
    assert!(failure.message().contains("mount-path-descendant"), "the refusal names the mode: {}", failure.message());
}

/// A host with no record refuses,
/// which is the safe half the mode answers before step 10's record exists.
#[test]
fn no_record_refuses_before_any_dataset_is_read() {
    let fixture: Fixture = Fixture::new("no-record");
    std::fs::remove_file(fixture.record()).expect("the fixture starts without a record");
    let pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("no record stands");
    assert_eq!(failure, MountPathDescendantFailure::NoRecordedCheckout { record: fixture.record() });
    assert!(failure.message().contains("no-recorded-checkout"), "{}", failure.message());
}

/// A record whose GUID the kernel no longer shows refuses,
/// and the refusal prints both numbers and the replace command.
#[test]
fn a_record_whose_guid_moved_refuses_and_names_the_replace() {
    let fixture: Fixture = Fixture::new("guid");
    let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    pool.edit(CHECKOUT_DATASET).guid = CHECKOUT_GUID + 1;
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("the record is stale");
    let MountPathDescendantFailure::RecordNoLongerMatches { field, .. } = &failure else {
        panic!("a moved GUID is the field that differs: {failure:?}");
    };
    assert_eq!(field, "GUID");
    let said: String = failure.message();
    assert!(said.contains("checkout-record-no-longer-matches"), "{said}");
    assert!(said.contains(&(CHECKOUT_GUID + 1).to_string()), "the refusal prints the kernel's GUID: {said}");
}

/// A record whose mountpoint the kernel no longer shows refuses.
#[test]
fn a_record_whose_mountpoint_moved_refuses() {
    let fixture: Fixture = Fixture::new("mountpoint");
    let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    pool.edit(CHECKOUT_DATASET).mountpoint = String::from("/home/user/elsewhere");
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("the record is stale");
    let MountPathDescendantFailure::RecordNoLongerMatches { field, .. } = &failure else {
        panic!("a moved mountpoint is the field that differs: {failure:?}");
    };
    assert_eq!(field, "mountpoint");
}

/// A container that moved inside the checkout makes the record stale.
#[test]
fn a_container_that_moved_inside_the_checkout_refuses() {
    let fixture: Fixture = Fixture::new("container");
    let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    pool.edit(CONTAINER).mountpoint = fixture.checkout().join("clones").display().to_string();
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("the container is inside the checkout");
    let MountPathDescendantFailure::RecordNoLongerMatches { field, .. } = &failure else {
        panic!("a moved container is the field that differs: {failure:?}");
    };
    assert_eq!(field, "container");
}

/// A checkout the mount table does not show mounted refuses,
/// which is the fact a delayed or failed checkout mount leaves.
#[test]
fn a_checkout_that_is_not_mounted_refuses() {
    let fixture: Fixture = Fixture::new("unmounted");
    let pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let table: ScriptedTable = ScriptedTable { rows: Vec::new() };
    let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("the checkout is not mounted");
    assert!(matches!(failure, MountPathDescendantFailure::CheckoutNotMounted { .. }), "{failure:?}");
    assert!(failure.message().contains("checkout-not-mounted"), "{}", failure.message());
}

/// The recorded checkout's own dataset is never placed by this mode.
#[test]
fn the_checkouts_own_dataset_is_refused() {
    let fixture: Fixture = Fixture::new("own-dataset");
    let pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let places: MountPlaces<'_> = MountPlaces { record: fixture.record(), configuration: fixture.folder.join("configuration.jsonc"), datasets: &pool, mounted: &table };
    let own: MountPathDescendantRequest = MountPathDescendantRequest { dataset: String::from(CHECKOUT_DATASET) };
    let failure: MountPathDescendantFailure = plan_with(&places, &own, true, 1_000).expect_err("the checkout is not placed by this mode");
    assert!(matches!(failure, MountPathDescendantFailure::DatasetOutsideTheCheckout { .. }), "{failure:?}");
}

/// A dataset in another pool refuses.
#[test]
fn a_dataset_in_another_pool_refuses() {
    let fixture: Fixture = Fixture::new("pool");
    let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let mut other: Reply = Reply::of("tank/nosnap/vm-builder-output", 99, nested_mountpoint(fixture.checkout().as_path()).display().to_string().as_str());
    other.name = String::from("tank/nosnap/vm-builder-output");
    pool.replies.push(other);
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let places: MountPlaces<'_> = MountPlaces { record: fixture.record(), configuration: fixture.folder.join("configuration.jsonc"), datasets: &pool, mounted: &table };
    let foreign: MountPathDescendantRequest = MountPathDescendantRequest { dataset: String::from("tank/nosnap/vm-builder-output") };
    let failure: MountPathDescendantFailure = plan_with(&places, &foreign, true, 1_000).expect_err("another pool's subtree may not sit inside a served checkout");
    assert!(matches!(failure, MountPathDescendantFailure::DatasetInAnotherPool { .. }), "{failure:?}");
}

/// A dataset whose mountpoint is none, `-` or `legacy` refuses,
/// because such a dataset is mounted by someone else's rule.
#[test]
fn a_dataset_without_a_mountpoint_refuses() {
    let fixture: Fixture = Fixture::new("no-mountpoint");
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    for value in ["none", "-", "legacy"] {
        let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
        pool.edit(NESTED).mountpoint = String::from(value);
        let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("no mountpoint is no mountpoint to place");
        assert!(matches!(failure, MountPathDescendantFailure::DatasetWithoutAMountpoint { .. }), "{value}: {failure:?}");
    }
}

/// A mountpoint outside the checkout refuses, naming the dataset.
#[test]
fn a_mountpoint_outside_the_checkout_refuses() {
    let fixture: Fixture = Fixture::new("outside");
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    for value in [String::from("/home/user/elsewhere/output"), String::from("/home/user"), fixture.checkout().display().to_string()] {
        let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
        pool.edit(NESTED).mountpoint = value.clone();
        let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("the mountpoint does not descend from the checkout");
        assert!(matches!(failure, MountPathDescendantFailure::DatasetOutsideTheCheckout { .. }), "{value}: {failure:?}");
        assert!(failure.message().contains(NESTED), "the refusal names the dataset: {}", failure.message());
    }
}

/// A dataset the kernel already has at its own mountpoint answers without
/// writing anything,
/// which is what makes an `ExecStart=` re-run harmless.
#[test]
fn a_dataset_already_mounted_at_its_own_mountpoint_answers() {
    let fixture: Fixture = Fixture::new("already");
    let pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let table: ScriptedTable = ScriptedTable {
        rows: vec![
            MountedDatasetInstance { dataset: String::from(CHECKOUT_DATASET), mount_point: fixture.checkout(), root: PathBuf::from("/") },
            MountedDatasetInstance { dataset: String::from(NESTED), mount_point: PathBuf::from(nested_mountpoint(fixture.checkout().as_path()).display().to_string().as_str()), root: PathBuf::from("/") },
        ],
    };
    let answer: Answer = plan(&fixture, &pool, &table, true).expect("the dataset stands where it belongs");
    let mountpoint: PathBuf = nested_mountpoint(fixture.checkout().as_path());
    assert_eq!(answer, Answer::AlreadyMountedAtItsOwnMountpoint { dataset: String::from(NESTED), mountpoint: mountpoint.clone() });
    assert_eq!(super::already_mounted_line(NESTED, mountpoint.as_path()), format!("already mounted: {NESTED} at {}", mountpoint.display()));
}

/// A dataset the kernel has mounted somewhere else refuses.
#[test]
fn a_dataset_mounted_elsewhere_refuses() {
    let fixture: Fixture = Fixture::new("elsewhere");
    let pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let table: ScriptedTable = ScriptedTable {
        rows: vec![
            MountedDatasetInstance { dataset: String::from(CHECKOUT_DATASET), mount_point: fixture.checkout(), root: PathBuf::from("/") },
            MountedDatasetInstance { dataset: String::from(NESTED), mount_point: PathBuf::from("/media/other"), root: PathBuf::from("/") },
        ],
    };
    let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("the dataset stands elsewhere");
    assert!(matches!(failure, MountPathDescendantFailure::DatasetAlreadyMountedElsewhere { .. }), "{failure:?}");
    assert!(failure.message().contains("dataset-already-mounted-elsewhere"), "{}", failure.message());
}

/// Every property an explicit mount refuses is refused here with the property
/// and its value.
#[test]
fn the_properties_an_explicit_mount_refuses_are_refused() {
    let fixture: Fixture = Fixture::new("unmountable");
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    for (edit, property) in [
        (Box::new(|reply: &mut Reply| reply.canmount = String::from("off")) as Box<dyn Fn(&mut Reply)>,
         CANMOUNT_PROPERTY),
        (Box::new(|reply: &mut Reply| reply.zoned = String::from("on")), ZONED_PROPERTY),
        (Box::new(|reply: &mut Reply| reply.redacted = 3), REDACTED_PROPERTY),
        (
            Box::new(|reply: &mut Reply| {
                reply.inconsistent = 1;
                reply.receive_resume_token = String::from("1-abcdef-12");
            }),
            INCONSISTENT_PROPERTY,
        ),
    ] {
        let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
        edit(pool.edit(NESTED));
        let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("the dataset refuses an explicit mount");
        let MountPathDescendantFailure::DatasetNotMountable { property: named, .. } = &failure else {
            panic!("a refused property is `dataset-not-mountable`: {failure:?}");
        };
        assert_eq!(named, property);
        assert!(failure.message().contains("dataset-not-mountable"), "{}", failure.message());
    }
}

/// An encrypted dataset whose key is not loaded refuses.
#[test]
fn an_encrypted_dataset_without_its_key_refuses() {
    let fixture: Fixture = Fixture::new("key");
    let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    pool.edit(NESTED).encryption = String::from("aes-256-gcm");
    pool.edit(NESTED).keystatus = String::from("unavailable");
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let failure: MountPathDescendantFailure = plan(&fixture, &pool, &table, true).expect_err("the key is not loaded");
    assert!(matches!(failure, MountPathDescendantFailure::EncryptionKeyNotLoaded { .. }), "{failure:?}");
    assert!(failure.message().contains("encryption-key-not-loaded"), "{}", failure.message());
}

/// A mountpoint holding something,
/// with `overlay` off,
/// refuses.
#[test]
fn a_mountpoint_that_holds_something_refuses() {
    let fixture: Fixture = Fixture::new("not-empty");
    let pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let folder: PathBuf = nested_mountpoint(fixture.checkout().as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the fixture mountpoint");
    std::fs::write(folder.join("a-file"), "held\n").expect("the fixture holds a file");
    let mut pool_with_output: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    pool_with_output.edit(NESTED).mountpoint = folder.display().to_string();
    let failure: MountPathDescendantFailure = plan(&fixture, &pool_with_output, &table, true).expect_err("the mountpoint holds a file and the dataset does not overlay");
    assert!(matches!(failure, MountPathDescendantFailure::MountpointNotEmpty { .. }), "{failure:?}");
    assert!(pool.replies.iter().all(|reply| return reply.overlay != "on"));
    assert!(failure.message().contains("mountpoint-not-empty"), "{}", failure.message());
}

/// Every proof that holds answers a plan whose attributes are the dataset's own
/// properties,
/// so the mount the walk places is the one the kernel's reply resolved.
#[test]
fn every_proof_that_holds_answers_a_plan_with_the_datasets_attributes() {
    let fixture: Fixture = Fixture::new("plan");
    let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let folder: PathBuf = nested_mountpoint(fixture.checkout().as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the fixture mountpoint");
    pool.edit(NESTED).mountpoint = folder.display().to_string();
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let answer: Answer = plan(&fixture, &pool, &table, true).expect("every proof answered");
    let Answer::Plan(plan) = answer else {
        panic!("a checkout mounted and a dataset not yet mounted answers a plan");
    };
    assert_eq!(plan.dataset, NESTED);
    assert_eq!(plan.checkout, fixture.checkout());
    assert_eq!(plan.mountpoint, folder);
    assert_eq!(plan.attributes & libc::MOUNT_ATTR_NOATIME, libc::MOUNT_ATTR_NOATIME, "`atime=off` resolves to `noatime`");
    assert_eq!(plan.attributes & libc::MOUNT_ATTR_RDONLY, 0, "a writable dataset in a writable pool is not read-only");
    assert_eq!(plan.attributes & libc::MOUNT_ATTR_NOSUID, 0, "`setuid=on` keeps the kernel's default");
}

/// A read-only pool resolves to a read-only mount,
/// which is libzfs's own rule.
#[test]
fn a_read_only_pool_resolves_to_a_read_only_mount() {
    let fixture: Fixture = Fixture::new("readonly");
    let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
    let folder: PathBuf = nested_mountpoint(fixture.checkout().as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the fixture mountpoint");
    pool.edit(NESTED).mountpoint = folder.display().to_string();
    pool.edit("zpcachyos").readonly = String::from("on");
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    let answer: Answer = plan(&fixture, &pool, &table, true).expect("every proof answered");
    let Answer::Plan(plan) = answer else {
        panic!("a proved state answers a plan");
    };
    assert_eq!(plan.attributes & libc::MOUNT_ATTR_RDONLY, libc::MOUNT_ATTR_RDONLY, "a pool imported read-only forces every mount read-only");
}

/// A dataset whose `atime` is on and `relatime` is off resolves to strict
/// access times,
/// which is upstream's own pair.
#[test]
fn the_atime_pairs_resolve_to_the_attributes_upstream_derives() {
    let fixture: Fixture = Fixture::new("atime");
    let folder: PathBuf = nested_mountpoint(fixture.checkout().as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the fixture mountpoint");
    let table: ScriptedTable = ScriptedTable::checkout_mounted(fixture.checkout().as_path());
    for (atime, relatime, wanted_none, wanted_strict) in [("off", "off", true, false), ("on", "on", false, false), ("on", "off", false, true)] {
        let mut pool: ScriptedPool = ScriptedPool::beside(fixture.checkout().as_path());
        pool.edit(NESTED).mountpoint = folder.display().to_string();
        pool.edit(NESTED).atime = String::from(atime);
        pool.edit(NESTED).relatime = String::from(relatime);
        let answer: Answer = plan(&fixture, &pool, &table, true).expect("every proof answered");
        let Answer::Plan(plan) = answer else {
            panic!("a proved state answers a plan");
        };
        assert_eq!(plan.attributes & libc::MOUNT_ATTR_NOATIME == libc::MOUNT_ATTR_NOATIME, wanted_none, "atime={atime} relatime={relatime}");
        assert_eq!(plan.attributes & libc::MOUNT_ATTR_STRICTATIME == libc::MOUNT_ATTR_STRICTATIME, wanted_strict, "atime={atime} relatime={relatime}");
    }
}
