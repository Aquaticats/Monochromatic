//! What:
//!  Controls of the per-slot root claim:
//!  two handlers cannot hold one slot at once,
//!  a dropped claim frees the slot,
//!  two slots are independent,
//!  a claim an owner left behind is taken over only when that owner is gone,
//!  the first claim after a boot makes its own directory,
//!  and a name that is no slot is refused.
//! Why:
//!  The design lets parallel handlers sweep at once but forbids conflicting
//!  actions on the same mount
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The claim is the repository's lock-free single-owner claim,
//!  so the controls hold the three cases that claim exists for:
//!  a live owner refuses the second claimant,
//!  a dead owner's record is taken over,
//!  and a killed claimant blocks nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(claims.claim('one').kind).toBe('held');
//! ```

use super::{
    ClaimRefusal,
    HeldSlotClaim,
    SLOT_CLAIM_DIRECTORY,
    SLOT_CLAIM_DIRECTORY_MODE,
    SLOT_CLAIM_FILE_MODE,
    SlotClaimDirectory,
};
use crate::process_identity::{
    current_birth_identity,
    process_birth_identity,
};
use crate::single_owner_claim::{
    plant_claim_for_tests,
    unused_pid_for_tests,
};
use crate::testing::test_support::fixture;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

/// This account's own number.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// A prepared fixture claim directory.
fn claims(name: &str) -> SlotClaimDirectory {
    let root: PathBuf = fixture(name);
    let directory: SlotClaimDirectory = SlotClaimDirectory::at(root.join("claims"), this_uid());
    directory.prepare().expect("the claim directory is made");
    return directory;
}

/// The live place is the designed path and requires root.
#[test]
fn the_live_place_is_the_designed_one() {
    let directory: SlotClaimDirectory = SlotClaimDirectory::live();
    assert_eq!(directory.directory().to_str(), Some(SLOT_CLAIM_DIRECTORY));
    assert_eq!(
        SLOT_CLAIM_DIRECTORY,
        "/run/git-worktree-clone-mount-slot-claims"
    );
    assert_eq!(SLOT_CLAIM_DIRECTORY_MODE, 0o755);
    assert_eq!(SLOT_CLAIM_FILE_MODE, 0o600);
    assert_eq!(
        directory
            .claim_generation_file("one", 1)
            .file_name()
            .and_then(|name| return name.to_str()),
        Some("one.1"),
        "the claim files are the shared claim's numbered generations"
    );
}

