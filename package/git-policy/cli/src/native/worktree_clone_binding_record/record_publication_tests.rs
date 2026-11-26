//! Controls for the record's publication and reading:
//!  a binding that round-trips through its flushed rename,
//!  an inventory published once,
//!  a record another account could write,
//!  and a record over its bound.

use crate::worktree_clone_binding_record::binding_encoding::{
    FolderIdentity,
    Request,
};
use crate::worktree_clone_binding_record::binding_inventory::{
    EntryKind,
    InventoryEntry,
    InventoryKind,
    InventoryRecord,
};
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::{
    Phase,
    RequestForm,
};
use crate::worktree_clone_binding_record::record_publication::RecordRefusalAtTheFile;
use crate::worktree_clone_binding_record::record_publication::{
    publish_inventory,
    read_binding,
    read_inventory,
    write_binding,
};
use crate::worktree_clone_binding_record::registration_home::{
    binding_name,
    open_or_create_home,
};
use crate::worktree_clone_mount_protocol::{
    DatasetGuid,
    RequestNonce,
};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The claim's first record,
/// which every later phase extends.
fn claimed() -> BindingRecord {
    let guid = |value: &str| return DatasetGuid::parse(value).expect("the fixture guid reads");
    let identity: FolderIdentity =
        FolderIdentity::decode("131074:1763750400:123456789").expect("the fixture identity reads");
    return BindingRecord {
        claim: 2,
        nonce: RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
            .expect("the fixture nonce reads"),
        request: Request::with_argument(RequestForm::Branch, b"main"),
        phase: Phase::Claimed,
        settle_reason: None,
        checkout_dataset: String::from("tank/monochromatic-git-checkout"),
        checkout_dataset_guid: guid("4815162342109876543"),
        repository: b"/home/user/Monochromatic/.git".to_vec(),
        repository_identity: identity,
        slot: b"/home/user/Monochromatic/.claude/worktrees/add-cache-layer".to_vec(),
        slot_identity: None,
        registration: None,
        registration_identity: None,
        pool: String::from("tank"),
        pool_guid: guid("17293822569102704641"),
        origin_snapshot: String::from(
            "tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f",
        ),
        origin_snapshot_guid: None,
        origin_snapshot_createtxg: None,
        origin_dataset: None,
        origin_dataset_guid: None,
        clone_dataset: String::from(
            "tank/monochromatic-git-worktree-clone-datasets/add-cache-layer",
        ),
        clone_guid: None,
        clone_createtxg: None,
        pointer: None,
    };
}

/// The home folder of one control.
fn home(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = crate::test_support::fixture(name);
    let common: PathBuf = root.join(".git");
    std::fs::create_dir(common.as_path()).expect("the common directory is made");
    let folder: PathBuf =
        crate::worktree_clone_binding_record::registration_home::home_folder(common.as_path());
    let _made: std::fs::File = open_or_create_home(common.as_path()).expect("the home is made");
    return (root, folder);
}

/// A binding written and read back holds the same facts,
/// and its own name is the slot's.
#[test]
fn a_binding_round_trips_through_its_replaced_write() {
    let (root, folder) = home("publish-binding");
    let handle: std::fs::File = std::fs::File::open(folder.as_path()).expect("the home opens");
    let record: BindingRecord = claimed();
    write_binding(&handle, "add-cache-layer", &record).expect("the binding is written");
    assert!(folder.join(binding_name("add-cache-layer")).is_file());
    let read: BindingRecord =
        read_binding(folder.as_path(), "add-cache-layer").expect("the binding reads");
    assert_eq!(read, record);
    crate::test_support::remove(root.as_path());
}

