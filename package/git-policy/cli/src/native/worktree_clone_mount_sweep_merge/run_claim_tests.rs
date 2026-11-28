//! What:
//!  Controls of the run claim:
//!  one pass takes it,
//!  a second taker is answered at once with the holder's identity,
//!  a released claim is free again,
//!  and a claim file that is no claim is reported rather than refused.
//! Why:
//!  The claim is what lets one event-driven pass at a time,
//!  and it must never wait:
//!  a held trigger marks and answers,
//!  which is the device's whole shape
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What a taker does",
//!  and veto entry V4).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const first = takeTheRunClaim(folder); const second = takeTheRunClaim(folder);
//! expect(second.kind).toBe('held-by-another');
//! ```

use super::{
    ClaimAnswer,
    SweepsFolder,
    take_the_run_claim,
};
use crate::test_support::fixture;
use std::path::PathBuf;

/// This test's folder,
/// which the thread's fixtures remove however the control ends.
fn folder(name: &str) -> SweepsFolder {
    let root: PathBuf = fixture(name);
    return SweepsFolder::at(root.join("sweeps"), rustix::process::geteuid().as_raw());
}

/// A claim nobody holds is taken,
/// and its generation is the first one.
#[test]
fn a_free_claim_is_taken_at_the_first_generation() {
    let folder: SweepsFolder = folder("claim-free");
    let ClaimAnswer::Held(claim) = take_the_run_claim(&folder) else {
        panic!("a free claim is taken");
    };
    assert_eq!(claim.path(), folder.run_claim_file(1).as_path());
}

/// A second taker is answered at once with the holder's identity,
/// and takes nothing.
#[test]
fn a_held_claim_answers_its_holder_without_waiting() {
    let folder: SweepsFolder = folder("claim-held");
    let ClaimAnswer::Held(claim) = take_the_run_claim(&folder) else {
        panic!("a free claim is taken");
    };
    let ClaimAnswer::HeldByAnother(owner) = take_the_run_claim(&folder) else {
        panic!("a held claim answers its holder");
    };
    assert_eq!(owner.pid, i64::from(std::process::id()));
    assert_eq!(owner.path, folder.run_claim_file(1));
    drop(claim);
}

/// A released claim is free again,
/// and the next taker's generation is one past the last.
#[test]
fn a_released_claim_is_taken_at_the_next_generation() {
    let folder: SweepsFolder = folder("claim-released");
    let ClaimAnswer::Held(claim) = take_the_run_claim(&folder) else {
        panic!("a free claim is taken");
    };
    claim.release().expect("a claim this account may release");
    let ClaimAnswer::Held(again) = take_the_run_claim(&folder) else {
        panic!("a released claim is free");
    };
    assert_eq!(again.path(), folder.run_claim_file(2).as_path());
}

/// A claim record that is no claim this route wrote is reported,
/// which is the device's unreadable arm and never a refusal of the pass.
#[test]
fn a_claim_record_that_is_no_claim_is_reported() {
    let folder: SweepsFolder = folder("claim-not-a-claim");
    folder.prepare().expect("the folder");
    std::fs::write(
        folder.run_claim_file(1).as_path(),
        b"a line no claim decoder reads",
    )
    .expect("a planted file");
    let ClaimAnswer::Unreadable {
        reason,
    } = take_the_run_claim(&folder)
    else {
        panic!("a file no claim decoder reads is reported");
    };
    assert!(
        reason.contains("the run claim"),
        "the report names the claim: {reason}"
    );
}

/// A folder that could not be made is reported,
/// so the pass runs without the claim.
#[test]
fn a_folder_that_cannot_be_made_is_reported() {
    let root: PathBuf = fixture("claim-unwritable-folder");
    let blocker: PathBuf = root.join("sweeps");
    std::fs::write(blocker.as_path(), b"not a folder").expect("a file where the folder goes");
    let folder: SweepsFolder = SweepsFolder::at(blocker, rustix::process::geteuid().as_raw());
    let ClaimAnswer::Unreadable {
        reason,
    } = take_the_run_claim(&folder)
    else {
        panic!("a folder that cannot be made is reported");
    };
    assert!(
        reason.contains("cannot be used"),
        "the report says the folder cannot be used: {reason}"
    );
}
