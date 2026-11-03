//! What:
//! The glue between a cargo command line and the steps the shadow runs around it:
//! where cargo will work,
//! the install step
//! before it,
//! and the catalog warning after it.
//! Why:
//! `shim.rs` decides whether to run cargo;
//! this module decides what happens around the run,
//! so each can be tested alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-step.ts
//! export function startDir(args: string[], cwd: string): string
//! export function beforeCargo(args: string[]): string | undefined  // a failure message
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
/// `OsString` is used by `start_dir`,
/// `prepare`,
/// `before_cargo`,
/// `warn_after_cargo`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { OsString } from 'std/ffi';
/// ```
use std::ffi::OsString;
/// What:
/// `use std::io::Write;` imports `Write` from the standard library so this file can name it unqualified.
/// Why:
/// `Write` is used by `warn_after_cargo`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Write } from 'std/io';
/// ```
use std::io::Write;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// What:
/// `use std::path::{Path, PathBuf};` imports `Path`,
/// `PathBuf` from the standard library so this file can name them unqualified.
/// Why:
/// `Path` is used by `start_dir`,
/// `lock_base_reader`;
/// `PathBuf` is used by `start_dir`,
/// `Prepared`,
/// `prepare`,
/// `warn_after_cargo`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};
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
/// `CatalogDocument` is used by `Prepared`,
/// `prepare`,
/// `lock_base_reader`;
/// `parse_catalog` is used by `prepare`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CatalogDocument, parse_catalog } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};

