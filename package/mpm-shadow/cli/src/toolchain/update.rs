//! What:
//! `update` and `update --check`:
//! resolve every tool of the plan to a version and artifacts,
//! then write the
//!           whole lock (or,
//! with `--check`,
//! only report what would move).
//! Why:
//! `update` is the one command that writes resolved values.
//! It writes only through the revision-checked
//!           publish,
//! never installs anything with `--check`,
//! and,
//! when an npm or PyPI lookup fails for a tool that is
//!           locked but missing from the machine,
//! syncs just the missing tools and retries each lookup once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update.ts
//! export function runUpdate(scope, env, runner, http, host, check: boolean): UpdateReport
//! export function formatUpdateReport(report: UpdateReport): string
//! ```

/// Imports what this module uses from `super::agree`.
use super::agree::{compare, refusal};
/// Imports what this module uses from `super::context`.
use super::context::{Context, Host};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `HttpGet` from `super::http`.
use super::http::HttpGet;
/// Imports what this module uses from `super::load`.
use super::load::{Loaded, load};
/// Imports what this module uses from `super::lock`.
use super::lock::{Lock, render_lock};
/// Imports what this module uses from `super::model`.
use super::model::{Declaration, LockedTool, Route, expand_version};
/// Imports `observe_tool` from `super::observe`.
use super::observe::observe_tool;
/// Imports `publish_lock` from `super::publish`.
use super::publish::publish_lock;
/// Imports `Runner` from `super::runner`.
use super::runner::Runner;
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope};
/// Imports `sync_tools` from `super::sync`.
use super::sync::sync_tools;
/// Imports what this module uses from `super::update_resolve`.
use super::update_resolve::{Resolution, ResolveError, resolve_tool};

/// What happened to one tool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutcomeKind {
    /// The lookup found the locked version and artifacts again.
    Unchanged,
    /// The version or artifacts moved.
    Updated {
        /// The locked version before.
        old_version: String,
        /// The version found.
        new_version: String,
    },
    /// The tool was not in the lock;
    /// this is its first resolution.
    Added {
        /// The version found.
        version: String,
    },
    /// The tool is pinned and the lock already holds that pin.
    Pinned,
    /// No resolver exists for the tool yet;
    /// its lock entry is kept as it is.
    Skipped {
        /// Why,
        /// naming the tool and route.
        reason: String,
    },
    /// The lookup failed;
    /// nothing is written.
    Failed {
        /// Why,
        /// naming the URL or the field.
        reason: String,
    },
}

/// One line of the report:
/// a tool and what happened.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outcome {
    /// Tool id.
    pub id: String,
    /// What happened.
    pub kind: OutcomeKind,
}

/// What `update` did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateReport {
    /// One outcome per plan tool,
    /// in plan order.
    pub outcomes: Vec<Outcome>,
    /// Whether `toolchain.lock.json` was written.
    pub wrote: bool,
    /// Tool ids the auto-sync installed before a retry.
    pub synced: Vec<String>,
}

/// The methods of `UpdateReport`.
impl UpdateReport {
    /// Whether any lookup failed.
    pub fn has_failures(&self) -> bool {
        return self.outcomes.iter().any(|outcome| return matches!(outcome.kind, OutcomeKind::Failed { .. }));
    }

    /// Whether any tool moved or was added (the condition `update --check` exits 1 for).
    pub fn has_movement(&self) -> bool {
        return self
            .outcomes
            .iter()
            .any(|outcome| return matches!(outcome.kind, OutcomeKind::Updated { .. } | OutcomeKind::Added { .. }));
    }
}

/// An outcome together with the resolution it produced (kept for the lock the update builds).
struct Resolved {
    /// What happened.
    outcome: Outcome,
    /// What to lock,
    /// when there is something.
    resolution: Option<Resolution>,
}

