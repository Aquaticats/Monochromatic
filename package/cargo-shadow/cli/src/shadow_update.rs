//! What:
//! The shadow's answer to `cargo update` and `cargo generate-lockfile`:
//! the pnpm command each stands for,
//! run with the
//! release-age and trust policy applied while versions are chosen.
//! Why:
//! The owner's rule (2026-11-03):
//! follow pnpm semantics.
//! `cargo update` is `pnpm update` and `cargo generate-lockfile` is
//! `pnpm install --lockfile-only`;
//! `update_plan.rs` documents the flag mapping.
//! Outside a catalog repository the command goes to
//! cargo untouched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-update.ts
//! export function runUpdateCommand(args: string[]): number | undefined  // undefined: run cargo itself
//! ```

// What:     `OsString` is operating-system text that need not be valid UTF-8 (sibling: `String`, always valid UTF-8).
// Why:      Command-line arguments pass through byte for byte, so they are kept as the OS gave them.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// What:
/// `use std::ffi::OsString;` imports `OsString` from the standard library so this file can name it unqualified.
/// Why:
/// `OsString` is used by `run_update_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { OsString } from 'std/ffi';
/// ```
use std::ffi::OsString;
/// What:
/// `use std::io::Write;` imports `Write` from the standard library so this file can name it unqualified.
/// Why:
/// `Write` is used by `report_install`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::Write;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `run_update_command`,
/// `check_direct`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;

/// What:
/// `use self::run::{run_full_update, run_update};` imports the two ways an update runs,
/// from the sibling file that holds them.
/// Why:
/// The command below picks one of them for the plan it was given,
/// and the run itself lives in the `run` part of this module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runFullUpdate, runUpdate } from 'crate/shadow_update/run';
/// ```
use self::run::{
    run_full_update,
    run_update,
};
/// What:
/// `use crate::adapt::text_of;` imports `text_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `text_of` is used by `check_direct`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { text_of } from 'crate/adapt';
/// ```
use crate::adapt::text_of;
/// What:
/// `use crate::commands::{print_findings, print_install_report};` imports `print_findings`,
/// `print_install_report` from another module of this crate so this file can name them unqualified.
/// Why:
/// `print_findings` is used by `report_install`;
/// `print_install_report` is used by `report_install`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { print_findings, print_install_report } from 'crate/commands';
/// ```
use crate::commands::{
    print_findings,
    print_install_report,
};
/// What:
/// `use crate::constants::{INSTALL_FAILED_EXIT_CODE, LOCKFILE_NAME};` imports `INSTALL_FAILED_EXIT_CODE`,
/// `LOCKFILE_NAME` from another module of this crate so this file can name them unqualified.
/// Why:
/// `INSTALL_FAILED_EXIT_CODE` is used by `run_update_command`,
/// `report_install`;
/// `LOCKFILE_NAME` is used by `run_update_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { INSTALL_FAILED_EXIT_CODE, LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::{
    INSTALL_FAILED_EXIT_CODE,
    LOCKFILE_NAME,
};
/// What:
/// `use crate::crates::LockSnapshot;` imports `LockSnapshot` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LockSnapshot` is used by `check_direct`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot } from 'crate/crates';
/// ```
use crate::crates::LockSnapshot;
/// What:
/// `use crate::crates::read_locks;` imports `read_locks` from another module of this crate so this file can name it unqualified.
/// Why:
/// `read_locks` is used by `check_direct`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { read_locks } from 'crate/crates';
/// ```
use crate::crates::read_locks;
/// What:
/// `use crate::install::{InstallReport, InstallRequest, install_rust_lockfiles};` imports `InstallReport`,
/// `InstallRequest`,
/// `install_rust_lockfiles` from another module of this crate so this file can name them unqualified.
/// Why:
/// `InstallReport` is used by `report_install`;
/// `InstallRequest` is used by `run_update_command`;
/// `install_rust_lockfiles` is used by `run_update_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InstallReport, InstallRequest, install_rust_lockfiles } from 'crate/install';
/// ```
use crate::install::{
    InstallReport,
    InstallRequest,
    install_rust_lockfiles,
};
/// What:
/// `use crate::lockfile::direct_dependency_names;` imports `direct_dependency_names` from another module of this crate so this file can name it unqualified.
/// Why:
/// `direct_dependency_names` is used by `check_direct`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { direct_dependency_names } from 'crate/lockfile';
/// ```
use crate::lockfile::direct_dependency_names;
/// What:
/// `use crate::shadow_step::{Prepared, now_millis, prepare};` imports `Prepared`,
/// `now_millis`,
/// `prepare` from another module of this crate so this file can name them unqualified.
/// Why:
/// `Prepared` is used by `run_update_command`;
/// `now_millis` is used by `run_update_command`;
/// `prepare` is used by `run_update_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Prepared, now_millis, prepare } from 'crate/shadow_step';
/// ```
use crate::shadow_step::{
    Prepared,
    now_millis,
    prepare,
};
/// What:
/// `use crate::update::UpdateKind;` imports `UpdateKind` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UpdateKind` is used by `run_update_command`,
/// which asks for a targeted update by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateKind } from 'crate/update';
/// ```
use crate::update::UpdateKind;
/// What:
/// `use crate::update_plan::{UpdatePlan, plan_update};` imports `UpdatePlan`,
/// `plan_update` from another module of this crate so this file can name them unqualified.
/// Why:
/// `UpdatePlan` is used by `run_update_command`;
/// `plan_update` is used by `run_update_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdatePlan, plan_update } from 'crate/update_plan';
/// ```
use crate::update_plan::{
    UpdatePlan,
    plan_update,
};

