//! What:
//! Runs the repository-wide catalog commands for the CLI:
//! install,
//! update.
//! Why:
//! They share one setup (repository root,
//! real cargo under the pinned toolchain,
//! crates.io registry) and one way of printing
//! a report and turning findings into an exit code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commands.ts
//! export async function main(args: string[]): Promise<void>
//! ```

/// What:
/// `use std::io::Write;` imports `Write` from the standard library so this file can name it unqualified.
/// Why:
/// `Write` is used by `print_install_report`,
/// `print_update_report`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::Write;
// Imports `std::path::PathBuf`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      `run_command` finds the repository root as a path and joins `rust-catalog.toml` onto it; the owned type lets it keep the root while building more paths.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// What:
/// `use std::path::PathBuf;` imports `PathBuf` from the standard library so this file can name it unqualified.
/// Why:
/// `PathBuf` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PathBuf } from 'std/path';
/// ```
use std::path::PathBuf;
/// What:
/// `use std::time::{SystemTime, UNIX_EPOCH};` imports `SystemTime`,
/// `UNIX_EPOCH` from the standard library so this file can name them unqualified.
/// Why:
/// `SystemTime` is used by `now_millis`;
/// `UNIX_EPOCH` is used by `now_millis`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SystemTime, UNIX_EPOCH } from 'std/time';
/// ```
use std::time::{SystemTime, UNIX_EPOCH};

/// What:
/// `use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};` imports `CatalogDocument`,
/// `parse_catalog` from the external crate `monochromatic_dependency_catalog` so this file can name them unqualified.
/// Why:
/// `CatalogDocument` is used by `run_command`;
/// `parse_catalog` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CatalogDocument, parse_catalog } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};

