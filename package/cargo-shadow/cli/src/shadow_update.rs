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
/// `Write` is used by `report_install`,
/// `run_full_update`,
/// `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::Write;
/// Imports the exclusive rename the stand-in is taken with.
use rustix::fs::{CWD, RenameFlags, renameat_with};
/// Imports the error code of a name that exists or a path that does not.
use rustix::io::Errno;
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
/// `use crate::commands::{print_findings, print_install_report, print_update_report};` imports `print_findings`,
/// `print_install_report`,
/// `print_update_report` from another module of this crate so this file can name them unqualified.
/// Why:
/// `print_findings` is used by `report_install`,
/// `run_update`;
/// `print_install_report` is used by `report_install`;
/// `print_update_report` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { print_findings, print_install_report, print_update_report } from 'crate/commands';
/// ```
use crate::commands::{print_findings, print_install_report, print_update_report};
/// What:
/// `use crate::constants::{CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE, LOCKFILE_NAME};` imports `CATALOG_MARKER`,
/// `INSTALL_FAILED_EXIT_CODE`,
/// `LOCKFILE_NAME` from another module of this crate so this file can name them unqualified.
/// Why:
/// `CATALOG_MARKER` is used by `run_update`;
/// `INSTALL_FAILED_EXIT_CODE` is used by `run_update_command`,
/// `report_install`,
/// `run_update`;
/// `LOCKFILE_NAME` is used by `run_update_command`,
/// `run_full_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE, LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::{CATALOG_MARKER, INSTALL_FAILED_EXIT_CODE, LOCKFILE_NAME};
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
use crate::install::{InstallReport, InstallRequest, install_rust_lockfiles};
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
/// `Prepared` is used by `run_update_command`,
/// `run_full_update`,
/// `run_update`;
/// `now_millis` is used by `run_update_command`,
/// `run_update`;
/// `prepare` is used by `run_update_command`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Prepared, now_millis, prepare } from 'crate/shadow_step';
/// ```
use crate::shadow_step::{Prepared, now_millis, prepare};
/// What:
/// `use crate::tighten::FileEnforcer;` imports `FileEnforcer` from another module of this crate so this file can name it unqualified.
/// Why:
/// `FileEnforcer` is used by `run_update`.
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
/// `UpdateKind` is used by `run_update_command`,
/// `run_full_update`,
/// `run_update`;
/// `UpdateReport` is used by `run_update`;
/// `UpdateRequest` is used by `run_update`;
/// `update_rust_lockfiles` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles } from 'crate/update';
/// ```
use crate::update::{UpdateKind, UpdateReport, UpdateRequest, update_rust_lockfiles};
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
use crate::update_plan::{UpdatePlan, plan_update};
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
/// `use crate::crates::LockSnapshot;` imports `LockSnapshot` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LockSnapshot` is used by `check_direct`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot } from 'crate/crates';
/// ```
use crate::crates::LockSnapshot;

