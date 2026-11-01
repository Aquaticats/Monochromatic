//! What:
//!  The call order around a guarded Git child:
//!       install the signal guard, run the child, release the locks and the transaction, drop the guard,
//!       then exit with the child's code.
//! Why:
//!  The incumbent releases the transaction before the signal guard goes away, so a signal sent during
//!  release is still relayed to the child.
//!  The order after a landed commit (automatic maintenance, then the post-commit hook) is not here:
//!  the incumbent runs maintenance without stopping on failure (`src/policy-engine/commit-transaction-conclusion.ts`),
//!  and the post-landing lane owns that sequence.
//!  Nothing calls this yet: the command path keeps refusing `git commit` until landing is merged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const guard = installSignalGuard(); const outcome = await run(child); await release(); guard.drop(); exit(exitCode(outcome));
//! ```

use super::forwarding::exit_code;
use super::signal_relay::SignalGuard;
use std::process::Command;

/// What:
///  The steps that release what a guarded child held, run after the child ends and before the guard drops.
/// Why:
///   Release runs while the guard still relays signals, so a signal sent during release reaches the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface ReleaseSteps { release(): Promise<void>; }
/// ```
pub trait ReleaseSteps {
    /// Releases the landing lock and removes the transaction.
    fn release(&mut self) -> Result<(), String>;
}

/// What:
///  Run a Git child under the signal guard, release what it held, and report its exit code.
/// Why:
///   Release runs even when the child fails to run, and each failure is reported, not hidden.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guard = installSignalGuard(); const outcome = await run(child); await release(); guard.drop(); exit(exitCode(outcome));
/// ```
pub fn run_guarded_child(command: &mut Command, release: &mut dyn ReleaseSteps) -> Result<i32, String> {
    let guard: SignalGuard = match SignalGuard::install() {
        Ok(installed) => installed,
        Err(error) => return Err(format!("installing the signal guard failed: {error}")),
    };
    let outcome = guard.run_child(command);
    let released: Result<(), String> = release.release();
    drop(guard);
    let child = match outcome {
        Ok(found) => found,
        Err(error) => return Err(format!("running the child failed: {error}")),
    };
    if let Err(error) = released {
        return Err(format!("releasing after the child failed: {error}"));
    }
    return Ok(exit_code(child));
}

#[cfg(test)]
#[path = "commit_lifecycle_tests.rs"]
mod tests;
