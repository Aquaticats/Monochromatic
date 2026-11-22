//! Controls for the slot identity store as a folder with one file per slot:
//! a publication is whole or absent at every instant,
//! exactly one of two recorders records a slot,
//! a folder that refuses a link publishes by the rename that never replaces,
//! a killed writer's staging entry is removed,
//! the per-file bound and the record's own key refuse what a writer outside this repository could make,
//! the parent record is replaced by one rename,
//! and the folder and every record answer the root-control terms.

use crate::worktree_clone_mount::slot_identity_store_publication::{
    BETWEEN_THE_BYTES_AND_THE_LINK, BETWEEN_THE_PARENT_BYTES_AND_THE_RENAME, THE_LINK_IS_ANSWERED_WITH, THE_RENAME_IS_ANSWERED_WITH, publish_parent_identity, publish_slot_identity,
    replace_parent_identity,
};
use crate::worktree_clone_mount::*;
use crate::clone_folder_identity::CloneFolderIdentity;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// An identity with the given inode,
/// so two records in one control never read alike.
fn identity(inode: u64) -> CloneFolderIdentity {
    return CloneFolderIdentity { device: 66304, inode, owner: 1000, mode: 0o755, access_acl: None, default_acl: None };
}

/// A slot record for `name`,
/// which is what a writer publishes.
fn slot_record(name: &str, inode: u64) -> StoredIdentity {
    return StoredIdentity::Slot { name: name.to_string(), identity: identity(inode) };
}

/// The store folder of one control,
/// holding no record.
fn store_folder(root: &Path) -> PathBuf {
    let folder: PathBuf = root.join("git-worktree-clone-slot-identities");
    std::fs::create_dir(folder.as_path()).expect("the store folder is made");
    return folder;
}

/// Write one record file below `folder`,
/// the way a writer of this repository publishes it.
fn write_record(folder: &Path, name: &str, record: &StoredIdentity) {
    let text: String = format!("{STORE_HEADER}\n{}\n", record.encode());
    std::fs::write(folder.join(name), text.as_bytes()).expect("the record file is written");
}

/// The store of one control:
/// a folder recording the parent `inode` and every given slot.
fn store(root: &Path, parent_inode: u64, slots: &[(&str, u64)]) -> (PathBuf, CloneFolderIdentity) {
    let folder: PathBuf = store_folder(root);
    let parent: CloneFolderIdentity = identity(parent_inode);
    write_record(folder.as_path(), PARENT_RECORD_NAME, &StoredIdentity::Parent(parent.clone()));
    for (name, inode) in slots {
        write_record(folder.as_path(), name, &slot_record(name, *inode));
    }
    return (folder, parent);
}

/// The names of the entries below `folder`,
/// sorted,
/// which is what the store's folder holds and nothing else.
fn entries(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder).expect("the folder lists").flatten().map(|entry| return entry.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    return names;
}

/// The bytes of one entry below `folder`.
fn bytes(folder: &Path, name: &str) -> Vec<u8> {
    return std::fs::read(folder.join(name)).expect("the entry reads");
}

/// Clear every publication hook,
/// so a control that panics does not leave one for the next.
fn clear_hooks() {
    BETWEEN_THE_BYTES_AND_THE_LINK.with(|hook| { *hook.borrow_mut() = None });
    BETWEEN_THE_PARENT_BYTES_AND_THE_RENAME.with(|hook| { *hook.borrow_mut() = None });
    THE_LINK_IS_ANSWERED_WITH.with(|answer| { answer.set(None) });
    THE_RENAME_IS_ANSWERED_WITH.with(|answer| { answer.set(None) });
}

/// What:
///  A publication holds every byte of its staging file before its record's name exists,
///  so a reader in that window sees no record and one staging entry,
///  and after the publication the record reads whole with no staging entry left.
/// Why:
///  A record's name must never hold a prefix of its bytes;
///  a kill in the window leaves the staging entry and no record,
/// which is control 4's state.
#[test]
fn a_record_is_published_whole_and_a_reader_never_sees_it_partly_written() {
    let root: PathBuf = fixture("store-published-whole");
    let (folder, parent) = store(root.as_path(), 300, &[]);
    let seen: std::sync::Arc<std::sync::Mutex<Vec<(Vec<String>, Option<Vec<u8>>)>>> = std::sync::Arc::default();
    let during: std::sync::Arc<std::sync::Mutex<Vec<(Vec<String>, Option<Vec<u8>>)>>> = std::sync::Arc::clone(&seen);
    let watched: PathBuf = folder.clone();
    BETWEEN_THE_BYTES_AND_THE_LINK.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            let record: Option<Vec<u8>> = std::fs::read(watched.join("one")).ok();
            during.lock().expect("the control's own list").push((entries(watched.as_path()), record));
        }));
    });
    publish_slot_identity(folder.as_path(), "one", &identity(301), &parent).expect("the slot is recorded");
    clear_hooks();
    let in_the_window = seen.lock().expect("the control's own list").clone();
    assert_eq!(in_the_window.len(), 1, "the window was read once");
    let (names, record) = &in_the_window[0];
    assert!(record.is_none(), "the record's name holds nothing in the window");
    assert_eq!(names.len(), 2, "the window holds the parent record and the staging entry: {names:?}");
    let staging: &String = names.iter().find(|name| return name.as_str() != PARENT_RECORD_NAME).expect("a staging entry");
    assert!(is_known_staging_name(staging.as_str()), "the staging entry is one of the two known forms: {staging}");
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME), String::from("one")], "the publication leaves the record and no staging entry");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert_eq!(read.slot("one").expect("the slot reads"), identity(301));
    assert_eq!(bytes(folder.as_path(), "one"), format!("{STORE_HEADER}\n{}\n", slot_record("one", 301).encode()).into_bytes(), "the record file is its header and one record line");
    remove(root.as_path());
}

