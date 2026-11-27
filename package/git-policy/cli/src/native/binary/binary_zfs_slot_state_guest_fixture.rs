//! What:
//!  The guest arm's own fixture:
//!  where one slot and its clone stand,
//!  the binding record of any phase read from the guest's pool,
//!  the publication that puts it under the slot's name,
//!  and the live reads one slot's state is decided from.
//! Why:
//!  Plan step 12's guest arm makes a real registration,
//! a real clone and a real
//!  move,
//!  so the record it writes must name the guest's own datasets and identities
//!  rather than a host fixture's,
//!  and the arm file stays about the scenes it proves.
//!  Nothing here is part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const place = { checkout, common, pool, checkoutDataset, cloneDataset, slot, slotIdentity, registration };
//! publish(place, Phase.Filled, sbin, 'one');
//! ```

use super::zfs_sweep_guest::{
    NONCE,
    ORIGIN_PREFIX,
    zfs,
};
use git_policy_cli::worktree_clone_binding_record::binding_encoding::{
    FolderIdentity,
    Request,
};
use git_policy_cli::worktree_clone_binding_record::binding_record::BindingRecord;
use git_policy_cli::worktree_clone_binding_record::binding_words::{
    Phase,
    RequestForm,
};
use git_policy_cli::worktree_clone_binding_record::record_publication::write_binding;
use git_policy_cli::worktree_clone_binding_record::registration_home::open_or_create_home;
use git_policy_cli::worktree_clone_mount_protocol::{
    DatasetGuid,
    RequestNonce,
};
use git_policy_cli::worktree_clone_slot_state::slot_state_classification::SlotRouteState;
use git_policy_cli::worktree_clone_slot_state::slot_state_live_read::{
    GatedCloneIntentReads,
    SlotReadingSources,
    read_slot,
};
use git_policy_cli::worktree_clone_slot_state::slot_state_readings::{
    IntentFacts,
    SlotReadings,
};
use git_policy_cli::zfs_compatibility_gate::gate_verdict::LiveGate;
use git_policy_cli::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Output,
};

/// Where one arm's slot and clone stand.
pub(super) struct Place {
    /// The checkout's Git common directory.
    pub(super) common: PathBuf,
    /// The pool the clone was made in.
    pub(super) pool: String,
    /// The checkout's dataset.
    pub(super) checkout_dataset: String,
    /// The clone dataset the record names,
    /// which may not exist for a record written before the clone.
    pub(super) clone_dataset: String,
    /// The slot's whole path.
    pub(super) slot: PathBuf,
    /// The slot's underlay identity,
    /// read while nothing was mounted there.
    pub(super) slot_identity: FolderIdentity,
    /// The underlay pointer Git wrote into the empty slot,
    /// read before the clone was mounted over it.
    pub(super) pointer: Vec<u8>,
    /// The slot's Git registration.
    pub(super) registration: PathBuf,
}

/// One property of one dataset as `zfs get` prints it.
fn value_of(
    sbin: &str,
    dataset: &str,
    property: &str,
) -> String {
    let (ok, said, error) = zfs(
        sbin,
        "zfs",
        &["get", "-H", "-p", "-o", "value", property, dataset],
        None,
    );
    assert!(ok, "`{property}` of `{dataset}` is read: {said}{error}");
    return said.trim().to_string();
}

/// One dataset's canonical GUID.
pub(super) fn guid_of(
    sbin: &str,
    dataset: &str,
) -> DatasetGuid {
    return DatasetGuid::parse(value_of(sbin, dataset, "guid").as_str())
        .expect("the dataset's GUID reads");
}

/// One dataset's creation transaction group,
/// which is the group a clone's own snapshots stand above.
pub(super) fn createtxg_of(
    sbin: &str,
    dataset: &str,
) -> u64 {
    return value_of(sbin, dataset, "createtxg")
        .parse()
        .expect("the dataset's createtxg reads");
}

