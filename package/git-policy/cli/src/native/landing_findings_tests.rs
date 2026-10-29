//! Controls for the `concurrent-commit` finding of a landing that landed nothing.

use super::*;
use crate::capture_invocation::InvocationLayout;
use crate::policy_events::render_policy_event;
use crate::transaction_journal::RefFormat;
use std::path::PathBuf;

/// A capture on `refs/heads/main` at commit `a`, with the conclusion under test.
fn capture(conclusion: Conclusion, base: Base) -> InvocationCapture {
    return InvocationCapture {
        base,
        layout: InvocationLayout {
            symbolic_head: SymbolicHead::Branch(String::from("refs/heads/main")),
            target_ref: String::from("refs/heads/main"),
            conclusion,
            repository_root: PathBuf::from("/repo"),
            git_dir: PathBuf::from("/repo/.git"),
            common_dir: PathBuf::from("/repo/.git"),
            real_index_path: PathBuf::from("/repo/.git/index"),
            object_directory: PathBuf::from("/repo/.git/objects"),
            registry_root: PathBuf::from("/repo/.git/cli-git-transactions"),
            ref_format: RefFormat::Files,
            object_format: String::from("sha1"),
            empty_tree_oid: "4".repeat(40),
            invoked_at: String::from("2026-10-11T00:00:00.000Z"),
            config_snapshot: None,
        },
    };
}

/// The message of a finding.
fn message_of(event: Option<PolicyEvent>) -> String {
    match event {
        Some(PolicyEvent::CoreFinding { message, .. }) => return message,
        other => panic!("expected a core finding, got {other:?}"),
    }
}

/// A landing that landed has no finding, so nothing blocks the commit.
#[test]
fn a_landed_commit_has_no_finding() {
    let outcome = LandingOutcome::Landed { oid: "b".repeat(40) };
    assert_eq!(landing_finding(&outcome, &capture(Conclusion::None, Base::Unborn), None, false), None);
}

/// A switched branch renders the exact line of the incumbent, with the discarded commit named.
#[test]
fn a_switched_branch_renders_the_incumbents_line() {
    let outcome = LandingOutcome::BranchSwitched { current: SymbolicHead::Branch(String::from("refs/heads/other")) };
    let event = landing_finding(&outcome, &capture(Conclusion::None, Base::Commit("a".repeat(40))), Some("c0ffee"), false)
        .expect("a finding");
    assert_eq!(
        render_policy_event(0, &event),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"core-finding\",\"trigger\":\"pre-forward\",\"coreId\":\"concurrent-commit\",\"code\":\"concurrent-commit/branch-switched\",\"message\":\"HEAD now names refs/heads/other instead of refs/heads/main as it did when this commit started; nothing landed. The prepared commit c0ffee was discarded. Switch back or commit again on the current branch.\"}\n"
    );
    assert_eq!(LANDING_FINDING_EXIT_CODE, 1);
}

/// A detached `HEAD` is named as such, and a normalization names no discarded commit.
#[test]
fn a_detached_head_is_named_and_a_normalization_names_no_commit() {
    let outcome = LandingOutcome::BranchSwitched { current: SymbolicHead::Detached };
    let message = message_of(landing_finding(&outcome, &capture(Conclusion::None, Base::Unborn), None, false));
    assert_eq!(
        message,
        "HEAD now names detached HEAD instead of refs/heads/main as it did when this commit started; nothing landed. Switch back or commit again on the current branch."
    );
}

/// A moved target with nothing to replay onto says so.
#[test]
fn a_moved_target_without_a_commit_says_there_is_nothing_to_replay_onto() {
    let outcome = LandingOutcome::HeadMoved { current: Base::Unborn, replayable: false };
    let event = landing_finding(&outcome, &capture(Conclusion::None, Base::Commit("a".repeat(40))), Some("c0ffee"), false)
        .expect("a finding");
    assert!(matches!(&event, PolicyEvent::CoreFinding { code: "head-moved", core_id: "concurrent-commit", .. }));
    assert_eq!(
        message_of(Some(event)),
        format!(
            "refs/heads/main moved from {} to unborn while this commit was prepared, and the target no longer names a commit to replay it onto; nothing landed. The prepared commit c0ffee was discarded. Commit again against the new refs/heads/main.",
            "a".repeat(40)
        )
    );
}

/// An amend and each conclusion give their own reason, with the incumbent's wording.
#[test]
fn conclusions_give_their_own_reason() {
    let moved = LandingOutcome::HeadMoved { current: Base::Commit("b".repeat(40)), replayable: false };
    let reasons: [(Conclusion, &str); 4] = [
        (Conclusion::Amend, "an amend is never replayed"),
        (Conclusion::Merge, "an merge conclusion is never replayed"),
        (Conclusion::CherryPick, "an cherry-pick conclusion is never replayed"),
        (Conclusion::Revert, "an revert conclusion is never replayed"),
    ];
    for (conclusion, reason) in reasons {
        let message = message_of(landing_finding(&moved, &capture(conclusion, Base::Unborn), None, false));
        assert_eq!(
            message,
            format!(
                "refs/heads/main moved from unborn to {} while this commit was prepared, and {reason}; nothing landed. Commit again against the new refs/heads/main.",
                "b".repeat(40)
            )
        );
    }
}

/// Missing replay plumbing takes precedence over the conclusion as the reason.
#[test]
fn missing_replay_plumbing_is_the_reason_when_the_git_lacks_it() {
    let moved = LandingOutcome::HeadMoved { current: Base::Commit("b".repeat(40)), replayable: true };
    let message = message_of(landing_finding(&moved, &capture(Conclusion::Amend, Base::Unborn), None, true));
    assert!(
        message.contains("and this Git cannot replay it: its `git merge-tree` has no `--merge-base` option (Git 2.40.0 or later has it); nothing landed."),
        "{message}"
    );
}
