//! What:
//!  Controls of the `record-checkout` mode:
//!  each refusal before anything is written,
//!  the record a first run writes,
//!  the run that changes nothing,
//!  and the `--replace` a legitimate restore needs.
//! Why:
//!  The record decides which checkout the route places a mount under,
//!  so every proof and every refusal is held here against scripted reads and a
//!  fixture folder,
//!  with no pool,
//!  no `/dev/zfs` request and nothing of this machine's own record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = runWith(places, { checkout: fixture.checkout }, true, 0);
//! ```

use super::{
    RECORD_VERSION,
    RecordCheckoutFailure,
    RecordCheckoutPlaces,
    RecordedCheckout,
    run_with,
};
use crate::worktree_clone_mount_handler::handler_mode_arguments::RecordCheckoutRequest;
use crate::worktree_clone_mount_handler::sweep_checkout_mount::{
    CheckoutMount,
    CheckoutMountRefusal,
    CheckoutMounts,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::MOUNTPOINT_PROPERTY;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountHoldingAPath,
    MountedDatasetInstance,
    MountedDatasets,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DmuObjsetStats,
    NvList,
};
use std::path::{
    Path,
    PathBuf,
};

/// The dataset the checkout is on.
const DATASET: &str = "zpcachyos/nosnap/monochromatic-git-checkout";

/// The container beside it.
const CONTAINER: &str = "zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets";

/// The GUID the kernel shows.
const GUID: u64 = 1_234_567_890_123_456_789;

/// The GUID a restore gave the dataset.
const RESTORED_GUID: u64 = 9_876_543_210_987_654_321;

/// One property as OpenZFS writes it.
fn pair_text(value: &str) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    pair.add_str("source", "local");
    return pair;
}

/// The statistics of one dataset.
fn stats(guid: u64) -> DmuObjsetStats {
    return DmuObjsetStats {
        dds_num_clones: 0,
        dds_creation_txg: 9,
        dds_guid: guid,
        dds_type: 2,
        dds_is_snapshot: 0,
        dds_inconsistent: 0,
        dds_redacted: 0,
        dds_origin: [0; 256],
        dds_flags: 0,
    };
}

/// A pool holding the answers the control scripts.
struct ScriptedPool {
    /// One reply per dataset name.
    replies: Vec<(String, u64, NvList)>,
}

impl ScriptedPool {
    /// The pool of one checkout and its container.
    fn beside(
        guid: u64,
        checkout_mountpoint: &str,
        container: &str,
    ) -> ScriptedPool {
        let mut checkout_properties: NvList = NvList::new();
        checkout_properties.add_nvlist(MOUNTPOINT_PROPERTY, pair_text(checkout_mountpoint));
        let mut container_properties: NvList = NvList::new();
        container_properties.add_nvlist(
            MOUNTPOINT_PROPERTY,
            pair_text("/var/lib/monochromatic-git-policy-cli/clones"),
        );
        return ScriptedPool {
            replies: vec![
                (String::from(DATASET), guid, checkout_properties),
                (String::from(container), 77, container_properties),
            ],
        };
    }

    /// The same pool with the checkout's own `mountpoint` property changed.
    fn with_the_property(
        &self,
        mountpoint: &str,
    ) -> ScriptedPool {
        let mut properties: NvList = NvList::new();
        properties.add_nvlist(MOUNTPOINT_PROPERTY, pair_text(mountpoint));
        return ScriptedPool {
            replies: vec![
                (String::from(DATASET), self.guid(), properties),
                (String::from(CONTAINER), 77, self.container_properties()),
            ],
        };
    }

    /// The GUID the pool answers for the checkout.
    fn guid(&self) -> u64 {
        return self
            .replies
            .first()
            .map_or(0, |(_name, guid, _properties)| return *guid);
    }

    /// The container's own properties.
    fn container_properties(&self) -> NvList {
        return self
            .replies
            .iter()
            .find(|(name, _guid, _properties)| return name == CONTAINER)
            .map_or_else(NvList::new, |(_name, _guid, properties)| {
                return properties.clone();
            });
    }
}

