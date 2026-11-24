//! Controls for reading one binding's bytes:
//!  the header,
//!  an unknown key,
//!  an order or a repeat,
//!  a value the field cannot hold,
//!  a missing field and one ahead of its phase,
//!  the settle reason's pair,
//!  and the two size arms.

use crate::worktree_clone_binding_record::binding_encoding::{FolderIdentity, Request};
use crate::worktree_clone_binding_record::binding_parse::parse_binding;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::{Phase, RequestForm, SettleReason};
use crate::worktree_clone_binding_record::record_refusal::RecordRefusal;
use crate::worktree_clone_mount_protocol::{DatasetGuid, RequestNonce};

/// The record a fixture writes,
/// with every field its phase proves.
fn record() -> BindingRecord {
    let guid = |value: &str| return DatasetGuid::parse(value).expect("the fixture guid reads");
    let identity: FolderIdentity = FolderIdentity::decode("131074:1763750400:123456789").expect("the fixture identity reads");
    return BindingRecord {
        claim: 2,
        nonce: RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f").expect("the fixture nonce reads"),
        request: Request::with_argument(RequestForm::Branch, b"main"),
        phase: Phase::Mounted,
        settle_reason: None,
        checkout_dataset: String::from("tank/monochromatic-git-checkout"),
        checkout_dataset_guid: guid("4815162342109876543"),
        repository: b"/home/user/Monochromatic/.git".to_vec(),
        repository_identity: identity,
        slot: b"/home/user/Monochromatic/.claude/worktrees/add-cache-layer".to_vec(),
        slot_identity: Some(identity),
        registration: Some(b"/home/user/Monochromatic/.git/worktrees/add-cache-layer".to_vec()),
        registration_identity: Some(identity),
        pool: String::from("tank"),
        pool_guid: guid("17293822569102704641"),
        origin_snapshot: String::from("tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"),
        origin_snapshot_guid: Some(guid("16371409254608711923")),
        origin_snapshot_createtxg: Some(121_293),
        origin_dataset: Some(String::from("tank/monochromatic-git-checkout")),
        origin_dataset_guid: Some(guid("4815162342109876543")),
        clone_dataset: String::from("tank/monochromatic-git-worktree-clone-datasets/add-cache-layer"),
        clone_guid: Some(guid("4821193048571903102")),
        clone_createtxg: Some(121_294),
        pointer: Some(b"gitdir: /home/user/Monochromatic/.git/worktrees/add-cache-layer\n".to_vec()),
    };
}

/// The fixture's text,
/// with one line replaced or removed.
fn text_without(key: &str) -> String {
    let record: BindingRecord = record();
    let text: String = String::from_utf8(record.encode()).expect("the record is UTF-8");
    let kept: Vec<&str> = text.lines().filter(|line| return !line.starts_with(format!("{key}=").as_str())).collect();
    return format!("{}\n", kept.join("\n"));
}

/// The fixture's text with one line replaced by `line`.
fn text_with(key: &str, line: &str) -> String {
    let record: BindingRecord = record();
    let text: String = String::from_utf8(record.encode()).expect("the record is UTF-8");
    let replaced: Vec<String> = text
        .lines()
        .map(|found| return match found.starts_with(format!("{key}=").as_str()) {
            true => String::from(line),
            false => String::from(found),
        })
        .collect();
    return format!("{}\n", replaced.join("\n"));
}

/// A good record reads,
/// which is the control every refusal below is measured against.
#[test]
fn a_good_record_reads() {
    let record: BindingRecord = record();
    let read: BindingRecord = parse_binding(record.encode().as_slice()).expect("the fixture reads");
    assert_eq!(read, record);
}

/// Another first line refuses by the line it found.
#[test]
fn another_header_refuses() {
    let text: String = String::from_utf8(record().encode()).expect("UTF-8");
    let replaced: String = text.replacen("# monochromatic-git-policy-cli zfs worktree clone binding, version 1", "# something else", 1);
    let refusal: RecordRefusal = parse_binding(replaced.as_bytes()).expect_err("the header refused");
    assert!(matches!(refusal, RecordRefusal::Header { .. }), "{}", refusal.message());
}

