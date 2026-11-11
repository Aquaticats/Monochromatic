//! What:
//!  The `mono/dependent-version-bump` check:
//!  the native planner over the lifecycle's
//!       prepared candidate state,
//!  its findings,
//!  the corrections a direct fix applies,
//!  and the ones a
//!       commit adds to itself.
//! Why:
//!  The check keeps what the ported policy reported
//!  (`package/git-policy/cli/src/optional/repository-policy/dependent-version-bump-policy.ts`,
//!  deleted on 2026-10-13 in `642b6d72cb`):
//!  one
//!      finding per publishable dependent of a raised manifest,
//!  each with a full-content
//!      patch,
//!  on `direct-check`,
//!  `direct-fix` and a wrapped `git commit`
//!  (`doc/decision/private-npm-registry.md:158-177`,
//!  the owner's 2026-09-15 decision:
//!  a hand bump and its
//!      dependents' patch bumps land in one commit,
//!  and the worktree copies take the committed
//!      bytes while they still match `HEAD`).
//!  Every other forwarded command,
//!  `git add`
//!      above all,
//!  applies no correction and so is not planned.
//!  A dependent whose copies hold
//!      something else is named and left exactly as it is:
//!  the commit lands without it rather
//!      than failing or overwriting somebody's work.
//!  A real index that could not be listed proves
//!      none of them,
//!  so each dependent it would have proved is named with that reason
//!      instead of being taken for an index that holds nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const findings = await findDependentBumps(context); // patches applied by a direct fix
//! ```

/// The candidate's mode,
///  which a correction keeps.
use crate::candidate_object::CandidateMode;
/// The candidate's change kinds.
use crate::candidate_record::CandidateChange as RecordChange;
/// The candidate and version types.
use crate::candidate_version::{Candidate as LayerCandidate, CandidateVersion};
/// The planner's failures,
///  modes and messages.
use crate::dependent_version_content::{
    ContentUnavailable, PlanError, TrackedMode, display_path, plan_error_message,
};
/// The planner's view of the candidate state.
use crate::dependent_version_lifecycle::{LifecycleWorkspace, lifecycle_workspace};
/// The planner's bump of one manifest.
use crate::dependent_version_plan::ManifestBump;
/// The planner's request,
///  decision and findings.
use crate::dependent_version_policy::{
    Candidate, CandidateChange, DependentFinding, DependentRequest, find_dependent_bumps,
    should_plan,
};
/// The engine's failure codes.
/// The event pathname type the outcome and finding carry.
use crate::event_path::EventPath;
use crate::diagnostics::EngineFailureCode;
/// The pathname conversion every worktree read uses.
use crate::git_metadata::path_from_git_bytes;
/// The precondition for a file a direct fix did not select,
///  its reasons and its message.
use crate::policy_added_path::{
    ADDED_PATH_STAGED_REASON, added_path_message, head_mode, worktree_refusal,
};
/// The commit path's notes,
///  real-index listing and per-bump correction.
use crate::policy_dependent_version_commit::{
    RealIndexListing, added_entries, committed_correction, left_behind, note_of, noted,
};
/// The lifecycle's candidates and the correction a fix applies.
use crate::policy_content::{ContentState, Correction, LifecycleContent};
/// The finding,
///  note and outcome types of a check.
use crate::policy_engine::{PolicyFinding, PolicyNote, PolicyOutcome};
/// The lifecycle points.
use crate::policy_trigger::Trigger;
/// The facts interface that prepares candidates and locates the worktree.
use crate::repository_facts::RepositoryFacts;
/// The worktree's top level.
use crate::worktree_identity::worktree_root;
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
///  and that `git monochromatic-git-policy-cli fix` applies it.
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
///  Everything one dependent-version check reads and returns.
///  `'a` names how long the borrows of the
///       candidate state,
///  the lifecycle and the provider last;
///  `'index` is the real index's own.
/// Why:
///   One record keeps the six facts of a check together,
///  so no caller can pair a
///       trigger with the wrong lifecycle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DependentCheck = { content; lifecycle; facts; trigger; forwardsCommit; realIndex };
/// ```
pub struct DependentCheck<'a, 'index, F: RepositoryFacts> {
    /// The shared candidate state.
    pub content: &'a mut ContentState,
    /// What the lifecycle asks the candidate layer for.
    pub lifecycle: &'a LifecycleContent,
    /// The repository facts this invocation reads.
    pub facts: &'a mut F,
    /// The lifecycle point.
    pub trigger: Trigger,
    /// Whether the forwarded command is `git commit`,
    ///  which adds the dependents' manifests to
    ///       itself.
    pub forwards_commit: bool,
    /// The real index a landing writes,
    ///  when this lifecycle may add a path to what it
    ///       lands.
    pub real_index: Option<&'index Path>,
}

