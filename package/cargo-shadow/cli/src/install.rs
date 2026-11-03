//! What:
//! The install step,
//! the counterpart of `pnpm install`:
//! makes each listed crate root's `Cargo.lock` satisfy its manifests,
//! moving as little as the manifests force.
//! Why:
//! `mise run sync:files` renders catalog floors and path-crate versions into the manifests and never touches a lockfile,
//! so a
//! crate whose lockfile held an older release is left with a lockfile its own manifest refuses;
//! any plain cargo run then
//! rewrites it through cargo's own resolver,
//! past the trust policy.
//! pnpm repairs the same situation with `pnpm install`,
//! and follows these lockfile rules (<https://pnpm.io/settings/build>,
//! <https://pnpm.io/settings/store>,
//! read 2026-11-03):
//!
//! - Outside CI a stale lockfile is updated (`--no-frozen-lockfile`).
//! - In CI,
//!   or with `--frozen-lockfile`,
//!   a stale lockfile is a failure that names the roots and writes nothing.
//! - `--dry-run` reports what an install would update and writes nothing.
//! - A root that still satisfies its manifests keeps its lockfile byte for byte;
//!   in a stale root the dependencies whose
//!   requirements still hold keep their locked versions.
//!
//! Per crate root,
//! offline:
//! `cargo update --workspace --locked --offline` asks whether the lockfile already satisfies the
//! manifests (reads the index only,
//! writes nothing);
//! a refusal means stale;
//! `cargo update --workspace --offline` re-locks the
//! workspace members and what their changed requirements force;
//! `cargo metadata --locked --offline` confirms the result.
//! A
//! non-frozen install is one transaction:
//! every lockfile is snapshotted first,
//! and any failure or refusal restores all of
//! them.
//! A registry version that no lockfile held before goes through the release-age and trust checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install.ts
//! export async function installRustLockfiles({ root, frozenLockfile, dryRun, now, runCargo, registry }): Promise<InstallReport>
//! ```

