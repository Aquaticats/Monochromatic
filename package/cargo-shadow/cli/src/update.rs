//! What:
//! The catalog update,
//! the counterpart of `pnpm update`:
//! moves lockfiles to newer releases under the release-age and trust
//! policy in `rust-catalog.toml`,
//! as one transaction.
//! Why:
//! pnpm's `update` is its own policy-aware command (<https://pnpm.io/cli/update>,
//! read 2026-11-03):
//! it moves dependencies
//! within their ranges and applies `minimumReleaseAge` and `trustPolicy` while it chooses.
//! Cargo's resolver applies neither,
//! so the shadow answers `cargo update` with this procedure instead of passing it to cargo's resolver.
//!
//! Two kinds.
//! The full kind re-resolves each crate root with
//! `cargo -Zunstable-options generate-lockfile --publish-time <now minus minimumReleaseAge>`,
//! which picks the newest releases
//! published before the cutoff (only `generate-lockfile` accepts `--publish-time`,
//! re-probed 2026-10-29),
//! then moves crates
//! the catalog exempts from the release age to their newest release.
//! The targeted kind runs the user's `cargo update` command
//! (named packages,
//! `--precise`,
//! `--breaking`) as given.
//! Both then collect the registry versions no lockfile held before,
//! set aside what the policy refuses,
//! check what is left,
//! and keep the new lockfiles only when nothing failed and it is not a
//! dry run;
//! otherwise every lockfile is restored.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update.ts
//! export async function updateRustLockfiles({ root, dryRun, now, runCargo, registry }): Promise<UpdateReport>
//! ```

