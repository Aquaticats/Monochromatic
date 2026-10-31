//! What:     `sync`: install every locked tool that is not already at its locked version, in dependency order, then make
//!           the postinstall links, rebuild the exposure directory and run the postinstall commands.
//! Why:      This is the installer. It reads only the lock (which `sync` first proves agrees with the plan), probes
//!           before installing so a second run changes nothing, and keeps going after one tool fails so the report
//!           names every failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sync.ts
//! export function syncOrder(tools: LockedTool[]): LockedTool[]
//! export function syncTools(ctx: Context, only?: string[]): SyncReport
//! export function runSync(scope, env, runner, http, host): SyncReport  // throws when plan and lock disagree
//! ```

use super::context::{Context, Host};
use super::error::ToolchainError;
use super::exposure::{rebuild_exposure, run_postinstall_commands, run_postinstall_links, unresolvable_bins};
use super::http::HttpGet;
use super::install::install_tool;
use super::layout::applies_to_family;
use super::load::load_agreeing;
use super::lock::Lock;
use super::model::LockedTool;
use super::observe::observe_tool;
use super::parse_output::version_matches;
use super::plan::Plan;
use super::runner::Runner;
use super::scope::{EnvMap, Scope};

/// The tools installed first, in this order, because the others need them: the `bin` downloader, `uv`, `mpm`, the Rust
/// toolchain, `cargo-binstall`, Go, Node and `pnpm`.
pub const STAGING_IDS: [&str; 9] = ["bin", "uv", "mpm", "rustup", "rust", "cargo-binstall", "go", "node", "pnpm"];

/// What:     The tools in install order: the staging tools that exist first (in staging order), then the rest in lock
///           order.
/// Why:      Installing `npm:wrangler` needs `pnpm`, which needs `node`; the order is a data table, not a dependency
///           solver, because the repository's dependency chain is short and fixed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncOrder(tools: LockedTool[]): LockedTool[]
/// ```
pub fn sync_order(tools: &[LockedTool]) -> Vec<&LockedTool> {
    let mut ordered: Vec<&LockedTool> = Vec::new();
    for staging_id in STAGING_IDS {
        if let Some(found) = tools.iter().find(|tool| return tool.declaration.id == staging_id) {
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

/// What `sync` did: tools already at their version, tools installed, tools that failed (with why), declared bins still
/// unresolved afterwards, and warnings (a shared install being replaced).
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
}

/// What:     The warning for an install that would replace a different version, or `None` when the tool is absent or
///           already matches.
/// Why:      Several worktrees share one install directory per tool version, and a manager-level install (`cargo
///           binstall`, `uv tool install`) replaces the one version everywhere.
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

/// What:     Syncs the tools of `ctx.tools` (all of them, or only those whose ids are in `only`), in install order.
/// Why:      `update` calls it with `only` set to install just the npm and PyPI tools its lookups need; `sync` calls it
///           with `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncTools(ctx: Context, only?: string[]): SyncReport
/// ```
pub fn sync_tools(ctx: &Context, only: Option<&[String]>) -> SyncReport {
    let mut report: SyncReport = SyncReport::default();
    let entries: Vec<&LockedTool> = sync_order(ctx.tools)
        .into_iter()
        .filter(|tool| return applies_to_family(&tool.declaration.os, ctx.host.family))
        .filter(|tool| return only.is_none_or(|ids| return ids.contains(&tool.declaration.id)))
        .collect();
    for tool in &entries {
        let observed: Option<String> = observe_tool(ctx, tool);
        if let Some(warning) = replacement_warning(&tool.declaration.id, &observed, &tool.version) {
            report.warnings.push(warning);
        }
        let current: bool = observed.as_deref().is_some_and(|found| return version_matches(found, &tool.version));
        if current {
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
    return report;
}

/// What:     The `sync` command: load the plan and lock (refusing when they disagree), then sync every tool.
/// Why:      Acting on a lock that does not describe the plan would install the wrong tools.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runSync(scope, env, runner, http, host): SyncReport
/// ```
pub fn run_sync(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
) -> Result<SyncReport, ToolchainError> {
    let (_plan, lock): (Plan, Lock) = load_agreeing(scope)?;
    let context: Context = Context { scope, env, runner, http, host, tools: &lock.tools };
    return Ok(sync_tools(&context, None));
}

/// The text `sync` prints for a report (warnings first, then the counts and each failure).
pub fn format_sync_report(report: &SyncReport) -> String {
    let mut text: String = String::new();
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