/// A rewritten binding replaces its own bytes,
/// and no staging name is left.
#[test]
fn a_rewritten_binding_replaces_the_record() {
    let (root, folder) = home("publish-rewrite");
    let handle: std::fs::File = std::fs::File::open(folder.as_path()).expect("the home opens");
    let mut record: BindingRecord = claimed();
    write_binding(&handle, "add-cache-layer", &record).expect("the first write lands");
    record.phase = Phase::Slot;
    record.slot_identity =
        Some(FolderIdentity::decode("131089:1763750420:135792468").expect("the identity reads"));
    write_binding(&handle, "add-cache-layer", &record).expect("the second write lands");
    assert_eq!(
        read_binding(folder.as_path(), "add-cache-layer").expect("the binding reads"),
        record
    );
    let left: Vec<String> = std::fs::read_dir(folder.as_path())
        .expect("the home lists")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().into_owned())
        .filter(|name| return name.ends_with(".partial"))
        .collect();
    assert!(left.is_empty(), "staging names left: {left:?}");
    crate::test_support::remove(root.as_path());
}

/// The second publication of one inventory refuses,
/// and the first reads back.
#[test]
fn an_inventory_is_published_once() {
    let (root, folder) = home("publish-inventory");
    let handle: std::fs::File = std::fs::File::open(folder.as_path()).expect("the home opens");
    let guid: DatasetGuid =
        DatasetGuid::parse("4815162342109876543").expect("the fixture guid reads");
    let record: InventoryRecord = InventoryRecord {
        kind: InventoryKind::Clone,
        pool_guid: DatasetGuid::parse("17293822569102704641").expect("the pool guid reads"),
        dataset: String::from("tank/monochromatic-git-worktree-clone-datasets/add-cache-layer"),
        dataset_guid: guid,
        bounds: String::from("tank/monochromatic-git-worktree-clone-datasets/add-cache-layer"),
        bounds_guid: guid,
        bounds_createtxg: 121_294,
        entries: vec![InventoryEntry {
            kind: EntryKind::Snapshot,
            guid,
            createtxg: 121_300,
            name: String::from("a-snapshot"),
        }],
    };
    publish_inventory(&handle, "add-cache-layer", InventoryKind::Clone, &record)
        .expect("the inventory is published");
    let second: std::io::Error =
        publish_inventory(&handle, "add-cache-layer", InventoryKind::Clone, &record)
            .expect_err("the second publication refuses");
    assert_eq!(second.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(
        read_inventory(folder.as_path(), "add-cache-layer", InventoryKind::Clone)
            .expect("the inventory reads"),
        record
    );
    crate::test_support::remove(root.as_path());
}

/// A record another account could write refuses,
/// and one over its bound refuses with both numbers.
#[test]
fn a_widened_record_and_an_oversized_one_refuse() {
    let (root, folder) = home("publish-refusals");
    let handle: std::fs::File = std::fs::File::open(folder.as_path()).expect("the home opens");
    let record: BindingRecord = claimed();
    write_binding(&handle, "add-cache-layer", &record).expect("the binding is written");
    let path: PathBuf = folder.join(binding_name("add-cache-layer"));
    let mut widened: std::fs::Permissions = std::fs::metadata(path.as_path())
        .expect("the record's metadata reads")
        .permissions();
    widened.set_mode(0o666);
    std::fs::set_permissions(path.as_path(), widened.clone()).expect("the mode is widened");
    let refusal: RecordRefusalAtTheFile =
        read_binding(folder.as_path(), "add-cache-layer").expect_err("the widened record refuses");
    assert!(
        matches!(refusal, RecordRefusalAtTheFile::WritableByOthers { .. }),
        "{}",
        refusal.message(path.as_path())
    );
    widened.set_mode(0o600);
    std::fs::set_permissions(path.as_path(), widened).expect("the mode is narrowed again");
    std::fs::write(path.as_path(), vec![b'#'; 50_000]).expect("the long file is written");
    let size_refusal: RecordRefusalAtTheFile =
        read_binding(folder.as_path(), "add-cache-layer").expect_err("the long record refuses");
    assert!(
        matches!(
            size_refusal,
            RecordRefusalAtTheFile::TooLarge {
                found: 50_000,
                bound: 43_027
            }
        ),
        "{}",
        size_refusal.message(path.as_path())
    );
    crate::test_support::remove(root.as_path());
}