/// A key this release does not write refuses,
/// even though the wrapper's own records skip what they do not ask for.
#[test]
fn an_unknown_key_refuses() {
    let text: String = text_without("pointer");
    let with_a_foreign_key: String = text.replace("cloneGuid=", "tariff=3\ncloneGuid=");
    let refusal: RecordRefusal = parse_binding(with_a_foreign_key.as_bytes()).expect_err("the unknown key refused");
    assert!(matches!(refusal, RecordRefusal::UnknownKey { ref key, .. } if key == "tariff"), "{}", refusal.message());
}

/// A field written twice,
/// which arrives out of order,
/// refuses.
#[test]
fn a_repeated_field_refuses() {
    let text: String = text_with("nonce", "claim=3");
    let refusal: RecordRefusal = parse_binding(text.as_bytes()).expect_err("the order refused");
    assert!(matches!(refusal, RecordRefusal::KeyOutOfOrder { .. }), "{}", refusal.message());
}

/// A value the field cannot hold refuses by its key.
#[test]
fn a_value_the_field_cannot_hold_refuses() {
    let text: String = text_with("phase", "phase=committed");
    let refusal: RecordRefusal = parse_binding(text.as_bytes()).expect_err("the phase refused");
    assert!(matches!(refusal, RecordRefusal::Value { ref key, .. } if key == "phase"), "{}", refusal.message());
    let bad_claim: String = text_with("claim", "claim=-1");
    let claim_refusal: RecordRefusal = parse_binding(bad_claim.as_bytes()).expect_err("the claim refused");
    assert!(matches!(claim_refusal, RecordRefusal::Value { ref key, .. } if key == "claim"), "{}", claim_refusal.message());
}

/// A field its phase requires and the record lacks refuses.
#[test]
fn a_missing_field_refuses() {
    let text: String = text_without("registrationIdentity");
    let refusal: RecordRefusal = parse_binding(text.as_bytes()).expect_err("the missing field refused");
    assert!(matches!(refusal, RecordRefusal::Missing { ref key, .. } if key == "registrationIdentity"), "{}", refusal.message());
}

/// A settle reason beside a phase other than `settling` refuses,
/// and a settling record without one refuses.
#[test]
fn the_settle_reason_is_written_only_at_settling() {
    let mut beside_a_phase: BindingRecord = record();
    beside_a_phase.settle_reason = Some(SettleReason::Enospc);
    let refusal: RecordRefusal = parse_binding(beside_a_phase.encode().as_slice()).expect_err("the pair refused");
    assert!(matches!(refusal, RecordRefusal::SettleReason { .. }), "{}", refusal.message());
    let mut settling: BindingRecord = record();
    settling.phase = Phase::Settling;
    let missing_reason: RecordRefusal = parse_binding(settling.encode().as_slice()).expect_err("the missing reason refused");
    assert!(matches!(missing_reason, RecordRefusal::SettleReason { .. }), "{}", missing_reason.message());
    settling.settle_reason = Some(SettleReason::Enospc);
    assert_eq!(parse_binding(settling.encode().as_slice()).expect("the settling record reads"), settling);
}

/// One line over the widest form refuses by size,
/// and so does a whole file over the bound.
#[test]
fn the_two_size_arms_refuse() {
    let long_slot: String = crate::worktree_clone_binding_record::binding_encoding::bytes_as_hexadecimal(&vec![b'a'; 4_150]);
    let text: String = text_with("slot", format!("slot={long_slot}").as_str());
    let refusal: RecordRefusal = parse_binding(text.as_bytes()).expect_err("the long line refused");
    assert!(matches!(refusal, RecordRefusal::LineTooLarge { .. }), "{}", refusal.message());
    let padding: String = String::from("#");
    let bytes: Vec<u8> = [record().encode(), vec![b'\n'; 50_000]].concat();
    let file_refusal: RecordRefusal = parse_binding(bytes.as_slice()).expect_err("the long file refused");
    let _ = padding;
    assert!(matches!(file_refusal, RecordRefusal::TooLarge { .. }), "{}", file_refusal.message());
}

/// A record filed under a name whose path is not UTF-8 refuses no path,
/// because the slot path is stored as bytes and decoded only when a caller asks.
#[test]
fn a_slot_path_that_is_not_utf8_still_reads_as_bytes() {
    let mut record: BindingRecord = record();
    record.slot = vec![0xff, 0xfe, b'/', b'x'];
    let read: BindingRecord = parse_binding(record.encode().as_slice()).expect("the record reads");
    assert_eq!(read.slot, record.slot);
    assert_eq!(read.slot_path(), None);
}