/// The lockfile text that stands in for a missing lockfile while a full update creates it.
const EMPTY_LOCK: &str = "version = 4\n";

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
        UpdatePlan::FrozenCheck | UpdatePlan::Install { .. } if lock_exists => {
            // `match`: picks the arm whose pattern fits the value.
            let (frozen, dry_run) = match planned.plan {
                UpdatePlan::Install { dry_run } => (prepared.frozen && !dry_run, dry_run),
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
        UpdatePlan::Targeted { dry_run, cargo_args, names, precise } => {
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
            return Some(run_update(&prepared, &crate_dirs, dry_run, &UpdateKind::Targeted { cargo_args: &cargo_args }, &planned.forward, &mut sink));
        }
        UpdatePlan::Full { dry_run } => {
            // `Some`: the present or the absent value of an `Option`.
            // Borrow `&prepared`: lends the value without copying or giving it away.
            return Some(run_full_update(&prepared, &crate_dirs, dry_run, &planned.forward, &mut sink));
        }
        UpdatePlan::Install { dry_run } => {
            // A missing lockfile: pnpm install --lockfile-only creates it, and here the full update does, under the policy.
            return Some(run_full_update(&prepared, &crate_dirs, dry_run, &planned.forward, &mut sink));
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
fn check_direct(root: &Path, crate_dir: &str, names: &[String]) -> Result<(), String> {
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
    let direct = direct_dependency_names(snapshots.first().map(LockSnapshot::source_str).unwrap_or(""))?;
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
fn report_install(outcome: Result<InstallReport, crate::install::InstallError>, dry_run: bool, sink: &mut dyn Write) -> i32 {
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

/// What:
/// `fn run_full_update(prepared, crate_dirs, dry_run, forward, sink) -> i32` runs the full catalog update for one crate root.
/// Why:
/// When no lockfile exists a stand-in is written so the transaction has a snapshot,
/// and it is removed again if the update is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runFullUpdate(args): number
/// ```
fn run_full_update(prepared: &Prepared, crate_dirs: &[String], dry_run: bool, forward: &[String], sink: &mut dyn Write) -> i32 {
    let lock_path = prepared.catalog_root.join(&crate_dirs[0]).join(LOCKFILE_NAME);
    // Borrow `&lock_path`: lends the value without copying or giving it away.
    return run_with_placeholder_lock(&lock_path, &prepared.catalog_root.join(crate::journal::QUARANTINE_DIR), dry_run, &mut FullUpdate { prepared, crate_dirs, dry_run, forward, sink });
}

/// What:
/// `pub trait PlaceholderBody` is the work that runs while a stand-in lockfile may be in place:
/// `run(&mut self)` does it and returns the exit status.
/// Why:
/// `run_with_placeholder_lock` takes the work as a value instead of a closure,
/// so the work can be a struct that holds what it needs.
/// Every `FnMut() -> i32` is a `PlaceholderBody` too,
/// so a test passes a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface PlaceholderBody { run(): number; }
/// ```
pub trait PlaceholderBody {
    /// What:
    /// `run(&mut self)` does the work once.
    /// Why:
    /// The exit status decides whether the stand-in is kept or removed.
    fn run(&mut self) -> i32;
}

/// What:
/// The implementation of `PlaceholderBody` for every function that takes nothing and returns an exit status.
/// Why:
/// Tests and callers that already hold a closure need no wrapper.
impl<F: FnMut() -> i32> PlaceholderBody for F {
    /// What:
    /// Calls the function.
    /// Why:
    /// The function is the work.
    fn run(&mut self) -> i32 {
        return self();
    }
}

/// What:
/// `struct FullUpdate<'a>` is the work of one full update of the crate roots.
/// Why:
/// It holds what the closure that stood in `run_full_update` captured.
struct FullUpdate<'a> {
    /// The prepared catalog and its settings.
    prepared: &'a Prepared,
    /// The crate roots to update.
    crate_dirs: &'a [String],
    /// Whether the update is a dry run.
    dry_run: bool,
    /// The arguments forwarded to cargo.
    forward: &'a [String],
    /// Where the update writes its report.
    sink: &'a mut dyn Write,
}

/// What:
/// The implementation of `PlaceholderBody` for `FullUpdate`.
/// Why:
/// `run_with_placeholder_lock` runs it with the stand-in lockfile in place.
impl PlaceholderBody for FullUpdate<'_> {
    /// What:
    /// Runs the full update.
    /// Why:
    /// The exit status is the update's.
    fn run(&mut self) -> i32 {
        return run_update(self.prepared, self.crate_dirs, self.dry_run, &UpdateKind::Full, self.forward, self.sink);
    }
}

/// What:
/// `pub fn run_with_placeholder_lock(lock_path, dry_run, body) -> i32` runs `body` with a stand-in lockfile in place when the
/// crate root has none,
/// and removes the stand-in again if the update is refused or was a dry run.
/// Why:
/// The transaction needs a lockfile to snapshot;
/// a root without one is created by the update itself.
/// The stand-in is created exclusively (`create_new`),
/// so a lockfile another writer published first is never truncated.
/// At the end it is taken by an atomic rename to a unique name and only then read:
/// a stand-in is deleted,
/// anything else is linked back to the path (or kept under the taken name when the path was
/// taken again),
/// so a lockfile another writer published during the update is never deleted.
/// The path is briefly empty between the take and the link back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runWithPlaceholderLock(lockPath: string, dryRun: boolean, body: () => number): number
/// ```
pub fn run_with_placeholder_lock(lock_path: &Path, keep_dir: &Path, dry_run: bool, body: &mut dyn PlaceholderBody) -> i32 {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    return run_with_placeholder_lock_hooked(lock_path, keep_dir, dry_run, body, &crate::journal_replace::no_hook);
}

/// What:
/// `fn remove_placeholder(lock_path: &Path, keep_dir: &Path)` takes the stand-in lockfile away from its path without ever
/// unlinking an inode that something may still have open.
/// Why:
/// Review finding (3c P1-3):
/// reading the taken file and then unlinking it still loses the data of a writer that opened the
/// stand-in earlier and writes after the read.
/// The file is taken by an exclusive rename to a unique name (an existing name is never overwritten),
/// read,
/// and then:
/// a stand-in is given a surviving name in `keep_dir` before the taken name is removed;
/// anything else is linked back to the path (a link never replaces something published meanwhile),
/// or stays under its taken
/// name,
/// reported on stderr.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removePlaceholder(lockPath: string, keepDir: string): void  // rename (no replace), read, keep, then unlink the taken name
/// ```
fn remove_placeholder(lock_path: &Path, keep_dir: &Path) {
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `taken: Option<std::path::PathBuf>`.
    let mut taken_name: Option<std::path::PathBuf> = None;
    for _attempt in 0..MAX_TAKEN_NAMES {
        // `let mut`: a binding that may change.
        // Typed binding: `name: std::ffi::OsString`.
        // What:     `OsString` is operating-system text that need not be valid UTF-8 (sibling: `String`, always valid UTF-8).
        // Why:      Command-line arguments pass through byte for byte, so they are kept as the OS gave them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let mut name: std::ffi::OsString = lock_path.as_os_str().to_os_string();
        // Macro `format!`: builds or prints text (or a list) from a template.
        name.push(format!(".taken.{}", crate::journal::unique_suffix()));
        // Typed binding: `candidate: std::path::PathBuf`.
        let candidate: std::path::PathBuf = std::path::PathBuf::from(name);
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&candidate`: lends the value without copying or giving it away.
        match renameat_with(CWD, lock_path, CWD, &candidate, RenameFlags::NOREPLACE) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(()) => {
                // `Some`: the present or the absent value of an `Option`.
                taken_name = Some(candidate);
                break;
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(code) if code == Errno::EXIST => continue,
            // Nothing is at the path any more: nothing to remove.
            Err(code) if code == Errno::NOENT => return,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(code) => {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!("cargo-shadow: warning: the stand-in {} could not be taken for removal: {code}", lock_path.display());
                return;
            }
        }
    }
    // `Some`: the present or the absent value of an `Option`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    let Some(taken) = taken_name else {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: no free name to take the stand-in {} under; it was left in place", lock_path.display());
        return;
    };
    let is_placeholder: bool = match crate::file_owner::observe(&taken) {
        Ok(Some(seen)) => seen.text == EMPTY_LOCK,
        Ok(None) => return,
        Err(error) => {
            eprintln!("cargo-shadow: warning: {} could not be read, so it is kept: {error}", taken.display());
            false
        }
    };
    // One rule for both branches (review finding 4c F3): the taken inode gets a surviving name in the kept directory before its
    // taken name is removed; linking it back to the live path is not a preservation name, because the path is writable by others.
    let keep: crate::journal::KeepAs = crate::journal::KeepAs::in_directory(keep_dir, if is_placeholder { ".placeholder" } else { ".lockfile" });
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&taken`: lends the value without copying or giving it away.
    let kept = match crate::journal_replace::keep_name(&taken, lock_path, &keep) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: warning: the lockfile taken from {} is kept as {} because it could not be given a kept name ({error})", lock_path.display(), taken.display());
            if !is_placeholder {
                // Borrow `&taken`: lends the value without copying or giving it away.
                // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
                // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // void expr;   // or try { expr } catch {}
                // ```
                let _ = std::fs::hard_link(&taken, lock_path);
            }
            return;
        }
    };
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&taken`: lends the value without copying or giving it away.
    if !is_placeholder && let Err(error) = std::fs::hard_link(&taken, lock_path) {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: a lockfile written during the run is kept as {} ({error})", kept.display());
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&taken`: lends the value without copying or giving it away.
    if let Err(error) = std::fs::remove_file(&taken) {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: {} could not be removed after it was kept as {}: {error}", taken.display(), kept.display());
    }
}

/// The most names tried to take the stand-in under.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
/// Like the other name searches:
/// one try is the normal case and the collision count was not counted.
const MAX_TAKEN_NAMES: u32 = 100;

/// What:
/// `pub fn run_with_placeholder_lock_hooked(lock_path, dry_run, body, before_cleanup) -> i32` is `run_with_placeholder_lock` with a
/// hook that runs just before the cleanup takes the file.
/// Why:
/// That gap is where another writer can publish a real lockfile;
/// the hook lets a test publish one exactly there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runWithPlaceholderLockHooked(lockPath: string, dryRun: boolean, body: () => number, beforeCleanup: (path: string) => void): number
/// ```
pub fn run_with_placeholder_lock_hooked(lock_path: &Path, keep_dir: &Path, dry_run: bool, body: &mut dyn PlaceholderBody, before_cleanup: &dyn Fn(&Path)) -> i32 {
    // `create_new(true)` fails with `AlreadyExists` instead of truncating (the exclusive-create flag `wx` of Node), so the
    // check and the creation are one step another writer cannot slip into.
    let created_placeholder: bool = match std::fs::OpenOptions::new().write(true).create_new(true).open(lock_path) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(mut file) => file.write_all(EMPTY_LOCK.as_bytes()).is_ok(),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => false,
    };
    // Typed binding: `code: i32`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let code: i32 = body.run();
    if created_placeholder && (code != 0 || dry_run) {
        before_cleanup(lock_path);
        remove_placeholder(lock_path, keep_dir);
    }
    return code;
}

/// What:
/// `fn run_update(prepared, crate_dirs, dry_run, kind, forward, sink) -> i32` runs an update of either kind and prints its report.
/// Why:
/// One place turns an update outcome into text and an exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runUpdate(args): number
/// ```
fn run_update(prepared: &Prepared, crate_dirs: &[String], dry_run: bool, kind: &UpdateKind, forward: &[String], sink: &mut dyn Write) -> i32 {
    let catalog_path = prepared.catalog_root.join(CATALOG_MARKER);
    let outcome = update_rust_lockfiles(
        // Borrow `&UpdateRequest`: lends the value without copying or giving it away.
        &UpdateRequest {
            // Borrow `&prepared.catalog_root`: lends the value without copying or giving it away.
            root: &prepared.catalog_root,
            crate_dirs,
            dry_run,
            now: now_millis(),
            // Borrow `&prepared.document.settings`: lends the value without copying or giving it away.
            settings: &prepared.document.settings,
            // Borrow `&catalog_path`: lends the value without copying or giving it away.
            catalog_path: &catalog_path,
            // Borrow `&prepared.runner`: lends the value without copying or giving it away.
            runner: &prepared.runner,
            // Borrow `&prepared.registry`: lends the value without copying or giving it away.
            registry: &prepared.registry,
            forward,
            // Borrow `&FileEnforcer`: lends the value without copying or giving it away.
            syncer: &FileEnforcer,
            prune_catalog: false,
            lock_base: prepared.lock_base.as_deref(),
        },
        kind,
    );
    // `match`: picks the arm whose pattern fits the value.
    return match outcome {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(report) => {
            // Typed binding: `finished: UpdateReport`.
            let finished: UpdateReport = report;
            // Borrow `&finished`: lends the value without copying or giving it away.
            print_update_report(&finished, dry_run, sink);
            if finished.problems.is_empty() {
                0
            } else {
                // Borrow `&finished.problems`: lends the value without copying or giving it away.
                print_findings("Update refused; every Cargo.lock was restored", &finished.problems);
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
