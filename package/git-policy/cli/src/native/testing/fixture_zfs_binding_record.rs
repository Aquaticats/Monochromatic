//! What:
//!  One binding record whose slot,
//!  registration and pointer are a fixture's own paths,
//!  shaped the way the add writes it,
//!  and the publication that puts it under a fixture's Git common directory.
//! Why:
//!  The pointer and repair controls read the record a real add writes,
//!  and a record naming `/repo/.git` would prove nothing about a fixture's
//!  registration
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The binding").
//!  Nothing here is part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = recordNaming(Phase.Filled, slot, registration, pointer, 'tank/clones/one');
//! publishUnder(commonDir, record, 'one');
//! ```

use crate::worktree_clone_binding_record::binding_encoding::{
    FolderIdentity,
    Request,
};
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::{
    Phase,
    RequestForm,
};
use crate::worktree_clone_binding_record::record_publication::write_binding;
use crate::worktree_clone_binding_record::registration_home::open_or_create_home;
use crate::worktree_clone_mount_protocol::{
    DatasetGuid,
    RequestNonce,
};
use std::path::{
    Path,
    PathBuf,
};

/// The clone dataset a fixture record names unless a control chooses another.
pub(crate) const CLONE_DATASET: &str = "tank/monochromatic-git-worktree-clone-datasets/one";

/// The checkout dataset a fixture record names.
pub(crate) const CHECKOUT_DATASET: &str = "tank/monochromatic-git-checkout";

/// The pool a fixture record names.
pub(crate) const POOL: &str = "tank";

/// The nonce a fixture record carries,
/// which its origin snapshot name repeats.
pub(crate) const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// One GUID the fixture uses.
fn guid(value: &str) -> DatasetGuid {
    return DatasetGuid::parse(value).expect("the fixture guid reads");
}

/// One folder identity the fixture uses.
fn identity() -> FolderIdentity {
    return FolderIdentity::decode("131074:1763750400:123456789")
        .expect("the fixture identity reads");
}

/// What:
///  One record of one fixture's link,
///  with every field the phase proves.
/// Why:
///  The pointer and repair rules read the slot path,
///  the registration path and the pointer bytes,
///  so a control must be able to name the fixture's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = recordNaming(Phase.Filled, slot, registration, pointerBytes);
/// ```
pub(crate) fn record_naming(
    phase: Phase,
    slot: &Path,
    registration: &Path,
    pointer: Option<Vec<u8>>,
) -> BindingRecord {
    let with_registration: bool = matches!(
        phase,
        Phase::Registered
            | Phase::Snapshotted
            | Phase::Cloned
            | Phase::Mounted
            | Phase::Filled
            | Phase::Settling
    );
    return BindingRecord {
        claim: 1,
        nonce: RequestNonce::parse(NONCE).expect("the fixture nonce reads"),
        request: Request::with_argument(RequestForm::Branch, b"topic"),
        phase,
        settle_reason: None,
        checkout_dataset: String::from(CHECKOUT_DATASET),
        checkout_dataset_guid: guid("4815162342109876543"),
        repository: b"/fixture/.git".to_vec(),
        repository_identity: identity(),
        slot: slot.as_os_str().as_encoded_bytes().to_vec(),
        slot_identity: Some(identity()),
        registration: with_registration
            .then(|| return registration.as_os_str().as_encoded_bytes().to_vec()),
        registration_identity: with_registration.then_some(identity()),
        pool: String::from(POOL),
        pool_guid: guid("17293822569102704641"),
        origin_snapshot: format!("{CHECKOUT_DATASET}@git-worktree-clone-origin-{NONCE}"),
        origin_snapshot_guid: Some(guid("16371409254608711923")),
        origin_snapshot_createtxg: Some(121_293),
        origin_dataset: Some(String::from(CHECKOUT_DATASET)),
        origin_dataset_guid: Some(guid("4815162342109876543")),
        clone_dataset: String::from(CLONE_DATASET),
        clone_guid: Some(guid("4821193048571903102")),
        clone_createtxg: Some(121_294),
        pointer,
    };
}

/// What:
///  Publish one fixture record under a fixture's Git common directory.
/// Why:
///  The readers under control take a common directory and find the home
///  themselves,
///  so a control writes through the release writer rather than by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishUnder(commonDir, record, 'one');
/// ```
pub(crate) fn publish_under(
    common_dir: &Path,
    record: &BindingRecord,
    slot: &str,
) {
    let home: std::fs::File = open_or_create_home(common_dir).expect("the fixture home opens");
    write_binding(&home, slot, record).expect("the fixture record writes");
}

/// What:
///  One fixture repository with a common directory to hold records.
/// Why:
///  The records live under a Git common directory,
///  and a control needs the folder without a Git repository's other files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const common = commonDirectoryIn(root); // root/.git
/// ```
pub(crate) fn common_directory_in(root: &Path) -> PathBuf {
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the fixture common directory");
    return common;
}

/// What:
///  One fixture repository with one real registration of one slot beside it:
///  the worktree Git made at the slot's path,
///  the registration under the common directory,
///  and the pointer Git wrote into the slot.
/// Why:
///  A control that meets a slot from a command line needs Git's own
///  registration and pointer,
///  and three folders' controls would otherwise each build the same four.
///  The slot's last name is the name the record is filed under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = aRegisteredSlot('command-move'); // repository, common, slot, registration, pointer
/// ```
pub(crate) struct RegisteredSlot {
    /// The fixture root.
    pub root: PathBuf,
    /// The repository Git made.
    pub repository: PathBuf,
    /// Its Git common directory.
    pub common: PathBuf,
    /// The worktree's path,
    /// whose last component is the slot's name.
    pub slot: PathBuf,
    /// The registration Git wrote for it.
    pub registration: PathBuf,
    /// The bytes Git wrote into the slot's pointer.
    pub pointer: Vec<u8>,
}

/// What:
///  Make one fixture repository and one real registration of one slot in it.
/// Why:
///  Git writes the registration and the pointer,
///  so what a control reads is what an add would have left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = aRegisteredSlot(name);
/// ```
pub(crate) fn a_registered_slot(name: &str) -> RegisteredSlot {
    let root: PathBuf = crate::test_support::fixture(name);
    let repository: PathBuf = crate::test_support::repository(root.as_path(), "repository");
    let slot: PathBuf = root.join(SLOT_LAST_NAME);
    crate::test_support::git(
        repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            slot.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let common: PathBuf = repository.join(".git");
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("pointer reads");
    let registration: PathBuf = common.join("worktrees").join(SLOT_LAST_NAME);
    return RegisteredSlot {
        root,
        repository,
        common,
        slot,
        registration,
        pointer,
    };
}

/// The last name every fixture slot's path carries,
/// which is the name its record is filed under.
const SLOT_LAST_NAME: &str = "one";
