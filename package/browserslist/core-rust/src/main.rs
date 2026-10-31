//! What:
//!  The `monochromatic-browserslist` binary: prints the result of one command, or the error and exit code 1.
//! Why:
//!  `mise` tasks and the conformance harness call it; all logic lives in the library's `cli::run`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! try { process.stdout.write(run(process.argv.slice(2))); } catch (error) { console.error(String(error)); process.exitCode = 1; }
//! ```

use monochromatic_browserslist::cli::run;
use std::io::Write;

/// What:
///  The program entry point.
/// Why:
///  Rust runs `main` first; it prints the command output or the error message and sets the exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function main(): Promise<number>;
/// ```
fn main() -> std::process::ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(text) => {
            let _ = std::io::stdout().write_all(text.as_bytes());
            return std::process::ExitCode::SUCCESS;
        }
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "{error}");
            return std::process::ExitCode::FAILURE;
        }
    }
}
