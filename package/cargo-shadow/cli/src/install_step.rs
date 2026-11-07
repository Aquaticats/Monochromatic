//! What:
//! The shadow's install step:
//! before a cargo command that resolves dependencies,
//! make the `Cargo.lock` of the Cargo workspace
//! that holds the working directory satisfy its manifests,
//! under the catalog's trust policy.
//! Why:
//! This is pnpm's `verifyDepsBeforeRun: install` default ("Automatically runs install if `node_modules` is not up to
//! date",
//! <https://pnpm.io/settings/build>,
//! read 2026-11-03) for lockfiles:
//! a stale lock is brought in line by the same code as
//! the install command (`install.rs`),
//! in CI or under the user's own `--locked` or `--frozen` it fails naming the stale lock
//! instead (pnpm's `frozenLockfile`),
//! and the user's cargo command is then run unchanged by the caller.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-step.ts
//! export function runInstallStep(request: StepRequest): StepOutcome
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `read_catalog`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
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
/// `Path` is used by `StepRequest`,
/// `relative_crate_dir`,
/// `read_catalog`,
/// `write_stamp`;
/// `PathBuf` is used by `read_catalog`,
/// `write_stamp`,
/// `run_install_step`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};` imports `CatalogDocument`,
/// `parse_catalog` from the external crate `monochromatic_dependency_catalog` so this file can name them unqualified.
/// Why:
/// `CatalogDocument` is used by `read_catalog`,
/// `run_install_step`;
/// `parse_catalog` is used by `read_catalog`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CatalogDocument, parse_catalog } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::{CatalogDocument, parse_catalog};

/// What:
/// `use crate::cargo_run::CargoRunner;` imports `CargoRunner` from another module of this crate so this file can name it unqualified.
/// Why:
/// `CargoRunner` is used by `StepRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoRunner } from 'crate/cargo_run';
/// ```
use crate::cargo_run::CargoRunner;
/// What:
/// `use crate::constants::{CATALOG_MARKER, LOCKFILE_NAME};` imports `CATALOG_MARKER`,
/// `LOCKFILE_NAME` from another module of this crate so this file can name them unqualified.
/// Why:
/// `CATALOG_MARKER` is used by `read_catalog`;
/// `LOCKFILE_NAME` is used by `write_stamp`,
/// `run_install_step`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CATALOG_MARKER, LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::{CATALOG_MARKER, LOCKFILE_NAME};
/// What:
/// `use crate::install::{InstallError, InstallReport, InstallRequest, LockBaseFactory, VERIFY_ARGS, arguments_for, install_rust_lockfiles, verification_cached};` imports `InstallError`,
/// `InstallReport`,
/// `InstallRequest`,
/// `VERIFY_ARGS`,
/// `arguments_for`,
/// `install_rust_lockfiles` from another module of this crate so this file can name them unqualified.
/// Why:
/// `InstallError` is used by `run_install_step`;
/// `InstallReport` is used by `run_install_step`;
/// `InstallRequest` is used by `run_install_step`;
/// `VERIFY_ARGS` is used by `write_stamp`;
/// `arguments_for` is used by `write_stamp`;
/// `install_rust_lockfiles` is used by `run_install_step`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InstallError, InstallReport, InstallRequest, VERIFY_ARGS, arguments_for, install_rust_lockfiles } from 'crate/install';
/// ```
use crate::install::{InstallError, InstallReport, InstallRequest, LockBaseFactory, VERIFY_ARGS, arguments_for, install_rust_lockfiles, verification_cached};
/// What:
/// `use crate::lockfile::LockedVersion;` imports `LockedVersion` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LockedVersion` is used by `StepOutcome`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion } from 'crate/lockfile';
/// ```
use crate::lockfile::LockedVersion;
/// What:
/// `use crate::registry::Registry;` imports `Registry` from another module of this crate so this file can name it unqualified.
/// Why:
/// `Registry` is used by `StepRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;
/// What:
/// `use crate::stamp::{is_fresh, manifests_from_metadata, record, stamp_file};` imports `is_fresh`,
/// `manifests_from_metadata`,
/// `record`,
/// `stamp_file` from another module of this crate so this file can name them unqualified.
/// Why:
/// `is_fresh` is used by `run_install_step`;
/// `manifests_from_metadata` is used by `write_stamp`;
/// `record` is used by `write_stamp`;
/// `stamp_file` is used by `write_stamp`,
/// `run_install_step`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { is_fresh, manifests_from_metadata, record, stamp_file } from 'crate/stamp';
/// ```
use crate::stamp::{is_fresh, manifests_from_metadata, record, stamp_file};
/// What:
/// `use crate::workspace::{find_cargo_root, find_catalog_root};` imports `find_cargo_root`,
/// `find_catalog_root` from another module of this crate so this file can name them unqualified.
/// Why:
/// `find_cargo_root` is used by `run_install_step`;
/// `find_catalog_root` is used by `run_install_step`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { find_cargo_root, find_catalog_root } from 'crate/workspace';
/// ```
use crate::workspace::{find_cargo_root, find_catalog_root};
/// What:
/// `use crate::adapt::text_of;` imports `text_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `text_of` is used by `read_catalog`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { text_of } from 'crate/adapt';
/// ```
use crate::adapt::text_of;

