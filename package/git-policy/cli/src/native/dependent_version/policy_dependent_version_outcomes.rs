//! What:
//!  What one dependent-version check answers with:
//!  the outcome "nothing found",
//!  the failure each cause is coded as,
//!  the finding the engine reports,
//!  the correction a direct fix or a commit applies,
//!  and the conversions from the planner's own records.
//! Why:
//!  The check in `policy_dependent_version.rs` reads the candidate state and decides;
//! these are the shapes it answers with,
//! kept apart so the decision stays one readable function.
//! Every message here names the file it is about and both remedies,
//! because a person reads it and acts on it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clean = (): PolicyOutcome => ({ kind: 'findings', findings: [] });
//! ```

/// The candidate's mode,
///  which a correction keeps.
use crate::candidate_object::CandidateMode;
/// The candidate's change kinds.
use crate::candidate_record::CandidateChange as RecordChange;
/// The candidate and version types.
use crate::candidate_version::Candidate as LayerCandidate;
/// The planner's failures,
///  modes and messages.
use crate::dependent_version_content::{
    ContentUnavailable,
    PlanError,
    TrackedMode,
    display_path,
    plan_error_message,
};
/// The planner's view of the candidate state.
use crate::dependent_version_lifecycle::LifecycleWorkspace;
/// The planner's bump of one manifest.
use crate::dependent_version_plan::ManifestBump;
/// The planner's candidate and findings.
use crate::dependent_version_policy::{
    Candidate,
    CandidateChange,
    DependentFinding,
};
/// The engine's failure codes.
use crate::diagnostics::EngineFailureCode;
/// The event pathname type the finding carries.
use crate::event_path::EventPath;
/// The pathname conversion every worktree read uses.
use crate::git_metadata::path_from_git_bytes;
/// The precondition for a file a direct fix did not select,
///  its reasons and its message.
use crate::policy_added_path::{
    ADDED_PATH_STAGED_REASON,
    added_path_message,
    head_mode,
    worktree_refusal,
};
/// The correction a fix applies.
use crate::policy_content::Correction;
/// The finding and outcome types of a check.
use crate::policy_engine::{
    PolicyFinding,
    PolicyOutcome,
};
/// Borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;
/// What:
///  The outcome "nothing found".
/// Why:
///   Most lifecycles end this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clean = (): PolicyOutcome => ({ kind: 'findings', findings: [] });
/// ```
pub(crate) fn clean() -> PolicyOutcome {
    return PolicyOutcome::Findings(Vec::new());
}

/// What:
///  The outcome of a check that could not finish,
///  with its code.
/// Why:
///   Unreadable content and a refused correction each have their own code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failed = (code, message): PolicyOutcome => ({ kind: 'failed', code, message });
/// ```
pub(crate) fn failed(
    code: EngineFailureCode,
    message: String,
) -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code,
        message,
        path: None,
    };
}

/// What:
///  The planner's candidate for a layer candidate.
/// Why:
///   The planner names only the path and the change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plannerCandidate = (c) => ({ path: c.path, change: c.change });
/// ```
pub(crate) fn planner_candidate(candidate: &LayerCandidate) -> Candidate {
    let change: CandidateChange = match candidate.change {
        RecordChange::Added => CandidateChange::Added,
        RecordChange::Modified => CandidateChange::Modified,
        RecordChange::Deleted => CandidateChange::Deleted,
    };
    return Candidate {
        path: candidate.path.clone(),
        change,
    };
}

/// What:
///  The outcome for a planner failure,
///  coded by its cause.
/// Why:
///   Content that could not be read is `content-unavailable`;
///  content the planner
///       cannot use is `policy-incomplete`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const planFailure = (error: PlanError): PolicyOutcome => ({ kind: 'failed', code: codeOf(error), message: planErrorMessage(error) });
/// ```
pub(crate) fn plan_failure(error: &PlanError) -> PolicyOutcome {
    let code: EngineFailureCode = match error {
        PlanError::ContentUnavailable(_) => EngineFailureCode::ContentUnavailable,
        PlanError::PolicyIncomplete(_) => EngineFailureCode::PolicyIncomplete,
    };
    return failed(code, plan_error_message(error));
}

