//! What:     The `status` command: refuse when the plan and lock disagree, otherwise compare what is installed with
//!           the lock, one line per tool, and report whether anything drifted.
//! Why:      `status` is the read-only half of `sync`; it never installs and never touches the network.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // status-cmd.ts
//! export function runStatus(scope: Scope, env: EnvMap, runner: Runner, host: Host): { text: string; drifted: boolean }
//! ```

use super::context::{Context, Host};
use super::error::ToolchainError;
use super::http::NoNetwork;
use super::load::load_agreeing;
use super::lock::Lock;
use super::observe::{format_status, observe_all};
use super::plan::Plan;
use super::runner::Runner;
use super::scope::{EnvMap, Scope};

/// The text to print and whether any tool drifted (the exit code is 1 when it did).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    /// One line per tool and a summary line.
    pub text: String,
    /// Whether any applicable tool is absent or at another version.
    pub drifted: bool,
}

/// What:     Runs `status` in a scope.
/// Why:      The runner and host are parameters so tests observe a fake machine; the network is closed, since
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
    let drifted: bool = observations.iter().any(|observation| return !observation.in_sync);
    return Ok(StatusReport { text: format_status(&observations), drifted });
}
