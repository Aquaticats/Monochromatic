//! What:
//! What a caller runs `install` through:
//! the sync under a lock policy,
//! the same sync for a command line that named tools to add,
//! `install --lockfile-only`,
//! and the frozen install a bare `mpm-shadow install` is.
//! Why:
//! The policy decides whether a plan ahead of the lock is resolved or refused,
//! and every entry point here ends in the one sync,
//! so a caller never assembles the steps itself.
//! A caller imports from `sync` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function runSyncWith(scope, env, runner, http, host, policy: LockPolicy): SyncReport;
//! export function runSyncWithNamed(scope, env, runner, http, host, policy: LockPolicy, named: string[]): SyncReport;
//! export function runLockfileOnly(scope, env, runner, http, host, policy: LockPolicy): LockPreparation;
//! export function runSync(scope, env, runner, http, host): SyncReport;
//! ```

/// Imports what this file uses from `super::super::context`.
use super::super::context::{
    Context,
    Host,
};
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports `HttpGet` from `super::super::http`.
use super::super::http::HttpGet;
/// Imports `load_agreeing` from `super::super::load`.
use super::super::load::load_agreeing;
/// Imports `Lock` from `super::super::lock`.
use super::super::lock::Lock;
/// Imports the plan `load_agreeing` answers with.
use super::super::plan::Plan;
/// Imports `Runner` from `super::super::runner`.
use super::super::runner::Runner;
/// Imports what this file uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports `retire_after_install` from `super::super::sync_old_copies`.
use super::super::sync_old_copies::retire_after_install;

/// Imports the lock preparation,
/// and the step that brings the lock up to date with the plan.
use super::preparing_the_lock::{
    LockPreparation,
    prepare_lock,
};
/// Imports the lock policy every entry point passes,
/// and the reason a frozen one names.
use super::the_frozen_policy::{
    FrozenReason,
    LockPolicy,
};
/// Imports the sync that installs the tools.
use super::the_sync_itself::sync_tools;
/// Imports the report the sync answers with.
use super::what_the_sync_did::SyncReport;
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
    return run_sync_with_named(scope, env, runner, http, host, policy, &[]);
}

/// What:
/// `run_sync_with` for a command line that named tools to add,
/// whose ids are `named`.
/// Why:
/// `pnpm add --frozen-lockfile <package>` writes `package.json` and the lockfile all the same
/// (measured with pnpm 12.18.0 on 2026-11-13),
/// so a frozen run resolves the entries its own command line added;
/// every other entry keeps the frozen refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runSyncWithNamed(scope, env, runner, http, host, policy: LockPolicy, named: string[]): SyncReport
/// ```
pub fn run_sync_with_named(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
    policy: LockPolicy,
    named: &[String],
) -> Result<SyncReport, ToolchainError> {
    let prepared: LockPreparation = prepare_lock(scope, env, runner, http, host, policy, named)?;
    let (_plan, lock): (Plan, Lock) = load_agreeing(scope)?;
    let context: Context = Context {
        scope,
        env,
        runner,
        http,
        host,
        tools: &lock.tools,
    };
    let mut report: SyncReport = sync_tools(&context, None);
    report.resolved = prepared.resolved;
    report.rewritten = prepared.rewritten;
    retire_after_install(&context, &mut report);
    return Ok(report);
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
    let prepared: LockPreparation = prepare_lock(scope, env, runner, http, host, policy, &[])?;
    load_agreeing(scope)?;
    return Ok(prepared);
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
    return run_sync_with(
        scope,
        env,
        runner,
        http,
        host,
        LockPolicy::Frozen(FrozenReason::Flag),
    );
}
