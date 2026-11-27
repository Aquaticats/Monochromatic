//! What:
//! The stale crate roots of an install and the update that repairs them:
//! the three cargo command lines the install step runs and how a cargo option is threaded into them,
//! whether this run is in continuous integration,
//! the read-only check that finds a stale lockfile,
//! the offline update of one root under a recorded call,
//! and the transaction body that holds every newly locked version to the policy.
//! Why:
//! Every cargo call the install makes is one of those three lines,
//! and each of them runs in a crate root whose lockfile may be another process's:
//! so the call is recorded before it can write,
//! and a refused version is set aside before anything is kept.
//! Split out of `install.rs`;
//! the public items are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install_stale_roots.ts: argumentsFor(args, offline), isContinuousIntegration(value), staleRoots(request), updateStaleRoots(request, before, produced)
//! ```

/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a box whose contents can be changed through a shared reference;
/// the check that nothing else holds it happens at run time instead of at compile time.
/// Why:
/// `produced` records each lockfile the moment this run's cargo wrote it,
/// and the recording helpers hold only a shared borrow of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are shared mutable references by default; a RefCell is what Rust needs to get that.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::collections::BTreeSet;` imports the sorted set (siblings:
/// `HashSet`,
/// unordered and faster,
/// and `Vec`,
/// which keeps duplicates).
/// Why:
/// A version that two baselines did not hold is checked once,
/// however many baselines named it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // new Set()
/// ```
use std::collections::BTreeSet;

