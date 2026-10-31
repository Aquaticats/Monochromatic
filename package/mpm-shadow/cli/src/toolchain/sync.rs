//! What:
//! `sync`:
//! install every locked tool that is not already at its locked version,
//! in dependency order,
//! then make
//!           the postinstall links,
//! rebuild the exposure directory and run the postinstall commands.
//! Why:
//! This is the installer.
//! It reads only the lock (which `sync` first proves agrees with the plan),
//! probes
//!           before installing so a second run changes nothing,
//! and keeps going after one tool fails so the report
//!           names every failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync.ts
//! export function syncOrder(tools: LockedTool[]): LockedTool[]
//! export function syncTools(ctx: Context, only?: string[]): SyncReport
//! export function runSync(scope, env, runner, http, host): SyncReport  // throws when plan and lock disagree
//! ```

/// Imports what this module uses from `super::cargo_installs`.
use std::collections::BTreeSet;

use super::agree::{Disagreement, compare, refusal};
use super::cargo_installs::{is_repository_published, older_lock_warnings, point_current_at_newest};
/// Imports what this module uses from `super::context`.
use super::context::{Context, Host};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::exposure`.
use super::exposure::{rebuild_exposure, run_postinstall_commands, run_postinstall_links, unresolvable_bins};
/// Imports `HttpGet` from `super::http`.
use super::http::HttpGet;
/// Imports `install_tool` from `super::install`.
use super::install::install_tool;
/// Imports `applies_to_family` from `super::layout`.
use super::layout::applies_to_family;
/// Imports `load_agreeing` from `super::load`.
use super::load::{Loaded, load, load_agreeing};
/// Imports `Lock` from `super::lock`.
use super::lock::Lock;
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Route};
/// Imports what this module uses from `super::observe`.
use super::observe::{observe_tool, package_name, satisfies_lock};
/// Imports `version_matches` from `super::parse_output`.
use super::parse_output::version_matches;
/// Imports `Plan` from `super::plan`.
use super::plan::Plan;
/// Imports `Runner` from `super::runner`.
use super::runner::Runner;
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope};
/// Imports the shared named helpers.
use super::update::{OutcomeKind, update_loaded};
use super::util::find_tool;

/// The tools installed first,
/// in this order,
/// because the others need them:
/// the `bin` downloader,
/// `uv`,
/// `mpm`,
/// the Rust
/// toolchain,
/// `cargo-binstall`,
/// Go,
/// Node and `pnpm`.
pub const STAGING_IDS: [&str; 9] = ["bin", "uv", "mpm", "rustup", "rust", "cargo-binstall", "go", "node", "pnpm"];

/// What:
/// The tools in install order:
/// the staging tools that exist first (in staging order),
/// then the rest in lock
///           order.
/// Why:
/// Installing `npm:wrangler` needs `pnpm`,
/// which needs `node`;
/// the order is a data table,
/// not a dependency
///           solver,
/// because the repository's dependency chain is short and fixed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncOrder(tools: LockedTool[]): LockedTool[]
/// ```
pub fn sync_order(tools: &[LockedTool]) -> Vec<&LockedTool> {
    let mut ordered: Vec<&LockedTool> = Vec::new();
    for staging_id in STAGING_IDS {
        if let Some(found) = find_tool(tools, staging_id) {
            ordered.push(found);
        }
    }
    for tool in tools {
        if !STAGING_IDS.contains(&tool.declaration.id.as_str()) {
            ordered.push(tool);
        }
    }
    return ordered;
}

/// What `sync` did:
/// tools already at their version,
/// tools installed,
/// tools that failed (with why),
/// declared bins still
/// unresolved afterwards,
/// and warnings (a shared install being replaced).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SyncReport {
    /// Tool ids already at the locked version.
    pub unchanged: Vec<String>,
    /// Tool ids installed by this run.
    pub installed: Vec<String>,
    /// Tool id and failure text.
    pub failed: Vec<(String, String)>,
    /// Declared bin names without a resolving link.
    pub missing_bins: Vec<String>,
    /// Warnings printed before the install that caused them.
    pub warnings: Vec<String>,
    /// The tools `sync` resolved and locked itself before installing,
    /// as tool id and locked version:
    /// the plan entries that were new or whose resolution-affecting fields changed.
    pub resolved: Vec<(String, String)>,
}

