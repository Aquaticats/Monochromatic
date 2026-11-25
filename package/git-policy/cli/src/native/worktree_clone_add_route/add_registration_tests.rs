//! Controls for the registration scan:
//!  a complete locked registration is adopted,
//!  a lock-only entry is cleared through the recipe,
//!  a slot with a pointer and no entry refuses,
//!  nothing at all asks for Git's own add,
//!  a person's lock refuses,
//!  and another slot's entry is left where it is.

use super::{ScanInput, ScanOutcome, scan_and_settle};
use crate::test_support::{fixture, remove};
use crate::worktree_clone_binding_record::binding_encoding::{FolderIdentity, Request};
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::{Phase, RequestForm};
use crate::worktree_clone_binding_record::lock_words::lock_file_bytes;
use crate::worktree_clone_binding_record::record_publication::write_binding;
use crate::worktree_clone_binding_record::registration_home::{home_folder, open_or_create_home};
use crate::worktree_clone_mount_protocol::{DatasetGuid, RequestNonce};
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// A repository's common directory, its home, a slot, and the nonce one add
/// wrote its lock with.
struct World {
    /// The fixture root.
    root: PathBuf,
    /// The common Git directory.
    common: PathBuf,
    /// The wrapper's record home.
    home: PathBuf,
    /// The slot's path.
    slot_path: PathBuf,
    /// The add's nonce.
    nonce: RequestNonce,
}

