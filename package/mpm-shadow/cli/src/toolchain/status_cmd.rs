//! What:
//! The `status` command:
//! refuse when the plan and lock disagree,
//! otherwise compare what is installed with
//!           the lock,
//! one line per tool,
//! and report whether anything drifted.
//! Why:
//! `status` is the read-only half of `sync`;
//! it never installs and never touches the network.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // status-cmd.ts
//! export function runStatus(scope: Scope, env: EnvMap, runner: Runner, host: Host): { text: string; drifted: boolean }
//! ```

/// Imports what this module uses from `super::context`.
use super::context::{Context, Host};
/// Imports `older_lock_warnings` from `super::cargo_installs`.
use super::cargo_installs::older_lock_warnings;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `NoNetwork` from `super::http`.
use super::http::NoNetwork;
/// Imports `load_agreeing` from `super::load`.
use super::load::load_agreeing;
/// Imports `Lock` from `super::lock`.
use super::lock::Lock;
/// Imports what this module uses from `super::observe`.
use super::observe::{format_status, observe_all};
/// Imports `Plan` from `super::plan`.
use super::plan::Plan;
/// Imports `Runner` from `super::runner`.
use super::runner::Runner;
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope};

/// The text to print and whether any tool drifted (the exit code is 1 when it did).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    /// One line per tool and a summary line.
    pub text: String,
    /// Whether any applicable tool is absent or at another version.
    pub drifted: bool,
}

/// What:
/// Runs `status` in a scope.
/// Why:
/// The runner and host are parameters so tests observe a fake machine;
/// the network is closed,
/// since
///           observing installed tools needs none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runStatus(scope, env, runner, host): StatusReport  // throws when plan and lock disagree
/// ```
pub fn run_status(scope: &Scope, env: &EnvMap, runner: &dyn Runner, host: &Host) -> Result<StatusReport, ToolchainError> {
    let (_plan, lock): (Plan, Lock) = load_agreeing(scope)?;
    let context: Context = Context { scope, env, runner, http: &NoNetwork, host, tools: &lock.tools };
    let observations = observe_all(&context, &lock.tools);
    let mut drifted: bool = false;
    for observation in &observations {
        if !observation.in_sync {
            drifted = true;
        }
    }
    let mut text: String = format_status(&observations);
    for warning in older_lock_warnings(scope, &lock.tools, host.family) {
        text.push_str(&format!("toolchain status: warning: {warning}\n"));
    }
    return Ok(StatusReport { text, drifted });
}