/// What the install step did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepOutcome {
    /// Nothing to do here,
    /// with the reason.
    Skipped(
        /// What:
        /// The reason the step did nothing,
        /// as text (`String`,
        /// owned;
        /// sibling:
        /// `&str`,
        /// borrowed).
        /// Why:
        /// The shadow ignores a skip silently,
        /// but tests and logs can say why no install check ran.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// { kind: 'skipped', reason: string }
        /// ```
        String,
    ),
    /// The recorded fingerprints still match:
    /// no cargo was started.
    FastPath,
    /// The authoritative check found the lockfile satisfied.
    Passed,
    /// The lockfile was stale and was updated.
    Updated {
        /// The repo-relative crate root.
        crate_dir: String,
        /// The registry versions newly locked.
        added: Vec<LockedVersion>,
    },
    /// The policy refuses a release (a local run found no older release in range that passes,
    /// a frozen check does not look for one),
    /// a registry the policy cannot judge is locked,
    /// or a lockfile this run's cargo wrote could not be put back:
    /// the caller must not run the user's command.
    /// Carries the message to show.
    Refused(
        /// The findings,
        /// as text (`String`,
        /// owned).
        String,
    ),
    /// The step failed (cargo or the registry could not be asked,
    /// or a frozen check found a stale lockfile),
    /// with the message to show.
    Failed(
        /// What:
        /// The message to show the user,
        /// as text (`String`,
        /// owned).
        /// Why:
        /// The shadow prints it as the warning that explains why `--locked` was added to the user's command.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// { kind: 'failed', message: string }
        /// ```
        String,
    ),
}

/// What the step needs.
pub struct StepRequest<'a> {
    /// Where cargo will work.
    pub start_dir: &'a Path,
    /// Fail on a stale lockfile instead of updating it.
    pub frozen: bool,
    /// Keep every cargo call offline;
    /// false in CI,
    /// where no registry index is cached yet.
    pub offline: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
    /// The maker of the base-revision lockfile reader an install needs under `trustLockfile`.
    pub lock_base: Option<LockBaseFactory<'a>>,
    /// How the base revision is spelled (`CARGO_SHADOW_TRUST_BASE`,
    /// or `HEAD`):
    /// part of the key of a remembered policy pass.
    pub trust_base: &'a str,
}

