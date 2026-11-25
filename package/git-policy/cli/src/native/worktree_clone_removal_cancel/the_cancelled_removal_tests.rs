//! What:
//!  Controls for what a removal restores when it cancels itself:
//!  each stop point's first line,
//!  the holder it names,
//!  the destroyed set it reports,
//!  and the status the removal exits with.
//! Why:
//!  The project's answer makes a seen holder cancel before any delete and decide that a
//!  cancellation after part of the history is gone attaches the clone again and reports each
//!  destroyed snapshot
//!  (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
//!  "A cancellation after part of the history is gone").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(done.lines()[0]).toContain('nothing was destroyed and every snapshot survives');
//! ```

use super::{CANCELLED_BY_ITSELF, TheSelfCancellation, TheSelfCancellationDone, WhereItStopped, the_removal_cancels_itself};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_removal_proof_and_plan::fixtures::guid;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_cancel::cancel_stand_ins::{RecordingMountRequest, ScriptedStaged, an_order};
use crate::worktree_clone_removal_cancel::the_destroy_outcomes::TheDestroyOutcomes;
use crate::worktree_clone_removal_cancel::the_reversing_request::{MountAnswer, MountIdentity};
use crate::worktree_clone_removal_cancel::the_staged_state::StagedSettlement;
use std::path::{Path, PathBuf};

/// One folder identity,
/// written from its own grammar so a control needs no folder to answer it.
fn an_identity() -> crate::worktree_clone_binding_record::binding_encoding::FolderIdentity {
    return crate::worktree_clone_binding_record::binding_encoding::FolderIdentity::decode("2:3:4").expect("the grammar reads one identity");
}


/// The record the cancellation names.
fn recorded() -> RecordedBinding {
    return RecordedBinding {
        slot: String::from("one"),
        slot_path: PathBuf::from("/repo/.claude/worktrees/one"),
        pool: String::from("tank"),
        pool_guid: guid(1_234_567_890_123_456_789),
        clone_dataset: String::from("tank/clone"),
        clone_guid: guid(7_828_944_690_444_299_989),
        clone_createtxg: 17,
        origin_dataset: String::from("tank/mono-checkout"),
        origin_dataset_guid: guid(4_815_162_342_109_876_543),
        origin_snapshot: String::from("tank/mono-checkout@origin"),
        origin_snapshot_guid: guid(11_050_373_805_378_911_967),
        origin_snapshot_createtxg: 15,
        registration_id: Vec::from("one".as_bytes()),
        registration_identity: an_identity(),
    };
}

/// The answer a handler gives for the clone put back at its slot.
fn an_answer() -> MountAnswer {
    return MountAnswer {
        dataset: guid(7_828_944_690_444_299_989),
        operation: MountOperation::Mount,
        nonce: crate::worktree_clone_mount_protocol::request_nonce::RequestNonce::parse("9c410000000000000000000000000000").expect("a nonce"),
        outcome: ReceiptOutcome::Done,
        mount: Some(MountIdentity { mount_id: 3, source: String::from("tank/clone"), canonical_path: String::from("/repo/.claude/worktrees/one") }),
        failed_step: None,
        errno: None,
    };
}

/// One cancellation at one stop point.
fn a_cancellation(name: &str, stop: WhereItStopped, destroyed: &[&str], holders: &[&str]) -> TheSelfCancellationDone {
    let common: PathBuf = fixture(&format!("cancel-itself-{name}"));
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_answer()) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::Removed { record: true, claim: true } };
    let outcomes: TheDestroyOutcomes = TheDestroyOutcomes {
        destroyed: destroyed.iter().map(|lost| return String::from(*lost)).collect(),
        everything: Vec::new(),
    };
    let holder_lines: Vec<String> = holders.iter().map(|holder| return String::from(*holder)).collect();
    let record: RecordedBinding = recorded();
    return the_removal_cancels_itself(&TheSelfCancellation {
        common_dir: common.as_path(),
        recorded: &record,
        slot_path: Path::new("/repo/.claude/worktrees/one"),
        plan: None,
        outcomes: &outcomes,
        stop,
        holders: holder_lines.as_slice(),
        mount_request: &request,
        staged: &staged,
    })
    .expect("the cancellation reverses the request");
}

