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
/// `use crate::journal::{begin, finish, process_exists, recover, restore_if_unchanged};` imports `begin`,
/// `finish`,
/// `process_exists`,
/// `recover`,
/// `restore_if_unchanged` from another module of this crate so this file can name them unqualified.
/// Why:
/// `begin` is used by `install_rust_lockfiles`;
/// `finish` is used by `install_rust_lockfiles`;
/// `process_exists` is used by `install_rust_lockfiles`;
/// `recover` is used by `install_rust_lockfiles`;
/// `restore_if_unchanged` is used by `install_rust_lockfiles`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { begin, finish, process_exists, recover, restore_if_unchanged } from 'crate/journal';
/// ```
use crate::journal::{begin, finish, process_exists, recover, restore_if_unchanged};
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
/// What:
/// `use crate::verify::{verify_added_since_base, verify_locked_entries};` imports `verify_added_since_base`,
/// `verify_locked_entries` from another module of this crate so this file can name them unqualified.
/// Why:
/// `verify_added_since_base` is used by `install_rust_lockfiles`;
/// `verify_locked_entries` is used by `install_rust_lockfiles`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { verify_added_since_base, verify_locked_entries } from 'crate/verify';
/// ```
use crate::verify::{verify_added_since_base, verify_locked_entries};

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
    /// Reads one crate's lockfile text at the base revision (empty when absent there);
    /// a frozen install under `trustLockfile`
    /// checks only the versions this reader's lockfiles did not hold.
    pub lock_base: Option<&'a dyn Fn(&str) -> String>,
}

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

