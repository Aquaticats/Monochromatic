//! What:
//! The exit code a shell reports for a child process that ended:
//! the child's own code,
//! or `128 + signal` when a signal ended it.
//! Why:
//! Two features run the real tool as a child instead of replacing the process with it:
//! the version and help announcement (`announce.rs`)
//! and the output cap (`output_cap.rs`).
//! Both must report exactly what a shell reports for the same process,
//! so the conversion lives once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // child-exit.ts
//! export function shellExitCode(result: SpawnSyncReturns<Buffer>): number {
//!   return result.status ?? 128 + (result.signal ? signalNumber(result.signal) : 0);
//! }
//! ```

/// Imports `ExitStatus`,
/// how a child process ended.
use std::process::ExitStatus;

/// Imports the Unix-only trait that adds `.signal()` to an exit status,
/// so a tool killed by a signal gets the shell's `128 + signal`.
use std::os::unix::process::ExitStatusExt;

/// What:
/// `pub fn shell_exit_code(status: ExitStatus) -> i32` declares a public function turning how a process ended into the
/// code a shell would report for it.
/// `ExitStatus` is how `std::process` says a child ended (a code, a signal, or neither).
/// Why:
/// The shadow must never invent a status:
/// a script that checks `$?` after a shadowed `rg` must see what it would see after the real one.
/// The code is `i32` because `std::process::exit` takes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function shellExitCode(status: ExitStatus): number {
///   if (status.code !== null) return status.code;
///   if (status.signal !== null) return 128 + status.signal;
///   return 1;
/// }
/// ```
pub fn shell_exit_code(status: ExitStatus) -> i32 {
    // What:     `if let Some(code) = status.code()` runs the block when the child ended by exiting itself.
    // Why:      That code is reported as it is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (status.code !== null) return status.code;
    // ```
    if let Some(code) = status.code() {
        return code;
    }
    // What:     `status.signal()` names the signal that ended the child; the shell convention adds it to 128.
    // Why:      `rg` ended by SIGTERM (143) and one ended by a kill (137) must read the same through the shadow.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (status.signal !== null) return 128 + status.signal;
    // ```
    if let Some(signal) = status.signal() {
        return 128 + signal;
    }
    return 1;
}

/// The unit test of the conversion.
#[cfg(test)]
#[path = "child_exit_tests.rs"]
mod tests;
