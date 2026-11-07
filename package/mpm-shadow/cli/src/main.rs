//! What:
//! The program entry point.
//! It looks at the name it was started under:
//! `mpm` means shadow mode,
//! except that a first word that is one of pnpm's verbs (`mpm i -g`) runs the command-line tool;
//! anything
//!           else (`mpm-shadow`) means the command-line tool.
//! Why:
//! One binary serves both roles,
//! so the link the toolchain plan creates as `mpm` needs no second program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // main.ts
//! const invokedAs = path.basename(process.argv[1]);
//! process.exit(invokedAs === 'mpm' ? runShim(process.argv.slice(2)) : runCli(process.argv));
//! ```

/// What:
/// `use mpm_shadow::cli::run_cli;` imports a function from this package's own library crate (`mpm_shadow`,
///           declared in `Cargo.toml`).
/// Why:
/// `main` only chooses the mode;
/// all logic lives in the library.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runCli } from './cli.ts';
/// ```
use mpm_shadow::cli::run_cli;

/// Imports the check that a word is one of this tool's pnpm verbs or their aliases.
use mpm_shadow::cli::names_a_declared_verb;

/// Imports the name the shadow answers to.
use mpm_shadow::constants::SHADOWED_NAME;

/// Imports the launcher mode.
use mpm_shadow::toolchain::launcher::{launch_record_of_current_exe, run_launcher};

/// Imports the shadow-mode function.
use mpm_shadow::shim::run_shim;

/// Imports the owned operating-system text type.
use std::ffi::OsString;

/// Imports the process-environment module.
use std::env;

/// Imports `Path` to take the file name off argv[0].
use std::path::Path;

/// What:
/// `fn main()` is the entry point the operating system calls;
/// the exit code is set by `std::process::exit`.
/// Why:
/// Every run starts here.
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

    // A hard link the toolchain made for a repository-published tool carries a record beside it: run as a launcher.
    if let Some(record) = launch_record_of_current_exe() {
        std::process::exit(run_launcher(&record, &all_arguments));
    }

    if invoked_as == SHADOWED_NAME {
        // What:     `all_arguments.get(1)` is the first word after argv[0] as an `Option` (absent when there is
        //           none); `.and_then(|word| return word.to_str())` keeps it only when it is valid UTF-8 text.
        //           The `if let Some(first) = ... && names_a_declared_verb(first)` runs the block only when that
        //           word is present and is one of pnpm's verbs or aliases (`install`, `i`, `update`, `up`, ...).
        // Why:      The owner wants pnpm's verbs on `mpm` itself ("I want pnpm style verbs.", 2026-11-03), so
        //           `mpm i -g` is this tool's global install, the same as `mpm-shadow i -g`. Any other command
        //           line, including one that starts with mpm's own options, stays the real mpm's.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const first = process.argv[2];
        // if (first !== undefined && namesADeclaredVerb(first)) process.exit(runCli(textOf(process.argv.slice(1))));
        // ```
        if let Some(first) = all_arguments.get(1).map(std::ffi::OsString::as_os_str).and_then(std::ffi::OsStr::to_str)
            && names_a_declared_verb(first)
        {
            // argv[0] stays `mpm`, so the usage lines say `mpm install`, the command the person typed.
            std::process::exit(run_cli(text_of(all_arguments)));
        }
        // `all_arguments[1..].to_vec()` copies everything after argv[0] into a new owned list.
        let exit_code: i32 = run_shim(all_arguments[1..].to_vec());
        std::process::exit(exit_code);
    }

    let exit_code: i32 = run_cli(text_of(all_arguments));
    std::process::exit(exit_code);
}

/// What:
/// `fn text_of(arguments: Vec<OsString>) -> Vec<String>` declares a private function that turns the raw
///           arguments (`OsString`,
/// the operating system's bytes) into owned texts (`String`),
///           replacing bytes that are not UTF-8 with the replacement character.
/// Why:
/// The command-line tool wants plain text;
/// clap would otherwise report invalid UTF-8 itself.
///           Both the `mpm-shadow` start and an `mpm` start with a pnpm verb need it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function textOf(args: Buffer[]): string[] { return args.map((argument) => argument.toString('utf8')); }
/// ```
fn text_of(arguments: Vec<OsString>) -> Vec<String> {
    let mut text_arguments: Vec<String> = Vec::new();
    for argument in arguments {
        // `.to_string_lossy().to_string()` reads the bytes as UTF-8 and copies them into an owned `String`.
        text_arguments.push(argument.to_string_lossy().to_string());
    }
    return text_arguments;
}