/// What:
/// `use crate::cargo_run::RealCargo;` imports `RealCargo` from another module of this crate so this file can name it unqualified.
/// Why:
/// `RealCargo` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RealCargo } from 'crate/cargo_run';
/// ```
use crate::cargo_run::RealCargo;
/// What:
/// `use crate::cli::Command;` imports `Command` from another module of this crate so this file can name it unqualified.
/// Why:
/// `Command` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command } from 'crate/cli';
/// ```
use crate::cli::Command;
/// What:
/// `use crate::constants::{CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE};` imports `CATALOG_MARKER`,
/// `INSTALL_FAILED_EXIT_CODE` from another module of this crate so this file can name them unqualified.
/// Why:
/// `CATALOG_MARKER` is used by `run_command`;
/// `INSTALL_FAILED_EXIT_CODE` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE } from 'crate/constants';
/// ```
use crate::constants::{CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE};
/// What:
/// `use crate::crates::locked_crate_dirs;` imports `locked_crate_dirs` from another module of this crate so this file can name it unqualified.
/// Why:
/// `locked_crate_dirs` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { locked_crate_dirs } from 'crate/crates';
/// ```
use crate::crates::locked_crate_dirs;
/// What:
/// `use crate::http::UreqFetcher;` imports `UreqFetcher` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UreqFetcher` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UreqFetcher } from 'crate/http';
/// ```
use crate::http::UreqFetcher;
/// What:
/// `use crate::install::{InstallReport, InstallRequest, install_rust_lockfiles, is_continuous_integration};` imports `InstallReport`,
/// `InstallRequest`,
/// `install_rust_lockfiles`,
/// `is_continuous_integration` from another module of this crate so this file can name them unqualified.
/// Why:
/// `InstallReport` is used by `print_install_report`;
/// `InstallRequest` is used by `run_command`;
/// `install_rust_lockfiles` is used by `run_command`;
/// `is_continuous_integration` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InstallReport, InstallRequest, install_rust_lockfiles, is_continuous_integration } from 'crate/install';
/// ```
use crate::install::{InstallReport, InstallRequest, install_rust_lockfiles, is_continuous_integration};
/// What:
/// `use crate::real_binary::find_real_cargo;` imports `find_real_cargo` from another module of this crate so this file can name it unqualified.
/// Why:
/// `find_real_cargo` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { find_real_cargo } from 'crate/real_binary';
/// ```
use crate::real_binary::find_real_cargo;
/// What:
/// `use crate::registry::CratesIo;` imports `CratesIo` from another module of this crate so this file can name it unqualified.
/// Why:
/// `CratesIo` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CratesIo } from 'crate/registry';
/// ```
use crate::registry::CratesIo;
/// What:
/// `use crate::shadow_step::trust_base_setup;` imports `trust_base_setup` from another module of this crate so this file can name it unqualified.
/// Why:
/// `trust_base_setup` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { trust_base_setup } from 'crate/shadow_step';
/// ```
use crate::shadow_step::trust_base_setup;
/// What:
/// `use crate::toolchain::cargo_toolchain;` imports `cargo_toolchain` from another module of this crate so this file can name it unqualified.
/// Why:
/// `cargo_toolchain` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { cargo_toolchain } from 'crate/toolchain';
/// ```
use crate::toolchain::cargo_toolchain;
/// What:
/// `use crate::tighten::FileEnforcer;` imports `FileEnforcer` from another module of this crate so this file can name it unqualified.
/// Why:
/// `FileEnforcer` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FileEnforcer } from 'crate/tighten';
/// ```
use crate::tighten::FileEnforcer;
/// What:
/// `use crate::update::{UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles};` imports `UpdateKind`,
/// `UpdateReport`,
/// `UpdateRequest`,
/// `update_rust_lockfiles` from another module of this crate so this file can name them unqualified.
/// Why:
/// `UpdateKind` is used by `run_command`;
/// `UpdateReport` is used by `print_update_report`;
/// `UpdateRequest` is used by `run_command`;
/// `update_rust_lockfiles` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles } from 'crate/update';
/// ```
use crate::update::{UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles};
/// What:
/// `use crate::workspace::find_catalog_root;` imports `find_catalog_root` from another module of this crate so this file can name it unqualified.
/// Why:
/// `find_catalog_root` is used by `run_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { find_catalog_root } from 'crate/workspace';
/// ```
use crate::workspace::find_catalog_root;
/// What:
/// `use crate::adapt::millis_of;` imports `millis_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `millis_of` is used by `now_millis`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { millis_of } from 'crate/adapt';
/// ```
use crate::adapt::millis_of;

/// Prints findings under a heading,
/// with the pointer to the settings.
pub fn print_findings(heading: &str, problems: &[String]) {
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      The findings go to standard error under a heading that names the refused step, one bullet per finding.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    eprintln!("{heading}: {} finding(s).", problems.len());
    for problem in problems {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("- {problem}");
    }
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("Settings live in rust-catalog.toml; see doc/decision/rust-dependency-catalog.md.");
}

/// Prints an install report.
pub fn print_install_report(report: &InstallReport, dry_run: bool, sink: &mut dyn Write) {
    // Macro `writeln!`: builds or prints text (or a list) from a template.
    // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
    // Why:      The report goes to a writer the caller supplies; a failed write to it is not a reason to fail a finished install, so the results are dropped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // void expr;   // or try { expr } catch {}
    // ```
    let _ = writeln!(sink, 
        "{} lockfiles, {} did not satisfy their manifests{}",
        report.crate_dirs.len(),
        report.stale.len(),
        if report.stale.is_empty() { "." } else { ":" }
    );
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      The report is only printed, so its fields are walked by reference.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    for crate_dir in &report.stale {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "  {crate_dir}");
    }
    if !report.added.is_empty() {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "{} versions no lockfile held before:", report.added.len());
    }
    // Borrow `&report.added`: lends the value without copying or giving it away.
    for added in &report.added {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "  {} {}", added.name, added.version);
    }
    // Borrow `&report.warnings`: lends the value without copying or giving it away.
    for warning in &report.warnings {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Warning: {warning}");
    }
    if dry_run {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Dry run: every Cargo.lock was restored.");
    } else if report.written {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Every Cargo.lock satisfies its manifests. Review the diff.");
    }
}