/// The transaction body:
/// update the stale roots,
/// then check the new versions;
/// restoring on refusal is the caller's.
fn update_stale_roots(request: &InstallRequest, before: &[LockSnapshot], produced: &mut Option<Vec<LockSnapshot>>) -> Result<InstallReport, InstallError> {
    // `let mut`: a binding that may change.
    // Typed binding: `stale: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut stale: Vec<String> = Vec::new();
    for crate_dir in request.crate_dirs {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        if !is_stale(request, crate_dir)? {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: {crate_dir}: lockfile already satisfies its manifests");
            continue;
        }
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: {crate_dir}: lockfile is stale, updating the workspace offline");
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&UPDATE_ARGS`: lends the value without copying or giving it away.
        run_or_fail(request, crate_dir, &UPDATE_ARGS)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&VERIFY_ARGS`: lends the value without copying or giving it away.
        run_or_fail(request, crate_dir, &VERIFY_ARGS)?;
        // `.clone()`: an independent owned copy.
        stale.push(crate_dir.clone());
    }
    // Typed binding: `after: Vec<LockSnapshot>`.
    let after: Vec<LockSnapshot> =
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        read_locks(request.root, request.crate_dirs).map_err(io_error)?;
    // What cargo produced, taken before the slow registry checks: a restore later replaces a lockfile only if it still holds this.
    *produced = Some(after.clone());
    // Borrow `&sources_of`: lends the value without copying or giving it away.
    // Typed binding: `added: Vec<LockedVersion>`.
    let added: Vec<LockedVersion> = added_versions(&sources_of(before), &sources_of(&after))
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|message|`: an anonymous function, like a TS arrow function.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Macro `format!`: builds or prints text (or a list) from a template.
        .map_err(invalid_lock_toml)?;
    // Typed binding: `age: CheckOutcome`.
    let age: CheckOutcome = if added.is_empty() {
        CheckOutcome::default()
    } else {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&added`: lends the value without copying or giving it away.
        check_release_age(&added, request.settings, request.registry, request.now)?
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      The trust lookup runs only when the install added versions, because an unchanged lockfile has nothing new to check and costs no request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Borrow `&added`: lends the value without copying or giving it away.
    // Typed binding: `trust: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let trust: Vec<String> = if added.is_empty() { Vec::new() } else { check_trust(&added, request.settings, request.registry, request.now)? };
    // `let mut`: a binding that may change.
    // Typed binding: `problems: Vec<String>`.
    let mut problems: Vec<String> = age.problems;
    problems.extend(trust);
    // Typed binding: `keep: bool`.
    let keep: bool = problems.is_empty() && !request.dry_run;
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(InstallReport {
        // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
        // Why:      The report owns its crate roots, so `to_vec` copies them out of the request's borrowed slice.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // [...slice]
        // ```
        crate_dirs: request.crate_dirs.to_vec(),
        stale,
        added,
        problems,
        warnings: age.warnings,
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
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `stale: Vec<String>`.
        let stale: Vec<String> = stale_roots(request)?;
        if !stale.is_empty() {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: frozen install: {} stale lockfiles, nothing written", stale.len());
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(InstallError::Stale { stale_dirs: stale });
        }
        // Typed binding: `locks: Vec<LockSnapshot>`.
        let locks: Vec<LockSnapshot> =
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            read_locks(request.root, request.crate_dirs).map_err(io_error)?;
        // `trustLockfile = true` (the owner's decision of 2026-10-29): committed lockfiles are trusted, so only the versions no
        // lockfile held at the base revision are checked. With it false, pnpm's default: every lockfile entry is re-validated
        // against the release-age and trust policy, and a lockfile that passed before is remembered by its digest (verify.rs).
        let problems: Vec<String> = if request.settings.trust_lockfile {
            // Closure `||`: an anonymous function, like a TS arrow function.
            // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
            // Why:      `ok_or_else` turns a missing base reader (`None`) into the `Io` error that says `trustLockfile` needs a base revision, instead of trusting everything silently.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { return compute(); } catch { return undefined; }
            // ```
            let read_base = request.lock_base.ok_or_else(|| {
                return InstallError::Io {
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                    message: "trustLockfile is true in rust-catalog.toml, but no base-revision lockfile reader was given, so nothing can be trusted".to_string(),
                };
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            })?;
            // Borrow `&locks`: lends the value without copying or giving it away.
            verify_added_since_base(&locks, read_base, request.settings, request.registry, request.now)
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Closure `|message|`: an anonymous function, like a TS arrow function.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                .map_err(registry_error)?
        } else {
            // Borrow `&locks`: lends the value without copying or giving it away.
            verify_locked_entries(request.root, &locks, request.settings, request.registry, request.now)
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Closure `|message|`: an anonymous function, like a TS arrow function.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                .map_err(registry_error)?
        };
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.to_vec()`: copies the slice into a new owned `Vec`.
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
    // Typed binding: `journaled: bool`.
    let journaled: bool = begin(request.root, &before).is_ok();
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      `produced` starts as `None` because cargo has not written yet; the update step fills it with what cargo produced, before the slow registry checks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `produced: Option<Vec<LockSnapshot>>`.
    let mut produced: Option<Vec<LockSnapshot>> = None;
    // Borrow `&before`: lends the value without copying or giving it away.
    // Typed binding: `outcome: Result<InstallReport, InstallError>`.
    let outcome: Result<InstallReport, InstallError> = update_stale_roots(request, &before, &mut produced);
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
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `state: Vec<LockSnapshot>`.
        let state: Vec<LockSnapshot> = match produced {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found,
            // `None`: the present or the absent value of an `Option`.
            // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
            // Why:      If the lockfiles cannot be re-read after a failure, an empty list gives no sign of a concurrent writer, so every lockfile that differs from its snapshot is restored.
            // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // value ?? fallback
            // ```
            None => read_locks(request.root, request.crate_dirs).unwrap_or_default(),
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&before`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        restore_warnings = restore_if_unchanged(request.root, &before, &state).map_err(io_error)?;
        // Borrow `&restore_warnings`: lends the value without copying or giving it away.
        for warning in &restore_warnings {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: warning: {warning}");
        }
    }
    if journaled {
        finish(request.root);
    }
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