/// What:
///  Two recorders of one slot leave exactly one record,
///  the winner's bytes,
///  no staging entry of either,
///  and the loser's `SlotAlreadyRecorded` carries the record that won.
/// Why:
///  The store takes no lock,
/// so the kernel's own exclusivity at the record's name is the whole answer:
///  a loser that rewrote the record or failed the sweep would lose the identity every attach compares against.
#[test]
fn two_recorders_of_one_slot_publish_once_and_the_second_reads_the_first() {
    let root: PathBuf = fixture("store-two-recorders");
    let (folder, parent) = store(root.as_path(), 310, &[]);
    let second: CloneFolderIdentity = identity(312);
    let raced_by: CloneFolderIdentity = second.clone();
    let outcome: std::sync::Arc<std::sync::Mutex<Vec<Result<(), StoreRefusal>>>> = std::sync::Arc::default();
    let held: std::sync::Arc<std::sync::Mutex<Vec<Result<(), StoreRefusal>>>> = std::sync::Arc::clone(&outcome);
    let raced: PathBuf = folder.clone();
    let raced_parent: CloneFolderIdentity = parent.clone();
    BETWEEN_THE_BYTES_AND_THE_LINK.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            // The second recorder publishes while this one holds every byte and no name yet.
            let wrote: Result<(), StoreRefusal> = publish_slot_identity(raced.as_path(), "one", &raced_by, &raced_parent);
            held.lock().expect("the control's own list").push(wrote);
        }));
    });
    let first: Result<(), StoreRefusal> = publish_slot_identity(folder.as_path(), "one", &identity(311), &parent);
    clear_hooks();
    let second_wrote = outcome.lock().expect("the control's own list").clone();
    assert_eq!(second_wrote.len(), 1, "the second recorder ran once, inside the window");
    assert_eq!(second_wrote[0], Ok(()), "the second recorder published first and won");
    let loser: StoreRefusal = first.expect_err("the first recorder lost the race");
    assert!(matches!(loser, StoreRefusal::SlotAlreadyRecorded { ref slot, .. } if slot == "one"), "{loser:?}");
    assert!(matches!(loser, StoreRefusal::SlotAlreadyRecorded { identity: ref carried, .. } if carried == &second), "the loser carries the record that won");
    assert!(loser.message().contains("the record that won"), "the refusal says which record stands: {}", loser.message());
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME), String::from("one")], "one record and neither staging entry");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert_eq!(read.slot("one").expect("the slot reads"), second, "the record holds the winner's identity");
    remove(root.as_path());
}

/// What:
///  A publication whose link fails leaves no record and no staging entry,
///  and the same on the fallback's rename.
/// Why:
///  A failed write must not leave a name a reader could read or a remover must find;
///  the second arm runs the fallback a folder without hard links takes.
#[test]
fn a_publication_that_fails_leaves_no_record_and_no_staging_entry() {
    let root: PathBuf = fixture("store-failed-publication");
    let (folder, parent) = store(root.as_path(), 320, &[]);
    THE_LINK_IS_ANSWERED_WITH.with(|answer| { answer.set(Some(libc::EIO)) });
    let refused: StoreRefusal = publish_slot_identity(folder.as_path(), "one", &identity(321), &parent).expect_err("the link failed");
    assert!(matches!(refused, StoreRefusal::Write { .. }), "{refused:?}");
    assert!(refused.message().contains(folder.to_string_lossy().as_ref()), "the folder is named: {}", refused.message());
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME)], "the failed link left no record and no staging entry");
    THE_LINK_IS_ANSWERED_WITH.with(|answer| { answer.set(Some(libc::EPERM)) });
    THE_RENAME_IS_ANSWERED_WITH.with(|answer| { answer.set(Some(libc::EIO)) });
    let rename_refused: StoreRefusal = publish_slot_identity(folder.as_path(), "two", &identity(322), &parent).expect_err("both calls failed");
    clear_hooks();
    assert!(matches!(rename_refused, StoreRefusal::Write { .. }), "{rename_refused:?}");
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME)], "the failed rename left no record and no staging entry");
    remove(root.as_path());
}