impl DatasetReader for ScriptedPool {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        let Some((_name, guid, properties)) = self
            .replies
            .iter()
            .find(|(name, _, _)| return name == dataset)
        else {
            return Err(ReadFailure::at(format!(
                "this scripted pool holds no `{dataset}`"
            )));
        };
        return Ok(DatasetIdentity {
            stats: stats(*guid),
            properties: properties.clone(),
        });
    }

    fn next_child(
        &self,
        _parent: &str,
        _cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// The mount the checkout is on.
struct ScriptedMounts {
    /// The mount's source.
    source: String,
    /// The mount's filesystem type.
    filesystem_type: String,
    /// The mount's mount point.
    mount_point: PathBuf,
}

impl CheckoutMounts for ScriptedMounts {
    fn mount_holding(
        &self,
        _checkout: &Path,
    ) -> Result<CheckoutMount, CheckoutMountRefusal> {
        return Ok(CheckoutMount {
            source: self.source.clone(),
            filesystem_type: self.filesystem_type.clone(),
            mount_point: self.mount_point.clone(),
            shared: None,
        });
    }
}

/// The mount table rows the configuration's discovery reads,
/// which a complete fixture configuration never asks.
struct ScriptedTable {
    /// The rows.
    rows: Vec<MountedDatasetInstance>,
}

impl MountedDatasets for ScriptedTable {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        return Ok(self.rows.clone());
    }

    fn mount_holding(
        &self,
        _path: &Path,
    ) -> Result<Option<MountHoldingAPath>, String> {
        return Ok(None);
    }
}

/// What:
///  A control's own folder,
///  whose checkout is a folder inside it and whose configuration names that
///  checkout and the container.
/// Why:
///  The mode reads the configuration for the container and compares the
///  checkout's own mountpoint against the path it was given,
///  so a fixture that named this machine's checkout would read this machine's
///  table;
///  a fixture that names its own folder asks nothing of the system.
struct Fixture {
    /// The folder the record,
    /// the configuration and the checkout live in.
    folder: PathBuf,
}

impl Fixture {
    /// A fixture of this control's own,
    /// with a configuration naming the checkout and the container.
    fn new(name: &str) -> Fixture {
        let folder: PathBuf =
            crate::test_support::fixture(format!("record-checkout-{name}").as_str());
        let _records: std::io::Result<()> = std::fs::create_dir_all(folder.join("records"));
        let _checkout: std::io::Result<()> = std::fs::create_dir_all(folder.join("checkout"));
        let fixture: Fixture = Fixture {
            folder,
        };
        fixture.write_configuration(CONTAINER);
        return fixture;
    }

    /// The checkout this control records.
    fn checkout(&self) -> PathBuf {
        return self.folder.join("checkout");
    }

    /// The configuration the mode reads.
    fn configuration(&self) -> PathBuf {
        return self.folder.join("configuration.jsonc");
    }

    /// The record's path.
    fn record(&self) -> PathBuf {
        return self
            .folder
            .join("records")
            .join("git-worktree-clone-checkout.json");
    }

    /// The configuration's text,
    /// naming the checkout and the container so nothing is discovered.
    fn write_configuration(
        &self,
        container: &str,
    ) {
        let document: String = format!(
            "{{\"cloneMounts\": {{\"checkout\": \"{}\", \"checkoutDataset\": \"{DATASET}\", \"parent\": \"{container}\"}}}}\n",
            self.checkout().display()
        );
        std::fs::write(self.configuration(), document).expect("the fixture configuration");
    }

    /// The request a control records,
    /// and the one that admits a restored GUID.
    fn request(&self) -> RecordCheckoutRequest {
        return RecordCheckoutRequest {
            checkout: self.checkout(),
            replace: None,
        };
    }

    /// The request that admits the GUID the kernel shows.
    fn replacing(
        &self,
        guid: u64,
    ) -> RecordCheckoutRequest {
        return RecordCheckoutRequest {
            checkout: self.checkout(),
            replace: Some(guid.to_string()),
        };
    }
}

/// One run over the fixture,
/// with the mount the checkout is on and the pool's answers.
fn record(
    fixture: &Fixture,
    pool: &ScriptedPool,
    mounted_at: &Path,
    wanted: &RecordCheckoutRequest,
    running_as_root: bool,
) -> Result<String, RecordCheckoutFailure> {
    let mounts: ScriptedMounts = ScriptedMounts {
        source: String::from(DATASET),
        filesystem_type: String::from("zfs"),
        mount_point: mounted_at.to_path_buf(),
    };
    let table: ScriptedTable = ScriptedTable {
        rows: Vec::new(),
    };
    let places: RecordCheckoutPlaces<'_> = RecordCheckoutPlaces {
        configuration: fixture.configuration(),
        record: fixture.record(),
        mounts: &mounts,
        mounted: &table,
        datasets: pool,
    };
    return run_with(
        &places,
        wanted,
        running_as_root,
        if running_as_root { 0 } else { 1_000 },
    );
}

