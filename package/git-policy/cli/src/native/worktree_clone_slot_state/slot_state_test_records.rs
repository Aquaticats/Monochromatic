//! What:
//!  One binding record of any phase,
//!  shaped the way the add's own controls shape it,
//!  and the publication that puts it under a fixture's Git common directory.
//! Why:
//!  The slot-state controls and the move-refusal controls read the same record
//!  family,
//!  and a second copy of the fixture would let one of them drift from the
//!  grammar the add writes
//!  (`binding_record_tests.rs` is the add's own copy).
//!  Nothing here is part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = recordAt(Phase.Filled, '/repo/.claude/worktrees/topic');
//! publishInFixture(commonDir, record, 'topic');
//! ```

use crate::worktree_clone_binding_record::binding_encoding::{
    FolderIdentity,
    Request,
};
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::{
    Phase,
    RequestForm,
    SettleReason,
};
use crate::worktree_clone_binding_record::record_publication::write_binding;
use crate::worktree_clone_binding_record::registration_home::open_or_create_home;
use crate::worktree_clone_mount_protocol::{
    DatasetGuid,
    RequestNonce,
};
use std::path::Path;

/// The clone dataset a fixture record names.
pub(crate) const CLONE_DATASET: &str = "tank/monochromatic-git-worktree-clone-datasets/topic";

/// The pool a fixture record names.
pub(crate) const POOL: &str = "tank";

/// One dataset GUID the fixture uses.
pub(crate) fn guid(value: &str) -> DatasetGuid {
    return DatasetGuid::parse(value).expect("the fixture guid reads");
}

/// The record one phase holds at one slot,
/// with every field that phase proves.
pub(crate) fn record_at(
    phase: Phase,
    slot: &Path,
) -> BindingRecord {
    let identity: FolderIdentity =
        FolderIdentity::decode("131074:1763750400:123456789").expect("the fixture identity reads");
    let slot_name: String = slot.file_name().map_or_else(String::new, |name| {
        return name.to_string_lossy().into_owned();
    });
    let with_slot: bool = phase != Phase::Claimed;
    let with_registration: bool = matches!(
        phase,
        Phase::Registered
            | Phase::Snapshotted
            | Phase::Cloned
            | Phase::Mounted
            | Phase::Filled
            | Phase::Settling
    );
    let with_snapshot: bool = matches!(
        phase,
        Phase::Snapshotted | Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling
    );
    let with_clone: bool = matches!(
        phase,
        Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling
    );
    let registration: Vec<u8> = format!("/repo/.git/worktrees/{slot_name}").into_bytes();
    let pointer: Vec<u8> = format!("gitdir: /repo/.git/worktrees/{slot_name}\n").into_bytes();
    return BindingRecord {
        claim: 2,
        nonce: RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
            .expect("the fixture nonce reads"),
        request: Request::with_argument(RequestForm::Branch, b"main"),
        phase,
        settle_reason: match phase {
            Phase::Settling => Some(SettleReason::Enospc),
            _ => None,
        },
        checkout_dataset: String::from("tank/monochromatic-git-checkout"),
        checkout_dataset_guid: guid("4815162342109876543"),
        repository: b"/repo/.git".to_vec(),
        repository_identity: identity,
        slot: slot.as_os_str().as_encoded_bytes().to_vec(),
        slot_identity: with_slot.then_some(identity),
        registration: with_registration.then_some(registration),
        registration_identity: with_registration.then_some(identity),
        pool: String::from(POOL),
        pool_guid: guid("17293822569102704641"),
        origin_snapshot: String::from(
            "tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f",
        ),
        origin_snapshot_guid: with_snapshot.then(|| return guid("16371409254608711923")),
        origin_snapshot_createtxg: with_snapshot.then_some(121_293),
        origin_dataset: with_snapshot
            .then(|| return String::from("tank/monochromatic-git-checkout")),
        origin_dataset_guid: with_snapshot.then(|| return guid("4815162342109876543")),
        clone_dataset: String::from(CLONE_DATASET),
        clone_guid: with_clone.then(|| return guid("4821193048571903102")),
        clone_createtxg: with_clone.then_some(121_294),
        pointer: with_registration.then_some(pointer),
    };
}

/// Publish one fixture record under a fixture's Git common directory.
///
/// Why:
///  The readers under control take a common directory and find the home
///  themselves,
///  so a control writes through the release writer rather than by hand.
pub(crate) fn publish(
    common_dir: &Path,
    record: &BindingRecord,
    slot: &str,
) {
    let home: std::fs::File = open_or_create_home(common_dir).expect("the fixture home opens");
    write_binding(&home, slot, record).expect("the fixture record writes");
}
