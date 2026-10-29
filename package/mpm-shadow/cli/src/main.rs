//! What:     The program entry point. It looks at the name it was started under: `mpm` means shadow mode; anything
//!           else (`mpm-shadow`) means the command-line tool.
//! Why:      One binary serves both roles, so the link the toolchain plan creates as `mpm` needs no second program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // main.ts
//! const invokedAs = path.basename(process.argv[1]);
//! process.exit(invokedAs === 'mpm' ? runShim(process.argv.slice(2)) : runCli(process.argv));
//! ```

/// What:     `use mpm_shadow::cli::run_cli;` imports a function from this package's own library crate (`mpm_shadow`,
///           declared in `Cargo.toml`).
/// Why:      `main` only chooses the mode; all logic lives in the library.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runCli } from './cli.ts';
/// ```
use mpm_shadow::cli::run_cli;

/// Imports the name the shadow answers to.
use mpm_shadow::constants::SHADOWED_NAME;

/// Imports the shadow-mode function.
use mpm_shadow::shim::run_shim;

/// Imports the owned operating-system text type.
use std::ffi::OsString;

/// Imports the process-environment module.
use std::env;

/// Imports `Path` to take the file name off argv[0].
use std::path::Path;

/// What:     `fn main()` is the entry point the operating system calls; the exit code is set by `std::process::exit`.
/// Why:      Every run starts here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function main(): void { ... process.exit(code); }
/// ```
fn main() {
    // `env::args_os().collect()` gathers every argument as `OsString`s into a `Vec`; mpm arguments may be invalid
    // UTF-8, and `env::args()` would panic on them.
    let all_arguments: Vec<OsString> = env::args_os().collect();

    // The name the program was started under: argv[0] keeping only its final path segment, as text.
    let mut invoked_as: String = String::new();
    if let Some(file_name) = Path::new(&all_arguments[0]).file_name() {
        invoked_as = file_name.to_string_lossy().to_string();
    }

    if invoked_as == SHADOWED_NAME {
        // `all_arguments[1..].to_vec()` copies everything after argv[0] into a new owned list.
        let exit_code: i32 = run_shim(all_arguments[1..].to_vec());
        std::process::exit(exit_code);
    }

    // The command-line tool wants plain text; clap would otherwise report invalid UTF-8 itself.
    let mut text_arguments: Vec<String> = Vec::new();
    for argument in all_arguments {
        text_arguments.push(argument.to_string_lossy().to_string());
    }
    let exit_code: i32 = run_cli(text_arguments);
    std::process::exit(exit_code);
}