/// What:
/// The warning for an install that would replace a different version,
/// or `None` when the tool is absent or
///           already matches.
/// Why:
/// Several worktrees share one install directory per tool version,
/// and a manager-level install (`cargo
///           binstall`,
/// `uv tool install`) replaces the one version everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replacementWarning(id: string, observed: string | undefined, locked: string): string | undefined
/// ```
pub fn replacement_warning(id: &str, observed: &Option<String>, locked: &str) -> Option<String> {
    let found: &str = observed.as_deref()?;
    if version_matches(found, locked) {
        return None;
    }
    return Some(format!(
        "{id} is installed at {found} but this tree's lock pins {locked}; installing {locked} replaces it for every tree \
         that shares this install, and a tree whose lock pins the other version replaces it back on its next sync"
    ));
}

/// What:
/// Syncs the tools of `ctx.tools` (all of them,
/// or only those whose ids are in `only`),
/// in install order.
/// Why:
/// `update` calls it with `only` set to install just the npm and PyPI tools its lookups need;
/// `sync` calls it
///           with `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncTools(ctx: Context, only?: string[]): SyncReport
/// ```
pub fn sync_tools(ctx: &Context, only: Option<&[String]>) -> SyncReport {
    let mut report: SyncReport = SyncReport::default();
    let mut entries: Vec<&LockedTool> = Vec::new();
    for tool in sync_order(ctx.tools) {
        if !applies_to_family(&tool.declaration.os, ctx.host.family) {
            continue;
        }
        let selected: bool = match only {
            Some(ids) => ids.contains(&tool.declaration.id),
            None => true,
        };
        if selected {
            entries.push(tool);
        }
    }
    for tool in &entries {
        let observed: Option<String> = observe_tool(ctx, tool);
        // A cargo-binstall install lives in a version directory of its own, so nothing shared is replaced.
        if tool.declaration.route != Route::CargoBinstall
            && let Some(warning) = replacement_warning(&tool.declaration.id, &observed, &tool.version)
        {
            report.warnings.push(warning);
        }
        let current: bool = match observed.as_deref() {
            Some(found) => satisfies_lock(tool, found),
            None => false,
        };
        if current {
            if is_repository_published(tool)
                && let Err(error) = point_current_at_newest(&ctx.scope.toolchain_root, &package_name(tool))
            {
                report.failed.push((tool.declaration.id.clone(), error.message));
            }
            report.unchanged.push(tool.declaration.id.clone());
            continue;
        }
        match install_tool(ctx, tool) {
            Ok(()) => report.installed.push(tool.declaration.id.clone()),
            Err(error) => report.failed.push((tool.declaration.id.clone(), error.message)),
        }
    }
    if let Err(error) = run_postinstall_links(ctx, &entries) {
        report.failed.push(("postinstall links".to_string(), error.message));
    }
    if let Err(error) = rebuild_exposure(ctx, &entries) {
        report.failed.push(("exposure directory".to_string(), error.message));
    }
    let commands = run_postinstall_commands(ctx, &entries);
    report.failed.extend(commands.failed);
    report.missing_bins = unresolvable_bins(ctx.scope, &entries);
    report.warnings.extend(older_lock_warnings(ctx.scope, ctx.tools, ctx.host.family));
    return report;
}

/// What:
/// The reason a frozen `sync` did not resolve a plan that is ahead of the lock.
/// Why:
/// The refusal tells the person which switch or environment stopped `sync` from locking the entries itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrozenReason {
    /// `--frozen-lock` was given.
    Flag,
    /// A CI environment was detected (the `ci-info` rules pnpm uses).
    Ci,
}

/// What:
/// Whether `sync` may resolve plan entries the lock lacks.
/// Why:
/// It mirrors `pnpm install`:
/// resolve and write the lockfile entries a changed manifest needs when not frozen,
/// refuse when frozen (the default in CI).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockPolicy = { kind: 'resolve' } | { kind: 'frozen'; reason: 'flag' | 'ci' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockPolicy {
    /// Resolve new and re-edited entries alone, write only their rows, then install.
    Resolve,
    /// Refuse a plan that is ahead of the lock and name `mpm-shadow update <ids>`.
    Frozen(FrozenReason),
}

/// The refusal for a frozen `sync` whose plan is ahead of the lock:
/// the usual disagreement message plus why `sync` did not resolve.
fn frozen_refusal(verdict: &Disagreement, reason: FrozenReason) -> ToolchainError {
    let base: ToolchainError = refusal(verdict);
    let why: &str = match reason {
        FrozenReason::Flag => "`--frozen-lock` was given",
        FrozenReason::Ci => {
            "a CI environment was detected (the CI variable or another marker; the rules of pnpm and ci-info), \
             where the lock is frozen; `CI=false` or `--no-frozen-lock` turns that off"
        }
    };
    return ToolchainError::drift(format!(
        "{}\nmpm-shadow sync did not resolve them itself because {why}; resolve and commit the lock, then sync again",
        base.message
    ));
}