/// Prints an update report.
pub fn print_update_report(report: &UpdateReport, dry_run: bool, sink: &mut dyn Write) {
    // Macro `writeln!`: builds or prints text (or a list) from a template.
    // `let _ =`: discards the result on purpose.
    let _ = writeln!(sink, "{} lockfiles, {} newly locked versions:", report.crate_dirs.len(), report.added.len());
    // Borrow `&report.added`: lends the value without copying or giving it away.
    for added in &report.added {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "  {} {}", added.name, added.version);
    }
    // Borrow `&report.fell_back`: lends the value without copying or giving it away.
    for crate_dir in &report.fell_back {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Resolved {crate_dir} without the release-age cutoff (minimumReleaseAgeStrict = false, or crates exempt through minimumReleaseAgeExclude); crates that are not exempt are checked against it afterwards.");
    }
    // Borrow `&report.warnings`: lends the value without copying or giving it away.
    for warning in &report.warnings {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Warning: {warning}");
    }
    // Borrow `&report.pruned`: lends the value without copying or giving it away.
    for selector in &report.pruned {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Pruned {selector} from rust-catalog.toml.");
    }
    if dry_run {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Dry run: every Cargo.lock was restored.");
    } else if report.written {
        // Macro `writeln!`: builds or prints text (or a list) from a template.
        // `let _ =`: discards the result on purpose.
        let _ = writeln!(sink, "Kept the new lockfiles. Next: `mise run catalog:tighten`, then `mise run sync:files`, then `cargo-shadow install`.");
    }
}

/// Milliseconds since 1970 now.
fn now_millis() -> i64 {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      `now_millis` maps the elapsed duration to milliseconds with the named function `millis_of`; no closure is left in that line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A clock set before 1970 yields 0 so the release-age arithmetic stays defined instead of failing the command.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      `duration_since(...)` then `map` converts a successful reading to milliseconds and leaves the error for `unwrap_or`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // What:     `value as i64` converts a number to another numeric type (siblings: `i32`, `u32`, `u64`, `usize`, `f64`); it truncates silently if the value does not fit.
    // Why:      `millis_of` narrows the `u128` millisecond count of a `Duration` to `i64`, the type of every timestamp in the crate.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Number(value)  // JS has one number type, so no cast is needed
    // ```
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      `i64` milliseconds hold dates far beyond this century and subtract cleanly; the `u128` from `Duration` is converted once, in `millis_of`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    return SystemTime::now().duration_since(UNIX_EPOCH).map(millis_of).unwrap_or(0);
}

