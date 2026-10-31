//! What:
//! `install`:
//! install every locked tool that is not already at its locked version,
//! in dependency order,
//! then make
//!           the postinstall links,
//! rebuild the exposure directory and run the postinstall commands.
//! Why:
//! This is the installer.
//! It reads only the lock (which `install` first proves agrees with the plan),
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

/// Imports `BTreeSet`,
/// a set sorted by value,
/// from `std::collections`.
use std::collections::BTreeSet;

/// Imports what this module uses from `super::agree`.
use super::agree::{Disagreement, compare, refusal};
/// Imports what this module uses from `super::cargo_installs`.
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
/// Imports what this module uses from `super::load`.
use super::load::{Loaded, load, load_agreeing};
/// Imports what this module uses from `super::lock_cmd`.
use super::lock_cmd::{LockMode, run_lock};
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
/// Imports what this module uses from `super::update`.
use super::update::{OutcomeKind, update_loaded};
/// Imports the tool lookup.
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

/// What `install` did:
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
    /// The tools `install` resolved and locked itself before installing,
    /// as tool id and locked version:
    /// the plan entries that were new or whose resolution-affecting fields changed.
    pub resolved: Vec<(String, String)>,
    /// What `install` wrote into the lock from the plan without a lookup:
    /// one sentence per offline-fixable difference,
    /// such as a changed `bins` list.
    pub rewritten: Vec<String>,
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
/// `install` calls it
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
/// The reason a frozen `install` did not resolve a plan that is ahead of the lock.
/// Why:
/// The refusal tells the person which switch or environment stopped `install` from locking the entries itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrozenReason {
    /// `--frozen-lockfile` was given.
    Flag,
    /// A CI environment was detected (the `ci-info` rules pnpm uses).
    Ci,
}

/// What:
/// Whether `install` may resolve plan entries the lock lacks.
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
    /// Resolve new and re-edited entries alone,
    /// write only their rows,
    /// then install.
    Resolve,
    /// Refuse a plan that is ahead of the lock and name `mpm-shadow update <ids>`;
    /// the reason says which switch or environment froze it.
    Frozen(
        /// Why the lock is frozen.
        FrozenReason,
    ),
}

/// The refusal for a frozen `install` whose plan is ahead of the lock:
/// the usual disagreement message plus why `install` did not resolve.
fn frozen_refusal(verdict: &Disagreement, reason: FrozenReason) -> ToolchainError {
    let base: ToolchainError = refusal(verdict);
    let why: &str = match reason {
        FrozenReason::Flag => "`--frozen-lockfile` was given",
        FrozenReason::Ci => {
            "a CI environment was detected (the CI variable or another marker, by the rules of pnpm and ci-info), \
             where the lock is frozen; `CI=false` or `--no-frozen-lockfile` turns that off"
        }
    };
    return ToolchainError::drift(format!(
        "{}\nmpm-shadow install did not resolve them itself because {why}; run `mpm-shadow update <ids>` (or `mpm-shadow install --no-frozen-lockfile`), commit the lock, then install again",
        base.message
    ));
}

/// What:
/// Resolves the plan entries the lock lacks or holds for an older request,
/// and only those,
/// and writes only their rows.
/// Why:
/// This is `mpm-shadow update <those ids>` done by `install`:
/// no other row moves,
/// even when the registries have moved on.
/// A lookup that fails stops the sync before anything is installed,
/// because a lock that does not describe the plan would install the wrong tools.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveBeforeInstalling(loaded, only: string[] | undefined, scope, env, runner, http, host): [string, string][]
/// ```
fn resolve_before_installing(
    loaded: &Loaded,
    ids: Option<&[String]>,
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
) -> Result<Vec<(String, String)>, ToolchainError> {
    let mut only: Option<BTreeSet<String>> = None;
    if let Some(given) = ids {
        only = Some(given.iter().cloned().collect());
    }
    let report = update_loaded(loaded, scope, env, runner, http, host, false, only.as_ref())?;
    if report.has_failures() {
        let mut message: String = String::from(
            "mpm-shadow install could not resolve the plan entries the lock lacks, so it wrote nothing and installed nothing:\n",
        );
        for outcome in &report.outcomes {
            if let OutcomeKind::Failed { reason } = &outcome.kind {
                message.push_str(&format!("  {}: {reason}\n", outcome.id));
            }
        }
        message.push_str("fix the lookup and run `mpm-shadow install` again, or run `mpm-shadow update <id>` to see it on its own");
        return Err(ToolchainError::failure(message));
    }
    let mut resolved: Vec<(String, String)> = Vec::new();
    for outcome in &report.outcomes {
        match &outcome.kind {
            OutcomeKind::Added { version } => resolved.push((outcome.id.clone(), version.clone())),
            OutcomeKind::Updated { new_version, .. } => resolved.push((outcome.id.clone(), new_version.clone())),
            // Looked up again and found the same version (a re-edited entry whose answer did not change).
            OutcomeKind::Unchanged | OutcomeKind::Pinned => {
                if let Some(parsed) = &loaded.lock
                    && let Some(locked) = parsed.lock.tool(&outcome.id)
                {
                    resolved.push((outcome.id.clone(), locked.version.clone()));
                }
            }
            _ => {}
        }
    }
    return Ok(resolved);
}

