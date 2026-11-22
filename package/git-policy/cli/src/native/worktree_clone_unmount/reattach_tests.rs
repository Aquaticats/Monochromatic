//! What:
//!  Controls for the checked move back to the slot.
//! Why:
//!  A destination covered by a mount,
//!  a slot folder whose identity moved,
//!  and a proof that fails on either half each stop the move or the claim,
//!  and the arm asserts the step and whether the move ran.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(rollBackToSlot(request).kind).toBe('reattached');
//! ```

use crate::reattach::{roll_back_to_slot, ReattachRequest, ReattachResult};
use crate::slot_walk_calls::SlotWalk;
use crate::staged_folder_identity::StagedFolderIdentity;
use crate::staging_calls::{StagingCall, StagingCallRefusal};
use crate::staging_calls_control::RecordingCalls;
use std::path::{Path, PathBuf};

/// The walk a control drives.
struct FakeWalk {
    slot: PathBuf,
    covered: bool,
    again: u64,
}

impl SlotWalk for FakeWalk {
    fn open_the_slot(&self, _checkout: &Path, _slot: &str) -> Result<std::fs::File, StagingCallRefusal> {
        return std::fs::File::open(self.slot.as_path()).map_err(|error| return StagingCallRefusal { call: StagingCall::OpenSlot, errno: error.raw_os_error().unwrap_or(0), detail: error.to_string() });
    }

    fn open_the_slot_uncrossed(&self, _checkout: &Path, _slot: &str) -> Result<std::fs::File, StagingCallRefusal> {
        if self.covered {
            return Err(StagingCallRefusal { call: StagingCall::OpenSlot, errno: libc::EXDEV, detail: String::from("a mount covers the slot") });
        }
        return self.open_the_slot(_checkout, _slot);
    }

    fn look_again(&self, _checkout: &Path, _slot: &str) -> Result<u64, StagingCallRefusal> {
        return Ok(self.again);
    }
}

/// The slot,
/// its identity and the row a control reads.
struct Fixture {
    slot: PathBuf,
    staging: std::fs::File,
    identity: StagedFolderIdentity,
    calls: RecordingCalls,
}

/// The fixture slot and the row the move back compares.
fn fixture(name: &str, parent: u64) -> Fixture {
    let root: PathBuf = crate::testing::test_support::fixture_root().join(name);
    let slot: PathBuf = root.join(".claude/worktrees/one");
    std::fs::create_dir_all(slot.as_path()).expect("the fixture slot is made");
    let folder: std::fs::File = std::fs::File::open(slot.as_path()).expect("the fixture slot opens");
    let identity: StagedFolderIdentity = StagedFolderIdentity::of(&folder).expect("the slot's identity is read");
    let calls = RecordingCalls { mount_id: 41, observed: slot.clone(), ..RecordingCalls::default() };
    calls.mount_ids.borrow_mut().extend([40, 41, 41]);
    *calls.facts.borrow_mut() = Some(crate::fs_mount_options::MountFacts {
        mount_id: 41,
        parent_id: parent,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: slot.clone(),
        filesystem_type: String::from("zfs"),
        source: String::from("pool/clones/one"),
        mount_options: Vec::new(),
        super_options: Vec::new(),
    });
    let staging: std::fs::File = std::fs::File::open(slot.as_path()).expect("the fixture staging folder opens");
    return Fixture { slot, staging, identity, calls };
}

/// One request over the fixture.
fn request<'a>(fixture: &'a Fixture, identity: &'a StagedFolderIdentity, walk: &'a FakeWalk, checkout: &'a Path) -> ReattachRequest<'a> {
    return ReattachRequest {
        checkout,
        slot: "one",
        clone_source: "pool/clones/one",
        clone_mount_id: 41,
        slot_identity: identity,
        slot_parent_mount_id: 40,
        clone_at_staging: &fixture.staging,
        calls: &fixture.calls,
        walk,
    };
}

/// A free slot receives the clone and both halves of the proof pass.
#[test]
fn a_free_slot_receives_the_clone() {
    let fixture = fixture("reattach-free", 40);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: false, again: 41 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let identity: StagedFolderIdentity = fixture.identity;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert_eq!(result.word(), "reattached");
    assert!(fixture.calls.ran(StagingCall::Move));
}

/// A slot covered by a mount refuses before the move.
#[test]
fn a_covered_slot_refuses_before_the_move() {
    let fixture = fixture("reattach-covered", 40);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: true, again: 41 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let identity: StagedFolderIdentity = fixture.identity;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert!(matches!(result, ReattachResult::Refused { ref step, .. } if step == "open-slot"), "{result:?}");
    assert!(!fixture.calls.ran(StagingCall::Move));
}

/// A slot whose identity moved refuses before the move.
#[test]
fn a_slot_whose_identity_moved_refuses() {
    let fixture = fixture("reattach-identity", 40);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: false, again: 41 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let mut identity: StagedFolderIdentity = fixture.identity;
    identity.inode += 1;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert!(matches!(result, ReattachResult::Refused { ref step, .. } if step == "slot-identity"), "{result:?}");
    assert!(!fixture.calls.ran(StagingCall::Move));
}

/// A proof whose fresh walk reaches another mount refuses the placement.
#[test]
fn a_proof_that_fails_the_walk_refuses() {
    let fixture = fixture("reattach-proof-walk", 40);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: false, again: 99 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let identity: StagedFolderIdentity = fixture.identity;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert!(matches!(result, ReattachResult::Refused { ref step, .. } if step == "post-move-check"), "{result:?}");
}

/// A row whose parent is another mount refuses the placement.
#[test]
fn a_proof_that_fails_the_parent_refuses() {
    let fixture = fixture("reattach-proof-parent", 39);
    let walk = FakeWalk { slot: fixture.slot.clone(), covered: false, again: 41 };
    let checkout: PathBuf = fixture.slot.ancestors().nth(3).expect("the checkout is above the slot").to_path_buf();
    let identity: StagedFolderIdentity = fixture.identity;
    let request = request(&fixture, &identity, &walk, checkout.as_path());
    let result: ReattachResult = roll_back_to_slot(&request);
    assert!(matches!(result, ReattachResult::Refused { ref step, .. } if step == "post-move-check"), "{result:?}");
}