impl World {
    /// A world whose slot holds nothing.
    fn new(name: &str) -> World {
        let root: PathBuf = fixture(name);
        let common: PathBuf = root.join(".git");
        std::fs::create_dir_all(common.join("worktrees").as_path()).expect("the registrations folder");
        open_or_create_home(common.as_path()).expect("the home");
        let slot_path: PathBuf = root.join(".claude").join("worktrees").join("topic");
        std::fs::create_dir_all(Path::new(slot_path.parent().expect("the slot's parent"))).expect("the worktrees folder");
        return World { home: home_folder(common.as_path()), common, root, slot_path, nonce: RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f").expect("the nonce") };
    }

    /// The scan over this world.
    fn scan(&self) -> Result<ScanOutcome, String> {
        return scan_and_settle(&ScanInput { common_dir: self.common.as_path(), home: self.home.as_path(), slot: "topic", slot_path: self.slot_path.as_path(), nonce: &self.nonce });
    }

    /// The registration directory Git would make.
    fn registration(&self) -> PathBuf {
        return self.common.join("worktrees").join("topic");
    }

    /// A complete registration with this add's lock, its back-link, the slot's
    /// pointer, a `commondir` and a `HEAD`.
    fn a_complete_registration(&self) {
        let registration: PathBuf = self.registration();
        std::fs::create_dir_all(registration.as_path()).expect("the registration");
        std::fs::write(registration.join("locked"), lock_file_bytes(&self.nonce)).expect("the lock");
        std::fs::write(registration.join("gitdir"), format!("{}\n", self.slot_path.join(".git").display()).as_bytes()).expect("the back-link");
        std::fs::write(registration.join("commondir"), b"../..\n").expect("the common directory");
        std::fs::write(registration.join("HEAD"), b"ref: refs/heads/topic\n").expect("the head");
        std::fs::create_dir_all(self.slot_path.as_path()).expect("the slot");
        std::fs::write(self.slot_path.join(".git"), format!("gitdir: {}\n", registration.display()).as_bytes()).expect("the slot's pointer");
    }

    /// The record the add wrote at its claim,
    /// with the facts the lock's classification reads,
    /// through the writer the add itself uses.
    fn write_the_add_s_record(&self) {
        let home: std::fs::File = open_or_create_home(self.common.as_path()).expect("the home opens");
        let guid = |value: &str| return DatasetGuid::parse(value).expect("the fixture guid reads");
        let record: BindingRecord = BindingRecord {
            claim: 1,
            nonce: self.nonce.clone(),
            request: Request::with_argument(RequestForm::Branch, b"topic"),
            phase: Phase::Slot,
            settle_reason: None,
            checkout_dataset: String::from("tank/monochromatic-git-checkout"),
            checkout_dataset_guid: guid("4815162342109876543"),
            repository: self.common.as_os_str().as_encoded_bytes().to_vec(),
            repository_identity: FolderIdentity::of_folder(self.common.as_path()).expect("the common directory's identity reads"),
            slot: self.slot_path.as_os_str().as_encoded_bytes().to_vec(),
            slot_identity: Some(FolderIdentity::of_folder(self.slot_path.as_path()).expect("the slot's identity reads")),
            registration: None,
            registration_identity: None,
            pool: String::from("tank"),
            pool_guid: guid("17293822569102704641"),
            origin_snapshot: String::from("tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"),
            origin_snapshot_guid: None,
            origin_snapshot_createtxg: None,
            origin_dataset: None,
            origin_dataset_guid: None,
            clone_dataset: String::from("tank/clones/topic"),
            clone_guid: None,
            clone_createtxg: None,
            pointer: None,
        };
        write_binding(&home, "topic", &record).expect("the record is written");
    }

    /// Remove the whole fixture.
    fn close(self) {
        remove(self.root.as_path());
    }
}

#[test]
fn a_complete_locked_registration_is_adopted_with_its_pointer() {
    let world: World = World::new("scan-adopt");
    world.a_complete_registration();
    world.write_the_add_s_record();
    let outcome: ScanOutcome = world.scan().expect("the scan reads the registration");
    let ScanOutcome::Adopt { registration, .. } = outcome else { panic!("the complete registration is adopted") };
    assert_eq!(registration.path, world.registration());
    assert_eq!(registration.pointer, std::fs::read(world.slot_path.join(".git")).expect("the pointer"));
    assert!(registration.identity.inode() > 0);
    world.close();
}

#[test]
fn a_lock_only_entry_is_cleared_and_git_runs_again() {
    let world: World = World::new("scan-clear");
    let registration: PathBuf = world.registration();
    std::fs::create_dir_all(registration.as_path()).expect("the registration");
    std::fs::write(registration.join("locked"), lock_file_bytes(&world.nonce)).expect("the lock");
    let outcome: ScanOutcome = world.scan().expect("the row is cleared");
    let ScanOutcome::RunGit { cleared } = outcome else { panic!("a cleared row asks for Git's add") };
    assert!(cleared.contains(&String::from("locked")), "{cleared:?}");
    assert!(cleared.contains(&String::from("worktrees/topic")), "{cleared:?}");
    assert!(!registration.exists(), "the registration is gone");
    world.close();
}

#[test]
fn a_slot_with_a_pointer_and_no_entry_refuses_and_names_the_slot() {
    let world: World = World::new("scan-refuse");
    std::fs::create_dir_all(world.slot_path.as_path()).expect("the slot");
    std::fs::write(world.slot_path.join(".git"), b"gitdir: /somewhere/else\n").expect("a pointer");
    let refusal: String = world.scan().expect_err("nothing proves whose pointer it is");
    assert!(refusal.contains("nothing proves this add wrote that pointer"), "{refusal}");
    assert!(refusal.contains("running it again resumes"), "{refusal}");
    assert!(world.slot_path.join(".git").is_file(), "nothing was removed");
    world.close();
}

#[test]
fn nothing_at_all_asks_for_git_s_own_add() {
    let world: World = World::new("scan-rerun");
    let outcome: ScanOutcome = world.scan().expect("an empty listing asks for Git");
    assert_eq!(outcome, ScanOutcome::RunGit { cleared: Vec::new() });
    world.close();
}

#[test]
fn a_person_s_lock_refuses_and_changes_nothing() {
    let world: World = World::new("scan-foreign-lock");
    world.a_complete_registration();
    world.write_the_add_s_record();
    std::fs::write(world.registration().join("locked"), b"locked by a person, not the wrapper\n").expect("the person's lock");
    let refusal: String = world.scan().expect_err("a person's lock is refused");
    assert!(refusal.contains("is locked by words its add did not write"), "{refusal}");
    assert!(refusal.contains("Nothing was removed or rewritten"), "{refusal}");
    assert_eq!(std::fs::read(world.registration().join("locked")).expect("the lock"), b"locked by a person, not the wrapper\n");
    world.close();
}

/// What:
///  The put-back the decision of 2026-11-22 asks of the add's own resume:
///  a complete registration whose lock is gone is adopted,
///  the lock is written again,
///  and the command prints one line saying it did.
/// Why:
///  This is the command that already holds the slot's claim and has proved the
///  registration,
///  so the broken arm is the same fixture with the write removed.
#[test]
fn a_lost_lock_is_written_again_and_the_line_printed() {
    let world: World = World::new("scan-put-back");
    world.a_complete_registration();
    world.write_the_add_s_record();
    std::fs::remove_file(world.registration().join("locked")).expect("the lock is lost");
    let outcome: ScanOutcome = world.scan().expect("a lost lock is written again");
    let ScanOutcome::Adopt { wrote_the_lock, .. } = outcome else { panic!("the registration is adopted") };
    let line: String = wrote_the_lock.expect("the command says it wrote the lock again");
    assert!(line.contains("had lost its managed Git lock"), "{line}");
    assert!(line.contains("wrote it again for add nonce"), "{line}");
    assert_eq!(std::fs::read(world.registration().join("locked")).expect("the lock reads"), lock_file_bytes(&world.nonce), "the lock holds the sentence and Git's newline");
    world.close();
}

/// A lock a killed write cut short is completed in place,
/// because the bytes are the wrapper's own past the whole nonce.
#[test]
fn a_cut_short_lock_is_completed_in_place() {
    let world: World = World::new("scan-cut-short");
    world.a_complete_registration();
    world.write_the_add_s_record();
    let whole: Vec<u8> = lock_file_bytes(&world.nonce);
    let cut: usize = crate::worktree_clone_binding_record::lock_words::LOCK_WORDS_PREFIX.len() + 40;
    std::fs::write(world.registration().join("locked"), &whole[..cut]).expect("the cut-short lock");
    let outcome: ScanOutcome = world.scan().expect("a cut-short lock is completed");
    let ScanOutcome::Adopt { wrote_the_lock, registration } = outcome else { panic!("the registration is adopted") };
    assert!(wrote_the_lock.is_none(), "completing a lock is not writing a lost one, so no line is owed");
    assert!(registration.path.ends_with("topic"));
    assert_eq!(std::fs::read(world.registration().join("locked")).expect("the lock reads"), whole);
    world.close();
}

/// An empty lock is neither this add's sentence nor a person's words the reader can quote,
/// and the add refuses it with the classification's own sentence and writes nothing over it.
#[test]
fn an_empty_lock_refuses_and_is_not_overwritten() {
    let world: World = World::new("scan-empty-lock");
    world.a_complete_registration();
    world.write_the_add_s_record();
    std::fs::write(world.registration().join("locked"), b"").expect("the empty lock");
    let refusal: String = world.scan().expect_err("an empty lock is not the add's");
    assert!(refusal.contains("is locked by words its add did not write"), "{refusal}");
    assert_eq!(std::fs::read(world.registration().join("locked")).expect("the lock"), b"");
    world.close();
}

#[test]
fn another_slot_s_entry_is_left_where_it_is() {
    let world: World = World::new("scan-other-slot");
    // Another slot's binding owns `topic1`, so this slot's scan never clears it.
    std::fs::write(world.home.join("topic1.binding"), b"# another add's binding\n").expect("the other slot's binding");
    let other: PathBuf = world.common.join("worktrees").join("topic1");
    std::fs::create_dir_all(other.as_path()).expect("the other registration");
    std::fs::write(other.join("locked"), lock_file_bytes(&world.nonce)).expect("the other lock");
    let outcome: ScanOutcome = world.scan().expect("the other entry is left");
    assert_eq!(outcome, ScanOutcome::RunGit { cleared: Vec::new() }, "this slot's own entry is nothing");
    assert!(other.join("locked").is_file(), "the other slot's lock stands");
    let _path: &Path = world.root.as_path();
    let _bytes: &[u8] = world.root.as_os_str().as_bytes();
    world.close();
}
