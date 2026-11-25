//! What:
//!  The live Git settle's controls:
//!  a cut-short lock written again and released,
//!  the matching registration and its pointer removed,
//!  the empty underlay folder removed,
//!  and the two refusals that keep everything.
//! Why:
//!  The settle is the one removal-side step that removes a registration,
//!  a pointer and a folder the next add reuses,
//!  so it is proved on a disposable repository with real directories and no pool
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//!  step 5).
//!  Nothing here opens `/dev/zfs`:
//! the destruction proof is the seam's value.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = TheLiveSettle.settleTheRegistration(request);
//! expect(answer.removedTheRegistration).toBe(true);
//! ```

use crate::private_storage::{create_private_directory_all, write_private_file};
use crate::subvolume::fs_beneath::identity_of;
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::binding_encoding::{FolderIdentity, bytes_as_hexadecimal, path_as_hexadecimal};
use crate::worktree_clone_binding_record::binding_words::BINDING_HEADER;
use crate::worktree_clone_binding_record::registration_home::{binding_name, home_folder};
use crate::worktree_clone_managed_lock::clone_destruction_proof::{CloneDestructionProof, TheLiveCloneAnswer, prove_the_recorded_clone_is_gone};
use crate::worktree_clone_managed_lock::lock_installation::write_the_lost_lock;
use crate::worktree_clone_managed_lock::recorded_lock_facts::RecordedClone;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{RegistrationSettle, SettleAnswer, SettleRequest, TheLiveSettle};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::PathBuf;

/// The slot every control settles.
const SLOT: &str = "fix-flaky-test";

/// The clone's full dataset name.
const CLONE: &str = "tank/wt/fix-flaky-test";

/// The clone's GUID.
const CLONE_GUID: u64 = 7_828_944_690_444_299_989;

/// The add's nonce,
/// which every lock sentence is recomputed from.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// One disposable repository with a registration,
/// a slot whose pointer names it,
/// and a binding that records both identities.
struct SettledFixture {
    common: PathBuf,
    slot: PathBuf,
    registration: PathBuf,
    slot_identity: FolderIdentity,
    registration_identity: FolderIdentity,
}

impl SettledFixture {
    /// One healthy fixture,
    /// with the managed lock standing as the add wrote it.
    fn healthy(name: &str, with_lock: bool) -> SettledFixture {
        let root: PathBuf = fixture(&format!("removal-settle-{name}"));
        let common: PathBuf = root.join(".git");
        std::fs::create_dir_all(common.join("worktrees")).expect("the worktrees folder");
        let registration: PathBuf = common.join("worktrees").join(SLOT);
        std::fs::create_dir_all(registration.as_path()).expect("the registration");
        let slot: PathBuf = root.join(".claude").join("worktrees").join(SLOT);
        std::fs::create_dir_all(slot.as_path()).expect("the slot");
        std::fs::write(slot.join(".git"), format!("gitdir: {}\n", registration.display())).expect("the slot's pointer");
        let slot_identity: FolderIdentity = FolderIdentity::of_folder(slot.as_path()).expect("the underlay's identity");
        let registration_identity: FolderIdentity = FolderIdentity::of_folder(registration.as_path()).expect("the registration's identity");
        let fixture: SettledFixture = SettledFixture { common, slot, registration, slot_identity, registration_identity };
        fixture.write_the_binding();
        if with_lock {
            let nonce: RequestNonce = RequestNonce::parse(NONCE).expect("the control's nonce is canonical");
            let _ = write_the_lost_lock(fixture.registration.as_path(), SLOT, &nonce);
        }
        return fixture;
    }

