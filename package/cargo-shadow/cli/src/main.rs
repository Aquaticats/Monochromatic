//! What:
//! The program entry point.
//! It looks at the name it was started under:
//! `cargo` means shadow mode;
//! anything else
//! (`cargo-shadow`) means the command-line tool.
//! Why:
//! One binary serves both roles,
//! so the link the toolchain plan creates as `cargo` needs no second program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // main.ts
//! const invokedAs = path.basename(process.argv[1]);
//! process.exit(invokedAs === 'cargo' ? runShim(process.argv.slice(2)) : runCli(process.argv));
//! ```

/// Imports the functions of this package's own library crate (`cargo_shadow`,
/// declared in `Cargo.toml`).
use cargo_shadow::cli::run_cli;
/// Imports the name the shadow answers to.
use cargo_shadow::constants::{PLUGIN_WORD, SHADOWED_NAME};
/// Imports the shadow-mode function.
use cargo_shadow::shim::run_shim;

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
    // `env::args_os().collect()` gathers every argument as `OsString`s into a `Vec`; cargo arguments may be invalid
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
    // Cargo runs a `cargo-shadow` executable on PATH as `cargo shadow <args>`, passing the word `shadow` first; drop it so
    // `cargo shadow install` and `cargo-shadow install` mean the same.
    if text_arguments.len() > 1 && text_arguments[1] == PLUGIN_WORD {
        text_arguments.remove(1);
    }
    let exit_code: i32 = run_cli(text_arguments);
    std::process::exit(exit_code);
}
