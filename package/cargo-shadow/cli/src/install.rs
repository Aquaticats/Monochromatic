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
/// `use crate::checks::{CheckOutcome, check_policy};` imports `CheckOutcome`,
/// `check_policy` from another module of this crate so this file can name them unqualified.
/// Why:
/// `check_policy` runs the release-age and trust checks together and keeps a refusal when a later lookup fails;
/// `CheckOutcome` carries the findings and warnings of the install and of the frozen verification.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CheckOutcome, checkPolicy } from 'crate/checks';
/// ```
use crate::checks::{CheckOutcome, check_policy};
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
/// `use crate::journal::{begin, finish, run_recording, process_exists, recover, restore_if_unchanged};` imports `begin`,
/// `finish`,
/// `run_recording`,
/// `process_exists`,
/// `recover`,
/// `restore_if_unchanged` from another module of this crate so this file can name them unqualified.
/// Why:
/// `begin` is used by `install_rust_lockfiles`;
/// `finish` is used by `install_rust_lockfiles`;
/// `run_recording` is used by `update_stale_roots`;
/// `process_exists` is used by `install_rust_lockfiles`;
/// `recover` is used by `install_rust_lockfiles`;
/// `restore_if_unchanged` is used by `install_rust_lockfiles`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { begin, finish, run_recording, process_exists, recover, restore_if_unchanged } from 'crate/journal';
/// ```
use crate::journal::{begin, recover, restore_if_unchanged, run_recording};
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
/// Re-exports the error type (it lives in `install_error.rs`).
pub use crate::install_error::InstallError;
/// Imports the helpers that classify a failure into an install error.
use crate::install_error::{invalid_lock_toml, io_error, io_message, registry_error, unapproved_roots};
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
/// `use crate::verify::{is_verified, mark_verified, verify_added_since_base_outcome, verify_locked_entries_outcome};` imports
/// `is_verified`,
/// `mark_verified`,
/// `verify_added_since_base`,
/// `verify_locked_entries` from another module of this crate so this file can name them unqualified.
/// Why:
/// The frozen check and the remembered passes live in `verify.rs`;
/// `install_rust_lockfiles` and `update_stale_roots` use them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isVerified, markVerified, verifyAddedSinceBase, verifyLockedEntries } from 'crate/verify';
/// ```
use crate::verify::{is_verified, mark_verified, verify_added_since_base_outcome, verify_locked_entries_outcome};
/// What:
/// `pub use crate::install_verify::{LockBaseFactory, LockBaseReader, policy_findings, verification_cached, verification_scope};` imports and
/// re-exports the helpers split out of this file,
/// so callers keep writing `crate::install::verification_cached`.
/// Why:
/// The file was split to stay under the line limit;
/// the callers and the tests are unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { LockBaseFactory, LockBaseReader, verificationCached, verificationScope } from './install-verify.ts';
/// ```
pub use crate::install_verify::{LockBaseFactory, LockBaseReader, policy_findings, verification_cached, verification_scope};
/// What:
/// `use crate::install_verify::{base_reader, set_aside_pass};` imports the two helpers that only this crate calls.
/// Why:
/// `update_stale_roots` and `install_rust_lockfiles` read the base revision and set refused releases aside with them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { baseReader, setAsidePass } from 'crate/install_verify';
/// ```
use crate::install_verify::{base_reader, set_aside_pass};
/// Imports `Claim`,
/// what the ownership evidence says about a file a child wrote.
use crate::file_owner::Claim;

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
    /// The key of the base revision:
    /// the commit it resolved to
    /// (so a base that moves is a different base),
    /// or `unresolved:<spelling>` when it names no commit
    /// (`shadow_step::trust_base_setup` supplies it);
    /// part of the key of a remembered pass under `trustLockfile`,
    /// so a pass against one base is never taken for another.
    pub trust_base: &'a str,
    /// The freshness stamp already showed that every lockfile satisfies its manifests,
    /// so the staleness check (a cargo call) is skipped;
    /// the policy checks are not.
    pub known_consistent: bool,
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
pub(crate) fn sources_of(snapshots: &[LockSnapshot]) -> Vec<String> {
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
        // The call is recorded whether it succeeds or fails: a cargo that wrote a lockfile and then exited with an error left that
        // lockfile behind, and the rollback must know it is this run's.
        let (updated, _claim): (Result<(), InstallError>, Claim) = run_recording(request.root, crate_dir, produced, || { return run_or_fail(request, crate_dir, &UPDATE_ARGS); }).map_err(io_error)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        updated?;
        run_or_fail(request, crate_dir, &VERIFY_ARGS)?;
        stale.push(crate_dir.clone());
    }
    // Typed binding: `before_sources: Vec<String>`.
    let before_sources: Vec<String> = sources_of(before);
    // `.clone()`: an independent owned copy.
    // `let mut`: a binding that may change.
    // Typed binding: `baselines: Vec<Vec<String>>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut baselines: Vec<Vec<String>> = vec![before_sources.clone()];
    // `let mut`: a binding that may change.
    // Typed binding: `notes: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut notes: Vec<String> = Vec::new();
    // The versions this update newly locked.
    if !stale.is_empty() {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&before_sources`: lends the value without copying or giving it away.
        notes.extend(set_aside_pass(request, &before_sources, produced)?);
    }
    // The existing entries, unless a remembered pass covers this exact lockfile text, policy and base.
    let scope: String = verification_scope(request.settings, false, request.trust_base);
    // Typed binding: `covered: bool`.
    let covered: bool = verification_cached(request.root, request.crate_dirs, request.settings, false, request.trust_base);
    if !covered {
        // Typed binding: `baseline: Vec<String>`.
        let baseline: Vec<String> = if request.settings.trust_lockfile {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Typed binding: `reader: LockBaseReader`.
            let reader: LockBaseReader = base_reader(request)?;
            // `let mut`: a binding that may change.
            // Typed binding: `texts: Vec<String>`.
            // Constructor `Vec::new()`: an empty owned collection or a boxed value.
            let mut texts: Vec<String> = Vec::new();
            for dir in request.crate_dirs {
                texts.push(reader(dir));
            }
            texts
        } else {
            // Constructor `Vec::new()`: an empty owned collection or a boxed value.
            Vec::new()
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&baseline`: lends the value without copying or giving it away.
        notes.extend(set_aside_pass(request, &baseline, produced)?);
        baselines.push(baseline);
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `after: Vec<LockSnapshot>`.
    let after: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(io_error)?;
    // Borrow `&after`: lends the value without copying or giving it away.
    // Typed binding: `after_sources: Vec<String>`.
    let after_sources: Vec<String> = sources_of(&after);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&before_sources`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `added: Vec<LockedVersion>`.
    let added: Vec<LockedVersion> = added_versions(&before_sources, &after_sources).map_err(invalid_lock_toml)?;
    // Everything either baseline did not hold is checked, once.
    let mut checked: BTreeSet<LockedVersion> = BTreeSet::new();
    // Borrow `&baselines`: lends the value without copying or giving it away.
    for baseline in &baselines {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&after_sources`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        for version in added_versions(baseline, &after_sources).map_err(invalid_lock_toml)? {
            checked.insert(version);
        }
    }
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Typed binding: `versions: Vec<LockedVersion>`.
    let versions: Vec<LockedVersion> = checked.into_iter().collect();
    // Typed binding: `age: CheckOutcome`.
    let outcome: CheckOutcome = if versions.is_empty() { CheckOutcome::default() } else { check_policy(&versions, request.settings, request.registry, request.now)? };
    // Typed binding: `problems: Vec<String>`.
    let problems: Vec<String> = outcome.problems;
    // Only a pass is remembered, and only for the text that is live now.
    // Review finding (3c P2-4): a pass that carried a warning is not remembered, so the next call reports it again.
    if problems.is_empty() && outcome.warnings.is_empty() && !covered {
        // Borrow `&after`: lends the value without copying or giving it away.
        for snapshot in &after {
            // Borrow `&scope`: lends the value without copying or giving it away.
            mark_verified(request.root, snapshot, request.settings, &scope);
        }
    }
    // `let mut`: a binding that may change.
    // Typed binding: `warnings: Vec<String>`.
    let mut warnings: Vec<String> = outcome.warnings;
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
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Typed binding: `stale: Vec<String>`.
            let stale: Vec<String> = stale_roots(request)?;
            if !stale.is_empty() {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!("cargo-shadow: frozen install: {} stale lockfiles, nothing written", stale.len());
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                return Err(InstallError::Stale { stale_dirs: stale });
            }
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `locks: Vec<LockSnapshot>`.
        let locks: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(io_error)?;
        // Typed binding: `outcome: CheckOutcome`.
        let outcome: CheckOutcome = if request.settings.trust_lockfile {
            // Typed binding: `scope: String`.
            let scope: String = verification_scope(request.settings, true, request.trust_base);
            // `let mut`: a binding that may change.
            // Typed binding: `pending: Vec<LockSnapshot>`.
            // Constructor `Vec::new()`: an empty owned collection or a boxed value.
            let mut pending: Vec<LockSnapshot> = Vec::new();
            // Borrow `&locks`: lends the value without copying or giving it away.
            for snapshot in &locks {
                // Borrow `&scope`: lends the value without copying or giving it away.
                if !is_verified(request.root, snapshot, request.settings, &scope) {
                    // `.clone()`: an independent owned copy.
                    pending.push(snapshot.clone());
                }
            }
            if pending.is_empty() {
                // Constructor `Vec::new()`: an empty owned collection or a boxed value.
                CheckOutcome::default()
            } else {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Typed binding: `read_base: LockBaseReader`.
                let read_base: LockBaseReader = base_reader(request)?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Borrow `&pending`: lends the value without copying or giving it away.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                // Typed binding: `found: Vec<String>`.
                let found: CheckOutcome = verify_added_since_base_outcome(&pending, &*read_base, request.settings, request.registry, request.now).map_err(registry_error)?;
                // A pass with a warning is not remembered, so the warning is shown again.
                if found.problems.is_empty() && found.warnings.is_empty() {
                    // Borrow `&pending`: lends the value without copying or giving it away.
                    for snapshot in &pending {
                        // Borrow `&scope`: lends the value without copying or giving it away.
                        mark_verified(request.root, snapshot, request.settings, &scope);
                    }
                }
                found
            }
        } else {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Borrow `&locks`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            verify_locked_entries_outcome(request.root, &locks, request.settings, request.registry, request.now).map_err(registry_error)?
        };
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
        // Why:      The result can be stored or modified independently of the slice it came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // [...slice]
        // ```
        return Ok(InstallReport { crate_dirs: request.crate_dirs.to_vec(), problems: outcome.problems, warnings: outcome.warnings, ..InstallReport::default() });
    }
    // Closure `|old, live|`: an anonymous function, like a TS arrow function.
    // Borrow `&process_exists`: lends the value without copying or giving it away.
    // Typed binding: `recovered: Vec<String>`.
    let recovered: Vec<String> = recover(request.root, &|old, live| { return policy_findings(request.settings, request.registry, request.now, old, live); });
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
    let transaction = begin(request.root, &before, false).map_err(io_error)?;
    if let Some(notice) = transaction.kept_notice() {
        eprintln!("{notice}");
    }
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
        let restored: std::io::Result<Vec<String>> = restore_if_unchanged(request.root, &before, &produced.borrow());
        // Review finding (2c 1): output of this run's cargo that is still live after the rollback is unapproved, and the shim's
        // `--locked` fallback would build it, so the step stops instead of reporting an operational failure.
        let still_live: Vec<String> = unapproved_roots(request.root, &produced.borrow());
        // Review finding (3c P1-1): a policy refusal is a successful report with problems, not an error; a rollback that cannot
        // complete stops the command whether cargo failed or its output was refused, and the refusal travels in the message.
        if !still_live.is_empty() || restored.is_err() {
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&outcome`: lends the value without copying or giving it away.
            // Typed binding: `refusal: String`.
            let refusal: String = match &outcome {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                Ok(report) if !report.problems.is_empty() => format!("\nThe policy refused it: {}", report.problems.join("; ")),
                // Constructor `String::new()`: an empty owned collection or a boxed value.
                _ => String::new(),
            };
            return Err(InstallError::Unsettled {
                message: format!(
                    "cargo-shadow: this run's cargo changed {} and the change could not be put back ({}), so the command was not run{refusal}",
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                    if still_live.is_empty() { "a lockfile".to_string() } else { still_live.join(", ") },
                    match &restored {
                        Ok(warnings) => warnings.join("; "),
                        Err(error) => error.to_string(),
                    }
                ),
            });
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        restore_warnings = restored.map_err(io_error)?;
        // Borrow `&restore_warnings`: lends the value without copying or giving it away.
        for warning in &restore_warnings {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: warning: {warning}");
        }
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Review finding (4c F1): a finish error never replaces an established refusal (or any other result): it is reported beside it,
    // and the journal stays for the next recovery to settle.
    let finish_warning: Option<String> = transaction.finish().err().map(|error| { return format!("the transaction journal could not be completed ({error}); the next run settles it"); });
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&finish_warning`: lends the value without copying or giving it away.
    if let Some(warning) = &finish_warning {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: {warning}");
    }
    // Closure `|mut report|`: an anonymous function, like a TS arrow function.
    // Adapter `.map(`: transforms or selects items with a closure.
    return match outcome {
        Ok(mut report) => {
            report.warnings.extend(recovered);
            report.warnings.extend(restore_warnings);
            report.warnings.extend(finish_warning);
            Ok(report)
        }
        Err(error) => Err(error),
    };
}