/// The answer of one proved run,
/// with the checkout's own mount and the container's name.
fn recorded(
    fixture: &Fixture,
    guid: u64,
    container: &str,
    wanted: &RecordCheckoutRequest,
) -> Result<String, RecordCheckoutFailure> {
    fixture.write_configuration(container);
    let checkout: String = fixture.checkout().display().to_string();
    let pool: ScriptedPool = ScriptedPool::beside(guid, checkout.as_str(), container);
    let mountpoint: PathBuf = fixture.checkout();
    return record(fixture, &pool, mountpoint.as_path(), wanted, true);
}

/// A caller that is not root is refused before anything is read.
#[test]
fn a_caller_that_is_not_root_is_refused_first() {
    let fixture: Fixture = Fixture::new("not-root");
    let pool: ScriptedPool = ScriptedPool::beside(
        GUID,
        fixture.checkout().display().to_string().as_str(),
        CONTAINER,
    );
    let mountpoint: PathBuf = fixture.checkout();
    let failure: RecordCheckoutFailure = record(
        &fixture,
        &pool,
        mountpoint.as_path(),
        &fixture.request(),
        false,
    )
    .expect_err("an account may not record the checkout");
    assert_eq!(
        failure,
        RecordCheckoutFailure::NotRoot {
            effective_user: 1_000
        }
    );
    assert!(
        failure.message().starts_with(&format!(
            "{}: ",
            crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name()
        )),
        "the refusal opens with the words: {}",
        failure.message()
    );
    assert!(!fixture.record().exists(), "nothing was written");
}

/// A path that is not the dataset's own mountpoint is refused.
#[test]
fn a_path_inside_the_mount_is_not_a_mount_point() {
    let fixture: Fixture = Fixture::new("inside");
    let pool: ScriptedPool = ScriptedPool::beside(
        GUID,
        fixture.checkout().display().to_string().as_str(),
        CONTAINER,
    );
    let failure: RecordCheckoutFailure = record(
        &fixture,
        &pool,
        Path::new("/home/user"),
        &fixture.request(),
        true,
    )
    .expect_err("the mountpoint is elsewhere");
    assert!(
        matches!(failure, RecordCheckoutFailure::NotAMountPoint { .. }),
        "{failure:?}"
    );
    assert!(
        failure.message().contains("checkout-not-a-mount-point"),
        "{}",
        failure.message()
    );
}

/// A mount that is not ZFS is refused,
/// because a record for it would make the route's gate pass on a checkout it
/// cannot clone.
#[test]
fn a_checkout_on_another_filesystem_is_refused() {
    let fixture: Fixture = Fixture::new("ext4");
    let pool: ScriptedPool = ScriptedPool::beside(
        GUID,
        fixture.checkout().display().to_string().as_str(),
        CONTAINER,
    );
    let mounts: ScriptedMounts = ScriptedMounts {
        source: String::from("/dev/sda1"),
        filesystem_type: String::from("ext4"),
        mount_point: fixture.checkout(),
    };
    let table: ScriptedTable = ScriptedTable {
        rows: Vec::new(),
    };
    let places: RecordCheckoutPlaces<'_> = RecordCheckoutPlaces {
        configuration: fixture.configuration(),
        record: fixture.record(),
        mounts: &mounts,
        mounted: &table,
        datasets: &pool,
    };
    let failure: RecordCheckoutFailure =
        run_with(&places, &fixture.request(), true, 0).expect_err("an ext4 mount is no dataset");
    assert_eq!(
        failure,
        RecordCheckoutFailure::NotOnZfs {
            checkout: fixture.checkout(),
            filesystem_type: String::from("ext4")
        }
    );
    assert!(
        failure.message().contains("checkout-not-on-zfs"),
        "{}",
        failure.message()
    );
}

/// The dataset's own `mountpoint` property must be the path,
/// or the gate would compare the record against a folder.
#[test]
fn a_dataset_whose_mountpoint_property_is_another_path_is_refused() {
    let fixture: Fixture = Fixture::new("property");
    let beside: ScriptedPool = ScriptedPool::beside(
        GUID,
        fixture.checkout().display().to_string().as_str(),
        CONTAINER,
    );
    let pool: ScriptedPool = beside.with_the_property("/home/user/elsewhere");
    let mountpoint: PathBuf = fixture.checkout();
    let failure: RecordCheckoutFailure = record(
        &fixture,
        &pool,
        mountpoint.as_path(),
        &fixture.request(),
        true,
    )
    .expect_err("the property names another path");
    assert!(
        matches!(failure, RecordCheckoutFailure::NotAMountPoint { .. }),
        "{failure:?}"
    );
    assert!(
        failure.message().contains("/home/user/elsewhere"),
        "the refusal names the property: {}",
        failure.message()
    );
}