/// What:
/// `use crate::cargo_run::RealCargo;` imports `RealCargo` from another module of this crate so this file can name it unqualified.
/// Why:
/// `RealCargo` is used by `Prepared`,
/// `prepare`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RealCargo } from 'crate/cargo_run';
/// ```
use crate::cargo_run::RealCargo;
/// What:
/// `use crate::catalog_warning::warn_about_manifest;` imports `warn_about_manifest` from another module of this crate so this file can name it unqualified.
/// Why:
/// `warn_about_manifest` is used by `warn_after_cargo`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { warn_about_manifest } from 'crate/catalog_warning';
/// ```
use crate::catalog_warning::warn_about_manifest;
/// What:
/// `use crate::constants::{CATALOG_MARKER, MANIFEST_NAME};` imports `CATALOG_MARKER`,
/// `MANIFEST_NAME` from another module of this crate so this file can name them unqualified.
/// Why:
/// `CATALOG_MARKER` is used by `prepare`;
/// `MANIFEST_NAME` is used by `warn_after_cargo`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CATALOG_MARKER, MANIFEST_NAME } from 'crate/constants';
/// ```
use crate::constants::{CATALOG_MARKER, MANIFEST_NAME};
/// What:
/// `use crate::git_base::{explicit_base_from_environment, git_lock_reader_at, resolve_trust_base};` imports `base_from_environment`,
/// `git_base_lock_reader` from another module of this crate so this file can name them unqualified.
/// Why:
/// `base_from_environment` is used by `lock_base_reader`;
/// `git_base_lock_reader` is used by `lock_base_reader`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { base_from_environment, git_base_lock_reader } from 'crate/git_base';
/// ```
use crate::git_base::{explicit_base_from_environment, git_lock_reader_at, resolve_trust_base};
/// What:
/// `use crate::http::UreqFetcher;` imports `UreqFetcher` from another module of this crate so this file can name it unqualified.
/// Why:
/// `UreqFetcher` is used by `prepare`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UreqFetcher } from 'crate/http';
/// ```
use crate::http::UreqFetcher;
/// What:
/// `use crate::install::{LockBaseReader, is_continuous_integration};` imports `is_continuous_integration` from another module of this crate so this file can name it unqualified.
/// Why:
/// `is_continuous_integration` is used by `prepare`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { is_continuous_integration } from 'crate/install';
/// ```
use crate::install::{LockBaseReader, is_continuous_integration};
/// What:
/// `use crate::install_step::{StepOutcome, StepRequest, run_install_step};` imports `StepOutcome`,
/// `StepRequest`,
/// `run_install_step` from another module of this crate so this file can name them unqualified.
/// Why:
/// `StepOutcome` is used by `before_cargo`;
/// `StepRequest` is used by `before_cargo`;
/// `run_install_step` is used by `before_cargo`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { StepOutcome, StepRequest, run_install_step } from 'crate/install_step';
/// ```
use crate::install_step::{StepOutcome, StepRequest, run_install_step};
/// What:
/// `use crate::options::{asserts_lock, change_dir, manifest_path};` imports `asserts_lock`,
/// `change_dir`,
/// `manifest_path` from another module of this crate so this file can name them unqualified.
/// Why:
/// `asserts_lock` is used by `prepare`,
/// `with_locked`;
/// `change_dir` is used by `start_dir`;
/// `manifest_path` is used by `start_dir`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { asserts_lock, change_dir, manifest_path } from 'crate/options';
/// ```
use crate::options::{asserts_lock, change_dir, manifest_path};
/// What:
/// `use crate::real_binary::find_real_cargo;` imports `find_real_cargo` from another module of this crate so this file can name it unqualified.
/// Why:
/// `find_real_cargo` is used by `prepare`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { find_real_cargo } from 'crate/real_binary';
/// ```
use crate::real_binary::find_real_cargo;
/// What:
/// `use crate::registry::CratesIo;` imports `CratesIo` from another module of this crate so this file can name it unqualified.
/// Why:
/// `CratesIo` is used by `Prepared`,
/// `prepare`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CratesIo } from 'crate/registry';
/// ```
use crate::registry::CratesIo;
/// What:
/// `use crate::toolchain::cargo_toolchain;` imports `cargo_toolchain` from another module of this crate so this file can name it unqualified.
/// Why:
/// `cargo_toolchain` is used by `prepare`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { cargo_toolchain } from 'crate/toolchain';
/// ```
use crate::toolchain::cargo_toolchain;
/// What:
/// `use crate::workspace::{find_cargo_root, find_catalog_root};` imports `find_cargo_root`,
/// `find_catalog_root` from another module of this crate so this file can name them unqualified.
/// Why:
/// `find_cargo_root` is used by `prepare`;
/// `find_catalog_root` is used by `prepare`,
/// `warn_after_cargo`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { find_cargo_root, find_catalog_root } from 'crate/workspace';
/// ```
use crate::workspace::{find_cargo_root, find_catalog_root};
/// What:
/// `use crate::adapt::text_of;` imports `text_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `text_of` is used by `prepare`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { text_of } from 'crate/adapt';
/// ```
use crate::adapt::text_of;
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

/// What:
/// `pub fn start_dir(arguments: &[OsString], current: &Path) -> PathBuf` is where cargo will look for its manifest:
/// the
/// directory of `--manifest-path` when given,
/// else the working directory moved by `-C`.
/// Why:
/// The install step runs for the workspace cargo will use,
/// and `cargo check --manifest-path other/Cargo.toml` uses another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function startDir(args: string[], cwd: string): string
/// ```
pub fn start_dir(arguments: &[OsString], current: &Path) -> PathBuf {
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
    let base: PathBuf = match change_dir(arguments) {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        Some(directory) => current.join(directory),
        // `None`: the present or the absent value of an `Option`.
        None => current.to_path_buf(),
    };
    // `Some`: the present or the absent value of an `Option`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    if let Some(manifest) = manifest_path(arguments) {
        // Typed binding: `full: PathBuf`.
        let full: PathBuf = base.join(manifest);
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(parent) = full.parent() {
            return parent.to_path_buf();
        }
    }
    return base;
}

/// Milliseconds since 1970 now.
pub fn now_millis() -> i64 {
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
    return SystemTime::now().duration_since(UNIX_EPOCH).map(millis_of).unwrap_or(0);
}

