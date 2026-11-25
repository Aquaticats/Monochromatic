//! What:
//!  Controls for the per-clone removal claim:
//!  its folder and its name,
//!  a take that holds,
//!  a second take that is refused while the first lives,
//!  and a release the next take takes over.
//! Why:
//!  A live removal must refuse the command with the owner's identity,
//!  and a dead owner's claim must be replaced by the next generation with nothing cleaned by hand
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The removal claim").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const first = claim.take(guid); expect(() => claim.take(guid)).toThrow(/pid/); first.release();
//! ```

use super::{ClaimHeld, LiveTheRemovalClaim, REMOVAL_CLAIM_FOLDER, TheRemovalClaim, removal_claim_folder, removal_claim_prefix};
use crate::lock::single_owner_claim::live_owner_of;
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;

/// The clone's GUID the controls claim.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("7828944690444299989").expect("a canonical GUID");
}

#[test]
fn the_claim_lives_in_the_routes_own_folder_under_its_own_name() {
    let common: std::path::PathBuf = fixture("removal-claim-names");
    assert_eq!(removal_claim_folder(common.as_path()), common.join(REMOVAL_CLAIM_FOLDER));
    assert_eq!(removal_claim_prefix(guid()).to_string_lossy(), "7828944690444299989.removal");
}

#[test]
fn a_take_holds_and_its_release_hands_the_next_generation_over() {
    let common: std::path::PathBuf = fixture("removal-claim-take");
    let claim: LiveTheRemovalClaim<'_> = LiveTheRemovalClaim { common_dir: common.as_path() };
    let held: ClaimHeld = claim.take(guid()).expect("a folder with no claim is taken");
    assert!(held.owner().contains("this command"), "{}", held.owner());
    let folder: std::path::PathBuf = removal_claim_folder(common.as_path());
    let prefix: std::ffi::OsString = removal_claim_prefix(guid());
    assert!(
        live_owner_of(folder.as_path(), prefix.as_os_str()).expect("the claim folder reads").is_some(),
        "positive control: the claim this process holds counts as a live owner"
    );
    held.release().expect("the claim is released");
    assert!(live_owner_of(folder.as_path(), prefix.as_os_str()).expect("the released folder reads").is_none(), "a released claim names no live owner");
    let again: ClaimHeld = claim.take(guid()).expect("a released claim is taken by the next generation");
    again.release().expect("the second claim is released");
}

#[test]
fn a_second_take_while_the_first_lives_names_the_owner() {
    let common: std::path::PathBuf = fixture("removal-claim-busy");
    let claim: LiveTheRemovalClaim<'_> = LiveTheRemovalClaim { common_dir: common.as_path() };
    let held: ClaimHeld = claim.take(guid()).expect("the first take holds");
    match claim.take(guid()).expect_err("a live owner refuses the command") {
        super::CancelRefusal::ALiveRemoval { owner } => {
            let pid: String = std::process::id().to_string();
            assert!(owner.contains(pid.as_str()), "the refusal names this process: {owner}");
            assert!(owner.contains("removal"), "the refusal names the claim's file: {owner}");
        }
        other => panic!("a live owner refuses as one, not {other:?}"),
    }
    held.release().expect("the first claim is released");
}

#[test]
fn a_controls_claim_creates_no_file_and_releases_nothing() {
    let held: ClaimHeld = ClaimHeld::for_a_control("4242");
    assert_eq!(held.owner(), "4242");
    held.release().expect("a control's claim has nothing to release");
}
