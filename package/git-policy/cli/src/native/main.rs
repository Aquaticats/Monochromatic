//! What:
//!  The thin native cli-git executable.
//! Why:
//!  Process facts are gathered once here;
//!  every decision lives in the library.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.exit(runProcess(process.argv.slice(2), Object.entries(process.env)));
//! ```

/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments and environment values reach Git exactly as the caller gave them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.argv / process.env, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  The program entry point:
///  collect arguments and environment,
///  run,
///  exit.
/// Why:
///   `args_os`/`vars_os` never decode,
///  so non-UTF-8 arguments neither panic nor
///       change.
///  A forwarded command replaces this process and never reaches the exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function main(): never { process.exit(runProcess(argv, env)); }
/// ```
fn main() {
    // The linked scanner's catch boundaries run after the panic hook, so a hook that never
    // prints a panic's message is installed before anything can scan.
    git_policy_cli::panic_notice::install_payload_free_panic_hook();
    // What: `.skip(1)` drops the program name; `.collect()` gathers the rest into an
    //       owned list. `Vec<OsString>` is that list type.
    // Why:  Git receives only the caller's arguments, in order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const args = process.argv.slice(2);
    // ```
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    // `vars_os` yields every environment entry as an undecoded name/value pair.
    let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    // What: `argv[0]` is the start name; a hook entry is recognized by it, and Git starts hooks by their path.
    // Why:  `current_exe` resolves symbolic links and would lose the hooks directory.
    let start_name: Option<OsString> = std::env::args_os().next();
    // `i32` is the signed 32-bit type of process exit codes.
    let code: i32 = git_policy_cli::hook_dispatch::run_executable(
        start_name.as_deref(),
        arguments.as_slice(),
        environment.as_slice(),
    );
    std::process::exit(code);
}
