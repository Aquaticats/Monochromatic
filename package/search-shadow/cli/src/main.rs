//! What:
//! The program entry point.
//! It looks at the name it was started under:
//! a shadowed name (`rg` / `grep` / `find` and so on) means shadow mode;
//! anything else (`search-shadow`) means the command-line tool.
//! Why:
//! One binary serves both roles,
//! so the links created by `search-shadow link` need no second program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // main.ts
//! const invokedAs = path.basename(process.argv[1]);
//! process.exit(SHADOWED_NAMES.includes(invokedAs) ? runShim(invokedAs, process.argv.slice(2)) : runCli(process.argv));
//! ```

/// What:
/// `use search_shadow::...` imports items from this package's own library crate
/// (`search_shadow` is declared in `Cargo.toml`).
/// `cli::run_cli` is a function inside the `cli` module.
/// Why:
/// `main` only chooses the mode;
/// all logic lives in the library.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runCli } from './cli.ts';
/// ```
use search_shadow::cli::run_cli;

/// Imports the list of shadowed names.
use search_shadow::constants::SHADOWED_NAMES;

/// Imports the shadow-mode function.
use search_shadow::shim::run_shim;

/// Imports the owned operating-system text type.
use std::ffi::OsString;

/// Imports the process-environment module.
use std::env;

/// Imports `Path` to take the file name off argv[0].
use std::path::Path;

/// What:
/// `fn main()` is the entry point the operating system calls.
/// It returns nothing;
/// the exit code is set by `std::process::exit`.
/// Why:
/// Every run starts here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function main(): void { ... process.exit(code); }
/// ```
fn main() {
    // What:     `env::args_os().collect()` gathers every command-line argument as `OsString`s into a `Vec`; the
    // annotation names the list type so `collect` knows what to build.
    // Why:      Search patterns may be invalid UTF-8; `env::args()` would panic on them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const args: Buffer[] = process.argv;
    // ```
    let all_arguments: Vec<OsString> = env::args_os().collect();

    // What:     The name the program was started under: take argv[0], keep only its final path segment, and make it
    // text. `unwrap_or_default()` substitutes an empty text if any step has nothing.
    // Why:      A symlink named `rg` makes argv[0] end in `rg`, which selects shadow mode.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const invokedAs = path.basename(process.argv[0]);
    // ```
    let mut invoked_as: String = String::new();
    // `if let Some(file_name) = ...file_name()` runs the block when argv[0] has a final segment; `&all_arguments[0]`
    // lends the first element without copying it.
    if let Some(file_name) = Path::new(&all_arguments[0]).file_name() {
        invoked_as = file_name.to_string_lossy().to_string();
    }

    if SHADOWED_NAMES.contains(&invoked_as.as_str()) {
        // `all_arguments[1..].to_vec()` copies everything after argv[0] into a new owned list.
        let exit_code: i32 = run_shim(&invoked_as, all_arguments[1..].to_vec());
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