/// What:
/// `pub fn run_command(command: Command) -> i32` runs one catalog command from the repository that holds the working
/// directory and returns the exit code:
/// 0 on success,
/// 1 on a failure or a policy refusal.
/// Why:
/// Errors and findings go to standard error,
/// reports to standard output,
/// so a pipeline can read the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function runCommand(command: Command): Promise<number>
/// ```
pub fn run_command(command: Command) -> i32 {
    // The lean-folder tools need no catalog: they start from the checkout that holds the working directory.
    if let Command::Lean { action } = &command {
        return crate::lean_command::run_lean_action(action);
    }
    // Acknowledging a journal needs no catalog: it names the journal and the workspace follows from its path.
    if let Command::Acknowledge { journal } = &command {
        return match crate::acknowledge::acknowledge(journal) {
            Ok(message) => {
                println!("{message}");
                0
            }
            Err(reason) => {
                eprintln!("cargo-shadow: {reason}");
                INSTALL_FAILED_EXIT_CODE
            }
        };
    }
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      If the working directory cannot be read, the catalog cannot be found, so the command prints why and exits with the failure code.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      `current` is declared as `PathBuf` because the repository lookup walks up from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let current: PathBuf = match std::env::current_dir() {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      The `Ok`/`Err` arms hand the directory on or print why the command cannot start.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: no working directory: {error}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&current`: lends the value without copying or giving it away.
    // Typed binding: `root: PathBuf`.
    let root: PathBuf = match find_catalog_root(&current) {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      `find_catalog_root` returns `Some(root)` inside a repository with `rust-catalog.toml` and `None` elsewhere, where the CLI refuses with a message.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        Some(found) => found,
        // `None`: the present or the absent value of an `Option`.
        None => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: no {CATALOG_MARKER} above {}", current.display());
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `text: String`.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      The catalog text is read once as an owned `String` and handed to the parser.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let text: String = match std::fs::read_to_string(root.join(CATALOG_MARKER)) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: {CATALOG_MARKER}: {error}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&text`: lends the value without copying or giving it away.
    // Typed binding: `document: CatalogDocument`.
    let document: CatalogDocument = match parse_catalog(&text, CATALOG_MARKER) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("{error}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    // Borrow `&root`: lends the value without copying or giving it away.
    // Typed binding: `registry: CratesIo`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      `CratesIo::with_defaults` is given the repository root and two boxed `UreqFetcher`s, so the registry reads over HTTP through one pointer type each.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let registry: CratesIo = CratesIo::with_defaults(&root, Box::new(UreqFetcher::new()), Box::new(UreqFetcher::new()));
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `program: PathBuf`.
    let program: PathBuf = match find_real_cargo() {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `None`: the present or the absent value of an `Option`.
        None => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: no real cargo found on PATH");
            return crate::constants::NOT_FOUND_EXIT_CODE;
        }
    };
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `explicit: String`.
    let explicit: String = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_default();
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&root`: lends the value without copying or giving it away.
    // Typed binding: `toolchain: String`.
    let toolchain: String = match cargo_toolchain(&root, &explicit) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok((chosen, warning)) => {
            // `Some`: the present or the absent value of an `Option`.
            // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
            // Why:      A toolchain warning is printed once, before any cargo call; absent means there is nothing to say.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (value !== undefined) { const inner = value; ... }
            // ```
            if let Some(notice) = warning {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!("cargo-shadow: warning: {notice}");
            }
            chosen
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(message) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: toolchain.lock.json is not readable: {message}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    // Typed binding: `runner: RealCargo`.
    let runner: RealCargo = RealCargo { program, toolchain, bound: std::time::Duration::from_secs(crate::cargo_run::CARGO_CALL_SECONDS) };
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&root`: lends the value without copying or giving it away.
    // Typed binding: `crate_dirs: Vec<String>`.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      `crate_dirs` lists every locked crate root in the repository; the install or the update below borrows it for its whole run.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let crate_dirs: Vec<String> = match locked_crate_dirs(&root) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: {error}");
            return INSTALL_FAILED_EXIT_CODE;
        }
    };
    // `match`: picks the arm whose pattern fits the value.
    match command {
        Command::Install { frozen_lockfile, no_frozen_lockfile, dry_run } => {
            // Typed binding: `frozen: bool`.
            let frozen: bool = if frozen_lockfile {
                true
            } else if no_frozen_lockfile {
                false
            } else {
                // Borrow `&std`: lends the value without copying or giving it away.
                // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
                is_continuous_integration(&std::env::var("CI").unwrap_or_default())
            };
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&root`: lends the value without copying or giving it away.
            let (trust_base, reader) = trust_base_setup(&root, &document, is_continuous_integration(&std::env::var("CI").unwrap_or_default()));
            let outcome = install_rust_lockfiles(&InstallRequest {
                // Borrow `&root`: lends the value without copying or giving it away.
                root: &root,
                // Borrow `&crate_dirs`: lends the value without copying or giving it away.
                crate_dirs: &crate_dirs,
                frozen,
                // Borrow `&std`: lends the value without copying or giving it away.
                // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
                offline: !is_continuous_integration(&std::env::var("CI").unwrap_or_default()),
                dry_run,
                now: now_millis(),
                // Borrow `&document.settings`: lends the value without copying or giving it away.
                settings: &document.settings,
                // Borrow `&runner`: lends the value without copying or giving it away.
                runner: &runner,
                // Borrow `&registry`: lends the value without copying or giving it away.
                registry: &registry,
                lock_base: reader.as_deref(),
                trust_base: &trust_base,
                known_consistent: false,
            });
            // `match`: picks the arm whose pattern fits the value.
            return match outcome {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(report) => {
                    // Borrow `&report`: lends the value without copying or giving it away.
                    print_install_report(&report, dry_run, &mut std::io::stdout());
                    if report.problems.is_empty() {
                        0
                    } else {
                        // Borrow `&report.problems`: lends the value without copying or giving it away.
                        print_findings("Install refused; no Cargo.lock was left changed", &report.problems);
                        INSTALL_FAILED_EXIT_CODE
                    }
                }
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(error) => {
                    // Macro `eprintln!`: builds or prints text (or a list) from a template.
                    eprintln!("{error}");
                    INSTALL_FAILED_EXIT_CODE
                }
            };
        }
        Command::Update { dry_run } => {
            // Borrow `&root`: lends the value without copying or giving it away.
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            let (_update_trust_base, update_reader) = trust_base_setup(&root, &document, is_continuous_integration(&std::env::var("CI").unwrap_or_default()));
            let outcome = update_rust_lockfiles(
                // Borrow `&UpdateRequest`: lends the value without copying or giving it away.
                &UpdateRequest {
                    // Borrow `&root`: lends the value without copying or giving it away.
                    root: &root,
                    // Borrow `&crate_dirs`: lends the value without copying or giving it away.
                    crate_dirs: &crate_dirs,
                    dry_run,
                    now: now_millis(),
                    // Borrow `&document.settings`: lends the value without copying or giving it away.
                    settings: &document.settings,
                    // Borrow `&root.join`: lends the value without copying or giving it away.
                    catalog_path: &root.join(CATALOG_MARKER),
                    // Borrow `&runner`: lends the value without copying or giving it away.
                    runner: &runner,
                    // Borrow `&registry`: lends the value without copying or giving it away.
                    registry: &registry,
                    // Borrow `&[`: lends the value without copying or giving it away.
                    forward: &[],
                    // Borrow `&FileEnforcer`: lends the value without copying or giving it away.
                    syncer: &FileEnforcer,
                    prune_catalog: true,
                    lock_base: update_reader.as_deref(),
                },
                // Borrow `&UpdateKind`: lends the value without copying or giving it away.
                &UpdateKind::Full,
            );
            // `match`: picks the arm whose pattern fits the value.
            return match outcome {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(report) => {
                    // Borrow `&report`: lends the value without copying or giving it away.
                    print_update_report(&report, dry_run, &mut std::io::stdout());
                    if report.problems.is_empty() {
                        0
                    } else {
                        // Borrow `&report.problems`: lends the value without copying or giving it away.
                        print_findings("Update refused; every Cargo.lock was restored", &report.problems);
                        INSTALL_FAILED_EXIT_CODE
                    }
                }
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(error) => {
                    // Macro `eprintln!`: builds or prints text (or a list) from a template.
                    eprintln!("{error}");
                    INSTALL_FAILED_EXIT_CODE
                }
            };
        }
        // Handled before the catalog is read; reaching this arm means the early return above was removed.
        Command::Lean { .. } => {
            eprintln!("cargo-shadow: lean reached the catalog commands; this is a defect");
            return INSTALL_FAILED_EXIT_CODE;
        }
        // Handled before the catalog is read; reaching this arm means the early return above was removed.
        Command::Acknowledge { .. } => {
            eprintln!("cargo-shadow: acknowledge reached the catalog commands; this is a defect");
            return INSTALL_FAILED_EXIT_CODE;
        }
    }
}