/// A container inside the checkout,
/// and one that holds it,
/// are each refused with the direction the proof found.
#[test]
fn a_container_that_is_not_beside_the_checkout_is_refused() {
    let fixture: Fixture = Fixture::new("container");
    for (container, direction) in [
        (
            "zpcachyos/nosnap/monochromatic-git-checkout/clones",
            "the container is inside the checkout",
        ),
        ("zpcachyos/nosnap", "the container holds the checkout"),
        (
            "zpcachyos/nosnap/monochromatic-git-checkout",
            "they are the same dataset",
        ),
    ] {
        let failure: RecordCheckoutFailure =
            recorded(&fixture, GUID, container, &fixture.request())
                .expect_err("the container is not beside the checkout");
        let RecordCheckoutFailure::ContainerNotBesideTheCheckout {
            because,
            ..
        } = &failure
        else {
            panic!("a container that is not beside the checkout is refused: {failure:?}");
        };
        assert_eq!(because, direction);
        assert!(
            failure
                .message()
                .contains("container-not-beside-the-checkout"),
            "{}",
            failure.message()
        );
    }
}

/// A container in another pool is refused,
/// because a foreign pool's subtree would sit inside a served checkout.
#[test]
fn a_container_in_another_pool_is_refused() {
    let fixture: Fixture = Fixture::new("pool");
    let failure: RecordCheckoutFailure = recorded(
        &fixture,
        GUID,
        "tank/nosnap/monochromatic-git-worktree-clone-datasets",
        &fixture.request(),
    )
    .expect_err("the container is in another pool");
    assert!(
        matches!(
            failure,
            RecordCheckoutFailure::ContainerNotBesideTheCheckout { .. }
        ),
        "{failure:?}"
    );
}

/// The first run writes the record with every field the kernel gave,
/// and the same run again writes nothing.
#[test]
fn the_first_run_records_and_the_second_agrees() {
    let fixture: Fixture = Fixture::new("first");
    let line: String =
        recorded(&fixture, GUID, CONTAINER, &fixture.request()).expect("the checkout is proved");
    let checkout: String = fixture.checkout().display().to_string();
    assert_eq!(
        line,
        format!(
            "recorded: {checkout} is {DATASET} (GUID {GUID}), in pool zpcachyos, beside container {CONTAINER}; the record is {}",
            fixture.record().display()
        )
    );
    let text: String = std::fs::read_to_string(fixture.record()).expect("the record is written");
    assert!(
        text.contains(&format!("\"guid\": \"{GUID}\"")),
        "the GUID is the kernel's, as text: {text}"
    );
    assert!(
        text.contains(&format!("\"version\": {RECORD_VERSION}")),
        "{text}"
    );
    assert!(
        text.contains(&format!("\"dataset\": \"{DATASET}\"")),
        "{text}"
    );
    assert!(
        text.contains(&format!("\"container\": \"{CONTAINER}\"")),
        "{text}"
    );
    assert!(
        text.contains(&format!("\"checkout\": \"{checkout}\"")),
        "{text}"
    );
    let again: String =
        recorded(&fixture, GUID, CONTAINER, &fixture.request()).expect("every field agrees");
    assert_eq!(
        again,
        format!(
            "already recorded: {}, and every field agrees with the kernel",
            fixture.record().display()
        )
    );
}

/// A restored GUID refuses with both numbers and the exact command to run.
#[test]
fn a_restored_guid_refuses_and_names_the_replace_command() {
    let fixture: Fixture = Fixture::new("restored");
    recorded(&fixture, GUID, CONTAINER, &fixture.request()).expect("the first record");
    let failure: RecordCheckoutFailure =
        recorded(&fixture, RESTORED_GUID, CONTAINER, &fixture.request())
            .expect_err("the record differs");
    let RecordCheckoutFailure::RecordDiffers {
        recorded,
        kernel,
        field,
    } = &failure
    else {
        panic!("a differing GUID is refused as that field: {failure:?}");
    };
    assert_eq!(recorded, &GUID.to_string());
    assert_eq!(kernel, &RESTORED_GUID.to_string());
    assert!(
        matches!(field, super::DifferingField::Guid { .. }),
        "{field:?}"
    );
    let said: String = failure.message();
    assert!(
        said.starts_with(&format!(
            "{}: checkout-record-differs:",
            crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name()
        )),
        "every refusal opens with the words: {said}"
    );
    assert!(
        said.contains(&format!("--replace {RESTORED_GUID}")),
        "the refusal prints the exact command: {said}"
    );
    assert!(
        std::fs::read_to_string(fixture.record())
            .expect("the record stands")
            .contains(&GUID.to_string()),
        "nothing was written"
    );
}