/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a value that can be changed through a shared reference (sibling:
/// `&mut`,
/// which needs exclusive access).
/// Why:
/// The record of what this run's cargo calls wrote (`produced`) grows while the transaction body holds a shared reference to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// Imports `BTreeSet`,
/// an ordered set (sibling:
/// `HashSet`,
/// unordered),
/// which collects the versions to check once each.
use std::collections::BTreeSet;
/// What:
/// `use std::fmt;` imports `fmt` from the standard library so this file can name it unqualified.
/// Why:
/// `fmt` is used by the `impl` of `InstallError`,
/// `fmt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fmt } from 'std';
/// ```
use std::fmt;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `InstallRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports `Settings` from the external crate `monochromatic_dependency_catalog` so this file can name it unqualified.
/// Why:
/// `Settings` is used by `InstallRequest`,
/// `policy_findings`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::Settings;

/// What:
/// `use crate::cargo_run::{CargoOutcome, CargoRunner};` imports `CargoOutcome`,
/// `CargoRunner` from another module of this crate so this file can name them unqualified.
/// Why:
/// `CargoOutcome` is used by `run_cargo`,
/// `run_or_fail`;
/// `CargoRunner` is used by `InstallRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoOutcome, CargoRunner } from 'crate/cargo_run';
/// ```
use crate::cargo_run::{CargoOutcome, CargoRunner};
/// What:
/// `use crate::checks::{CheckOutcome, check_release_age, check_trust};` imports `CheckOutcome`,
/// `check_release_age`,
/// `check_trust` from another module of this crate so this file can name them unqualified.
/// Why:
/// `CheckOutcome` is used by `update_stale_roots`;
/// `check_release_age` is used by `update_stale_roots`,
/// `policy_findings`;
/// `check_trust` is used by `update_stale_roots`,
/// `policy_findings`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CheckOutcome, check_release_age, check_trust } from 'crate/checks';
/// ```
use crate::checks::{CheckOutcome, check_release_age, check_trust};
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports `LockSnapshot`,
/// `read_locks` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockSnapshot` is used by `sources_of`,
/// `update_stale_roots`,
/// `policy_findings`,
/// `install_rust_lockfiles`;
/// `read_locks` is used by `update_stale_roots`,
/// `install_rust_lockfiles`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, read_locks } from 'crate/crates';
/// ```
use crate::crates::{LockSnapshot, read_locks};
/// What:
/// `use crate::journal::{begin, finish, note_produced, process_exists, recover, restore_if_unchanged};` imports `begin`,
/// `finish`,
/// `note_produced`,
/// `process_exists`,
/// `recover`,
/// `restore_if_unchanged` from another module of this crate so this file can name them unqualified.
/// Why:
/// `begin` is used by `install_rust_lockfiles`;
/// `finish` is used by `install_rust_lockfiles`;
/// `note_produced` is used by `update_stale_roots`;
/// `process_exists` is used by `install_rust_lockfiles`;
/// `recover` is used by `install_rust_lockfiles`;
/// `restore_if_unchanged` is used by `install_rust_lockfiles`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { begin, finish, note_produced, process_exists, recover, restore_if_unchanged } from 'crate/journal';
/// ```
use crate::journal::{begin, finish, note_produced, process_exists, recover, restore_if_unchanged};
/// What:
/// `use crate::http::RegistryError;` imports `RegistryError` from another module of this crate so this file can name it unqualified.
/// Why:
/// `RegistryError` is used by the `impl` of `InstallError`,
/// `from`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegistryError } from 'crate/http';
/// ```
use crate::http::RegistryError;
/// What:
/// `use crate::lockfile::{LockedVersion, added_versions};` imports `LockedVersion`,
/// `added_versions` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockedVersion` is used by `InstallReport`,
/// `update_stale_roots`,
/// `policy_findings`;
/// `added_versions` is used by `update_stale_roots`,
/// `policy_findings`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion, added_versions } from 'crate/lockfile';
/// ```
use crate::lockfile::{LockedVersion, added_versions};
/// What:
/// `use crate::registry::Registry;` imports `Registry` from another module of this crate so this file can name it unqualified.
/// Why:
/// `Registry` is used by `InstallError`,
/// `fmt`,
/// `from`,
/// `InstallRequest`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;
/// Imports the set-aside round,
/// which moves a refused release to the newest older one that passes.
use crate::set_aside::{SetAsideRequest, set_aside_until_clean};
/// What:
/// `use crate::verify::{ALL_ENTRIES_SCOPE, is_verified, mark_verified, verify_added_since_base, verify_locked_entries};` imports
/// `ALL_ENTRIES_SCOPE`,
/// `is_verified`,
/// `mark_verified`,
/// `verify_added_since_base`,
/// `verify_locked_entries` from another module of this crate so this file can name them unqualified.
/// Why:
/// The frozen check,
/// the remembered passes and their scope names live in `verify.rs`;
/// `install_rust_lockfiles`,
/// `update_stale_roots` and `verification_cached` use them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ALL_ENTRIES_SCOPE, isVerified, markVerified, verifyAddedSinceBase, verifyLockedEntries } from 'crate/verify';
/// ```
use crate::verify::{ALL_ENTRIES_SCOPE, is_verified, mark_verified, verify_added_since_base, verify_locked_entries};

/// Asks whether the lockfile satisfies the manifests:
/// exits 0 without writing when it does.
pub const CHECK_ARGS: [&str; 4] = ["update", "--workspace", "--locked", "--offline"];

/// Re-locks the workspace members and whatever their changed requirements force,
/// keeping every other locked release.
pub const UPDATE_ARGS: [&str; 3] = ["update", "--workspace", "--offline"];

/// The consistency check run after the update.
pub const VERIFY_ARGS: [&str; 5] = ["metadata", "--locked", "--offline", "--format-version", "1"];

/// What:
/// `pub fn arguments_for(args: &[&str], offline: bool) -> Vec<String>` is an argument list as owned text,
/// without `--offline`
/// when the run may use the network.
/// Why:
/// A fresh CI runner has no cached registry index,
/// so an offline check cannot read it;
/// a developer machine has one,
/// and the
/// offline form keeps the check from touching the network at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const argumentsFor = (args: string[], offline: boolean) => offline ? args : args.filter((a) => a !== '--offline');
/// ```
pub fn arguments_for(args: &[&str], offline: bool) -> Vec<String> {
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      `arguments_for` drops `--offline` from the argument list when the run may use the network (CI), and that rule is a small callback per argument.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      The runner takes owned `String` arguments, so each borrowed `&str` of the constant argument arrays is copied once when the list is built.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      `arguments_for` hands the finished list to the runner, so the lazy filter and copy steps must run here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The constant argument arrays (`CHECK_ARGS`, `UPDATE_ARGS`, `VERIFY_ARGS`) stay untouched and reusable: the list is a walk over them, not a consumption.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      Filter, then copy to owned text, is the whole of `arguments_for`: drop `--offline` when online, then build the `Vec<String>` the runner takes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    return args.iter().filter(|argument| { return offline || **argument != "--offline"; }).map(ToString::to_string).collect();
}