/// What:
/// `use crate::cargo_run::CargoOutcome;` imports the exit status and output of one real cargo call.
/// Why:
/// A stale check and an offline update both read the exit status of cargo rather than treating a refusal as an error by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoOutcome } from 'crate/cargo_run';
/// ```
use crate::cargo_run::CargoOutcome;
/// What:
/// `use crate::checks::{CheckOutcome, check_policy};` imports the findings of the policy checks and the call that runs them together.
/// Why:
/// The transaction body judges every version no baseline held,
/// and keeps a refusal when a later lookup fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CheckOutcome, check_policy } from 'crate/checks';
/// ```
use crate::checks::{
    CheckOutcome,
    check_policy,
};
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports the snapshot of one lockfile and the call that reads one per crate root.
/// Why:
/// The lockfiles the update leaves are compared with the ones taken before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, read_locks } from 'crate/crates';
/// ```
use crate::crates::{
    LockSnapshot,
    read_locks,
};
/// What:
/// `use crate::file_owner::Claim;` imports what the ownership evidence says about the files a child may have written.
/// Why:
/// A recorded cargo call answers a claim,
/// and the body keeps it only to know the call was recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Claim } from 'crate/file_owner';
/// ```
use crate::file_owner::Claim;
/// What:
/// `use crate::install::{InstallReport, InstallRequest};` imports what an install is asked and what it answers.
/// Why:
/// The body takes the request and builds the report the entry point returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InstallReport, InstallRequest } from 'crate/install';
/// ```
use crate::install::{
    InstallReport,
    InstallRequest,
};
/// What:
/// `use crate::install_error::{InstallError, invalid_lock_toml, io_error, io_message};` imports the failure of an install and the classifications this file reports.
/// Why:
/// A cargo call that cannot start or their recorded work that cannot be read is an I/O failure,
/// and a lockfile that cannot be parsed is named as such.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InstallError, invalid_lock_toml, io_error } from 'crate/install_error';
/// ```
use crate::install_error::{
    InstallError,
    invalid_lock_toml,
    io_error,
    io_message,
};
/// What:
/// `use crate::install_verify::{LockBaseReader, base_reader, set_aside_pass, verification_cached, verification_scope};` imports the reader of the
/// lockfile text at the trust base,
/// the pass that sets refused versions aside,
/// and the two halves of the remembered verification.
/// Why:
/// The body checks what is new since the base under `trustLockfile`,
/// and remembers a pass so the next call costs no lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockBaseReader, base_reader, set_aside_pass, verification_cached, verification_scope } from 'crate/install_verify';
/// ```
use crate::install_verify::{
    LockBaseReader,
    base_reader,
    set_aside_pass,
    verification_cached,
    verification_scope,
};
/// What:
/// `use crate::journal::run_recording;` imports the runner that records what a cargo call wrote.
/// Why:
/// The offline update of a root is recorded,
/// so a lockfile it wrote and then failed on is known to be this run's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { run_recording } from 'crate/journal';
/// ```
use crate::journal::run_recording;
/// What:
/// `use crate::lockfile::{LockedVersion, added_versions};` imports the record of one locked registry crate and the difference between two sets of texts.
/// Why:
/// Everything a baseline did not hold is what the policy judges and what the report lists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion, added_versions } from 'crate/lockfile';
/// ```
use crate::lockfile::{
    LockedVersion,
    added_versions,
};
/// What:
/// `use crate::verify::mark_verified;` imports the record of a passing verification.
/// Why:
/// Only a pass,
/// with no problem and no warning,
/// is remembered,
/// and only for the lockfile text that is live now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mark_verified } from 'crate/verify';
/// ```
use crate::verify::mark_verified;
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
pub fn arguments_for(
    args: &[&str],
    offline: bool,
) -> Vec<String> {
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
    let mut kept: Vec<String> = Vec::new();
    for argument in args.iter() {
        if offline || *argument != "--offline" {
            kept.push(argument.to_string());
        }
    }
    return kept;
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

/// Runs cargo in a crate root;
/// a start failure is an `Io` error.
fn run_cargo(
    request: &InstallRequest,
    crate_dir: &str,
    args: &[&str],
) -> Result<CargoOutcome, InstallError> {
    return request
        .runner
        // Borrow `&request.root.join`: lends the value without copying or giving it away.
        .run(
            &request.root.join(crate_dir),
            &arguments_for(args, request.offline),
        )
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
pub fn is_stale(
    request: &InstallRequest,
    crate_dir: &str,
) -> Result<bool, InstallError> {
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

/// What:
/// `struct UpdateOffline<'a> { request, crate_dir }` is the work of updating one crate root's workspace offline.
/// Why:
/// `run_recording` takes the work as a value,
/// and this holds what the closure that stood here captured.
struct UpdateOffline<'a> {
    /// The install request that names the cargo runner.
    request: &'a InstallRequest<'a>,
    /// The crate root the update runs in.
    crate_dir: &'a str,
}

/// What:
/// The implementation of `RecordedWork` for `UpdateOffline`.
/// Why:
/// `run_recording` runs it between its before and after observations.
impl crate::file_owner::RecordedWork<Result<(), InstallError>> for UpdateOffline<'_> {
    /// What:
    /// Runs the offline update and fails with the crate named when cargo exits non-zero.
    /// Why:
    /// The result is the one the closure returned.
    fn run(self) -> Result<(), InstallError> {
        return run_or_fail(self.request, self.crate_dir, &UPDATE_ARGS);
    }
}

/// Runs one cargo command in a crate root and fails with the crate named when it exits non-zero.
fn run_or_fail(
    request: &InstallRequest,
    crate_dir: &str,
    args: &[&str],
) -> Result<(), InstallError> {
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
/// `pub(crate) fn update_stale_roots(request, before, produced) -> Result<InstallReport, InstallError>` is the transaction body:
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
pub(crate) fn update_stale_roots(
    request: &InstallRequest,
    before: &[LockSnapshot],
    produced: &RefCell<Vec<LockSnapshot>>,
) -> Result<InstallReport, InstallError> {
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
        let (updated, _claim): (Result<(), InstallError>, Claim) = run_recording(
            request.root,
            crate_dir,
            produced,
            UpdateOffline {
                request,
                crate_dir,
            },
        )
        .map_err(io_error)?;
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
    let covered: bool = verification_cached(
        request.root,
        request.crate_dirs,
        request.settings,
        false,
        request.trust_base,
    );
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
    let after: Vec<LockSnapshot> =
        read_locks(request.root, request.crate_dirs).map_err(io_error)?;
    // Borrow `&after`: lends the value without copying or giving it away.
    // Typed binding: `after_sources: Vec<String>`.
    let after_sources: Vec<String> = sources_of(&after);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&before_sources`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `added: Vec<LockedVersion>`.
    let added: Vec<LockedVersion> =
        added_versions(&before_sources, &after_sources).map_err(invalid_lock_toml)?;
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
    let outcome: CheckOutcome = if versions.is_empty() {
        CheckOutcome::default()
    } else {
        check_policy(&versions, request.settings, request.registry, request.now)?
    };
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