/// What the steps around a cargo command need,
/// found once per run.
pub struct Prepared {
    /// Where cargo will work.
    pub start: PathBuf,
    /// The repository root.
    pub catalog_root: PathBuf,
    /// The repo-relative crate root of the workspace,
    /// when it is below the repository.
    pub crate_dir: Option<String>,
    /// The validated catalog.
    pub document: CatalogDocument,
    /// The real cargo under the pinned toolchain.
    pub runner: RealCargo,
    /// crates.io.
    pub registry: CratesIo,
    /// Whether a stale lockfile is a failure:
    /// in CI,
    /// or the user's own command carries `--locked` or `--frozen`.
    pub frozen: bool,
    /// The maker of the base-revision lockfile reader,
    /// present under `trustLockfile`;
    /// git is started only when a check that is not remembered needs the reader.
    pub lock_base: Option<OwnedLockBaseFactory>,
    /// How the base revision is spelled (`CARGO_SHADOW_TRUST_BASE`,
    /// or `HEAD`).
    pub trust_base: String,
    /// Whether the install step keeps cargo offline:
    /// always,
    /// except in CI,
    /// where no registry index is cached yet.
    pub offline: bool,
}

/// What:
/// `fn missing_cargo_message() -> String` words the absence of a real `cargo` on `PATH`.
/// Why:
/// The install step needs the real cargo,
/// and without it the shadow reports that instead of looping on itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const missingCargoMessage = () => 'cargo-shadow: no real cargo found on PATH for the install step';
/// ```
fn missing_cargo_message() -> String {
    return "cargo-shadow: no real cargo found on PATH for the install step".to_string();
}

/// What:
/// `fn catalog_read_failure(error: std::io::Error) -> String` words a failure to read `rust-catalog.toml`.
/// Why:
/// The install step names the file it could not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const catalogReadFailure = (error: Error) => `cargo-shadow: rust-catalog.toml: ${error.message}`;
/// ```
fn catalog_read_failure(error: std::io::Error) -> String {
    return format!("cargo-shadow: {CATALOG_MARKER}: {error}");
}