/// A token that is not the kernel's GUID is refused,
/// and the record stands.
#[test]
fn a_token_that_is_not_the_kernels_guid_is_refused() {
    let fixture: Fixture = Fixture::new("token");
    recorded(&fixture, GUID, CONTAINER, &fixture.request()).expect("the first record");
    let failure: RecordCheckoutFailure =
        recorded(&fixture, RESTORED_GUID, CONTAINER, &fixture.replacing(1))
            .expect_err("the token is not the kernel's");
    assert_eq!(
        failure,
        RecordCheckoutFailure::ReplaceTokenDiffers {
            kernel: RESTORED_GUID.to_string(),
            typed: String::from("1")
        }
    );
    assert!(
        failure.message().contains("replace-token-differs"),
        "{}",
        failure.message()
    );
    assert!(
        std::fs::read_to_string(fixture.record())
            .expect("the record stands")
            .contains(&GUID.to_string()),
        "nothing was written"
    );
}

/// The kernel's own GUID replaces the record and the line prints both.
#[test]
fn the_kernels_guid_replaces_the_record() {
    let fixture: Fixture = Fixture::new("replace");
    recorded(&fixture, GUID, CONTAINER, &fixture.request()).expect("the first record");
    let line: String = recorded(
        &fixture,
        RESTORED_GUID,
        CONTAINER,
        &fixture.replacing(RESTORED_GUID),
    )
    .expect("the kernel's GUID admits the dataset");
    let checkout: String = fixture.checkout().display().to_string();
    assert_eq!(
        line,
        format!(
            "replaced: {checkout} keeps its path, dataset, container and pool; its GUID was {GUID} and is now {RESTORED_GUID}. The record is {}.",
            fixture.record().display()
        )
    );
    assert!(
        std::fs::read_to_string(fixture.record())
            .expect("the record is written")
            .contains(&RESTORED_GUID.to_string())
    );
    let again: String = recorded(&fixture, RESTORED_GUID, CONTAINER, &fixture.request())
        .expect("every field agrees now");
    assert!(again.starts_with("already recorded:"));
}

/// The kernel's own GUID admits the GUID and nothing else:
/// a container that also differs is refused,
/// and the record keeps the container it holds.
///
/// The two differences are one run's,
/// because the GUID is the first field the comparison reaches and the
/// replacement is the only admitted change:
/// a run whose container differs is a different container,
/// which no `--replace` token names.
#[test]
fn the_kernels_guid_does_not_replace_a_differing_container() {
    let fixture: Fixture = Fixture::new("replace-two-fields");
    recorded(&fixture, GUID, CONTAINER, &fixture.request()).expect("the first record");
    let elsewhere: &str = "zpcachyos/nosnap/another-clone-container";
    let failure: RecordCheckoutFailure = recorded(
        &fixture,
        RESTORED_GUID,
        elsewhere,
        &fixture.replacing(RESTORED_GUID),
    )
    .expect_err("a differing container is refused even with the kernel's GUID");
    let RecordCheckoutFailure::RecordDiffers {
        recorded: held,
        kernel,
        field,
    } = &failure
    else {
        panic!("a differing container is refused as that field: {failure:?}");
    };
    assert_eq!(
        held, CONTAINER,
        "the refusal names the container the record holds"
    );
    assert_eq!(
        kernel, elsewhere,
        "the refusal names the container the configuration names"
    );
    assert!(
        matches!(field, super::DifferingField::Container { .. }),
        "{field:?}"
    );
    let said: String = failure.message();
    assert!(
        said.starts_with(&format!(
            "{}: checkout-record-differs:",
            crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name()
        )),
        "every refusal opens with the words: {said}"
    );
    assert!(
        !said.contains("--replace"),
        "no flag admits a container, so the way forward names none: {said}"
    );
    let text: String = std::fs::read_to_string(fixture.record()).expect("the record stands");
    assert!(
        text.contains(CONTAINER),
        "the recorded container was not rewritten: {text}"
    );
    assert!(
        text.contains(&GUID.to_string()),
        "the recorded GUID was not replaced: {text}"
    );
}