/// The values of the `CI` variable that mean this is not a continuous integration run.
const NOT_CI_VALUES: [&str; 3] = ["", "0", "false"];

/// Whether the environment is a continuous integration run,
/// where `pnpm install` defaults `--frozen-lockfile` to true.
pub fn is_continuous_integration(ci_value: &str) -> bool {
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      `is_continuous_integration` compares the lower-cased `CI` value with the list of not-CI values without moving either.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    return !NOT_CI_VALUES.contains(&ci_value.to_lowercase().as_str());
}

/// The outcome of an install run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstallReport {
    /// Every crate root examined.
    pub crate_dirs: Vec<String>,
    /// Roots whose lockfile did not satisfy its manifests (updated,
    /// or only reported under `--dry-run`).
    pub stale: Vec<String>,
    /// Registry versions the install newly locked (kept or restored).
    pub added: Vec<LockedVersion>,
    /// Findings that refused the install.
    pub problems: Vec<String>,
    /// Findings reported without refusing.
    pub warnings: Vec<String>,
    /// Whether updated lockfiles were kept.
    pub written: bool,
}

/// Why an install did not complete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstallError {
    /// A frozen install found stale lockfiles.
    Stale {
        /// The repo-relative crate roots whose lockfile is stale.
        stale_dirs: Vec<String>,
    },
    /// Cargo could not bring a crate's lockfile in line.
    Cargo {
        /// The repo-relative crate directory.
        crate_dir: String,
        /// The command that failed.
        command: String,
        /// Cargo's output.
        output: String,
    },
    /// A registry lookup failed.
    Registry {
        /// What failed.
        message: String,
    },
    /// A file could not be read or written,
    /// or cargo could not be started.
    Io {
        /// What failed.
        message: String,
    },
}

/// What:
/// `impl fmt::Display for InstallError` lets an install failure be printed with `{}`.
/// Why:
/// The shadow and the CLI print the error to the user,
/// and each variant carries its own remedy text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class InstallError extends Error { toString() { ... } }
/// ```
impl fmt::Display for InstallError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the message of whichever variant the error is (a `match` over the cases).
    /// Why:
    /// A stale-lock failure names every stale root,
    /// a cargo failure shows cargo's output and the offline hint,
    /// so the user can act without reading the code.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { switch (this.kind) { case "stale": ...; case "cargo": ...; } }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      The message of an `InstallError` depends on which case failed (stale roots, cargo, registry, I/O), and the compiler checks that no case lacks a message.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        match self {
            InstallError::Stale { stale_dirs } => {
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      `writeln!` writes each line of the stale-lock message into the formatter, so the stale roots print one per line.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                writeln!(
                    formatter,
                    "Frozen install failed: {} Cargo.lock file(s) are present but need an update to satisfy their manifests.",
                    stale_dirs.len()
                // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
                // Why:      A failed write into the formatter ends the message at once and is reported to the caller of `fmt`.
                // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const value = await expr;   // throws to the caller on failure
                // ```
                )?;
                for dir in stale_dirs {
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // Macro `writeln!`: builds or prints text (or a list) from a template.
                    writeln!(formatter, "- {dir}")?;
                }
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Nothing was written. Frozen is the default in CI, as in pnpm; `--locked` or `--frozen` on the cargo command asks for it too.")?;
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(
                    formatter,
                    "To update them, run `cargo check` (or `cargo-shadow install --no-frozen-lockfile`) outside CI and commit the Cargo.lock changes."
                );
            }
            InstallError::Cargo { crate_dir, command, output } => {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "`{command}` failed in {crate_dir}; every lockfile was restored.")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Cargo output:\n{}", output.trim())?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "The install runs cargo offline, so it can pick only releases the local registry index and cache already hold.")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "When cargo says an index entry or package is missing, fetch it once with network access (`cargo fetch` in the crate)")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "and run the install again, or run `cargo update`, which resolves online under the release-age and trust policy.")?;
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "When cargo reports conflicting requirements, fix the manifest or the rust-catalog.toml floor that it names.");
            }
            InstallError::Registry { message } => {
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "A registry lookup failed; every lockfile was restored: {message}");
            }
            InstallError::Io { message } => {
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "{message}");
            }
        }
    }
}

