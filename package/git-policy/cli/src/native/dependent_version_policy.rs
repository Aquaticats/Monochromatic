//! What: The decision of `mono/dependent-version-bump`: when to plan, and the findings a
//!       plan becomes.
//! Why: The plan is a pre-commit step. Another forwarded command such as `git add` could
//!      not apply its patches, so it plans only for `git commit` among forwarded commands,
//!      and for `git cli-git check` and `git cli-git fix`; and only when a workspace manifest
//!      is modified, so ordinary commits read nothing. Each stale dependent is one finding
//!      whose patch rewrites a tracked manifest that may not be a candidate, which adds that
//!      manifest to the commit (`dependent-version-bump-policy.ts:178-235`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await findDependentBumps(context);
//! ```

/// What: `use` brings names from sibling files into this file.
/// Why:  The decision reads the trigger, candidate paths and the plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { planWorkspaceBumps } from './dependent_version_plan.ts';
/// ```
use super::dependent_version_content::{PlanError, TrackedMode, WorkspaceContent};
/// Which candidate paths are workspace manifests.
use super::dependent_version_paths::is_workspace_manifest_path;
/// The plan the findings come from.
use super::dependent_version_plan::{ManifestBump, PlanOutcome, plan_workspace_bumps};
/// The incumbent's message for a version that cannot be bumped.
use super::dependent_version_release::unsupported_message;
/// The lifecycle point the policy runs at.
use super::policy_trigger::Trigger;

/// What: The finding code of a dependent whose version must move with a raised dependency.
/// Why:  Callers and transcripts identify the finding by this stable code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DEPENDENT_VERSION_STALE_CODE = 'dependent-version-stale';
/// ```
pub const DEPENDENT_VERSION_STALE_CODE: &str = "dependent-version-stale";

/// What: The finding code of a dependent whose version cannot be bumped automatically.
/// Why:  It carries no patch; the person bumps that package by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DEPENDENT_VERSION_UNSUPPORTED_CODE = 'dependent-version-unsupported';
/// ```
pub const DEPENDENT_VERSION_UNSUPPORTED_CODE: &str = "dependent-version-unsupported";

/// What: How a candidate path changed against the base.
/// Why:  Only a modified manifest can hold a raised version; a new one has no base version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CandidateChange = 'added' | 'modified' | 'deleted';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateChange {
    /// New at this state.
    Added,
    /// Present at both states with different content or mode.
    Modified,
    /// Removed at this state.
    Deleted,
}

/// What: One candidate path and how it changed.
/// Why:  The decision to plan looks at candidate paths only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Candidate = { path: string; change: CandidateChange };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// Repository-relative path.
    pub path: Vec<u8>,
    /// Its change.
    pub change: CandidateChange,
}

/// What: What the lifecycle tells the policy. `&'request [Candidate]` borrows the
///       candidate list for as long as the request lives.
/// Why:  `forwards_commit` is whether the forwarded subcommand is `commit`; it matters only
///       for the pre-forward trigger.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DependentRequest = { trigger: PolicyTrigger; forwardsCommit: boolean; candidates: Candidate[] };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct DependentRequest<'request> {
    /// The lifecycle point.
    pub trigger: Trigger,
    /// Whether the forwarded command is `git commit`.
    pub forwards_commit: bool,
    /// The candidates of this lifecycle.
    pub candidates: &'request [Candidate],
}

/// What: One finding of this policy. `Option<ManifestBump>` is the patch, or nothing.
/// Why:  The wiring turns `patch` into a full-content patch of the tracked manifest at
///       `path`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DependentFinding = { code: string; message: string; path?: string; patch?: ManifestBump };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependentFinding {
    /// `dependent-version-stale` or `dependent-version-unsupported`.
    pub code: &'static str,
    /// The explanation for the person who ran the command.
    pub message: String,
    /// The manifest the finding is about, when it names one.
    pub path: Option<Vec<u8>>,
    /// The bump that settles it, when it can be automatic.
    pub patch: Option<ManifestBump>,
}

/// What: Whether this lifecycle should plan at all.
/// Why:  A forwarded command other than `commit` cannot apply the patches; a change that
///       modifies no workspace manifest cannot raise a version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shouldPlan(request: DependentRequest): boolean;
/// ```
pub fn should_plan(request: &DependentRequest) -> bool {
    if request.trigger == Trigger::PreForward && !request.forwards_commit {
        return false;
    }
    return request.candidates.iter().any(|candidate: &Candidate| {
        return candidate.change == CandidateChange::Modified
            && is_workspace_manifest_path(&candidate.path);
    });
}

/// What: The stale finding for one planned bump.
/// Why:  The message names the dependent, the raised packages and both versions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${bump.name} reaches a package bumped in this commit (${bumpedNames.join(', ')}); bump it from ${bump.from} to ${bump.to} in the same commit.`
/// ```
fn stale_finding(bump: &ManifestBump, bumped_names: &[String]) -> DependentFinding {
    return DependentFinding {
        code: DEPENDENT_VERSION_STALE_CODE,
        message: format!(
            "{} reaches a package bumped in this commit ({}); bump it from {} to {} in the same commit.",
            bump.planned.name,
            bumped_names.join(", "),
            bump.planned.bump.from,
            bump.planned.bump.to
        ),
        path: Some(bump.path.clone()),
        patch: Some(bump.clone()),
    };
}

/// What: The findings of this policy for one lifecycle point.
/// Why:  A bump of a manifest that is not an ordinary file cannot be patched and is
///       dropped, as in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function findDependentBumps(context: PolicyContext): Promise<PolicyFinding[]>;
/// ```
pub fn find_dependent_bumps(
    request: &DependentRequest,
    content: &mut dyn WorkspaceContent,
) -> Result<Vec<DependentFinding>, PlanError> {
    if !should_plan(request) {
        return Ok(Vec::new());
    }
    match plan_workspace_bumps(content)? {
        PlanOutcome::Unsupported(unsupported) => {
            return Ok(vec![DependentFinding {
                code: DEPENDENT_VERSION_UNSUPPORTED_CODE,
                message: unsupported_message(&unsupported),
                path: None,
                patch: None,
            }]);
        }
        PlanOutcome::Planned(plan) => {
            return Ok(plan
                .bumps
                .iter()
                .filter(|bump: &&ManifestBump| {
                    return matches!(bump.mode, TrackedMode::Regular | TrackedMode::Executable);
                })
                .map(|bump: &ManifestBump| return stale_finding(bump, &plan.bumped_names))
                .collect());
        }
    }
}

/// The trigger rule, the candidate rule and the findings, including the incumbent's policy
/// tests.
#[cfg(test)]
#[path = "dependent_version_policy_tests.rs"]
mod tests;