/// What:
///  A writer killed between its bytes and its link leaves one staging entry and no record,
///  and one killed between its link and its own removal leaves the record whole beside a staging entry naming the same file.
/// Why:
///  Both states must be readable:
/// no reader reads a staging entry,
///  and the record a kill left is whole because a link publishes every byte at once.
#[test]
fn a_killed_writer_leaves_one_staging_entry_and_no_record() {
    let root: PathBuf = fixture("store-killed-writer");
    let (folder, _parent) = store(root.as_path(), 330, &[]);
    let before_the_link: String = format!(".{}", "a".repeat(32));
    std::fs::write(folder.join(before_the_link.as_str()), format!("{STORE_HEADER}\n{}\n", slot_record("one", 331).encode())).expect("a staging file a kill left");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert!(matches!(read.slot("one"), Err(StoreRefusal::NoSlotRecord { .. })), "the staging entry is no record");
    assert_eq!(read.slot_names().expect("the listing ignores a staging entry"), Vec::<String>::new(), "a staging entry is not a slot");
    assert_eq!(entries(folder.as_path()), vec![before_the_link.clone(), String::from(PARENT_RECORD_NAME)], "one staging entry and no record");
    // A kill between the link and the staging name's removal leaves the record whole beside the name that also holds it.
    let published: String = format!(".{}", "b".repeat(32));
    std::fs::write(folder.join(published.as_str()), format!("{STORE_HEADER}\n{}\n", slot_record("two", 332).encode())).expect("a staging file a kill left");
    std::fs::hard_link(folder.join(published.as_str()), folder.join("two")).expect("the link a kill landed after");
    let read_after: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert_eq!(read_after.slot("two").expect("the record reads whole"), identity(332));
    let staging_links: u64 = std::fs::metadata(folder.join(published.as_str())).expect("the staging entry's metadata").nlink();
    assert_eq!(staging_links, 2, "the staging name and the record name hold one file");
    remove(root.as_path());
}

/// What:
///  The removal takes both known staging forms and nothing else:
///  the records stay as they were,
///  and a dot entry matching neither form is left for an administrator.
/// Why:
///  The two forms are what this repository's writers make;
///  a third form means a writer this layout has not accounted for,
///  which only `doctor` reports and nobody removes.
#[test]
fn the_adoption_removes_the_staging_entries_a_killed_writer_left() {
    let root: PathBuf = fixture("store-staging-removal");
    let (folder, _parent) = store(root.as_path(), 340, &[("one", 341)]);
    let record_before: Vec<u8> = bytes(folder.as_path(), "one");
    let slot_staging: String = format!(".{}", "c".repeat(32));
    let parent_staging: String = String::from("..parent.0123456789abcdef.partial");
    let unknown: String = String::from(".a-third-form");
    for name in [slot_staging.as_str(), parent_staging.as_str(), unknown.as_str()] {
        std::fs::write(folder.join(name), b"staged bytes").expect("a staging entry");
    }
    let store_read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    let mut removed: Vec<String> = store_read.remove_staging_entries().expect("the known forms are removed");
    removed.sort();
    assert_eq!(removed, vec![parent_staging.clone(), slot_staging.clone()], "exactly the two known forms are removed and answered");
    assert_eq!(entries(folder.as_path()), vec![unknown.clone(), String::from(PARENT_RECORD_NAME), String::from("one")], "the records stay and the unknown dot entry is left");
    assert_eq!(bytes(folder.as_path(), "one"), record_before, "the record is untouched");
    remove(root.as_path());
}

/// What:
///  A folder that refuses a hard link publishes the record by the rename that never replaces,
///  with the staging name gone.
/// Why:
///  A publication must fall back only where the folder makes no links,
///  and the fallback keeps the one-winner property and the cleanup.
#[test]
fn the_publication_falls_back_where_the_folder_refuses_a_link() {
    let root: PathBuf = fixture("store-link-fallback");
    let (folder, parent) = store(root.as_path(), 350, &[]);
    THE_LINK_IS_ANSWERED_WITH.with(|answer| { answer.set(Some(libc::EPERM)) });
    publish_slot_identity(folder.as_path(), "one", &identity(351), &parent).expect("the rename publishes the record");
    clear_hooks();
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME), String::from("one")], "the record is published and the staging name is gone");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert_eq!(read.slot("one").expect("the slot reads"), identity(351));
    remove(root.as_path());
}

