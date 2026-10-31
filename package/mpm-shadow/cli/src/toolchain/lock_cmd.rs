//! What:
//! The `lock` command (offline:
//! rewrite the lock's declaration fields from the plan,
//! keeping every resolved
//!           version and artifact) and `lock --check` (the toolchain drift gate:
//! write nothing,
//! fail on any difference).
//! Why:
//! People edit the plan;
//! `lock` brings the committed lock up to date without the network,
//! and CI runs
//!           `lock --check` beside `file-enforcer check` so a plan edit without a lock refresh,
//! or a hand edit of a
//!           resolved version,
//! fails the build.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lock-cmd.ts
//! export type LockMode = 'check' | 'write';
//! export function runLock(scope: Scope, mode: LockMode): string  // throws ToolchainError
//! ```

/// Imports what this module uses from `super::agree`.
use super::agree::{compare, project, refusal};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::load`.
use super::load::{Loaded, load};
/// Imports what this module uses from `super::lock`.
use super::lock::{Lock, parse_lock, render_lock};
/// Imports `publish_lock` from `super::publish`.
use super::publish::publish_lock;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;

/// What:
/// What `lock` does with the files:
/// only `Check`,
/// or rewrite (`Write`).
/// Why:
/// A lock is derived,
/// so there is no mode that accepts a lock someone else wrote:
/// `update` rebuilds it from the plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockMode = 'check' | 'write';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockMode {
    /// `lock --check`:
    /// write nothing,
    /// fail on any difference.
    Check,
    /// `lock`:
    /// project the plan onto the lock and write it.
    Write,
}

/// What:
/// Runs `lock` in one of its modes in one scope and returns the line to print on success.
/// Why:
/// All modes load the same files and use the same validator;
/// only the last step differs (write or not).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runLock(scope: Scope, mode: LockMode): string
/// ```
pub fn run_lock(scope: &Scope, mode: LockMode) -> Result<String, ToolchainError> {
    let loaded: Loaded = load(scope)?;
    let Some(parsed) = &loaded.lock else {
        return Err(ToolchainError::drift(format!(
            "{} does not exist; run `mpm-shadow update` to resolve the plan's tools and write it",
            scope.lock_file.display()
        )));
    };
    let verdict = compare(&loaded.plan, parsed);
    if mode == LockMode::Check {
        if !verdict.is_empty() {
            return Err(refusal(&verdict));
        }
        return Ok(format!(
            "toolchain lock is current: {} tools, digest verified ({})",
            parsed.lock.tools.len(),
            scope.lock_file.display()
        ));
    }
    if verdict.digest_problem.is_some() || verdict.needs_update() {
        return Err(refusal(&verdict));
    }
    let projected: Lock = project(&loaded.plan, &parsed.lock)?;
    let text: String = render_lock(&projected);
    if loaded.lock_text.as_deref() == Some(text.as_str()) {
        return Ok(format!("toolchain lock already current: {}", scope.lock_file.display()));
    }
    // The text must itself be a lock mpm-shadow accepts: decode what is about to be published.
    parse_lock(&text, "the new toolchain.lock.json")?;
    publish_lock(scope, &loaded.revisions, &text)?;
    return Ok(format!(
        "toolchain lock rewritten from the plan, {} tools, resolution kept ({})",
        projected.tools.len(),
        scope.lock_file.display()
    ));
}