/// What:
/// `pub fn prepare(arguments: &[OsString]) -> Result<Option<Prepared>, String>` finds the repository,
/// the workspace,
/// the
/// catalog,
/// the real cargo and the toolchain for a command line;
/// `Ok(None)` means the command is not in a catalog repository.
/// Why:
/// The real cargo for the steps is the first `cargo` on PATH that is not this binary,
/// run under the toolchain the repository
/// pins,
/// so every lockfile the shadow writes has one format whatever toolchain the user's own command selects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function prepare(args: string[]): Prepared | undefined  // throws
/// ```
pub fn prepare(arguments: &[OsString]) -> Result<Option<Prepared>, String> {
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    // Typed binding: `current: PathBuf`.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    let current: PathBuf = match std::env::current_dir() {
        Ok(found) => found,
        Err(error) => return Err(format!("cargo-shadow: no working directory: {error}")),
    };
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // Typed binding: `start: PathBuf`.
    let start: PathBuf = start_dir(arguments, &current);
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&start`: lends the value without copying or giving it away.
    // Typed binding: `catalog_root: PathBuf`.
    let catalog_root: PathBuf = match find_catalog_root(&start) {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // `None`: the present or the absent value of an `Option`.
        None => return Ok(None),
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `||`: an anonymous function, like a TS arrow function.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
    // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return compute(); } catch { return undefined; }
    // ```
    // Typed binding: `program: PathBuf`.
    let program: PathBuf = find_real_cargo().ok_or_else(missing_cargo_message)?;
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `explicit: String`.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let explicit: String = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_default();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|message|`: an anonymous function, like a TS arrow function.
    // Borrow `&catalog_root`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let (toolchain, warning) = match cargo_toolchain(&catalog_root, &explicit) {
        Ok(found) => found,
        Err(message) => return Err(format!("cargo-shadow: toolchain.lock.json is not readable: {message}")),
    };
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    if let Some(text) = warning {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: {text}");
    }
    // Typed binding: `catalog_text: String`.
    let catalog_text: String = std::fs::read_to_string(catalog_root.join(CATALOG_MARKER))
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Macro `format!`: builds or prints text (or a list) from a template.
        .map_err(catalog_read_failure)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&catalog_text`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `document: CatalogDocument`.
    let document: CatalogDocument = parse_catalog(&catalog_text, CATALOG_MARKER).map_err(text_of)?;
    // Borrow `&std`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `in_ci: bool`.
    let in_ci: bool = is_continuous_integration(&std::env::var("CI").unwrap_or_default());
    // Closure `|cargo_root|`: an anonymous function, like a TS arrow function.
    // Borrow `&start`: lends the value without copying or giving it away.
    // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
    // Adapter `.and_then(`: transforms or selects items with a closure.
    // Typed binding: `crate_dir: Option<String>`.
    let crate_dir: Option<String> = find_cargo_root(&start).and_then(|cargo_root| { return cargo_root.strip_prefix(&catalog_root).ok().map(|relative| { return relative.to_string_lossy().replace('\\', "/"); }); });
    // Closure `||`: an anonymous function, like a TS arrow function.
    // Typed binding: `frozen: bool`.
    let frozen: bool = in_ci || asserts_lock(arguments);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&str`: lends the value without copying or giving it away.
    // Typed binding: `lock_base: Option<Box<dyn Fn(&str) -> Strin...`.
    // What:     `&dyn Trait` is a borrowed value of any type that implements the trait (a TS interface type); the call goes through a lookup table at run time.
    // Why:      The caller may pass the real implementation or a test fake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function f(registry: Registry) { ... }  // an interface type
    // ```
    let (trust_base, lock_base): (String, Option<OwnedLockBaseFactory>) = trust_base_setup(&catalog_root, &document, in_ci);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    return Ok(Some(Prepared {
        start,
        lock_base,
        trust_base,
        // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
        // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: T[] / Set / Map / number
        // ```
        runner: RealCargo { program, toolchain, bound: std::time::Duration::from_secs(crate::cargo_run::CARGO_CALL_SECONDS) },
        // Borrow `&catalog_root`: lends the value without copying or giving it away.
        // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
        // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const list: string[] = [];   // or new Set() / new Map()
        // ```
        registry: CratesIo::with_defaults(&catalog_root, Box::new(UreqFetcher::new()), Box::new(UreqFetcher::new())),
        catalog_root,
        crate_dir,
        document,
        frozen,
        offline: !in_ci,
    }));
}

/// What:
/// `pub type OwnedLockBaseFactory = Box<dyn Fn() -> Result<LockBaseReader, String>>` is an owned maker of the base-revision
/// lockfile reader.
/// Why:
/// The prepared step keeps it beside the other settings and lends it to the install as `&dyn Fn`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnedLockBaseFactory = () => (crateDir: string) => string  // throws when the base cannot be read
/// ```
pub type OwnedLockBaseFactory = Box<dyn Fn() -> Result<LockBaseReader, String>>;

