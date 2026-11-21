//! What:
//!  Controls of the per-slot root claim:
//!  two handlers cannot hold one slot at once,
//!  a dropped claim frees the slot,
//!  two slots are independent,
//!  and a name that is no slot is refused.
//! Why:
//!  The design lets parallel handlers sweep at once but forbids conflicting
//!  actions on the same mount
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The lock is the kernel's own `flock`,
//!  so the control holds a real second open file rather than a copy of the
//!  rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(claims.claim('one').kind).toBe('held');
//! ```

use super::{
    ClaimRefusal, HeldSlotClaim, SLOT_CLAIM_DIRECTORY, SLOT_CLAIM_DIRECTORY_MODE, SLOT_CLAIM_FILE_MODE, SLOT_CLAIM_SUFFIX, SlotClaimDirectory,
};
use crate::testing::test_support::fixture;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

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
    assert_eq!(SLOT_CLAIM_DIRECTORY, "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-claims");
    assert_eq!(SLOT_CLAIM_DIRECTORY_MODE, 0o755);
    assert_eq!(SLOT_CLAIM_FILE_MODE, 0o600);
    assert_eq!(SLOT_CLAIM_SUFFIX, ".claim");
    assert_eq!(directory.claim_file("one").file_name().and_then(|name| return name.to_str()), Some("one.claim"));
}

/// The directory takes its designed mode,
/// and one claim's file takes its own.
#[test]
fn the_claim_directory_and_file_take_their_modes() {
    let directory: SlotClaimDirectory = claims("claim-modes");
    let mode: u32 = std::fs::metadata(directory.directory()).expect("the directory exists").permissions().mode() & 0o777;
    assert_eq!(mode, SLOT_CLAIM_DIRECTORY_MODE);
    let held: HeldSlotClaim = directory.claim("one").expect("the slot is free");
    assert_eq!(held.slot(), "one");
    let file: PathBuf = directory.claim_file("one");
    let file_mode: u32 = std::fs::metadata(file.as_path()).expect("the claim file exists").permissions().mode() & 0o777;
    assert_eq!(file_mode, SLOT_CLAIM_FILE_MODE);
    held.release();
}

/// One slot is held by one handler:
/// a second claim is refused rather than queued,
/// and dropping the first frees it.
#[test]
fn one_slot_is_held_by_one_handler() {
    let directory: SlotClaimDirectory = claims("claim-exclusive");
    let first: HeldSlotClaim = directory.claim("one").expect("the slot is free");
    let refusal: ClaimRefusal = directory.claim("one").expect_err("a second holder is refused");
    assert_eq!(refusal, ClaimRefusal::Held { slot: String::from("one") });
    assert!(refusal.message().contains("Nothing was opened"), "{}", refusal.message());
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
    assert!(matches!(directory.claim("two"), Err(ClaimRefusal::Held { .. })));
    again.release();
    two.release();
}

/// A name that is no slot name is refused before any file is opened,
/// and a wrong directory owner or mode is refused too.
#[test]
fn a_foreign_name_or_directory_is_refused() {
    let directory: SlotClaimDirectory = claims("claim-names");
    for name in ["", ".", "..", "a/b", ".hidden", "two words", "tab\there"] {
        let refusal: ClaimRefusal = directory.claim(name).expect_err("not a slot name");
        assert!(matches!(refusal, ClaimRefusal::SlotName { .. }), "{name}: {refusal:?}");
    }
    let wrong_owner: SlotClaimDirectory = SlotClaimDirectory::at(directory.directory().to_path_buf(), this_uid() + 1);
    assert!(matches!(wrong_owner.claim("one"), Err(ClaimRefusal::DirectoryOwner { .. })));
    std::fs::set_permissions(directory.directory(), std::fs::Permissions::from_mode(0o700)).expect("the directory takes another mode");
    assert!(matches!(directory.claim("one"), Err(ClaimRefusal::DirectoryMode { found: 0o700, .. })));
}

/// An existing claim file with another mode is refused before it is locked,
/// and a link at the claim directory's path is refused without changing the
/// folder it names.
#[test]
fn another_claim_file_or_a_linked_directory_is_refused() {
    let directory: SlotClaimDirectory = claims("claim-file-mode");
    let held: HeldSlotClaim = directory.claim("one").expect("the slot is free");
    held.release();
    std::fs::set_permissions(directory.claim_file("one"), std::fs::Permissions::from_mode(0o666)).expect("the claim file takes another mode");
    assert!(matches!(directory.claim("one"), Err(ClaimRefusal::File { .. })));
    let target: PathBuf = fixture("claim-linked-target");
    std::fs::set_permissions(target.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the target takes a private mode");
    let link_parent: PathBuf = fixture("claim-linked-parent");
    let link: PathBuf = link_parent.join("claims");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("the link is made");
    let linked: SlotClaimDirectory = SlotClaimDirectory::at(link, this_uid());
    assert!(matches!(linked.prepare(), Err(ClaimRefusal::Directory { .. })));
    let mode: u32 = std::fs::metadata(target.as_path()).expect("the target exists").permissions().mode() & 0o777;
    assert_eq!(mode, 0o700, "a refused prepare changed nothing");
}