/// What:
///  A folder that refuses both the link and the rename that never replaces refuses the write,
///  names the folder and both calls,
///  and leaves no record and no staging entry.
/// Why:
///  The one refusal must not fall back to a rename that could replace another run's record,
///  and it must say which calls the folder answered.
#[test]
fn the_publication_refuses_where_the_folder_refuses_both_a_link_and_rename_that_never_replaces() {
    let root: PathBuf = fixture("store-both-refused");
    let (folder, parent) = store(root.as_path(), 360, &[]);
    THE_LINK_IS_ANSWERED_WITH.with(|answer| { answer.set(Some(libc::EOPNOTSUPP)) });
    THE_RENAME_IS_ANSWERED_WITH.with(|answer| { answer.set(Some(libc::EINVAL)) });
    let refused: StoreRefusal = publish_slot_identity(folder.as_path(), "one", &identity(361), &parent).expect_err("the folder makes neither a link nor a no-replace rename");
    clear_hooks();
    assert!(matches!(refused, StoreRefusal::Write { .. }), "{refused:?}");
    let message: String = refused.message();
    assert!(message.contains(folder.to_string_lossy().as_ref()), "the folder is named: {message}");
    assert!(message.contains("link") && message.contains("rename"), "both calls are named: {message}");
    assert!(message.contains("nothing is published by a rename that could replace"), "the refusal says what it will not do: {message}");
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME)], "nothing was published and no staging entry is left");
    remove(root.as_path());
}

/// What:
///  A reader racing the parent's replacement reads the old record whole in the window,
///  `.parent` is never absent,
///  and the new identity stands afterwards.
/// Why:
///  The replacement is one rename,
///  so a reader sees either whole record and never a third state,
///  and a stale read refuses rather than permitting an attach elsewhere.
#[test]
fn a_parent_record_is_replaced_by_one_rename_and_a_reader_sees_one_whole_record() {
    let root: PathBuf = fixture("store-parent-replaced");
    let (folder, parent) = store(root.as_path(), 370, &[]);
    let seen: std::sync::Arc<std::sync::Mutex<Vec<CloneFolderIdentity>>> = std::sync::Arc::default();
    let during: std::sync::Arc<std::sync::Mutex<Vec<CloneFolderIdentity>>> = std::sync::Arc::clone(&seen);
    let watched: PathBuf = folder.clone();
    BETWEEN_THE_PARENT_BYTES_AND_THE_RENAME.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            let read: SlotIdentityStore = SlotIdentityStore::open(watched.as_path()).expect("the store opens in the window");
            assert_eq!(read.parent().expect("the record is never absent"), parent, "the reader in the window reads the old record");
            during.lock().expect("the control's own list").push(read.parent().expect("the old record"));
        }));
    });
    replace_parent_identity(folder.as_path(), &identity(371)).expect("the parent record is replaced");
    clear_hooks();
    assert_eq!(seen.lock().expect("the control's own list").len(), 1, "a reader stood in the window");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert_eq!(read.parent().expect("the new record"), identity(371));
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME)], "one rename leaves the record and no staging name");
    remove(root.as_path());
}

/// What:
///  The adoption's replacement writes no slot file:
///  every slot file keeps its bytes,
///  its inode and its modification time.
/// Why:
///  An adoption must never overwrite a surviving slot identity,
///  and the layout meets that with its shape:
///  `.parent` holds nothing but the parent's identity.
#[test]
fn the_adoption_writes_no_slot_file() {
    let root: PathBuf = fixture("store-adoption-slot-files");
    let (folder, _parent) = store(root.as_path(), 380, &[("one", 381), ("two", 382)]);
    let before: Vec<(u64, i64, Vec<u8>)> = ["one", "two"]
        .iter()
        .map(|name| {
            let metadata: std::fs::Metadata = std::fs::metadata(folder.join(name)).expect("the slot file's metadata");
            return (metadata.ino(), metadata.mtime(), bytes(folder.as_path(), name));
        })
        .collect();
    replace_parent_identity(folder.as_path(), &identity(383)).expect("the parent record is replaced");
    let after: Vec<(u64, i64, Vec<u8>)> = ["one", "two"]
        .iter()
        .map(|name| {
            let metadata: std::fs::Metadata = std::fs::metadata(folder.join(name)).expect("the slot file's metadata");
            return (metadata.ino(), metadata.mtime(), bytes(folder.as_path(), name));
        })
        .collect();
    assert_eq!(after, before, "every slot file keeps its inode, its modification time and its bytes");
    remove(root.as_path());
}

/// What:
///  A publisher whose walked parent differs from `.parent` publishes nothing,
///  naming every field that moved.
/// Why:
///  A record taken under a parent nobody compared would be a first observation an adoption could not trust,
///  and the accepted design refuses managed mutations on a mismatched parent.
#[test]
fn a_publisher_whose_walked_parent_differs_from_the_parent_record_publishes_nothing() {
    let root: PathBuf = fixture("store-parent-differs");
    let (folder, _parent) = store(root.as_path(), 390, &[]);
    let mut walked: CloneFolderIdentity = identity(391);
    walked.mode = 0o700;
    let refused: StoreRefusal = publish_slot_identity(folder.as_path(), "one", &identity(392), &walked).expect_err("the parent moved");
    assert!(matches!(refused, StoreRefusal::ParentIdentity { ref fields, .. } if fields.contains(&crate::clone_folder_identity::CloneFolderField::Inode) && fields.contains(&crate::clone_folder_identity::CloneFolderField::Mode)), "{refused:?}");
    let message: String = refused.message();
    assert!(message.contains("inode") && message.contains("mode"), "every field that moved is named: {message}");
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME)], "no record and no staging entry");
    remove(root.as_path());
}