/// What:
///  Check the lifecycle's candidate state.
///  `<F: RepositoryFacts>` accepts any facts
///       provider;
///  `&mut ContentState` lends the shared candidates and takes corrections;
///  `forwards_commit`
///       says the forwarded command is `git commit`,
///  which adds the dependents' manifests to
///       itself;
///  `real_index` is the real index the landing writes,
/// or nothing for a lifecycle
///       that adds no path.
/// Why:
///   See the module comment.
///  Corrections are proposed on `direct-fix` and on the commit path,
///  the two
///       lifecycles that apply them;
///  every proposal of the pass is checked before any
///       is recorded,
///  so a refused one leaves the pass without partial corrections.
///  A commit
///       reports a dependent it may not add and lands without it,
///  because a commit never fails
///       over a copy somebody is holding;
///  a listing that could not run leaves every dependent it would
///       have proved out of the commit the same way,
///  each named with Git's own reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkDependentVersion(check: DependentCheck): Promise<PolicyOutcome>;
/// ```
pub fn check_dependent_version<F: RepositoryFacts>(check: DependentCheck<'_, '_, F>) -> PolicyOutcome {
    let DependentCheck {
        content,
        lifecycle,
        facts,
        trigger,
        forwards_commit,
        real_index,
    } = check;
    // A commit applies the ripple itself; every other forwarded command can only report one.
    let plans_commit: bool = trigger == Trigger::PreForward && forwards_commit;
    if trigger == Trigger::PreForward && !plans_commit {
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
        forwards_commit,
        candidates: candidates.as_slice(),
    };
    if !should_plan(&request) {
        return clean();
    }
    let root: Option<PathBuf> = if trigger == Trigger::DirectFix || plans_commit {
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
                "mono/dependent-version-bump found no prepared candidates; this is a defect in monochromatic-git-policy-cli.",
            ),
        );
    };
    let mut workspace: LifecycleWorkspace<'_> =
        lifecycle_workspace(store, corrected, listed.as_slice());
    let findings: Vec<DependentFinding> = match find_dependent_bumps(&request, &mut workspace) {
        Ok(found) => found,
        // A commit never fails over the ripple: what the planner could not read or decide is reported, and it lands without it.
        Err(error) if plans_commit => return noted(&plan_error_message(&error)),
        Err(error) => return plan_failure(&error),
    };
    let real: RealIndexListing = added_entries(
        plans_commit,
        &findings,
        version.as_ref(),
        facts,
        root.as_deref(),
        real_index,
    );
    let mut kept: Vec<DependentFinding> = Vec::new();
    let mut proposals: Vec<Correction> = Vec::new();
    let mut notes: Vec<PolicyNote> = Vec::new();
    for finding in findings {
        // A dependent nobody can bump automatically is named by a direct command and reported by a commit, which still lands.
        let Some(bump) = finding.patch.clone() else {
            if plans_commit {
                notes.push(note_of(
                    finding.message.as_str(),
                    finding.path.as_deref(),
                    Some(crate::policy_events::LEFT_ALONE_HINT),
                ));
            } else {
                kept.push(finding);
            }
            continue;
        };
        let selected: bool = version.candidate_at_path(&bump.path).is_some();
        if !plans_commit {
            if trigger == Trigger::DirectFix {
                match correction_of(&bump, selected, &mut workspace, root.as_deref()) {
                    Ok(correction) => proposals.push(correction),
                    Err(outcome) => return outcome,
                }
            }
            kept.push(finding);
            continue;
        }
        match committed_correction(&bump, selected, &mut workspace, root.as_deref(), &real) {
            Ok(correction) => {
                proposals.push(correction);
                kept.push(finding);
            }
            Err(left) => notes.push(left_behind(&bump, &left)),
        }
    }
    for correction in proposals {
        content.propose(correction);
    }
    let reported: Vec<PolicyFinding> = kept.iter().map(policy_finding).collect();
    if notes.is_empty() {
        return PolicyOutcome::Findings(reported);
    }
    return PolicyOutcome::FindingsWithNotes {
        findings: reported,
        notes,
    };
}

/// Check controls over fixture repositories stay out of the release executable.
#[cfg(test)]
#[path = "policy_dependent_version_tests.rs"]
mod tests;
