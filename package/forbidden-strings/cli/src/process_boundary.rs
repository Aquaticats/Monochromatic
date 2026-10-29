//! What:
//!  Standalone process ownership of panic output and final exit handling.
//! Why:
//!  A caught matcher panic must not leak scanned bytes through Rust's default panic hook.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Configure process output once, then guard the named startup operation.
//! ```

/// Import the existing application error type.
use anyhow::Result;
/// Imports the process exit status returned once the guarded operation settles.
use std::process::ExitCode;

/// What:
///  Omit default panic payload output;
///  the operation's catch boundary supplies the redacted diagnostic.
/// Why:
///  Payloads,
///  thread names and caller locations are not assumed safe to print.
/// This callback alone does not catch an unwind and must be installed only by the process owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function omitPanicPayload(_information): void {}
/// ```
pub(super) fn omit_panic_payload(_information: &std::panic::PanicHookInfo<'_>) {}

/// Run one named operation,
///  preserving ordinary CLI errors and redacting unexpected panic payloads.
pub(super) fn run(operation: fn() -> Result<i32>) -> ExitCode {
    // The callback has no borrowed captures; catch_unwind controls propagation, independently of the installed hook.
    match std::panic::catch_unwind(operation) {
        Ok(Ok(code)) => return ExitCode::from(code as u8),
        Ok(Err(error)) => {
            eprintln!("forbidden-strings: {error}");
            return ExitCode::from(2);
        }
        Err(_payload) => {
            // Never format the payload: rule text and candidate bytes may be embedded in it.
            eprintln!("forbidden-strings: scanner operation panicked; no complete scan result is available.");
            return ExitCode::from(2);
        }
    }
}

/// Process-isolated controls avoid swapping hooks around concurrent tests or scan calls.
#[cfg(test)]
#[path = "process_boundary_tests.rs"]
mod tests;
