//! main support for the forbidden-strings scanner.
/// The standalone process owns panic output;
///  loading the library never changes a host's hook.
mod process_boundary;
/// Imports dependencies used by this module.
// What:     `use forbidden_strings::run_cli_from_env;` imports the
//           public lib entry point. The crate's library target is
//           named `forbidden_strings` (underscores; the `[[bin]]`
//           name keeps the hyphen for the on-disk binary). The lib
//           owns argv parsing, env var reads, ruleset loading, the
//           parallel scan, and stderr emission; this file just
//           translates its return into the OS exit code.
// Why:      Move every interesting branch out of `main`. Tests
//           (`tests/integration.rs`) still drive the binary as a
//           subprocess, but additional fuzz targets and unit tests
//           can now exercise `run_cli_from_env` (and the helpers it
//           re-exports through the `fuzzing` Cargo feature) without
//           spawning a child process.
//
// In TS you'd write (pseudocode):
// ```ts
// import { runCliFromEnv } from "./lib";
// ```
use forbidden_strings::run_cli_from_env;

/// Imports dependencies used by this module.
// What:     `use std::process::ExitCode;` imports the typed wrapper
//           for OS exit codes. Returning `ExitCode` from `main` is
//           the idiomatic way to set the exit status from Rust.
// Why:      We need it to translate `run_cli_from_env`'s `i32`
//           into the typed value `main` returns.
//
// In TS you'd write (pseudocode):
// ```ts
// // No type; just a number.
// ```
use std::process::ExitCode;

/// Select the existing environment filter without introducing a captured fallback callback.
fn logging_filter() -> tracing_subscriber::EnvFilter {
    if let Ok(filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        return filter;
    }
    return tracing_subscriber::EnvFilter::new("info");
}

/// Initialize the existing logger and run the CLI inside the process's unwind boundary.
fn initialized_cli() -> anyhow::Result<i32> {
    // Keep all startup logging behavior inside the same failure boundary as rule loading and scanning.
    tracing_subscriber::fmt()
        .with_env_filter(logging_filter())
        .with_writer(std::io::stderr)
        .init();
    return run_cli_from_env();
}

/// Implements `main`.
// What:     `fn main() -> ExitCode` is the program entry point. It
//           dispatches to `run_cli_from_env` and converts the
//           returned `Result<i32>` into an `ExitCode`. The
//           `Err` arm prints the catastrophic error to stderr with
//           a fixed `forbidden-strings:` prefix and exits 2; the
//           process boundary also catches unexpected unwinds and emits
//           a fixed redacted failure. The startup hook omits payloads;
//           the separate catch boundary determines the final status.
// Why:      Keep `main` to a five-line wrapper so the lib is the
//           sole carrier of business logic, and tests can drive
//           every code path without spawning a subprocess.
//
// In TS you'd write (pseudocode):
// ```ts
// try { process.exit(await runCliFromEnv()); }
// catch (e) { console.error(`forbidden-strings: ${e}`); process.exit(2); }
// ```
fn main() -> ExitCode {
    // What:     `match run_cli_from_env() { Ok(code) => ..., Err(e) => ... }`
    //           destructures the `Result<i32>`. `Ok(code)`
    //           binds the inner `i32` and the arm converts it to
    //           `ExitCode::from(code as u8)` -- the cast is safe in
    //           practice because every `Ok` arm in the lib returns
    //           a value in `{0, 1, 2}`, well inside `u8` range.
    //           `Err(e)` binds the error message, prints it to
    //           stderr with the conventional prefix, and exits 2.
    // Why:      Bridge the lib's testable return shape to the OS-
    //           expected `ExitCode`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { process.exit(code); }
    // catch (e) { console.error(`forbidden-strings: ${e}`); process.exit(2); }
    // ```
    // Install once before logger initialization or worker creation, never around individual concurrent scans.
    std::panic::set_hook(Box::new(process_boundary::omit_panic_payload));
    // The hook controls only output; the separate named-operation boundary catches unwinds and returns exit 2.
    return process_boundary::run(initialized_cli);
}

/// Process-isolated filter controls do not alter production startup or the embedding host's environment.
#[cfg(test)]
#[path = "main_logging_tests.rs"]
mod logging_tests;
