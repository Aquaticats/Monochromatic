//! What:
//! Bringing the lock up to date with the plan the way `pnpm install --lockfile-only` does,
//! resolving only the entries the lock lacks or holds for an older request,
//! and writing only their rows.
//! Why:
//! `install` runs it before installing,
//! and `install --lockfile-only` runs it alone;
//! no other row moves,
//! even when the registries have moved on,
//! and a lookup that fails stops the sync before anything is installed.
//! A caller imports from `sync` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type LockPreparation = { resolved: [string, string][]; rewritten: string[] };
//! export function prepareLock(scope, env, runner, http, host, policy: LockPolicy, named: string[]): LockPreparation;
//! ```

/// Imports `BTreeSet`,
/// a set sorted by value,
/// from `std::collections`.
use std::collections::BTreeSet;

/// Imports what this file uses from `super::super::agree`.
use super::super::agree::{
    Disagreement,
    compare,
};
/// Imports the host the resolution is made for.
use super::super::context::Host;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports `HttpGet` from `super::super::http`.
use super::super::http::HttpGet;
/// Imports what this file uses from `super::super::load`.
use super::super::load::{
    Loaded,
    load,
};
/// Imports what this file uses from `super::super::lock_cmd`.
use super::super::lock_cmd::{
    LockMode,
    run_lock,
};
/// Imports `Runner` from `super::super::runner`.
use super::super::runner::Runner;
/// Imports what this file uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports what this file uses from `super::super::update`.
use super::super::update::{
    OutcomeKind,
    update_loaded,
};

/// Imports the lock policy this file obeys,
/// and the refusal a frozen run prints.
use super::the_frozen_policy::{
    FrozenReason,
    LockPolicy,
    frozen_refusal,
};
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
            if let OutcomeKind::Failed {
                reason,
            } = &outcome.kind
            {
                message.push_str(&format!("  {}: {reason}\n", outcome.id));
            }
        }
        message.push_str("fix the lookup and run `mpm-shadow install` again, or run `mpm-shadow update <id>` to see it on its own");
        return Err(ToolchainError::failure(message));
    }
    let mut resolved: Vec<(String, String)> = Vec::new();
    for outcome in &report.outcomes {
        match &outcome.kind {
            OutcomeKind::Added {
                version,
            } => resolved.push((outcome.id.clone(), version.clone())),
            OutcomeKind::Updated {
                new_version,
                ..
            } => resolved.push((outcome.id.clone(), new_version.clone())),
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
/// export function prepareLock(scope, env, runner, http, host, policy: LockPolicy, named: string[]): { resolved: [string, string][]; rewritten: string[] }
/// ```
pub fn prepare_lock(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
    policy: LockPolicy,
    named: &[String],
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
                resolved =
                    resolve_before_installing(&loaded, None, scope, env, runner, http, host)?;
            }
        },
        Some(parsed) => {
            let verdict: Disagreement = compare(&loaded.plan, parsed);
            if verdict.digest_problem.is_none() && verdict.needs_update() {
                match policy {
                    LockPolicy::Frozen(reason) => {
                        // A frozen run resolves what its own command line added and nothing else: every other id that
                        // needs a lookup, and every difference the lock could take from the plan without one, refuses.
                        let needed: Vec<String> = verdict.ids_needing_update();
                        let only_named: Vec<String> = needed
                            .iter()
                            .filter(|id| return named.contains(id))
                            .cloned()
                            .collect();
                        let nothing_else_differs: bool = only_named.len() == needed.len()
                            && verdict.offline_fixable_lines().is_empty();
                        if only_named.is_empty() || !nothing_else_differs {
                            return Err(frozen_refusal(&verdict, reason));
                        }
                        resolved = resolve_before_installing(
                            &loaded,
                            Some(&only_named),
                            scope,
                            env,
                            runner,
                            http,
                            host,
                        )?;
                    }
                    LockPolicy::Resolve => {
                        let ids: Vec<String> = verdict.ids_needing_update();
                        resolved = resolve_before_installing(
                            &loaded,
                            Some(&ids),
                            scope,
                            env,
                            runner,
                            http,
                            host,
                        )?;
                        // The same write also carried the plan's other declaration changes into the lock.
                        rewritten = verdict.offline_fixable_lines();
                    }
                }
            } else if verdict.digest_problem.is_none()
                && !verdict.is_empty()
                && policy == LockPolicy::Resolve
            {
                // Only the declaration half differs: pnpm's non-frozen install updates the lockfile for any manifest change.
                run_lock(scope, LockMode::Write)?;
                rewritten = verdict.offline_fixable_lines();
            }
        }
    }
    return Ok(LockPreparation {
        resolved,
        rewritten,
    });
}