/// The directory takes its designed mode,
/// and one claim's generation file takes its own.
#[test]
fn the_claim_directory_and_file_take_their_modes() {
    let directory: SlotClaimDirectory = claims("claim-modes");
    let mode: u32 = std::fs::metadata(directory.directory())
        .expect("the directory exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, SLOT_CLAIM_DIRECTORY_MODE);
    let held: HeldSlotClaim = directory.claim("one").expect("the slot is free");
    assert_eq!(held.slot(), "one");
    let file: PathBuf = directory.claim_generation_file("one", 1);
    let file_mode: u32 = std::fs::metadata(file.as_path())
        .expect("the claim file exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(file_mode, SLOT_CLAIM_FILE_MODE);
    held.release();
}

/// What:
///  The first claim after a boot makes the claim directory itself,
///  at its designed mode,
///  instead of refusing every claim until something else made it.
/// Why:
///  The claims live under `/run`,
///  so every boot starts with no directory,
///  and a first sweep whose claim refused would mount nothing until an
///  earlier mode made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claims = SlotClaimDirectory.at(empty.join('never-made'), uid); expect(claims.claim('one').kind).toBe('held');
/// ```
#[test]
fn a_first_claim_in_a_missing_directory_is_taken() {
    let parent: PathBuf = fixture("claim-fresh-boot");
    let directory: SlotClaimDirectory =
        SlotClaimDirectory::at(parent.join("never-made"), this_uid());
    assert!(
        !directory.directory().exists(),
        "the boot starts with no claims directory"
    );
    let held: HeldSlotClaim = directory
        .claim("one")
        .expect("the first claim after a boot makes its own directory");
    let mode: u32 = std::fs::metadata(directory.directory())
        .expect("the claim made the directory")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, SLOT_CLAIM_DIRECTORY_MODE);
    let file: PathBuf = directory.claim_generation_file("one", 1);
    let file_mode: u32 = std::fs::metadata(file.as_path())
        .expect("the claim record exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(file_mode, SLOT_CLAIM_FILE_MODE);
    held.release();
}

/// One slot is held by one handler:
/// a second claim is refused rather than queued,
/// and releasing the first frees it.
#[test]
fn one_slot_is_held_by_one_handler() {
    let directory: SlotClaimDirectory = claims("claim-exclusive");
    let first: HeldSlotClaim = directory.claim("one").expect("the slot is free");
    let refusal: ClaimRefusal = directory
        .claim("one")
        .expect_err("a second holder is refused");
    assert_eq!(
        refusal,
        ClaimRefusal::Held {
            slot: String::from("one")
        }
    );
    assert!(
        refusal.message().contains("Nothing was opened"),
        "{}",
        refusal.message()
    );
    first.release();
    let second: HeldSlotClaim = directory.claim("one").expect("the slot is free again");
    second.release();
}

/// Two slots are independent,
/// and a dropped claim frees only its own.
#[test]
fn two_slots_are_independent() {
    let directory: SlotClaimDirectory = claims("claim-two-slots");
    let one: HeldSlotClaim = directory.claim("one").expect("slot one is free");
    let two: HeldSlotClaim = directory.claim("two").expect("slot two is free");
    assert_ne!(one.slot(), two.slot());
    one.release();
    let again: HeldSlotClaim = directory.claim("one").expect("slot one is free again");
    assert!(matches!(
        directory.claim("two"),
        Err(ClaimRefusal::Held { .. })
    ));
    again.release();
    two.release();
}

/// What:
///  A claim record naming a process that is gone is taken over,
///  and the next generation is published.
/// Why:
///   A killed claimant leaves its record and nothing else,
///  so the slot must free itself with no cleanup by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantClaim(deadPid); expect(claims.claim('one').kind).toBe('held');
/// ```
#[test]
fn a_dead_owners_claim_is_taken_over() {
    let directory: SlotClaimDirectory = claims("claim-dead-owner");
    let dead_pid: i64 = unused_pid_for_tests();
    plant_claim_for_tests(
        directory.directory(),
        std::ffi::OsStr::new("one"),
        1,
        dead_pid,
        "no process was born at this reading",
    );
    let held: HeldSlotClaim = directory
        .claim("one")
        .expect("a dead owner's record holds nothing");
    assert!(
        directory.claim_generation_file("one", 2).exists(),
        "the takeover publishes the next generation"
    );
    held.release();
}

/// What:
///  A claimant that really ran and was then killed blocks nothing.
/// Why:
///  The record names its process birth,
///  so a record of a process that existed and was killed is proved dead rather
///  than trusted or waited on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = spawn('sleep'); plantClaim(child.pid, birthOf(child)); kill(child); expect(claims.claim('one').kind).toBe('held');
/// ```
#[test]
fn a_killed_claimant_blocks_nothing() {
    let directory: SlotClaimDirectory = claims("claim-killed");
    let mut child: std::process::Child = std::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("the stand-in claimant starts");
    let pid: i64 = i64::from(child.id());
    let birth: String = process_birth_identity(pid)
        .expect("the stand-in claimant's birth is readable")
        .expect("the stand-in claimant is alive");
    plant_claim_for_tests(
        directory.directory(),
        std::ffi::OsStr::new("one"),
        1,
        pid,
        birth.as_str(),
    );
    assert_eq!(
        directory
            .claim("one")
            .expect_err("the live stand-in claimant holds the slot"),
        ClaimRefusal::Held {
            slot: String::from("one")
        }
    );
    child.kill().expect("the stand-in claimant is killed");
    let _reaped: std::process::ExitStatus = child.wait().expect("the stand-in claimant is reaped");
    // The kernel may need a moment before `/proc` stops naming the killed process.
    let deadline: std::time::Instant = std::time::Instant::now() + Duration::from_secs(5);
    while process_birth_identity(pid)
        .expect("the process table is readable")
        .is_some()
        && std::time::Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    let held: HeldSlotClaim = directory
        .claim("one")
        .expect("a killed claimant leaves nothing that blocks the next one");
    held.release();
}

/// What:
///  The record this process publishes names this process's birth,
///  which is what a live-owner refusal is decided on.
/// Why:
///  A process id alone is reused,
///  so the claim must name the birth identity too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(claimText).toContain(birthIdentity());
/// ```
#[test]
fn the_record_names_this_process_by_its_birth() {
    let directory: SlotClaimDirectory = claims("claim-birth");
    let held: HeldSlotClaim = directory.claim("one").expect("the slot is free");
    let text: String = std::fs::read_to_string(directory.claim_generation_file("one", 1))
        .expect("the record is readable");
    assert!(
        text.contains(&format!("pid={}", std::process::id())),
        "{text}"
    );
    assert!(
        text.contains(
            current_birth_identity()
                .expect("this process has a birth identity")
                .as_str()
        ),
        "{text}"
    );
    held.release();
}

/// What:
///  A name that is no slot name is refused before any file is opened,
///  a directory owned by another account is refused,
///  and a directory with another mode is given the designed one before the
///  claim is taken.
/// Why:
///  The claim makes and proves its directory exactly as `prepare` does,
///  so a fresh boot's absent directory,
///  a foreign owner and a wrong mode each answer the way the receipts and
///  readiness writers answer them:
///  the owner is the trust boundary and is never changed,
///  while the mode is the claim's own to set.
#[test]
fn a_foreign_name_or_directory_is_refused() {
    let directory: SlotClaimDirectory = claims("claim-names");
    for name in ["", ".", "..", "a/b", ".hidden", "two words", "tab\there"] {
        let refusal: ClaimRefusal = directory.claim(name).expect_err("not a slot name");
        assert!(
            matches!(refusal, ClaimRefusal::SlotName { .. }),
            "{name}: {refusal:?}"
        );
    }
    let wrong_owner: SlotClaimDirectory =
        SlotClaimDirectory::at(directory.directory().to_path_buf(), this_uid() + 1);
    assert!(matches!(
        wrong_owner.claim("one"),
        Err(ClaimRefusal::DirectoryOwner { .. })
    ));
    std::fs::set_permissions(
        directory.directory(),
        std::fs::Permissions::from_mode(0o700),
    )
    .expect("the directory takes another mode");
    let held: HeldSlotClaim = directory
        .claim("one")
        .expect("the claim gives the directory its designed mode and takes the slot");
    let mode: u32 = std::fs::metadata(directory.directory())
        .expect("the directory exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, SLOT_CLAIM_DIRECTORY_MODE,
        "the claim set the designed mode before it claimed"
    );
    held.release();
}

/// A claim record this route did not write is refused and named,
/// and a link at the claim directory's path is refused without changing the
/// folder it names.
#[test]
fn a_foreign_claim_record_or_a_linked_directory_is_refused() {
    let directory: SlotClaimDirectory = claims("claim-record");
    std::fs::write(
        directory.claim_generation_file("one", 1),
        "not a claim this route wrote\n",
    )
    .expect("the foreign record is written");
    let refusal: ClaimRefusal = directory
        .claim("one")
        .expect_err("a foreign record is refused rather than read as an owner");
    assert!(
        matches!(refusal, ClaimRefusal::Records { .. }),
        "{refusal:?}"
    );
    let target: PathBuf = fixture("claim-linked-target");
    std::fs::set_permissions(target.as_path(), std::fs::Permissions::from_mode(0o700))
        .expect("the target takes a private mode");
    let link_parent: PathBuf = fixture("claim-linked-parent");
    let link: PathBuf = link_parent.join("claims");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("the link is made");
    let linked: SlotClaimDirectory = SlotClaimDirectory::at(link, this_uid());
    assert!(matches!(
        linked.prepare(),
        Err(ClaimRefusal::Directory { .. })
    ));
    let mode: u32 = std::fs::metadata(target.as_path())
        .expect("the target exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700, "a refused prepare changed nothing");
}

/// What:
///  The refusal's way forward names the typed `doctor` command and says this
///  release prints no line for a claim and installs no handler unit.
/// Why:
///  The sentence said the `doctor` command reports a claim once plan step 10
///  adds it:
///  step 10 built the claim,
/// `doctor` reaches none of the route's checks,
/// and a
///  person who typed the bare words got Git's own refusal instead of a line.
#[test]
fn the_refusal_names_the_typed_doctor_and_no_claim_line() {
    let refusal: ClaimRefusal = ClaimRefusal::Held {
        slot: String::from("one"),
    };
    let message: String = refusal.message();
    assert!(
        message.contains("`git monochromatic-git-policy-cli doctor`"),
        "the doctor is named as a person types it: {message}"
    );
    assert!(
        message.contains("installs no handler unit"),
        "the sentence says this release installs the unit nowhere: {message}"
    );
    assert!(
        !message.contains("once plan step 10"),
        "and claims no claim line a later step would add: {message}"
    );
}
