//! What:
//!  Plan step 13's lock arms in the throwaway guest,
//!  on a file-backed pool:
//!  Git's own `worktree add --lock --reason` writing the wrapper's sentence,
//!  the clone mounted over the slot and unmounted,
//!  the pointer deleted,
//!  the lock read as the wrapper's own through the two-fact proof,
//!  the three prune commands keeping the registration while it stands,
//!  a person's lock read as theirs,
//!  the put-back writing a lost lock again,
//!  the release refused without this slot's proof and made after the clone is destroyed,
//!  and the same three commands then removing the registration,
//!  which is the control that the lock is what kept it.
//! Why:
//!  Plan step 13's "VM measurements" is item 6's lock arms extended to installing the lock after pointer loss,
//!  and the design's `M1` says the arms run in the guest on the releases the route ships:
//!  a plain directory on a person's machine cannot show that an unmounted clone's registration survives a prune,
//!  and the mount covering the slot is what makes the pointer's loss the state the lock outlives
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "Measurements owed",
//!  `M1`,
//!  and "Recovery").
//!  The arm runs only where `CLI_GIT_ZFS_LOCK_GUEST` names a disposable guest
//!  (`package/git-policy/cli/e2e/managed-git-lock-guest.ts`) and prints `SKIP` everywhere else,
//!  so a host run says what it did not run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await git('worktree add --no-checkout --lock --reason <sentence> <slot> HEAD');
//! await zfs('clone <checkout>@lock-arm <container>/one'); await zfs('mount <clone>');
//! await zfs('unmount <clone>'); await rm(join(slot, '.git')); await classify();
//! ```

use super::zfs_sweep_guest::{command_ok, make_the_designs_layout, required, zfs, zfs_ok, GUEST_VARIABLE as SWEEP_GUEST_VARIABLE, POOL_VARIABLE, SBIN_VARIABLE, CHECKOUT_VARIABLE};
use git_policy_cli::worktree_clone_binding_record::{BindingRecord, FolderIdentity, Phase, Request, RequestForm, lock_file_bytes, lock_words, open_or_create_home, write_binding};
use git_policy_cli::worktree_clone_mount_protocol::{DatasetGuid, RequestNonce};
use git_policy_cli::worktree_clone_managed_lock::{
    InstallAnswer, LockOwnership, ManagedLockState, RecordedClone, ReleaseRefusal, TheLiveCloneAnswer, classify_the_managed_lock, prove_the_recorded_clone_is_gone, read_the_recorded_facts,
    release_the_managed_lock, write_the_lost_lock,
};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The variable the guest driver sets for these arms.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_LOCK_GUEST";

/// The pool the driver prepared,
/// read through the sweep's own variable so one guest serves both arms.
const POOL: &str = POOL_VARIABLE;

/// The OpenZFS userspace the driver prepared.
const SBIN: &str = SBIN_VARIABLE;

/// The checkout the arm walks.
const CHECKOUT: &str = CHECKOUT_VARIABLE;

/// The slot's name,
/// which Git derives the registration's name from.
const SLOT: &str = "one";

/// The arm's nonce.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The snapshot the arm's clone comes from.
const ORIGIN: &str = "lock-arm";

/// The one person's lock this arm plants.
const A_PERSONS_WORDS: &str = "held by a person";

/// Whether this run is inside the guest.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP worktree_clone_managed_lock::guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname").map(|text| return text.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    assert!(std::env::var(SWEEP_GUEST_VARIABLE).is_err() || std::env::var(SWEEP_GUEST_VARIABLE).as_deref() == Ok(name.as_str()), "the sweep's guest variable names no other guest");
    return true;
}

/// Run one plain Git command in the guest's checkout,
/// requiring it to succeed.
fn git_ok(checkout: &Path, arguments: &[&str]) {
    let output: std::process::Output = Command::new("git")
        .current_dir(checkout)
        .args(["-c", "user.name=lock-arm", "-c", "user.email=lock-arm@example.invalid", "-c", "commit.gpgsign=false"])
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("git {arguments:?} could not start: {error}"));
    assert!(output.status.success(), "git {arguments:?} was refused: {}", String::from_utf8_lossy(output.stderr.as_slice()));
}

/// One dataset's GUID,
/// which the record carries.
fn guid_of(sbin: &str, dataset: &str) -> DatasetGuid {
    let (_ok, said, _error) = zfs(sbin, "zfs", &["get", "-H", "-o", "value", "guid", dataset], None);
    return DatasetGuid::parse(said.trim()).expect("the dataset's GUID reads");
}

/// The bytes of one path,
/// as a record writes them.
fn path_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    return path.as_os_str().as_bytes().to_vec();
}

