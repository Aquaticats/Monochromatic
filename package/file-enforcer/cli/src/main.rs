//! What:
//!  The `file-enforcer` executable:
//! hand the process arguments to the library and exit with its code.
//! Why:
//!  All behavior lives in `file_enforcer_cli` so tests can call it without starting a process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! #!/usr/bin/env node
//! process.exit(await main(process.argv.slice(2)));
//! ```

/// Imports `ExitCode` from `std::process`.
use std::process::ExitCode;

/// What:
///  Run the command line and convert the code into the process exit status.
///  `ExitCode` is the value `main` returns to set the exit status (TS:
/// `process.exitCode`).
/// Why:
///  Returning (not calling `exit`) lets destructors and buffered writes finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = code;
/// ```
fn main() -> ExitCode {
    let arguments: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let code = file_enforcer_cli::run_command_line(&arguments);
    return ExitCode::from(u8::try_from(code).unwrap_or(1));
}
