//! What:
//! The update itself:
//! every plan tool looked up once,
//! one tool at a time,
//! the lock written when anything moved,
//! and the auto-sync that installs a missing tool before its lookup is tried a second time.
//! Why:
//! A plan entry the lock cannot hold that the selected names leave out is refused before any lookup,
//! so a named update never writes a lock it did not fully resolve;
//! and a second failure stays a failure naming the tool.
//! A caller imports from `update` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function updateLoaded(loaded, scope, env, runner, http, host, check, only?: Set<string>): UpdateReport;
//! function retryAfterSync(ctx, plan, results, synced): void;
//! ```

/// Imports a set sorted by value,
/// which holds the tool ids a command line selected.
use std::collections::BTreeSet;

/// Imports the plan-versus-lock comparison,
/// its verdict and its refusal.
use super::super::agree::{
    Disagreement,
    compare,
    refusal,
};
/// Imports the bundle of seams every resolver receives,
/// and the machine it pretends to be.
use super::super::context::{
    Context,
    Host,
};
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports `HttpGet` from `super::super::http`.
use super::super::http::HttpGet;
/// Imports what this file uses from `super::super::load`.
use super::super::load::Loaded;
/// Imports the lock text this file renders and writes.
use super::super::lock::{
    Lock,
    render_lock,
};
/// Imports what this file uses from `super::super::model`.
use super::super::model::{
    Declaration,
    LockedTool,
    Route,
    expand_version,
};
/// Imports the probe that decides whether a failed tool is missing from the machine.
use super::super::observe::observe_tool;
/// Imports `publish_lock` from `super::super::publish`.
use super::super::publish::publish_lock;
/// Imports `Runner` from `super::super::runner`.
use super::super::runner::Runner;
/// Imports what this file uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports `sync_tools` from `super::super::sync`.
use super::super::sync::sync_tools;
/// Imports what this file uses from `super::super::update_resolve`.
use super::super::update_resolve::keep;
/// Imports the refusal for a plan entry the selected names leave out.
use super::super::update_select::{
    left_out_refusal,
    unnamed_needing_resolution,
};
/// Imports the tool lookup.
use super::super::util::find_tool;

/// Imports the resolution of one tool,
/// and whether the names left it out.
use super::resolving_one_tool::{
    Resolved,
    is_left_out,
    prerequisites,
    resolve_one,
};
/// Imports the report this file fills in.
use super::what_an_update_did::{
    Outcome,
    OutcomeKind,
    UpdateReport,
};
/// What:
/// Runs `update` over files `load` already read,
/// for the exact tool ids in `only` (every tool when `only` is `None`).
/// Why:
/// `install` resolves the new or re-edited plan entries it finds in the files it just read,
/// with the revision check of the publish covering the gap between that read and the write,
/// and `run_update_for` selects by pattern first.
/// A plan entry the lock cannot hold that `only` leaves out is refused before any lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function updateLoaded(loaded, scope, env, runner, http, host, check, only?: Set<string>): UpdateReport
/// ```
pub fn update_loaded(
    loaded: &Loaded,
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
    check: bool,
    only: Option<&BTreeSet<String>>,
) -> Result<UpdateReport, ToolchainError> {
    let mut old_tools: Vec<LockedTool> = Vec::new();
    if let Some(parsed) = &loaded.lock {
        let verdict = compare(&loaded.plan, parsed);
        if verdict.digest_problem.is_some() {
            return Err(refusal(&verdict));
        }
        old_tools = parsed.lock.tools.clone();
    }
    if let Some(selected) = only {
        let mut disagreement: Option<Disagreement> = None;
        if let Some(parsed) = &loaded.lock {
            disagreement = Some(compare(&loaded.plan, parsed));
        }
        let left_out: Vec<String> =
            unnamed_needing_resolution(&loaded.plan, disagreement.as_ref(), selected);
        if !left_out.is_empty() {
            return Err(left_out_refusal(&left_out));
        }
    }
    let context: Context = Context {
        scope,
        env,
        runner,
        http,
        host,
        tools: &old_tools,
    };
    let mut results: Vec<Resolved> = Vec::new();
    for declaration in &loaded.plan.tools {
        let current: Option<&LockedTool> = find_tool(&old_tools, &declaration.id);
        match current {
            Some(locked) if is_left_out(only, &declaration.id) => {
                results.push(Resolved {
                    outcome: Outcome {
                        id: declaration.id.clone(),
                        kind: OutcomeKind::Kept,
                    },
                    resolution: Some(keep(locked)),
                });
            }
            _ => results.push(resolve_one(&context, declaration, current)),
        }
    }
    let mut synced: Vec<String> = Vec::new();
    if !check {
        retry_after_sync(&context, &loaded.plan.tools, &mut results, &mut synced);
    }
    let mut outcomes: Vec<Outcome> = Vec::new();
    for result in &results {
        outcomes.push(result.outcome.clone());
    }
    let mut report: UpdateReport = UpdateReport {
        outcomes,
        wrote: false,
        synced,
    };
    if report.has_failures() || check {
        return Ok(report);
    }
    let mut tools: Vec<LockedTool> = Vec::new();
    for (declaration, result) in loaded.plan.tools.iter().zip(results.iter()) {
        let Some(resolution) = &result.resolution else {
            return Err(ToolchainError::failure(format!(
                "internal error: {} has no resolution but no failure",
                declaration.id
            )));
        };
        tools.push(LockedTool {
            declaration: expand_version(declaration, &resolution.version),
            version: resolution.version.clone(),
            artifacts: resolution.artifacts.clone(),
        });
    }
    let text: String = render_lock(&Lock {
        tools,
    });
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
fn retry_after_sync(
    context: &Context,
    plan: &[Declaration],
    results: &mut [Resolved],
    synced: &mut Vec<String>,
) {
    let mut candidates: Vec<usize> = Vec::new();
    for (index, declaration) in plan.iter().enumerate() {
        let failed: bool = matches!(results[index].outcome.kind, OutcomeKind::Failed { .. });
        let manager_route: bool = matches!(declaration.route, Route::MpmPnpm | Route::MpmUvx);
        let Some(locked) = find_tool(context.tools, &declaration.id) else {
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
            if !ids.iter().any(|held| return held == *prerequisite) {
                ids.push((*prerequisite).to_string());
            }
        }
        ids.push(plan[*index].id.clone());
    }
    let report = sync_tools(context, Some(&ids));
    synced.extend(report.installed);
    for index in candidates {
        let first_reason: String = match &results[index].outcome.kind {
            OutcomeKind::Failed {
                reason,
            } => reason.clone(),
            _ => String::new(),
        };
        let current: Option<&LockedTool> = find_tool(context.tools, &plan[index].id);
        let mut second: Resolved = resolve_one(context, &plan[index], current);
        if let OutcomeKind::Failed {
            reason,
        } = &second.outcome.kind
        {
            second.outcome.kind = OutcomeKind::Failed {
                reason: format!(
                    "{} (looked up twice: before and after installing the missing tool; first failure: {first_reason})",
                    reason
                ),
            };
        }
        results[index] = second;
    }
}
