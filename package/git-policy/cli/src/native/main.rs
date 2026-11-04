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
    // What: `.skip(1)` drops the program name; `.collect()` gathers the rest into an
    //       owned list. `Vec<OsString>` is that list type.
    // Why:  Git receives only the caller's arguments, in order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const args = process.argv.slice(2);
    // ```
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    // The service mode is chosen by the leading words (`cli-git service run`), never by the name this program was started
    // as, and before anything else of the wrapper runs: the service's process setup is its own, and the wrapper's start
    // never takes it (doc/planning/cli-git-resident-service.md, "One executable").
    if git_policy_cli::service_main::is_service_start(arguments.as_slice()) {
        let code: i32 = match git_policy_cli::service_main::service_options(arguments.as_slice()) {
            Ok(options) => git_policy_cli::service_main::run(options.as_slice()),
            Err(word) => {
                git_policy_cli::service_error!("an argument of the service is not text: {}", word.to_string_lossy());
                git_policy_cli::service_main::EXIT_REFUSED
            }
        };
        std::process::exit(code);
    }
    // The linked scanner's catch boundaries run after the panic hook, so a hook that never
    // prints a panic's message is installed before anything can scan.
    git_policy_cli::panic_notice::install_payload_free_panic_hook();
    // `vars_os` yields every environment entry as an undecoded name/value pair.
    let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    // `i32` is the signed 32-bit type of process exit codes.
    let code: i32 = git_policy_cli::entry::run_process(arguments.as_slice(), environment.as_slice());
    std::process::exit(code);
}
