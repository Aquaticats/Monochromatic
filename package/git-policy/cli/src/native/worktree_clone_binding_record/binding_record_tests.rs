//! Controls for the binding's own shape:
//!  every phase's required keys,
//!  and a record that round-trips through its own bytes.

use crate::worktree_clone_binding_record::binding_encoding::{FolderIdentity, Request};
use crate::worktree_clone_binding_record::binding_words::{Phase, RequestForm, SettleReason};
use crate::worktree_clone_mount_protocol::{DatasetGuid, RequestNonce};

/// The record a phase's fixture holds,
/// with every field that phase proves.
pub fn record_at(phase: Phase) -> crate::worktree_clone_binding_record::binding_record::BindingRecord {
    let guid = |value: &str| return DatasetGuid::parse(value).expect("the fixture guid reads");
    let identity: FolderIdentity = FolderIdentity::decode("131074:1763750400:123456789").expect("the fixture identity reads");
    let with_slot: bool = phase != Phase::Claimed;
    let with_registration: bool = matches!(phase, Phase::Registered | Phase::Snapshotted | Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling);
    let with_snapshot: bool = matches!(phase, Phase::Snapshotted | Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling);
    let with_clone: bool = matches!(phase, Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling);
    return crate::worktree_clone_binding_record::binding_record::BindingRecord {
        claim: 2,
        nonce: RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f").expect("the fixture nonce reads"),
        request: Request::with_argument(RequestForm::Branch, b"main"),
        phase,
        settle_reason: match phase {
            Phase::Settling => Some(SettleReason::Enospc),
            _ => None,
        },
        checkout_dataset: String::from("tank/monochromatic-git-checkout"),
        checkout_dataset_guid: guid("4815162342109876543"),
        repository: b"/home/user/Monochromatic/.git".to_vec(),
        repository_identity: identity,
        slot: b"/home/user/Monochromatic/.claude/worktrees/add-cache-layer".to_vec(),
        slot_identity: with_slot.then_some(identity),
        registration: with_registration.then(|| return b"/home/user/Monochromatic/.git/worktrees/add-cache-layer".to_vec()),
        registration_identity: with_registration.then_some(identity),
        pool: String::from("tank"),
        pool_guid: guid("17293822569102704641"),
        origin_snapshot: String::from("tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"),
        origin_snapshot_guid: with_snapshot.then(|| return guid("16371409254608711923")),
        origin_snapshot_createtxg: with_snapshot.then_some(121_293),
        origin_dataset: with_snapshot.then(|| return String::from("tank/monochromatic-git-checkout")),
        origin_dataset_guid: with_snapshot.then(|| return guid("4815162342109876543")),
        clone_dataset: String::from("tank/monochromatic-git-worktree-clone-datasets/add-cache-layer"),
        clone_guid: with_clone.then(|| return guid("4821193048571903102")),
        clone_createtxg: with_clone.then_some(121_294),
        pointer: with_registration.then(|| return b"gitdir: /home/user/Monochromatic/.git/worktrees/add-cache-layer\n".to_vec()),
    };
}

/// The keys each phase requires are the ones its proofs wrote before it.
#[test]
fn each_phase_requires_the_keys_its_proofs_wrote() {
    let claimed: Vec<&str> = crate::worktree_clone_binding_record::binding_record::required_keys_for(Phase::Claimed);
    assert!(claimed.contains(&"slot"));
    assert!(!claimed.contains(&"slotIdentity"));
    assert!(!claimed.contains(&"registration"));
    let registered: Vec<&str> = crate::worktree_clone_binding_record::binding_record::required_keys_for(Phase::Registered);
    assert!(registered.contains(&"slotIdentity"));
    assert!(registered.contains(&"registration"));
    assert!(registered.contains(&"registrationIdentity"));
    assert!(registered.contains(&"pointer"));
    assert!(!registered.contains(&"cloneGuid"));
    let mounted: Vec<&str> = crate::worktree_clone_binding_record::binding_record::required_keys_for(Phase::Mounted);
    assert!(mounted.contains(&"originSnapshotGuid"));
    assert!(mounted.contains(&"cloneGuid"));
    assert!(mounted.contains(&"cloneCreatetxg"));
}

/// Every phase's record writes and reads back the same facts.
#[test]
fn every_phases_record_round_trips() {
    let phases: [Phase; 8] = [Phase::Claimed, Phase::Slot, Phase::Registered, Phase::Snapshotted, Phase::Cloned, Phase::Mounted, Phase::Filled, Phase::Settling];
    for phase in phases {
        let record = record_at(phase);
        let bytes: Vec<u8> = record.encode();
        let read = crate::worktree_clone_binding_record::binding_parse::parse_binding(bytes.as_slice()).unwrap_or_else(|refusal| panic!("phase `{}` refused: {}", phase.word(), refusal.message()));
        assert_eq!(read, record, "phase `{}`", phase.word());
    }
}

/// The record's bytes begin with the header and hold one key per line,
/// in the grammar's order.
#[test]
fn the_bytes_begin_with_the_header_and_follow_the_order() {
    let bytes: Vec<u8> = record_at(Phase::Mounted).encode();
    let text: &str = std::str::from_utf8(bytes.as_slice()).expect("the record is UTF-8");
    assert!(text.starts_with(crate::worktree_clone_binding_record::binding_words::BINDING_HEADER));
    let keys: Vec<&str> = text.lines().skip(1).map(|line| return line.split_once('=').expect("a field line").0).collect();
    let expected: Vec<&str> = crate::worktree_clone_binding_record::binding_words::BINDING_KEYS_IN_ORDER.iter().copied().filter(|key| return keys.contains(key)).collect();
    assert_eq!(keys, expected);
}

/// A field whose phase has not arrived refuses,
/// which is the one arm of "a phase ahead of the live facts" a record can show alone.
#[test]
fn a_field_written_before_its_phase_refuses() {
    let mut record = record_at(Phase::Slot);
    record.registration = Some(b"/home/user/Monochromatic/.git/worktrees/add-cache-layer".to_vec());
    let refusal = crate::worktree_clone_binding_record::binding_parse::parse_binding(record.encode().as_slice()).expect_err("the field precedes its phase");
    assert!(matches!(refusal, crate::worktree_clone_binding_record::record_refusal::RecordRefusal::AheadOfItsPhase { .. }), "{}", refusal.message());
}