/// Whether the plan's declaration and the lock's differ in a field that decides what is resolved.
fn resolution_changed(declaration: &Declaration, locked: &LockedTool) -> bool {
    let expanded: Declaration = expand_version(declaration, &locked.version);
    return expanded.request != locked.declaration.request
        || expanded.route != locked.declaration.route
        || expanded.manager != locked.declaration.manager
        || expanded.package_id != locked.declaration.package_id
        || expanded.os != locked.declaration.os
        || expanded.options.asset_pattern != locked.declaration.options.asset_pattern
        || expanded.options.cargo_git_url != locked.declaration.options.cargo_git_url
        || expanded.options.bootstrap != locked.declaration.options.bootstrap;
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
fn resolve_one(ctx: &Context, declaration: &Declaration, current: Option<&LockedTool>) -> Resolved {
    let id: String = declaration.id.clone();
    match resolve_tool(ctx, declaration, current) {
        Ok(resolution) => {
            let kind: OutcomeKind = match current {
                None => OutcomeKind::Added { version: resolution.version.clone() },
                Some(locked) if locked.version == resolution.version && locked.artifacts == resolution.artifacts => {
                    if matches!(declaration.request, super::model::Request::Pinned(_)) { OutcomeKind::Pinned } else { OutcomeKind::Unchanged }
                }
                Some(locked) => OutcomeKind::Updated { old_version: locked.version.clone(), new_version: resolution.version.clone() },
            };
            return Resolved { outcome: Outcome { id, kind }, resolution: Some(resolution) };
        }
        Err(ResolveError::NotImplemented(reason)) => {
            if let Some(locked) = current
                && !resolution_changed(declaration, locked)
            {
                let kept: Resolution = Resolution { version: locked.version.clone(), artifacts: locked.artifacts.clone() };
                return Resolved { outcome: Outcome { id, kind: OutcomeKind::Skipped { reason } }, resolution: Some(kept) };
            }
            return Resolved { outcome: Outcome { id, kind: OutcomeKind::Failed { reason } }, resolution: None };
        }
        Err(ResolveError::Failed(reason)) => {
            return Resolved { outcome: Outcome { id, kind: OutcomeKind::Failed { reason } }, resolution: None };
        }
    }
}

/// The tools an npm or PyPI tool needs installed first,
/// by route.
fn prerequisites(route: Route) -> &'static [&'static str] {
    return match route {
        Route::MpmPnpm => &["node", "pnpm"],
        Route::MpmUvx => &["uv"],
        _ => &[],
    };
}

/// What:
/// Runs `update` (or `update --check`) in a scope.
/// Why:
/// With `check` set nothing is installed and nothing is written.
/// Otherwise a failed npm or PyPI lookup for a
///           tool that is locked but not installed triggers one sync of just those tools (and what they need),
/// then one
///           retry of each such lookup;
/// a second failure stays a failure naming the tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runUpdate(scope, env, runner, http, host, check): UpdateReport
/// ```
pub fn run_update(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
    check: bool,
) -> Result<UpdateReport, ToolchainError> {
    let loaded: Loaded = load(scope)?;
    let mut old_tools: Vec<LockedTool> = Vec::new();
    if let Some(parsed) = &loaded.lock {
        let verdict = compare(&loaded.plan, parsed);
        if verdict.digest_problem.is_some() {
            return Err(refusal(&verdict));
        }
        old_tools = parsed.lock.tools.clone();
    }
    let context: Context = Context { scope, env, runner, http, host, tools: &old_tools };
    let mut results: Vec<Resolved> = Vec::new();
    for declaration in &loaded.plan.tools {
        let current: Option<&LockedTool> = old_tools.iter().find(|tool| return tool.declaration.id == declaration.id);
        results.push(resolve_one(&context, declaration, current));
    }
    let mut synced: Vec<String> = Vec::new();
    if !check {
        retry_after_sync(&context, &loaded.plan.tools, &mut results, &mut synced);
    }
    let outcomes: Vec<Outcome> = results.iter().map(|result| return result.outcome.clone()).collect();
    let mut report: UpdateReport = UpdateReport { outcomes, wrote: false, synced };
    if report.has_failures() || check {
        return Ok(report);
    }
    let mut tools: Vec<LockedTool> = Vec::new();
    for (declaration, result) in loaded.plan.tools.iter().zip(results.iter()) {
        let Some(resolution) = &result.resolution else {
            return Err(ToolchainError::failure(format!("internal error: {} has no resolution but no failure", declaration.id)));
        };
        tools.push(LockedTool {
            declaration: expand_version(declaration, &resolution.version),
            version: resolution.version.clone(),
            artifacts: resolution.artifacts.clone(),
        });
    }
    let text: String = render_lock(&Lock { tools });
    if loaded.lock_text.as_deref() != Some(text.as_str()) {
        publish_lock(scope, &loaded.revisions, &text)?;
        report.wrote = true;
    }
    return Ok(report);
}