/// The crate root relative to the repository,
/// with forward slashes;
/// `None` when it is not inside it.
fn relative_crate_dir(catalog_root: &Path, cargo_root: &Path) -> Option<String> {
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
    // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return compute(); } catch { return undefined; }
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let relative: &Path = cargo_root.strip_prefix(catalog_root).ok()?;
    // Typed binding: `text: String`.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let text: String = relative.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        return None;
    }
    // `Some`: the present or the absent value of an `Option`.
    return Some(text);
}

/// Reads and validates `rust-catalog.toml`.
fn read_catalog(catalog_root: &Path) -> Result<CatalogDocument, String> {
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = catalog_root.join(CATALOG_MARKER);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // Borrow `&path`: lends the value without copying or giving it away.
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    // Typed binding: `text: String`.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    let text: String = match fs::read_to_string(&path) {
        Ok(found) => found,
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&text`: lends the value without copying or giving it away.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // `.map_err(...)`: converts the callee's error into this function's error type.
    return parse_catalog(&text, CATALOG_MARKER).map_err(text_of);
}

/// Records the stamp after a pass:
/// one `cargo metadata` call lists the manifests the lockfile was resolved from.
fn write_stamp(request: &StepRequest, catalog_root: &Path, crate_dir: &str) {
    // Typed binding: `cargo_root: PathBuf`.
    let cargo_root: PathBuf = catalog_root.join(crate_dir);
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    // Borrow `&cargo_root`: lends the value without copying or giving it away.
    let outcome = match request.runner.run(&cargo_root, &arguments_for(&VERIFY_ARGS, request.offline)) {
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
        Err(_) => return,
    };
    if !outcome.ok {
        return;
    }
    // Borrow `&outcome.stdout`: lends the value without copying or giving it away.
    // Typed binding: `manifests: Vec<PathBuf>`.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let manifests: Vec<PathBuf> = manifests_from_metadata(&outcome.stdout);
    if manifests.is_empty() {
        return;
    }
    // Borrow `&stamp_file`: lends the value without copying or giving it away.
    record(&stamp_file(catalog_root, crate_dir), &cargo_root.join(LOCKFILE_NAME), &manifests);
}

/// What:
/// `pub fn run_install_step(request: &StepRequest) -> StepOutcome` runs the step for the workspace of `request.start_dir`.
/// Why:
/// Outside a catalog repository,
/// or in a workspace without a lockfile,
/// the step does nothing.
/// Otherwise a matching stamp
/// answers at once;
/// without one,
/// the install over this one crate root answers,
/// and a pass or a kept update records a stamp.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runInstallStep(request): StepOutcome
/// ```
pub fn run_install_step(request: &StepRequest) -> StepOutcome {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `catalog_root: PathBuf`.
    let catalog_root: PathBuf = match find_catalog_root(request.start_dir) {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `None`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        None => return StepOutcome::Skipped("no rust-catalog.toml above the working directory".to_string()),
    };
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `cargo_root: PathBuf`.
    let cargo_root: PathBuf = match find_cargo_root(request.start_dir) {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `None`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        None => return StepOutcome::Skipped("no Cargo.toml above the working directory".to_string()),
    };
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&catalog_root`: lends the value without copying or giving it away.
    // Typed binding: `crate_dir: String`.
    let crate_dir: String = match relative_crate_dir(&catalog_root, &cargo_root) {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `None`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        None => return StepOutcome::Skipped("the Cargo workspace is not below the catalog repository".to_string()),
    };
    // Typed binding: `lock: PathBuf`.
    let lock: PathBuf = cargo_root.join(LOCKFILE_NAME);
    if !lock.is_file() {
        // Macro `format!`: builds or prints text (or a list) from a template.
        return StepOutcome::Skipped(format!("{crate_dir} has no {LOCKFILE_NAME} yet"));
    }
    // Borrow `&catalog_root`: lends the value without copying or giving it away.
    // Typed binding: `stamp: PathBuf`.
    let stamp: PathBuf = stamp_file(&catalog_root, &crate_dir);
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&catalog_root`: lends the value without copying or giving it away.
    // Typed binding: `document: CatalogDocument`.
    let document: CatalogDocument = match read_catalog(&catalog_root) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(message) => return StepOutcome::Failed(message),
    };
    // Review finding (the freshness stamp bypasses the policy checks): the stamp certifies that the lockfile satisfies its
    // manifests, nothing about the policy, so the settings are read first and the fast path needs both the stamp and a
    // remembered policy pass of this mode, these settings and this base.
    let fresh: bool = is_fresh(&stamp, &lock);
    // `.clone()`: an independent owned copy.
    // Typed binding: `crate_dirs_for_cache: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let crate_dirs_for_cache: Vec<String> = vec![crate_dir.clone()];
    // Borrow `&catalog_root`: lends the value without copying or giving it away.
    if fresh && verification_cached(&catalog_root, &crate_dirs_for_cache, &document.settings, request.frozen, request.trust_base) {
        // The fast path starts no transaction, so it announces the kept files itself (promised on every run past the count).
        crate::journal_transaction::announce_kept_files_for(&catalog_root, &crate_dirs_for_cache);
        return StepOutcome::FastPath;
    }
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // Typed binding: `crate_dirs: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let crate_dirs: Vec<String> = vec![crate_dir.clone()];
    // Borrow `&InstallRequest`: lends the value without copying or giving it away.
    // Typed binding: `report: Result<InstallReport, InstallError>`.
    let report: Result<InstallReport, InstallError> = install_rust_lockfiles(&InstallRequest {
        // Borrow `&catalog_root`: lends the value without copying or giving it away.
        root: &catalog_root,
        // Borrow `&crate_dirs`: lends the value without copying or giving it away.
        crate_dirs: &crate_dirs,
        frozen: request.frozen,
        offline: request.offline,
        dry_run: false,
        now: request.now,
        // Borrow `&document.settings`: lends the value without copying or giving it away.
        settings: &document.settings,
        runner: request.runner,
        registry: request.registry,
        lock_base: request.lock_base,
        trust_base: request.trust_base,
        known_consistent: fresh,
    });
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `finished: InstallReport`.
    let finished: InstallReport = match report {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // A refusal of the policy or a lockfile that could not be put back stops the command; no `--locked` fallback may build it.
        Err(error @ (InstallError::Refused { .. } | InstallError::Unsettled { .. })) => return StepOutcome::Refused(error.to_string()),
        Err(error) => return StepOutcome::Failed(error.to_string()),
    };
    if !finished.problems.is_empty() {
        // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
        // Why:      The value is built up or replaced step by step below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let x = ...;   // plain Rust `let` is TS `const`
        // ```
        // Typed binding: `lines: Vec<String>`.
        // Macro `vec!`: builds or prints text (or a list) from a template.
        let mut lines: Vec<String> = vec![format!(
            "Install refused; no Cargo.lock was left changed: {} finding(s).",
            finished.problems.len()
        )];
        // Borrow `&finished.problems`: lends the value without copying or giving it away.
        for problem in &finished.problems {
            // Macro `format!`: builds or prints text (or a list) from a template.
            lines.push(format!("- {problem}"));
        }
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        lines.push("Settings live in rust-catalog.toml; see doc/decision/rust-dependency-catalog.md.".to_string());
        // Delegated pick (2026-11-03): a policy refusal stops the command on every path, frozen or local, as pnpm's frozen
        // install fails on a rejected lockfile entry before any fetch. A stale lockfile or an operational failure is not a
        // policy refusal and stays a warning plus `--locked`.
        return StepOutcome::Refused(lines.join("\n"));
    }
    // Borrow `&catalog_root`: lends the value without copying or giving it away.
    write_stamp(request, &catalog_root, &crate_dir);
    if finished.stale.is_empty() {
        return StepOutcome::Passed;
    }
    return StepOutcome::Updated { crate_dir, added: finished.added };
}