/// What:
///  The engine's finding for a planner finding.
///  A finding with a patch says a fix is
///       available on every lifecycle,
///  as the installed wrapper's stage says.
/// Why:
///   The person sees the dependent,
///  the versions,
///  and that `git monochromatic-git-policy-cli fix` applies it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const policyFinding = (f) => ({ code: f.code, message: f.message, path: f.path, fix: f.patch ? 'available' : 'none' });
/// ```
pub(crate) fn policy_finding(finding: &DependentFinding) -> PolicyFinding {
    return PolicyFinding {
        code: finding.code,
        message: finding.message.clone(),
        path: finding.path.as_deref().map(EventPath::from_git_bytes),
        location: None,
        fix_available: finding.patch.is_some(),
    };
}

/// What:
///  The `patch-conflict` outcome for a file the fix did not select.
/// Why:
///   The message names the file,
///  the reason,
///  and both remedies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const conflict = (path, reason): PolicyOutcome => ({ kind: 'failed', code: 'patch-conflict', message: directFixMessage({ path, reason }) });
/// ```
fn conflict(
    path: &str,
    reason: &str,
) -> PolicyOutcome {
    return failed(
        EngineFailureCode::PatchConflict,
        added_path_message(path, reason),
    );
}

/// What:
///  The planner's failure for an unreadable tracked file,
///  coded as `content-unavailable`.
/// Why:
///   The failure names the file the listing could not read,
///  so the policy reports it instead of guessing.
fn unreadable_failure(error: ContentUnavailable) -> PolicyOutcome {
    return plan_failure(&PlanError::ContentUnavailable(error));
}

/// What:
///  Whether a candidate changes its path,
///  so the tracked listing must hold it.
/// Why:
///   A deleted path is absent from the index by definition and is not required there.
pub(crate) fn is_listed_change(candidate: &&Candidate) -> bool {
    return candidate.change != CandidateChange::Deleted;
}

/// What:
///  The path of a candidate,
///  copied for the tracked listing.
/// Why:
///   The listing keeps owned pathnames,
///  so each candidate's path is copied once.
pub(crate) fn candidate_path(candidate: &Candidate) -> Vec<u8> {
    return candidate.path.clone();
}

/// What:
///  The correction a direct fix applies for one bump,
///  after checking a file the fix
///       did not select.
///  `Err` is the outcome that ends the check instead.
/// Why:
///   A selected file is corrected from its candidate bytes.
///  An unselected one must hold
///       `HEAD`'s blob in the index and the worktree;
///  then its candidate bytes are `HEAD`'s,
///       and the installer's check that the worktree still holds them repeats the
///       precondition at install time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function correctionOf(bump, selected, workspace, root): Promise<Correction>; // throws PolicyOutcome
/// ```
pub(crate) fn correction_of(
    bump: &ManifestBump,
    selected: bool,
    workspace: &mut LifecycleWorkspace<'_>,
    root: Option<&Path>,
) -> Result<Correction, PolicyOutcome> {
    let mode: CandidateMode = if bump.mode == TrackedMode::Executable {
        CandidateMode::Executable
    } else {
        CandidateMode::Regular
    };
    let correction: Correction = Correction {
        path: bump.path.clone(),
        mode,
        before: Rc::from(bump.original.as_slice()),
        after: Rc::from(bump.replacement.as_slice()),
    };
    let (false, Some(top_level)) = (selected, root) else {
        return Ok(correction);
    };
    let shown: String = display_path(bump.path.as_slice());
    let Some(index) = workspace
        .index_entry(&bump.path)
        .map_err(unreadable_failure)?
    else {
        return Err(conflict(shown.as_str(), ADDED_PATH_STAGED_REASON));
    };
    let head = workspace
        .head_entry(&bump.path)
        .map_err(unreadable_failure)?;
    let head_file_mode: CandidateMode = match head_mode(head.as_ref(), &index) {
        Ok(found) => found,
        Err(reason) => return Err(conflict(shown.as_str(), reason)),
    };
    let Some(relative) = path_from_git_bytes(&bump.path) else {
        return Err(failed(
            EngineFailureCode::ContentUnavailable,
            format!(
                "mono/dependent-version-bump cannot name the worktree copy of {shown} on this system."
            ),
        ));
    };
    let file: PathBuf = top_level.join(relative);
    match worktree_refusal(file.as_path(), head_file_mode, bump.original.as_slice()) {
        Ok(None) => return Ok(correction),
        Ok(Some(reason)) => return Err(conflict(shown.as_str(), reason)),
        Err(error) => {
            return Err(failed(
                EngineFailureCode::ContentUnavailable,
                format!(
                    "mono/dependent-version-bump could not read the worktree copy of {shown}: {error}."
                ),
            ));
        }
    }
}
