//! What:
//! Runs the repository-wide catalog commands for the CLI: install, update.
//! Why:
//! They share one setup (repository root, real cargo under the pinned toolchain, crates.io registry) and one way of printing
//! a report and turning findings into an exit code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commands.ts
//! export async function main(args: string[]): Promise<void>
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::io::Write;
// Imports `std::path::PathBuf`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::path::PathBuf;
// Imports `std::time::{SystemTime, UNIX_EPOCH}`.
use std::time::{SystemTime, UNIX_EPOCH};

// Imports `monochromatic_dependency_catalog::{CatalogD...`.
use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};

// Imports `crate::cargo_run::RealCargo`.
use crate::cargo_run::RealCargo;
// Imports `crate::cli::Command`.
use crate::cli::Command;
// Imports `crate::constants::{CATALOG_MARKER, INSTALL_...`.
use crate::constants::{CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE};
// Imports `crate::crates::locked_crate_dirs`.
use crate::crates::locked_crate_dirs;
// Imports `crate::http::UreqFetcher`.
use crate::http::UreqFetcher;
// Imports `crate::install::{InstallReport, InstallRequ...`.
use crate::install::{InstallReport, InstallRequest, install_rust_lockfiles, is_continuous_integration};
// Imports `crate::real_binary::find_real_cargo`.
use crate::real_binary::find_real_cargo;
// Imports `crate::registry::CratesIo`.
use crate::registry::CratesIo;
// Imports `crate::shadow_step::lock_base_reader`.
use crate::shadow_step::lock_base_reader;
// Imports `crate::toolchain::cargo_toolchain`.
use crate::toolchain::cargo_toolchain;
// Imports `crate::tighten::FileEnforcer`.
use crate::tighten::FileEnforcer;
// Imports `crate::update::{UpdateKind, UpdateReport, U...`.
use crate::update::{UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles};
// Imports `crate::workspace::find_catalog_root`.
use crate::workspace::find_catalog_root;

/// Prints findings under a heading, with the pointer to the settings.
pub fn print_findings(heading: &str, problems: &[String]) {
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
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
    // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
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
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
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
        let _ = writeln!(sink, "Resolved {crate_dir} without the release-age cutoff (minimumReleaseAgeStrict = false).");
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
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // What:     `value as i64` converts a number to another numeric type (siblings: `i32`, `u32`, `u64`, `usize`, `f64`); it truncates silently if the value does not fit.
    // Why:      Time arithmetic and sizes come in different integer widths and must be made equal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Number(value)  // JS has one number type, so no cast is needed
    // ```
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    return SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| { return elapsed.as_millis() as i64; }).unwrap_or(0);
}

/// What:
/// `pub fn run_command(command: Command) -> i32` runs one catalog command from the repository that holds the working
/// directory and returns the exit code: 0 on success, 1 on a failure or a policy refusal.
/// Why:
/// Errors and findings go to standard error, reports to standard output, so a pipeline can read the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function runCommand(command: Command): Promise<number>
/// ```
pub fn run_command(command: Command) -> i32 {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let current: PathBuf = match std::env::current_dir() {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
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
        // Why:      Absence is a value the caller must check; there is no `null`.
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
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
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
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
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
            // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
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
    let runner: RealCargo = RealCargo { program, toolchain };
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&root`: lends the value without copying or giving it away.
    // Typed binding: `crate_dirs: Vec<String>`.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
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
            let reader = match lock_base_reader(&root, &document, frozen) {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(found) => found,
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(message) => {
                    // Macro `eprintln!`: builds or prints text (or a list) from a template.
                    eprintln!("cargo-shadow: {message}");
                    return INSTALL_FAILED_EXIT_CODE;
                }
            };
            // Borrow `&InstallRequest`: lends the value without copying or giving it away.
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
    }
}