/// Imports `RefCell`,
/// a list or value that can be changed through a shared reference;
/// it holds the record of the lockfiles this run wrote.
use std::cell::RefCell;
/// What:
/// `use std::fmt;` imports `fmt` from the standard library so this file can name it unqualified.
/// Why:
/// `fmt` is used by the `impl` of `UpdateError`,
/// `fmt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fmt } from 'std';
/// ```
use std::fmt;
/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `UpdateRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports `Settings` from the external crate `monochromatic_dependency_catalog` so this file can name it unqualified.
/// Why:
/// `Settings` is used by `UpdateRequest`,
/// `run_update`.
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
/// `CargoOutcome` is used by `run_or_fail`,
/// `resolve_crate`;
/// `CargoRunner` is used by `UpdateRequest`.
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
/// `CheckOutcome` is used by `run_update`;
/// `check_release_age` is used by `run_update`;
/// `check_trust` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CheckOutcome, check_release_age, check_trust } from 'crate/checks';
/// ```
use crate::checks::CheckOutcome;
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports `LockSnapshot`,
/// `read_locks` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockSnapshot` is used by `resolve_all`,
/// `sources_of`,
/// `run_update`,
/// `update_rust_lockfiles`;
/// `read_locks` is used by `resolve_all`,
/// `run_update`,
/// `update_rust_lockfiles`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, read_locks } from 'crate/crates';
/// ```
use crate::crates::{LockSnapshot, read_locks};
/// What:
/// `use crate::install::{LockBaseFactory, policy_findings};` imports `policy_findings` from another module of this crate so this file can name it unqualified.
/// Why:
/// `policy_findings` is used by `update_rust_lockfiles`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { policy_findings } from 'crate/install';
/// ```
use crate::install::{LockBaseFactory, policy_findings};
/// What:
/// `use crate::journal::{begin, finish, note_produced, process_exists, recover, restore_if_unchanged};` imports `begin`,
/// `finish`,
/// `process_exists`,
/// `recover`,
/// `restore_if_unchanged` from another module of this crate so this file can name them unqualified.
/// Why:
/// `begin` is used by `update_rust_lockfiles`;
/// `finish` is used by `update_rust_lockfiles`;
/// `process_exists` is used by `update_rust_lockfiles`;
/// `recover` is used by `update_rust_lockfiles`;
/// `restore_if_unchanged` is used by `update_rust_lockfiles`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { addManifests, begin, finish, processExists, recover, restoreIfUnchanged, runRecording } from 'crate/journal';
/// ```
use crate::journal::{begin, recover, restore_if_unchanged, run_recording, Transaction};
/// Imports `Claim`,
/// what the ownership evidence says about a file a child wrote.
use crate::file_owner::Claim;
/// Imports the manifest transaction that `--breaking` needs:
/// cargo rewrites `Cargo.toml` as well as the lockfile.
use crate::manifest_edit::{ManifestEdit, restore_manifests, run_recording_manifests, snapshot_manifests};
/// Imports the manifest list of a `cargo metadata` answer.
use crate::stamp::manifests_from_metadata;
/// Imports the keep rule of a rollback and the quarantine directory,
/// where the files a rollback displaces are kept.
use crate::journal::{KeepAs, QUARANTINE_DIR, ReplaceOutcome, replace_keeping};
/// What:
/// `use crate::lockfile::{LockedVersion, added_versions, parse_registry_versions, registry_versions};` imports `LockedVersion`,
/// `added_versions`,
/// `parse_registry_versions`,
/// `registry_versions` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockedVersion` is used by `UpdateReport`,
/// `run_update`;
/// `added_versions` is used by `run_update`;
/// `parse_registry_versions` is used by `excluded_update_specs`;
/// `registry_versions` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion, added_versions, parse_registry_versions, registry_versions } from 'crate/lockfile';
/// ```
use crate::lockfile::{LockedVersion, added_versions, parse_registry_versions, registry_versions};
/// Imports the verification an install runs on the lockfiles it leaves.
use crate::update_verify::verify_update;
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports the check of the release a `--precise` names.
/// What:
/// `use crate::prune::prune_excludes;` imports `prune_excludes` from another module of this crate so this file can name it unqualified.
/// Why:
/// `prune_excludes` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { prune_excludes } from 'crate/prune';
/// ```
use crate::prune::prune_excludes;
/// What:
/// `use crate::registry::Registry;` imports `Registry` from another module of this crate so this file can name it unqualified.
/// Why:
/// `Registry` is used by `UpdateRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;
/// What:
/// `use crate::selectors::{matches_pattern, parse_selector};` imports `matches_pattern`,
/// `parse_selector` from another module of this crate so this file can name them unqualified.
/// Why:
/// `matches_pattern` is used by `excluded_update_specs`;
/// `parse_selector` is used by `excluded_update_specs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { matches_pattern, parse_selector } from 'crate/selectors';
/// ```
use crate::selectors::{matches_pattern, parse_selector};
/// What:
/// `use crate::set_aside::{SetAsideRequest, set_aside_until_clean};` imports `SetAsideRequest`,
/// `set_aside_until_clean` from another module of this crate so this file can name them unqualified.
/// Why:
/// `SetAsideRequest` is used by `run_update`;
/// `set_aside_until_clean` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SetAsideRequest, set_aside_until_clean } from 'crate/set_aside';
/// ```
use crate::set_aside::{SetAsideRequest, set_aside_until_clean};
/// What:
/// `use crate::tighten::{Syncer, raise_floors, raised_names};` imports `Syncer`,
/// `raise_floors`,
/// `raised_names` from another module of this crate so this file can name them unqualified.
/// Why:
/// `Syncer` is used by `UpdateRequest`;
/// `raise_floors` is used by `run_update`;
/// `raised_names` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Syncer, raise_floors, raised_names } from 'crate/tighten';
/// ```
use crate::tighten::{Syncer, raise_floors, raised_names};
/// What:
/// `use crate::time_text::publish_time_cutoff;` imports `publish_time_cutoff` from another module of this crate so this file can name it unqualified.
/// Why:
/// `publish_time_cutoff` is used by `run_update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { publish_time_cutoff } from 'crate/time_text';
/// ```
use crate::time_text::publish_time_cutoff;
/// What:
/// `use crate::update_resolve::resolve_all;` imports the resolution of every crate root of a full update.
/// Why:
/// `run_update` calls it for `UpdateKind::Full`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { resolveAll } from 'crate/update_resolve';
/// ```
use crate::update_resolve::resolve_all;
/// Imports the file-system error module (the result type of the recorded calls).
use std::io;

/// Which procedure the update runs.
pub enum UpdateKind<'a> {
    /// Re-resolve every listed crate root under the release-age cutoff (`pnpm update` with no names).
    Full,
    /// Run the user's `cargo update` arguments as given (`pnpm update <name>[@<version>]`).
    Targeted {
        /// The complete cargo arguments,
        /// starting with `update`.
        cargo_args: &'a [String],
    },
}

/// What an update needs.
pub struct UpdateRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots to update,
    /// repo-relative.
    pub crate_dirs: &'a [String],
    /// Report without keeping anything.
    pub dry_run: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The catalog file,
    /// edited to prune stale excludes after a kept full update.
    pub catalog_path: &'a Path,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
    /// Cargo options forwarded to every cargo call (`--offline`,
    /// `--config`,
    /// `-Z`,
    /// `--color`,
    /// `-q`,
    /// `-v`).
    pub forward: &'a [String],
    /// Renders the catalog into the manifests after `--breaking` raised floors.
    pub syncer: &'a dyn Syncer,
    /// Prune exclude selectors that no locked version needs.
    /// Only an update that covers every lockfile of the repository may
    /// say that nothing needs a selector;
    /// an update of some workspaces must leave the lists alone.
    pub prune_catalog: bool,
    /// Makes the reader of one crate's lockfile text at the trust base (empty when absent there);
    /// under `trustLockfile` the update checks every version the base did not hold,
    /// as an install does.
    /// A factory,
    /// because resolving the base may start git.
    pub lock_base: Option<LockBaseFactory<'a>>,
}

