//! What:
//! Resolving one plan tool:
//! the outcome it produced,
//! the resolution to lock when there is one,
//! whether a named update leaves the tool out,
//! and the tools an npm or PyPI tool needs installed first.
//! Why:
//! A resolver that does not exist yet is not a failure for a tool whose entry is already in the lock and whose
//! resolution-affecting fields did not change,
//! so the entry is kept and reported;
//! and a named update looks up only what it selected.
//! A caller imports from `update` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function resolveOne(ctx: Context, declaration: Declaration, current?: LockedTool): Resolved;
//! function isLeftOut(only: Set<string> | undefined, id: string): boolean;
//! function prerequisites(route: Route): string[];
//! ```

/// Imports a set sorted by value,
/// which holds the names a command line selected.
use std::collections::BTreeSet;

/// Imports the bundle of seams every resolver receives.
use super::super::context::Context;
/// Imports the request a tool is resolved from,
/// and the row the lock already holds.
use super::super::model::{
    Declaration,
    LockedTool,
    Route,
};
/// Imports what a resolution answers with,
/// and the resolution itself.
use super::super::update_resolve::{
    Resolution,
    ResolveError,
    resolve_tool,
};

/// Imports one line of the report,
/// and what happened to the tool.
use super::what_an_update_did::{
    Outcome,
    OutcomeKind,
};
/// An outcome together with the resolution it produced (kept for the lock the update builds).
pub(super) struct Resolved {
    /// What happened.
    pub(super) outcome: Outcome,
    /// What to lock,
    /// when there is something.
    pub(super) resolution: Option<Resolution>,
}

/// What:
/// Resolves one plan tool into an outcome (and the resolution to lock when there is one).
/// Why:
/// A resolver that does not exist yet is not a failure for a tool whose entry is already in the lock and whose
///           resolution-affecting fields did not change:
/// the entry is kept and reported.
/// For a new tool or a changed
///           pin it is a failure,
/// because there is nothing to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveOne(ctx: Context, declaration: Declaration, current?: LockedTool): Resolved
/// ```
pub(super) fn resolve_one(
    ctx: &Context,
    declaration: &Declaration,
    current: Option<&LockedTool>,
) -> Resolved {
    let id: String = declaration.id.clone();
    match resolve_tool(ctx, declaration, current) {
        Ok(resolution) => {
            let kind: OutcomeKind = match current {
                None => OutcomeKind::Added {
                    version: resolution.version.clone(),
                },
                Some(locked)
                    if locked.version == resolution.version
                        && locked.artifacts == resolution.artifacts =>
                {
                    if declaration.request.is_exact() {
                        OutcomeKind::Pinned
                    } else {
                        OutcomeKind::Unchanged
                    }
                }
                Some(locked) => OutcomeKind::Updated {
                    old_version: locked.version.clone(),
                    new_version: resolution.version.clone(),
                },
            };
            return Resolved {
                outcome: Outcome {
                    id,
                    kind,
                },
                resolution: Some(resolution),
            };
        }
        Err(ResolveError::Failed(reason)) => {
            return Resolved {
                outcome: Outcome {
                    id,
                    kind: OutcomeKind::Failed {
                        reason,
                    },
                },
                resolution: None,
            };
        }
    }
}

/// Whether a named update leaves this tool out:
/// names were given and the id is not among the selected ones.
pub(super) fn is_left_out(
    only: Option<&BTreeSet<String>>,
    id: &str,
) -> bool {
    return match only {
        Some(selected) => !selected.contains(id),
        None => false,
    };
}

/// The tools an npm or PyPI tool needs installed first,
/// by route.
pub(super) fn prerequisites(route: Route) -> &'static [&'static str] {
    return match route {
        Route::MpmPnpm => &["node", "pnpm"],
        Route::MpmUvx => &["uv"],
        _ => &[],
    };
}
