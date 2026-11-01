//! What:
//!  The normalization-only transaction: a correction whose complete intended tree equals the recorded base lands as
//!  a reconciliation of the index and the worktree copies, without an empty commit.
//! Why:
//!  The incumbent's `completeNoChangeTransaction` lands the normalization through the landing critical section
//!  against the recorded base, so a base that moved since invocation fails without touching the index
//!  (`src/policy-engine/commit-transaction-no-change.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await completeNoChangeTransaction({ workspace, capture, gitPath, cwd, repositoryRoot, mode, eligible, ... });
//! ```

use super::capture_invocation::InvocationCapture;
use super::commit_conclusion::events_with_summary;
use super::commit_landing_lock::acquire_landing_lock;
use super::commit_transaction::{CommitInvocation, TransactionResult, TransactionStop, failed};
use super::commit_workspace::CommitWorkspace;
use super::landing_attempt::{LandingOutcome, LandingPayload, LandingRequest, land_transaction};
use super::landing_findings::{LANDING_FINDING_EXIT_CODE, landing_finding};
use super::owner_lock::OwnerLock;
use super::path_bytes::path_message_text;
use super::policy_events::PolicyEvent;
use super::transaction_git::{GitFailure, GitRequest, run_git_checked};

/// The core of the no-change finding.
pub const NO_CHANGE_CORE_ID: &str = "commit-normalization";

/// The code of the no-change finding.
pub const NO_CHANGE_CODE: &str = "no-change";
use super::transaction_journal::{AddedPath, Base, TransactionMode};
use super::worktree_completion::{InstallResult, install_added_worktree_files};


/// What:
///  What the no-change decision reads.
/// Why:
///   The decision compares the intended tree with the recorded base's tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { workspace, capture, gitPath, cwd, repositoryRoot, mode, eligible, intendedTreeOid, committedPaths, ... }
/// ```
pub struct NoChangeInput<'a> {
    /// The invocation's fixed facts.
    pub invocation: &'a CommitInvocation<'a>,
    /// The published workspace.
    pub workspace: &'a mut CommitWorkspace,
    /// The invocation capture.
    pub capture: &'a InvocationCapture,
    /// The original selection mode.
    pub mode: TransactionMode,
    /// False for an explicit allow-empty, an amend and a sequencer conclusion.
    pub eligible: bool,
    /// The completely settled private tree.
    pub intended_tree: &'a str,
    /// The selection including policy additions.
    pub committed_paths: &'a [Vec<u8>],
    /// Selected corrected files that match their initial bytes.
    pub selected_worktree_paths: &'a [AddedPath],
    /// The settled pass's events.
    pub events: &'a [PolicyEvent],
    /// How many passes changed content.
    pub changed_passes: u64,
    /// The corrected paths, for the summary.
    pub changed_paths: &'a [Vec<u8>],
    /// The backoff budget for a foreign `index.lock`.
    pub index_lock_timeout_ms: u64,
}

/// What:
///  The tree of the recorded base commit, never a later moving `HEAD`.
/// Why:
///   The normalization is judged against what the invocation saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git rev-parse --verify <base>^{tree}
/// ```
fn base_tree(input: &NoChangeInput<'_>, base: &str) -> Result<String, GitFailure> {
    let revision: String = format!("{base}^{{tree}}");
    let mut request: GitRequest = GitRequest::new(input.invocation.cwd, &["rev-parse", "--verify", revision.as_str()]);
    request.without_prefix = true;
    let output = run_git_checked(input.invocation.context, &request)?;
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  The message of the no-change finding.
/// Why:
///   Worktree copies that changed since are kept and named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hasConflicts ? `No commit created: ... ${JSON.stringify(conflicted)}. Compare them with HEAD before retrying.` : '...'
/// ```
fn no_change_message(conflicted: &[Vec<u8>]) -> String {
    if conflicted.is_empty() {
        return String::from(
            "No commit created: normalization removed every selected change; matching worktree and index copies were reconciled. Hint: if you were committing only whitespace or final-newline edits, they were normalized away, so there is nothing to commit; run `git status` to confirm.",
        );
    }
    // The message shows each name as text; a name that is not UTF-8 shows its replacement characters.
    let names: Vec<String> = conflicted.iter().map(|path| { return path_message_text(path.as_slice()); }).collect();
    let listed: String = serde_json::to_string(&names).unwrap_or_default();
    return format!(
        "No commit created: normalization removed every selected change; index copies were reconciled, but worktree copies changed and were kept: {listed}. Compare them with HEAD before retrying. Hint: if you were committing only whitespace or final-newline edits, they were normalized away; `git diff HEAD -- <path>` shows what still differs."
    );
}

/// What:
///  Complete an index and worktree correction without creating an empty commit.
/// Why:
///   Nothing is returned when Git still has a commit to make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await completeNoChangeTransaction({ ... });
/// ```
pub fn complete_no_change_transaction(input: NoChangeInput<'_>) -> Result<Option<TransactionResult>, TransactionStop> {
    let Base::Commit(base) = input.capture.base.clone() else {
        return Ok(None);
    };
    if !input.eligible {
        return Ok(None);
    }
    if base_tree(&input, base.as_str())? != input.intended_tree {
        return Ok(None);
    }
    let summary: Vec<PolicyEvent> = events_with_summary(input.events.to_vec(), input.changed_passes, input.changed_paths);
    let payload: LandingPayload = LandingPayload::NormalizeOnly {
        landed_tree_oid: String::from(input.intended_tree),
    };
    let request: LandingRequest<'_> = LandingRequest {
        capture: input.capture,
        mode: input.mode,
        payload: &payload,
        committed_paths: input.committed_paths,
        added_paths: &[],
        selected_worktree_paths: input.selected_worktree_paths,
        index_lock_timeout_ms: input.index_lock_timeout_ms,
        attempt: 1,
    };
    let outcome: LandingOutcome = {
        let lock: OwnerLock = acquire_landing_lock(
            input.invocation.context,
            input.invocation.cwd,
            &input.capture.layout,
            input.invocation.environment,
        )
        .map_err(|error| { return failed(error.0); })?;
        land_transaction(
            input.invocation.context,
            input.invocation.cwd,
            &lock,
            &mut input.workspace.landing,
            &request,
        )?
    };
    if let Some(event) = landing_finding(&outcome, input.capture, None, false) {
        let mut events: Vec<PolicyEvent> = summary;
        events.push(event);
        return Ok(Some(TransactionResult {
            events,
            exit_code: LANDING_FINDING_EXIT_CODE,
            landed_oid: None,
            uses_lfs: false,
        }));
    }
    let completion: InstallResult = install_added_worktree_files(
        input.invocation.context,
        input.invocation.cwd,
        input.capture.layout.repository_root.as_path(),
        input.selected_worktree_paths,
        Some(input.workspace.object_directory.as_path()),
    )
    .map_err(|error| { return TransactionStop::Message(error.0); })?;
    input.workspace.finish_transaction();
    let mut events: Vec<PolicyEvent> = summary;
    events.push(PolicyEvent::CoreFinding {
        core_id: NO_CHANGE_CORE_ID,
        code: NO_CHANGE_CODE,
        message: no_change_message(completion.conflicted.as_slice()),
    });
    return Ok(Some(TransactionResult {
        events,
        exit_code: 1,
        landed_oid: None,
        uses_lfs: false,
    }));
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_no_change_tests.rs"]
mod tests;