/// The outcome of an update.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateReport {
    /// The crate roots updated.
    pub crate_dirs: Vec<String>,
    /// Registry versions newly locked (kept or restored).
    pub added: Vec<LockedVersion>,
    /// Findings that refused the update.
    pub problems: Vec<String>,
    /// Findings reported without refusing,
    /// including set-asides.
    pub warnings: Vec<String>,
    /// Roots resolved without the release-age cutoff (`minimumReleaseAgeStrict = false`).
    pub fell_back: Vec<String>,
    /// Whether the new lockfiles were kept.
    pub written: bool,
    /// Exclude selectors pruned from the catalog.
    pub pruned: Vec<String>,
}

/// Why an update did not complete;
/// every lockfile was restored first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateError {
    /// Cargo could not resolve a crate.
    Cargo {
        /// The repo-relative crate directory.
        crate_dir: String,
        /// The command that failed.
        command: String,
        /// Cargo's output.
        output: String,
    },
    /// A registry lookup,
    /// file operation or parse failed.
    Other {
        /// What failed.
        message: String,
    },
}

/// What:
/// `impl fmt::Display for UpdateError` lets an update failure be printed with `{}`.
/// Why:
/// The shadow and the CLI print the error,
/// and a cargo failure carries the remedies for a release-age or toolchain problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UpdateError extends Error { toString() { ... } }
/// ```
impl fmt::Display for UpdateError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the message of whichever variant the error is (a `match` over the cases).
    /// Why:
    /// A cargo failure shows cargo's output plus the release-age and toolchain remedies,
    /// so the user can act without reading the code.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.kind === "cargo" ? this.cargoMessage() : this.message; }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      The message of an `UpdateError` differs for a cargo failure (command, output, remedies) and for the catch-all, and the compiler checks both are covered.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        match self {
            UpdateError::Cargo { crate_dir, command, output } => {
                // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
                // Why:      A failed write into the formatter ends the message and is reported to the caller of `fmt`.
                // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const value = await expr;   // throws to the caller on failure
                // ```
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      `writeln!` writes the failed command and crate on the first line and each remedy on its own line.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                writeln!(formatter, "`{command}` failed in {crate_dir}; every lockfile this run changed was put back (a file another process wrote meanwhile is kept and named in the warnings).")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Cargo output:\n{}", output.trim())?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "Fix the cause cargo names. When it is that no release old enough satisfies a requirement,")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "either wait, add the crate to minimumReleaseAgeExclude in rust-catalog.toml,")?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Macro `writeln!`: builds or prints text (or a list) from a template.
                writeln!(formatter, "or set minimumReleaseAgeStrict = false to fall back to a younger release.")?;
                // Macro `write!`: builds or prints text (or a list) from a template.
                return write!(formatter, "When cargo says `-Z` needs the nightly channel, set RUSTUP_TOOLCHAIN to the nightly toolchain.lock.json pins.");
            }
            // Macro `write!`: builds or prints text (or a list) from a template.
            UpdateError::Other { message } => return write!(formatter, "{message}"),
        }
    }
}