/// What:
///  A publisher in a store with no parent record publishes nothing,
///  and a store folder that is not there refuses before anything is created.
/// Why:
///  `.parent` is what a slot's record is taken against,
///  and the setup run is what writes it.
#[test]
fn a_publisher_in_a_store_with_no_parent_record_publishes_nothing() {
    let root: PathBuf = fixture("store-no-parent");
    let folder: PathBuf = store_folder(root.as_path());
    let refused: StoreRefusal = publish_slot_identity(folder.as_path(), "one", &identity(400), &identity(401)).expect_err("nothing to compare a slot against");
    assert!(matches!(refused, StoreRefusal::NoParentRecord { .. }), "{refused:?}");
    assert!(refused.message().contains("setup"), "the setup run is named: {}", refused.message());
    assert_eq!(entries(folder.as_path()), Vec::<String>::new(), "nothing was created");
    let absent: PathBuf = root.join("absent");
    let missing: StoreRefusal = publish_slot_identity(absent.as_path(), "one", &identity(400), &identity(401)).expect_err("the folder is not there");
    assert!(matches!(missing, StoreRefusal::Missing { .. }), "{missing:?}");
    assert!(!absent.exists(), "a publication never invents the store folder");
    remove(root.as_path());
}

/// What:
///  Every name that is no plain slot name is refused before anything is created.
/// Why:
///  A name with a slash,
///  a dot name or a name with a space could not be a record this folder files by a slot's name,
///  and a refusal that created a file first would leave one no reader reads.
#[test]
fn a_publisher_refuses_a_name_that_is_no_plain_slot_name() {
    let root: PathBuf = fixture("store-slot-names");
    let (folder, parent) = store(root.as_path(), 410, &[]);
    for refused_name in ["", "one/two", ".", "..", ".hidden", "with space", "with\ttab"] {
        let refused: StoreRefusal = publish_slot_identity(folder.as_path(), refused_name, &identity(411), &parent).expect_err(refused_name);
        assert!(matches!(refused, StoreRefusal::SlotName { .. }), "{refused_name}: {refused:?}");
        assert!(refused.message().contains("no slot name"), "{}", refused.message());
    }
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME)], "no refused name created anything");
    remove(root.as_path());
}

/// The widest identity this route's capture can make:
/// two ACLs of `MAXIMUM_ACL_BYTES` and the widest digits each other field can hold,
/// which is the token the bound's arithmetic is about.
fn widest_identity(inode: u64) -> CloneFolderIdentity {
    let mut widest: CloneFolderIdentity = identity(inode);
    widest.device = u64::MAX;
    widest.inode = u64::MAX;
    widest.owner = u32::MAX;
    widest.mode = 0o7777;
    widest.access_acl = Some(vec![0x02; MAXIMUM_ACL_BYTES]);
    widest.default_acl = Some(vec![0x02; MAXIMUM_ACL_BYTES]);
    return widest;
}

/// What:
///  A record file of exactly the bound reads,
///  and a file one byte over it is refused by name and size;
///  a publication whose record would be one byte over refuses before it creates anything.
/// Why:
///  The bound is derived from the kernel's own limits,
///  so the widest record a writer of this repository makes must read,
///  and a wider file can only come from a writer outside it.
#[test]
fn a_record_larger_than_the_bound_is_refused_by_name_and_size() {
    let root: PathBuf = fixture("store-record-bound");
    let folder: PathBuf = store_folder(root.as_path());
    let widest: StoredIdentity = StoredIdentity::Slot { name: "a".repeat(255), identity: widest_identity(420) };
    let text: String = format!("{STORE_HEADER}\n{}\n", widest.encode());
    assert_eq!(text.len() as u64, RECORD_MAXIMUM_BYTES, "the widest record this repository makes is exactly the bound");
    std::fs::write(folder.join("a".repeat(255)), text.as_bytes()).expect("a record at the bound");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert_eq!(read.slot("a".repeat(255).as_str()).expect("the record at the bound reads"), widest_identity(420));
    // A hundred bytes over, not one: the read's own cap reports the bound it reads and not the file's size, so a
    // file of the bound plus one could answer either way and the control would prove neither.
    let over: String = format!("{STORE_HEADER}\n{}\n{}", widest.encode(), "#".repeat(100));
    let over_name: String = "b".repeat(255);
    let over_size: u64 = over.len() as u64;
    assert_eq!(over_size, RECORD_MAXIMUM_BYTES + 100, "the fixture is a hundred bytes over the bound");
    std::fs::write(folder.join(over_name.as_str()), over.as_bytes()).expect("a record over the bound");
    let refused: StoreRefusal = read.slot(over_name.as_str()).expect_err("a hundred bytes over the bound");
    assert!(matches!(refused, StoreRefusal::RecordTooLarge { bytes, bound, .. } if bytes == over_size && bound == RECORD_MAXIMUM_BYTES), "{refused:?}");
    let message: String = refused.message();
    assert!(message.contains(&RECORD_MAXIMUM_BYTES.to_string()), "the bound is named: {message}");
    assert!(message.contains(&over_size.to_string()), "the file's size is named: {message}");
    // A publication whose record would be one byte over refuses before it creates even a staging name.
    let live: CloneFolderIdentity = identity(422);
    write_record(folder.as_path(), PARENT_RECORD_NAME, &StoredIdentity::Parent(live.clone()));
    let publication: StoreRefusal = publish_slot_identity(folder.as_path(), "c".repeat(256).as_str(), &widest_identity(421), &live).expect_err("one byte over the bound");
    assert!(matches!(publication, StoreRefusal::RecordTooLarge { bytes, .. } if bytes == RECORD_MAXIMUM_BYTES + 1), "{publication:?}");
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME), "a".repeat(255), over_name.clone()], "a refused publication created nothing");
    remove(root.as_path());
}

