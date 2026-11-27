//! What:
//!  The `mono/dependent-version-bump` check:
//!  the native planner over the lifecycle's
//!       prepared candidate state,
//!  and the one function that decides.
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
//!  The shapes it answers with are `policy_dependent_version_outcomes.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const findings = await findDependentBumps(context); // patches applied by a direct fix
//! ```

/// The candidate and version types.
use crate::candidate_version::CandidateVersion;
/// The planner's messages.
use crate::dependent_version_content::plan_error_message;
/// The planner's view of the candidate state.
use crate::dependent_version_lifecycle::{
    LifecycleWorkspace,
    lifecycle_workspace,
};
/// The planner's request,
///  decision and findings.
use crate::dependent_version_policy::{
    Candidate,
    DependentFinding,
    DependentRequest,
    find_dependent_bumps,
    should_plan,
};
/// The engine's failure codes.
use crate::diagnostics::EngineFailureCode;
/// The lifecycle's candidates and the correction a fix applies.
use crate::policy_content::{
    ContentState,
    Correction,
    LifecycleContent,
};
/// The commit path's notes,
///  real-index listing and per-bump correction.
use crate::policy_dependent_version_commit::{
    RealIndexListing,
    added_entries,
    committed_correction,
    left_behind,
    note_of,
    noted,
};
/// What this check answers with,
/// and the step that reads one bump.
use crate::policy_dependent_version_outcomes::{
    candidate_path,
    clean,
    correction_of,
    failed,
    is_listed_change,
    plan_failure,
    planner_candidate,
    policy_finding,
};
/// The finding,
///  note and outcome types of a check.
use crate::policy_engine::{
    PolicyFinding,
    PolicyNote,
    PolicyOutcome,
};
/// The lifecycle points.
use crate::policy_trigger::Trigger;
/// The facts interface that prepares candidates and locates the worktree.
use crate::repository_facts::RepositoryFacts;
/// The repository location the commit path names its Git reads with.
use crate::repository_location::RepositoryLocation;
/// The worktree's top level.
use crate::worktree_identity::worktree_root;
/// Borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;
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
pub fn check_dependent_version<F: RepositoryFacts>(
    check: DependentCheck<'_, '_, F>
) -> PolicyOutcome {
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
    let location: Option<RepositoryLocation> = if trigger == Trigger::DirectFix || plans_commit {
        match facts.location() {
            Ok(found) => Some(found),
            Err(message) => return failed(EngineFailureCode::ContentUnavailable, message),
        }
    } else {
        None
    };
    // The repository Git located once, never the caller's raw options: reading a relative `--git-dir` again from a
    //  discovered directory applies the caller's `-C` a second time (`repository_selection::of_identity`).
    let mut root: Option<PathBuf> = None;
    if let Some(found) = location.as_ref()
        && let Some(top) = worktree_root(&found.identity)
    {
        root = Some(top.to_path_buf());
    }
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
        location.as_ref(),
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
