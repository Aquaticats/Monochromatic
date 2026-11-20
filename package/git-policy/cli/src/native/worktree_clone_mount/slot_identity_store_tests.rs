//! Controls for the slot identity store:
//! a store reads back what was written,
//! an absent one asks for the setup run,
//! a line this repository did not write refuses the whole read,
//! a slot is recorded once and never refreshed,
//! and the adoption's write keeps every slot record.

use super::*;
use crate::clone_folder_identity::CloneFolderIdentity;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// The header a test store starts with,
/// which the reader requires.
const HEADER: &str = "# monochromatic-git-policy-cli worktree clone slot identities, version 1";

/// An identity with the given inode,
/// so two records in one test never read alike.
fn identity(inode: u64) -> CloneFolderIdentity {
    return CloneFolderIdentity { device: 66304, inode, owner: 1000, mode: 0o755, access_acl: None, default_acl: None };
}

/// A store file holding `lines` below the header.
fn store(root: &Path, lines: &[String]) -> PathBuf {
    let path: PathBuf = root.join("slot-identities");
    let mut text: String = String::from(HEADER);
    text.push('\n');
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    std::fs::write(&path, text).expect("store");
    return path;
}

/// A parent record line for `identity`.
fn parent_line(identity: &CloneFolderIdentity) -> String {
    return format!("parent - {}", identity.encode());
}

/// A slot record line for `name` and `identity`.
fn slot_line(name: &str, identity: &CloneFolderIdentity) -> String {
    return format!("slot {name} {}", identity.encode());
}

/// What:
///  A store written by the setup run and the sweep reads back,
///  the parent and each slot under their own key.
/// Why:
///  Every later check compares against this one read,
///  so a lookup answering the wrong record would compare against another folder's identity.
#[test]
fn a_written_store_reads_back() {
    let root: PathBuf = fixture("slot-store-read");
    let parent: CloneFolderIdentity = identity(300);
    let first: CloneFolderIdentity = identity(301);
    let second: CloneFolderIdentity = identity(302);
    let path: PathBuf = store(root.as_path(), &[parent_line(&parent), slot_line("one", &first), String::from("# a comment is nothing"), slot_line("two", &second)]);
    let read: SlotIdentityStore = SlotIdentityStore::read(path.as_path()).expect("store reads");
    assert_eq!(read.path(), path.as_path());
    assert_eq!(read.parent(), Some(&parent));
    assert_eq!(read.slot("one"), Some(&first));
    assert_eq!(read.slot("two"), Some(&second));
    assert_eq!(read.slot("three"), None, "a slot nobody recorded is nothing, and its own refusal follows");
    assert_eq!(read.slot_names(), vec![String::from("one"), String::from("two")]);
    let bare: PathBuf = store(root.as_path(), &[]);
    let without_records: SlotIdentityStore = SlotIdentityStore::read(bare.as_path()).expect("a header alone is a store");
    assert_eq!(without_records.parent(), None);
    remove(root.as_path());
}

/// What:
///  An absent store refuses,
///  names the path,
///  and says the setup run writes it.
/// Why:
///  Creating it here would mean inventing the identity the attach must compare against.
#[test]
fn an_absent_store_asks_for_the_setup_run() {
    let root: PathBuf = fixture("slot-store-absent");
    let path: PathBuf = root.join("slot-identities");
    let refused: StoreRefusal = SlotIdentityStore::read(path.as_path()).expect_err("nothing there");
    assert!(matches!(refused, StoreRefusal::Missing { .. }), "{refused:?}");
    let message: String = refused.message();
    assert!(message.contains(path.to_string_lossy().as_ref()), "the path is named: {message}");
    assert!(message.contains("setup"), "the setup run is named: {message}");
    assert!(message.contains("Ways forward"), "the ways forward are given: {message}");
    let appended: StoreRefusal = append_slot_identity(path.as_path(), "one", &identity(1)).expect_err("nothing to add to");
    assert!(matches!(appended, StoreRefusal::Missing { .. }), "{appended:?}");
    assert!(!path.exists(), "an append never invents the store");
    remove(root.as_path());
}

