//! What:
//!  The three answers every shipped check gives in this folder:
//!  nothing found,
//!  one finding about the command,
//!  and a repository fact that could not be read.
//! Why:
//!  The branch-worktree check,
//!  the linked-worktree check and the nested-repository check
//!  each end with one of these,
//!  so they live below all three
//!  instead of in the file that joins the checks to the engine,
//!  which every check would then use in return.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const factUnavailable = (message: string): PolicyOutcome => ({ kind: 'failed', code: 'content-unavailable', message });
//! const clean = (): PolicyOutcome => ({ kind: 'findings', findings: [] });
//! const rejected = (code: string, message: string): PolicyOutcome => ({ kind: 'findings', findings: [{ code, message }] });
//! ```

/// The code a check reports when a repository fact could not be read.
use crate::diagnostics::EngineFailureCode;
/// The engine's check interface:
///  one finding,
///  and what a check answers with.
use crate::policy_engine::{
    PolicyFinding,
    PolicyOutcome,
};

/// What:
///  The outcome of a check whose repository fact could not be read.
///       `String` is the owned reason the facts provider gave.
/// Why:
///   Every fact these checks ask for is read from the repository,
///  so its absence is
///       `content-unavailable`,
///  never a failure of the policy itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const factUnavailable = (message: string): PolicyOutcome => ({ kind: 'failed', code: 'content-unavailable', message });
/// ```
pub(crate) fn fact_unavailable(message: String) -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code: EngineFailureCode::ContentUnavailable,
        message,
        // `None`: a repository fact is about no single file.
        path: None,
    };
}

/// What:
///  The outcome "nothing found".
///  `Vec::<PolicyFinding>::new()` is an empty owned list.
/// Why:
///   Most checks end this way;
///  one helper keeps them identical.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clean = (): PolicyOutcome => ({ kind: 'findings', findings: [] });
/// ```
pub(crate) fn clean() -> PolicyOutcome {
    return PolicyOutcome::Findings(Vec::<PolicyFinding>::new());
}

/// What:
///  The outcome "exactly one finding about the command".
///  `&'static str` is text
///       baked into the program;
///  `String` is owned text.
/// Why:
///   The four command policies each report at most one finding,
///  without a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rejected = (code: string, message: string): PolicyOutcome => ({ kind: 'findings', findings: [{ code, message }] });
/// ```
pub(crate) fn rejected(
    code: &'static str,
    message: String,
) -> PolicyOutcome {
    // `vec![...]` is a macro building an owned list from literal items.
    return PolicyOutcome::Findings(vec![PolicyFinding {
        code,
        message,
        // `None` is the "absent" case of `Option`.
        path: None,
        location: None,
        fix_available: false,
        commit_correction: None,
    }]);
}