/// What:
///  A good record copied and hard-linked under another name refuses,
///  naming the name it is filed under and the key it carries.
/// Why:
///  A record's key is the name it is filed under,
///  so a record for one folder cannot decide another slot's binding.
#[test]
fn a_record_whose_key_is_not_its_file_name_is_refused() {
    let root: PathBuf = fixture("store-another-name");
    let (folder, _parent) = store(root.as_path(), 430, &[("one", 431)]);
    std::fs::copy(folder.join("one"), folder.join("two")).expect("a copied record");
    std::fs::hard_link(folder.join("one"), folder.join("three")).expect("a hard-linked record");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    for name in ["two", "three"] {
        let refused: StoreRefusal = read.slot(name).expect_err("the record's key is another name");
        assert!(matches!(refused, StoreRefusal::RecordUnderAnotherName { ref filed_under, ref recorded, .. } if filed_under == name && recorded == "one"), "{refused:?}");
        assert!(refused.message().contains(name), "the name it is filed under is named: {}", refused.message());
        assert!(refused.message().contains('`' ), "the key it carries is named: {}", refused.message());
    }
    remove(root.as_path());
}

/// What:
///  A name that is neither a record this layout names nor a staging entry refuses the listing,
///  while both known staging forms and an unknown dot entry are left out of it;
///  the removal takes the known forms only.
/// Why:
///  A foreign name could be a slot whose record a later read would miss,
///  so the whole set the adoption verifies must refuse it.
#[test]
fn a_foreign_entry_refuses_the_listing_and_every_staging_form_does_not() {
    let root: PathBuf = fixture("store-foreign-entry");
    let (folder, _parent) = store(root.as_path(), 440, &[("one", 441)]);
    let slot_staging: String = format!(".{}", "d".repeat(32));
    let parent_staging: String = String::from("..parent.fedcba9876543210.partial");
    for name in [slot_staging.as_str(), parent_staging.as_str(), ".unknown-form"] {
        std::fs::write(folder.join(name), b"staged").expect("a staging entry");
    }
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert_eq!(read.slot_names().expect("the staging entries are left out"), vec![String::from("one")]);
    std::fs::write(folder.join("with space"), format!("{STORE_HEADER}\n")).expect("a foreign entry");
    let refused: StoreRefusal = read.slot_names().expect_err("a foreign name refuses the listing");
    clear_hooks();
    assert!(matches!(refused, StoreRefusal::ForeignEntry { ref entry, .. } if entry == "with space"), "{refused:?}");
    assert!(refused.message().contains("with space"), "the entry is named: {}", refused.message());
    std::fs::remove_file(folder.join("with space")).expect("the foreign entry goes");
    assert_eq!(read.slot_names().expect("the listing reads again"), vec![String::from("one")]);
    remove(root.as_path());
}

/// What:
///  An absent store folder and an absent record each refuse with their own cause,
///  and a link at a record's name is never read through.
/// Why:
///  A missing folder asks for the setup run,
///  a missing record asks for the sweep that publishes it,
///  and a link at a record's name would make the route compare against a file the account chose.
#[test]
fn an_absent_store_folder_asks_for_the_setup_run_and_a_link_at_a_record_name_is_never_read_through() {
    let root: PathBuf = fixture("store-absent-and-link");
    let absent: PathBuf = root.join("absent");
    let refused: StoreRefusal = SlotIdentityStore::open(absent.as_path()).expect_err("nothing there");
    assert!(matches!(refused, StoreRefusal::Missing { .. }), "{refused:?}");
    let message: String = refused.message();
    assert!(message.contains(absent.to_string_lossy().as_ref()), "the path is named: {message}");
    assert!(message.contains("setup") && message.contains("Ways forward"), "the setup run and the ways forward are named: {message}");
    let (folder, _parent) = store(root.as_path(), 450, &[]);
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::write(elsewhere.as_path(), format!("{STORE_HEADER}\n{}\n", slot_record("one", 451).encode())).expect("the link's target");
    std::os::unix::fs::symlink(elsewhere.as_path(), folder.join("one")).expect("a link at a record's name");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    let linked: StoreRefusal = read.slot("one").expect_err("a link is not read through");
    assert!(matches!(linked, StoreRefusal::Unreadable { .. }), "{linked:?}");
    assert!(linked.message().contains("non-regular"), "{}", linked.message());
    remove(root.as_path());
}

