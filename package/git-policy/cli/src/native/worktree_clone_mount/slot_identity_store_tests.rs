//! Controls for the slot identity store:
//! a store reads back what was written,
//! an absent one asks for the setup run,
//! a line this repository did not write refuses the whole read,
//! a slot is recorded once and never refreshed,
//! and the adoption's write keeps every slot record.

use super::*;
use crate::clone_folder_identity::CloneFolderIdentity;
use crate::test_support::{fixture, remove};
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
    assert!(!store_lock_path(path.as_path()).exists(), "an append never invents the lock beside a store that is not there");
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
    let broken: Vec<(String, &str)> = vec![
        (String::new(), "the file is empty"),
        (String::from("# another tool's file\n"), "the first line must be the store's own header"),
        (format!("{HEADER}\nparent\n"), "a record holds three space-separated fields"),
        (format!("{HEADER}\nparent - {} extra\n", good.encode()), "a record holds three space-separated fields"),
        (format!("{HEADER}\nparent - 1:2\n"), "is not a recorded folder identity"),
        (format!("{HEADER}\nfolder - {}\n", good.encode()), "is neither"),
        (format!("{HEADER}\nparent one {}\n", good.encode()), "a parent record's key"),
        (format!("{HEADER}\nslot one/two {}\n", good.encode()), "it holds a path separator"),
    ];
    for (text, expected) in &broken {
        let bad: PathBuf = root.join("bad-store");
        std::fs::write(&bad, text).expect("bad store");
        let refused_2: StoreRefusal = SlotIdentityStore::read(bad.as_path()).expect_err(text.as_str());
        assert!(refused_2.message().contains(expected), "{expected} is in {}", refused_2.message());
        assert!(refused_2.message().contains(bad.to_string_lossy().as_ref()), "the path is named: {}", refused_2.message());
    }
    let numbered: PathBuf = root.join("bad-store");
    std::fs::write(&numbered, format!("{HEADER}\nparent - {}\nfolder - {}\n", good.encode(), good.encode())).expect("bad store");
    let refused_3: StoreRefusal = SlotIdentityStore::read(numbered.as_path()).expect_err("bad kind on line 3");
    assert!(refused_3.message().contains("line 3"), "the line is named: {}", refused_3.message());
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
    let refused_4: StoreRefusal = SlotIdentityStore::read(path.as_path()).expect_err("one slot twice");
    assert!(matches!(refused_4, StoreRefusal::DuplicateKey { number: 4, ref key, .. } if key.as_str() == "one"), "{refused_4:?}");
    assert!(refused_4.message().contains('4'), "the second line is named: {}", refused_4.message());
    let twice_parent: PathBuf = store(root.as_path(), &[parent_line(&good), parent_line(&identity(322))]);
    let refused_5: StoreRefusal = SlotIdentityStore::read(twice_parent.as_path()).expect_err("one parent twice");
    assert!(matches!(refused_5, StoreRefusal::DuplicateKey { ref key, .. } if key.as_str() == "-"), "{refused_5:?}");
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
    let refused_6: StoreRefusal = append_slot_identity(path.as_path(), "one", &identity(332)).expect_err("recorded already");
    assert!(matches!(refused_6, StoreRefusal::SlotAlreadyRecorded { ref slot, .. } if slot.as_str() == "one"), "{refused_6:?}");
    assert_eq!(std::fs::read(path.as_path()).expect("bytes"), before, "the store is unchanged");
    assert!(refused_6.message().contains("never refreshed"), "{}", refused_6.message());
    for refused_name in ["", "one/two", ".", "..", ".hidden", "with space", "with\ttab"] {
        let refused_7: StoreRefusal = append_slot_identity(path.as_path(), refused_name, &identity(333)).expect_err(refused_name);
        assert!(matches!(refused_7, StoreRefusal::SlotName { .. }), "{refused_name}: {refused_7:?}");
        assert!(refused_7.message().contains("no slot name"), "{}", refused_7.message());
    }
    let no_parent: PathBuf = store(root.as_path(), &[]);
    let refused_8: StoreRefusal = append_slot_identity(no_parent.as_path(), "one", &recorded).expect_err("nothing to compare a slot against");
    assert!(matches!(refused_8, StoreRefusal::NoParentRecord { .. }), "{refused_8:?}");
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
    let path: PathBuf = store(root.as_path(), &[parent_line(&old_parent), String::from("# a comment is no record and need not survive a rewrite"), slot_line("one", &slot)]);
    let adopted: CloneFolderIdentity = identity(342);
    record_parent_identity(path.as_path(), &adopted).expect("the parent record is replaced");
    let read: SlotIdentityStore = SlotIdentityStore::read(path.as_path()).expect("store reads");
    assert_eq!(read.parent(), Some(&adopted));
    assert_eq!(read.slot("one"), Some(&slot), "a surviving slot identity is not overwritten");
    let text: String = std::fs::read_to_string(path.as_path()).expect("text");
    assert!(text.starts_with(HEADER), "the file keeps its header: {text}");
    assert_eq!(text.lines().filter(|line| { return line.starts_with("parent "); }).count(), 1, "one parent record: {text}");
    assert_eq!(text.lines().filter(|line| { return line.starts_with("slot "); }).count(), 1, "the slot record is kept: {text}");
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
    let refused_9: StoreRefusal = SlotIdentityStore::read(path.as_path()).expect_err("a link is not read through");
    assert!(matches!(refused_9, StoreRefusal::Unreadable { .. }), "{refused_9:?}");
    assert!(refused_9.message().contains("non-regular"), "{}", refused_9.message());
    let refused_10: StoreRefusal = append_slot_identity(path.as_path(), "one", &identity(351)).expect_err("a link is not written through");
    assert!(matches!(refused_10, StoreRefusal::Write { .. }), "a refused append is not a read problem: {refused_10:?}");
    assert!(refused_10.message().contains("could not be written"), "{}", refused_10.message());
    assert!(refused_10.message().contains("symbolic link"), "{}", refused_10.message());
    remove(root.as_path());
}

