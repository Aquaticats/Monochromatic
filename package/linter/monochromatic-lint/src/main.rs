//! What:
//!  The `monochromatic-lint` executable's entry point.
//! Why:
//!  All behavior lives in the library so that tests,
//!  the fuzz sidecar and this binary run the
//! same code;
//!  this file only hands the process's exit status back to the operating system.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.exitCode = runProcess();
//! ```

/// What:
///  Import the typed process exit status.
/// Why:
///  Returning it from `main` sets the exit code without calling `process::exit`,
///  so buffered
/// output is flushed and destructors run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No type; assign a number to process.exitCode.
/// ```
use std::process::ExitCode;

/// What:
///  Run the linter and report 0 for a clean run,
///  1 for failing findings,
///  2 for a setup,
/// usage or processing failure.
/// Why:
///  The operating system calls this function when the program starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function main(): number { return runProcess(); }
/// ```
fn main() -> ExitCode {
    return ExitCode::from(monochromatic_lint::run_process::run_process());
}