/// What:
/// `fn run_or_fail(request, crate_dir, args) -> Result<CargoOutcome, UpdateError>` runs one cargo command in a crate root and turns a non-zero exit into an error naming the crate.
/// `Result<T, E>` is Rust's success-or-failure value (sibling:
/// `Option<T>`,
/// a value or nothing);
/// `?` on one returns early with the failure.
/// Why:
/// Every cargo step of an update fails the same way,
/// so the message is in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runOrFail(args): Promise<CargoOutcome>  // throws
/// ```
pub(crate) fn run_or_fail(request: &UpdateRequest, crate_dir: &str, args: &[String]) -> Result<CargoOutcome, UpdateError> {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      `outcome` is declared as `CargoOutcome`; only the lines after it turn a non-zero exit into an `UpdateError::Cargo`, so the type states what the runner returned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let outcome: CargoOutcome = request
        .runner
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      The runner needs the crate root path and the arguments for the call only, so both are lent.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        .run(&request.root.join(crate_dir), args)
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      `other` is a named function, not a closure, so the runner's `String` error becomes the catch-all `UpdateError` in one `map_err` call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
        // Why:      `run_or_fail` hands the runner's `String` error to `other`, which makes it the catch-all `UpdateError`, so `?` can return it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { ... } catch (error) { throw new MyError(String(error)); }
        // ```
        .map_err(other)?;
    if !outcome.ok {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      `run_or_fail` returns a cargo failure as an `UpdateError::Cargo` value naming the crate and the command, so the caller restores and reports.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      The error stores the crate directory and the command as owned text because it outlives this call and is printed by the caller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(UpdateError::Cargo { crate_dir: crate_dir.to_string(), command: format!("cargo {}", args.join(" ")), output: outcome.output });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(outcome);
}

/// What:
/// `fn matches_any_pattern(patterns, name) -> bool` tells whether a crate name fits at least one name pattern.
/// Why:
/// A crate that an exemption pattern names is updated on its own,
/// outside the release-age cutoff.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const matchesAnyPattern = (patterns: string[], name: string) => patterns.some((p) => matchesPattern(p, name));
/// ```
fn matches_any_pattern(patterns: &[String], name: &str) -> bool {
    for pattern in patterns {
        if matches_pattern(pattern, name) {
            return true;
        }
    }
    return false;
}

/// What:
/// `pub fn excluded_update_specs(lock: &str, selectors: &[String]) -> Result<Vec<String>, String>` lists `name@version` for
/// each locked registry crate that a name-only selector exempts from the release age.
/// Why:
/// Those crates move to their newest release with `cargo update`;
/// a selector naming exact versions only exempts those
/// versions from the checks and does not move the crate,
/// because the cutoff already chose its version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function excludedUpdateSpecs({ lock, selectors }): string[]
/// ```
pub fn excluded_update_specs(lock: &str, selectors: &[String]) -> Result<Vec<String>, String> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      `excluded_update_specs` collects the name-only patterns one selector at a time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `patterns: Vec<String>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      No pattern exists until a selector is read, and a catalog without name-only selectors yields none.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      Patterns are copied out of the selectors into owned text so the returned list owns what it holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The pattern list is only read afterwards, so a `Vec` owned by this function is enough.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut patterns: Vec<String> = Vec::new();
    for selector in selectors {
        let parsed = parse_selector(selector);
        if parsed.versions.is_empty() {
            patterns.push(parsed.pattern);
        }
    }
    if patterns.is_empty() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Ok(Vec::new());
    }
    // `let mut`: a binding that may change.
    // Typed binding: `specs: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut specs: Vec<String> = Vec::new();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    for locked in parse_registry_versions(lock)? {
        // Closure `|pattern|`: an anonymous function, like a TS arrow function.
        // Borrow `&locked.name`: lends the value without copying or giving it away.
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      Matching each locked crate against every pattern reads the patterns without consuming them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        if matches_any_pattern(&patterns, &locked.name) {
            specs.push(locked.key());
        }
    }
    specs.sort();
    specs.dedup();
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(specs);
}



/// What:
/// `fn sources_of(snapshots) -> Vec<String>` copies the lockfile texts out of snapshots;
/// `.clone()` makes an independent copy of owned text.
/// Why:
/// The lockfile diff takes plain texts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sourcesOf = (snapshots) => snapshots.map((s) => s.source)
/// ```
fn sources_of(snapshots: &[LockSnapshot]) -> Vec<String> {
    // Closure `|snapshot|`: an anonymous function, like a TS arrow function.
    // `.clone()`: an independent owned copy.
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      `sources_of` returns plain lockfile texts to the diff, so the lazy copies must run here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    return snapshots.iter().map(LockSnapshot::source_text).collect();
}

/// What:
/// `fn other(message) -> UpdateError` wraps a message in the catch-all update error.
/// Why:
/// `map_err(other)` turns a plain text failure into the error type of the function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const other = (message: string) => new UpdateError(message)
/// ```
pub(crate) fn other(message: String) -> UpdateError {
    return UpdateError::Other { message };
}

/// What:
/// `fn other_text(error) -> UpdateError` wraps any displayable error as the catch-all update error (`impl Display`:
/// any type that prints with `{}`).
/// Why:
/// `.map_err(other_text)` converts the I/O and parse errors of the update steps without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const otherText = (error: unknown) => new UpdateError(String(error))
/// ```
pub(crate) fn other_text(error: impl fmt::Display) -> UpdateError {
    return other(error.to_string());
}

/// What:
/// `fn is_precise_option(argument: &str) -> bool` tells whether one cargo argument is `--precise` in either spelling (`--precise x` or `--precise=x`).
/// Why:
/// An update that pins an exact version must not have another version set aside,
/// so this run skips the set-aside step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPreciseOption = (argument: string) => argument === '--precise' || argument.startsWith('--precise=');
/// ```
fn is_precise_option(argument: &str) -> bool {
    return argument == "--precise" || argument.starts_with("--precise=");
}