/// What:
///  A second setup run whose live parent equals the record succeeds,
///  and one whose live parent differs refuses `ParentAlreadyRecorded` and writes nothing.
/// Why:
///  The setup run's boot unit runs on every boot,
///  so a second run is the normal case and must not report an error,
///  while only the administrator's adoption may replace the record.
#[test]
fn a_second_setup_run_compares_the_live_parent_with_the_parent_record() {
    let root: PathBuf = fixture("store-second-setup");
    let folder: PathBuf = store_folder(root.as_path());
    let live: CloneFolderIdentity = identity(460);
    publish_parent_identity(folder.as_path(), &live, &live).expect("the first setup run publishes the parent record");
    publish_parent_identity(folder.as_path(), &live, &live).expect("a second run finds the same folder and succeeds");
    let mut moved: CloneFolderIdentity = live.clone();
    moved.inode = 461;
    let refused: StoreRefusal = publish_parent_identity(folder.as_path(), &moved, &moved).expect_err("the live folder is not the recorded one");
    assert!(matches!(refused, StoreRefusal::ParentAlreadyRecorded { ref fields, .. } if fields.contains(&crate::clone_folder_identity::CloneFolderField::Inode)), "{refused:?}");
    let message: String = refused.message();
    assert!(message.contains("adopt-worktree-parent"), "the adoption is named: {message}");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    assert_eq!(read.parent().expect("the record is the first run's"), live, "a refused second run replaces nothing");
    assert_eq!(entries(folder.as_path()), vec![String::from(PARENT_RECORD_NAME)], "no staging entry is left");
    remove(root.as_path());
}

/// What:
///  The store reads a record file whose grammar is the one a control writes,
///  and a partial file refuses naming the line where it stopped.
/// Why:
///  A record's name must hold its header and one record line,
///  and a partial file is the administrator's to repair rather than a state to retry.
#[test]
fn a_partial_record_refuses_naming_the_line_where_it_stopped() {
    let root: PathBuf = fixture("store-partial");
    let (folder, _parent) = store(root.as_path(), 470, &[("one", 471)]);
    std::fs::write(folder.join("two"), format!("{STORE_HEADER}\n")).expect("a header and no record");
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the store opens");
    let refused: StoreRefusal = read.slot("two").expect_err("a header alone is no record");
    assert!(matches!(refused, StoreRefusal::Line { .. }), "{refused:?}");
    assert!(refused.message().contains("no record line"), "{}", refused.message());
    std::fs::write(folder.join("three"), format!("{STORE_HEADER}\n{}\n{}\n", slot_record("three", 472).encode(), slot_record("three", 473).encode())).expect("two record lines");
    let twice: StoreRefusal = read.slot("three").expect_err("a record file holds one record");
    assert!(matches!(twice, StoreRefusal::Line { number: 3, .. }), "{twice:?}");
    assert!(twice.message().contains("line 3"), "the second record line is named: {}", twice.message());
    remove(root.as_path());
}

/// Environment variable that makes the control below run its inner half inside a user namespace,
/// where this run's own files are root's.
#[cfg(target_os = "linux")]
const ROOT_SIDE_VARIABLE: &str = "SLOT_IDENTITY_STORE_FOLDER_ROOT_SIDE";

/// What:
///  A store folder and its records written by root read under root control,
///  a record another account could write is refused,
///  and so is a folder another account could write.
/// Why:
///  A record decides which folder a clone may be attached to,
///  so a record or a folder this account could rewrite decides nothing,
///  and the refusal must name which of the two it is.
#[cfg(target_os = "linux")]
#[test]
fn a_record_this_account_could_rewrite_is_refused() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        ROOT_SIDE_VARIABLE,
        "worktree_clone_mount::slot_identity_store::tests::a_record_this_account_could_rewrite_is_refused",
    ) {
        return;
    }
    let root: PathBuf = fixture("store-root-side-records");
    let (folder, parent) = store(root.as_path(), 480, &[("one", 481)]);
    let read: SlotIdentityStore = SlotIdentityStore::open_under_root_control(folder.as_path()).expect("a store root wrote reads");
    assert_eq!(read.parent().expect("the parent record reads"), parent);
    std::fs::set_permissions(folder.join("one"), std::fs::Permissions::from_mode(0o666)).expect("a record another account can write");
    let record_refused: StoreRefusal = SlotIdentityStore::open_under_root_control(folder.as_path()).expect("the folder is root's").slot("one").expect_err("a record another account can write is refused");
    assert!(matches!(record_refused, StoreRefusal::NotUnderRootControl { ref what, .. } if what == "the record file"), "{record_refused:?}");
    assert!(record_refused.message().contains("0666"), "the mode is named: {}", record_refused.message());
    std::fs::set_permissions(folder.join("one"), std::fs::Permissions::from_mode(0o644)).expect("the record back to root's own");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o777)).expect("the folder widened");
    let folder_refused: StoreRefusal = SlotIdentityStore::open_under_root_control(folder.as_path()).expect_err("a folder this account could write decides nothing");
    assert!(matches!(folder_refused, StoreRefusal::NotUnderRootControl { ref what, .. } if what == "the store folder"), "{folder_refused:?}");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(STORE_FOLDER_MODE)).expect("the folder back to root's own");
    let plain: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the grammar's own open reads the same folder");
    assert_eq!(plain.slot("one").expect("the grammar's reader reads it"), identity(481));
    remove(root.as_path());
}

