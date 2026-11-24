//! What:
//!  Controls of the one removal plan step 18 performs:
//!  the two known staging forms go,
//!  every dot entry matching neither stays,
//!  and `.parent` and every slot record keep their bytes.
//! Why:
//!  The removal is the step's only write,
//!  and a store the adoption also writes is the one place a wrong removal
//!  would destroy a record the route depends on,
//!  so the control plants both forms beside a record and a stranger's name and
//!  reads every byte of the survivors.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removal = remove_known_staging_forms({ store }); // ['.a1b2...', '..parent.3f.partial']
//! expect(readFileSync(parentRecord)).toEqual(before);
//! ```

use super::{StagingRemoval, StagingRemovalRequest, remove_known_staging_forms};
use crate::worktree_clone_mount::{CloneFolderIdentity, PARENT_RECORD_NAME, STORE_HEADER, SlotIdentityStore};
use std::path::{Path, PathBuf};

/// A store folder with a parent record,
/// one slot record,
/// both staging forms and one dot name nobody accounts for.
fn planted_store(name: &str) -> (PathBuf, Vec<u8>, Vec<u8>) {
    use std::io::Write as _;
    let root: PathBuf = crate::test_support::fixture(name);
    let store_path: PathBuf = root.join("store");
    std::fs::create_dir(&store_path).expect("store folder");
    let identity: CloneFolderIdentity = CloneFolderIdentity { device: 2049, inode: 42, owner: 1000, mode: 0o700, access_acl: None, default_acl: None };
    let parent_bytes: Vec<u8> = format!("{STORE_HEADER}\nparent - {}\n", identity.encode()).into_bytes();
    let slot_bytes: Vec<u8> = format!("{STORE_HEADER}\nslot one {}\n", identity.encode()).into_bytes();
    std::fs::write(store_path.join(PARENT_RECORD_NAME), &parent_bytes).expect("parent record");
    std::fs::write(store_path.join("one"), &slot_bytes).expect("slot record");
    for planted in [".0123456789abcdef0123456789abcdef", "..parent.0123456789abcdef.partial", ".partial-write"] {
        let mut file = std::fs::File::create(store_path.join(planted)).expect("planted entry");
        file.write_all(b"left by a killed writer").expect("planted bytes");
    }
    return (store_path, parent_bytes, slot_bytes);
}

/// Every file directly under the store,
/// with its bytes.
fn surviving(path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut names: Vec<String> = Vec::new();
    for listed in std::fs::read_dir(path).expect("the store lists") {
        let entry = listed.expect("one entry");
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    return names.into_iter().map(|name| return (name.clone(), std::fs::read(path.join(name.as_str())).expect("a survivor's bytes"))).collect();
}

/// Both known forms are removed and answered by name.
#[test]
fn both_known_staging_forms_are_removed_and_named() {
    let (store_path, _, _) = planted_store("doctor-staging-removal-known");
    let store: SlotIdentityStore = SlotIdentityStore::open(store_path.as_path()).expect("the store opens");
    let removal: StagingRemoval = remove_known_staging_forms(StagingRemovalRequest { store: &store }).expect("the removal runs");
    assert_eq!(removal.removed, vec![String::from("..parent.0123456789abcdef.partial"), String::from(".0123456789abcdef0123456789abcdef")], "{removal:?}");
    let left: Vec<String> = surviving(store_path.as_path()).into_iter().map(|(name, _bytes)| return name).collect();
    assert_eq!(left, vec![String::from(".parent"), String::from(".partial-write"), String::from("one")], "{left:?}");
}

/// `.parent` and the slot record keep their bytes,
/// and the stranger's dot name stays.
#[test]
fn the_parent_record_and_every_slot_record_keep_their_bytes() {
    let (store_path, parent_bytes, slot_bytes) = planted_store("doctor-staging-removal-bytes");
    let before_unknown: Vec<u8> = std::fs::read(store_path.join(".partial-write")).expect("the planted stranger");
    let store: SlotIdentityStore = SlotIdentityStore::open(store_path.as_path()).expect("the store opens");
    let _removal: StagingRemoval = remove_known_staging_forms(StagingRemovalRequest { store: &store }).expect("the removal runs");
    let survivors: Vec<(String, Vec<u8>)> = surviving(store_path.as_path());
    assert_eq!(survivors.len(), 3, "{survivors:?}");
    assert_eq!(std::fs::read(store_path.join(PARENT_RECORD_NAME)).expect("parent bytes"), parent_bytes);
    assert_eq!(std::fs::read(store_path.join("one")).expect("slot bytes"), slot_bytes);
    assert_eq!(std::fs::read(store_path.join(".partial-write")).expect("stranger bytes"), before_unknown);
}

/// A store holding no staging form answers nothing removed and leaves every
/// file where it was.
#[test]
fn a_store_without_staging_forms_loses_nothing() {
    let root: PathBuf = crate::test_support::fixture("doctor-staging-removal-none");
    let store_path: PathBuf = root.join("store");
    std::fs::create_dir(&store_path).expect("store folder");
    let identity: CloneFolderIdentity = CloneFolderIdentity { device: 2049, inode: 42, owner: 1000, mode: 0o700, access_acl: None, default_acl: None };
    std::fs::write(store_path.join(PARENT_RECORD_NAME), format!("{STORE_HEADER}\nparent - {}\n", identity.encode())).expect("parent record");
    let store: SlotIdentityStore = SlotIdentityStore::open(store_path.as_path()).expect("the store opens");
    let removal: StagingRemoval = remove_known_staging_forms(StagingRemovalRequest { store: &store }).expect("the removal runs");
    assert!(removal.removed.is_empty(), "{removal:?}");
    assert_eq!(surviving(store_path.as_path()).len(), 1);
}

/// A store folder another account may not write answers a refusal naming it,
/// rather than a pass that removed nothing.
#[test]
fn a_store_the_account_may_not_write_is_a_refusal() {
    use std::os::unix::fs::PermissionsExt as _;
    let (store_path, _, _) = planted_store("doctor-staging-removal-readonly");
    let store: SlotIdentityStore = SlotIdentityStore::open(store_path.as_path()).expect("the store opens");
    std::fs::set_permissions(store_path.as_path(), std::fs::Permissions::from_mode(0o555)).expect("a folder this account may not write");
    let outcome: Result<StagingRemoval, crate::health::doctor_zfs_route::store_staging_removal::StagingRemovalRefusal> = remove_known_staging_forms(StagingRemovalRequest { store: &store });
    std::fs::set_permissions(store_path.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the folder is writable again for the cleanup");
    let refusal = outcome.expect_err("a folder this account may not write cannot be cleared");
    assert!(!refusal.why.is_empty());
}
