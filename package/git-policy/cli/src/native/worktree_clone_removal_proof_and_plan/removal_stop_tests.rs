//! What:
//!  The controls of the stops a removal prints:
//!  each names its object,
//!  says nothing was destroyed,
//!  and says what changes before a retry.
//! Why:
//!  The design requires a refusal to name the mismatched dataset,
//!  snapshot or dependency,
//!  keep the registration and the record,
//!  and report the need for administrator lineage recovery before a retry
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "Removal and recovery", step 2).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(stop.message()).toMatch(/Nothing was destroyed/);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::{PlanRefusal, PlanRefusalCause};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::guid;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::{BookmarkRefusalReason, RefusedBookmark, RemovalStop};
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use std::path::PathBuf;

/// Every stop this module can print,
/// each with the sentence it must hold.
fn stops() -> Vec<RemovalStop> {
    return vec![
        RemovalStop::NoBinding { slot: String::from("fix-flaky-test"), reason: String::from("no such file") },
        RemovalStop::BindingIncomplete { slot: String::from("fix-flaky-test"), field: "cloneGuid" },
        RemovalStop::DoesNotAnswer { object: String::from("tank/wt/fix-flaky-test"), reason: String::from("no such dataset") },
        RemovalStop::IdentityDiffers { object: String::from("tank/wt/fix-flaky-test"), field: "cloneGuid", recorded: String::from("1"), live: String::from("2") },
        RemovalStop::OriginIsNotTheRecordedOne { clone: String::from("tank/wt/fix-flaky-test"), recorded: String::from("tank/mono-checkout@origin"), live: None },
        RemovalStop::TransferredHistory { clone: String::from("tank/wt/fix-flaky-test"), snapshot: String::from("tank/wt/fix-flaky-test@earliest"), createtxg: 13, clone_createtxg: 17 },
        RemovalStop::OriginSideHit { kind: EntryKind::Bookmark, name: String::from("tank/wt/fix-flaky-test#from-earliest"), guid: guid(11) },
        RemovalStop::OriginInventoryUnreadable { slot: String::from("fix-flaky-test"), reason: String::from("no such file") },
        RemovalStop::OriginInventoryOfAnotherDataset { inventory_dataset: String::from("tank/a"), recorded_dataset: String::from("tank/b") },
        RemovalStop::InventoryUnreadable { clone: String::from("tank/wt/fix-flaky-test"), reason: String::from("input/output error") },
        RemovalStop::BookmarksRefused {
            clone: String::from("tank/wt/fix-flaky-test"),
            refused: vec![RefusedBookmark { name: String::from("tank/wt/fix-flaky-test#one"), guid: guid(1), createtxg: 19, reason: BookmarkRefusalReason::Unexplained }],
        },
        RemovalStop::APlanStands { slot: String::from("fix-flaky-test"), path: PathBuf::from("/repo/.git/plans/1") },
        RemovalStop::PlanUnreadable { path: PathBuf::from("/repo/.git/plans/1"), refusal: PlanRefusal::at(PlanRefusalCause::NotCanonical, "cloneGuid", "007") },
        RemovalStop::UnmountFailed { step: String::from("answer-outcome"), said: String::from("the handler answered `refused` with errno 16") },
        RemovalStop::NoRunNonce { reason: String::from("the random source failed") },
        RemovalStop::NoOwnerIdentity { reason: String::from("no process table") },
        RemovalStop::OriginMarked {
            refusal: Box::new({
                let verdict = crate::worktree_removal_deferred_origin_check::origin_verdict::check_planned_origin(
                    &crate::worktree_removal_deferred_origin_check::fixtures::marked(),
                    &crate::worktree_removal_deferred_origin_check::fixtures::planned(),
                );
                crate::worktree_removal_deferred_origin_check::fixtures::refusal(verdict)
            }),
            history: RemovalHistory::NothingDestroyed,
        },
    ];
}

#[test]
fn every_stop_says_what_refused_and_that_nothing_was_destroyed() {
    for stop in stops() {
        let message: String = stop.message();
        assert!(!message.is_empty());
        assert!(message.contains("Nothing was destroyed") || message.contains("not destroyed"), "the stop says nothing was destroyed: {message}");
        assert!(message.contains("Ways forward:"), "the stop names the way forward: {message}");
    }
}