/// Whether a second exclusive lock on `path` can be taken now,
/// which says whether the append under test holds it.
fn can_lock_exclusively(path: &Path) -> bool {
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).write(true).open(path).expect("the file opens");
    // SAFETY: `flock` reads the descriptor and the flag only, and touches no memory of ours.
    let taken: libc::c_int = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    return taken == 0;
}

/// What:
///  The read,
///  the checks and the append are one step under one exclusive lock of the lock file beside the store:
///  a second exclusive lock cannot be taken on that file while they run,
///  the store itself is not locked,
///  and the same probe takes the lock file once the append returns.
/// Why:
///  Two sweeps that meet on one slot both read "no record" and both appended before this lock existed,
///  and the duplicate then refused every later read until an administrator edited a root-owned file.
///  The lock stands on a file of its own because an exclusive `flock` needs no write bit of the file it locks:
///  on the store,
///  every account that may read the store could hold it and stall root's own sweep.
#[test]
fn the_append_holds_one_lock_on_the_lock_file_from_the_read_to_the_write() {
    let root: PathBuf = fixture("slot-store-lock");
    let path: PathBuf = store(root.as_path(), &[parent_line(&identity(360))]);
    let lock: PathBuf = store_lock_path(path.as_path());
    assert!(!lock.exists(), "positive control: no lock file stands before the append");
    let seen: Arc<Mutex<Vec<(bool, bool)>>> = Arc::default();
    let during: Arc<Mutex<Vec<(bool, bool)>>> = Arc::clone(&seen);
    let probed_store: PathBuf = path.clone();
    let probed_lock: PathBuf = lock.clone();
    BETWEEN_THE_READ_AND_THE_APPEND.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            let store_free: bool = can_lock_exclusively(&probed_store);
            let lock_free: bool = can_lock_exclusively(&probed_lock);
            during.lock().expect("the control's own list").push((store_free, lock_free));
        }));
    });
    let recorded: Result<(), StoreRefusal> = append_slot_identity(path.as_path(), "one", &identity(361));
    BETWEEN_THE_READ_AND_THE_APPEND.with(|hook| { *hook.borrow_mut() = None });
    recorded.expect("a slot with no record is recorded");
    assert_eq!(*seen.lock().expect("the control's own list"), vec![(true, false)], "the append holds the lock file and never the store");
    assert!(can_lock_exclusively(lock.as_path()), "the same probe takes the lock file once the append returns");
    let made: std::fs::Permissions = std::fs::metadata(lock.as_path()).expect("the lock's metadata").permissions();
    assert_eq!(made.mode() & 0o777, STORE_LOCK_MODE, "the append makes the lock root's own, so no other account may open it");
    let read: SlotIdentityStore = SlotIdentityStore::read(path.as_path()).expect("one record, so the store reads");
    assert_eq!(read.slot("one"), Some(&identity(361)));
    remove(root.as_path());
}