/// What:
/// `pub fn trust_base_setup(catalog_root, document, in_ci) -> (String, Option<OwnedLockBaseFactory>)` chooses the trust base
/// under `trustLockfile = true` and returns the key a remembered pass is filed under,
/// with the maker of the lockfile reader.
/// Why:
/// Delegated pick (2026-11-03):
/// a local run compares with the upstream branch,
/// CI keeps its base,
/// and the key is the resolved
/// commit,
/// not how the base is spelled,
/// so a base that moves is a different base.
/// A base that names no commit yields the
/// key `unresolved:<spelling>` and a factory that says why,
/// so only a check that needs the base fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trustBaseSetup(catalogRoot: string, document: CatalogDocument, inCi: boolean): [string, (() => Reader) | undefined]
/// ```
#[cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(avoidable_closure, reason = "the returned maker keeps the resolved base and the repository root after this function returns")
)]
pub fn trust_base_setup(catalog_root: &Path, document: &CatalogDocument, in_ci: bool) -> (String, Option<OwnedLockBaseFactory>) {
    if !document.settings.trust_lockfile {
        // `None`: the present or the absent value of an `Option`.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return (String::new(), None);
    }
    let base = resolve_trust_base(catalog_root, in_ci, explicit_base_from_environment().as_deref());
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&base.commit`: lends the value without copying or giving it away.
    // Typed binding: `key: String`.
    let key: String = match &base.commit {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.clone()`: an independent owned copy.
        Ok(commit) => commit.clone(),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        Err(_) => format!("unresolved:{}", base.spelling),
    };
    // Typed binding: `root: PathBuf`.
    let root: PathBuf = catalog_root.to_path_buf();
    // Typed binding: `factory: OwnedLockBaseFactory`.
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    let factory: OwnedLockBaseFactory = Box::new(move || {
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&base.commit`: lends the value without copying or giving it away.
        return match &base.commit {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(commit) => {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!("cargo-shadow: comparing lockfiles with {} ({commit})", base.spelling);
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // Borrow `&root`: lends the value without copying or giving it away.
                Ok(git_lock_reader_at(&root, commit))
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.clone()`: an independent owned copy.
            Err(message) => Err(message.clone()),
        };
    });
    // `Some`: the present or the absent value of an `Option`.
    return (key, Some(factory));
}

/// What:
/// `pub enum Preflight { Failed(String), Refused(String) }` is what the install step tells the shim when the user's command
/// should not simply run.
/// Why:
/// `Failed` is an operational problem (no network,
/// a cargo error,
/// a stale lockfile under a frozen check):
/// the shim warns and
/// runs the command with `--locked`,
/// as recorded in `doc/planning/cargo-shadow-update.md`.
/// `Refused` is a policy refusal with
/// no older release that passes (local,
/// and by the delegated pick of 2026-11-03 frozen too):
/// the command stops,
/// because a
/// refused release must never run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Preflight = { kind: 'failed'; message: string } | { kind: 'refused'; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Preflight {
    /// An operational problem or a stale lockfile under a frozen check:
    /// warn,
    /// then run with `--locked`.
    Failed(
        /// The message to show as the warning,
        /// as owned text.
        String,
    ),
    /// A policy refusal:
    /// stop.
    Refused(
        /// The findings to show,
        /// as owned text.
        String,
    ),
}

/// What:
/// `pub fn before_cargo(arguments: &[OsString]) -> Option<String>` runs the install step for the command line and returns the
/// message to show when the command must not run.
/// Why:
/// `None` means run cargo.
/// In CI,
/// or when the user's own command carries `--locked` or `--frozen`,
/// a stale lock is a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function beforeCargo(args: string[]): string | undefined
/// ```
pub fn before_cargo(arguments: &[OsString]) -> Option<Preflight> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `prepared: Prepared`.
    let prepared: Prepared = match prepare(arguments) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        Ok(Some(found)) => found,
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        Ok(None) => return None,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        Err(message) => return Some(Preflight::Failed(message)),
    };
    // Borrow `&StepRequest`: lends the value without copying or giving it away.
    // Typed binding: `outcome: StepOutcome`.
    let outcome: StepOutcome = run_install_step(&StepRequest {
        // Borrow `&prepared.start`: lends the value without copying or giving it away.
        start_dir: &prepared.start,
        frozen: prepared.frozen,
        offline: prepared.offline,
        now: now_millis(),
        // Borrow `&prepared.runner`: lends the value without copying or giving it away.
        runner: &prepared.runner,
        // Borrow `&prepared.registry`: lends the value without copying or giving it away.
        registry: &prepared.registry,
        lock_base: prepared.lock_base.as_deref(),
        trust_base: &prepared.trust_base,
    });
    return preflight_of(outcome);
}