/// What:
/// Runs `install` under a lock policy.
/// Why:
/// A plan that disagrees with the lock only in what `install --lockfile-only` fixes offline,
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
    let prepared: LockPreparation = prepare_lock(scope, env, runner, http, host, policy)?;
    let (_plan, lock): (Plan, Lock) = load_agreeing(scope)?;
    let context: Context = Context { scope, env, runner, http, host, tools: &lock.tools };
    let mut report: SyncReport = sync_tools(&context, None);
    report.resolved = prepared.resolved;
    report.rewritten = prepared.rewritten;
    return Ok(report);
}

/// What `prepare_lock` did to the lock.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LockPreparation {
    /// The tools it resolved and locked,
    /// as tool id and version.
    pub resolved: Vec<(String, String)>,
    /// One sentence per plan change it wrote into the lock without a lookup.
    pub rewritten: Vec<String>,
}

/// What:
/// Brings the lock up to date with the plan the way `pnpm install --lockfile-only` does,
/// without installing anything,
/// and returns the tool ids and versions it locked.
/// Why:
/// `install` runs it before installing,
/// and `install --lockfile-only` runs it alone.
/// A frozen policy resolves nothing into an existing lock;
/// no lock is resolved in full unless `--frozen-lockfile` was given.
/// Whatever still disagrees afterwards (what offline `install --lockfile-only` fixes) is left to the caller's check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function prepareLock(scope, env, runner, http, host, policy: LockPolicy): { resolved: [string, string][]; rewritten: string[] }
/// ```
pub fn prepare_lock(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
    policy: LockPolicy,
) -> Result<LockPreparation, ToolchainError> {
    let loaded: Loaded = load(scope)?;
    let mut resolved: Vec<(String, String)> = Vec::new();
    let mut rewritten: Vec<String> = Vec::new();
    match &loaded.lock {
        None => match policy {
            // pnpm: `--frozen-lockfile` fails when no lockfile is present (ERR_PNPM_NO_LOCKFILE).
            LockPolicy::Frozen(FrozenReason::Flag) => {
                return Err(ToolchainError::drift(format!(
                    "{} does not exist and `--frozen-lockfile` forbids install from creating it; \
                     run `mpm-shadow install --no-frozen-lockfile`, or `mpm-shadow update`, to resolve the plan's tools and write it",
                    scope.lock_file.display()
                )));
            }
            // pnpm: the CI default is frozen only "if a lockfile is present", so without one it resolves and writes.
            LockPolicy::Frozen(FrozenReason::Ci) | LockPolicy::Resolve => {
                resolved = resolve_before_installing(&loaded, None, scope, env, runner, http, host)?;
            }
        },
        Some(parsed) => {
            let verdict: Disagreement = compare(&loaded.plan, parsed);
            if verdict.digest_problem.is_none() && verdict.needs_update() {
                match policy {
                    LockPolicy::Frozen(reason) => return Err(frozen_refusal(&verdict, reason)),
                    LockPolicy::Resolve => {
                        let ids: Vec<String> = verdict.ids_needing_update();
                        resolved = resolve_before_installing(&loaded, Some(&ids), scope, env, runner, http, host)?;
                        // The same write also carried the plan's other declaration changes into the lock.
                        rewritten = verdict.offline_fixable_lines();
                    }
                }
            } else if verdict.digest_problem.is_none() && !verdict.is_empty() && policy == LockPolicy::Resolve {
                // Only the declaration half differs: pnpm's non-frozen install updates the lockfile for any manifest change.
                run_lock(scope, LockMode::Write)?;
                rewritten = verdict.offline_fixable_lines();
            }
        }
    }
    return Ok(LockPreparation { resolved, rewritten });
}

/// What:
/// Runs `install --lockfile-only` online:
/// brings the lock up to date and proves it agrees with the plan,
/// installing nothing.
/// Why:
/// This is the way to lock a new plan entry without installing it,
/// as `pnpm install --lockfile-only` does for a new dependency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runLockfileOnly(scope, env, runner, http, host, policy: LockPolicy): LockPreparation
/// ```
pub fn run_lockfile_only(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
    policy: LockPolicy,
) -> Result<LockPreparation, ToolchainError> {
    let prepared: LockPreparation = prepare_lock(scope, env, runner, http, host, policy)?;
    load_agreeing(scope)?;
    return Ok(prepared);
}

/// The line `install` prints for a plan change it wrote into the lock without a lookup.
pub fn rewritten_line(description: &str) -> String {
    return format!("toolchain install: lock updated from the plan: {description}\n");
}

/// The line `install` prints for a tool it locked itself.
pub fn locked_line(id: &str, version: &str) -> String {
    return format!(
        "toolchain install: locked {id} {version} (the plan was ahead of the lock; resolved alone, no other lock row moved)\n"
    );
}

/// Runs `install` frozen:
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

/// The text `install` prints for a report (the tools it locked first,
/// then warnings,
/// the counts and each failure).
pub fn format_sync_report(report: &SyncReport) -> String {
    let mut text: String = String::new();
    for (id, version) in &report.resolved {
        text.push_str(&locked_line(id, version));
    }
    for description in &report.rewritten {
        text.push_str(&rewritten_line(description));
    }
    for warning in &report.warnings {
        text.push_str(&format!("toolchain install: warning: {warning}\n"));
    }
    text.push_str(&format!(
        "toolchain install: {} installed, {} unchanged, {} failed\n",
        report.installed.len(),
        report.unchanged.len(),
        report.failed.len()
    ));
    for (id, message) in &report.failed {
        text.push_str(&format!("toolchain install: {id} failed: {message}\n"));
    }
    if !report.missing_bins.is_empty() {
        text.push_str(&format!("toolchain install: unresolved bins: {}\n", report.missing_bins.join(", ")));
    }
    return text;
}