/// What:
/// The auto-sync-and-retry rule:
/// for each failed npm or PyPI lookup of a tool that is in the lock and not
///           installed,
/// install just those tools (with what they need),
/// then look each one up again once.
/// Why:
/// Written to the user's answer "Require sync first,
/// and auto-sync on such failures and retry."
/// A tool that is
///           installed,
/// or not in the lock,
/// is never synced,
/// and a second failure is final.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retryAfterSync(ctx, plan, results, synced): void
/// ```
fn retry_after_sync(context: &Context, plan: &[Declaration], results: &mut [Resolved], synced: &mut Vec<String>) {
    let mut candidates: Vec<usize> = Vec::new();
    for (index, declaration) in plan.iter().enumerate() {
        let failed: bool = matches!(results[index].outcome.kind, OutcomeKind::Failed { .. });
        let manager_route: bool = matches!(declaration.route, Route::MpmPnpm | Route::MpmUvx);
        let Some(locked) = context.tools.iter().find(|tool| return tool.declaration.id == declaration.id) else {
            continue;
        };
        if failed && manager_route && observe_tool(context, locked).is_none() {
            candidates.push(index);
        }
    }
    if candidates.is_empty() {
        return;
    }
    let mut ids: Vec<String> = Vec::new();
    for index in &candidates {
        for prerequisite in prerequisites(plan[*index].route) {
            if !ids.iter().any(|id| return id == prerequisite) {
                ids.push((*prerequisite).to_string());
            }
        }
        ids.push(plan[*index].id.clone());
    }
    let report = sync_tools(context, Some(&ids));
    synced.extend(report.installed);
    for index in candidates {
        let first_reason: String = match &results[index].outcome.kind {
            OutcomeKind::Failed { reason } => reason.clone(),
            _ => String::new(),
        };
        let current: Option<&LockedTool> = context.tools.iter().find(|tool| return tool.declaration.id == plan[index].id);
        let mut second: Resolved = resolve_one(context, &plan[index], current);
        if let OutcomeKind::Failed { reason } = &second.outcome.kind {
            second.outcome.kind = OutcomeKind::Failed {
                reason: format!("{} (looked up twice: before and after installing the missing tool; first failure: {first_reason})", reason),
            };
        }
        results[index] = second;
    }
}

/// The text `update` prints:
/// one line per tool and a summary.
pub fn format_update_report(report: &UpdateReport, check: bool) -> String {
    let mut text: String = String::new();
    for id in &report.synced {
        text.push_str(&format!("synced    {id}  (was missing from the machine)\n"));
    }
    for outcome in &report.outcomes {
        let line: String = match &outcome.kind {
            OutcomeKind::Unchanged => format!("unchanged {}", outcome.id),
            OutcomeKind::Pinned => format!("pinned    {}", outcome.id),
            OutcomeKind::Updated { old_version, new_version } => format!("updated   {}  {old_version} -> {new_version}", outcome.id),
            OutcomeKind::Added { version } => format!("added     {}  {version}", outcome.id),
            OutcomeKind::Skipped { reason } => format!("skipped   {}  {reason}", outcome.id),
            OutcomeKind::Failed { reason } => format!("failed    {}  {reason}", outcome.id),
        };
        text.push_str(&line);
        text.push('\n');
    }
    let count = |predicate: fn(&OutcomeKind) -> bool| return report.outcomes.iter().filter(|outcome| return predicate(&outcome.kind)).count();
    let moved: usize = count(|kind| return matches!(kind, OutcomeKind::Updated { .. } | OutcomeKind::Added { .. }));
    let skipped: usize = count(|kind| return matches!(kind, OutcomeKind::Skipped { .. }));
    let failed: usize = count(|kind| return matches!(kind, OutcomeKind::Failed { .. }));
    if failed > 0 {
        text.push_str(&format!("toolchain update: {failed} lookups failed; toolchain.lock.json was not written\n"));
    } else if check {
        text.push_str(&format!("toolchain update --check: {moved} tools would move, {skipped} skipped; nothing was written or installed\n"));
    } else if report.wrote {
        text.push_str(&format!("toolchain update: wrote toolchain.lock.json ({moved} tools moved, {skipped} skipped)\n"));
    } else {
        text.push_str(&format!("toolchain update: toolchain.lock.json is already current ({skipped} skipped)\n"));
    }
    return text;
}