    /// The binding a removal reads for its lock facts and its identities.
    fn write_the_binding(&self) {
        let home: PathBuf = home_folder(self.common.as_path());
        create_private_directory_all(home.as_path()).expect("the binding home");
        let lines: Vec<String> = vec![
            String::from(BINDING_HEADER),
            String::from("claim=1"),
            format!("nonce={NONCE}"),
            format!("request=branch {}", bytes_as_hexadecimal(b"main")),
            String::from("phase=filled"),
            String::from("checkoutDataset=tank/mono-checkout"),
            String::from("checkoutDatasetGuid=4815162342109876543"),
            format!("repository={}", path_as_hexadecimal(self.common.as_path())),
            String::from("repositoryIdentity=1:2:-:-"),
            format!("slot={}", path_as_hexadecimal(self.slot.as_path())),
            format!("slotIdentity={}", self.slot_identity.encode()),
            format!("registration={}", path_as_hexadecimal(self.registration.as_path())),
            format!("registrationIdentity={}", self.registration_identity.encode()),
            String::from("pool=tank"),
            String::from("poolGuid=1234567890123456789"),
            String::from("originSnapshot=tank/mono-checkout@git-worktree-clone-origin-445566778899aabbccddeeff00112233"),
            String::from("originSnapshotGuid=11050373805378911967"),
            String::from("originSnapshotCreatetxg=15"),
            String::from("originDataset=tank/mono-checkout"),
            String::from("originDatasetGuid=4815162342109876543"),
            format!("cloneDataset={CLONE}"),
            format!("cloneGuid={CLONE_GUID}"),
            String::from("cloneCreatetxg=17"),
            String::from("pointer=6769746469723a202f6e6f77686572650a"),
        ];
        let mut text: String = String::new();
        for line in lines {
            text.push_str(line.as_str());
            text.push('\n');
        }
        write_private_file(home.join(binding_name(SLOT)).as_path(), text.as_bytes()).expect("the binding record");
    }

    /// The registration directory's identity as the plan's `admin` field spells it.
    fn admin(&self) -> String {
        let folder: std::fs::File = std::fs::File::open(self.registration.as_path()).expect("the registration opens");
        return identity_of(&folder).expect("the registration's identity").encode();
    }

    /// The proof a settle takes.
    fn proof(&self) -> CloneDestructionProof {
        let clone: RecordedClone = RecordedClone { dataset: String::from(CLONE), guid: Some(DatasetGuid::parse(CLONE_GUID.to_string().as_str()).expect("a GUID")) };
        return prove_the_recorded_clone_is_gone(SLOT, &clone, &TheLiveCloneAnswer::Absent).expect("the absence is proved");
    }

}

/// What:
///  One settle's request,
///  with the registration's own name bytes and the proof a control made.
/// Why:
///  The request borrows the proof and the name,
///  so a control holds both while the settle runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = aRequest(fixture, proof, SlotBytes, admin);
/// ```
fn a_request<'a>(fixture: &'a SettledFixture, proof: &'a CloneDestructionProof, name: &'a [u8], admin: &'a str) -> SettleRequest<'a> {
    return SettleRequest {
        common_dir: fixture.common.as_path(),
        slot: SLOT,
        slot_path: fixture.slot.as_path(),
        registration_id: name,
        admin,
        slot_identity: Some(fixture.slot_identity),
        proof,
    };
}

#[test]
fn a_standing_lock_is_released_and_the_registration_the_pointer_and_the_underlay_go() {
    let fixture: SettledFixture = SettledFixture::healthy("healthy", true);
    let proof: CloneDestructionProof = fixture.proof();
    let name: Vec<u8> = SLOT.as_bytes().to_vec();
    let admin: String = fixture.admin();
    let request: SettleRequest<'_> = a_request(&fixture, &proof, name.as_slice(), admin.as_str());
    let answer: SettleAnswer = TheLiveSettle.settle_the_registration(&request).expect("the settle finishes");
    assert!(answer.released_the_lock, "the managed lock went");
    assert!(answer.removed_the_registration, "the registration and the pointer went");
    assert!(answer.removed_the_underlay, "the empty underlay went");
    assert!(!answer.wrote_the_lost_lock, "a standing lock is not written again");
    assert!(std::fs::symlink_metadata(fixture.registration.as_path()).is_err(), "no registration stands");
    assert!(std::fs::symlink_metadata(fixture.slot.as_path()).is_err(), "no underlay stands");
}