/// What:
///  A file whose lines are not records of this repository refuses the whole read,
///  naming the path and the line,
///  while the store written the same way reads.
/// Why:
///  A skipped line could hide the record of the slot being checked,
///  and then a slot that was never verified would look new.
#[test]
fn a_line_this_repository_did_not_write_refuses_the_read() {
    let root: PathBuf = fixture("slot-store-lines");
    let good: CloneFolderIdentity = identity(310);
    let path: PathBuf = store(root.as_path(), &[parent_line(&good), slot_line("one", &good)]);
    assert!(SlotIdentityStore::read(path.as_path()).is_ok(), "positive control");
    let broken: [(&str, &str); 8] = [
        ("", "the file is empty"),
        ("# another tool's file\n", "the first line must be the store's own header"),
        (&format!("{HEADER}\nparent\n"), "a record holds three space-separated fields"),
        (&format!("{HEADER}\nparent - {} extra\n", good.encode()), "a record holds three space-separated fields"),
        (&format!("{HEADER}\nparent - 1:2\n"), "is not a recorded folder identity"),
        (&format!("{HEADER}\nfolder - {}\n", good.encode()), "is neither"),
        (&format!("{HEADER}\nparent one {}\n", good.encode()), "a parent record's key"),
        (&format!("{HEADER}\nslot one/two {}\n", good.encode()), "it holds a path separator"),
    ];
    for (text, expected) in broken {
        let bad: PathBuf = root.join("bad-store");
        std::fs::write(&bad, text).expect("bad store");
        let refused: StoreRefusal = SlotIdentityStore::read(bad.as_path()).expect_err(text);
        assert!(refused.message().contains(expected), "{expected} is in {}", refused.message());
        assert!(refused.message().contains(bad.to_string_lossy().as_ref()), "the path is named: {}", refused.message());
    }
    let numbered: PathBuf = root.join("bad-store");
    std::fs::write(&numbered, format!("{HEADER}\nparent - {}\nfolder - {}\n", good.encode(), good.encode())).expect("bad store");
    let refused: StoreRefusal = SlotIdentityStore::read(numbered.as_path()).expect_err("bad kind on line 3");
    assert!(refused.message().contains("line 3"), "the line is named: {}", refused.message());
    remove(root.as_path());
}

/// What:
///  One key recorded twice refuses the read,
///  naming the key and the line that repeats it.
/// Why:
///  Two identities for one folder decide nothing,
///  so the read must not pick either of them.
#[test]
fn one_key_recorded_twice_is_refused() {
    let root: PathBuf = fixture("slot-store-duplicate");
    let good: CloneFolderIdentity = identity(320);
    let path: PathBuf = store(root.as_path(), &[parent_line(&good), slot_line("one", &good), slot_line("one", &identity(321))]);
    let refused: StoreRefusal = SlotIdentityStore::read(path.as_path()).expect_err("one slot twice");
    assert!(matches!(refused, StoreRefusal::DuplicateKey { number: 4, ref key, .. } if key == "one"), "{refused:?}");
    assert!(refused.message().contains('4'), "the second line is named: {}", refused.message());
    let twice_parent: PathBuf = store(root.as_path(), &[parent_line(&good), parent_line(&identity(322))]);
    let refused: StoreRefusal = SlotIdentityStore::read(twice_parent.as_path()).expect_err("one parent twice");
    assert!(matches!(refused, StoreRefusal::DuplicateKey { ref key, .. } if key == "-"), "{refused:?}");
    remove(root.as_path());
}

