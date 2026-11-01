//! What:
//!  The `mono/dependent-version-bump` check:
//!  the native planner over the lifecycle's
//!       prepared candidate state,
//!  its findings,
//!  and the corrections a direct fix applies.
//! Why:
//!  The installed wrapper's policy (`dependent-version-bump-policy.ts`) reports one
//!      finding per publishable dependent of a raised manifest,
//!  each with a full-content
//!      patch,
//!  on `direct-check` and `direct-fix`;
//!  on `pre-forward` it plans only for a
//!      forwarded `commit`.
//!  A real `git commit` runs through the commit transaction instead of being forwarded,
//!  so `pre-forward` reaches this check only
//!      for `git add`,
//!  where the policy finds nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const findings = await findDependentBumps(context); // patches applied by a direct fix
//! ```

/// The candidate's mode,
///  which a correction keeps.
use super::candidate_object::CandidateMode;
/// The candidate's change kinds.
use super::candidate_record::CandidateChange as RecordChange;
/// The candidate and version types.
use super::candidate_version::{Candidate as LayerCandidate, CandidateVersion};
/// The planner's failures,
///  modes and messages.
use super::dependent_version_content::{
    ContentUnavailable, PlanError, TrackedMode, display_path, plan_error_message,
};
/// The planner's view of the candidate state.
use super::dependent_version_lifecycle::{LifecycleWorkspace, lifecycle_workspace};
/// The planner's bump of one manifest.
use super::dependent_version_plan::ManifestBump;
/// The planner's request,
///  decision and findings.
use super::dependent_version_policy::{
    Candidate, CandidateChange, DependentFinding, DependentRequest, find_dependent_bumps,
    should_plan,
};
/// The engine's failure codes.
/// The event pathname type the outcome and finding carry.
use super::event_path::EventPath;
use super::diagnostics::EngineFailureCode;
/// The pathname conversion every worktree read uses.
use super::git_metadata::path_from_git_bytes;
/// The precondition for a file the fix did not select.
use super::policy_added_path::{
    ADDED_PATH_STAGED_REASON, added_path_message, head_mode, worktree_refusal,
};
/// The lifecycle's candidates and the correction a fix applies.
use super::policy_content::{ContentState, Correction, LifecycleContent};
/// The finding and outcome types of a check.
use super::policy_engine::{PolicyFinding, PolicyOutcome};
/// The lifecycle points.
use super::policy_trigger::Trigger;
/// The facts interface that prepares candidates and locates the worktree.
use super::repository_facts::RepositoryFacts;
/// The worktree's top level.
use super::worktree_identity::worktree_root;
/// Borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
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
fn clean() -> PolicyOutcome {
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
fn failed(code: EngineFailureCode, message: String) -> PolicyOutcome {
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
fn planner_candidate(candidate: &LayerCandidate) -> Candidate {
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
fn plan_failure(error: &PlanError) -> PolicyOutcome {
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
///  and that `git cli-git fix` applies it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const policyFinding = (f) => ({ code: f.code, message: f.message, path: f.path, fix: f.patch ? 'available' : 'none' });
/// ```
fn policy_finding(finding: &DependentFinding) -> PolicyFinding {
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
fn conflict(path: &str, reason: &str) -> PolicyOutcome {
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
fn is_listed_change(candidate: &&Candidate) -> bool {
    return candidate.change != CandidateChange::Deleted;
}

/// What:
///  The path of a candidate,
///  copied for the tracked listing.
/// Why:
///   The listing keeps owned pathnames,
///  so each candidate's path is copied once.
fn candidate_path(candidate: &Candidate) -> Vec<u8> {
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
fn correction_of(
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
    let Some(index) = workspace.index_entry(&bump.path).map_err(unreadable_failure)? else {
        return Err(conflict(shown.as_str(), ADDED_PATH_STAGED_REASON));
    };
    let head = workspace.head_entry(&bump.path).map_err(unreadable_failure)?;
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

/// What:
///  Check the lifecycle's candidate state.
///  `<F: RepositoryFacts>` accepts any facts
///       provider;
///  `&mut ContentState` lends the shared candidates and takes corrections.
/// Why:
///   See the module comment.
///  Corrections are proposed only on `direct-fix`,
///  the one
///       lifecycle that applies them;
///  every proposal of the pass is checked before any
///       is recorded,
///  so a refused one leaves the pass without partial corrections.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkDependentVersion(content, lifecycle, facts, trigger): Promise<PolicyOutcome>;
/// ```
pub fn check_dependent_version<F: RepositoryFacts>(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    facts: &mut F,
    trigger: Trigger,
) -> PolicyOutcome {
    if trigger == Trigger::PreForward {
        return clean();
    }
    let version: Rc<CandidateVersion> = match content.version(lifecycle, facts) {
        Ok(Some(found)) => found,
        Ok(None) => return clean(),
        Err(outcome) => return outcome,
    };
    let candidates: Vec<Candidate> = version.candidates().iter().map(planner_candidate).collect();
    let request: DependentRequest<'_> = DependentRequest {
        trigger,
        forwards_commit: false,
        candidates: candidates.as_slice(),
    };
    if !should_plan(&request) {
        return clean();
    }
    let root: Option<PathBuf> = if trigger == Trigger::DirectFix {
        match facts.location() {
            Ok(location) => worktree_root(&location.identity).map(Path::to_path_buf),
            Err(message) => return failed(EngineFailureCode::ContentUnavailable, message),
        }
    } else {
        None
    };
    let listed: Vec<Vec<u8>> = candidates
        .iter()
        .filter(is_listed_change)
        .map(candidate_path)
        .collect();
    let Some((store, corrected)) = content.store_with_corrections() else {
        return failed(
            EngineFailureCode::ContentUnavailable,
            String::from(
                "mono/dependent-version-bump found no prepared candidates; this is a defect in cli-git.",
            ),
        );
    };
    let mut workspace: LifecycleWorkspace<'_> =
        lifecycle_workspace(store, corrected, listed.as_slice());
    let findings: Vec<DependentFinding> = match find_dependent_bumps(&request, &mut workspace) {
        Ok(found) => found,
        Err(error) => return plan_failure(&error),
    };
    let mut proposals: Vec<Correction> = Vec::new();
    for finding in &findings {
        if let (Trigger::DirectFix, Some(bump)) = (trigger, finding.patch.as_ref()) {
            let selected: bool = version.candidate_at_path(&bump.path).is_some();
            match correction_of(bump, selected, &mut workspace, root.as_deref()) {
                Ok(correction) => proposals.push(correction),
                Err(outcome) => return outcome,
            }
        }
    }
    for correction in proposals {
        content.propose(correction);
    }
    return PolicyOutcome::Findings(findings.iter().map(policy_finding).collect());
}

/// Check controls over fixture repositories stay out of the release executable.
#[cfg(test)]
#[path = "policy_dependent_version_tests.rs"]
mod tests;