/// `--replace` with no record stands refuses,
/// because then there is nothing to replace.
#[test]
fn a_replace_with_no_record_refuses() {
    let fixture: Fixture = Fixture::new("no-record");
    let failure: RecordCheckoutFailure =
        recorded(&fixture, GUID, CONTAINER, &fixture.replacing(GUID))
            .expect_err("nothing stands to replace");
    assert_eq!(
        failure,
        RecordCheckoutFailure::NothingToReplace {
            record: fixture.record()
        }
    );
    assert!(
        failure.message().contains("nothing-to-replace"),
        "{}",
        failure.message()
    );
    assert!(!fixture.record().exists());
}

/// The record's folder is created only when it is absent,
/// and a folder another account could write is refused.
#[test]
fn the_folder_is_made_and_a_folder_another_account_could_write_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let fixture: Fixture = Fixture::new("folder");
    std::fs::remove_dir_all(fixture.folder.join("records"))
        .expect("the fixture starts without the record's folder");
    recorded(&fixture, GUID, CONTAINER, &fixture.request()).expect("the folder is made");
    assert!(fixture.record().exists());
    let mode: u32 = std::fs::metadata(fixture.folder.join("records"))
        .expect("the folder")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, super::RECORD_FOLDER_MODE, "the folder is 0755");
    std::fs::set_permissions(
        fixture.folder.join("records"),
        std::fs::Permissions::from_mode(0o777),
    )
    .expect("the fixture widens the folder");
    let failure: RecordCheckoutFailure = recorded(&fixture, GUID, CONTAINER, &fixture.request())
        .expect_err("a folder another account could write refuses");
    let RecordCheckoutFailure::RecordUnwritable {
        reason,
        ..
    } = &failure
    else {
        panic!("a folder another account could write is refused: {failure:?}");
    };
    assert!(reason.contains("0777"), "{reason}");
}

/// A configuration that names a number where text belongs is refused,
/// and the refusal names the file.
#[test]
fn a_configuration_that_refuses_names_the_file() {
    let fixture: Fixture = Fixture::new("configuration");
    std::fs::write(
        fixture.configuration(),
        "{\"cloneMounts\": {\"parent\": 7}}\n",
    )
    .expect("the fixture configuration");
    let pool: ScriptedPool = ScriptedPool::beside(
        GUID,
        fixture.checkout().display().to_string().as_str(),
        CONTAINER,
    );
    let failure: RecordCheckoutFailure = record(
        &fixture,
        &pool,
        fixture.checkout().as_path(),
        &fixture.request(),
        true,
    )
    .expect_err("the file names a number");
    assert!(
        matches!(failure, RecordCheckoutFailure::Configuration(_)),
        "{failure:?}"
    );
    assert!(
        failure.message().contains("configuration.jsonc"),
        "{}",
        failure.message()
    );
}

/// The record reads back exactly what the mode wrote,
/// and a record that is not this mode's shape is a refusal.
#[test]
fn a_record_that_is_not_this_modes_is_refused() {
    let fixture: Fixture = Fixture::new("shape");
    recorded(&fixture, GUID, CONTAINER, &fixture.request()).expect("the first record");
    let read_back: Option<RecordedCheckout> =
        super::read_record(fixture.record().as_path()).expect("the record reads");
    let Some(stored) = read_back else {
        panic!("the record is there");
    };
    assert_eq!(stored.guid, GUID.to_string());
    assert_eq!(stored.dataset, DATASET);
    assert_eq!(stored.container, CONTAINER);
    assert_eq!(stored.pool, String::from("zpcachyos"));
    assert_eq!(stored.checkout, fixture.checkout());
    std::fs::write(fixture.record(), "{\"version\": 1}\n")
        .expect("the fixture writes a record without fields");
    let failure: RecordCheckoutFailure = super::read_record(fixture.record().as_path())
        .expect_err("a record without fields is refused");
    assert!(
        matches!(failure, RecordCheckoutFailure::RecordUnreadable { .. }),
        "{failure:?}"
    );
}

/// The property the record's proof reads is the one the kernel writes.
#[test]
fn the_property_the_proof_reads_is_the_kernels() {
    assert_eq!(MOUNTPOINT_PROPERTY, "mountpoint");
}