/// What:
/// Resolves the plan entries the lock lacks or holds for an older request,
/// and only those,
/// and writes only their rows.
/// Why:
/// This is `mpm-shadow update <those ids>` done by `sync`:
/// no other row moves,
/// even when the registries have moved on.
/// A lookup that fails stops the sync before anything is installed,
/// because a lock that does not describe the plan would install the wrong tools.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveBeforeInstalling(loaded, ids, scope, env, runner, http, host): [string, string][]
/// ```
fn resolve_before_installing(
    loaded: &Loaded,
    ids: &[String],
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
) -> Result<Vec<(String, String)>, ToolchainError> {
    let only: BTreeSet<String> = ids.iter().cloned().collect();
    let report = update_loaded(loaded, scope, env, runner, http, host, false, Some(&only))?;
    if report.has_failures() {
        let mut message: String = String::from(
            "mpm-shadow sync could not resolve the plan entries the lock lacks, so it wrote nothing and installed nothing:\n",
        );
        for outcome in &report.outcomes {
            if let OutcomeKind::Failed { reason } = &outcome.kind {
                message.push_str(&format!("  {}: {reason}\n", outcome.id));
            }
        }
        message.push_str("fix the lookup and run `mpm-shadow sync` again, or run `mpm-shadow update <id>` to see it on its own");
        return Err(ToolchainError::failure(message));
    }
    let mut resolved: Vec<(String, String)> = Vec::new();
    for outcome in &report.outcomes {
        match &outcome.kind {
            OutcomeKind::Added { version } => resolved.push((outcome.id.clone(), version.clone())),
            OutcomeKind::Updated { new_version, .. } => resolved.push((outcome.id.clone(), new_version.clone())),
            _ => {}
        }
    }
    return Ok(resolved);
}

/// What:
/// Runs `sync` under a lock policy.
/// Why:
/// A plan that disagrees with the lock only in what `lock` fixes offline,
/// or in the digest,
/// is refused as before.
/// A plan that is ahead of the lock (a new entry,
/// or a resolution-affecting edit) is refused naming `mpm-shadow update <ids>` when frozen,
/// and otherwise those entries are resolved alone and written to the lock,
/// as `pnpm install` does after a dependency was added,
/// before the install starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runSyncWith(scope, env, runner, http, host, policy: LockPolicy): SyncReport
/// ```
pub fn run_sync_with(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
    policy: LockPolicy,
) -> Result<SyncReport, ToolchainError> {
    let loaded: Loaded = load(scope)?;
    let mut resolved: Vec<(String, String)> = Vec::new();
    if let Some(parsed) = &loaded.lock {
        let verdict: Disagreement = compare(&loaded.plan, parsed);
        if verdict.digest_problem.is_none() && verdict.needs_update() {
            match policy {
                LockPolicy::Frozen(reason) => return Err(frozen_refusal(&verdict, reason)),
                LockPolicy::Resolve => {
                    resolved = resolve_before_installing(&loaded, &verdict.ids_needing_update(), scope, env, runner, http, host)?;
                }
            }
        }
    }
    let (_plan, lock): (Plan, Lock) = load_agreeing(scope)?;
    let context: Context = Context { scope, env, runner, http, host, tools: &lock.tools };
    let mut report: SyncReport = sync_tools(&context, None);
    report.resolved = resolved;
    return Ok(report);
}

/// Runs `sync` frozen:
/// it never resolves and never uses the network to read versions.
pub fn run_sync(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
) -> Result<SyncReport, ToolchainError> {
    return run_sync_with(scope, env, runner, http, host, LockPolicy::Frozen(FrozenReason::Flag));
}

pub fn format_sync_report(report: &SyncReport) -> String {
    let mut text: String = String::new();
    for (id, version) in &report.resolved {
        text.push_str(&format!(
            "toolchain sync: locked {id} {version} (the plan was ahead of the lock; resolved alone, no other lock row moved)\n"
        ));
    }
    for warning in &report.warnings {
        text.push_str(&format!("toolchain sync: warning: {warning}\n"));
    }
    text.push_str(&format!(
        "toolchain sync: {} installed, {} unchanged, {} failed\n",
        report.installed.len(),
        report.unchanged.len(),
        report.failed.len()
    ));
    for (id, message) in &report.failed {
        text.push_str(&format!("toolchain sync: {id} failed: {message}\n"));
    }
    if !report.missing_bins.is_empty() {
        text.push_str(&format!("toolchain sync: unresolved bins: {}\n", report.missing_bins.join(", ")));
    }
    return text;
}
