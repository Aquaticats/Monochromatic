//! What:
//! The `list` command:
//! refuse when the plan and lock disagree,
//! otherwise compare what is installed with
//!           the lock,
//! one line per tool,
//! and report whether anything drifted.
//! Why:
//! `list` is the read-only half of `install`;
//! it never installs and never touches the network.
//! It starts `rustup` twice for a `rustup-toolchain` row whose toolchain is at its version:
//! once for the toolchain the row's channel is,
//! and once for the components that toolchain holds,
//! whose `rust-std-<triple>` lines are the targets as well.
//! It waits for each answer with no bound of its own,
//! so a `rustup` that never answers makes it wait (L-7 of the read of issue 6634,
//! measured 2026-11-27).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // status-cmd.ts
//! export function runStatus(scope: Scope, env: EnvMap, runner: Runner, host: Host): { text: string; drifted: boolean }
//! ```

/// Imports `older_lock_warnings` from `super::cargo_installs`.
use super::cargo_installs::older_lock_warnings;
/// Imports what this module uses from `super::context`.
use super::context::{
    Context,
    Host,
};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `NoNetwork` from `super::http`.
use super::http::NoNetwork;
/// Imports the question that says which declared Rust parts a toolchain lacks.
use super::install_rust_android::missing_rust_parts;
/// Imports `load_agreeing` from `super::load`.
use super::load::load_agreeing;
/// Imports `Lock` from `super::lock`.
use super::lock::Lock;
/// Imports the route a row installs by,
/// so the Rust question is asked of the rows it applies to.
use super::model::Route;
/// Imports what this module uses from `super::observe`.
use super::observe::{
    format_status,
    observe_all,
};
/// Imports `plan_old_copies` from `super::old_copies`.
use super::old_copies::plan_old_copies;
/// Imports what this module uses from `super::old_copy_apply`.
use super::old_copy_apply::preview_lines;
/// Imports `Plan` from `super::plan`.
use super::plan::Plan;
/// Imports `Runner` from `super::runner`.
use super::runner::Runner;
/// Imports what this module uses from `super::scope`.
use super::scope::{
    EnvMap,
    Scope,
};

/// The text to print and whether any tool drifted (the exit code is 1 when it did).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReport {
    /// One line per tool and a summary line.
    pub text: String,
    /// Whether any applicable tool is absent or at another version.
    pub drifted: bool,
}

/// What:
/// Runs `list` in a scope.
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
pub fn run_status(
    scope: &Scope,
    env: &EnvMap,
    runner: &dyn Runner,
    host: &Host,
) -> Result<StatusReport, ToolchainError> {
    let (_plan, lock): (Plan, Lock) = load_agreeing(scope)?;
    let context: Context = Context {
        scope,
        env,
        runner,
        http: &NoNetwork,
        host,
        tools: &lock.tools,
    };
    let mut observations = observe_all(&context, &lock.tools);
    // What:
    // the same observations, with each `rustup-toolchain` row that is otherwise at its version asked
    // which declared components and targets its toolchain lacks, from the one component listing.
    // Why:
    // `install` adds a declared part the toolchain does not have, so a toolchain without one is drift;
    // `list` said `ok` and `1 tools in sync` while the declared `rustfmt` component was missing
    // (issue 6634, Low point 5).
    // A row whose toolchain is absent or at another version is already drift, so it is not asked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const observations = observeAll(context, lock.tools).map((observation) => {
    //   const tool = lock.tools.find((candidate) => candidate.declaration.id === observation.id);
    //   if (!observation.inSync || tool?.declaration.route !== 'rustup-toolchain') return observation;
    //   const parts = missingRustParts(context, tool);
    //   return { ...observation, missingParts: parts, inSync: parts.length === 0 };
    // });
    // ```
    for observation in observations.iter_mut() {
        if !observation.in_sync {
            continue;
        }
        let Some(tool) = lock
            .tools
            .iter()
            .find(|tool| return tool.declaration.id == observation.id)
        else {
            continue;
        };
        if tool.declaration.route != Route::RustupToolchain {
            continue;
        }
        match missing_rust_parts(&context, tool) {
            Ok(parts) if !parts.is_empty() => {
                observation.in_sync = false;
                observation.missing_parts = parts;
            }
            Ok(_) => {}
            // A listing that fails is not a clean pass: the tool's state could not be read, and the line
            // says so with the failure's own words.
            Err(error) => {
                observation.in_sync = false;
                observation.missing_parts = vec![format!(
                    "its components and targets could not be read: {}",
                    error.message
                )];
            }
        }
    }
    let mut drifted: bool = false;
    for observation in &observations {
        if !observation.in_sync {
            drifted = true;
        }
    }
    let mut text: String = format_status(&observations);
    for warning in older_lock_warnings(scope, &lock.tools, host.family) {
        text.push_str(&format!("toolchain list: warning: {warning}\n"));
    }
    // An old copy is not drift (the lock is satisfied); `list` names what `install` would do with each, and exits as before.
    for line in preview_lines(&plan_old_copies(&context, &[])) {
        text.push_str(&format!("toolchain list: warning: {line}\n"));
    }
    return Ok(StatusReport {
        text,
        drifted,
    });
}