/// What:
/// `impl From<RegistryError> for InstallError` lets a `RegistryError` convert into an `InstallError` automatically where `?` is used.
/// Why:
/// The policy checks fail with a registry error,
/// and the install step reports everything as an install error without wrapping each call by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS: catch (e) { throw new InstallError(e) }
/// ```
impl From<RegistryError> for InstallError {
    /// What:
    /// `fn from(error)` wraps the registry error's message in the `Registry` variant.
    /// Why:
    /// The variant tells the user the lockfiles were restored because a lookup failed,
    /// not because cargo did.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static from(error: RegistryError): InstallError { return new InstallError("registry", error.message); }
    /// ```
    fn from(error: RegistryError) -> InstallError {
        return InstallError::Registry { message: error.message };
    }
}

/// What an install run needs.
pub struct InstallRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots to examine,
    /// repo-relative.
    pub crate_dirs: &'a [String],
    /// Fail on a stale lockfile instead of updating it (pnpm's `--frozen-lockfile`,
    /// the CI default).
    pub frozen: bool,
    /// Keep every cargo call offline;
    /// false in CI,
    /// where no registry index is cached yet.
    pub offline: bool,
    /// Report what would be updated,
    /// then restore every lockfile.
    pub dry_run: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
    /// Makes the reader of one crate's lockfile text at the base revision (empty when absent there);
    /// an install under `trustLockfile` checks only the versions this reader's lockfiles did not hold.
    /// A factory,
    /// not a reader,
    /// because resolving the base starts git,
    /// which a call whose checks are all remembered must not pay for.
    pub lock_base: Option<LockBaseFactory<'a>>,
    /// How the base revision is spelled (`CARGO_SHADOW_TRUST_BASE`,
    /// or `HEAD`);
    /// part of the key of a remembered pass under `trustLockfile`,
    /// so a pass against one base is never taken for another.
    pub trust_base: &'a str,
    /// The freshness stamp already showed that every lockfile satisfies its manifests,
    /// so the staleness check (a cargo call) is skipped;
    /// the policy checks are not.
    pub known_consistent: bool,
}

/// A reader of one crate root's lockfile text at the base revision.
pub type LockBaseReader = Box<dyn Fn(&str) -> String>;

/// Makes a `LockBaseReader`,
/// or says why the base cannot be read.
pub type LockBaseFactory<'a> = &'a dyn Fn() -> Result<LockBaseReader, String>;

/// What:
/// `fn io_error(error) -> InstallError` wraps any displayable error as the `Io` variant of the install error.
/// Why:
/// `.map_err(io_error)` converts file and parse errors of the install steps without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ioError = (error: unknown) => new InstallError("io", String(error))
/// ```
fn io_error(error: impl fmt::Display) -> InstallError {
    return InstallError::Io { message: error.to_string() };
}

/// What:
/// `fn io_message(message) -> InstallError` wraps a message text as the `Io` variant.
/// Why:
/// Helpers that already return text (`Result<_, String>`) convert with `.map_err(io_message)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ioMessage = (message: string) => new InstallError("io", message)
/// ```
fn io_message(message: String) -> InstallError {
    return InstallError::Io { message };
}

/// What:
/// `fn registry_error(message) -> InstallError` wraps a message text as the `Registry` variant.
/// Why:
/// A failed policy lookup is reported as a registry failure,
/// so the user knows the lockfiles were restored because crates.io could not be asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registryError = (message: string) => new InstallError("registry", message)
/// ```
fn registry_error(message: String) -> InstallError {
    return InstallError::Registry { message };
}

/// Runs cargo in a crate root;
/// a start failure is an `Io` error.
fn run_cargo(request: &InstallRequest, crate_dir: &str, args: &[&str]) -> Result<CargoOutcome, InstallError> {
    return request
        .runner
        // Borrow `&request.root.join`: lends the value without copying or giving it away.
        .run(&request.root.join(crate_dir), &arguments_for(args, request.offline))
        // Closure `|message|`: an anonymous function, like a TS arrow function.
        // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
        // Why:      `io_message` converts the `String` error of `read_locks` into `InstallError::Io`, the error type `install_rust_lockfiles` returns, so `?` can pass it on.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { ... } catch (error) { throw new MyError(String(error)); }
        // ```
        .map_err(io_message);
}