/// The record the add would have written for this slot at the registration phase.
fn write_the_record(checkout: &Path, slot: &Path, registration: &Path, pointer: &[u8], slot_identity: FolderIdentity, sbin: &str, pool: &str, checkout_dataset: &str, clone: &str) {
    let common: PathBuf = checkout.join(".git");
    let home: std::fs::File = open_or_create_home(common.as_path()).expect("the record's folder opens");
    let record: BindingRecord = BindingRecord {
        claim: 1,
        nonce: RequestNonce::parse(NONCE).expect("the arm's nonce reads"),
        request: Request::with_argument(RequestForm::Commit, b"HEAD"),
        phase: Phase::Registered,
        settle_reason: None,
        checkout_dataset: String::from(checkout_dataset),
        checkout_dataset_guid: guid_of(sbin, checkout_dataset),
        repository: path_bytes(common.as_path()),
        repository_identity: FolderIdentity::of_folder(common.as_path()).expect("the common directory's identity reads"),
        slot: path_bytes(slot),
        slot_identity: Some(slot_identity),
        registration: Some(path_bytes(registration)),
        registration_identity: Some(FolderIdentity::of_folder(registration).expect("the registration's identity reads")),
        pool: String::from(pool),
        pool_guid: guid_of(sbin, pool),
        origin_snapshot: format!("{checkout_dataset}@{ORIGIN}"),
        origin_snapshot_guid: None,
        origin_snapshot_createtxg: None,
        origin_dataset: None,
        origin_dataset_guid: None,
        clone_dataset: String::from(clone),
        clone_guid: Some(guid_of(sbin, clone)),
        clone_createtxg: None,
        pointer: Some(pointer.to_vec()),
    };
    write_binding(&home, SLOT, &record).expect("the record is written");
}