/// What:
///  The truth table of the root-control check:
///  root with no group or other write bit is under root control,
///  an owner that is not root is not,
///  and so is a mode another account can write.
/// Why:
///  The folder and every record answer the same two terms,
///  and a control proves the terms themselves rather than a fixture's owner.
#[test]
fn the_control_says_which_owner_and_mode_are_root_s_own() {
    assert_eq!(store_control_refusal(STORE_OWNER, RECORD_MODE), None);
    assert_eq!(store_control_refusal(STORE_OWNER, 0o600), None);
    for owner in [1, 1000, u32::MAX] {
        let refused: String = store_control_refusal(owner, 0o600).expect("an owner that is not root");
        assert!(refused.contains(&owner.to_string()), "{refused}");
        assert!(refused.contains("not root"), "{refused}");
    }
    for mode in [0o664, 0o666, 0o622, 0o602] {
        let refused: String = store_control_refusal(STORE_OWNER, mode).expect("a mode another account can write");
        assert!(refused.contains(&format!("{mode:04o}")), "{refused}");
    }
}

/// What:
///  The example store of the design's section 1 is a store this reader reads:
///  `.parent` holding the specification's own access ACL,
///  and one record file per slot.
/// Why:
///  A person copies those files into the setup run's store,
///  so an example the reader refuses teaches two wrong things at once:
///  that a comment may stand above the header,
///  and that any bytes are a kernel ACL.
#[test]
fn the_specification_s_store_example_is_a_store_this_reader_reads() {
    let root: PathBuf = fixture("store-example");
    let folder: PathBuf = store_folder(root.as_path());
    let spec: String = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/SPEC.md")).expect("the specification");
    let example: Vec<String> = the_store_example(spec.as_str());
    assert_eq!(example.len(), 3, "the section shows three files: the parent's and two slots'");
    for (index, name) in [PARENT_RECORD_NAME, "add-cache-layer", "fix-flaky-test"].iter().enumerate() {
        std::fs::write(folder.join(name), example[index].as_bytes()).expect("the specification's example file");
    }
    let read: SlotIdentityStore = SlotIdentityStore::open(folder.as_path()).expect("the specification's example is a store this reader reads");
    let parent: CloneFolderIdentity = read.parent().expect("the example records the parent folder");
    assert!(read.slot("add-cache-layer").is_ok() && read.slot("fix-flaky-test").is_ok(), "the example records two slots");
    assert_eq!(read.slot_names().expect("the listing reads"), vec![String::from("add-cache-layer"), String::from("fix-flaky-test")]);
    let access: &[u8] = parent.access_acl.as_deref().expect("the example shows an access ACL");
    assert_eq!(access.get(..4), Some([0x02, 0x00, 0x00, 0x00].as_slice()), "the kernel's version word comes first");
    assert_eq!(access.len() % 8, 4, "a version word and then eight bytes per ACL entry, and no other length is a blob");
    let tags: Vec<u16> = access[4..].as_chunks::<8>().0.iter().map(|entry| return u16::from_le_bytes([entry[0], entry[1]])).collect();
    assert!(tags.contains(&0x10), "a stored ACL with a named entry carries a mask: {tags:?}");
    remove(root.as_path());
}

/// The files of the specification's store example,
/// which is the first three `text` blocks of the store section,
/// as each file would hold them.
fn the_store_example(spec: &str) -> Vec<String> {
    let heading: usize = spec.find("#### The recorded identities store").expect("the store section");
    let mut files: Vec<String> = Vec::new();
    let mut cursor: usize = heading;
    while files.len() < 3 {
        let Some(fence) = spec[cursor..].find("```text\n") else {
            break;
        };
        let opening: usize = cursor + fence + "```text\n".len();
        let closing: usize = opening + spec[opening..].find("```").expect("the block's closing fence");
        let block: &str = &spec[opening..closing];
        // Only the example's file blocks hold a record this grammar reads.
        if block.starts_with(STORE_HEADER) {
            files.push(String::from(block));
        }
        cursor = closing;
    }
    return files;
}
