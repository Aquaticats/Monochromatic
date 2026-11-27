//! What:
//! What a caller runs `update` or `outdated` through:
//! every tool of the plan,
//! or only the tools the names a person wrote select.
//! Why:
//! With `check` set nothing is installed and nothing is written;
//! otherwise a failed npm or PyPI lookup for a tool that is locked but not installed triggers one sync of just
//! those tools (and what they need),
//! then one retry of each such lookup.
//! A caller imports from `update` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function runUpdate(scope, env, runner, http, host, check): UpdateReport;
//! export function runUpdateFor(scope, env, runner, http, host, check, names: string[]): UpdateReport;
//! ```

/// Imports `BTreeSet`,
/// a set sorted by value.
use std::collections::BTreeSet;

/// Imports what this file uses from `super::super::context`.
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
/// Imports `Runner` from `super::super::runner`.
use super::super::runner::Runner;
/// Imports what this file uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports `select_tools` from `super::super::update_select`.
use super::super::update_select::select_tools;

/// Imports the update over files that are already read.
use super::the_update_itself::update_loaded;
/// Imports the report the update answers with.
use super::what_an_update_did::UpdateReport;
/// What:
/// Runs `update` (or `outdated`) over every tool of the plan in a scope.
/// Why:
/// It is `run_update_for` with no names.
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
    return run_update_for(scope, env, runner, http, host, check, &[]);
}

/// What:
/// Runs `update` (or `outdated`) in a scope for the tools the `names` patterns select
/// (every tool when `names` is empty).
/// Why:
/// With `check` set nothing is installed and nothing is written.
/// Otherwise a failed npm or PyPI lookup for a
///           tool that is locked but not installed triggers one sync of just those tools (and what they need),
/// then one
///           retry of each such lookup;
/// a second failure stays a failure naming the tool.
/// With names,
/// only the selected tools are looked up;
/// every other row keeps its version and artifacts and takes the plan's current bins,
/// env and options,
/// as offline `install --lockfile-only` would give it.
/// A new tool is therefore resolved alone by naming it,
/// and a plan entry the lock cannot hold that the names left out is refused before any lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runUpdateFor(scope, env, runner, http, host, check, names: string[]): UpdateReport
/// ```
pub fn run_update_for(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    http: &dyn HttpGet,
    host: &Host,
    check: bool,
    names: &[String],
) -> Result<UpdateReport, ToolchainError> {
    let loaded: Loaded = load(scope)?;
    let mut only: Option<BTreeSet<String>> = None;
    if !names.is_empty() {
        only = Some(select_tools(&loaded.plan, names)?);
    }
    return update_loaded(
        &loaded,
        scope,
        env,
        runner,
        http,
        host,
        check,
        only.as_ref(),
    );
}