/// What:
///  An append that cannot take the lock inside the wait it allows refuses,
///  names the lock,
///  and writes nothing.
/// Why:
///  A holder that never finishes must refuse rather than hold the command open,
///  and a refusal that does not name the lock sends a person to the wrong file.
#[test]
fn an_append_that_cannot_take_the_lock_refuses_and_writes_nothing() {
    let root: PathBuf = fixture("slot-store-locked");
    let path: PathBuf = store(root.as_path(), &[parent_line(&identity(362))]);
    let lock: PathBuf = store_lock_path(path.as_path());
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    let holder: std::fs::File = options.read(true).write(true).create(true).mode(STORE_LOCK_MODE).open(lock.as_path()).expect("the lock file");
    // SAFETY: `flock` reads the descriptor and the flag only, and touches no memory of ours.
    assert_eq!(unsafe { libc::flock(holder.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) }, 0, "the control holds the lock");
    let before: Vec<u8> = std::fs::read(path.as_path()).expect("bytes");
    THE_LOCK_WAIT_ALLOWED.with(|allowed| { *allowed.borrow_mut() = Some(std::time::Duration::from_millis(20)); });
    let refused: StoreRefusal = append_slot_identity(path.as_path(), "one", &identity(363)).expect_err("another run holds the lock");
    THE_LOCK_WAIT_ALLOWED.with(|allowed| { *allowed.borrow_mut() = None });
    assert!(matches!(refused, StoreRefusal::Locked { .. }), "{refused:?}");
    assert!(refused.message().contains(lock.to_string_lossy().as_ref()), "the lock is named: {}", refused.message());
    assert!(refused.message().contains("held by another run"), "the holder is named: {}", refused.message());
    assert_eq!(std::fs::read(path.as_path()).expect("bytes"), before, "nothing was written");
    assert!(SlotIdentityStore::read(path.as_path()).expect("the unmoved store reads").slot("one").is_none());
    remove(root.as_path());
}

/// What:
///  A link planted at the lock's name is refused,
///  never followed.
/// Why:
///  The lock decides which append writes the store;
///  a link there would let the account point the lock at a file it can lock itself.
#[test]
fn a_link_at_the_lock_name_is_refused() {
    let root: PathBuf = fixture("slot-store-lock-link");
    let path: PathBuf = store(root.as_path(), &[parent_line(&identity(364))]);
    let elsewhere: PathBuf = root.join("elsewhere-lock");
    std::fs::write(&elsewhere, b"not a lock").expect("elsewhere");
    std::os::unix::fs::symlink(&elsewhere, store_lock_path(path.as_path())).expect("link");
    let refused: StoreRefusal = append_slot_identity(path.as_path(), "one", &identity(365)).expect_err("a link is not written through");
    assert!(matches!(refused, StoreRefusal::Write { .. }), "{refused:?}");
    assert!(refused.message().contains("symbolic link"), "{}", refused.message());
    remove(root.as_path());
}

/// What:
///  A store larger than the bytes one read accepts is refused,
///  naming the bound,
///  while a store inside it reads.
/// Why:
///  The read took whatever size the file had:
///  a replaced or runaway file was read into memory before its lines were even looked at.
#[test]
fn a_store_larger_than_the_read_accepts_is_refused() {
    let root: PathBuf = fixture("slot-store-large");
    let path: PathBuf = root.join("slot-identities");
    let filler: String = format!("{HEADER}\n#{}\n", "0".repeat(usize::try_from(STORE_MAXIMUM_BYTES).expect("the bound fits")));
    std::fs::write(&path, filler.as_bytes()).expect("a store larger than the bound");
    let refused: StoreRefusal = SlotIdentityStore::read(path.as_path()).expect_err("a runaway store is refused");
    assert!(matches!(refused, StoreRefusal::Unreadable { .. }), "{refused:?}");
    assert!(refused.message().contains(STORE_MAXIMUM_BYTES.to_string().as_str()), "the bound is named: {}", refused.message());
    let inside: PathBuf = store(root.as_path(), &[parent_line(&identity(370))]);
    assert!(SlotIdentityStore::read(inside.as_path()).is_ok(), "positive control: a store inside the bound reads");
    remove(root.as_path());
}

