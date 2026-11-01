//! What:
//!  The optional `mono/forbidden-root-context` policy:
//!  a `CONTEXT.md` file at the
//!       repository's top level must not enter the index or be checked.
//! Why:
//!  This repository keeps no context files;
//!  agents read the source on every probe
//!      (`doc/agent/domain.md`).
//!  The rule and its finding are the installed wrapper's
//!      (`package/git-policy/repository/src/index.ts`):
//!  only the top-level pathname
//!      counts,
//!  and removing the file is always allowed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // if (candidates.some(c => c.path === 'CONTEXT.md' && c.change !== 'deleted')) return [finding];
//! ```

/// Import the candidate change kinds.
use super::candidate_record::CandidateChange;
/// Import the version type.
use super::candidate_version::CandidateVersion;
/// Import the event form of a pathname.
use super::event_path::EventPath;
/// Import the lifecycle's candidates.
use super::policy_content::{ContentState, LifecycleContent};
/// Import the finding and outcome types of a check.
use super::policy_engine::{PolicyFinding, PolicyOutcome};
/// Import the facts interface that prepares candidates.
use super::repository_facts::RepositoryFacts;
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The one pathname the policy forbids,
///  relative to the repository's top level.
pub const ROOT_CONTEXT_PATH: &str = "CONTEXT.md";

/// The policy-local code of the finding.
pub const ROOT_CONTEXT_CODE: &str = "root-context-forbidden";

/// The message of the finding,
///  as the installed wrapper words it.
pub const ROOT_CONTEXT_MESSAGE: &str = "Root CONTEXT.md is forbidden; read source code directly.";

/// What:
///  Check the lifecycle's candidates for a top-level `CONTEXT.md` that is not a deletion.
///       `<F: RepositoryFacts>` accepts any facts provider.
/// Why:
///   Only pathnames and change kinds are read,
///  never content;
///  at most one finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkRootContext(content, lifecycle, facts): Promise<PolicyOutcome>;
/// ```
pub fn check_root_context<F: RepositoryFacts>(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    facts: &mut F,
) -> PolicyOutcome {
    let version: Rc<CandidateVersion> = match content.version(lifecycle, facts) {
        Ok(Some(found)) => found,
        // `Vec::new()` is the empty list: a lifecycle without candidates has nothing to report.
        Ok(None) => return PolicyOutcome::Findings(Vec::new()),
        Err(outcome) => return outcome,
    };
    // `if let Some(candidate) = ...` runs only when the version lists the pathname.
    if let Some(candidate) = version.candidate_at_path(ROOT_CONTEXT_PATH.as_bytes())
        && candidate.change != CandidateChange::Deleted
    {
        // `vec![...]` builds the one-finding list.
        return PolicyOutcome::Findings(vec![PolicyFinding {
            code: ROOT_CONTEXT_CODE,
            message: String::from(ROOT_CONTEXT_MESSAGE),
            // The fixed name is UTF-8, so it is its own exact form.
            path: Some(EventPath::from_git_bytes(ROOT_CONTEXT_PATH.as_bytes())),
            location: None,
            fix_available: false,
        }]);
    }
    return PolicyOutcome::Findings(Vec::new());
}

/// Check controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_root_context_tests.rs"]
mod tests;
