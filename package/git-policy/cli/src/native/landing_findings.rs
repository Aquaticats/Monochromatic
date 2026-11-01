//! What:
//!  The `concurrent-commit` finding for a landing that landed nothing because `HEAD` was switched or the target moved.
//! Why:
//!  Both leave the ref, the real index and the worktree bytes unchanged and end the commit with exit status 1
//!  (`src/policy-engine/commit-landing-findings.ts`).
//!  This module only builds the event and names the exit status;
//!  the commit path that renders it is a later slice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = landingFindingResult({ pass, outcome, capture, preparedOid, replayPlumbingMissing });
//! ```

/// The invocation facts the message names.
use super::capture_invocation::InvocationCapture;
/// The event and its rendering.
use super::policy_events::PolicyEvent;
/// Landing outcomes and journal enums.
use super::landing_attempt::LandingOutcome;
use super::transaction_journal::{Base, Conclusion, SymbolicHead};

/// Exit status of a commit that landed nothing because of a concurrent change.
pub const LANDING_FINDING_EXIT_CODE: i32 = 1;

/// What:
///  Describes a target value.
/// Why:
///   The message names the commit, or says the branch has none yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value.kind === 'commit' ? value.oid : 'unborn'
/// ```
fn describe_target(value: &Base) -> &str {
    return match value {
        Base::Commit(oid) => oid.as_str(),
        Base::Unborn => "unborn",
    };
}

/// What:
///  Describes a symbolic `HEAD` target.
/// Why:
///   The message names the branch, or says `HEAD` is detached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value.kind === 'branch' ? value.ref : 'detached HEAD'
/// ```
fn describe_head(value: &SymbolicHead) -> &str {
    return match value {
        SymbolicHead::Branch(reference) => reference.as_str(),
        SymbolicHead::Detached => "detached HEAD",
    };
}

/// What:
///  Why a moved target cannot take this commit by replay.
/// Why:
///   The message tells the person what to do instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// replayPlumbingMissing ? '...' : capture.conclusion === 'none' ? '...' : `an ${...} is never replayed`
/// ```
fn replay_reason(capture: &InvocationCapture, replay_plumbing_missing: bool) -> String {
    if replay_plumbing_missing {
        return String::from(
            "this Git cannot replay it: its `git merge-tree` has no `--merge-base` option (Git 2.40.0 or later has it)",
        );
    }
    return match capture.layout.conclusion {
        Conclusion::None => String::from("the target no longer names a commit to replay it onto"),
        Conclusion::Amend => String::from("an amend is never replayed"),
        Conclusion::Merge => String::from("an merge conclusion is never replayed"),
        Conclusion::CherryPick => String::from("an cherry-pick conclusion is never replayed"),
        Conclusion::Revert => String::from("an revert conclusion is never replayed"),
    };
}

/// What:
///  The blocking finding of an outcome that landed nothing;
///  none for a landing that landed.
/// Why:
///   `landingFindingResult`:
///  the finding is a fixed-transform rejection of the `concurrent-commit` transform,
///  and the commit exits with `LANDING_FINDING_EXIT_CODE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createCoreFindingEvent({ sequence, coreId: 'concurrent-commit', code, message })
/// ```
pub fn landing_finding(
    outcome: &LandingOutcome,
    capture: &InvocationCapture,
    prepared_oid: Option<&str>,
    replay_plumbing_missing: bool,
) -> Option<PolicyEvent> {
    let prepared: String = match prepared_oid {
        Some(oid) => format!(" The prepared commit {oid} was discarded."),
        None => String::new(),
    };
    match outcome {
        LandingOutcome::Landed { .. } => return None,
        LandingOutcome::BranchSwitched { current } => {
            return Some(PolicyEvent::CoreFinding {
                core_id: "concurrent-commit",
                code: "branch-switched",
                message: format!(
                    "HEAD now names {} instead of {} as it did when this commit started; nothing landed.{prepared} Switch back or commit again on the current branch.",
                    describe_head(current),
                    describe_head(&capture.layout.symbolic_head),
                ),
            });
        }
        LandingOutcome::HeadMoved { current, .. } => {
            let target: &str = capture.layout.target_ref.as_str();
            return Some(PolicyEvent::CoreFinding {
                core_id: "concurrent-commit",
                code: "head-moved",
                message: format!(
                    "{target} moved from {} to {} while this commit was prepared, and {}; nothing landed.{prepared} Commit again against the new {target}.",
                    describe_target(&capture.base),
                    describe_target(current),
                    replay_reason(capture, replay_plumbing_missing),
                ),
            });
        }
    }
}

/// Finding controls stay out of the release executable.
#[cfg(test)]
#[path = "landing_findings_tests.rs"]
mod tests;