/// The truth table of the root-control check:
/// root with no group or other write bit is under root control,
/// an owner that is not root is not,
/// and so is a mode another account can write.
#[test]
fn the_control_says_which_owner_and_mode_are_root_s_own() {
    assert_eq!(store_control_refusal(STORE_OWNER, 0o644), None);
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
///  The route's reader refuses a store this account owns,
///  naming the owner,
///  while the plain reader reads the same file,
///  and a store whose folder another account can write is refused as well.
/// Why:
///  A record the attach's account could rewrite decides nothing,
///  and the refusal must say which of the two paths is not root's.
#[test]
fn the_route_refuses_a_store_this_account_could_rewrite() {
    let root: PathBuf = fixture("slot-store-control");
    let path: PathBuf = store(root.as_path(), &[parent_line(&identity(380))]);
    assert!(SlotIdentityStore::read(path.as_path()).is_ok(), "positive control: the grammar reads the file");
    let refused: StoreRefusal = SlotIdentityStore::read_under_root_control(path.as_path()).expect_err("this account owns the store");
    assert!(matches!(refused, StoreRefusal::NotUnderRootControl { .. }), "{refused:?}");
    assert!(refused.message().contains("not under root control"), "{}", refused.message());
    assert!(refused.message().contains(&format!("user {}", nix_uid_of(root.as_path()))), "the owner is named: {}", refused.message());
    assert!(refused.message().contains(path.to_string_lossy().as_ref()), "the path is named: {}", refused.message());
    remove(root.as_path());
}

/// The owner of a fixture directory,
/// which is the account this run's fixtures belong to.
fn nix_uid_of(path: &Path) -> u32 {
    use std::os::unix::fs::MetadataExt;
    return std::fs::metadata(path).expect("the fixture's metadata").uid();
}

/// Environment variable that makes the control below run its inner half,
/// inside a user namespace of its own,
/// where this run's own files are root's.
#[cfg(target_os = "linux")]
const ROOT_SIDE_VARIABLE: &str = "SLOT_IDENTITY_STORE_ROOT_SIDE";

/// What:
///  A store written by root reads through the route's reader,
///  and a store whose mode lets another account write it is still refused.
/// Why:
///  The positive arm of the root-control check needs a root-owned store,
///  which this account cannot make outside a namespace;
///  inside a user namespace the run is root and its own files are root's,
///  so both the accept and the refuse are exercised on a real file.
#[cfg(target_os = "linux")]
#[test]
fn a_store_written_by_root_is_read_under_root_control() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        ROOT_SIDE_VARIABLE,
        "worktree_clone_mount::slot_identity_store::tests::a_store_written_by_root_is_read_under_root_control",
    ) {
        return;
    }
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("slot-store-root-side");
    let path: PathBuf = store(root.as_path(), &[parent_line(&identity(390))]);
    let read: SlotIdentityStore = SlotIdentityStore::read_under_root_control(path.as_path()).expect("a store written by root reads");
    assert_eq!(read.parent(), Some(&identity(390)));
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o666)).expect("a store another account can write");
    let refused: StoreRefusal = SlotIdentityStore::read_under_root_control(path.as_path()).expect_err("a store another account can write is not root's");
    assert!(matches!(refused, StoreRefusal::NotUnderRootControl { .. }), "{refused:?}");
    assert!(refused.message().contains("0666"), "the mode is named: {}", refused.message());
    remove(root.as_path());
}

/// Environment variable that makes the control below run its inner half,
/// inside a user namespace of its own,
/// where this run's own files are root's.
#[cfg(target_os = "linux")]
const ROOT_SIDE_WRITES_VARIABLE: &str = "SLOT_IDENTITY_STORE_ROOT_SIDE_WRITES";