/// Whether a crate root's lockfile no longer satisfies its manifests,
/// without writing anything.
pub fn is_stale(request: &InstallRequest, crate_dir: &str) -> Result<bool, InstallError> {
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      `is_stale` answers with a plain boolean, and a cargo that cannot even start comes back as an `InstallError`, never as a stale lockfile.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&CHECK_ARGS`: lends the value without copying or giving it away.
    return Ok(!run_cargo(request, crate_dir, &CHECK_ARGS)?.ok);
}

/// Runs one cargo command in a crate root and fails with the crate named when it exits non-zero.
fn run_or_fail(request: &InstallRequest, crate_dir: &str, args: &[&str]) -> Result<(), InstallError> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      `outcome` is declared as `CargoOutcome` so the reader sees a non-zero exit is data here, tested with `.ok`, not an error by itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let outcome: CargoOutcome = run_cargo(request, crate_dir, args)?;
    if !outcome.ok {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(InstallError::Cargo {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            crate_dir: crate_dir.to_string(),
            // Macro `format!`: builds or prints text (or a list) from a template.
            command: format!("cargo {}", arguments_for(args, request.offline).join(" ")),
            output: outcome.output,
        });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// The lockfile texts of snapshots.
fn sources_of(snapshots: &[LockSnapshot]) -> Vec<String> {
    // Closure `|snapshot|`: an anonymous function, like a TS arrow function.
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      `added_versions` takes plain lockfile texts, so `sources_of` copies the text out of each snapshot instead of passing the snapshot itself.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    return snapshots.iter().map(LockSnapshot::source_text).collect();
}

/// The crate roots whose lockfile is stale,
/// found with read-only checks.
pub fn stale_roots(request: &InstallRequest) -> Result<Vec<String>, InstallError> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      `stale` collects the crate roots found stale, one at a time, in the loop that follows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `stale: Vec<String>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      The list starts empty and only a root whose lockfile is out of date is added.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      Each stale root is kept as owned text so it can be moved into the error that names the stale roots.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      `stale` is a `Vec<String>` the function owns and fills, then moves into the error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut stale: Vec<String> = Vec::new();
    for crate_dir in request.crate_dirs {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        if is_stale(request, crate_dir)? {
            // `.clone()`: an independent owned copy.
            stale.push(crate_dir.clone());
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(stale);
}

/// What:
/// `fn invalid_lock_toml(message: String) -> InstallError` turns a TOML parse message of a `Cargo.lock` into an install error.
/// Why:
/// A lockfile that cannot be parsed stops the install with a message that names the file kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invalidLockToml = (message: string) => new InstallError(`a Cargo.lock is not valid TOML: ${message}`);
/// ```
fn invalid_lock_toml(message: String) -> InstallError {
    return InstallError::Io { message: format!("a Cargo.lock could not be read for its registry versions: {message}") };
}

/// What:
/// `pub fn verification_scope(settings, frozen, trust_base) -> String` names the check a remembered pass belongs to.
/// Why:
/// A pass of the frozen check says nothing about the local check (which also sets refused releases aside),
/// and a pass
/// against one base revision says nothing about another,
/// so the remembered passes are keyed by this text as well as by the
/// lockfile and the settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const verificationScope = (settings: Settings, frozen: boolean, trustBase: string): string
/// ```
pub fn verification_scope(settings: &Settings, frozen: bool, trust_base: &str) -> String {
    let mode: &str = if frozen { "frozen" } else { "local" };
    if settings.trust_lockfile {
        return format!("{mode}-since-base:{trust_base}");
    }
    if frozen {
        return ALL_ENTRIES_SCOPE.to_string();
    }
    return format!("{mode}-all-entries");
}

/// What:
/// `pub fn verification_cached(root, crate_dirs, settings, frozen, trust_base) -> bool` says whether every lockfile already
/// passed the policy check of this mode,
/// under these settings and this base.
/// Why:
/// The freshness stamp answers "do the lockfiles satisfy the manifests";
/// this answers "were they checked against the policy".
/// A call may take the fast path only when both are true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function verificationCached({ root, crateDirs, settings, frozen, trustBase }): boolean
/// ```
pub fn verification_cached(root: &Path, crate_dirs: &[String], settings: &Settings, frozen: bool, trust_base: &str) -> bool {
    let snapshots: Vec<LockSnapshot> = match read_locks(root, crate_dirs) {
        Ok(found) => found,
        Err(_) => return false,
    };
    let scope: String = verification_scope(settings, frozen, trust_base);
    for snapshot in &snapshots {
        if !is_verified(root, snapshot, settings, &scope) {
            return false;
        }
    }
    return true;
}

/// What:
/// `fn base_reader(request) -> Result<LockBaseReader, InstallError>` makes the base-revision lockfile reader.
/// Why:
/// An install under `trustLockfile = true` compares with the base,
/// and without a reader nothing can be trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function baseReader(request: InstallRequest): (crateDir: string) => string  // throws
/// ```
fn base_reader(request: &InstallRequest) -> Result<LockBaseReader, InstallError> {
    let factory: LockBaseFactory = request.lock_base.ok_or_else(|| {
        return InstallError::Io {
            message: "trustLockfile is true in rust-catalog.toml, but no base-revision lockfile reader was given, so nothing can be trusted".to_string(),
        };
    })?;
    return factory().map_err(io_message);
}

/// What:
/// `fn set_aside_pass(request, baseline, produced) -> Result<Vec<String>, InstallError>` sets every release the policy
/// refuses (among the versions `baseline` did not hold) aside for the newest older release that passes,
/// prints one line per move and returns the lines.
/// Why:
/// pnpm resolves to the newest release older than the cutoff instead of refusing,
/// and the owner decided (2026-11-03) that a
/// local install does the same for a committed lockfile:
/// the refused release never runs,
/// the older copy does.
/// The moves go through the transaction (`produced` records each lockfile this run wrote),
/// so another writer's lockfile is
/// never replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setAsidePass(request, baseline, produced): Promise<string[]>
/// ```
fn set_aside_pass(request: &InstallRequest, baseline: &[String], produced: &RefCell<Vec<LockSnapshot>>) -> Result<Vec<String>, InstallError> {
    let forward: Vec<String> = if request.offline { vec!["--offline".to_string()] } else { Vec::new() };
    let lines: Vec<String> = set_aside_until_clean(&SetAsideRequest {
        root: request.root,
        crate_dirs: request.crate_dirs,
        before: baseline,
        registry: request.registry,
        settings: request.settings,
        now: request.now,
        runner: request.runner,
        forward: &forward,
        produced,
    })
    .map_err(io_message)?;
    for line in &lines {
        eprintln!("cargo-shadow: {line}");
    }
    return Ok(lines);
}

/// What:
/// `fn update_stale_roots(request, before, produced) -> Result<InstallReport, InstallError>` is the transaction body:
/// update the stale roots,
/// set refused releases aside,
/// then check what is left;
/// restoring on refusal is the caller's.
/// Why:
/// Three sets of versions are held to the policy:
/// the ones this update newly locked,
/// and,
/// unless a remembered pass covers
/// them,
/// the existing entries (all of them,
/// or under `trustLockfile = true` the ones new since the base revision).
/// A refused one is set aside for the newest older release that passes;
/// only a refusal with no such release stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function updateStaleRoots(request, before, produced): Promise<InstallReport>
/// ```
fn update_stale_roots(request: &InstallRequest, before: &[LockSnapshot], produced: &RefCell<Vec<LockSnapshot>>) -> Result<InstallReport, InstallError> {
    let mut stale: Vec<String> = Vec::new();
    for crate_dir in request.crate_dirs {
        // The stamp already showed this root consistent, so no cargo call is spent on asking again.
        if request.known_consistent || !is_stale(request, crate_dir)? {
            eprintln!("cargo-shadow: {crate_dir}: lockfile already satisfies its manifests");
            continue;
        }
        eprintln!("cargo-shadow: {crate_dir}: lockfile is stale, updating the workspace offline");
        run_or_fail(request, crate_dir, &UPDATE_ARGS)?;
        // Review finding (early-error rollback): this root now holds what this run's cargo wrote, so a rollback may replace it
        // (and only while it still holds this); roots the run never reached are never recorded and never restored.
        note_produced(request.root, crate_dir, produced);
        run_or_fail(request, crate_dir, &VERIFY_ARGS)?;
        stale.push(crate_dir.clone());
    }
    let before_sources: Vec<String> = sources_of(before);
    let mut baselines: Vec<Vec<String>> = vec![before_sources.clone()];
    let mut notes: Vec<String> = Vec::new();
    // The versions this update newly locked.
    if !stale.is_empty() {
        notes.extend(set_aside_pass(request, &before_sources, produced)?);
    }
    // The existing entries, unless a remembered pass covers this exact lockfile text, policy and base.
    let scope: String = verification_scope(request.settings, false, request.trust_base);
    let covered: bool = verification_cached(request.root, request.crate_dirs, request.settings, false, request.trust_base);
    if !covered {
        let baseline: Vec<String> = if request.settings.trust_lockfile {
            let reader: LockBaseReader = base_reader(request)?;
            request.crate_dirs.iter().map(|dir| { return reader(dir); }).collect()
        } else {
            Vec::new()
        };
        notes.extend(set_aside_pass(request, &baseline, produced)?);
        baselines.push(baseline);
    }
    let after: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(io_error)?;
    let after_sources: Vec<String> = sources_of(&after);
    let added: Vec<LockedVersion> = added_versions(&before_sources, &after_sources).map_err(invalid_lock_toml)?;
    // Everything either baseline did not hold is checked, once.
    let mut checked: BTreeSet<LockedVersion> = BTreeSet::new();
    for baseline in &baselines {
        for version in added_versions(baseline, &after_sources).map_err(invalid_lock_toml)? {
            checked.insert(version);
        }
    }
    let versions: Vec<LockedVersion> = checked.into_iter().collect();
    let age: CheckOutcome = if versions.is_empty() {
        CheckOutcome::default()
    } else {
        check_release_age(&versions, request.settings, request.registry, request.now)?
    };
    let trust: Vec<String> = if versions.is_empty() { Vec::new() } else { check_trust(&versions, request.settings, request.registry, request.now)? };
    let mut problems: Vec<String> = age.problems;
    problems.extend(trust);
    // Only a pass is remembered, and only for the text that is live now.
    if problems.is_empty() && !covered {
        for snapshot in &after {
            mark_verified(request.root, snapshot, request.settings, &scope);
        }
    }
    let mut warnings: Vec<String> = age.warnings;
    warnings.extend(notes);
    let keep: bool = problems.is_empty() && !request.dry_run;
    return Ok(InstallReport {
        crate_dirs: request.crate_dirs.to_vec(),
        stale,
        added,
        problems,
        warnings,
        written: keep,
    });
}

/// The policy findings for the registry versions `after` holds that `before` did not;
/// used to judge a killed run's leftovers.
pub fn policy_findings(settings: &Settings, registry: &dyn Registry, now: i64, before: &[LockSnapshot], after: &[LockSnapshot]) -> Vec<String> {
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&sources_of`: lends the value without copying or giving it away.
    // Typed binding: `added: Vec<LockedVersion>`.
    let added: Vec<LockedVersion> = match added_versions(&sources_of(before), &sources_of(after)) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `vec!`: builds or prints text (or a list) from a template.
        Err(message) => return vec![format!("a Cargo.lock could not be read for its registry versions: {message}")],
    };
    if added.is_empty() {
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Vec::new();
    }
    // `let mut`: a binding that may change.
    // Typed binding: `findings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut findings: Vec<String> = Vec::new();
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&added`: lends the value without copying or giving it away.
    match check_release_age(&added, settings, registry, now) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(outcome) => findings.extend(outcome.problems),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => findings.push(error.message),
    }
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&added`: lends the value without copying or giving it away.
    match check_trust(&added, settings, registry, now) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(problems) => findings.extend(problems),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => findings.push(error.message),
    }
    return findings;
}

/// What:
/// `pub fn install_rust_lockfiles(request: &InstallRequest) -> Result<InstallReport, InstallError>` runs the install over the
/// listed crate roots.
/// Why:
/// A frozen install only checks and fails naming the stale roots.
/// A non-frozen install first finishes any run that was killed
/// (`journal.rs`),
/// then snapshots every lockfile,
/// updates the stale ones,
/// checks the newly locked versions,
/// and puts the
/// snapshots back on a failure,
/// a refusal or a dry run,
/// but only into lockfiles that still hold what this run produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function installRustLockfiles(args): Promise<InstallReport>  // throws StaleLockfilesError, CargoInstallError
/// ```
pub fn install_rust_lockfiles(request: &InstallRequest) -> Result<InstallReport, InstallError> {
    if request.frozen {
        // The stamp showed the lockfiles consistent, so no cargo call asks again; the policy check below still runs.
        if !request.known_consistent {
            let stale: Vec<String> = stale_roots(request)?;
            if !stale.is_empty() {
                eprintln!("cargo-shadow: frozen install: {} stale lockfiles, nothing written", stale.len());
                return Err(InstallError::Stale { stale_dirs: stale });
            }
        }
        let locks: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(io_error)?;
        let problems: Vec<String> = if request.settings.trust_lockfile {
            let scope: String = verification_scope(request.settings, true, request.trust_base);
            let mut pending: Vec<LockSnapshot> = Vec::new();
            for snapshot in &locks {
                if !is_verified(request.root, snapshot, request.settings, &scope) {
                    pending.push(snapshot.clone());
                }
            }
            if pending.is_empty() {
                Vec::new()
            } else {
                let read_base: LockBaseReader = base_reader(request)?;
                let found: Vec<String> = verify_added_since_base(&pending, &*read_base, request.settings, request.registry, request.now).map_err(registry_error)?;
                if found.is_empty() {
                    for snapshot in &pending {
                        mark_verified(request.root, snapshot, request.settings, &scope);
                    }
                }
                found
            }
        } else {
            verify_locked_entries(request.root, &locks, request.settings, request.registry, request.now).map_err(registry_error)?
        };
        return Ok(InstallReport { crate_dirs: request.crate_dirs.to_vec(), problems, ..InstallReport::default() });
    }
    // Closure `|old, live|`: an anonymous function, like a TS arrow function.
    // Borrow `&process_exists`: lends the value without copying or giving it away.
    // Typed binding: `recovered: Vec<String>`.
    let recovered: Vec<String> = recover(request.root, &process_exists, &|old, live| { return policy_findings(request.settings, request.registry, request.now, old, live); });
    // Borrow `&recovered`: lends the value without copying or giving it away.
    for message in &recovered {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: {message}");
    }
    // Typed binding: `before: Vec<LockSnapshot>`.
    let before: Vec<LockSnapshot> =
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        read_locks(request.root, request.crate_dirs).map_err(io_error)?;
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("cargo-shadow: snapshotted {} lockfiles", before.len());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Review finding (failed begin): without a complete recovery record nothing may be mutated, so a journal that cannot be
    // started stops the install here, before the first cargo call.
    begin(request.root, &before).map_err(io_error)?;
    // What:     `RefCell<Vec<LockSnapshot>>` is a list that can be added to through a shared reference (sibling: `&mut Vec`, which needs exclusive access); `RefCell::new(Vec::new())` makes an empty one.
    // Why:      `produced` starts empty because cargo has not written yet; each cargo write is recorded the moment it succeeded, so a failure never has to guess what the run wrote.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const produced: LockSnapshot[] = [];
    // ```
    let produced: RefCell<Vec<LockSnapshot>> = RefCell::new(Vec::new());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Typed binding: `outcome: Result<InstallReport, InstallError>`.
    let outcome: Result<InstallReport, InstallError> = update_stale_roots(request, &before, &produced);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&outcome`: lends the value without copying or giving it away.
    // Typed binding: `keep: bool`.
    // Macro `matches!`: builds or prints text (or a list) from a template.
    let keep: bool = matches!(&outcome, Ok(report) if report.written);
    // `let mut`: a binding that may change.
    // Typed binding: `restore_warnings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut restore_warnings: Vec<String> = Vec::new();
    if !keep {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: restoring every lockfile that this run changed");
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&before`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        restore_warnings = restore_if_unchanged(request.root, &before, &produced.borrow()).map_err(io_error)?;
        // Borrow `&restore_warnings`: lends the value without copying or giving it away.
        for warning in &restore_warnings {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: warning: {warning}");
        }
    }
    finish(request.root);
    // Closure `|mut report|`: an anonymous function, like a TS arrow function.
    // Adapter `.map(`: transforms or selects items with a closure.
    return match outcome {
        Ok(mut report) => {
            report.warnings.extend(recovered);
            report.warnings.extend(restore_warnings);
            Ok(report)
        }
        Err(error) => Err(error),
    };
}