#[test]
fn a_lost_lock_is_written_again_then_released() {
    let fixture: SettledFixture = SettledFixture::healthy("lost-lock", false);
    let proof: CloneDestructionProof = fixture.proof();
    let name: Vec<u8> = SLOT.as_bytes().to_vec();
    let admin: String = fixture.admin();
    let request: SettleRequest<'_> = a_request(&fixture, &proof, name.as_slice(), admin.as_str());
    let answer: SettleAnswer = TheLiveSettle.settle_the_registration(&request).expect("the settle finishes");
    assert!(answer.wrote_the_lost_lock, "the settle writes a lost lock again first");
    assert_eq!(answer.lines.len(), 1, "and says so once");
    assert!(answer.lines[0].contains("had lost its managed Git lock"));
    assert!(answer.released_the_lock);
}

#[test]
fn a_registration_of_another_identity_refuses_and_removes_nothing() {
    let fixture: SettledFixture = SettledFixture::healthy("other-identity", true);
    let proof: CloneDestructionProof = fixture.proof();
    let name: Vec<u8> = SLOT.as_bytes().to_vec();
    let request: SettleRequest<'_> = SettleRequest {
        common_dir: fixture.common.as_path(),
        slot: SLOT,
        slot_path: fixture.slot.as_path(),
        registration_id: name.as_slice(),
        admin: "9:9:-:-",
        slot_identity: Some(fixture.slot_identity),
        proof: &proof,
    };
    let stop: RemovalStop = TheLiveSettle.settle_the_registration(&request).expect_err("another identity is another removal's registration");
    assert!(matches!(stop, RemovalStop::TheSettleRefused { .. }));
    assert!(std::fs::symlink_metadata(fixture.registration.as_path()).is_ok(), "the registration stands");
    assert!(std::fs::symlink_metadata(fixture.slot.join(".git").as_path()).is_ok(), "the pointer stands");
    assert!(std::fs::symlink_metadata(fixture.registration.join("locked").as_path()).is_ok(), "the lock stands");
}

#[test]
fn an_underlay_holding_an_entry_refuses_and_keeps_everything_it_removed_after_the_registration() {
    let fixture: SettledFixture = SettledFixture::healthy("filled-underlay", true);
    std::fs::write(fixture.slot.join("a-person's-file"), b"keep me").expect("a person's file");
    let proof: CloneDestructionProof = fixture.proof();
    let name: Vec<u8> = SLOT.as_bytes().to_vec();
    let admin: String = fixture.admin();
    let request: SettleRequest<'_> = a_request(&fixture, &proof, name.as_slice(), admin.as_str());
    let stop: RemovalStop = TheLiveSettle.settle_the_registration(&request).expect_err("a filled underlay is not a leftover");
    assert!(matches!(stop, RemovalStop::TheSettleRefused { .. }));
    assert!(stop.message().contains("never removes a folder a person filled"), "{}", stop.message());
    assert!(std::fs::symlink_metadata(fixture.slot.join("a-person's-file").as_path()).is_ok(), "the person's file stands");
}

#[test]
fn a_pointer_naming_another_registration_refuses() {
    let fixture: SettledFixture = SettledFixture::healthy("other-pointer", true);
    std::fs::write(fixture.slot.join(".git"), b"gitdir: /somewhere/else\n").expect("the pointer names another registration");
    let proof: CloneDestructionProof = fixture.proof();
    let name: Vec<u8> = SLOT.as_bytes().to_vec();
    let admin: String = fixture.admin();
    let request: SettleRequest<'_> = a_request(&fixture, &proof, name.as_slice(), admin.as_str());
    let stop: RemovalStop = TheLiveSettle.settle_the_registration(&request).expect_err("the pointer names another registration");
    assert!(matches!(stop, RemovalStop::TheSettleRefused { .. }));
    assert!(std::fs::symlink_metadata(fixture.registration.as_path()).is_ok(), "the registration stands");
}