/// What:
///  The whole arm:
///  Git's own lock,
///  the mount over it,
///  the pointer's loss,
///  the three prune commands,
///  a person's lock,
///  the put-back,
///  the refused release,
///  and the release after the clone is destroyed.
/// Why:
///  Each marker below is one arm the driver requires,
///  and the unlock control at the end is the positive control the probe needs:
///  the same three commands must remove the registration once the lock is released.
#[test]
fn the_managed_lock_outlives_the_pointer_and_goes_after_the_clone() {
    if !guest() {
        return;
    }
    let pool: String = required(POOL);
    let sbin: String = required(SBIN);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT));
    let (checkout_dataset, container): (String, String) = make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
    git_ok(checkout.as_path(), &["init", "--initial-branch=main"]);
    std::fs::write(checkout.join("README"), b"the lock arm's first file\n").expect("the checkout's first file is written");
    git_ok(checkout.as_path(), &["add", "README"]);
    git_ok(checkout.as_path(), &["commit", "--message", "the arm's first commit"]);
    let slot: PathBuf = checkout.join(".claude").join("worktrees").join(SLOT);
    let sentence: String = lock_words(&RequestNonce::parse(NONCE).expect("the arm's nonce reads"));
    git_ok(checkout.as_path(), &["worktree", "add", "--no-checkout", "--lock", "--reason", sentence.as_str(), slot.display().to_string().as_str(), "HEAD"]);
    let registration: PathBuf = checkout.join(".git").join("worktrees").join(SLOT);
    // F2's control on this guest's Git: the file Git wrote is the sentence and one newline, byte for byte.
    let written: Vec<u8> = std::fs::read(registration.join("locked")).expect("Git's lock reads");
    assert_eq!(written, lock_file_bytes(&RequestNonce::parse(NONCE).expect("the arm's nonce reads")), "Git's `--lock --reason` wrote the wrapper's sentence");
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("the slot's pointer reads");
    // The record is written while the slot is the underlay, before the clone covers it (the binding's A4).
    let slot_identity: FolderIdentity = FolderIdentity::of_folder(slot.as_path()).expect("the underlay folder's identity reads");
    zfs_ok(sbin.as_str(), &["snapshot", format!("{checkout_dataset}@{ORIGIN}").as_str()]);
    let clone: String = format!("{container}/{SLOT}");
    zfs_ok(sbin.as_str(), &["clone", format!("{checkout_dataset}@{ORIGIN}").as_str(), clone.as_str()]);
    write_the_record(checkout.as_path(), slot.as_path(), registration.as_path(), pointer.as_slice(), slot_identity, sbin.as_str(), pool.as_str(), checkout_dataset.as_str(), clone.as_str());
    // The clone covers the slot, and its unmount puts the underlay back.
    zfs_ok(sbin.as_str(), &["set", format!("mountpoint={}", slot.display()).as_str(), clone.as_str()]);
    zfs_ok(sbin.as_str(), &["mount", clone.as_str()]);
    command_ok("mount", &["--make-private", slot.display().to_string().as_str()]);
    zfs_ok(sbin.as_str(), &["unmount", clone.as_str()]);
    // Item 6's deletion: the underlay's pointer goes, and the lock is the only protection left.
    std::fs::remove_file(slot.join(".git")).expect("the underlay pointer goes");
    let common: PathBuf = checkout.join(".git");
    let classification = classify_the_managed_lock(common.as_path(), SLOT, slot.as_path());
    assert!(matches!(classification.state, ManagedLockState::Managed { .. }), "the lock reads as the add's through the two-fact proof: {:?}", classification.state);
    assert!(classification.the_slot_shows_no_pointer(), "the slot shows no `.git`");
    println!("LOCK classified managed-nonce=1 pointer=absent");
    // Item 6's three commands, with the lock standing.
    git_ok(checkout.as_path(), &["worktree", "prune"]);
    git_ok(checkout.as_path(), &["gc", "--quiet"]);
    git_ok(checkout.as_path(), &["maintenance", "run", "--auto"]);
    assert!(registration.is_dir(), "the locked registration survives the three commands");
    println!("LOCK prune-kept-the-locked-registration=true");
    // A person's lock reads as theirs and is never released.
    git_ok(checkout.as_path(), &["worktree", "unlock", slot.display().to_string().as_str()]);
    git_ok(checkout.as_path(), &["worktree", "lock", "--reason", A_PERSONS_WORDS, slot.display().to_string().as_str()]);
    let operator = classify_the_managed_lock(common.as_path(), SLOT, slot.as_path());
    assert_eq!(operator.ownership(), LockOwnership::AnOperatorsLock);
    println!("LOCK operator class=an-operators-lock");
    // The put-back, after the person's lock goes and the slot's claim is the arm's to hold.
    git_ok(checkout.as_path(), &["worktree", "unlock", slot.display().to_string().as_str()]);
    let lost = classify_the_managed_lock(common.as_path(), SLOT, slot.as_path());
    assert_eq!(lost.ownership(), LockOwnership::NoLockStands, "the lost lock is the state the put-back answers");
    match write_the_lost_lock(registration.as_path(), SLOT, &RequestNonce::parse(NONCE).expect("the arm's nonce reads")) {
        InstallAnswer::Wrote { line } => assert!(line.contains("had lost its managed Git lock"), "the one line says what it did: {line}"),
        other => panic!("the put-back wrote nothing: {other:?}"),
    }
    let again = classify_the_managed_lock(common.as_path(), SLOT, slot.as_path());
    assert!(matches!(again.state, ManagedLockState::Managed { .. }), "the put-back's lock reads as the add's: {:?}", again.state);
    println!("LOCK put-back wrote=yes line=yes");
    let facts = read_the_recorded_facts(common.as_path(), SLOT).expect("the record reads");
    let held: std::fs::File = std::fs::File::open(registration.as_path()).expect("the registration is held open");
    // The release refuses a proof made for another clone, which is what a stale settle would hand it.
    let elsewhere: RecordedClone = RecordedClone { dataset: format!("{container}/another"), guid: facts.clone.guid };
    let another: git_policy_cli::worktree_clone_managed_lock::CloneDestructionProof = prove_the_recorded_clone_is_gone(SLOT, &elsewhere, &TheLiveCloneAnswer::Absent).expect("the other clone is gone");
    assert!(matches!(
        release_the_managed_lock(&held, registration.as_path(), SLOT, &facts, &another),
        Err(ReleaseRefusal::TheProofIsForAnotherClone { .. })
    ));
    println!("LOCK release refused-with-another-clones-proof=yes");
    // The clone is destroyed, and the release goes through with this slot's own proof.
    zfs_ok(sbin.as_str(), &["destroy", "-f", clone.as_str()]);
    let proof: git_policy_cli::worktree_clone_managed_lock::CloneDestructionProof = prove_the_recorded_clone_is_gone(SLOT, &facts.clone, &TheLiveCloneAnswer::Absent).expect("the clone is gone");
    release_the_managed_lock(&held, registration.as_path(), SLOT, &facts, &proof).expect("the release goes through");
    assert!(!registration.join("locked").exists(), "the lock is gone");
    assert!(registration.is_dir(), "the registration still stands, because the recipe removes it next");
    println!("LOCK released=yes registration-still-stands=true");
    // The positive control: the same three commands remove the registration once the lock is released.
    git_ok(checkout.as_path(), &["worktree", "prune"]);
    assert!(!registration.exists(), "the unlocked registration goes, which is what the locked arms kept");
    println!("LOCK unlock-control-lost-the-registration=true");
    println!("LOCK done");
}