#[test]
fn an_arm_that_destroyed_nothing_says_every_snapshot_survives() {
    for stop in [WhereItStopped::BeforeTheMove, WhereItStopped::InStaging, WhereItStopped::ByAScanAfterTheUnmount] {
        let done: TheSelfCancellationDone = a_cancellation(&format!("nothing-{stop:?}"), stop, &[], &["mnt:[4026532396] holds the clone"]);
        let text: String = done.lines().join("\n");
        assert!(text.contains("nothing was destroyed and every snapshot survives"), "{stop:?}: {text}");
        assert!(text.contains("mounted at its slot"), "{stop:?}: {text}");
        assert!(text.contains("mnt:[4026532396] holds the clone"), "{stop:?}: {text}");
        assert!(!text.contains("cannot be restored"), "{stop:?}: {text}");
        assert_eq!(done.exit_code(), CANCELLED_BY_ITSELF);
    }
}

#[test]
fn an_arm_that_destroyed_a_chunk_reports_each_name_and_that_it_cannot_be_restored() {
    let done: TheSelfCancellationDone = a_cancellation(
        "chunk",
        WhereItStopped::ByAScanBeforeADelete,
        &["tank/clone@sanoid-hourly-2026-11-22_03", "tank/clone@sanoid-hourly-2026-11-22_04"],
        &[],
    );
    let text: String = done.lines().join("\n");
    assert!(text.contains("snapshots were destroyed and cannot be restored"), "{text}");
    assert!(text.contains("tank/clone@sanoid-hourly-2026-11-22_03"), "{text}");
    assert!(text.contains("tank/clone@sanoid-hourly-2026-11-22_04"), "{text}");
    assert!(text.contains("This removal destroyed 2 snapshot(s)"), "{text}");
    assert!(text.contains(done.nonce.as_str()), "the report names the request's own nonce: {text}");
    assert!(text.contains("mount 3"), "{text}");
    assert!(text.contains("staging folder, its record and its claim are gone"), "{text}");
    assert!(text.contains("cancel-worktree-removal"), "the way forward names the command: {text}");
    assert_eq!(done.exit_code(), CANCELLED_BY_ITSELF);
}

#[test]
fn the_head_destroy_arm_says_the_snapshot_list_was_empty_and_that_a_holder_outlasted_it() {
    let done: TheSelfCancellationDone = a_cancellation("head", WhereItStopped::AtTheHeadDestroy, &["tank/clone@one"], &[]);
    let text: String = done.lines().join("\n");
    assert!(text.contains("dataset is busy"), "{text}");
    assert!(text.contains("with the clone's own snapshot list empty"), "{text}");
    assert!(text.contains("tank/clone@one"), "{text}");
}

#[test]
fn the_forced_arms_failure_says_what_it_was_asked_for_and_that_the_clone_stands_at_its_slot() {
    let done: TheSelfCancellationDone = a_cancellation("forced", WhereItStopped::TheForcedArmsFailure, &[], &["the copy is locked by a user namespace of its own"]);
    let text: String = done.lines().join("\n");
    assert!(text.contains("asked to end the holders other than your protected set and could not"), "{text}");
    assert!(text.contains("the copy is locked by a user namespace of its own"), "{text}");
    assert!(text.contains("mounted at its slot"), "{text}");
}

#[test]
fn every_stop_point_says_which_one_it_is_and_where_it_stopped() {
    for stop in [
        WhereItStopped::BeforeTheMove,
        WhereItStopped::InStaging,
        WhereItStopped::ByAScanAfterTheUnmount,
        WhereItStopped::ByAScanBeforeADelete,
        WhereItStopped::AtTheHeadDestroy,
        WhereItStopped::TheForcedArmsFailure,
    ] {
        assert!(!stop.clause().is_empty(), "{stop:?} carries no clause");
        let done: TheSelfCancellationDone = a_cancellation(&format!("every-{stop:?}"), stop, &[], &[]);
        assert_eq!(done.stop, stop);
        let text: String = done.lines().join("\n");
        assert!(text.contains(stop.clause()), "{stop:?}: {text}");
        assert!(text.contains("tank/clone"), "the dataset is named: {text}");
        assert!(text.contains(&guid(7_828_944_690_444_299_989).file_name()), "the GUID is named: {text}");
        assert!(text.contains("Ways forward"), "a way forward is named: {text}");
    }
}