/// What:
/// `fn is_breaking_option(argument: &str) -> bool` tells whether one cargo argument is `--breaking`.
/// Why:
/// Only a `--breaking` update raises the catalog floors,
/// as `pnpm update --latest` raises the ranges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isBreakingOption = (argument: string) => argument === '--breaking';
/// ```
fn is_breaking_option(argument: &str) -> bool {
    return argument == "--breaking";
}

/// What:
/// `fn sync_recording(request, edits) -> io::Result<Result<String, String>>` runs the manifest sync with every crate root's manifests
/// recorded,
/// so what a sync that fails halfway wrote goes back with the transaction.
/// Why:
/// Review finding (2c 11):
/// the sync rewrites manifests,
/// and a partial rewrite was not part of the restore.
/// The outer `io::Result` is a failure to record;
/// the inner `Result` is the sync's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncRecording(request: UpdateRequest, edits: ManifestEdit[]): Result<string, string>  // throws if recording fails
/// ```
fn sync_recording(request: &UpdateRequest, edits: &RefCell<Vec<ManifestEdit>>) -> io::Result<Result<String, String>> {
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `let mut`: a binding that may change.
    // Typed binding: `run: Box<dyn FnOnce() -> io::Result<Result<...`.
    // What:     `&dyn Trait` is a borrowed value of any type that implements the trait (a TS interface type); the call goes through a lookup table at run time.
    // Why:      The caller may pass the real implementation or a test fake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function f(registry: Registry) { ... }  // an interface type
    // ```
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    let mut run: Box<dyn FnOnce() -> io::Result<Result<String, String>> + '_> = Box::new(|| { return Ok(request.syncer.sync(request.root)); });
    for crate_dir in request.crate_dirs {
        let inner = run;
        // Closure `|answer|`: an anonymous function, like a TS arrow function.
        // Adapter `.and_then(`: transforms or selects items with a closure.
        // Constructor `Box::new(`: an empty owned collection or a boxed value.
        run = Box::new(move || { return run_recording_manifests(crate_dir, edits, inner).and_then(std::convert::identity); });
    }
    return run();
}

/// What:
/// `fn raised_floor_warning(change: &str) -> String` words one raised floor as a warning line.
/// Why:
/// The user sees which catalog floors the `--breaking` update raised.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const raisedFloorWarning = (change: string) => `Raised the catalog floor ${change}`;
/// ```
fn raised_floor_warning(change: &str) -> String {
    return format!("Raised the catalog floor {change}");
}

/// What:
/// `fn workspace_manifests(request, crate_dir) -> Result<Vec<PathBuf>, UpdateError>` lists the manifests of the Cargo workspace
/// at a crate root,
/// from `cargo metadata --no-deps`.
/// Why:
/// The `--breaking` transaction has to know every file cargo may rewrite before cargo runs;
/// if the list cannot be made,
/// nothing has been mutated yet and the update stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function workspaceManifests(request, crateDir): Promise<string[]>  // throws when metadata fails
/// ```
fn workspace_manifests(request: &UpdateRequest, crate_dir: &str) -> Result<Vec<PathBuf>, UpdateError> {
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `let mut`: a binding that may change.
    // Typed binding: `args: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut args: Vec<String> = vec!["metadata".to_string(), "--no-deps".to_string(), "--format-version".to_string(), "1".to_string()];
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    args.extend(request.forward.iter().cloned());
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&request.root.join`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `outcome: CargoOutcome`.
    let outcome: CargoOutcome = request.runner.run(&request.root.join(crate_dir), &args).map_err(other)?;
    if !outcome.ok {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(UpdateError::Cargo { crate_dir: crate_dir.to_string(), command: format!("cargo {}", args.join(" ")), output: outcome.output });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&outcome.stdout`: lends the value without copying or giving it away.
    return Ok(manifests_from_metadata(&outcome.stdout));
}