/// What:
///  The route's two writes refuse a store or a lock file this account could rewrite or open,
///  and record a slot and a parent in a store root wrote.
/// Why:
///  A record decides which folder a clone may be attached to,
///  so a store the account could rewrite decides nothing;
///  the check stands inside the step that holds the lock,
///  and the lock file is refused when another account may open it,
///  because an exclusive `flock` needs no write bit of the file it locks.
///  Inside a user namespace this run is root and its own files are root's,
///  so the accept and both refusals are exercised on real files.
#[cfg(target_os = "linux")]
#[test]
fn the_route_s_writes_refuse_a_store_or_lock_this_account_could_rewrite() {
    if !crate::worktree_clone_mount::root_namespace_test_support::in_a_root_namespace(
        ROOT_SIDE_WRITES_VARIABLE,
        "worktree_clone_mount::slot_identity_store::tests::the_route_s_writes_refuse_a_store_or_lock_this_account_could_rewrite",
    ) {
        return;
    }
    let root: PathBuf = fixture("slot-store-root-writes");
    let path: PathBuf = store(root.as_path(), &[parent_line(&identity(400))]);
    append_slot_identity_under_root_control(path.as_path(), "one", &identity(401)).expect("a store written by root records a slot");
    assert_eq!(SlotIdentityStore::read(path.as_path()).expect("store reads").slot("one"), Some(&identity(401)));
    let lock: PathBuf = store_lock_path(path.as_path());
    let made: std::fs::Permissions = std::fs::metadata(lock.as_path()).expect("the lock's metadata").permissions();
    assert_eq!(made.mode() & 0o777, STORE_LOCK_MODE, "the append makes the lock root's own");
    std::fs::set_permissions(lock.as_path(), std::fs::Permissions::from_mode(0o644)).expect("a lock another account may open");
    let refused_lock: StoreRefusal = append_slot_identity_under_root_control(path.as_path(), "two", &identity(402)).expect_err("an account that may open the lock could hold it");
    assert!(matches!(refused_lock, StoreRefusal::NotUnderRootControl { .. }), "{refused_lock:?}");
    assert!(refused_lock.message().contains("0644"), "the mode is named: {}", refused_lock.message());
    assert!(refused_lock.message().contains("the lock that keeps two appends apart"), "the lock is named: {}", refused_lock.message());
    std::fs::set_permissions(lock.as_path(), std::fs::Permissions::from_mode(STORE_LOCK_MODE)).expect("the lock back to root's own");
    append_slot_identity_under_root_control(path.as_path(), "two", &identity(402)).expect("a root-owned store records a second slot");
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o666)).expect("a store another account can write");
    let refused_store: StoreRefusal = append_slot_identity_under_root_control(path.as_path(), "three", &identity(403)).expect_err("a store this account could rewrite decides nothing");
    assert!(matches!(refused_store, StoreRefusal::NotUnderRootControl { .. }), "{refused_store:?}");
    assert!(refused_store.message().contains("0666"), "the mode is named: {}", refused_store.message());
    let refused_parent: StoreRefusal = record_parent_identity_under_root_control(path.as_path(), &identity(404)).expect_err("the adoption refuses it too");
    assert!(matches!(refused_parent, StoreRefusal::NotUnderRootControl { .. }), "{refused_parent:?}");
    let recorded_parent: CloneFolderIdentity = identity(405);
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o600)).expect("the store back to root's own");
    record_parent_identity_under_root_control(path.as_path(), &recorded_parent).expect("a root-owned store records the adopted parent");
    let read: SlotIdentityStore = SlotIdentityStore::read(path.as_path()).expect("store reads");
    assert_eq!(read.parent(), Some(&recorded_parent));
    assert_eq!(read.slot("two"), Some(&identity(402)), "the adoption keeps every slot record");
    remove(root.as_path());
}

/// What:
///  The store example in `SPEC.md` is a store this reader reads,
///  and its ACL field is the kernel's own blob rather than any bytes.
/// Why:
///  A person copies those lines into the file the setup run writes,
///  so an example the reader refuses teaches two wrong things at once:
///  that a comment may stand above the header,
///  and that any bytes are a kernel ACL.
#[test]
fn the_specification_s_store_example_is_a_store_this_reader_reads() {
    let spec: String = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/SPEC.md")).expect("the specification");
    let example: String = the_store_example(spec.as_str());
    let root: PathBuf = fixture("slot-store-example");
    let path: PathBuf = root.join("slot-identities");
    std::fs::write(&path, example.as_bytes()).expect("the specification's example");
    let read: SlotIdentityStore = SlotIdentityStore::read(path.as_path()).expect("the specification's example is a store this reader reads");
    let parent: &CloneFolderIdentity = read.parent().expect("the example records the parent folder");
    assert!(read.slot("example").is_some(), "the example records one slot");
    let access: &[u8] = parent.access_acl.as_deref().expect("the example shows an access ACL");
    assert_eq!(access.get(..4), Some([0x02, 0x00, 0x00, 0x00].as_slice()), "the kernel's version word comes first");
    assert_eq!(access.len() % 8, 4, "a version word and then eight bytes per ACL entry, and no other length is a blob");
    let tags: Vec<u16> = access[4..].as_chunks::<8>().0.iter().map(|entry| return u16::from_le_bytes([entry[0], entry[1]])).collect();
    assert!(tags.contains(&0x10), "a stored ACL with a named entry carries a mask: {tags:?}");
    remove(root.as_path());
}

/// The record lines of the specification's store example,
/// which is the first `text` block of the store section,
/// as the file would hold them.
fn the_store_example(spec: &str) -> String {
    let heading: usize = spec.find("#### The recorded identities store").expect("the store section");
    let opening: usize = heading + spec[heading..].find("```text\n").expect("the store example") + "```text\n".len();
    let closing: usize = opening + spec[opening..].find("```").expect("the example's closing fence");
    return String::from(&spec[opening..closing]);
}