/// Running one update of either kind,
/// and the stand-in lockfile a full update writes (split out of this module).
mod run;

/// The stand-in lockfile a full update writes,
/// and the work that runs while it is in place (split out of the shadow update module).
pub use self::run::{
    PlaceholderBody,
    run_with_placeholder_lock,
    run_with_placeholder_lock_hooked,
};

/// What:
/// `pub fn run_update_command(arguments: &[OsString]) -> Option<i32>` runs the command and returns its exit code,
/// or `None`
/// when it is not in a catalog repository (the caller then runs cargo itself).
/// Why:
/// Every outcome prints to standard error,
/// as cargo's own `update` does,
/// so a pipeline reading standard output sees nothing
/// new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runUpdateCommand(args: string[]): number | undefined
/// ```
pub fn run_update_command(arguments: &[OsString]) -> Option<i32> {
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
    let prepared: Prepared = match prepare(arguments) {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        Ok(Some(found)) => found,
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        Ok(None) => return None,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(message) => {
            // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
            // Why:      Text building and printing are written as one call instead of string concatenation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // `..${x}..`   // template literal
            // console.error(text);   // eprintln!
            // [a, b]            // vec![a, b]
            // ```
            eprintln!("{message}");
            // `Some`: the present or the absent value of an `Option`.
            return Some(INSTALL_FAILED_EXIT_CODE);
        }
    };
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // Typed binding: `crate_dir: String`.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let crate_dir: String = prepared.crate_dir.clone()?;
    // `match`: picks the arm whose pattern fits the value.
    let planned = match plan_update(arguments) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(message) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: {message}");
            // `Some`: the present or the absent value of an `Option`.
            return Some(INSTALL_FAILED_EXIT_CODE);
        }
    };
    // `.clone()`: an independent owned copy.
    // Typed binding: `crate_dirs: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let crate_dirs: Vec<String> = vec![crate_dir.clone()];
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // Typed binding: `root: &Path`.
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let root: &Path = &prepared.catalog_root;
    // Borrow `&crate_dir`: lends the value without copying or giving it away.
    // Typed binding: `lock_exists: bool`.
    let lock_exists: bool = root.join(&crate_dir).join(LOCKFILE_NAME).is_file();
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    let mut sink = std::io::stderr();
    // `match`: picks the arm whose pattern fits the value.
    match planned.plan {
        UpdatePlan::FrozenCheck
        | UpdatePlan::Install {
            ..
        } if lock_exists => {
            // `match`: picks the arm whose pattern fits the value.
            let (frozen, dry_run) = match planned.plan {
                UpdatePlan::Install {
                    dry_run,
                } => (prepared.frozen && !dry_run, dry_run),
                _ => (true, false),
            };
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&prepared.document`: lends the value without copying or giving it away.
            let outcome = install_rust_lockfiles(&InstallRequest {
                root,
                // Borrow `&crate_dirs`: lends the value without copying or giving it away.
                crate_dirs: &crate_dirs,
                frozen,
                offline: prepared.offline,
                dry_run,
                now: now_millis(),
                // Borrow `&prepared.document.settings`: lends the value without copying or giving it away.
                settings: &prepared.document.settings,
                // Borrow `&prepared.runner`: lends the value without copying or giving it away.
                runner: &prepared.runner,
                // Borrow `&prepared.registry`: lends the value without copying or giving it away.
                registry: &prepared.registry,
                lock_base: prepared.lock_base.as_deref(),
                trust_base: &prepared.trust_base,
                known_consistent: false,
            });
            // `Some`: the present or the absent value of an `Option`.
            // Borrow `&mut sink`: lends the value without copying or giving it away.
            return Some(report_install(outcome, dry_run, &mut sink));
        }
        UpdatePlan::Targeted {
            dry_run,
            cargo_args,
            names,
            precise,
        } => {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Borrow `&crate_dir`: lends the value without copying or giving it away.
            if precise && let Err(message) = check_direct(root, &crate_dir, &names) {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!("cargo-shadow: {message}");
                // `Some`: the present or the absent value of an `Option`.
                return Some(INSTALL_FAILED_EXIT_CODE);
            }
            // `Some`: the present or the absent value of an `Option`.
            // Borrow `&prepared`: lends the value without copying or giving it away.
            return Some(run_update(
                &prepared,
                &crate_dirs,
                dry_run,
                &UpdateKind::Targeted {
                    cargo_args: &cargo_args,
                },
                &planned.forward,
                &mut sink,
            ));
        }
        UpdatePlan::Full {
            dry_run,
        } => {
            // `Some`: the present or the absent value of an `Option`.
            // Borrow `&prepared`: lends the value without copying or giving it away.
            return Some(run_full_update(
                &prepared,
                &crate_dirs,
                dry_run,
                &planned.forward,
                &mut sink,
            ));
        }
        UpdatePlan::Install {
            dry_run,
        } => {
            // A missing lockfile: pnpm install --lockfile-only creates it, and here the full update does, under the policy.
            return Some(run_full_update(
                &prepared,
                &crate_dirs,
                dry_run,
                &planned.forward,
                &mut sink,
            ));
        }
        UpdatePlan::FrozenCheck => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: {crate_dir} has no {LOCKFILE_NAME} to assert about");
            // `Some`: the present or the absent value of an `Option`.
            return Some(INSTALL_FAILED_EXIT_CODE);
        }
    }
}

/// What:
/// `fn check_direct(root, crate_dir, names) -> Result<(), String>` applies pnpm's rule for `pnpm update <name>@<version>`:
/// the name must be a direct dependency.
/// `Result<(), String>` is success with no value or an error text.
/// Why:
/// pnpm fails with `ERR_PNPM_UPDATE_VERSION_ON_INDIRECT_DEP` for an indirect one;
/// the shadow refuses `--precise` the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkDirect(root: string, crateDir: string, names: string[]): void  // throws
/// ```
fn check_direct(
    root: &Path,
    crate_dir: &str,
    names: &[String],
) -> Result<(), String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // Borrow `&[`: lends the value without copying or giving it away.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    let snapshots = read_locks(root, &[crate_dir.to_string()]).map_err(text_of)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|snapshot|`: an anonymous function, like a TS arrow function.
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
    let direct = direct_dependency_names(
        snapshots
            .first()
            .map(LockSnapshot::source_str)
            .unwrap_or(""),
    )?;
    for name in names {
        if !direct.contains(name) {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!(
                "{name} is not a direct dependency of {crate_dir}, so --precise cannot pin it (pnpm: ERR_PNPM_UPDATE_VERSION_ON_INDIRECT_DEP, which names overrides for indirect dependencies; use [patch] or a direct requirement in Cargo.toml)"
            ));
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// What:
/// `fn report_install(outcome, dry_run, sink) -> i32` prints an install outcome to the writer and returns the exit code.
/// `&mut dyn Write` is a borrowed writer of any type (a TS `Writable`).
/// Why:
/// Standard error is the sink,
/// so cargo's standard output stays free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportInstall(outcome, dryRun: boolean, sink: Writable): number
/// ```
fn report_install(
    outcome: Result<InstallReport, crate::install::InstallError>,
    dry_run: bool,
    sink: &mut dyn Write,
) -> i32 {
    // `match`: picks the arm whose pattern fits the value.
    return match outcome {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(report) => {
            // Borrow `&report`: lends the value without copying or giving it away.
            print_install_report(&report, dry_run, sink);
            if report.problems.is_empty() {
                0
            } else {
                // Borrow `&report.problems`: lends the value without copying or giving it away.
                print_findings(
                    "Install refused; no Cargo.lock was left changed",
                    &report.problems,
                );
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

/// What:
/// `#[cfg(test)] mod shadow_update_tests;` declares the tests in `src/shadow_update_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the stand-in lockfile that a full update of a root without a lockfile uses."
/// so the behavior of `shadow_update` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // shadow_update_tests.test.ts, run by the test runner only
/// ```
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "shadow_update_tests.rs"]
mod shadow_update_tests;