/// What:
/// `fn run_update(request, kind, before, produced) -> Result<UpdateReport, UpdateError>` is the transaction body:
/// resolve or run the user's update,
/// set refused versions aside,
/// check what is left,
/// prune and raise catalog entries when kept.
/// `&RefCell<Vec<...>>` is a borrowed list the function may add to.
/// Why:
/// The caller restores on an error or a refusal;
/// `produced` records each lockfile the moment this run's cargo wrote it,
/// so the restore can tell a concurrent writer from this run,
/// and a root this run never wrote is never restored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runUpdate(args): Promise<UpdateReport>
/// ```
fn run_update(request: &UpdateRequest, kind: &UpdateKind, before: &[LockSnapshot], produced: &RefCell<Vec<LockSnapshot>>, edits: &RefCell<Vec<ManifestEdit>>, transaction: &Transaction) -> Result<UpdateReport, UpdateError> {
    // Borrow `&Settings`: lends the value without copying or giving it away.
    // Typed binding: `settings: &Settings`.
    let settings: &Settings = request.settings;
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      With a release age of zero there is no cutoff, and the empty text makes `resolve_crate` run a plain `generate-lockfile`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Typed binding: `cutoff: String`.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    let cutoff: String = if settings.minimum_release_age == 0 { String::new() } else { publish_time_cutoff(request.now, settings.minimum_release_age) };
    // `let mut`: a binding that may change.
    // Typed binding: `fell_back: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut fell_back: Vec<String> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `set_aside_allowed: bool`.
    let mut set_aside_allowed: bool = true;
    // `match`: picks the arm whose pattern fits the value.
    match kind {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&cutoff`: lends the value without copying or giving it away.
        UpdateKind::Full => fell_back = resolve_all(request, &cutoff, produced)?,
        UpdateKind::Targeted { cargo_args } => {
            // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
            // Why:      `args` starts as a copy of the user's cargo arguments so forwarded options can be appended without changing the caller's slice.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // [...slice]
            // ```
            // `let mut`: a binding that may change.
            // Typed binding: `args: Vec<String>`.
            let mut args: Vec<String> = cargo_args.to_vec();
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            args.extend(request.forward.iter().cloned());
            // An explicit `--precise` names the version the user wants: nothing is set aside in its place, and the policy judges
            // that version after the run whether or not the command changed the lockfile (`precise_target.rs`).
            set_aside_allowed = !cargo_args.iter().map(String::as_str).any(is_precise_option);
            // Review finding (`--breaking` edits outside the transaction): cargo's breaking update rewrites the requirements in the
            // manifests, so their text is taken before the run and restored with the lockfiles when the update is not kept.
            let breaking: bool = cargo_args.iter().map(String::as_str).any(is_breaking_option);
            if breaking {
                for crate_dir in request.crate_dirs {
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // Typed binding: `paths: Vec<PathBuf>`.
                    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
                    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // // TS has one type for this: string
                    // ```
                    let paths: Vec<PathBuf> = workspace_manifests(request, crate_dir)?;
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // Borrow `&paths`: lends the value without copying or giving it away.
                    // `.map_err(...)`: converts the callee's error into this function's error type.
                    snapshot_manifests(crate_dir, &paths, edits).map_err(other_text)?;
                }
                // Delegated pick (2026-11-03): the manifests are journaled, so a killed run is recovered for them too.
                transaction.add_manifests(&edits.borrow()).map_err(other_text)?;
            }
            for crate_dir in request.crate_dirs {
                // Both the lockfile and the manifests of this root are attributed by the evidence in `file_owner.rs`, whether the
                // call succeeds or fails; the manifests of other roots are never claimed by this call.
                let (ran, _claim): (io::Result<Result<CargoOutcome, UpdateError>>, Claim) = run_recording(request.root, crate_dir, produced, || {
                    // Borrow `&args`: lends the value without copying or giving it away.
                    return run_recording_manifests(crate_dir, edits, || { return run_or_fail(request, crate_dir, &args); });
                })
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                .map_err(other_text)?;
                // `.map_err(...)`: converts the callee's error into this function's error type.
                ran.map_err(other_text)??;
            }
        }
    }
    // Typed binding: `before_sources: Vec<String>`.
    let before_sources: Vec<String> = sources_of(before);
    // `let mut`: a binding that may change.
    // Typed binding: `warnings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut warnings: Vec<String> = Vec::new();
    if set_aside_allowed {
        // Borrow `&SetAsideRequest`: lends the value without copying or giving it away.
        warnings = set_aside_until_clean(&SetAsideRequest {
            root: request.root,
            crate_dirs: request.crate_dirs,
            // Borrow `&before_sources`: lends the value without copying or giving it away.
            before: &before_sources,
            registry: request.registry,
            settings,
            now: request.now,
            runner: request.runner,
            forward: request.forward,
            produced,
        })
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        .map_err(other)?;
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `after_snapshots: Vec<LockSnapshot>`.
    let after_snapshots: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(other_text)?;
    // Borrow `&after_snapshots`: lends the value without copying or giving it away.
    // Typed binding: `after: Vec<String>`.
    let after: Vec<String> = sources_of(&after_snapshots);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&before_sources`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `added: Vec<LockedVersion>`.
    let added: Vec<LockedVersion> = added_versions(&before_sources, &after).map_err(other)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&added`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `age: CheckOutcome`.
    // Review finding (3c P1-2): the update is judged by the lockfiles it leaves, the way an install judges them, not by the
    // version pairs it added: a refused version that was already locked, or an edge moved to one, is not new as a pair.
    let outcome: CheckOutcome = verify_update(request, &after_snapshots)?;
    // `let mut`: a binding that may change.
    // Typed binding: `problems: Vec<String>`.
    let mut problems: Vec<String> = outcome.problems;
    warnings.extend(outcome.warnings);
    // Typed binding: `keep: bool`.
    let mut keep: bool = problems.is_empty() && !request.dry_run;
    // `let mut`: a binding that may change.
    // Typed binding: `pruned: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut pruned: Vec<String> = Vec::new();
    // Macro `matches!`: builds or prints text (or a list) from a template.
    if keep && request.prune_catalog && matches!(kind, UpdateKind::Full) {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `source: String`.
        let source: String = fs::read_to_string(request.catalog_path).map_err(other_text)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&after`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `locked: Vec<LockedVersion>`.
        let locked: Vec<LockedVersion> = registry_versions(&after).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&source`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let result = prune_excludes(&source, settings, &locked).map_err(other)?;
        if !result.removed.is_empty() {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&result.source`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::write(request.catalog_path, &result.source).map_err(other_text)?;
        }
        pruned = result.removed;
    }
    // `--breaking` is `pnpm update --latest`: the new range goes into the catalog entry and the sync renders it (tighten.rs).
    if keep && let UpdateKind::Targeted { cargo_args } = kind && cargo_args.iter().map(String::as_str).any(is_breaking_option) {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&before_sources`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `before_versions: Vec<LockedVersion>`.
        let before_versions: Vec<LockedVersion> = registry_versions(&before_sources).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&after`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `after_versions: Vec<LockedVersion>`.
        let after_versions: Vec<LockedVersion> = registry_versions(&after).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `source: String`.
        let source: String = fs::read_to_string(request.catalog_path).map_err(other_text)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&source`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let (raised_source, changes) = raise_floors(&source, &after_versions, &raised_names(&before_versions, &after_versions)).map_err(other)?;
        if !changes.is_empty() {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&raised_source`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            let catalog_keep: KeepAs = KeepAs::in_directory(&request.root.join(QUARANTINE_DIR), ".catalog.displaced");
            // Review finding (2c 6): the catalog is replaced only while it still holds the text that was read, never over another
            // writer's edit.
            let raised: ReplaceOutcome = replace_keeping(request.catalog_path, &source, &raised_source, &catalog_keep).map_err(other_text)?;
            // Macro `matches!`: builds or prints text (or a list) from a template.
            if !matches!(raised, ReplaceOutcome::Replaced(_)) {
                // Macro `format!`: builds or prints text (or a list) from a template.
                problems.push(format!("the catalog changed while the update ran, so its floors were not raised and the update was not kept ({raised:?})"));
                keep = false;
            }
            // Closure `|change|`: an anonymous function, like a TS arrow function.
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            // Adapter `.map(`: transforms or selects items with a closure.
            // Macro `format!`: builds or prints text (or a list) from a template.
            warnings.extend(changes.iter().map(String::as_str).map(raised_floor_warning));
            // `match`: picks the arm whose pattern fits the value.
            // Review finding (3c P2-3): an error from recording the manifests is a failed sync like any other, so the catalog goes back.
            let synced: Result<String, String> = if keep {
                // `match`: picks the arm whose pattern fits the value.
                match sync_recording(request, edits) {
                    // `Ok(...)`: builds the success or the failure variant of a `Result`.
                    Ok(answer) => answer,
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    // Macro `format!`: builds or prints text (or a list) from a template.
                    Err(error) => Err(format!("the manifests could not be recorded around the sync ({error})")),
                }
            } else {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                Err("the catalog was not raised".to_string())
            };
            // `match`: picks the arm whose pattern fits the value.
            match synced {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(done) => warnings.push(done),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // Delegated pick (2026-11-03): the sync belongs to the transaction. A catalog raised but not rendered would leave
                // the floors and the manifests disagreeing, so the catalog goes back and the caller restores the manifests and
                // the lockfiles; the finding names the failure.
                Err(message) => {
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // Borrow `&source`: lends the value without copying or giving it away.
                    // `.map_err(...)`: converts the callee's error into this function's error type.
                    // The catalog goes back only while it still holds the raised text this run wrote; an edit made meanwhile is kept.
                    if keep {
                        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                        // Borrow `&raised_source`: lends the value without copying or giving it away.
                        // `.map_err(...)`: converts the callee's error into this function's error type.
                        // Typed binding: `back: ReplaceOutcome`.
                        let back: ReplaceOutcome = replace_keeping(request.catalog_path, &raised_source, &source, &catalog_keep).map_err(other_text)?;
                        // Macro `matches!`: builds or prints text (or a list) from a template.
                        if !matches!(back, ReplaceOutcome::Replaced(_)) {
                            // Macro `format!`: builds or prints text (or a list) from a template.
                            warnings.push(format!("the catalog was edited while the sync ran and keeps that edit ({back:?})"));
                        }
                        // Macro `format!`: builds or prints text (or a list) from a template.
                        problems.push(format!("the catalog floors were not rendered into the manifests, so the update was not kept: {message}"));
                    }
                    keep = false;
                }
            }
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.to_vec()`: copies the slice into a new owned `Vec`.
    return Ok(UpdateReport { crate_dirs: request.crate_dirs.to_vec(), added, problems, warnings, fell_back, written: keep, pruned });
}

/// What:
/// `pub fn update_rust_lockfiles(request: &UpdateRequest, kind: &UpdateKind) -> Result<UpdateReport, UpdateError>` runs the
/// update over the listed crate roots.
/// Why:
/// It first finishes any run that was killed (`journal.rs`),
/// snapshots every lockfile,
/// and puts the snapshots back on a failure,
/// a refusal or a dry run,
/// but only into lockfiles that still hold what this run produced,
/// so a refused version never stays in
/// some crates and not others and a change written by another process is never discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function updateRustLockfiles(args): Promise<UpdateReport>
/// ```
pub fn update_rust_lockfiles(request: &UpdateRequest, kind: &UpdateKind) -> Result<UpdateReport, UpdateError> {
    // Closure `|old, live|`: an anonymous function, like a TS arrow function.
    // Borrow `&process_exists`: lends the value without copying or giving it away.
    // Typed binding: `recovered: Vec<String>`.
    let recovered: Vec<String> = recover(request.root, &|old, live| {
        return policy_findings(request.settings, request.registry, request.now, old, live);
    });
    // Borrow `&recovered`: lends the value without copying or giving it away.
    for message in &recovered {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: {message}");
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `before: Vec<LockSnapshot>`.
    let before: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(other_text)?;
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("cargo-shadow: snapshotted {} lockfiles", before.len());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Review finding (failed begin): without a complete recovery record nothing may be mutated, so a journal that cannot be
    // started stops the update here, before the first cargo call.
    let transaction = begin(request.root, &before, matches!(kind, UpdateKind::Targeted { cargo_args } if cargo_args.iter().map(String::as_str).any(is_breaking_option))).map_err(other_text)?;
    if let Some(notice) = transaction.kept_notice() {
        eprintln!("{notice}");
    }
    // What:     `RefCell<Vec<LockSnapshot>>` is a list that can be added to through a shared reference (sibling: `&mut Vec`, which needs exclusive access); `RefCell::new(Vec::new())` makes an empty one.
    // Why:      `produced` starts empty because cargo has not written yet; each cargo write is recorded the moment it succeeded.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const produced: LockSnapshot[] = [];
    // ```
    let produced: RefCell<Vec<LockSnapshot>> = RefCell::new(Vec::new());
    // The manifests a `--breaking` run may rewrite: empty for every other update.
    let edits: RefCell<Vec<ManifestEdit>> = RefCell::new(Vec::new());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Typed binding: `outcome: Result<UpdateReport, UpdateError>`.
    let outcome: Result<UpdateReport, UpdateError> = run_update(request, kind, &before, &produced, &edits, &transaction);
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
        restore_warnings = restore_if_unchanged(request.root, &before, &produced.borrow()).map_err(other_text)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&edits`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let (manifest_warnings, manifest_failure) = restore_manifests(&edits, &KeepAs::in_directory(&request.root.join(QUARANTINE_DIR), ".manifest.displaced"));
        restore_warnings.extend(manifest_warnings);
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(error) = manifest_failure {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(other_text(error));
        }
        // Borrow `&restore_warnings`: lends the value without copying or giving it away.
        for warning in &restore_warnings {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: warning: {warning}");
        }
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Review finding (4c F1): a finish error never replaces the update's result; it is reported beside it.
    let finish_warning: Option<String> = transaction.finish().err().map(|error| { return format!("the transaction journal could not be completed ({error}); the next run settles it"); });
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&finish_warning`: lends the value without copying or giving it away.
    if let Some(warning) = &finish_warning {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: {warning}");
    }
    // Closure `|mut report|`: an anonymous function, like a TS arrow function.
    // Adapter `.map(`: transforms or selects items with a closure.
    return outcome.map(|mut report| {
        report.warnings.extend(recovered);
        report.warnings.extend(restore_warnings);
        report.warnings.extend(finish_warning);
        return report;
    });
}