/// What:
/// `pub fn preflight_of(outcome: StepOutcome) -> Option<Preflight>` turns what the install step did into what the shim must do:
/// `None` runs the user's command as typed,
/// `Some(Preflight::Failed)` warns and runs it with `--locked`,
/// `Some(Preflight::Refused)` stops.
/// Why:
/// The decision whether a refusal stops the command is the one place the owner's rule of 2026-11-03 lives,
/// and a function of the outcome alone can be tested without a registry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function preflightOf(outcome: StepOutcome): Preflight | undefined
/// ```
pub fn preflight_of(outcome: StepOutcome) -> Option<Preflight> {
    match outcome {
        // `Some`: the present or the absent value of an `Option`.
        StepOutcome::Failed(message) => return Some(Preflight::Failed(message)),
        // `Some`: the present or the absent value of an `Option`.
        StepOutcome::Refused(message) => return Some(Preflight::Refused(message)),
        StepOutcome::Updated { crate_dir, added } => {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!(
                "cargo-shadow: {crate_dir}/Cargo.lock did not satisfy its manifests and was updated offline ({} newly locked registry versions passed the policy)",
                added.len()
            );
            // `None`: the present or the absent value of an `Option`.
            return None;
        }
        // `None`: the present or the absent value of an `Option`.
        // Closure `| StepOutcome::FastPath |`: an anonymous function, like a TS arrow function.
        StepOutcome::Skipped(_) | StepOutcome::FastPath | StepOutcome::Passed => return None,
    }
}

/// Writes the catalog warning for the manifest cargo worked on:
/// the one named by `--manifest-path`,
/// else the nearest
/// `Cargo.toml` above the start directory.
pub fn warn_after_cargo(arguments: &[OsString], sink: &mut dyn Write) {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `current: PathBuf`.
    let current: PathBuf = match std::env::current_dir() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => return,
    };
    // Borrow `&current`: lends the value without copying or giving it away.
    // Typed binding: `start: PathBuf`.
    let start: PathBuf = start_dir(arguments, &current);
    // Borrow `&start`: lends the value without copying or giving it away.
    if find_catalog_root(&start).is_none() {
        return;
    }
    for directory in start.ancestors() {
        // Typed binding: `manifest: PathBuf`.
        let manifest: PathBuf = directory.join(MANIFEST_NAME);
        if manifest.is_file() {
            // Borrow `&manifest`: lends the value without copying or giving it away.
            warn_about_manifest(&manifest, sink);
            return;
        }
    }
}

/// What:
/// `pub fn with_locked(arguments: &[OsString]) -> Vec<OsString>` is the user's command line with `--locked` added before any
/// `--`,
/// unless the user already passed `--locked` or `--frozen`.
/// Why:
/// This is the one place the shadow adds an option,
/// and only after a failed or refused install step:
/// pnpm runs the command
/// anyway when its install step fails,
/// because `pnpm run` never rewrites the lockfile;
/// cargo does rewrite a stale lock,
/// so the
/// faithful form of "run it anyway" is to run it with `--locked`,
/// which makes cargo stop with its own lock error where it would
/// have changed the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function withLocked(args: string[]): string[]
/// ```
pub fn with_locked(arguments: &[OsString]) -> Vec<OsString> {
    if asserts_lock(arguments) {
        // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
        // Why:      The result can be stored or modified independently of the slice it came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // [...slice]
        // ```
        return arguments.to_vec();
    }
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `result: Vec<OsString>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut result: Vec<OsString> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `added: bool`.
    let mut added: bool = false;
    for argument in arguments {
        if !added && argument == "--" {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            result.push(OsString::from("--locked"));
            added = true;
        }
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        result.push(argument.clone());
    }
    if !added {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        result.push(OsString::from("--locked"));
    }
    return result;
}