/// One binding record of one phase,
/// with every field that phase proves,
/// read from the guest's own pool.
fn record_at(
    phase: Phase,
    place: &Place,
    sbin: &str,
) -> BindingRecord {
    let common_identity: FolderIdentity = FolderIdentity::of_folder(place.common.as_path())
        .expect("the common directory's identity reads");
    let registration_identity: FolderIdentity =
        FolderIdentity::of_folder(place.registration.as_path())
            .expect("the registration's identity reads");
    let snapshot: String = format!("{}@{ORIGIN_PREFIX}{NONCE}", place.checkout_dataset);
    let pointer: Vec<u8> = place.pointer.clone();
    let with_clone: bool = matches!(
        phase,
        Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling
    );
    let clone_guid: Option<DatasetGuid> =
        with_clone.then(|| return guid_of(sbin, place.clone_dataset.as_str()));
    let clone_createtxg: Option<u64> = with_clone.then(|| {
        return createtxg_of(sbin, place.clone_dataset.as_str());
    });
    return BindingRecord {
        claim: 1,
        nonce: RequestNonce::parse(NONCE).expect("the arm's nonce reads"),
        request: Request::with_argument(RequestForm::Branch, b"topic"),
        phase,
        settle_reason: None,
        checkout_dataset: place.checkout_dataset.clone(),
        checkout_dataset_guid: guid_of(sbin, place.checkout_dataset.as_str()),
        repository: place.common.as_os_str().as_encoded_bytes().to_vec(),
        repository_identity: common_identity,
        slot: place.slot.as_os_str().as_encoded_bytes().to_vec(),
        slot_identity: Some(place.slot_identity),
        registration: Some(place.registration.as_os_str().as_encoded_bytes().to_vec()),
        registration_identity: Some(registration_identity),
        pool: place.pool.clone(),
        pool_guid: guid_of(sbin, place.pool.as_str()),
        origin_snapshot: snapshot.clone(),
        origin_snapshot_guid: Some(guid_of(sbin, snapshot.as_str())),
        origin_snapshot_createtxg: Some(
            value_of(sbin, snapshot.as_str(), "createtxg")
                .parse()
                .expect("the snapshot's createtxg reads"),
        ),
        origin_dataset: Some(place.checkout_dataset.clone()),
        origin_dataset_guid: Some(guid_of(sbin, place.checkout_dataset.as_str())),
        clone_dataset: place.clone_dataset.clone(),
        clone_guid,
        clone_createtxg,
        pointer: Some(pointer),
    };
}

/// Publish one record under a slot's name in the checkout's record home.
pub(super) fn publish(
    place: &Place,
    phase: Phase,
    sbin: &str,
    slot: &str,
) {
    let home: std::fs::File =
        open_or_create_home(place.common.as_path()).expect("the record home opens");
    write_binding(&home, slot, &record_at(phase, place, sbin)).expect("the record writes");
}

/// The facts one slot's state is read from,
/// with the live mount table and the gated intent read.
pub(super) fn readings_of(
    place: &Place,
    slot: &str,
    slot_path: &Path,
) -> SlotReadings {
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    let gate: LiveGate = LiveGate;
    let intents: GatedCloneIntentReads<'_> = GatedCloneIntentReads::new(&gate);
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    return read_slot(place.common.as_path(), slot, slot_path, &sources);
}

/// One intent's word for a marker.
pub(super) fn intent_word(facts: &IntentFacts) -> &'static str {
    return match facts {
        IntentFacts::Unasked => "unasked",
        IntentFacts::Absent => "none",
        IntentFacts::Mounting {
            ..
        } => "mount-requested",
        IntentFacts::Unmounting {
            ..
        } => "unmount-requested",
        IntentFacts::Inherited {
            ..
        } => "inherited",
        IntentFacts::Unreadable {
            ..
        } => "unreadable",
    };
}

/// One state's class word for a marker.
pub(super) fn class_of(state: &SlotRouteState) -> &'static str {
    return state.word();
}

/// Run one Git command in the checkout and hand back its answer.
pub(super) fn git(
    checkout: &Path,
    arguments: &[&str],
) -> Output {
    return Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("git {arguments:?} could not start: {error}"));
}