/// What:
///  A new slot is appended,
///  and the same slot's second record is refused with the file unchanged.
/// Why:
///  Never refreshing a slot identity is what keeps a replaced folder from being adopted by accident.
#[test]
fn a_slot_is_recorded_once_and_never_refreshed() {
    let root: PathBuf = fixture("slot-store-append");
    let parent: CloneFolderIdentity = identity(330);
    let path: PathBuf = store(root.as_path(), &[parent_line(&parent)]);
    let recorded: CloneFolderIdentity = identity(331);
    append_slot_identity(path.as_path(), "one", &recorded).expect("a slot with no record is recorded");
    let read: SlotIdentityStore = SlotIdentityStore::read(path.as_path()).expect("store reads");
    assert_eq!(read.slot("one"), Some(&recorded));
    assert_eq!(read.parent(), Some(&parent), "an append leaves the parent record as it was");
    let before: Vec<u8> = std::fs::read(path.as_path()).expect("bytes");
    let refused: StoreRefusal = append_slot_identity(path.as_path(), "one", &identity(332)).expect_err("recorded already");
    assert!(matches!(refused, StoreRefusal::SlotAlreadyRecorded { ref slot, .. } if slot == "one"), "{refused:?}");
    assert_eq!(std::fs::read(path.as_path()).expect("bytes"), before, "the store is unchanged");
    assert!(refused.message().contains("never refreshes a slot identity"), "{}", refused.message());
    for refused_name in ["", "one/two", ".", "..", "with space", "with\ttab"] {
        let refused: StoreRefusal = append_slot_identity(path.as_path(), refused_name, &identity(333)).expect_err(refused_name);
        assert!(matches!(refused, StoreRefusal::SlotName { .. }), "{refused_name}: {refused:?}");
        assert!(refused.message().contains("no slot name"), "{}", refused.message());
    }
    let no_parent: PathBuf = store(root.as_path(), &[]);
    let refused: StoreRefusal = append_slot_identity(no_parent.as_path(), "one", &recorded).expect_err("nothing to compare a slot against");
    assert!(matches!(refused, StoreRefusal::NoParentRecord { .. }), "{refused:?}");
    remove(root.as_path());
}

/// What:
///  The adoption's write replaces the parent record and keeps every slot record,
///  both lines and both identities.
/// Why:
///  An adoption is the one write allowed to replace the parent's identity,
///  and it must never overwrite a surviving slot identity.
#[test]
fn the_adoption_replaces_only_the_parent_record() {
    let root: PathBuf = fixture("slot-store-adoption");
    let old_parent: CloneFolderIdentity = identity(340);
    let slot: CloneFolderIdentity = identity(341);
    let path: PathBuf = store(root.as_path(), &[parent_line(&old_parent), slot_line("one", &slot), String::from("# kept")]);
    let adopted: CloneFolderIdentity = identity(342);
    record_parent_identity(path.as_path(), &adopted).expect("the parent record is replaced");
    let read: SlotIdentityStore = SlotIdentityStore::read(path.as_path()).expect("store reads");
    assert_eq!(read.parent(), Some(&adopted));
    assert_eq!(read.slot("one"), Some(&slot), "a surviving slot identity is not overwritten");
    let text: String = std::fs::read_to_string(path.as_path()).expect("text");
    assert!(text.starts_with(HEADER), "the file keeps its header: {text}");
    assert!(text.contains("# kept"), "a comment stays: {text}");
    assert_eq!(text.lines().filter(|line| line.starts_with("parent ")).count(), 1, "one parent record: {text}");
    let missing: PathBuf = root.join("absent");
    assert!(matches!(record_parent_identity(missing.as_path(), &adopted), Err(StoreRefusal::Missing { .. })), "an adoption never invents the store");
    remove(root.as_path());
}

/// What:
///  A link planted at the store's name is refused by the read and by the append,
///  never followed.
/// Why:
///  The store sits under root control;
///  a link at its name would make the route compare against a file the account chose.
#[test]
fn a_link_at_the_store_name_is_never_read_through() {
    let root: PathBuf = fixture("slot-store-link");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::write(&elsewhere, format!("{HEADER}\nparent - {}\n", identity(350).encode())).expect("elsewhere");
    let path: PathBuf = root.join("slot-identities");
    std::os::unix::fs::symlink(&elsewhere, &path).expect("link");
    let refused: StoreRefusal = SlotIdentityStore::read(path.as_path()).expect_err("a link is not read through");
    assert!(matches!(refused, StoreRefusal::Unreadable { .. }), "{refused:?}");
    assert!(refused.message().contains("non-regular"), "{}", refused.message());
    let refused: StoreRefusal = append_slot_identity(path.as_path(), "one", &identity(351)).expect_err("a link is not written through");
    assert!(matches!(refused, StoreRefusal::Unreadable { .. }), "{refused:?}");
    remove(root.as_path());
}
