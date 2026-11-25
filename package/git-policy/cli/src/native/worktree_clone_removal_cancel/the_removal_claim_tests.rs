//! What:
//!  Controls for the slot's claim as the command takes it:
//!  a take that holds,
//!  a second take that is refused while the first lives,
//!  and a dead owner's claim the next take replaces.
//! Why:
//!  A live command on the slot must refuse the cancel command with its process and claim file,
//!  and a dead owner's claim must be replaced by the next generation with nothing cleaned by hand
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The proof: which slot, which clone, which claim").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const first = claim.take('one'); expect(() => claim.take('one')).toThrow(/pid/);
//! ```

use super::{ClaimHeld, LiveTheRemovalClaim, TheRemovalClaim};
use crate::lock::single_owner_claim::live_owner_of;
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::{claim_prefix, home_folder};

/// The slot the controls claim.
const SLOT: &str = "fix-flaky-test";

#[test]
fn a_take_holds_and_its_drop_hands_the_next_generation_over() {
    let common: std::path::PathBuf = fixture("cancel-claim-take");
    let claim: LiveTheRemovalClaim<'_> = LiveTheRemovalClaim { common_dir: common.as_path() };
    let held: ClaimHeld = claim.take(SLOT).expect("a folder with no claim is taken");
    assert!(held.owner().contains("this command"), "{}", held.owner());
    let home: std::path::PathBuf = home_folder(common.as_path());
    let prefix: std::ffi::OsString = claim_prefix(SLOT);
    assert!(
        live_owner_of(home.as_path(), prefix.as_os_str()).expect("the claim folder reads").is_some(),
        "positive control: the claim this process holds counts as a live owner"
    );
    drop(held);
    assert!(
        live_owner_of(home.as_path(), prefix.as_os_str()).expect("the released folder reads").is_none(),
        "a dropped claim names no live owner"
    );
    let again: ClaimHeld = claim.take(SLOT).expect("a released claim is taken by the next generation");
    drop(again);
}

#[test]
fn a_second_take_while_the_first_lives_names_the_owner_and_its_file() {
    let common: std::path::PathBuf = fixture("cancel-claim-busy");
    let claim: LiveTheRemovalClaim<'_> = LiveTheRemovalClaim { common_dir: common.as_path() };
    let held: ClaimHeld = claim.take(SLOT).expect("the first take holds");
    match claim.take(SLOT).expect_err("a live owner refuses the command") {
        crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal::ALiveRemoval { owner } => {
            let pid: String = std::process::id().to_string();
            assert!(owner.contains(pid.as_str()), "the refusal names this process: {owner}");
            assert!(owner.contains("claim"), "the refusal names the claim's file: {owner}");
        }
        other => panic!("a live owner refuses as one, not {other:?}"),
    }
    drop(held);
}

#[test]
fn a_claims_owner_names_this_process_and_a_control_names_its_own() {
    let common: std::path::PathBuf = fixture("cancel-claim-owner");
    let claim: LiveTheRemovalClaim<'_> = LiveTheRemovalClaim { common_dir: common.as_path() };
    let held: ClaimHeld = claim.take(SLOT).expect("the take holds");
    assert!(held.owner().contains(&std::process::id().to_string()), "{}", held.owner());
    drop(held);
    assert_eq!(ClaimHeld::for_a_control("4242").owner(), "4242");
}
